//! Configuration options for Opentelementry.
//!
//! This module contains all configuration structures for setting up
//! Opentelementry with various backends and features.

pub mod foxglove;
pub mod logging;
pub mod opentelementry;
pub mod profiling;
pub mod service;
pub mod telemetry;
pub mod tracing;

pub use foxglove::FoxgloveOptions;
pub use logging::{LogLevel, LogOptions, LoggingOptions, ModuleOptions, TimeFormat};
pub use opentelementry::OpentelementryOptions;
pub use profiling::ProfilingOptions;
pub use service::{Environment, ServiceOptions};
pub use telemetry::{
    LoggingTelemetryOptions, MetricsTelemetryOptions, OTLPOptions, OtelOptions, TelemetryOptions,
    TracingTelemetryOptions,
};
pub use tracing::TracingOptions;
