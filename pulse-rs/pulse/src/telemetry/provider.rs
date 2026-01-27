//! OpenTelemetry telemetry provider.
//!
//! This module provides a unified telemetry provider that manages
//! OpenTelemetry logging and metrics exporters.

use anyhow::Result;
use std::sync::Arc;
use crate::options::{ServiceOptions, TelemetryOptions};
use crate::logging::OtelLogger;
use opentelemetry::logs::LoggerProvider as _;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{LogExporter, MetricExporter};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::Resource;

/// Telemetry provider for OpenTelemetry integration.
///
/// Manages logger and meter providers for sending telemetry data
/// to OpenTelemetry collectors via OTLP.
pub struct TelemetryProvider {
    logger_provider: Option<SdkLoggerProvider>,
    meter_provider: Option<Arc<SdkMeterProvider>>,
}

impl TelemetryProvider {
    /// Creates a new telemetry provider.
    ///
    /// # Arguments
    ///
    /// * `service_opts` - Service configuration
    /// * `telemetry_opts` - Telemetry configuration
    pub fn new(
        service_opts: &ServiceOptions,
        telemetry_opts: &TelemetryOptions,
    ) -> Result<Self> {
        let logger_provider = if telemetry_opts.otlp.enabled {
            let exporter = LogExporter::builder()
                .with_tonic()
                .with_endpoint(telemetry_opts.otlp.endpoint())
                .build()?;

            let resource = Resource::builder()
                .with_service_name(service_opts.name.clone())
                .with_attributes([
                    KeyValue::new("service.version", service_opts.version.clone()),
                    KeyValue::new("deployment.environment", service_opts.environment.to_string()),
                ])
                .build();

            Some(
                SdkLoggerProvider::builder()
                    .with_batch_exporter(exporter)
                    .with_resource(resource)
                    .build(),
            )
        } else {
            None
        };

        let meter_provider = if telemetry_opts.otlp.enabled {
            let exporter = MetricExporter::builder()
                .with_tonic()
                .with_endpoint(telemetry_opts.otlp.endpoint())
                .build()?;

            let resource = Resource::builder()
                .with_service_name(service_opts.name.clone())
                .with_attributes([
                    KeyValue::new("service.version", service_opts.version.clone()),
                    KeyValue::new("deployment.environment", service_opts.environment.to_string()),
                ])
                .build();

            Some(Arc::new(
                SdkMeterProvider::builder()
                    .with_periodic_exporter(exporter)
                    .with_resource(resource)
                    .build(),
            ))
        } else {
            None
        };

        Ok(Self { logger_provider, meter_provider })
    }

    /// Gets a logger instance with the specified name.
    ///
    /// # Arguments
    ///
    /// * `name` - Logger name
    pub fn get_logger(&self, name: &str) -> Option<OtelLogger> {
        self.logger_provider
            .as_ref()
            .map(|provider| OtelLogger::new(provider.logger(name.to_string())))
    }

    /// Returns the meter provider for creating metrics.
    pub fn meter_provider(&self) -> Option<Arc<SdkMeterProvider>> {
        self.meter_provider.clone()
    }

    /// Flushes all pending telemetry data.
    pub fn flush(&self) {
        if let Some(ref provider) = self.logger_provider {
            let _ = provider.force_flush();
        }
    }

    /// Shuts down the telemetry provider and flushes remaining data.
    pub fn shutdown(self) -> Result<()> {
        if let Some(provider) = self.logger_provider {
            let _ = provider.force_flush();
            let _ = provider.shutdown();
        }
        Ok(())
    }
}
