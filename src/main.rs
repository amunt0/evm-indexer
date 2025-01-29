use anyhow::Result;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod core;
mod models;
mod utils;

use crate::core::Indexer;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging()?;
    
    // Load configuration
    let config = config::Config::load()?;
    
    // Create and run indexer
    let mut indexer = Indexer::new(config).await?;
    indexer.run().await?;
    
    Ok(())
}

fn init_logging() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(tracing::Level::INFO)
        .init();
    Ok(())
}