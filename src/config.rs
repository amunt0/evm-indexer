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
    pub fn load() -> Result<Self> {
        let mut builder = ConfigSource::builder()
            .set_default("blocks_in_memory", 1000)?
            .set_default("rotation_blocks", 10000)?
            .set_default("metrics_port", 9090)?
            .set_default("data_dir", "./data")?;

        if let Ok(config_path) = std::env::var("CONFIG_PATH") {
            builder = builder.add_source(File::with_name(&config_path));
        }

        builder = builder.add_source(
            Environment::with_prefix("INDEXER")
                .separator("_")
                .try_parsing(true)
        );

        let config = builder.build()?;
        Ok(config.try_deserialize()?)
    }
}

