//! OTLP traces to `localhost:12005` via `#[instrument]` + batch export.
use pulse::tracing::instrument;
use pulse::{Environment, logger};

#[instrument]
fn simple_operation() {
    logger::info!("traced operation");
    std::thread::sleep(std::time::Duration::from_millis(50));
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _pulse = pulse::pulse_local_otel!()
        .with_service("simple-trace-test", "1.0.0")
        .environment(Environment::Development)
        .with_tracing()
        .build()?;

    logger::info!("Tracing → OTLP localhost:12005");

    simple_operation();

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    Ok(())
}
