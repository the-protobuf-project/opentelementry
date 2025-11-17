mod config;
mod error;
mod session;

#[cfg(feature = "logs")]
pub mod log;

#[cfg(feature = "trace")]
pub mod trace;

#[cfg(feature = "metrics")]
pub mod metrics;

pub use {
    config::Config,
    error::Error,
    session::{init, Session},
};

#[cfg(feature = "otel-api")]
pub use {opentelemetry as otel, opentelemetry_otlp as otlp, opentelemetry_sdk as otel_sdk};
