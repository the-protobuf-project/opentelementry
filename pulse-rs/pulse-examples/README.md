# Pulse examples (standalone crate)

Runs the same scenarios as `pulse-go/examples/`, as a **separate package** in the workspace.

## OpenTelemetry collector (port **12005**)

Examples that export OTLP use **gRPC** to `localhost:12005` (see `DEFAULT_OTEL_COLLECTOR_OTLP_PORT` in the `pulse` crate).

Configure your collector to listen for OTLP gRPC on **12005**, for example:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:12005
```

Or run a collector container with port mapping `12005:4317` and point Pulse at `localhost:12005`.

## Run

From `pulse-rs/`:

```bash
cargo run -p pulse-examples --bin logging
cargo run -p pulse-examples --bin logging_mcap
cargo run -p pulse-examples --bin metrics    # OTLP metrics → :12005
cargo run -p pulse-examples --bin tracing   # OTLP traces → :12005
cargo run -p pulse-examples --bin module_levels
```

## Ergonomics

- **`pulse_local_otel!()`** — builder preset for `localhost:12005`
- **`PulseBuilder::with_local_otel_collector()`** — same
- **`Pulse` `Drop`** — calls `close()` → flush + OTLP shutdown (safe to ignore explicit `close()`)
