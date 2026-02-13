from typing import Optional, Dict

from .options import (
    ServiceOptions,
    PulseOptions,
    Environment,
    LogLevel,
    ModuleOptions,
    from_config,
)
from ._private.logging import PulseLogger
from ._private.metrics import (
    PulseMetrics,
    set_current_pulse_metrics,
    reset_current_pulse_metrics,
)
from ._private.tracing import PulseTracing, set_current_pulse, reset_current_pulse
from ._private.foxglove import UnifiedMcapWriter


class PulseBuilder:
    """Builder for creating Pulse instances with fluent API.

    Configuration Priority (Lowest to Highest):
    1. Defaults (lowest priority)
    2. Config file (pulse.toml / pulse.yaml / pulse.json)
    3. Environment variables (PULSE_*)
    4. Code-based (builder methods) - highest priority

    Example:
        pulse = Pulse.new() \\
            .with_service("my-service", "1.0.0") \\
            .environment(Environment.PRODUCTION) \\
            .with_otlp("localhost", 4317) \\
            .build()
    """

    def __init__(self):
        self._config_path: Optional[str] = None
        self._name: Optional[str] = None
        self._version: Optional[str] = None
        self._description: Optional[str] = None
        self._environment: Optional[Environment] = None
        self._labels: Dict[str, str] = {}
        self._otlp_endpoint: Optional[str] = None
        self._otlp_auth_token: Optional[str] = None
        self._otlp_secure: Optional[bool] = None
        self._otlp_use_http: Optional[bool] = None
        self._mcap_path: Optional[str] = None
        self._tracing_enabled: bool = False
        self._log_level: Optional[LogLevel] = None
        self._service_from_code: bool = False

    def with_config(self, config_path: str) -> "PulseBuilder":
        """Load configuration from a specific file path."""
        self._config_path = config_path
        return self

    def with_service(self, name: str, version: str) -> "PulseBuilder":
        """Set service name and version.

        When this is called, it indicates the user wants to configure service via code,
        so we'll clear any service-level configuration from the config file to avoid collisions.
        """
        self._name = name
        self._version = version
        # Mark that service should be configured via code only
        self._service_from_code = True
        return self

    def description(self, desc: str) -> "PulseBuilder":
        """Set service description."""
        self._description = desc
        return self

    def environment(self, env: Environment) -> "PulseBuilder":
        """Set deployment environment."""
        self._environment = env
        return self

    def with_label(self, key: str, value: str) -> "PulseBuilder":
        """Add a custom label to all telemetry."""
        self._labels[key] = value
        return self

    def with_labels(self, labels: Dict[str, str]) -> "PulseBuilder":
        """Add multiple custom labels to all telemetry."""
        self._labels.update(labels)
        return self

    def with_otlp(self, host: str, port: int) -> "PulseBuilder":
        """Enable OTLP export to the specified endpoint."""
        self._otlp_endpoint = f"{host}:{port}"
        return self

    def with_otlp_endpoint(self, endpoint: str) -> "PulseBuilder":
        """Enable OTLP export to the specified endpoint string."""
        self._otlp_endpoint = endpoint
        return self

    def with_otlp_auth(self, token: str) -> "PulseBuilder":
        """Set OTLP authentication token."""
        self._otlp_auth_token = token
        return self

    def with_otlp_secure(self, secure: bool = True) -> "PulseBuilder":
        """Enable/disable TLS for OTLP connection."""
        self._otlp_secure = secure
        return self

    def with_otlp_http(self, use_http: bool = True) -> "PulseBuilder":
        """Use HTTP instead of gRPC for OTLP."""
        self._otlp_use_http = use_http
        return self

    def with_mcap(self, path: str) -> "PulseBuilder":
        """Enable MCAP recording to the specified file path."""
        self._mcap_path = path
        return self

    def with_log_level(self, level: LogLevel) -> "PulseBuilder":
        """Set the log level for this service/module.

        This acts as the code-level default. It can be overridden by the
        config file via [logging.modules.<service-name>] or env vars.

        Priority chain (highest to lowest):
            env var > TOML per-module override > with_log_level() > environment default

        Example:
            pulse = Pulse.new() \\
                .with_service("vision", "1.0.0") \\
                .with_log_level(LogLevel.MODULE_LEVEL_3) \\
                .build()
        """
        self._log_level = level
        return self

    def with_tracing(self) -> "PulseBuilder":
        """Enable distributed tracing."""
        self._tracing_enabled = True
        return self

    def build(self) -> "Pulse":
        """Build and return the Pulse instance."""
        # Load config from file (auto-discovery or specified path)
        service_opts, pulse_opts = from_config(self._config_path)

        # If with_service was called, ignore service-level config from file
        if self._service_from_code:
            from .options import ServiceOptions, Environment

            service_opts = ServiceOptions(
                name=self._name or "pulse-service",
                version=self._version or "1.0.0",
                description=self._description or "",
                environment=self._environment or Environment.DEVELOPMENT,
                labels=dict(self._labels),  # Copy builder labels
            )
        else:
            # Override with builder values (highest priority)
            if self._name:
                service_opts.name = self._name
            if self._version:
                service_opts.version = self._version
            if self._description:
                service_opts.description = self._description
            if self._environment:
                service_opts.environment = self._environment

            # Merge labels from builder with config
            if self._labels:
                service_opts.labels.update(self._labels)

        # Configure OTLP if specified via builder
        if self._otlp_endpoint:
            pulse_opts.telemetry.otlp.enabled = True
            pulse_opts.telemetry.otlp.endpoint = self._otlp_endpoint

        if self._otlp_auth_token:
            pulse_opts.telemetry.otlp.auth_token = self._otlp_auth_token

        if self._otlp_secure is not None:
            pulse_opts.telemetry.otlp.secure = self._otlp_secure

        if self._otlp_use_http is not None:
            pulse_opts.telemetry.otlp.use_http = self._otlp_use_http

        # Apply code-level log level (only if config didn't already set a per-module override)
        if self._log_level is not None and self._name:
            modules = pulse_opts.telemetry.logging.modules
            if self._name not in modules:
                modules[self._name] = ModuleOptions(level=self._log_level)

        # Enable tracing if requested
        if self._tracing_enabled:
            pulse_opts.telemetry.tracing.enabled = True
            pulse_opts.telemetry.otlp.enabled = True

        # Configure MCAP if specified
        if self._mcap_path:
            pulse_opts.foxglove.enabled = True
            pulse_opts.foxglove.mcap_path = self._mcap_path

        return Pulse(service_opts, pulse_opts)


