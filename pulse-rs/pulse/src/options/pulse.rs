//! Main Pulse configuration options.
//!
//! Aggregates all configuration options for the Pulse library.

use serde::{Deserialize, Serialize};
use super::{TelemetryOptions, FoxgloveOptions};

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
    pub telemetry: TelemetryOptions,
    pub foxglove: FoxgloveOptions,
}

impl PulseOptions {
    /// Creates new default Pulse options.
    pub fn new() -> Self {
        Self::default()
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
}
