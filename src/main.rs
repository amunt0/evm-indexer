use anyhow::Result;
use mimalloc::MiMalloc;
use tracing::{info, warn, error, Level};
use tracing_subscriber::fmt::time::ChronoUtc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

mod config;
mod core;
mod models;
mod utils;

use crate::core::Indexer;

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
        .with_writer(std::io::stdout) // Explicitly write to stdout
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(Level::INFO.into()))
        .init();

    // Log startup event with more details
    info!(
        event = "startup",
        message = "Starting eth-indexer",
        version = env!("CARGO_PKG_VERSION"),
        config_path = std::env::var("CONFIG_PATH").unwrap_or_default(),
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging first thing
    init_logging()?;
    
    // Log config loading attempt
    info!(event = "config_loading", message = "Loading configuration");
    
    // Load configuration
    let config = match config::Config::load() {
        Ok(config) => {
            info!(
                event = "config_loaded",
                message = "Configuration loaded successfully",
                rpc_endpoint = config.rpc_endpoint,
                metrics_port = config.metrics_port,
            );
            config
        },
        Err(e) => {
            error!(
                event = "config_error",
                message = "Failed to load configuration",
                error = %e,
            );
            return Err(e);
        }
    };
    
    // Create indexer
    info!(event = "indexer_init", message = "Initializing indexer");
    let mut indexer = match Indexer::new(config).await {
        Ok(indexer) => {
            info!(event = "indexer_ready", message = "Indexer initialized successfully");
            indexer
        },
        Err(e) => {
            error!(
                event = "indexer_init_error",
                message = "Failed to initialize indexer",
                error = %e,
            );
            return Err(e);
        }
    };

    // Run indexer (this should be an infinite loop inside run())
    info!(event = "indexer_starting", message = "Starting indexer main loop");
    match indexer.run().await {
        Ok(_) => {
            warn!(event = "indexer_exit", message = "Indexer main loop exited unexpectedly");
            Ok(())
        },
        Err(e) => {
            error!(
                event = "indexer_error",
                message = "Indexer encountered an error",
                error = %e,
            );
            Err(e)
        }
    }
}