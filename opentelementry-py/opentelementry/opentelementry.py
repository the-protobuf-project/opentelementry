from typing import Optional, Dict

from .options import (
    ServiceOptions,
    OpentelementryOptions,
    Environment,
    LogLevel,
    ModuleOptions,
    from_config,
)
from ._private.logging import OpentelementryLogger
from ._private.metrics import (
    OpentelementryMetrics,
    set_current_opentelementry_metrics,
    reset_current_opentelementry_metrics,
)
from ._private.tracing import (
    OpentelementryTracing,
    set_current_opentelementry,
    reset_current_opentelementry,
)
from ._private.foxglove import UnifiedMcapWriter


class OpentelementryBuilder:
    """Builder for creating Opentelementry instances with fluent API.

    Configuration Priority (Lowest to Highest):
    1. Defaults (lowest priority)
    2. Config file (opentelementry.toml / opentelementry.yaml / opentelementry.json)
    3. Environment variables (OPENTELEMENTRY_*)
    4. Code-based (builder methods) - highest priority

    Example:
        opentelementry = Opentelementry.new() \\
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

    def with_config(self, config_path: str) -> "OpentelementryBuilder":
        """Load configuration from a specific file path."""
        self._config_path = config_path
        return self

    def with_service(self, name: str, version: str) -> "OpentelementryBuilder":
        """Set service name and version.

        When this is called, it indicates the user wants to configure service via code,
        so we'll clear any service-level configuration from the config file to avoid collisions.
        """
        self._name = name
        self._version = version
        # Mark that service should be configured via code only
        self._service_from_code = True
        return self

    def description(self, desc: str) -> "OpentelementryBuilder":
        """Set service description."""
        self._description = desc
        return self

    def environment(self, env: Environment) -> "OpentelementryBuilder":
        """Set deployment environment."""
        self._environment = env
        return self

    def with_label(self, key: str, value: str) -> "OpentelementryBuilder":
        """Add a custom label to all telemetry."""
        self._labels[key] = value
        return self

    def with_labels(self, labels: Dict[str, str]) -> "OpentelementryBuilder":
        """Add multiple custom labels to all telemetry."""
        self._labels.update(labels)
        return self

    def with_otlp(self, host: str, port: int) -> "OpentelementryBuilder":
        """Enable OTLP export to the specified endpoint."""
        self._otlp_endpoint = f"{host}:{port}"
        return self

    def with_otlp_endpoint(self, endpoint: str) -> "OpentelementryBuilder":
        """Enable OTLP export to the specified endpoint string."""
        self._otlp_endpoint = endpoint
        return self

    def with_otlp_auth(self, token: str) -> "OpentelementryBuilder":
        """Set OTLP authentication token."""
        self._otlp_auth_token = token
        return self

    def with_otlp_secure(self, secure: bool = True) -> "OpentelementryBuilder":
        """Enable/disable TLS for OTLP connection."""
        self._otlp_secure = secure
        return self

    def with_otlp_http(self, use_http: bool = True) -> "OpentelementryBuilder":
        """Use HTTP instead of gRPC for OTLP."""
        self._otlp_use_http = use_http
        return self

    def with_mcap(self, path: str) -> "OpentelementryBuilder":
        """Enable MCAP recording to the specified file path."""
        self._mcap_path = path
        return self

    def with_log_level(self, level: LogLevel) -> "OpentelementryBuilder":
        """Set the log level for this service/module.

        This acts as the code-level default. It can be overridden by the
        config file via [logging.modules.<service-name>] or env vars.

        Priority chain (highest to lowest):
            env var > TOML per-module override > with_log_level() > environment default

        Example:
            opentelementry = Opentelementry.new() \\
                .with_service("vision", "1.0.0") \\
                .with_log_level(LogLevel.MODULE_LEVEL_3) \\
                .build()
        """
        self._log_level = level
        return self

    def with_tracing(self) -> "OpentelementryBuilder":
        """Enable distributed tracing."""
        self._tracing_enabled = True
        return self

    def build(self) -> "Opentelementry":
        """Build and return the Opentelementry instance."""
        # Load config from file (auto-discovery or specified path)
        service_opts, opentelementry_opts = from_config(self._config_path)

        # If with_service was called, ignore service-level config from file
        if self._service_from_code:
            service_opts = ServiceOptions(
                name=self._name or "opentelementry-service",
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
            opentelementry_opts.telemetry.otlp.enabled = True
            opentelementry_opts.telemetry.otlp.endpoint = self._otlp_endpoint

        if self._otlp_auth_token:
            opentelementry_opts.telemetry.otlp.auth_token = self._otlp_auth_token

        if self._otlp_secure is not None:
            opentelementry_opts.telemetry.otlp.secure = self._otlp_secure

        if self._otlp_use_http is not None:
            opentelementry_opts.telemetry.otlp.use_http = self._otlp_use_http

        # Apply code-level log level (only if config didn't already set a per-module override)
        if self._log_level is not None and self._name:
            modules = opentelementry_opts.telemetry.logging.modules
            if self._name not in modules:
                modules[self._name] = ModuleOptions(level=self._log_level)

        # Enable tracing if requested
        if self._tracing_enabled:
            opentelementry_opts.telemetry.tracing.enabled = True
            opentelementry_opts.telemetry.otlp.enabled = True

        # Configure MCAP if specified
        if self._mcap_path:
            opentelementry_opts.foxglove.enabled = True
            opentelementry_opts.foxglove.mcap_path = self._mcap_path

        return Opentelementry(service_opts, opentelementry_opts)


class Opentelementry:
    """
    Main Opentelementry framework class providing unified observability.

    Integrates:
    - Logging (logbook + OpenTelemetry + MCAP)
    - Metrics (OpenTelemetry + MCAP with Pydantic support)
    - Tracing (OpenTelemetry + MCAP with decorator support)

    Example using builder pattern:
        opentelementry = Opentelementry.new() \\
            .with_service("my-service", "1.0.0") \\
            .environment(Environment.PRODUCTION) \\
            .build()

    Example using direct instantiation:
        opentelementry = Opentelementry(service_opts, opentelementry_opts)
    """

    @classmethod
    def new(cls) -> "OpentelementryBuilder":
        """Create a new OpentelementryBuilder for fluent configuration."""
        return OpentelementryBuilder()

    def __init__(
        self, service_opts: ServiceOptions, opentelementry_opts: OpentelementryOptions
    ):
        self.service_opts = service_opts
        self.opentelementry_opts = opentelementry_opts

        # Initialize unified MCAP writer if enabled
        self.mcap_writer: Optional[UnifiedMcapWriter] = None
        if (
            opentelementry_opts.foxglove.enabled
            and opentelementry_opts.foxglove.mcap_path
        ):
            self.mcap_writer = UnifiedMcapWriter(
                mcap_path=opentelementry_opts.foxglove.mcap_path,
                service_name=service_opts.name,
            )

        # Initialize logging
        self.logger = OpentelementryLogger(
            service_opts=service_opts,
            logging_opts=opentelementry_opts.telemetry.logging,
            otlp_opts=opentelementry_opts.telemetry.otlp
            if opentelementry_opts.telemetry.otlp.enabled
            else None,
            mcap_writer=self.mcap_writer,
        )

        # Initialize metrics
        self.metrics = OpentelementryMetrics(
            service_opts=service_opts,
            metrics_opts=opentelementry_opts.telemetry.metrics,
            otlp_opts=opentelementry_opts.telemetry.otlp
            if opentelementry_opts.telemetry.otlp.enabled
            else None,
            mcap_writer=self.mcap_writer,
        )

        # Initialize tracing
        self.tracing = OpentelementryTracing(
            service_opts=service_opts,
            tracing_opts=opentelementry_opts.telemetry.tracing,
            otlp_opts=opentelementry_opts.telemetry.otlp
            if opentelementry_opts.telemetry.otlp.enabled
            else None,
            mcap_writer=self.mcap_writer,
        )

    def __enter__(self):
        """Enter context manager"""
        self._opentelementry_token = set_current_opentelementry(self)
        self._metrics_token = set_current_opentelementry_metrics(self)
        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit context manager and close resources"""
        reset_current_opentelementry_metrics(self._metrics_token)
        reset_current_opentelementry(self._opentelementry_token)
        self.close()
        return False

    def close(self):
        """Close all Opentelementry components and flush pending data"""
        # Close components in order
        self.tracing.close()
        self.metrics.close()
        self.logger.close()

        # Close MCAP writer last
        if self.mcap_writer:
            self.mcap_writer.close()
