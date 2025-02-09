use crate::config::Config;
use crate::models::Block;
use crate::utils::error::IndexerError;
use anyhow::Result;
use crossbeam::channel;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use tracing::{info, error};
use web3::{
    types::{BlockNumber, BlockId},
    Web3,
};

#[derive(Clone)]
pub struct BlockProcessor {
    web3_client: Web3<web3::transports::Http>,
    latest_block: Arc<AtomicU64>,
    buffer_size: usize,
    blocks_channel: (channel::Sender<Block>, channel::Receiver<Block>),
}

impl BlockProcessor {
    pub async fn new(config: &Config) -> Result<Self> {
        let transport = web3::transports::Http::new(&config.rpc_endpoint)?;
        let web3_client = Web3::new(transport);
        
        let blocks_channel = channel::bounded(config.blocks_in_memory);
        
        Ok(Self {
            web3_client,
            latest_block: Arc::new(AtomicU64::new(0)),
            buffer_size: config.blocks_in_memory,
            blocks_channel,
        })
    }
    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let block_number = self.web3_client
            .eth()
            .block_number()
            .await
            .map_err(|e| IndexerError::RpcError(e.to_string()))?;
        
        Ok(block_number.as_u64())
    }

    pub async fn process_blocks(&self, start_block: Option<u64>) -> Result<()> {
        info!(
            event = "block_processing_started",
            message = "Starting block processing",
            start_block = ?start_block,
        );

        let mut current_block = match start_block {
            Some(block) => {
                info!(
                    event = "using_start_block",
                    message = "Using provided start block",
                    block = block
                );
                block
            },
            None => {
                let latest = self.get_latest_block_number().await?;
                info!(
                    event = "using_latest_block",
                    message = "Using latest block as start",
                    block = latest
                );
                latest
            },
        };

        self.latest_block.store(current_block, Ordering::SeqCst);
        
        info!(
            event = "processing_loop_started",
            message = "Entering main processing loop",
            current_block = current_block
        );

        loop {
            let start_time = std::time::Instant::now();
            let latest_block = match self.get_latest_block_number().await {
                Ok(block) => block,
                Err(e) => {
                    error!(
                        event = "latest_block_error",
                        message = "Failed to get latest block number",
                        error = %e
                    );
                    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    continue;
                }
            };
            
            info!(
                event = "sync_status",
                message = "Block sync status",
                current_block = current_block,
                latest_block = latest_block,
                blocks_behind = latest_block.saturating_sub(current_block)
            );

            metrics.record_sync_status(current_block, latest_block);

            while current_block <= latest_block {
                match self.fetch_block(current_block).await {
                    Ok(block) => {
                        match self.blocks_channel.0.send(block.clone()) {
                            Ok(_) => {
                                metrics.record_block(&block);
                                metrics.record_processing_time(start_time);
                                
                                info!(
                                    event = "block_processed",
                                    message = "Successfully processed block",
                                    block_number = current_block,
                                    tx_count = block.transactions.len(),
                                );
                                self.latest_block.store(current_block, Ordering::SeqCst);
                                current_block += 1;
                            },
                            Err(e) => {
                                error!(
                                    event = "channel_send_error",
                                    message = "Failed to send block through channel",
                                    error = %e,
                                    block_number = current_block
                                );
                                continue;
                            }
                        }
                    }
                    Err(e) => {
                        error!(
                            event = "block_fetch_error",
                            message = "Failed to fetch block",
                            error = %e,
                            block_number = current_block
                        );
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }

            info!(
                event = "sync_complete",
                message = "Caught up to latest block, waiting for new blocks",
                current_block = current_block
            );
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }

    pub fn get_blocks_receiver(&self) -> channel::Receiver<Block> {
        self.blocks_channel.1.clone()
    }

    pub fn get_latest_processed_block(&self) -> u64 {
        self.latest_block.load(Ordering::SeqCst)
    }
}