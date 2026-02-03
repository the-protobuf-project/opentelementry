// Simple tracing example to verify OTLP export
use pulse::{Pulse, Environment, logger};
use pulse::tracing::instrument;

#[instrument]
fn simple_operation() {
    tracing::info!("This is a simple traced operation");
    std::thread::sleep(std::time::Duration::from_millis(100));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Auto-discovers pulse.toml config file
    let _pulse = Pulse::new()
        .with_service("simple-trace-test", "1.0.0")
        .environment(Environment::Development)
        .with_tracing()
        .build()?;

    logger::info!("=== SIMPLE TRACE TEST ===");
    logger::info!("Service: simple-trace-test");
    logger::info!("Sending ONE span to OTLP...");

    // Create a single simple span
    simple_operation();

    logger::info!("Span created. Waiting 5 seconds for export...");
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

    logger::info!("Done! Check Tempo for service: simple-trace-test");
    logger::info!("Look for span: simple_operation");

    Ok(())
}
