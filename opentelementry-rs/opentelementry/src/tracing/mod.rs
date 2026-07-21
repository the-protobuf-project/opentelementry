//! Distributed tracing support.
//!
//! This module provides OpenTelemetry-based distributed tracing with support
//! for both automatic instrumentation via `#[instrument]` and manual span management.

#[allow(clippy::module_inception)]
pub mod tracing;

pub use ::tracing as reexport;
pub use tracing::{OpentelementryTracing, Span, init_tokio_tracing};

/// Re-export Opentelementry's instrument macro.
///
/// Use this to automatically instrument functions with tracing spans.
///
/// # Examples
///
/// ```no_run
/// use opentelementry::tracing::instrument;
///
/// #[instrument]
/// async fn my_function() {
///     // Function body
/// }
/// ```
pub use opentelementry_derive::{instrument, trace};
