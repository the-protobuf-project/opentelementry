//! Configuration options for Pulse.
//!
//! This module contains all configuration structures for setting up
//! Pulse with various backends and features.

pub mod service;
pub mod pulse;
pub mod foxglove;
pub mod telemetry;

pub use service::{ServiceOptions, Environment};
pub use pulse::PulseOptions;
pub use foxglove::FoxgloveOptions;
pub use telemetry::{TelemetryOptions, OtelOptions};
