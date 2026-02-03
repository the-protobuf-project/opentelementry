# Pulse - Python SDK

A comprehensive observability framework for Python applications, providing
unified logging, metrics, and distributed tracing capabilities with
OpenTelemetry integration and MCAP recording for Foxglove Studio.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [Core Concepts](#core-concepts)
- [Examples](#examples)
- [Best Practices](#best-practices)

## Features

- **Logging** - Structured logging with OTLP export and MCAP recording
- **Metrics** - Pydantic-based metrics with automatic service name prefix
- **Tracing** - Decorator-based distributed tracing
- **MCAP Recording** - Unified file for Foxglove Studio visualization
- **Config-first** - Auto-discovers `pulse.toml` with environment variable overrides

## Installation

```bash
# Using pip
pip install git+https://github.com/machanirobotics/pulse.git#subdirectory=pulse-py

# Using uv (recommended)
cd pulse-py && uv sync
```

**Requirements:** Python 3.12+

## Quick Start

### 1. Create `pulse.toml` in your project root

```toml
[service]
name = "my-service"
version = "1.0.0"
environment = "development"

[telemetry.otlp]
endpoint = "otel.example.com"
auth_token = "your-token"
```

### 2. Use Pulse in your code

```python
from pulse import Pulse

# Auto-discovers pulse.toml config
with Pulse.new().build() as pulse:
    pulse.logger.info("Service started")
    pulse.logger.warning("Rate limit approaching", {"percent": 85})
```

That's it! No manual configuration needed.

## Configuration

Configuration is loaded in priority order (lowest to highest):

```mermaid
flowchart LR
    A[Defaults] --> B[pulse.toml]
    B --> C[.env / PULSE_*]
    C --> D[Builder methods]
```

### Config File (`pulse.toml`)

```toml
[service]
name = "my-service"
version = "1.0.0"
environment = "development"  # development | staging | production

[telemetry.otlp]
endpoint = "otel.example.com"  # Port 4317 auto-added for gRPC
auth_token = "your-token"
secure = false                  # Use TLS

[foxglove]
enabled = true
file_path = "/tmp/telemetry.mcap"

[tracing]
enabled = true
```

### Environment Variables

Override config with `PULSE_` prefix and double underscores for nesting:

```bash
# .env
PULSE_SERVICE__NAME=my-service
PULSE_TELEMETRY__OTLP__ENDPOINT=otel.example.com
PULSE_TELEMETRY__OTLP__AUTH_TOKEN=your-token
```

### Builder Pattern (Code Overrides)

```python
from pulse import Pulse, Environment

# Builder methods have highest priority
pulse = Pulse.new() \
    .with_service("my-service", "1.0.0") \
    .environment(Environment.PRODUCTION) \
    .with_otlp("otel.example.com", 4317) \
    .build()
```

## Core Concepts

### Logging

```python
from pulse import Pulse

with Pulse.new().build() as pulse:
    pulse.logger.info("User logged in", {"user_id": "123"})
    pulse.logger.warning("Rate limit", {"percent": 85})
    pulse.logger.error("Request failed", {"error": "timeout"})
```

### Metrics

Metrics are auto-prefixed with service name from config:

```python
import pulse
from pulse import Pulse, MetricsBaseModel

# No prefix needed - uses service name from pulse.toml
class LLMMetrics(MetricsBaseModel):
    tokens: int = pulse.Counter(description="Total tokens")
    latency: float = pulse.Histogram(description="Response time")
    active: int = pulse.Gauge(description="Active requests")

with Pulse.new().build() as p:
    metrics = LLMMetrics(tokens=150, latency=245.5, active=3)
    p.metrics.record(metrics)
    # Generates: my-service.tokens, my-service.latency, my-service.active
```

### Tracing

```python
import pulse
from pulse import Pulse, TracedOperation

@pulse.trace("process_request", auto_events=True)
def process_request(user_id: str):
    return {"status": "success"}

with Pulse.new().build() as p:
    # Using decorator
    result = process_request("user-123")

    # Using TracedOperation
    with TracedOperation(p.tracing, "pipeline") as op:
        op.step("loading")
        op.step("processing")
        op.step("saving")
```

## Examples

```bash
# Run examples
uv run python examples/logging/simple_example.py
uv run python examples/metrics/simple_example.py
uv run python examples/tracing/simple_example.py
```

## Best Practices

1. **Use `Pulse.new().build()`** - Auto-discovers config from `pulse.toml`
2. **Use context managers** - Ensures proper cleanup
3. **Structured logging** - Pass dicts, not f-strings
4. **No metric prefix needed** - Service name is auto-prefixed

## License

Copyright © 2026 Machani Robotics. Apache License 2.0.
