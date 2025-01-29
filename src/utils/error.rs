use thiserror::Error;

#[derive(Error, Debug)]
pub enum IndexerError {
    #[error("RPC error: {0}")]
    RpcError(String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}