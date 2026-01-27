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
//! use pulse::Pulse;
//! use pulse::logger;
//!
//! let _pulse = Pulse::builder("my-service", "1.0.0")
//!     .with_mcap("output.mcap")
//!     .build()
//!     .unwrap();
//!
//! logger::info!("Application started");
//! // Resources are automatically cleaned up when pulse goes out of scope
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
pub use options::Environment;

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
    /// Creates a new builder for configuring Pulse.
    ///
    /// # Arguments
    ///
    /// * `name` - Service name
    /// * `version` - Service version
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pulse::{Pulse, Environment};
    ///
    /// let pulse = Pulse::builder("my-service", "1.0.0")
    ///     .environment(Environment::Production)
    ///     .with_otlp("localhost", 4317)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder(name: impl Into<String>, version: impl Into<String>) -> PulseBuilder {
        PulseBuilder::new(name, version)
    }

    /// Creates a new Pulse instance (legacy API).
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

    /// Manually closes the Pulse instance and cleans up resources.
    ///
    /// This shuts down telemetry providers and closes MCAP writers.
    /// 
    /// **Note**: Resources are automatically cleaned up when Pulse goes out of scope
    /// via the Drop trait. You only need to call this manually if you want explicit
    /// error handling during cleanup.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pulse::{Pulse, options::{ServiceOptions, PulseOptions}};
    ///
    /// let service_opts = ServiceOptions::new("my-service", "1.0.0");
    /// let pulse_opts = PulseOptions::new();
    /// let mut pulse = Pulse::new(service_opts, pulse_opts).unwrap();
    ///
    /// // Use pulse...
    ///
    /// // Optional: Manually close if you need error handling
    /// pulse.close().unwrap();
    /// 
    /// // Otherwise, resources are cleaned up automatically when pulse goes out of scope
    /// ```
    pub fn close(&mut self) -> Result<()> {
        if let Some(t) = self.telemetry.take() {
            let _ = t.shutdown();
        }

        if let Some(writer) = self.mcap_writer.take() {
            let mut w = writer.lock().unwrap();
            w.close()?;
        }

        Ok(())
    }
}

impl Drop for Pulse {
    fn drop(&mut self) {
        if let Some(t) = self.telemetry.take() {
            let _ = t.shutdown();
        }

        if let Some(writer) = self.mcap_writer.take() {
            if let Ok(mut w) = writer.lock() {
                let _ = w.close();
            }
        }
    }
}

/// Builder for configuring and creating a Pulse instance.
///
/// Provides a fluent API for configuring observability options.
///
/// # Examples
///
/// ```no_run
/// use pulse::{Pulse, Environment};
///
/// let pulse = Pulse::builder("my-service", "1.0.0")
///     .description("My awesome service")
///     .environment(Environment::Production)
///     .with_otlp("localhost", 4317)
///     .with_mcap("output.mcap")
///     .build()
///     .unwrap();
/// ```
pub struct PulseBuilder {
    name: String,
    version: String,
    description: Option<String>,
    environment: options::Environment,
    otlp_host: Option<String>,
    otlp_port: Option<u16>,
    mcap_path: Option<String>,
}

impl PulseBuilder {
    /// Creates a new builder with service name and version.
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            environment: options::Environment::Development,
            otlp_host: None,
            otlp_port: None,
            mcap_path: None,
        }
    }

    /// Sets the service description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the deployment environment.
    pub fn environment(mut self, environment: options::Environment) -> Self {
        self.environment = environment;
        self
    }

    /// Enables OpenTelemetry OTLP export.
    ///
    /// # Arguments
    ///
    /// * `host` - OTLP collector host
    /// * `port` - OTLP collector port
    pub fn with_otlp(mut self, host: impl Into<String>, port: u16) -> Self {
        self.otlp_host = Some(host.into());
        self.otlp_port = Some(port);
        self
    }

    /// Enables MCAP recording to the specified file path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the MCAP output file
    pub fn with_mcap(mut self, path: impl Into<String>) -> Self {
        self.mcap_path = Some(path.into());
        self
    }

    /// Builds and initializes the Pulse instance.
    pub fn build(self) -> Result<Pulse> {
        let mut service_opts = options::ServiceOptions::new(&self.name, &self.version)
            .with_environment(self.environment);
        
        if let Some(desc) = self.description {
            service_opts = service_opts.with_description(desc);
        }

        let mut pulse_opts = options::PulseOptions::new();

        // Configure OTLP if specified
        if let (Some(host), Some(port)) = (self.otlp_host, self.otlp_port) {
            pulse_opts.telemetry.otlp.enabled = true;
            pulse_opts.telemetry.otlp.host = host;
            pulse_opts.telemetry.otlp.port = port;
        }

        // Configure MCAP if specified
        if let Some(path) = self.mcap_path {
            pulse_opts.foxglove.enabled = true;
            pulse_opts.foxglove.mcap_path = path;
        }

        Pulse::new(service_opts, pulse_opts)
    }
}
