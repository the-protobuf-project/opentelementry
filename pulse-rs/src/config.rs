use crate::{error::Result, Error};
use url::Url;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Config {
    #[serde(default = "Config::default_service_name")]
    pub service_name: String,

    #[serde(default = "Config::default_service_version")]
    pub service_version: String,

    pub uri: Option<String>,

    #[cfg(feature = "logs")]
    #[serde(default)]
    pub log: crate::log::LogConfig,

    #[cfg(feature = "trace")]
    #[serde(default)]
    pub trace: crate::trace::TraceConfig,

    #[cfg(feature = "metrics")]
    #[serde(default)]
    pub metrics: crate::metrics::MetricsConfig,
}

impl Config {
    fn default_service_name() -> String {
        env!("CARGO_PKG_NAME").to_owned()
    }

    fn default_service_version() -> String {
        env!("CARGO_PKG_VERSION").to_owned()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            service_name: Self::default_service_name(),
            service_version: Self::default_service_version(),
            uri: None,

            #[cfg(feature = "logs")]
            log: Default::default(),

            #[cfg(feature = "trace")]
            trace: Default::default(),

            #[cfg(feature = "metrics")]
            metrics: Default::default(),
        }
    }
}

pub(crate) enum UriScheme {
    Https,
    Http,
    Grpc,
}

pub(crate) trait UrlExt {
    fn supported_scheme(&self) -> Result<UriScheme>;
}

impl UrlExt for Url {
    fn supported_scheme(&self) -> Result<UriScheme> {
        Ok(match self.scheme() {
            "http" => UriScheme::Http,
            "https" => UriScheme::Https,
            "grpc" => UriScheme::Grpc,
            other => return Err(Error::UnsupportedUrlScheme(other.to_owned())),
        })
    }
}
