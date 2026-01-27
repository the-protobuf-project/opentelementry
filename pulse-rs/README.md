# Pulse - Rust SDK

A comprehensive observability framework for Rust applications, providing unified logging, metrics, and distributed tracing capabilities with OpenTelemetry integration and MCAP recording for Foxglove Studio.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
  - [Logging](#logging-with-structured-data)
  - [Metrics](#metrics-with-derive-macro)
  - [Distributed Tracing](#distributed-tracing)
  - [MCAP Recording](#mcap-recording)
- [Configuration](#configuration)
- [Examples](#examples)
- [API Reference](#api-reference)
- [Best Practices](#best-practices)

## Features

- **Structured Logging** - Colored console output with MCAP and OpenTelemetry backends
- **Metrics Collection** - Counters, histograms, and gauges with derive macro support
- **Distributed Tracing** - OpenTelemetry-based tracing with automatic instrumentation
- **MCAP Recording** - Record all telemetry to MCAP files for Foxglove Studio
- **Beautiful Console Output** - Colored, formatted logs with service context
- **Zero-Config** - Sensible defaults with environment variable overrides
- **Async-First** - Built on Tokio for high-performance async applications

## Installation

### From Git Repository

Add Pulse to your Rust project using Git:

```toml
[dependencies]
pulse = { git = "https://github.com/machanirobotics/pulse.git", subdirectory = "pulse-rs/pulse" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
anyhow = "1.0"
```

Or clone and build locally:

```bash
git clone https://github.com/machanirobotics/pulse.git
cd pulse/pulse-rs
cargo build --release
```

**Requirements:**

- Rust 1.91.0 or higher (specified in `rust-toolchain.toml`)
- OpenTelemetry Collector (optional, for production deployments)
- Foxglove Studio (optional, for MCAP visualization)

## Quick Start

### Basic Usage

```rust
use pulse::{Pulse, Environment, logger};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize Pulse with builder pattern
    let pulse = Pulse::builder("my-service", "1.0.0")
        .environment(Environment::Production)
        .build()?;
    
    // Use structured logging
    logger::info!("Service started");
    logger::debug!("Debug information");
    logger::warn!("Warning message");
    logger::error!("Error occurred");
    
    // Clean shutdown
    pulse.close()?;
    Ok(())
}
```

### Logging with Structured Data

```rust
use pulse::logger;
use serde::Serialize;

#[derive(Debug, Serialize)]
struct UserEvent {
    user_id: String,
    action: String,
    timestamp: i64,
}

let event = UserEvent {
    user_id: "user-123".to_string(),
    action: "login".to_string(),
    timestamp: chrono::Utc::now().timestamp(),
};

// Log with structured data
logger::info!("User action recorded").with_data(&event);

// Format specifiers
logger::info!("User {} performed action: {}", event.user_id, event.action);
```

### Metrics with Derive Macro

```rust
use pulse::derive::Metrics;

#[derive(Debug, Metrics)]
pub struct ApiMetrics {
    #[metric(name = "api.requests.total", description = "Total API requests", counter)]
    pub request_count: u64,

    #[metric(name = "api.latency_ms", description = "API latency in milliseconds", histogram)]
    pub latency_ms: f64,

    #[metric(name = "api.active_connections", description = "Active connections", gauge)]
    pub active_connections: f64,
}

// Record metrics from struct
let metrics = ApiMetrics {
    request_count: 100,
    latency_ms: 45.2,
    active_connections: 12.0,
};

pulse.metrics.record(&metrics)?;
```

### Direct Metrics Recording

```rust
// Record individual metrics
pulse.metrics.counter("requests_total", 1.0)?;
pulse.metrics.histogram("response_time_ms", 123.5)?;
pulse.metrics.gauge("memory_usage_mb", 256.0)?;
```

### Distributed Tracing

```rust
use pulse::tracing::instrument;

#[instrument]
async fn process_request(request_id: String) -> Result<String> {
    tracing::info!("Processing request");
    
    // Your business logic here
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok("Success".to_string())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pulse = Pulse::builder("my-service", "1.0.0")
        .with_otlp("localhost", 4317)
        .build()?;
    
    // Traced function calls
    process_request("req-123".to_string()).await?;
    
    pulse.close()?;
    Ok(())
}
```

## Configuration

### Builder API (Recommended)

The builder pattern provides a clean, fluent API for configuring Pulse:

```rust
use pulse::{Pulse, Environment};

let pulse = Pulse::builder("my-service", "1.0.0")
    .description("My awesome service")
    .environment(Environment::Production)
    .with_otlp("localhost", 4317)
    .with_mcap("output.mcap")
    .build()?;
```

**Available Builder Methods:**
- `.description(desc)` - Set service description
- `.environment(env)` - Set deployment environment (Development, Staging, Production, Jetson)
- `.with_otlp(host, port)` - Enable OpenTelemetry OTLP export
- `.with_mcap(path)` - Enable MCAP recording to file

### Legacy API

The original API is still supported for backward compatibility:

```rust
use pulse::options::{
    ServiceOptions,
    PulseOptions, 
    TelemetryOptions, 
    OtelOptions, 
    FoxgloveOptions,
    Environment
};

let service_opts = ServiceOptions::new("my-service", "1.0.0")
    .with_description("My awesome service")
    .with_environment(Environment::Production);

let pulse_opts = PulseOptions::new()
    .with_telemetry(
        TelemetryOptions::default()
            .with_otlp(OtelOptions::new("localhost", 4317))
    )
    .with_foxglove(FoxgloveOptions::new("output.mcap"));

let pulse = Pulse::new(service_opts, pulse_opts)?;
```

MCAP files can be opened in [Foxglove Studio](https://foxglove.dev/) for offline analysis and visualization.

## Environment Variables

### Log Level Configuration

```bash
# Set global log level
export RUST_LOG=info

# Set per-module log levels
export RUST_LOG=info,pulse=debug,my_service=trace

# Suppress noisy dependencies
export RUST_LOG=info,h2=warn,tonic=warn
```

### Custom Log Configuration

Create a `log4rs.yaml` file in your project root for advanced logging configuration:

```yaml
appenders:
  stdout:
    kind: console
    encoder:
      pattern: "{d} [{l}] {m}{n}"

root:
  level: info
  appenders:
    - stdout
```

## Examples

The `pulse-rs/pulse/examples/` directory contains complete working examples:

### Logging Example

```bash
cargo run --example logging
```

Demonstrates:
- Basic logging with format specifiers
- Structured data logging
- OpenTelemetry integration
- Multiple log levels

### Metrics Example

```bash
cargo run --example metrics
```

Demonstrates:
- Derive macro for metrics
- Direct metrics recording
- MCAP file generation
- Counter, histogram, and gauge metrics

### MCAP Logging Example

```bash
cargo run --example logging_mcap
```

Demonstrates:
- Recording logs to MCAP files
- Foxglove Studio integration

### Tracing Example

```bash
cargo run --example tracing
```

Demonstrates:
- Distributed tracing with OpenTelemetry
- Automatic span instrumentation
- OTLP export to collectors

## Project Structure

```
pulse-rs/
├── pulse/                    # Main library
│   ├── src/
│   │   ├── lib.rs           # Main Pulse struct and API
│   │   ├── logging/         # Logging implementation
│   │   ├── metrics/         # Metrics implementation
│   │   ├── tracing/         # Tracing implementation
│   │   ├── foxglove/        # MCAP writer
│   │   ├── telemetry/       # OpenTelemetry provider
│   │   ├── options/         # Configuration options
│   │   ├── traits/          # Shared traits
│   │   └── derive/          # Re-exports for macros
│   └── examples/            # Usage examples
├── pulse-derive/            # Procedural macros
│   └── src/
│       └── lib.rs           # Metrics derive macro
├── Cargo.toml               # Workspace configuration
└── .env.example             # Environment variables template
```

## API Reference

### Pulse Struct

The main entry point for the observability framework.

```rust
pub struct Pulse {
    pub logger: Logger,
    pub tracing: Option<PulseTracing>,
    pub metrics: Metrics,
}

impl Pulse {
    pub fn new(
        service_opts: ServiceOptions, 
        pulse_opts: PulseOptions
    ) -> Result<Self>;
    
    pub fn flush(&self);
    pub fn close(self) -> Result<()>;
    pub fn mcap_writer(&self) -> Option<Arc<Mutex<UnifiedMcapWriter>>>;
    pub fn meter_provider(&self) -> Option<Arc<SdkMeterProvider>>;
}
```

### Logger

Global logging macros available throughout your application.

```rust
use pulse::logger;

logger::trace!("Trace message");
logger::debug!("Debug message");
logger::info!("Info message");
logger::warn!("Warning message");
logger::error!("Error message");

// With structured data
logger::info!("Event occurred").with_data(&my_struct);

// With format specifiers
logger::info!("User {} logged in at {}", user_id, timestamp);
```

### Metrics

Record metrics with type safety.

```rust
// Direct recording
pulse.metrics.counter("metric_name", value)?;
pulse.metrics.histogram("metric_name", value)?;
pulse.metrics.gauge("metric_name", value)?;

// Struct-based recording
pulse.metrics.record(&my_metrics)?;
```

### Metrics Derive Macro

```rust
use pulse::derive::Metrics;

#[derive(Debug, Metrics)]
pub struct MyMetrics {
    #[metric(name = "my.counter", description = "A counter", counter)]
    pub counter_field: u64,

    #[metric(name = "my.histogram", description = "A histogram", histogram)]
    pub histogram_field: f64,

    #[metric(name = "my.gauge", description = "A gauge", gauge)]
    pub gauge_field: f64,
}
```

## Integration with Observability Stack

Pulse works seamlessly with the included OpenTelemetry stack:

```bash
# Start the observability stack
cd opentelemetry
docker compose up -d
```

This provides:
- **Grafana** at `http://localhost:3000` - Dashboards and visualization
- **Loki** - Log aggregation
- **Tempo** - Distributed tracing
- **Prometheus** - Metrics storage
- **OTLP Collector** at `localhost:4317` - Telemetry ingestion

Configure your application to send telemetry:

```rust
let pulse_opts = PulseOptions::new()
    .with_telemetry(
        TelemetryOptions::default()
            .with_otlp(OtelOptions::new("localhost", 4317))
    );
```

## Best Practices

### 1. Always Close Pulse

```rust
let pulse = Pulse::new(service_opts, pulse_opts)?;

// Your application logic

pulse.close()?; // Ensures all telemetry is flushed
```

### 2. Use Structured Logging

```rust
// Good: Structured data
logger::info!("User action").with_data(&event);

// Avoid: String interpolation for complex data
logger::info!("User action: {:?}", event);
```

### 3. Reuse Metrics Structs

```rust
let metrics = ApiMetrics { /* ... */ };

// Record periodically
loop {
    pulse.metrics.record(&metrics)?;
    tokio::time::sleep(Duration::from_secs(10)).await;
}
```

### 4. Use Tracing Instrumentation

```rust
#[instrument]
async fn my_function() {
    // Automatically creates spans
}
```

### 5. Configure Log Levels Appropriately

```bash
# Production
export RUST_LOG=info,my_service=info

# Development
export RUST_LOG=debug,my_service=trace
```

## Troubleshooting

### Logs Not Appearing

1. Check `RUST_LOG` environment variable
2. Verify log level is set appropriately
3. Ensure `pulse.close()` is called before exit

### Metrics Not Recorded

1. Verify OTLP collector is running
2. Check network connectivity to `localhost:4317`
3. Ensure metrics are recorded before `pulse.close()`

### MCAP Files Not Generated

1. Verify write permissions in output directory
2. Ensure `pulse.close()` is called to finalize the file
3. Check that `FoxgloveOptions` is configured

### Tracing Not Working

1. Verify OTLP is enabled in configuration
2. Check that `#[instrument]` is used on async functions
3. Ensure Tempo is running and accessible

## Performance Considerations

- **Async-First**: Built on Tokio for high-performance async workloads
- **Minimal Overhead**: Structured logging with minimal allocations
- **Batched Exports**: Telemetry is batched for efficient network usage
- **Configurable Sampling**: Control trace sampling rates for high-throughput services

## Contributing

Contributions are welcome! Please see the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## License

Copyright © 2026 Machani Robotics

Licensed under the Apache License, Version 2.0. See [LICENSE](../LICENSE) for details.

## Resources

- [Main Pulse Documentation](../README.md)
- [OpenTelemetry Documentation](https://opentelemetry.io/)
- [Foxglove Studio](https://foxglove.dev/)
- [Tokio Documentation](https://tokio.rs/)

## License

Copyright © 2026 Machani Robotics

Licensed under the Apache License, Version 2.0.
