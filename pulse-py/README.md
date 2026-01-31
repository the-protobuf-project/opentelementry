# Pulse - Python SDK

A comprehensive observability framework for Python applications, providing
unified logging, metrics, and distributed tracing capabilities with
OpenTelemetry integration and MCAP recording for Foxglove Studio.

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
  - [Logging](#logging)
  - [Metrics](#metrics)
  - [Distributed Tracing](#distributed-tracing)
  - [MCAP Recording](#mcap-recording)
- [Examples](#examples)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Best Practices](#best-practices)

## Features

### Logging

- Structured logging with logbook integration
- Automatic export to OpenTelemetry Protocol (OTLP) collectors
- MCAP file recording for offline analysis in Foxglove Studio
- Colorized console output with caller information
- Support for all standard log levels (debug, info, warning, error, critical)

### Metrics

- Decorator-based metric definitions using Pydantic models
- Automatic metric name inference from field names
- Support for counters, histograms, and gauges
- OpenTelemetry export to Prometheus via OTLP
- MCAP recording for time-series visualization
- Type-safe metric definitions with validation

### Distributed Tracing

- Decorator-based automatic span creation
- Manual span management with context managers
- Automatic trace context propagation
- Nested span support with parent-child relationships
- Event and attribute recording
- Export to Jaeger, Zipkin, or any OpenTelemetry-compatible backend
- MCAP recording for trace visualization

### MCAP Recording

- Unified MCAP file containing logs, metrics, and traces
- Compatible with Foxglove Studio for visualization
- JSON schema encoding for easy inspection
- Efficient binary format for storage and replay

## Installation

### From Git Repository

Add Pulse to your Python project using Git:

```bash
# Using pip with Git
pip install git+https://github.com/machanirobotics/pulse.git#subdirectory=pulse-py

# Or clone and install locally
git clone https://github.com/machanirobotics/pulse.git
cd pulse/pulse-py
pip install -e .
```

### Using uv (Recommended for Development)

```bash
cd pulse-py
uv sync
```

**Requirements:**

- Python 3.10 or higher
- OpenTelemetry Collector (optional, for production deployments)
- Foxglove Studio (optional, for MCAP visualization)

### Dependencies

Pulse automatically installs the following dependencies:

- `pydantic` - Data validation and settings management
- `logbook` - Flexible logging library
- `opentelemetry-api` - OpenTelemetry API
- `opentelemetry-sdk` - OpenTelemetry SDK
- `opentelemetry-exporter-otlp` - OTLP exporter
- `mcap` - MCAP file format support
- `python-dotenv` - Environment variable management

## Quick Start

```python
import pulse
from pulse import Pulse, ServiceOptions, PulseOptions, Environment
from pydantic import BaseModel

# Initialize Pulse with context manager (recommended)
with Pulse(
    service_opts=ServiceOptions(
        name="my-service",
        version="1.0.0",
        environment=Environment.PRODUCTION,
    ),
) as p:
    # Logging
    p.logger.info("Service started", {"version": "1.0.0"})
    
    # Metrics
    @pulse.PulseMetricsBaseModel(prefix="api")
    class APIMetrics(BaseModel):
        requests: int = pulse.Counter(description="Total requests")
        latency: float = pulse.Histogram(description="Request latency")
    
    metrics = APIMetrics(requests=1, latency=45.2)
    p.metrics.record(metrics)
    
    # Tracing
    @pulse.trace("process_request", auto_events=True)
    def process_request(user_id: str):
        p.logger.info("Processing", {"user_id": user_id})
        return {"status": "success"}
    
    result = process_request("user-123")

# Resources are automatically cleaned up on exit
```

## Core Concepts

### Logging Details

Pulse provides structured logging with automatic export to multiple backends:

```python
from pulse import Pulse, ServiceOptions

with Pulse(service_opts=ServiceOptions(name="my-service")) as p:
    # Basic logging
    p.logger.info("User logged in", {"user_id": "123", "ip": "192.168.1.1"})
    p.logger.warning("Rate limit approaching", {"current": 95, "limit": 100})
    p.logger.error("Request failed", {"error": "timeout"})
    
    # All log levels supported
    p.logger.debug("Debug information", {"details": "..."})
    p.logger.critical("System failure", {"component": "database"})
```

**Features:**

- Structured data as dictionaries
- Automatic caller file and line number detection
- Colorized console output
- Simultaneous export to OTLP and MCAP
- Service context automatically included

### Metrics Details

Define metrics using the `@pulse.PulseMetricsBaseModel` decorator with
automatic name inference:

```python
import pulse
from pydantic import BaseModel

# Define metrics with automatic name generation
@pulse.PulseMetricsBaseModel(prefix="llm")
class LLMMetrics(BaseModel):
    """LLM processing metrics"""
    tokens_processed: int = pulse.Counter(description="Total tokens processed")
    response_time: float = pulse.Histogram(description="Response time in ms")
    active_requests: int = pulse.Gauge(description="Active requests")
    cache_hit_rate: float = pulse.Gauge(description="Cache hit rate")

# Record metrics
metrics = LLMMetrics(
    tokens_processed=150,
    response_time=245.5,
    active_requests=3,
    cache_hit_rate=0.85,
)
p.metrics.record(metrics)
```

**Metric Types:**

- **Counter**: Monotonically increasing values (requests, errors, bytes)
- **Histogram**: Value distributions (latencies, sizes, durations)
- **Gauge**: Point-in-time values (memory, connections, queue depth)

**Metric Names:**

- Field `tokens_processed` with prefix `llm` becomes `llm.tokens.processed`
- Exported to Prometheus as `llm_tokens_processed_total`
- Underscores in field names converted to dots

**Function Decorator for Metrics:**

```python
@pulse.metric("api_requests", metric_type="counter")
def handle_request():
    return "processed"

@pulse.metric("api_latency", record_duration=True)
def slow_operation():
    time.sleep(0.1)
    return "done"
```

### Distributed Tracing Details

Automatic span creation and context propagation:

```python
import pulse

# Function decorator for automatic tracing
@pulse.trace("process_request", auto_events=True)
def process_request(user_id: str):
    # Automatically creates span with events:
    # - process_request_started
    # - process_request_completed (or _failed on error)
    return {"status": "success"}

# TracedOperation for step-by-step tracking
with pulse.TracedOperation(p.tracing, "data_pipeline") as op:
    op.step("loading_data")
    data = load_data()
    
    op.step("validating_data")
    validate(data)
    
    op.step("processing_data")
    result = process(data)

# Manual span management
with p.tracing.start_span("complex_operation", {"user_id": "123"}) as span:
    span.add_event("started")
    span.set_attribute("records", 100)
    
    # Nested span
    with p.tracing.start_span("sub_operation") as sub_span:
        sub_span.add_event("processing")
        result = do_work()
    
    span.add_event("completed")
```

**Trace Context Propagation:**

- Trace IDs automatically propagated across decorated functions
- Parent-child span relationships maintained automatically
- No manual context passing required

### MCAP Recording Details

All telemetry data can be recorded to a single MCAP file:

```python
from pulse import Pulse, ServiceOptions, PulseOptions, FoxgloveOptions

with Pulse(
    service_opts=ServiceOptions(name="my-service"),
    pulse_opts=PulseOptions(
        foxglove=FoxgloveOptions(
            enabled=True,
            mcap_path="telemetry.mcap",
        ),
    ),
) as p:
    # All logs, metrics, and traces are recorded to telemetry.mcap
    p.logger.info("Event occurred")
    p.metrics.record(metrics)
```

**Viewing MCAP Files:**

1. Install Foxglove Studio: <https://foxglove.dev/download>
2. Open the MCAP file in Foxglove
3. Add panels to visualize:
   - **Log Panel**: View structured logs
   - **Plot Panel**: Time-series metrics
   - **Indicator Panel**: Current gauge values
   - **Table Panel**: Trace spans and events

## Examples

The `examples/` directory contains complete working examples:

### Logging Examples

**Simple Logging** (`examples/logging/simple_example.py`):
```python
from pulse import Pulse, ServiceOptions

with Pulse(service_opts=ServiceOptions(name="logging-example")) as p:
    p.logger.info("Application started")
    p.logger.debug("Debug information", {"details": "..."})
    p.logger.warning("Warning message", {"threshold": 90})
    p.logger.error("Error occurred", {"error_code": 500})
```

**MCAP Logging** (`examples/logging/mcap_example.py`):
Demonstrates logging with MCAP recording for Foxglove Studio visualization.

### Metrics Examples

**Metrics with Pydantic Models** (`examples/metrics/simple_example.py`):

```python
import pulse
from pydantic import BaseModel

@pulse.PulseMetricsBaseModel(prefix="llm")
class LLMMetrics(BaseModel):
    tokens_processed: int = pulse.Counter(description="Total tokens")
    response_time: float = pulse.Histogram(description="Response time in ms")
    active_requests: int = pulse.Gauge(description="Active requests")
    cache_hit_rate: float = pulse.Gauge(description="Cache hit rate")

@pulse.PulseMetricsBaseModel(prefix="transcription")
class TranscriptionMetrics(BaseModel):
    audio_duration: float = pulse.Histogram(description="Audio duration in seconds")
    confidence: float = pulse.Gauge(description="Confidence score")
    word_count: int = pulse.Counter(description="Words transcribed")

# Record metrics
llm_metrics = LLMMetrics(
    tokens_processed=150,
    response_time=245.5,
    active_requests=3,
    cache_hit_rate=0.85,
)
p.metrics.record(llm_metrics)
```

### Tracing Examples

**Distributed Tracing** (`examples/tracing/simple_example.py`):

Complete example demonstrating a multi-component AI assistant pipeline with 7 traced components:

1. Input Processing
2. Context Retrieval
3. Intent Classification
4. Knowledge Search
5. Response Generation
6. Response Validation
7. Output Formatting

Each component uses `@pulse.trace()` decorator with `TracedOperation` for
detailed step tracking.

### Running Examples

```bash
# Make sure OTLP collector is running
docker-compose -f ../opentelemetry/compose.yaml up

# Run logging example
uv run python examples/logging/simple_example.py

# Run metrics example (generates data for 2 minutes)
uv run python examples/metrics/simple_example.py

# Run tracing example
uv run python examples/tracing/simple_example.py

# View metrics in Prometheus
open http://localhost:9090

# View traces in Jaeger
open http://localhost:16686
```

## Configuration

### ServiceOptions

```python
from pulse import ServiceOptions, Environment

service_opts = ServiceOptions(
    name="my-service",              # Service name (required)
    description="My service",        # Service description (optional)
    version="1.0.0",                # Service version (required)
    environment=Environment.PRODUCTION,  # DEVELOPMENT, STAGING, PRODUCTION, JETSON
)
```

### TelemetryOptions

```python
from pulse import PulseOptions, TelemetryOptions, OTLPOptions, MetricsOptions

pulse_opts = PulseOptions(
    telemetry=TelemetryOptions(
        metrics=MetricsOptions(
            enabled=True,
            export_interval_seconds=5,  # How often to export metrics
        ),
        otlp=OTLPOptions(
            host="localhost",
            port=4317,
            enabled=True,
        ),
    ),
)
```

### FoxgloveOptions

```python
from pulse import PulseOptions, FoxgloveOptions

pulse_opts = PulseOptions(
    foxglove=FoxgloveOptions(
        enabled=True,
        mcap_path="telemetry.mcap",  # Path to MCAP file
    ),
)
```

### Environment Variables

Create a `.env` file in your project root:

```bash
# Service Configuration
PULSE_SERVICE_NAME=my-service
PULSE_SERVICE_VERSION=1.0.0
PULSE_SERVICE_ENVIRONMENT=production

# Logging
PULSE_LOG_LEVEL=info

# OTLP Configuration
PULSE_OTLP_ENABLED=true
PULSE_OTLP_HOST=localhost
PULSE_OTLP_PORT=4317

# MCAP Configuration
PULSE_MCAP_ENABLED=true
PULSE_MCAP_PATH=telemetry.mcap

# Metrics
PULSE_METRICS_ENABLED=true
PULSE_METRICS_EXPORT_INTERVAL=5

# Tracing
PULSE_TRACING_ENABLED=true
```

Load configuration from environment:

```python
from pulse import Pulse, from_env

# Automatically loads from .env file
with Pulse(*from_env()) as p:
    p.logger.info("Configured from environment")
```

## Architecture

```text
Pulse SDK
├── Logger (logbook wrapper)
│   ├── Console output (logbook)
│   ├── OTLP export (OpenTelemetry)
│   └── MCAP recording
│
├── Metrics (Pydantic integration)
│   ├── @PulseMetricsBaseModel decorator
│   ├── Automatic name inference
│   ├── OTLP export (OpenTelemetry)
│   └── MCAP recording
│
├── Tracing (decorator-based)
│   ├── @pulse.trace() decorator
│   ├── TracedOperation context manager
│   ├── Automatic context propagation
│   ├── OTLP export (OpenTelemetry)
│   └── MCAP recording
│
└── UnifiedMcapWriter
    └── Single MCAP file for all telemetry
```

### Data Flow

```text
Application Code
       │
       ├──> Logger ──┬──> Console (logbook)
       │             ├──> OTLP Collector ──> Loki
       │             └──> MCAP File ──> Foxglove Studio
       │
       ├──> Metrics ─┬──> OTLP Collector ──> Prometheus
       │             └──> MCAP File ──> Foxglove Studio
       │
       └──> Tracing ─┬──> OTLP Collector ──> Jaeger/Tempo
                     └──> MCAP File ──> Foxglove Studio
```

## Best Practices

### 1. Use Context Managers

Always use the context manager pattern to ensure proper cleanup:

```python
with Pulse(service_opts=ServiceOptions(name="my-service")) as p:
    # Your code here
    pass
# Resources automatically cleaned up
```

### 2. Structured Logging

Pass dictionaries for structured data instead of string formatting:

```python
# Good
p.logger.info("User action", {"user_id": user_id, "action": "login"})

# Avoid
p.logger.info(f"User {user_id} performed login")
```

### 3. Metric Prefixes

Use meaningful prefixes for metric organization:

```python
@pulse.PulseMetricsBaseModel(prefix="api")  # api.requests.total
@pulse.PulseMetricsBaseModel(prefix="db")   # db.queries.total
@pulse.PulseMetricsBaseModel(prefix="cache") # cache.hits.total
```

### 4. Trace Decorators

Use `@pulse.trace()` for automatic instrumentation:

```python
@pulse.trace("database_query", auto_events=True)
def query_database(query: str):
    # Automatically traced with start/complete events
    return results
```

### 5. Error Handling

Errors are automatically captured in traces:

```python
@pulse.trace("risky_operation", auto_events=True)
def risky_operation():
    try:
        # Operation
        pass
    except Exception as e:
        # Error automatically recorded in span
        raise
```

### 6. MCAP for Development

Enable MCAP recording during development for debugging:

```python
pulse_opts = PulseOptions(
    foxglove=FoxgloveOptions(
        enabled=True,  # Enable in development
        mcap_path="debug.mcap",
    ),
)
```

### 7. Metric Naming Conventions

Follow Prometheus naming conventions:

- Use underscores for word separation
- Include units in names (`_seconds`, `_bytes`, `_total`)
- Use descriptive prefixes (`http_`, `db_`, `cache_`)

## Differences from Go SDK

While maintaining feature parity, the Python implementation has Pythonic adaptations:

1. **Decorators instead of struct tags**: Python uses `@pulse.trace()` and
   `@pulse.PulseMetricsBaseModel()` decorators
2. **Pydantic models**: Metrics use Pydantic with field helpers instead of
   Go struct tags
3. **Context variables**: Trace propagation uses Python's `contextvars`
   instead of Go's context
4. **No explicit context passing**: Python decorators handle context automatically
5. **Type hints**: Full type hint support for IDE autocomplete and type checking

## MCAP File Format

The MCAP file contains three channels:

1. `/pulse/logs`: Structured log entries with JSON schema
2. `/pulse/metrics`: Metric values with labels and timestamps
3. `/pulse/traces`: Trace spans with attributes and events

All channels use JSON schema encoding for easy inspection and visualization
in Foxglove Studio.

## Troubleshooting

### Metrics not appearing in Prometheus

1. Wait 30-60 seconds for metric export and Prometheus scrape
2. Check OTLP collector is running: `docker ps | grep otel`
3. Verify Prometheus targets: <http://localhost:9090/targets>
4. Check metric names in Prometheus: <http://localhost:9090/graph>

### Traces not appearing in Jaeger

1. Ensure OTLP collector is configured for trace export
2. Check Jaeger UI: <http://localhost:16686>
3. Verify trace context is being propagated (use `@pulse.trace()` decorator)

### MCAP file not created

1. Check `FoxgloveOptions.enabled` is `True`
2. Verify write permissions for `mcap_path`
3. Ensure `pulse.close()` is called or context manager is used

## License

Copyright © 2026 Machani Robotics

Licensed under the Apache License, Version 2.0.