class Pulse:
    """
    Main Pulse framework class providing unified observability.

    Integrates:
    - Logging (logbook + OpenTelemetry + MCAP)
    - Metrics (OpenTelemetry + MCAP with Pydantic support)
    - Tracing (OpenTelemetry + MCAP with decorator support)

    Example using builder pattern:
        pulse = Pulse.new() \\
            .with_service("my-service", "1.0.0") \\
            .environment(Environment.PRODUCTION) \\
            .build()

    Example using direct instantiation:
        pulse = Pulse(service_opts, pulse_opts)
    """

    @classmethod
    def new(cls) -> "PulseBuilder":
        """Create a new PulseBuilder for fluent configuration."""
        return PulseBuilder()

    def __init__(self, service_opts: ServiceOptions, pulse_opts: PulseOptions):
        self.service_opts = service_opts
        self.pulse_opts = pulse_opts

        # Initialize unified MCAP writer if enabled
        self.mcap_writer: Optional[UnifiedMcapWriter] = None
        if pulse_opts.foxglove.enabled and pulse_opts.foxglove.mcap_path:
            self.mcap_writer = UnifiedMcapWriter(
                mcap_path=pulse_opts.foxglove.mcap_path,
                service_name=service_opts.name,
            )

        # Initialize logging
        self.logger = PulseLogger(
            service_opts=service_opts,
            logging_opts=pulse_opts.telemetry.logging,
            otlp_opts=pulse_opts.telemetry.otlp
            if pulse_opts.telemetry.otlp.enabled
            else None,
            mcap_writer=self.mcap_writer,
        )

        # Initialize metrics
        self.metrics = PulseMetrics(
            service_opts=service_opts,
            metrics_opts=pulse_opts.telemetry.metrics,
            otlp_opts=pulse_opts.telemetry.otlp
            if pulse_opts.telemetry.otlp.enabled
            else None,
            mcap_writer=self.mcap_writer,
        )

        # Initialize tracing
        self.tracing = PulseTracing(
            service_opts=service_opts,
            tracing_opts=pulse_opts.telemetry.tracing,
            otlp_opts=pulse_opts.telemetry.otlp
            if pulse_opts.telemetry.otlp.enabled
            else None,
            mcap_writer=self.mcap_writer,
        )

    def __enter__(self):
        """Enter context manager"""
        self._pulse_token = set_current_pulse(self)
        self._metrics_token = set_current_pulse_metrics(self)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager and close resources"""
        reset_current_pulse_metrics(self._metrics_token)
        reset_current_pulse(self._pulse_token)
        self.close()
        return False

    def close(self):
        """Close all Pulse components and flush pending data"""
        # Close components in order
        self.tracing.close()
        self.metrics.close()
        self.logger.close()

        # Close MCAP writer last
        if self.mcap_writer:
            self.mcap_writer.close()
