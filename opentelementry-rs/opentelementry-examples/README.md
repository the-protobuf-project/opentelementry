# Opentelementry examples (standalone crate)

Runs the same scenarios as `opentelementry-go/examples/`, as a **separate package** in the workspace.

## OpenTelemetry collector (port **6009**)

Examples that export OTLP use **gRPC** to `localhost:6009` (see `DEFAULT_OTEL_COLLECTOR_OTLP_PORT` in the `opentelementry` crate).

Configure your collector to listen for OTLP gRPC on **6009**, for example:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:6009
```

Or run a collector container with port mapping `6009:4317` and point Opentelementry at `localhost:6009`.

## Run

From `opentelementry-rs/`:

```bash
cargo run -p opentelementry-examples --bin logging
cargo run -p opentelementry-examples --bin logging_mcap
cargo run -p opentelementry-examples --bin metrics    # OTLP metrics → :6009
cargo run -p opentelementry-examples --bin tracing   # OTLP traces → :6009
cargo run -p opentelementry-examples --bin module_levels
```

## Ergonomics

- **`opentelementry_local_otel!()`** — builder preset for `localhost:6009`
- **`OpentelementryBuilder::with_local_otel_collector()`** — same
- **`Opentelementry` `Drop`** — calls `close()` → flush + OTLP shutdown (safe to ignore explicit `close()`)
