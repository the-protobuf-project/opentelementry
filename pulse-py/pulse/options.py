"""Configuration options for Pulse SDK.

This module defines all configuration dataclasses and options for the Pulse SDK,
including service metadata, telemetry settings, and configuration loading.

Configuration Priority (Lowest to Highest):
1. Defaults (lowest priority)
2. Config file (pulse.toml / pulse.yaml / pulse.json)
3. Environment variables (PULSE_*)
4. Code-based (builder methods) - highest priority

Typical usage example:

    from pulse import Pulse, Environment
    
    # Auto-discovers pulse.toml config file
    pulse = Pulse.new() \\
        .with_service("my-service", "1.0.0") \\
        .environment(Environment.PRODUCTION) \\
        .build()
    
    # Or load from config file only
    service_opts, pulse_opts = from_config()
"""

from dataclasses import dataclass, field
from enum import Enum
from typing import Optional


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
        endpoint: OTLP endpoint (e.g., "localhost:4317" or "otel.example.com").
        auth_token: Bearer token for authentication.
        enabled: Whether OTLP export is enabled.
        secure: Use TLS for connection.
        use_http: Use HTTP instead of gRPC.
        headers: Custom headers for OTLP requests.
    """
    endpoint: str = "localhost:4317"
    auth_token: str = ""
    enabled: bool = False
    secure: bool = False
    use_http: bool = False
    headers: dict = field(default_factory=dict)
    
    @property
    def host(self) -> str:
        """Extract host from endpoint."""
        if ":" in self.endpoint:
            return self.endpoint.split(":")[0]
        return self.endpoint
    
    @property
    def port(self) -> int:
        """Extract port from endpoint."""
        if ":" in self.endpoint:
            try:
                return int(self.endpoint.split(":")[1])
            except (ValueError, IndexError):
                pass
        return 443 if self.secure else 4317


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


def from_config(config_path: Optional[str] = None) -> tuple[ServiceOptions, PulseOptions]:
    """Load Pulse configuration from config file and environment variables.
    
    Uses dynaconf to load configuration with the following priority:
    1. Defaults (lowest priority)
    2. Config file (pulse.toml / pulse.yaml / pulse.json)
    3. Environment variables (PULSE_*)
    
    Auto-discovers config files from:
    1. PULSE_CONFIG_PATH environment variable
    2. pulse.toml in current directory
    3. .config/pulse.toml
    
    Args:
        config_path: Optional path to config file. If not provided,
                    auto-discovers pulse.toml/yaml/json.
    
    Returns:
        A tuple containing (ServiceOptions, PulseOptions) loaded from config.
    
    Example:
        # Auto-discover pulse.toml
        service_opts, pulse_opts = from_config()
        
        # Or specify config path
        service_opts, pulse_opts = from_config("./config/pulse.toml")
    """
    from .config import settings, load_config
    
    # Load config (uses auto-discovery or specified path)
    if config_path:
        cfg = load_config(config_path)
    else:
        cfg = settings
    
    # Map environment string to enum
    env_map = {
        "development": Environment.DEVELOPMENT,
        "staging": Environment.STAGING,
        "production": Environment.PRODUCTION,
        "jetson": Environment.JETSON,
    }
    env_str = cfg.get("service.environment", "development").lower()
    service_environment = env_map.get(env_str, Environment.DEVELOPMENT)
    
    # Service configuration
    service_opts = ServiceOptions(
        name=cfg.get("service.name", "unnamed-service"),
        description=cfg.get("service.description", ""),
        version=cfg.get("service.version", "1.0.0"),
        environment=service_environment,
    )
    
    # OTLP configuration - auto-enable if endpoint is set to non-localhost
    otlp_endpoint = cfg.get("telemetry.otlp.endpoint", "")
    otlp_enabled = cfg.get("telemetry.otlp.enabled", None)
    
    # Auto-enable OTLP if endpoint is configured (and not localhost)
    if otlp_enabled is None:
        otlp_enabled = bool(otlp_endpoint and "localhost" not in otlp_endpoint and "127.0.0.1" not in otlp_endpoint)
    
    otlp_opts = OTLPOptions(
        endpoint=otlp_endpoint or "localhost:4317",
        auth_token=cfg.get("telemetry.otlp.auth_token", ""),
        enabled=otlp_enabled,
        secure=cfg.get("telemetry.otlp.secure", False),
        use_http=cfg.get("telemetry.otlp.use_http", False),
    )
    
    # Logging configuration
    logging_opts = LoggingOptions(
        enabled=True,
        level=cfg.get("logging.log.level", "INFO").upper(),
    )
    
    # Metrics configuration
    metrics_opts = MetricsOptions(
        enabled=cfg.get("telemetry.metrics.enabled", True),
        export_interval_seconds=cfg.get("telemetry.metrics.export_interval_seconds", 10),
    )
    
    # Tracing configuration
    tracing_opts = TracingOptions(
        enabled=cfg.get("tracing.enabled", True),
    )
    
    # Foxglove/MCAP configuration
    foxglove_opts = FoxgloveOptions(
        enabled=cfg.get("foxglove.enabled", False),
        mcap_path=cfg.get("foxglove.file_path", ""),
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


# Keep from_env as alias for backward compatibility
def from_env() -> tuple[ServiceOptions, PulseOptions]:
    """Load Pulse configuration from environment variables (deprecated).
    
    Use from_config() instead which supports both config files and env vars.
    """
    return from_config()
