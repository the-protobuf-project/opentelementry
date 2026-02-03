//! Distributed tracing support.
//!
//! This module provides OpenTelemetry-based distributed tracing with support
//! for both automatic instrumentation via `#[instrument]` and manual span management.

#[allow(clippy::module_inception)]
pub mod tracing;

pub use tracing::{PulseTracing, Span, init_tokio_tracing};

/// Re-export the instrument macro from tracing crate.
///
/// Use this to automatically instrument functions with tracing spans.
///
/// # Examples
///
/// ```no_run
/// use pulse::tracing::instrument;
///
/// #[instrument]
/// async fn my_function() {
///     // Function body
/// }
/// ```
pub use ::tracing::instrument;
