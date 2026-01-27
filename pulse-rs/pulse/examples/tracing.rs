// Simple tracing example to verify OTLP export
use pulse::logger;
use pulse::options::{Environment, OtelOptions, PulseOptions, ServiceOptions, TelemetryOptions};
use pulse::tracing::instrument;

#[instrument]
fn simple_operation() {
    tracing::info!("This is a simple traced operation");
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service_opts = ServiceOptions::new("simple-trace-test", "1.0.0")
        .with_environment(Environment::Development);

    let pulse_opts = PulseOptions::new()
        .with_telemetry(TelemetryOptions::default().with_otlp(OtelOptions::new("localhost", 4317)));

    let pulse = pulse::Pulse::new(service_opts.clone(), pulse_opts)?;

    logger::info!("=== SIMPLE TRACE TEST ===");
    logger::info!("Service: simple-trace-test");
    logger::info!("Sending ONE span to OTLP...");

    // Create a single simple span
    simple_operation();

    logger::info!("Span created. Waiting 5 seconds for export...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    logger::info!("Done! Check Tempo for service: simple-trace-test");
    logger::info!("Look for span: simple_operation");

    pulse.close()?;
    Ok(())
}
