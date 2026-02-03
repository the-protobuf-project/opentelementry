# Pulse [![codecov](https://codecov.io/gh/machanirobotics/pulse/graph/badge.svg?token=uXWq5jEJBz)](https://codecov.io/gh/machanirobotics/pulse)

![Pulse Logo](.assets/logo.png)

**Unified Observability Framework** - Production-grade logging, metrics,
tracing, and profiling for modern applications

## Overview

**Pulse** is a comprehensive observability framework that provides unified
telemetry for your applications. Built on OpenTelemetry standards, Pulse
makes it easy to instrument your code with structured logging, distributed
tracing, metrics collection, and continuous profiling.

Pulse is now open-sourced by **Machani Robotics** to help teams build
observable, maintainable systems.

## Features

- **Structured Logging** - Context-aware logging with automatic trace correlation
- **Metrics Collection** - Counters, histograms, and gauges with OpenTelemetry
- **Distributed Tracing** - End-to-end request tracking across services
- **Continuous Profiling** - Production performance analysis with Pyroscope
- **MCAP Recording** - Offline analysis with Foxglove Studio
- **Zero-Config Integration** - Works out of the box with sensible defaults
- **OpenTelemetry Native** - Standard protocols for maximum compatibility

## Quick Start

### Go SDK

Get started with Pulse in your Go applications:

```bash
go get github.com/machanirobotics/pulse/pulse-go
```

```go
package main

import (
    "github.com/machanirobotics/pulse/pulse-go"
    "github.com/machanirobotics/pulse/pulse-go/options"
)

func main() {
    p, err := pulse.New().
        WithService("robot-controller", "1.0.0").
        WithDescription("Controls robot arm movements").
        WithEnvironment(options.Production).
        WithAttributes(map[string]string{
            "robot.id":  "robot-001",
            "fleet.id":  "fleet-alpha",
            "region":    "us-west-2",
        }).
        WithOTLP("otel-collector", 4317).
        WithTracing().
        Build()
    if err != nil {
        panic(err)
    }
    defer p.Close()

    p.Logger.Info("Robot controller started")
}
```

**[📖 Full Go SDK Documentation →](pulse-go/README.md)**

### Python SDK

Get started with Pulse in your Python applications:

```bash
pip install pulse-py
```

```python
import pulse
from pulse import Pulse, ServiceOptions, PulseOptions, Environment

# Initialize Pulse
service_opts = ServiceOptions(
    name="my-service",
    version="1.0.0",
    environment=Environment.PRODUCTION,
)

pulse_opts = PulseOptions()

with Pulse(service_opts, pulse_opts) as p:
    # Use it!
    p.logger.info("Service started")
    
    # Record metrics
    class MyMetrics(pulse.MetricsBaseModel):
        requests: int = pulse.Counter(description="Total requests")
    
    p.metrics.record(MyMetrics(requests=1))
```

**[📖 Full Python SDK Documentation →](pulse-py/README.md)**

### Rust SDK

Get started with Pulse in your Rust applications:

```bash
cargo add pulse
```

```rust
use pulse::{Pulse, Environment, logger};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Auto-discovers pulse.toml config file
    let _pulse = Pulse::new()
        .with_service("my-service", "1.0.0")
        .environment(Environment::Production)
        .build()?;
    
    logger::info!("Service started");
    logger::warn!("Warning message");
    logger::error!("Error occurred");
    
    Ok(())
}
```

**[📖 Full Rust SDK Documentation →](pulse-rs/README.md)**

## Observability Stack

Pulse includes a complete, pre-configured observability stack powered by
industry-standard tools:

- **Loki** - Log aggregation
- **Tempo** - Distributed tracing
- **Prometheus** - Metrics storage
- **Pyroscope** - Continuous profiling
- **Grafana** - Unified dashboards
- **OpenTelemetry Collector** - Telemetry pipeline

### Running the Stack

```bash
cd otel
docker compose up -d
```

Access Grafana at `http://localhost:3000` with all datasources pre-configured.

**[📖 OpenTelemetry Stack Documentation →](opentelementry/README.md)**

## Architecture

```mermaid
graph TB
    App[Your Application]
    SDK[Pulse SDK]

    subgraph "Telemetry Signals"
        Logs[Logs]
        Metrics[Metrics]
        Traces[Traces]
        Profiles[Profiles]
    end

    subgraph "Collection & Storage"
        OTLP[OTLP Collector]
        Loki[Loki]
        Prometheus[Prometheus]
        Tempo[Tempo]
        Pyroscope[Pyroscope]
    end

    Grafana[Grafana Dashboards]

    App --> SDK
    SDK --> Logs
    SDK --> Metrics
    SDK --> Traces
    SDK --> Profiles

    Logs --> OTLP
    Metrics --> OTLP
    Traces --> OTLP
    Profiles --> Pyroscope

    OTLP --> Loki
    OTLP --> Prometheus
    OTLP --> Tempo

    Loki --> Grafana
    Prometheus --> Grafana
    Tempo --> Grafana
    Pyroscope --> Grafana
```

## Language Support

| Language | Status    | Documentation                            |
| -------- | --------- | ---------------------------------------- |
| Go       | ✅ Stable | [pulse-go/README.md](pulse-go/README.md) |
| Python   | ✅ Stable | [pulse-py/README.md](pulse-py/README.md) |
| Rust     | ✅ Stable | [pulse-rs/README.md](pulse-rs/README.md) |

## Use Cases

- **Microservices** - Track requests across service boundaries
- **API Services** - Monitor performance and errors
- **Robotics** - Record and analyze system behavior
- **ML Pipelines** - Trace data processing workflows
- **Production Debugging** - Correlate logs, traces, and metrics

## Contributing

We welcome contributions! Pulse is open-source and maintained by Machani Robotics.

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Submit a pull request

## License

Copyright © 2026 Machani Robotics

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

---

**Built with love by Machani Robotics** | Open Source Observability for Everyone
