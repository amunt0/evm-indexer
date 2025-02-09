use serde::Deserialize;
use std::path::PathBuf;
use anyhow::Result;
use config::{Config as ConfigSource, File, Environment};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rpc_endpoint: String,
    pub blocks_in_memory: usize,
    pub rotation_blocks: usize,
    pub data_dir: PathBuf,
    pub metrics_port: u16,
    pub start_block: Option<u64>,
}

impl Config {
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
                .unwrap_or_else(|_| "./data".to_string())),
        })
    }
}

