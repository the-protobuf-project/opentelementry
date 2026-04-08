//! OTLP metrics to `localhost:12005`.
use pulse::derive::Metrics;
use pulse::{Environment, logger};
use std::path::PathBuf;

#[derive(Debug, Metrics)]
pub struct LlmMetrics {
    #[metric(
        name = "llm.requests.total",
        description = "Total number of LLM requests",
        counter
    )]
    pub request_count: u64,

    #[metric(
        name = "llm.response.latency_ms",
        description = "LLM response latency in milliseconds",
        histogram
    )]
    pub latency_ms: f64,

    #[metric(
        name = "llm.cache.hit_rate",
        description = "LLM cache hit rate percentage",
        gauge
    )]
    pub cache_hit_rate: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mcap_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/metrics-example.mcap");
    let mut pulse = pulse::pulse_local_otel!()
        .with_service("metrics-example", "1.0.0")
        .environment(Environment::Development)
        .with_mcap(mcap_path.to_string_lossy())
        .build()?;

    logger::info!("Metrics → OTLP gRPC localhost:12005 (+ MCAP)");

    let llm_metrics = LlmMetrics {
        request_count: 42,
        latency_ms: 123.5,
        cache_hit_rate: 0.85,
    };

    pulse.metrics.record(&llm_metrics)?;

    for iteration in 0..5 {
        for i in 0..10 {
            pulse.metrics.counter("api.requests", 1.0)?;
            pulse
                .metrics
                .histogram("api.latency_ms", (i as f64) * 10.0 + 50.0)?;
            pulse
                .metrics
                .gauge("api.active_connections", (10 - i) as f64)?;
            pulse.metrics.record(&llm_metrics)?;
        }
        logger::debug!("metrics batch {iteration}");
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    pulse.flush()?;
    Ok(())
}
