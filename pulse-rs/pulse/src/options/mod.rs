//! Configuration options for Pulse.
//!
//! This module contains all configuration structures for setting up
//! Pulse with various backends and features.

pub mod foxglove;
pub mod logging;
pub mod profiling;
pub mod pulse;
pub mod service;
pub mod telemetry;
pub mod tracing;

pub use foxglove::FoxgloveOptions;
pub use logging::{LogOptions, LoggingOptions, TimeFormat};
pub use profiling::ProfilingOptions;
pub use pulse::PulseOptions;
pub use service::{Environment, ServiceOptions};
pub use telemetry::{
    LoggingTelemetryOptions, MetricsTelemetryOptions, OTLPOptions, OtelOptions, TelemetryOptions,
    TracingTelemetryOptions,
};
pub use tracing::TracingOptions;
