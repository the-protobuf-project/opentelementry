"""Configuration options for Pulse SDK.

This module defines all configuration dataclasses and options for the Pulse SDK,
including service metadata, telemetry settings, and environment-based configuration
loading.

Typical usage example:

    from pulse import ServiceOptions, PulseOptions, Environment
    
    service_opts = ServiceOptions(
        name="my-service",
        version="1.0.0",
        environment=Environment.PRODUCTION
    )
    
    pulse_opts = PulseOptions()
    pulse = Pulse(service_opts, pulse_opts)
    
    # Or load from environment variables
    service_opts, pulse_opts = from_env()
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional
import os


class Environment(str, Enum):
    """Deployment environment enumeration.
    
    Defines the possible deployment environments for a service.
    The environment affects default log levels and other behavior.
    
    Attributes:
        DEVELOPMENT: Development environment (verbose logging).
        STAGING: Staging/testing environment.
        PRODUCTION: Production environment (minimal logging).
        JETSON: NVIDIA Jetson device environment.
    """
    DEVELOPMENT = "development"
    STAGING = "staging"
    PRODUCTION = "production"
    JETSON = "jetson"


@dataclass
class OTLPOptions:
    """OpenTelemetry Protocol (OTLP) exporter configuration.
    
    Configures the OTLP exporter for sending telemetry data (logs, metrics, traces)
    to an OpenTelemetry collector.
    
    Attributes:
        host: OTLP collector hostname or IP address.
        port: OTLP collector gRPC port.
        enabled: Whether OTLP export is enabled.
    """
    host: str = "localhost"
    port: int = 4317
    enabled: bool = False


@dataclass
class LoggingOptions:
    """Logging configuration options.
    
    Configures the logging system including log levels and output.
    
    Attributes:
        enabled: Whether logging is enabled.
        level: Log level string. Can be: DEBUG, INFO, WARNING, ERROR, CRITICAL.
    """
    enabled: bool = True
    level: str = "INFO"


@dataclass
class MetricsOptions:
    """Metrics collection and export configuration.
    
    Configures the metrics system including collection and export intervals.
    
    Attributes:
        enabled: Whether metrics collection is enabled.
        export_interval_seconds: How often to export metrics to OTLP collector.
    """
    enabled: bool = True
    export_interval_seconds: int = 10


@dataclass
class TracingOptions:
    """Distributed tracing configuration.
    
    Configures the distributed tracing system for tracking requests across services.
    
    Attributes:
        enabled: Whether distributed tracing is enabled.
    """
    enabled: bool = True


@dataclass
class FoxgloveOptions:
    """Foxglove Studio MCAP recording configuration.
    
    Configures MCAP file recording for visualization in Foxglove Studio.
    MCAP files contain logs, metrics, and traces in a unified format.
    
    Attributes:
        enabled: Whether MCAP recording is enabled.
        mcap_path: File path where MCAP file will be written.
    """
    enabled: bool = False
    mcap_path: str = ""


@dataclass
class TelemetryOptions:
    """Unified telemetry configuration.
    
    Aggregates all telemetry-related configuration options including logging,
    metrics, tracing, and OTLP export.
    
    Attributes:
        logging: Logging configuration options.
        metrics: Metrics configuration options.
        tracing: Tracing configuration options.
        otlp: OTLP exporter configuration.
    """
    logging: LoggingOptions = field(default_factory=LoggingOptions)
    metrics: MetricsOptions = field(default_factory=MetricsOptions)
    tracing: TracingOptions = field(default_factory=TracingOptions)
    otlp: OTLPOptions = field(default_factory=OTLPOptions)


@dataclass
class ServiceOptions:
    """Service metadata and configuration.
    
    Defines the service identity and deployment environment. This information
    is included in all telemetry data for service identification.
    
    Attributes:
        name: Service name (required). Used in logs, metrics, and traces.
        description: Optional service description.
        version: Service version string.
        environment: Deployment environment (development, staging, production, jetson).
    """
    name: str
    description: str = ""
    version: str = "1.0.0"
    environment: Environment = Environment.DEVELOPMENT


@dataclass
class PulseOptions:
    """Main Pulse SDK configuration.
    
    Top-level configuration for the Pulse SDK, aggregating all subsystem options.
    
    Attributes:
        telemetry: Unified telemetry configuration (logging, metrics, tracing, OTLP).
        foxglove: Foxglove Studio MCAP recording configuration.
        logging: Optional override for logging configuration.
        tracing: Optional override for tracing configuration.
    """
    telemetry: TelemetryOptions = field(default_factory=TelemetryOptions)
    foxglove: FoxgloveOptions = field(default_factory=FoxgloveOptions)
    logging: Optional[LoggingOptions] = None
    tracing: Optional[TracingOptions] = None


def from_env() -> tuple[ServiceOptions, PulseOptions]:
    """Load Pulse configuration from environment variables.
    
    Reads configuration from environment variables and constructs ServiceOptions
    and PulseOptions objects. This is the recommended way to configure Pulse
    in containerized or cloud environments.
    
    Environment variables:
        SERVICE_NAME: Service name (default: "unnamed-service").
        SERVICE_VERSION: Service version (default: "1.0.0").
        SERVICE_DESCRIPTION: Service description (default: "").
        SERVICE_ENVIRONMENT: Environment - development, staging, production, jetson
            (default: "development").
        LOG_LEVEL: Log level - DEBUG, INFO, WARNING, ERROR, CRITICAL
            (default: "INFO").
        OTLP_ENABLED: Enable OTLP export - true/false (default: "false").
        OTLP_HOST: OTLP collector hostname (default: "localhost").
        OTLP_PORT: OTLP collector port (default: "4317").
        MCAP_ENABLED: Enable MCAP recording - true/false (default: "false").
        MCAP_PATH: MCAP file path (default: "").
        METRICS_ENABLED: Enable metrics - true/false (default: "true").
        METRICS_EXPORT_INTERVAL: Metrics export interval in seconds (default: "10").
        TRACING_ENABLED: Enable tracing - true/false (default: "true").
    
    Returns:
        A tuple containing (ServiceOptions, PulseOptions) loaded from environment.
    
    Example:
        # Set environment variables
        os.environ["SERVICE_NAME"] = "my-service"
        os.environ["LOG_LEVEL"] = "DEBUG"
        os.environ["OTLP_ENABLED"] = "true"
        
        # Load configuration
        service_opts, pulse_opts = from_env()
        pulse = Pulse(service_opts, pulse_opts)
    """
    # Service configuration
    service_name = os.getenv("SERVICE_NAME", "unnamed-service")
    service_version = os.getenv("SERVICE_VERSION", "1.0.0")
    service_env_str = os.getenv("SERVICE_ENVIRONMENT", "development").lower()
    
    # Map environment string to enum
    env_map = {
        "development": Environment.DEVELOPMENT,
        "staging": Environment.STAGING,
        "production": Environment.PRODUCTION,
        "jetson": Environment.JETSON,
    }
    service_environment = env_map.get(service_env_str, Environment.DEVELOPMENT)
    
    service_opts = ServiceOptions(
        name=service_name,
        description=os.getenv("SERVICE_DESCRIPTION", ""),
        version=service_version,
        environment=service_environment,
    )
    
    # OTLP configuration
    otlp_opts = OTLPOptions(
        host=os.getenv("OTLP_HOST", "localhost"),
        port=int(os.getenv("OTLP_PORT", "4317")),
        enabled=os.getenv("OTLP_ENABLED", "false").lower() == "true",
    )
    
    # Logging configuration
    logging_opts = LoggingOptions(
        enabled=True,
        level=os.getenv("LOG_LEVEL", "INFO").upper(),
    )
    
    # Metrics configuration
    metrics_opts = MetricsOptions(
        enabled=os.getenv("METRICS_ENABLED", "true").lower() == "true",
        export_interval_seconds=int(os.getenv("METRICS_EXPORT_INTERVAL", "10")),
    )
    
    # Tracing configuration
    tracing_opts = TracingOptions(
        enabled=os.getenv("TRACING_ENABLED", "true").lower() == "true",
    )
    
    # Foxglove/MCAP configuration
    foxglove_opts = FoxgloveOptions(
        enabled=os.getenv("MCAP_ENABLED", "false").lower() == "true",
        mcap_path=os.getenv("MCAP_PATH", ""),
    )
    
    # Build telemetry options
    telemetry_opts = TelemetryOptions(
        logging=logging_opts,
        metrics=metrics_opts,
        tracing=tracing_opts,
        otlp=otlp_opts,
    )
    
    pulse_opts = PulseOptions(
        telemetry=telemetry_opts,
        foxglove=foxglove_opts,
    )
    
    return service_opts, pulse_opts
