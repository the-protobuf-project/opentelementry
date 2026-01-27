//! Metrics collection and recording.
//!
//! This module provides functionality for collecting and recording metrics
//! to both OpenTelemetry and MCAP backends.

pub mod metrics;
mod mcap;

pub use metrics::{Metrics, MetricType, MetricField, RecordMetrics};
