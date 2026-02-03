//! Logging configuration options.

use serde::{Deserialize, Serialize};

/// Time format options for log output.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TimeFormat {
    #[default]
    RFC3339,
    RFC3339Nano,
    Kitchen,
    Stamp,
    Custom,
}

/// Log output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogOptions {
    /// Prefix string in log output.
    #[serde(default)]
    pub prefix: String,
    /// Include caller info in logs.
    #[serde(default = "default_true")]
    pub report_caller: bool,
    /// Include timestamp in logs.
    #[serde(default = "default_true")]
    pub report_timestamp: bool,
    /// Named time format enum.
    #[serde(default)]
    pub time_format_key: TimeFormat,
    /// Custom format if time_format_key == Custom.
    #[serde(default)]
    pub custom_format: String,
    /// Adjust call depth for correct file/line display.
    #[serde(default = "default_caller_offset")]
    pub caller_offset: i32,
}

fn default_true() -> bool {
    true
}

fn default_caller_offset() -> i32 {
    3
}

impl Default for LogOptions {
    fn default() -> Self {
        Self {
            prefix: String::new(),
            report_caller: true,
            report_timestamp: true,
            time_format_key: TimeFormat::default(),
            custom_format: String::new(),
            caller_offset: 3,
        }
    }
}

/// Logging options container.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LoggingOptions {
    /// Log options.
    #[serde(default)]
    pub log: LogOptions,
}
