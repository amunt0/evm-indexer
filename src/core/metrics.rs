use anyhow::Result;
use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::PrometheusBuilder;
use crate::models::Block;
use std::time::Instant;

#[derive(Clone)]
pub struct MetricsCollector {
    port: u16,
}

impl MetricsCollector {
    pub fn new(port: u16) -> Result<Self> {
        let builder = PrometheusBuilder::new()
            .with_http_listener(([0, 0, 0, 0], port));

        builder.install()?;

        Ok(Self { port })
    }

    pub fn record_block(&self, block: &Block) {
        counter!("blocks_processed_total").increment(1);
        counter!("transactions_processed_total").increment(block.transactions.len() as u64);
        gauge!("latest_block_number").set(block.number as f64);
    }

    pub fn record_processing_time(&self, start_time: Instant) {
        let duration = start_time.elapsed();
        histogram!("block_processing_time_seconds").record(duration.as_secs_f64());
    }
}