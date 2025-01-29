use crate::config::Config;
use crate::models::Block;
use crate::utils::error::IndexerError;
use anyhow::Result;
use crossbeam::channel;
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use tracing::{info, warn};
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

    async fn fetch_block(&self, block_number: u64) -> Result<Block> {
        let block = self.web3_client
            .eth()
            .block_with_txs(BlockId::Number(BlockNumber::Number(block_number.into())))
            .await
            .map_err(|e| IndexerError::RpcError(e.to_string()))?
            .ok_or_else(|| IndexerError::RpcError("Block not found".into()))?;

        let transactions = block.transactions.into_iter()
            .map(|tx| crate::models::Transaction {
                hash: format!("{:?}", tx.hash),
                from: format!("{:?}", tx.from),
                to: tx.to.map(|addr| format!("{:?}", addr)),
                value: tx.value.to_string(),
            })
            .collect();

        Ok(Block {
            number: block.number.unwrap().as_u64(),
            hash: format!("{:?}", block.hash.unwrap()),
            transactions,
            timestamp: block.timestamp.as_u64(),
        })
    }

    pub async fn process_blocks(&self, start_block: Option<u64>) -> Result<()> {
        let mut current_block = match start_block {
            Some(block) => block,
            None => self.get_latest_block_number().await?,
        };

        self.latest_block.store(current_block, Ordering::SeqCst);
        
        loop {
            let latest_block = self.get_latest_block_number().await?;
            
            while current_block <= latest_block {
                match self.fetch_block(current_block).await {
                    Ok(block) => {
                        if let Err(e) = self.blocks_channel.0.send(block) {
                            warn!("Failed to send block through channel: {}", e);
                            continue;
                        }
                        
                        info!("Processed block {}", current_block);
                        self.latest_block.store(current_block, Ordering::SeqCst);
                        current_block += 1;
                    }
                    Err(e) => {
                        warn!("Failed to fetch block {}: {}", current_block, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }

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