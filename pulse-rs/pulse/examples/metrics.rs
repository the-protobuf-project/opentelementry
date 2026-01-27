use pulse::logger;
use pulse::options::{ServiceOptions, PulseOptions, Environment, FoxgloveOptions, TelemetryOptions, OtelOptions};
use pulse::derive::Metrics;

#[derive(Debug, Metrics)]
pub struct LlmMetrics {
    #[metric(name = "llm.requests.total", description = "Total number of LLM requests", counter)]
    pub request_count: u64,

    #[metric(name = "llm.response.latency_ms", description = "LLM response latency in milliseconds", histogram)]
    pub latency_ms: f64,

    #[metric(name = "llm.cache.hit_rate", description = "LLM cache hit rate percentage", gauge)]
    pub cache_hit_rate: f64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service_opts = ServiceOptions::new("metrics-example", "1.0.0")
        .with_description("Metrics example service")
        .with_environment(Environment::Development);

    let pulse_opts = PulseOptions::new()
        .with_foxglove(FoxgloveOptions::new("pulse/examples/metrics.mcap"))
        .with_telemetry(
            TelemetryOptions::default()
                .with_otlp(OtelOptions::new("localhost", 4317))
        );

    let mut pulse = pulse::Pulse::new(service_opts.clone(), pulse_opts)?;

    logger::info!("Metrics Example Started");
    logger::info!("Sending metrics to OTEL collector at localhost:4317");

    // Record metrics using the macro-generated struct
    let llm_metrics = LlmMetrics {
        request_count: 42,
        latency_ms: 123.5,
        cache_hit_rate: 0.85,
    };

    pulse.metrics.record(&llm_metrics)?;
    logger::info!("Recorded LLM metrics from struct");

    // Or record metrics directly
    logger::info!("Recording metrics for 30 seconds...");
    
    for _iteration in 0..30 {
        for i in 0..10 {
            pulse.metrics.counter("api.requests", 1.0)?;
            pulse.metrics.histogram("api.latency_ms", (i as f64) * 10.0 + 50.0)?;
            pulse.metrics.gauge("api.active_connections", (10 - i) as f64)?;

            // Re-record LLM metrics every iteration to keep them visible
            pulse.metrics.record(&llm_metrics)?;

            logger::debug!("Recorded API metrics");
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    }

    logger::info!("Metrics recording completed");
    logger::info!("Closing MCAP file...");
    
    // Properly close to finalize MCAP file
    pulse.close()?;
    
    logger::info!("MCAP file finalized: examples/metrics.mcap");
    Ok(())
}
