//! Main Opentelementry configuration options.
//!
//! Aggregates all configuration options for the Opentelementry library.

use super::{FoxgloveOptions, LoggingOptions, ProfilingOptions, TelemetryOptions, TracingOptions};
use serde::{Deserialize, Serialize};

/// Main configuration options for Opentelementry.
///
/// # Examples
///
/// ```no_run
/// use opentelementry::options::{OpentelementryOptions, TelemetryOptions, FoxgloveOptions};
///
/// let opts = OpentelementryOptions::new()
///     .with_telemetry(TelemetryOptions::default())
///     .with_foxglove(FoxgloveOptions::new("output.mcap"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpentelementryOptions {
    /// Logging options for the service.
    #[serde(default)]
    pub logging: LoggingOptions,
    /// Foxglove options for the service.
    #[serde(default)]
    pub foxglove: FoxgloveOptions,
    /// Unified telemetry options (OpenTelemetry-based).
    #[serde(default)]
    pub telemetry: TelemetryOptions,
    /// Continuous profiling options (Pyroscope).
    #[serde(default)]
    pub profiling: ProfilingOptions,
    /// Distributed tracing options.
    #[serde(default)]
    pub tracing: TracingOptions,
}

impl OpentelementryOptions {
    /// Creates new default Opentelementry options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets logging configuration.
    pub fn with_logging(mut self, logging: LoggingOptions) -> Self {
        self.logging = logging;
        self
    }

    /// Sets telemetry configuration.
    pub fn with_telemetry(mut self, telemetry: TelemetryOptions) -> Self {
        self.telemetry = telemetry;
        self
    }

    /// Sets Foxglove configuration.
    pub fn with_foxglove(mut self, foxglove: FoxgloveOptions) -> Self {
        self.foxglove = foxglove;
        self
    }

    /// Sets profiling configuration.
    pub fn with_profiling(mut self, profiling: ProfilingOptions) -> Self {
        self.profiling = profiling;
        self
    }

    /// Sets tracing configuration.
    pub fn with_tracing(mut self, tracing: TracingOptions) -> Self {
        self.tracing = tracing;
        self
    }
}
