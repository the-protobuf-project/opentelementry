use crate::{
    config::{Config, UriScheme, UrlExt},
    error::Result,
};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::{runtime, trace::TracerProvider as OtelTracerProvider, Resource};
use tracing::{level_filters::LevelFilter, Subscriber};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{registry::LookupSpan, Layer};
use url::Url;

// rexporting trace macros
pub use {
    tracing::{
        debug, debug_span, error, error_span, info, info_span, trace, trace_span, warn, warn_span,
    },
    tracing_attributes::instrument, // TODO: proc macros have to rexported differently, since if used, tracing crate is required as dep
};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct TraceConfig {
    #[serde(default)]
    pub level: TraceLevel,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum TraceLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<TraceLevel> for tracing::Level {
    fn from(value: TraceLevel) -> Self {
        match value {
            TraceLevel::Error => tracing::Level::ERROR,
            TraceLevel::Warn => tracing::Level::WARN,
            TraceLevel::Info => tracing::Level::INFO,
            TraceLevel::Debug => tracing::Level::DEBUG,
            TraceLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl Default for TraceLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Clone)]
pub(crate) struct TracerProvider {
    level: TraceLevel,
    service_name: String,
    tracer_provider: Option<OtelTracerProvider>,
}

impl TracerProvider {
    pub fn new(otel_resource: Resource, config: &Config) -> Result<Self> {
        let tracer_provider = if let Some(uri) = config.uri.as_ref() {
            let uri = Url::parse(uri)?;
            let exporter = match uri.supported_scheme()? {
                UriScheme::Http | UriScheme::Https => SpanExporter::builder()
                    .with_http()
                    .with_endpoint(uri.to_string())
                    .build()?,
                UriScheme::Grpc => SpanExporter::builder()
                    .with_tonic()
                    .with_endpoint(uri.to_string())
                    .build()?,
            };

            let provider = OtelTracerProvider::builder()
                .with_resource(otel_resource)
                .with_batch_exporter(exporter, runtime::Tokio)
                .build();

            #[cfg(feature = "otel-api")]
            opentelemetry::global::set_tracer_provider(provider.clone());

            Some(provider)
        } else {
            None
        };

        Ok(Self {
            level: config.trace.level,
            service_name: config.service_name.clone(),
            tracer_provider,
        })
    }

    pub(crate) fn layer<S>(&self) -> Option<impl Layer<S>>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        self.tracer_provider
            .as_ref()
            .map(|t| t.tracer(self.service_name.clone()))
            .map(OpenTelemetryLayer::new)
            .map(|l| l.with_filter(LevelFilter::from_level(self.level.into())))
    }
}

impl Drop for TracerProvider {
    fn drop(&mut self) {
        if let Some(tracer_provider) = self.tracer_provider.take() {
            if let Err(e) = tracer_provider.shutdown() {
                eprintln!("failed to shutdown trace provider: {}", e);
            }
        }
    }
}
