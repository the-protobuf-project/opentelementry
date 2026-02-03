//! Distributed tracing with OpenTelemetry.
//!
//! This module provides integration with OpenTelemetry for distributed tracing,
//! supporting both tokio-rs/tracing instrumentation and manual span management.

use anyhow::Result;
use opentelemetry::trace::{Status, TracerProvider as _};
use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::SdkTracerProvider;

use crate::options::ServiceOptions;

/// Initialize tokio-rs/tracing with OpenTelemetry integration.
///
/// This sets up the tracing subscriber to send spans to OpenTelemetry/Tempo
/// via OTLP. Use the #[instrument] macro on functions to automatically create spans.
///
/// Set ENABLE_TRACING_DEFAULT=true to show TRACE logs from dependencies.
/// Default is to only show INFO and above.
pub fn init_tokio_tracing(service_opts: &ServiceOptions) -> Result<()> {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Create OTLP exporter for traces
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4317")
        .build()?;

    let resource = Resource::builder()
        .with_service_name(service_opts.name.clone())
        .with_attributes(vec![
            KeyValue::new("service.version", service_opts.version.clone()),
            KeyValue::new(
                "deployment.environment",
                service_opts.environment.to_string(),
            ),
        ])
        .build();

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(resource)
        .build();

    let tracer = provider.tracer(service_opts.name.clone());
    global::set_tracer_provider(provider);

    // Create OpenTelemetry tracing layer
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // Set up env filter - default to INFO level unless ENABLE_TRACING_DEFAULT is set
    // Hide OpenTelemetry, h2, tonic, and hyper internal logs by default
    let filter = if std::env::var("ENABLE_TRACING_DEFAULT").is_ok() {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,opentelemetry=warn,opentelemetry_sdk=warn,h2=warn,tonic=warn,hyper=warn,tower=warn"))
    };

    // Set up console logging for tracing events
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true);

    // Use try_init to avoid panic if already initialized
    let _ = tracing_subscriber::registry()
        .with(filter)
        .with(telemetry)
        .with(fmt_layer)
        .try_init();

    Ok(())
}

/// Pulse tracing for manual span management.
///
/// Provides manual control over span creation and management,
/// as an alternative to the `#[instrument]` macro.
///
/// # Examples
///
/// ```no_run
/// use pulse::tracing::PulseTracing;
/// use pulse::options::ServiceOptions;
///
/// let service_opts = ServiceOptions::new("my-service", "1.0.0");
/// let tracing = PulseTracing::new(&service_opts, Some("http://localhost:4317".to_string())).unwrap();
///
/// let mut span = tracing.start_span("my_operation");
/// span.set_attribute("key", "value");
/// span.end();
/// ```
pub struct PulseTracing {
    tracer: Option<opentelemetry_sdk::trace::Tracer>,
}

impl PulseTracing {
    /// Creates a new PulseTracing instance.
    ///
    /// # Arguments
    ///
    /// * `service_opts` - Service configuration
    /// * `otlp_endpoint` - Optional OTLP endpoint URL
    pub fn new(service_opts: &ServiceOptions, otlp_endpoint: Option<String>) -> Result<Self> {
        let tracer = if let Some(endpoint) = otlp_endpoint {
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()?;

            let resource = Resource::builder()
                .with_service_name(service_opts.name.clone())
                .with_attributes(vec![
                    KeyValue::new("service.version", service_opts.version.clone()),
                    KeyValue::new(
                        "deployment.environment",
                        service_opts.environment.to_string(),
                    ),
                ])
                .build();

            let provider = SdkTracerProvider::builder()
                .with_batch_exporter(exporter)
                .with_resource(resource)
                .build();

            global::set_tracer_provider(provider.clone());
            Some(provider.tracer(service_opts.name.clone()))
        } else {
            None
        };

        Ok(Self { tracer })
    }

    /// Starts a new span.
    ///
    /// # Arguments
    ///
    /// * `name` - Span name
    pub fn start_span(&self, name: &'static str) -> Span {
        Span::new(self.tracer.as_ref(), name)
    }

    /// Checks if tracing is enabled.
    pub fn is_enabled(&self) -> bool {
        self.tracer.is_some()
    }
}

/// A tracing span.
///
/// Represents a unit of work in distributed tracing.
pub struct Span {
    span: Option<opentelemetry_sdk::trace::Span>,
}

impl Span {
    /// Creates a new span (internal use).
    pub(crate) fn new(
        tracer: Option<&opentelemetry_sdk::trace::Tracer>,
        name: &'static str,
    ) -> Self {
        let span = tracer.map(|t| {
            use opentelemetry::trace::Tracer;
            t.start(name)
        });

        Self { span }
    }

    /// Sets an attribute on the span.
    ///
    /// # Arguments
    ///
    /// * `key` - Attribute key
    /// * `value` - Attribute value
    pub fn set_attribute(&mut self, key: &str, value: impl Into<opentelemetry::Value>) {
        if let Some(ref mut span) = self.span {
            use opentelemetry::trace::Span;
            span.set_attribute(KeyValue::new(key.to_string(), value.into()));
        }
    }

    /// Adds an event to the span.
    ///
    /// # Arguments
    ///
    /// * `name` - Event name
    pub fn add_event(&mut self, name: &str) {
        if let Some(ref mut span) = self.span {
            use opentelemetry::trace::Span;
            span.add_event(name.to_string(), vec![]);
        }
    }

    /// Records an error on the span.
    ///
    /// # Arguments
    ///
    /// * `error` - Error to record
    pub fn record_error(&mut self, error: &dyn std::error::Error) {
        if let Some(ref mut span) = self.span {
            use opentelemetry::trace::Span;
            span.record_error(error);
            span.set_status(Status::error(error.to_string()));
        }
    }

    /// Ends the span with success status.
    pub fn end(mut self) {
        if let Some(mut span) = self.span.take() {
            use opentelemetry::trace::Span;
            span.set_status(Status::Ok);
            span.end();
        }
    }
}

impl Drop for Span {
    fn drop(&mut self) {
        if let Some(mut span) = self.span.take() {
            use opentelemetry::trace::Span;
            span.end();
        }
    }
}
