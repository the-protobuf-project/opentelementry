# Opentelementry examples (standalone crate)

Runs the same scenarios as `opentelementry-go/examples/`, as a **separate package** in the workspace.

## OpenTelemetry collector (port **12005**)

Examples that export OTLP use **gRPC** to `localhost:12005` (see `DEFAULT_OTEL_COLLECTOR_OTLP_PORT` in the `opentelementry` crate).

Configure your collector to listen for OTLP gRPC on **12005**, for example:

```yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:12005
```

Or run a collector container with port mapping `12005:4317` and point Opentelementry at `localhost:12005`.

## Run

From `opentelementry-rs/`:

```bash
cargo run -p opentelementry-examples --bin logging
cargo run -p opentelementry-examples --bin logging_mcap
cargo run -p opentelementry-examples --bin metrics    # OTLP metrics → :12005
cargo run -p opentelementry-examples --bin tracing   # OTLP traces → :12005
cargo run -p opentelementry-examples --bin module_levels
```

## Ergonomics

- **`opentelementry_local_otel!()`** — builder preset for `localhost:12005`
- **`OpentelementryBuilder::with_local_otel_collector()`** — same
- **`Opentelementry` `Drop`** — calls `close()` → flush + OTLP shutdown (safe to ignore explicit `close()`)
