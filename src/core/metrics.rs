use anyhow::Result;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder};
use crate::models::Block;
use std::time::Instant;
use tracing::info;

#[derive(Clone)]
pub struct MetricsCollector {
    port: u16,
}

impl MetricsCollector {
    pub fn new(port: u16) -> Result<Self> {
        info!(
            event = "metrics_init",
            message = "Initializing metrics collector",
            port = port
        );

        let builder = PrometheusBuilder::new()
            .with_http_listener(([0, 0, 0, 0], port))
            .set_buckets(&[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])?;

        // Install global recorder
        builder.install()?;

        info!(
            event = "metrics_ready",
            message = "Metrics endpoint ready",
            endpoint = format!("http://0.0.0.0:{}/metrics", port)
        );

        Ok(Self { port })
    }

    pub fn record_block(&self, block: &Block) {
        counter!("blocks_processed_total").increment(1);
        counter!("transactions_processed_total").increment(block.transactions.len() as u64);
        gauge!("latest_block_number").set(block.number as f64);
        gauge!("latest_block_timestamp").set(block.timestamp as f64);
        gauge!("block_transaction_count").set(block.transactions.len() as f64);
    }

    pub fn record_processing_time(&self, start_time: Instant) {
        let duration = start_time.elapsed();
        histogram!("block_processing_time_seconds").record(duration.as_secs_f64());
    }

    pub fn record_sync_status(&self, current_block: u64, latest_block: u64) {
        gauge!("current_processing_block").set(current_block as f64);
        gauge!("chain_latest_block").set(latest_block as f64);
        gauge!("blocks_behind").set((latest_block.saturating_sub(current_block)) as f64);
    }
}