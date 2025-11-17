use crate::Result;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Options {
    pub service_name: String,
    pub service_version: String,
    pub log_level: LogLevel,
    pub network: NetworkOptions,
}

impl Options {
    pub fn from_default() -> Result<Self> {
        Ok(Figment::new()
            .merge::<OptionsSource>(Options::default().into())
            .merge(Toml::file("pulse.toml"))
            .merge(Env::prefixed("PULSE_").split("."))
            .extract()?)
    }

    pub fn merge_with(options: Options) -> Result<Self> {
        Ok(Figment::new()
            .merge(Toml::file("pulse.toml"))
            .merge::<OptionsSource>(options.into())
            .merge(Env::prefixed("PULSE_").split("."))
            .extract()?)
    }
}

impl Default for Options {
    fn default() -> Self {
        Self {
            service_name: env!("CARGO_PKG_NAME").to_owned(),
            service_version: env!("CARGO_PKG_VERSION").to_owned(),
            log_level: LogLevel::Info,
            network: Default::default(),
        }
    }
}

#[derive(Clone, Debug)]
struct OptionsSource {
    options: Options,
}

impl figment::Provider for OptionsSource {
    fn metadata(&self) -> figment::Metadata {
        figment::Metadata::named("Options struct")
    }

    fn data(
        &self,
    ) -> std::result::Result<
        figment::value::Map<figment::Profile, figment::value::Dict>,
        figment::Error,
    > {
        let s = serde_json::to_string(&self.options)
            .map_err(|e| figment::Error::from(e.to_string()))?;
        let entries = serde_json::from_str(&s).map_err(|e| figment::Error::from(e.to_string()))?;

        Ok(figment::value::Map::from([(
            figment::Profile::Default,
            entries,
        )]))
    }
}

impl From<Options> for OptionsSource {
    fn from(value: Options) -> Self {
        Self { options: value }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct NetworkOptions {
    pub host: String,
    pub port: u16,
    pub advanced_opts: AdvancedNetworkOptions,
}

impl NetworkOptions {
    pub fn uri(&self) -> String {
        format!(
            "{}://{}:{}",
            self.advanced_opts.connection_type.as_ref(),
            self.host,
            self.port
        )
    }
}

impl Default for NetworkOptions {
    fn default() -> Self {
        Self {
            host: "localhost".to_owned(),
            port: 4317,
            advanced_opts: Default::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AdvancedNetworkOptions {
    pub connection_type: ConnectionType,
}

impl Default for AdvancedNetworkOptions {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Grpc,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionType {
    Grpc,
    Http,
}

impl AsRef<str> for ConnectionType {
    fn as_ref(&self) -> &str {
        match self {
            ConnectionType::Grpc => "grpc",
            ConnectionType::Http => "http",
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LogLevel> for tracing::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => tracing::Level::ERROR,
            LogLevel::Warn => tracing::Level::WARN,
            LogLevel::Info => tracing::Level::INFO,
            LogLevel::Debug => tracing::Level::DEBUG,
            LogLevel::Trace => tracing::Level::TRACE,
        }
    }
}

impl From<LogLevel> for log::Level {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Trace => log::Level::Trace,
        }
    }
}
