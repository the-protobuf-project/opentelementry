pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    InvalidUrl(#[from] url::ParseError),

    #[error("unsupported url scheme: {0}")]
    UnsupportedUrlScheme(String),

    #[cfg(feature = "logs")]
    #[error("{0}")]
    SetLoggerError(#[from] log::SetLoggerError),

    #[cfg(feature = "logs")]
    #[error("{0}")]
    OtelLog(#[from] opentelemetry_sdk::logs::LogError),

    #[cfg(feature = "trace")]
    #[error("{0}")]
    OtelTrace(#[from] opentelemetry::trace::TraceError),

    #[cfg(feature = "metrics")]
    #[error("{0}")]
    OtelMetric(#[from] opentelemetry_sdk::metrics::MetricError),
}
