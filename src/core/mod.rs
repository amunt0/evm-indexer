mod block_processor;
mod metrics;
mod storage;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use futures::future::try_join_all;
use tracing::{info, error};
use crate::config::Config;

pub use block_processor::BlockProcessor;
pub use metrics::MetricsCollector;
pub use storage::StorageManager;

pub struct Indexer {
    block_processor: Arc<BlockProcessor>,
    storage_manager: Arc<Mutex<StorageManager>>,
    metrics_collector: MetricsCollector,
    config: Config,
}

impl Indexer {
    pub async fn new(config: Config) -> Result<Self> {
        let metrics_collector = MetricsCollector::new(config.metrics_port)?;
        let storage_manager = Arc::new(Mutex::new(StorageManager::new(&config)?));
        let block_processor = Arc::new(BlockProcessor::new(&config).await?);

        Ok(Self {
            block_processor,
            storage_manager,
            metrics_collector,
            config,
        })
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            tokio::signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        }
    }

    pub async fn run(&self) -> Result<()> {
        let block_receiver = self.block_processor.get_blocks_receiver();
        let config_start_block = self.config.start_block;
        let processor = self.block_processor.clone();
        let storage = self.storage_manager.clone();
        let metrics = self.metrics_collector.clone();

        let process_handle = tokio::spawn(async move {
            processor.process_blocks(config_start_block).await
        });

        let storage_handle = tokio::spawn(async move {
            while let Ok(block) = block_receiver.recv() {
                metrics.record_block(&block);
                let mut storage = storage.lock().await;
                if let Err(e) = storage.store_block(block).await {
                    error!("Failed to store block: {}", e);
                    break;
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        let handles = vec![process_handle, storage_handle];
        
        tokio::select! {
            _ = Self::shutdown_signal() => {
                info!(event = "shutdown", message = "Received shutdown signal");
                Ok(())
            }
            result = try_join_all(handles) => {
                // Properly handle the joined results
                for task_result in result? {
                    task_result?;
                }
                Ok(())
            }
        }
    }
}