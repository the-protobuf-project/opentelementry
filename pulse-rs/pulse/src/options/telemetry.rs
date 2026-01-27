//! OpenTelemetry configuration options.
//!
//! Configuration for OpenTelemetry OTLP exporters.

use serde::{Deserialize, Serialize};

/// OpenTelemetry OTLP configuration.
///
/// # Examples
///
/// ```no_run
/// use pulse::options::OtelOptions;
///
/// let opts = OtelOptions::new("localhost", 4317);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtelOptions {
    pub host: String,
    pub port: u16,
    pub enabled: bool,
}

impl Default for OtelOptions {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 4317,
            enabled: false,
        }
    }
}

impl OtelOptions {
    /// Creates new OTLP options with specified host and port.
    ///
    /// # Arguments
    ///
    /// * `host` - OTLP collector host
    /// * `port` - OTLP collector port
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            enabled: true,
        }
    }

    /// Returns the full OTLP endpoint URL.
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// Telemetry configuration options.
///
/// Contains all telemetry-related configuration including OTLP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryOptions {
    pub otlp: OtelOptions,
}

impl Default for TelemetryOptions {
    fn default() -> Self {
        Self {
            otlp: OtelOptions::default(),
        }
    }
}

impl TelemetryOptions {
    /// Sets OTLP configuration.
    pub fn with_otlp(mut self, otlp: OtelOptions) -> Self {
        self.otlp = otlp;
        self
    }
}
