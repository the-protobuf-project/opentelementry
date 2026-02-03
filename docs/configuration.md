# Pulse Configuration Guide

This document provides comprehensive documentation on how `pulse.toml` works, the configuration lifecycle, supported formats, and all available options.

## Table of Contents

- [Overview](#overview)
- [Supported Formats](#supported-formats)
- [Configuration Priority](#configuration-priority)
- [Configuration Lifecycle](#configuration-lifecycle)
- [File Discovery](#file-discovery)
- [Environment Variables](#environment-variables)
- [Complete Configuration Reference](#complete-configuration-reference)
- [SDK-Specific Details](#sdk-specific-details)

---

## Overview

Pulse uses a unified configuration system across all SDKs (Go, Python, Rust). The configuration defines:

- **Service identity** - Name, version, environment, and custom attributes
- **Telemetry settings** - OTLP export, metrics intervals
- **Logging options** - Caller info, timestamps, log levels
- **Tracing configuration** - Distributed tracing settings
- **Profiling** - Continuous profiling with Pyroscope
- **MCAP recording** - Foxglove Studio integration

---

## Supported Formats

Pulse supports multiple configuration file formats, auto-detected by file extension:

| Format | Extensions | Parser Library |
|--------|------------|----------------|
| TOML   | `.toml`    | Go: `knadh/koanf`, Python: `dynaconf`, Rust: `figment` |
| YAML   | `.yaml`, `.yml` | Same as above |
| JSON   | `.json`    | Same as above |

**Recommended:** Use TOML for its readability and native support for nested structures.

### Format Examples

**TOML** (Recommended):
```toml
[service]
name = "my-service"
version = "1.0.0"

[telemetry.otlp]
endpoint = "otel.example.com:4317"
```

**YAML**:
```yaml
service:
  name: my-service
  version: "1.0.0"

telemetry:
  otlp:
    endpoint: otel.example.com:4317
```

**JSON**:
```json
{
  "service": {
    "name": "my-service",
    "version": "1.0.0"
  },
  "telemetry": {
    "otlp": {
      "endpoint": "otel.example.com:4317"
    }
  }
}
```

---

## Configuration Priority

Configuration is loaded and merged in the following order (lowest to highest priority):

```
┌─────────────────────────────────────────────────────────────┐
│  4. Code (Builder Methods)          ← HIGHEST PRIORITY     │
├─────────────────────────────────────────────────────────────┤
│  3. Environment Variables (PULSE_*)                        │
├─────────────────────────────────────────────────────────────┤
│  2. Config File (pulse.toml / pulse.yaml / pulse.json)     │
├─────────────────────────────────────────────────────────────┤
│  1. Default Values                  ← LOWEST PRIORITY      │
└─────────────────────────────────────────────────────────────┘
```

**Key Points:**
- Later sources override earlier sources
- Environment variables always override config file values
- Builder methods in code have the final say
- Unspecified values fall back to sensible defaults

---

## Configuration Lifecycle

### 1. Initialization Phase

When you call `Pulse.new()` (or equivalent), the SDK begins the configuration loading process:

```mermaid
stateDiagram-v2
    [*] --> LoadDefaults: Pulse::new()
    
    LoadDefaults: Load Default Values
    note right of LoadDefaults
        Initialize all options with sensible defaults
        OTLP disabled, logging enabled, etc.
    end note
    
    LoadDefaults --> DiscoverConfig
    
    DiscoverConfig: Discover Config File
    note right of DiscoverConfig
        Check PULSE_CONFIG_PATH env var
        Search pulse.toml, pulse.yaml, pulse.json
        Search .config/pulse.toml, etc.
    end note
    
    DiscoverConfig --> ConfigFound: File exists
    DiscoverConfig --> LoadEnvVars: No file found
    
    ConfigFound --> ParseConfig
    
    ParseConfig: Parse & Merge Config File
    note right of ParseConfig
        Auto-detect format from extension
        Parse file contents
        Merge with defaults (file overrides defaults)
    end note
    
    ParseConfig --> LoadEnvVars
    
    LoadEnvVars: Load Environment Variables
    note right of LoadEnvVars
        Scan for PULSE_* prefixed variables
        Transform keys (PULSE_TELEMETRY_OTLP_HOST → nested)
        Merge with config (env vars override file)
    end note
    
    LoadEnvVars --> ApplyBuilder
    
    ApplyBuilder: Apply Builder Methods
    note right of ApplyBuilder
        .with_service(), .with_otlp(), etc.
        Code-level overrides have highest priority
    end note
    
    ApplyBuilder --> Validate
    
    Validate: Validate & Build
    note right of Validate
        Validate required fields
        Initialize telemetry providers
    end note
    
    Validate --> Ready: Success
    Validate --> Error: Validation failed
    
    Ready: Pulse Instance Ready
    Error: Configuration Error
    
    Ready --> [*]
    Error --> [*]
```

### 2. Runtime Behavior

Once initialized, configuration is **immutable**. To change configuration:
1. Close the existing Pulse instance
2. Create a new instance with updated configuration

---

## File Discovery

Pulse auto-discovers configuration files in this order:

### Discovery Priority

1. **`PULSE_CONFIG_PATH` environment variable** (if set)
2. **Current directory:**
   - `pulse.toml`
   - `pulse.yaml` / `pulse.yml`
   - `pulse.json`
3. **`.config` subdirectory:**
   - `.config/pulse.toml`
   - `.config/pulse.yaml` / `.config/pulse.yml`
   - `.config/pulse.json`

### Discovery Algorithm

```
function discoverConfigPath():
    // 1. Check environment variable first
    if PULSE_CONFIG_PATH is set and file exists:
        return PULSE_CONFIG_PATH
    
    // 2. Search in current directory
    for ext in [".toml", ".yaml", ".yml", ".json"]:
        if "pulse{ext}" exists:
            return "pulse{ext}"
    
    // 3. Search in .config directory
    for ext in [".toml", ".yaml", ".yml", ".json"]:
        if ".config/pulse{ext}" exists:
            return ".config/pulse{ext}"
    
    // 4. No config file found - use defaults only
    return null
```

### Explicit Path

You can bypass auto-discovery by specifying a path:

**Go:**
```go
opts, svc, _ := options.LoadConfigWithDefaults("/path/to/config.toml")
```

**Python:**
```python
from pulse.options import from_config
service_opts, pulse_opts = from_config("/path/to/config.toml")
```

**Rust:**
```rust
let config = PulseConfig::load_from("/path/to/config.toml")?;
```

---

## Environment Variables

Environment variables provide runtime configuration without modifying files.

### Naming Convention

| SDK | Prefix | Separator | Example |
|-----|--------|-----------|---------|
| Go | `PULSE_` | `_` (single underscore) | `PULSE_TELEMETRY_OTLP_ENDPOINT` |
| Python | `PULSE_` | `__` (double underscore) | `PULSE_TELEMETRY__OTLP__ENDPOINT` |
| Rust | `PULSE_` | `_` (single underscore) | `PULSE_TELEMETRY_OTLP_ENDPOINT` |

### Transformation Rules

Environment variable names are transformed to config paths:

```
PULSE_SERVICE_NAME          → service.name
PULSE_TELEMETRY_OTLP_HOST   → telemetry.otlp.host
PULSE_FOXGLOVE_ENABLED      → foxglove.enabled
```

### Common Environment Variables

```bash
# Service Configuration
PULSE_SERVICE_NAME=my-service
PULSE_SERVICE_VERSION=1.0.0
PULSE_SERVICE_ENVIRONMENT=production

# OTLP Configuration
PULSE_TELEMETRY_OTLP_ENDPOINT=otel.example.com:4317
PULSE_TELEMETRY_OTLP_AUTH_TOKEN=your-bearer-token
PULSE_TELEMETRY_OTLP_SECURE=true

# Feature Toggles
PULSE_FOXGLOVE_ENABLED=true
PULSE_PROFILING_ENABLED=true
PULSE_TRACING_ENABLED=true

# Config File Override
PULSE_CONFIG_PATH=/etc/pulse/config.toml
```

### Using `.env` Files

Python SDK supports `.env` files via `dynaconf`:

```bash
# .env
PULSE_SERVICE__NAME=my-service
PULSE_TELEMETRY__OTLP__ENDPOINT=otel.example.com:4317
```

---

## Complete Configuration Reference

### Full `pulse.toml` Example

```toml
# =============================================================================
# Service Configuration
# =============================================================================
[service]
name = "my-service"           # Required: Your service name
version = "1.0.0"             # Service version (semver recommended)
environment = "development"   # development | staging | production | jetson
description = "My awesome service"

# Global attributes added to ALL telemetry (logs, metrics, traces)
# Useful for robot IDs, device IDs, fleet IDs, etc.
[service.attributes]
robot_id = "robot-001"
fleet_id = "fleet-alpha"
region = "us-west-2"

# =============================================================================
# Telemetry Configuration (OpenTelemetry)
# =============================================================================
[telemetry]
enabled = true  # Master switch: enables logging, metrics, and tracing

# OTLP Exporter Configuration
# Sends telemetry to any OpenTelemetry-compatible backend:
#   - Grafana Cloud, Datadog, Honeycomb, Jaeger, etc.
[telemetry.otlp]
enabled = true
endpoint = "localhost:4317"   # OTLP endpoint (port auto-detected if omitted)
auth_token = ""               # Bearer token for authentication (optional)
secure = false                # Use TLS (auto-detected for non-localhost)
use_http = false              # Use HTTP instead of gRPC (default: gRPC)

# Custom headers for OTLP requests
[telemetry.otlp.headers]
# X-Custom-Header = "value"

# Metrics export interval
[telemetry.metrics]
export_interval_seconds = 10

# =============================================================================
# Logging Configuration
# =============================================================================
[logging.log]
report_caller = true          # Include file:line in logs
report_timestamp = true       # Include timestamp in logs
level = "info"                # debug | info | warn | error
caller_offset = 3             # Stack frame offset for caller info

# =============================================================================
# Foxglove MCAP Recording (Optional)
# =============================================================================
# Record logs/metrics to MCAP files for Foxglove Studio playback
[foxglove]
enabled = false
file_path = ""                # e.g., "./recordings/session.mcap"

# =============================================================================
# Continuous Profiling (Optional)
# =============================================================================
# Send CPU/memory profiles to Pyroscope or Grafana Cloud
[profiling]
enabled = false
server_address = "http://localhost:4040"
basic_auth_user = ""          # For Grafana Cloud
basic_auth_password = ""

# =============================================================================
# Distributed Tracing
# =============================================================================
[tracing]
enabled = true
sample_ratio = 1.0            # 0.0 to 1.0 (Rust only)
```

### Configuration Options Reference

#### `[service]` - Service Identity

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `name` | string | `"unnamed-service"` | Service name for identification |
| `version` | string | `"1.0.0"` | Service version (semver) |
| `environment` | string | `"development"` | Deployment environment |
| `description` | string | `""` | Human-readable description |

#### `[service.attributes]` - Custom Attributes

Key-value pairs added to all telemetry signals. Useful for:
- Robot/device identification
- Fleet/region tagging
- Custom metadata

#### `[telemetry]` - Telemetry Master Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `true` | Master switch for all telemetry |

#### `[telemetry.otlp]` - OTLP Exporter

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable OTLP export |
| `endpoint` | string | `"localhost:4317"` | OTLP collector endpoint |
| `auth_token` | string | `""` | Bearer token for auth |
| `secure` | bool | `false` | Use TLS connection |
| `use_http` | bool | `false` | Use HTTP instead of gRPC |
| `headers` | map | `{}` | Custom HTTP headers |

#### `[telemetry.metrics]` - Metrics Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `export_interval_seconds` | int | `10` | Metrics export interval |

#### `[logging.log]` - Logging Settings

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `report_caller` | bool | `true` | Include file:line in logs |
| `report_timestamp` | bool | `true` | Include timestamp |
| `level` | string | `"info"` | Log level |
| `caller_offset` | int | `3` | Stack frame offset |

#### `[foxglove]` - MCAP Recording

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable MCAP recording |
| `file_path` | string | `""` | Output file path |

#### `[profiling]` - Continuous Profiling

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `false` | Enable profiling |
| `server_address` | string | `"http://localhost:4040"` | Pyroscope server |
| `basic_auth_user` | string | `""` | Auth username |
| `basic_auth_password` | string | `""` | Auth password |

#### `[tracing]` - Distributed Tracing

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `enabled` | bool | `true` | Enable tracing |
| `sample_ratio` | float | `1.0` | Sampling ratio (Rust only) |

---

## SDK-Specific Details

### Go SDK

**Config Library:** [knadh/koanf](https://github.com/knadh/koanf)

```go
package main

import (
    "github.com/machanirobotics/pulse/pulse-go"
    "github.com/machanirobotics/pulse/pulse-go/options"
)

func main() {
    // Auto-discover config
    p, _ := pulse.New().Build()
    defer p.Close()
    
    // Or load config explicitly
    pulseOpts, serviceOpts, _ := options.LoadConfigWithDefaults("")
    
    // Or specify path
    pulseOpts, serviceOpts, _ := options.LoadConfigWithDefaults("/path/to/config.toml")
}
```

**Environment Variable Format:** Single underscore separator
```bash
PULSE_TELEMETRY_OTLP_ENDPOINT=localhost:4317
```

### Python SDK

**Config Library:** [dynaconf](https://www.dynaconf.com/)

```python
from pulse import Pulse
from pulse.options import from_config

# Auto-discover config
with Pulse.new().build() as pulse:
    pulse.logger.info("Hello")

# Or load config explicitly
service_opts, pulse_opts = from_config()

# Or specify path
service_opts, pulse_opts = from_config("/path/to/config.toml")
```

**Environment Variable Format:** Double underscore separator
```bash
PULSE_TELEMETRY__OTLP__ENDPOINT=localhost:4317
```

**`.env` File Support:** Yes (auto-loaded)

### Rust SDK

**Config Library:** [figment](https://docs.rs/figment)

```rust
use pulse::{Pulse, PulseConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Auto-discover config
    let _pulse = Pulse::new().build()?;
    
    // Or load config explicitly
    let config = PulseConfig::load()?;
    
    // Or specify path
    let config = PulseConfig::load_from("/path/to/config.toml")?;
    
    Ok(())
}
```

**Environment Variable Format:** Single underscore separator
```bash
PULSE_TELEMETRY_OTLP_ENDPOINT=localhost:4317
```

---

## Best Practices

### 1. Use Environment Variables for Secrets

Never commit secrets to config files:

```toml
# pulse.toml - NO SECRETS HERE
[telemetry.otlp]
endpoint = "otel.example.com:4317"
# auth_token loaded from environment
```

```bash
# Set via environment
export PULSE_TELEMETRY_OTLP_AUTH_TOKEN="your-secret-token"
```

### 2. Environment-Specific Configs

Use different config files per environment:

```
config/
├── pulse.development.toml
├── pulse.staging.toml
└── pulse.production.toml
```

```bash
export PULSE_CONFIG_PATH=config/pulse.production.toml
```

### 3. Use Service Attributes for Context

Add identifying attributes for better observability:

```toml
[service.attributes]
robot_id = "robot-001"
fleet_id = "fleet-alpha"
deployment_id = "deploy-abc123"
```

### 4. Start with Defaults

Pulse works out of the box. Only configure what you need:

```toml
# Minimal production config
[service]
name = "my-service"

[telemetry.otlp]
endpoint = "otel.example.com:4317"
auth_token = "token"
```

---

## Troubleshooting

### Config Not Loading

1. **Check file exists:** Ensure `pulse.toml` is in the current working directory
2. **Check permissions:** File must be readable
3. **Validate syntax:** Use a TOML validator
4. **Check discovery:** Set `PULSE_CONFIG_PATH` explicitly

### Environment Variables Not Working

1. **Check prefix:** Must start with `PULSE_`
2. **Check separator:** Go/Rust use `_`, Python uses `__`
3. **Check case:** Variable names are case-insensitive for keys

### Debug Configuration Loading

Enable debug logging to see configuration sources:

```bash
# See which config file is loaded
RUST_LOG=pulse=debug cargo run

# Python
PULSE_DEBUG=true python app.py
```
