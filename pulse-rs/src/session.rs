use crate::{error::Result, Config};
use once_cell::sync::OnceCell;
use opentelemetry::KeyValue;
use opentelemetry_sdk::Resource;

static _SESSION: OnceCell<Session> = OnceCell::new();

pub fn init(config: &Config) -> Result<Session> {
    _SESSION.get_or_try_init(|| Session::new(config)).cloned()
}

#[derive(Clone)]
pub struct Session {
    pub otel_resource: Resource,

    #[cfg(feature = "logs")]
    _logger_provider: crate::log::LoggerProvider,

    #[cfg(feature = "trace")]
    _tracer_provider: crate::trace::TracerProvider,

    #[cfg(feature = "metrics")]
    _metrics_provider: crate::metrics::MetricsProvider,
}

impl Session {
    #[allow(unused_variables)]
    pub(crate) fn new(config: &Config) -> Result<Self> {
        let otel_resource = Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                config.service_name.clone(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                config.service_version.clone(),
            ),
        ]);

        #[cfg(feature = "logs")]
        let logger_provider = {
            let provider = crate::log::LoggerProvider::new(otel_resource.clone(), config)?;
            log::set_boxed_logger(provider.logger()?)?;
            provider
        };

        #[cfg(feature = "trace")]
        let tracer_provider = crate::trace::TracerProvider::new(otel_resource.clone(), config)?;

        #[cfg(feature = "metrics")]
        let metrics_provider = crate::metrics::MetricsProvider::new(otel_resource.clone(), config)?;

        #[cfg(any(feature = "trace", feature = "metrics"))]
        {
            use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

            let registry = tracing_subscriber::registry();

            #[cfg(feature = "metrics")]
            let registry = registry.with(metrics_provider.layer());

            #[cfg(feature = "trace")]
            let registry = registry.with(tracer_provider.layer());

            registry.init()
        }

        Ok(Self {
            otel_resource: otel_resource.clone(),

            #[cfg(feature = "logs")]
            _logger_provider: logger_provider,

            #[cfg(feature = "trace")]
            _tracer_provider: tracer_provider,

            #[cfg(feature = "metrics")]
            _metrics_provider: metrics_provider,
        })
    }
}
