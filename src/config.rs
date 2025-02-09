use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub rpc_endpoint: String,
    pub blocks_in_memory: usize,
    pub metrics_port: u16,
    pub data_dir: PathBuf,
    pub rotation_blocks: u64,
    pub start_block: Option<u64>,
}

impl Config {
    pub fn load() -> Result<Self> {
        Self::from_env()
    }

    pub fn from_env() -> Result<Self> {
        Ok(Self {
            rpc_endpoint: std::env::var("RPC_ENDPOINT")
                .unwrap_or_else(|_| "https://rpc.sepolia.org".to_string()),
            blocks_in_memory: std::env::var("BLOCKS_IN_MEMORY")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(1000),
            metrics_port: std::env::var("METRICS_PORT")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(9090),
            data_dir: PathBuf::from(std::env::var("DATA_DIR")
                .unwrap_or_else(|_| "/data/eth-indexer".to_string())),
            rotation_blocks: std::env::var("ROTATION_BLOCKS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10000),
            start_block: std::env::var("START_BLOCK")
                .ok()
                .and_then(|v| {
                    // Try parsing as regular integer first
                    v.parse::<u64>().ok().or_else(|| {
                        // If that fails, try parsing as float and convert to integer
                        v.parse::<f64>().ok().map(|f| f as u64)
                    })
                }),
        })
    }
}