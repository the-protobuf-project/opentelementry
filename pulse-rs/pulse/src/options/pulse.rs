//! Main Pulse configuration options.
//!
//! Aggregates all configuration options for the Pulse library.

use serde::{Deserialize, Serialize};
use super::{FoxgloveOptions, LoggingOptions, ProfilingOptions, TelemetryOptions, TracingOptions};

/// Main configuration options for Pulse.
///
/// # Examples
///
/// ```no_run
/// use pulse::options::{PulseOptions, TelemetryOptions, FoxgloveOptions};
///
/// let opts = PulseOptions::new()
///     .with_telemetry(TelemetryOptions::default())
///     .with_foxglove(FoxgloveOptions::new("output.mcap"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PulseOptions {
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

impl PulseOptions {
    /// Creates new default Pulse options.
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
