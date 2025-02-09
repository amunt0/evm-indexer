use anyhow::Result;
use mimalloc::MiMalloc;
use tracing::{info, Level};
use tracing_subscriber::fmt::time::ChronoUtc;

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
    // Initialize JSON formatted logging
    tracing_subscriber::fmt()
        .json()
        .with_timer(ChronoUtc::rfc3339())
        .with_current_span(true)
        .with_span_list(true)
        .with_target(true)
        .with_file(true)
        .with_line_number(true)
        .with_level(true)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    // Log startup event
    info!(
        event = "startup",
        message = "Starting eth-indexer",
        version = env!("CARGO_PKG_VERSION"),
    );

    Ok(())
}

info!(
    event = "block_processed",
    message = "Successfully processed block",
    block_number = block.number,
    tx_count = block.transactions.len(),
);

error!(
    event = "processing_error",
    message = "Failed to process block",
    error = %e,
    block_number = current_block,
);
