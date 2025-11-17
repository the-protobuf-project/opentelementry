use crate::{
    config::{Config, UriScheme, UrlExt},
    error::Result,
};
use env_logger::fmt::{
    style::{AnsiColor, Style},
    Formatter,
};
use log::Record;
use opentelemetry_appender_log::OpenTelemetryLogBridge;
use opentelemetry_otlp::{LogExporter, WithExportConfig};
use opentelemetry_sdk::{logs::LoggerProvider as OtelLoggerProvider, runtime, Resource};
use std::io::Write as _;
use url::Url;

// rexporting log macros
pub use log::{debug, error, info, trace, warn};

#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct LogConfig {
    #[serde(default)]
    pub level: LogLevel,

    #[serde(default)]
    pub extra_modules: Vec<String>,
}

#[derive(serde::Deserialize, Debug, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
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

#[derive(Clone)]
pub(crate) struct LoggerProvider {
    service_name: String,
    config: LogConfig,
    otel_log_provider: Option<OtelLoggerProvider>,
}

impl LoggerProvider {
    pub(crate) fn new(otel_resource: Resource, config: &Config) -> Result<Self> {
        let otel_log_provider = if let Some(uri) = config.uri.as_ref() {
            let uri = Url::parse(uri)?;
            let exporter = match uri.supported_scheme()? {
                UriScheme::Https | UriScheme::Http => LogExporter::builder()
                    .with_http()
                    .with_endpoint(uri.to_string())
                    .build()?,
                UriScheme::Grpc => LogExporter::builder()
                    .with_tonic()
                    .with_endpoint(uri.to_string())
                    .build()?,
            };

            Some(
                OtelLoggerProvider::builder()
                    .with_resource(otel_resource)
                    .with_batch_exporter(exporter, runtime::Tokio)
                    .build(),
            )
        } else {
            None
        };

        Ok(Self {
            otel_log_provider,
            service_name: config.service_name.clone(),
            config: config.log.clone(),
        })
    }

    pub(crate) fn logger(&self) -> Result<Box<dyn log::Log>> {
        Logger::new(self, &self.service_name, &self.config)
            .map(|l| Box::new(l) as Box<dyn log::Log>)
    }
}

impl Drop for LoggerProvider {
    fn drop(&mut self) {
        if let Some(otel_log_provider) = self.otel_log_provider.take() {
            if let Err(e) = otel_log_provider.shutdown() {
                eprintln!("failed to shutdown log provider: {}", e);
            }
        }
    }
}

pub(crate) struct Logger {
    otel_logger: Option<Box<dyn log::Log>>,
    std_logger: Box<dyn log::Log>,
}

impl Logger {
    fn new(
        provider: &LoggerProvider,
        service_name: impl AsRef<str>,
        config: &LogConfig,
    ) -> Result<Self> {
        let otel_logger = provider
            .otel_log_provider
            .as_ref()
            .map(|l| Box::new(OpenTelemetryLogBridge::new(l)) as Box<dyn log::Log>);

        let service_name = service_name.as_ref();
        let std_logger = {
            let level: log::Level = config.level.into();
            log::set_max_level(level.to_level_filter());
            let styler = Styler::new(service_name);

            let mut builder = env_logger::Builder::new();
            builder.filter(Some(service_name), level.to_level_filter());
            builder.format(move |buf, record| styler.format(buf, record));

            for module in &config.extra_modules {
                builder.filter(Some(module), level.to_level_filter());
            }
            Box::new(builder.build())
        };

        Ok(Self {
            otel_logger,
            std_logger,
        })
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        self.std_logger.log(record);
        if let Some(ref otel_logger) = self.otel_logger.as_ref() {
            otel_logger.log(record);
        }
    }

    fn flush(&self) {
        self.std_logger.flush();
        if let Some(ref otel_logger) = self.otel_logger.as_ref() {
            otel_logger.flush();
        }
    }
}

struct Styler {
    service_name: String,
    timestamp_style: Style,
    service_name_style: Style,
    error_style: Style,
    warn_style: Style,
    info_style: Style,
    debug_style: Style,
    trace_style: Style,
}

impl Styler {
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            timestamp_style: Style::new()
                .fg_color(Some(AnsiColor::Black.into()))
                .italic(),
            service_name_style: Style::new().fg_color(Some(AnsiColor::Black.into())).bold(),
            error_style: Style::new().fg_color(Some(AnsiColor::Red.into())),
            warn_style: Style::new().fg_color(Some(AnsiColor::Yellow.into())),
            info_style: Style::new().fg_color(Some(AnsiColor::Green.into())),
            debug_style: Style::new().fg_color(Some(AnsiColor::Blue.into())),
            trace_style: Style::new().fg_color(Some(AnsiColor::BrightWhite.into())),
        }
    }

    pub fn format(
        &self,
        buf: &mut Formatter,
        record: &Record,
    ) -> std::result::Result<(), std::io::Error> {
        let timestamp_style = &self.timestamp_style;
        let level_style = match record.level() {
            log::Level::Error => &self.error_style,
            log::Level::Warn => &self.warn_style,
            log::Level::Info => &self.info_style,
            log::Level::Debug => &self.debug_style,
            log::Level::Trace => &self.trace_style,
        };
        let service_name_style = &self.service_name_style;
        let loc = record
            .module_path()
            .map(|p| format!("[{p}]"))
            .unwrap_or_default();

        writeln!(
            buf,
            "{timestamp_style}{}{timestamp_style:#} [{service_name_style}{}{service_name_style:#}:{level_style}{:5}{level_style:#}] {} {}",
            buf.timestamp(),
            &self.service_name,
            record.level(),
            loc,
            record.args(),
        )
    }
}
