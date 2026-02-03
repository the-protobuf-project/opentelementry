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
use opentelemetry_otlp::{LogExporter, MetricExporter, WithTonicConfig, WithHttpConfig};
use opentelemetry_otlp::WithExportConfig;
use tonic::metadata::{MetadataKey, MetadataMap, MetadataValue};
use std::collections::HashMap;
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
            let exporter = if telemetry_opts.otlp.use_http {
                // Use HTTP protocol
                let mut headers = HashMap::new();
                if let Some(ref token) = telemetry_opts.otlp.auth_token {
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                }
                for (key, value) in &telemetry_opts.otlp.headers {
                    headers.insert(key.clone(), value.clone());
                }

                LogExporter::builder()
                    .with_http()
                    .with_endpoint(format!("{}/v1/logs", telemetry_opts.otlp.endpoint_url()))
                    .with_headers(headers)
                    .build()?
            } else {
                // Use gRPC protocol
                let mut metadata = MetadataMap::new();
                if let Some(ref token) = telemetry_opts.otlp.auth_token {
                    metadata.insert("authorization", format!("Bearer {}", token).parse().unwrap());
                }
                for (key, value) in &telemetry_opts.otlp.headers {
                    if let (Ok(k), Ok(v)) = (
                        key.parse::<MetadataKey<_>>(),
                        value.parse::<MetadataValue<_>>()
                    ) {
                        metadata.insert(k, v);
                    }
                }

                LogExporter::builder()
                    .with_tonic()
                    .with_endpoint(telemetry_opts.otlp.endpoint_url())
                    .with_metadata(metadata)
                    .build()?
            };

            // Build resource attributes including custom service attributes
            let mut attrs = vec![
                KeyValue::new("service.version", service_opts.version.clone()),
                KeyValue::new("deployment.environment", service_opts.environment.to_string()),
            ];
            for (key, value) in &service_opts.attributes {
                attrs.push(KeyValue::new(key.clone(), value.clone()));
            }

            let resource = Resource::builder()
                .with_service_name(service_opts.name.clone())
                .with_attributes(attrs)
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
            let exporter = if telemetry_opts.otlp.use_http {
                // Use HTTP protocol
                let mut headers = HashMap::new();
                if let Some(ref token) = telemetry_opts.otlp.auth_token {
                    headers.insert("Authorization".to_string(), format!("Bearer {}", token));
                }
                for (key, value) in &telemetry_opts.otlp.headers {
                    headers.insert(key.clone(), value.clone());
                }

                MetricExporter::builder()
                    .with_http()
                    .with_endpoint(format!("{}/v1/metrics", telemetry_opts.otlp.endpoint_url()))
                    .with_headers(headers)
                    .build()?
            } else {
                // Use gRPC protocol
                let mut metadata = MetadataMap::new();
                if let Some(ref token) = telemetry_opts.otlp.auth_token {
                    metadata.insert("authorization", format!("Bearer {}", token).parse().unwrap());
                }
                for (key, value) in &telemetry_opts.otlp.headers {
                    if let (Ok(k), Ok(v)) = (
                        key.parse::<MetadataKey<_>>(),
                        value.parse::<MetadataValue<_>>()
                    ) {
                        metadata.insert(k, v);
                    }
                }

                MetricExporter::builder()
                    .with_tonic()
                    .with_endpoint(telemetry_opts.otlp.endpoint_url())
                    .with_metadata(metadata)
                    .build()?
            };

            // Build resource attributes including custom service attributes
            let mut attrs = vec![
                KeyValue::new("service.version", service_opts.version.clone()),
                KeyValue::new("deployment.environment", service_opts.environment.to_string()),
            ];
            for (key, value) in &service_opts.attributes {
                attrs.push(KeyValue::new(key.clone(), value.clone()));
            }

            let resource = Resource::builder()
                .with_service_name(service_opts.name.clone())
                .with_attributes(attrs)
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
