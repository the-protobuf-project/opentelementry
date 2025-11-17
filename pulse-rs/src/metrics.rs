use crate::{
    config::{Config, UriScheme, UrlExt},
    error::Result,
};
use opentelemetry_otlp::{MetricExporter, WithExportConfig};
use opentelemetry_sdk::{
    metrics::{PeriodicReader, SdkMeterProvider as OtelMeterProvider},
    runtime, Resource,
};
use std::time::Duration;
use tracing::Subscriber;
use tracing_opentelemetry::MetricsLayer;
use tracing_subscriber::{registry::LookupSpan, Layer};
use url::Url;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct MetricsConfig {
    #[serde(deserialize_with = "duration_str::deserialize_duration")]
    #[serde(default = "MetricsConfig::default_export_interval")]
    pub export_interval: Duration,
}

impl MetricsConfig {
    fn default_export_interval() -> Duration {
        Duration::from_secs(1)
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            export_interval: Self::default_export_interval(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct MetricsProvider {
    metrics_provider: Option<OtelMeterProvider>,
}

impl MetricsProvider {
    pub(crate) fn new(otel_resource: Resource, config: &Config) -> Result<Self> {
        let metrics_provider = if let Some(uri) = config.uri.as_ref() {
            let uri = Url::parse(uri)?;
            let exporter = match uri.supported_scheme()? {
                UriScheme::Https | UriScheme::Http => MetricExporter::builder()
                    .with_http()
                    .with_endpoint(uri.to_string())
                    .build()?,
                UriScheme::Grpc => MetricExporter::builder()
                    .with_tonic()
                    .with_endpoint(uri.to_string())
                    .build()?,
            };

            let reader = PeriodicReader::builder(exporter, runtime::Tokio)
                .with_interval(config.metrics.export_interval)
                .build();

            let provider = OtelMeterProvider::builder()
                .with_resource(otel_resource)
                .with_reader(reader)
                .build();

            #[cfg(feature = "otel-api")]
            opentelemetry::global::set_meter_provider(provider.clone());

            Some(provider)
        } else {
            None
        };

        Ok(Self { metrics_provider })
    }

    pub(crate) fn layer<S>(&self) -> Option<impl Layer<S>>
    where
        S: Subscriber + for<'span> LookupSpan<'span>,
    {
        self.metrics_provider
            .as_ref()
            .map(|p| MetricsLayer::new(p.clone()))
    }
}

impl Drop for MetricsProvider {
    fn drop(&mut self) {
        if let Some(metrics_provider) = self.metrics_provider.take() {
            if let Err(e) = metrics_provider.shutdown() {
                eprintln!("failed to shutdown metrics provider: {}", e);
            }
        }
    }
}

// record macro uses tracing::info! since metric's don't have levels associated with it
// so recording a metric using the info macro can be confusing. internally, the `MetricsLayer`
// ignores the level.
#[macro_export]
macro_rules! record {
    ($($args:tt)*) => {
        pulse_rs::trace::info!($($args)*)
    }
}

pub use record;
