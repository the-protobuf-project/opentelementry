//! Pulse - Unified observability library for robotics and distributed systems.
//!
//! Pulse provides integrated logging, metrics, and tracing with support for
//! multiple backends including console output, MCAP files (Foxglove), and
//! OpenTelemetry.
//!
//! # Features
//!
//! - **Logging**: Structured logging with colored console output, MCAP recording,
//!   and OpenTelemetry integration
//! - **Metrics**: Counter, histogram, and gauge metrics with derive macro support
//! - **Tracing**: Distributed tracing with OpenTelemetry
//! - **MCAP Recording**: Record logs, metrics, and traces to MCAP files for
//!   visualization in Foxglove Studio
//!
//! # Examples
//!
//! ```no_run
//! use pulse::{Pulse, options::{ServiceOptions, PulseOptions, FoxgloveOptions}};
//! use pulse::info;
//!
//! let service_opts = ServiceOptions::new("my-service", "1.0.0");
//! let pulse_opts = PulseOptions::new()
//!     .with_foxglove(FoxgloveOptions::new("output.mcap"));
//!
//! let pulse = Pulse::new(service_opts, pulse_opts).unwrap();
//!
//! info!("Application started");
//! pulse.metrics.counter("requests_total", 1.0).unwrap();
//!
//! pulse.close().unwrap();
//! ```

pub mod options;
pub mod foxglove;
pub mod logging;
pub mod telemetry;
pub mod metrics;
pub mod traits;
pub mod derive;
pub mod tracing;

use anyhow::Result;
use std::sync::{Arc, Mutex};

pub use logging::Logger;
pub use logging::global as logger;
pub use metrics::Metrics;

/// Main Pulse instance that manages all observability components.
///
/// This struct provides access to logging, metrics, and tracing functionality,
/// and manages the lifecycle of MCAP writers and telemetry providers.
pub struct Pulse {
    pub logger: Logger,
    pub tracing: Option<tracing::PulseTracing>,
    pub metrics: Metrics,
    mcap_writer: Option<Arc<Mutex<foxglove::UnifiedMcapWriter>>>,
    telemetry: Option<telemetry::TelemetryProvider>,
}

impl Pulse {
    /// Creates a new Pulse instance.
    ///
    /// Initializes all observability components based on the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `service_opts` - Service configuration (name, version, environment)
    /// * `pulse_opts` - Pulse configuration (telemetry, Foxglove options)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pulse::{Pulse, options::{ServiceOptions, PulseOptions}};
    ///
    /// let service_opts = ServiceOptions::new("my-service", "1.0.0");
    /// let pulse_opts = PulseOptions::new();
    /// let pulse = Pulse::new(service_opts, pulse_opts).unwrap();
    /// ```
    pub fn new(service_opts: options::ServiceOptions, pulse_opts: options::PulseOptions) -> Result<Self> {
        let formatter = logging::PulseFormatter::new();
        formatter.set_service_info(
            service_opts.name.clone(),
            service_opts.version.clone(),
            service_opts.environment.to_string(),
        );

        // Set default log level to INFO to hide TRACE/DEBUG from dependencies
        // Can be overridden with RUST_LOG environment variable
        let default_level = std::env::var("RUST_LOG")
            .ok()
            .and_then(|s| s.parse::<log::LevelFilter>().ok())
            .unwrap_or(log::LevelFilter::Info);

        let _ = log4rs::init_file("log4rs.yaml", Default::default())
            .or_else(|_| {
                let stdout = log4rs::append::console::ConsoleAppender::builder()
                    .encoder(Box::new(formatter))
                    .build();
                let config = log4rs::config::Config::builder()
                    .appender(log4rs::config::Appender::builder().build("stdout", Box::new(stdout)))
                    .build(log4rs::config::Root::builder().appender("stdout").build(default_level))
                    .unwrap();
                log4rs::init_config(config).map(|_| ())
            });

        let mcap_writer = if pulse_opts.foxglove.enabled && !pulse_opts.foxglove.mcap_path.is_empty() {
            let writer = foxglove::UnifiedMcapWriter::new(&service_opts, &pulse_opts.foxglove.mcap_path)?;
            Some(Arc::new(Mutex::new(writer)))
        } else {
            None
        };

        let mcap_log_writer = mcap_writer.as_ref().map(|writer| {
            logging::LogMcapWriter::new(&service_opts, Arc::clone(writer))
        }).transpose()?;

        let telemetry = telemetry::TelemetryProvider::new(&service_opts, &pulse_opts.telemetry).ok();
        let otel_logger = telemetry.as_ref().and_then(|t| t.get_logger("pulse"));

        let logger = Logger::new(
            service_opts.name.clone(),
            service_opts.version.clone(),
            service_opts.environment.to_string(),
            mcap_log_writer,
            otel_logger,
        );

        let global_logger = logging::GlobalLogger::new(
            service_opts.name.clone(),
            service_opts.version.clone(),
            service_opts.environment.to_string(),
            logger.mcap_writer_arc(),
            logger.otel_logger_arc(),
        );
        logging::init(global_logger);

        // Initialize tokio-rs/tracing with OpenTelemetry if OTLP is enabled
        if pulse_opts.telemetry.otlp.enabled {
            let _ = tracing::init_tokio_tracing(&service_opts);
        }

        // Initialize PulseTracing for manual span management if OTLP is enabled
        let tracing_instance = if pulse_opts.telemetry.otlp.enabled {
            let endpoint = format!("http://{}:{}", pulse_opts.telemetry.otlp.host, pulse_opts.telemetry.otlp.port);
            tracing::PulseTracing::new(&service_opts, Some(endpoint)).ok()
        } else {
            None
        };

        // Initialize metrics
        let metrics = metrics::Metrics::new(
            service_opts.clone(),
            mcap_writer.clone(),
            telemetry.as_ref().and_then(|t| t.meter_provider()),
        )?;

        Ok(Self {
            logger,
            tracing: tracing_instance,
            metrics,
            mcap_writer,
            telemetry,
        })
    }

    /// Flushes all pending telemetry data.
    ///
    /// This should be called before shutting down to ensure all data is sent.
    pub fn flush(&self) {
        if let Some(ref t) = self.telemetry {
            t.flush();
        }
    }

    /// Returns a reference to the MCAP writer if configured.
    pub fn mcap_writer(&self) -> Option<Arc<Mutex<foxglove::UnifiedMcapWriter>>> {
        self.mcap_writer.clone()
    }

    /// Returns the OpenTelemetry meter provider if configured.
    pub fn meter_provider(&self) -> Option<Arc<opentelemetry_sdk::metrics::SdkMeterProvider>> {
        self.telemetry.as_ref().and_then(|t| t.meter_provider())
    }

    /// Closes the Pulse instance and cleans up resources.
    ///
    /// This shuts down telemetry providers and closes MCAP writers.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pulse::{Pulse, options::{ServiceOptions, PulseOptions}};
    ///
    /// let service_opts = ServiceOptions::new("my-service", "1.0.0");
    /// let pulse_opts = PulseOptions::new();
    /// let pulse = Pulse::new(service_opts, pulse_opts).unwrap();
    ///
    /// // Use pulse...
    ///
    /// pulse.close().unwrap();
    /// ```
    pub fn close(self) -> Result<()> {
        if let Some(t) = self.telemetry {
            let _ = t.shutdown();
        }

        if let Some(writer) = self.mcap_writer {
            let mut w = writer.lock().unwrap();
            w.close()?;
        }

        Ok(())
    }
}
