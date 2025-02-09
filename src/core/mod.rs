use tracing::{info, warn, error};
use tokio::signal;

impl Indexer {
    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
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
            }
            result = try_join_all(handles) => {
                result??;
            }
        }

        Ok(())
    }
}