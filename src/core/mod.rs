mod block_processor;
mod metrics;
mod storage;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::Mutex;
use crate::config::Config;
use futures::future::try_join_all;
use tracing::{info, error};

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
        let block_processor = Arc::new(BlockProcessor::new(&config, metrics_collector.clone()).await?);

        Ok(Self {
            block_processor,
            storage_manager,
            metrics_collector,
            config,
        })
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
        
        for result in try_join_all(handles).await? {
            result?;
        }

        Ok(())
    }
}