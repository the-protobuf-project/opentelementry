//! Declarative macros for quick OTLP setup (aligned with `opentelementry-go` builder ergonomics).
//!
//! Default collector address for the `opentelementry-examples` workspace crate is **gRPC OTLP on port 12005**.
//! Configure your OpenTelemetry Collector to receive OTLP gRPC on that port.

/// Default OTLP **gRPC** port for a local collector used by `opentelementry-examples`.
///
/// Standard ports are 4317 (gRPC) and 4318 (HTTP). This project uses **12005** so examples
/// can run alongside a default collector without port clashes.
pub const DEFAULT_OTEL_COLLECTOR_OTLP_PORT: u16 = 12_005;

/// Starts a [`crate::OpentelementryBuilder`] pointed at `localhost` and [`DEFAULT_OTEL_COLLECTOR_OTLP_PORT`].
///
/// ```ignore
/// let _opentelementry = opentelementry::opentelementry_local_otel!()
///     .with_service("my-svc", "1.0.0")
///     .with_tracing()
///     .build()?;
/// ```
#[macro_export]
macro_rules! opentelementry_local_otel {
    () => {
        $crate::Opentelementry::new()
            .with_otlp("localhost", $crate::DEFAULT_OTEL_COLLECTOR_OTLP_PORT)
    };
}

/// Same as [`opentelementry_local_otel!`] but from an explicit service name/version builder base.
#[macro_export]
macro_rules! opentelementry_local_otel_service {
    ($name:expr, $version:expr) => {
        $crate::opentelementry_local_otel!().with_service($name, $version)
    };
}
