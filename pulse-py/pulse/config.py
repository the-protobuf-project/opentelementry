"""Dynaconf-based configuration management for Pulse SDK.

Configuration Priority (lowest to highest):
1. Default values (lowest priority)
2. Config file (pulse.toml / pulse.yaml / pulse.json)
3. Environment variables (.env file or PULSE_* env vars)
4. Code-based (builder methods) - highest priority

Environment variables use PULSE_ prefix with double underscores for nesting:
    PULSE_SERVICE__NAME=my-service
    PULSE_TELEMETRY__OTLP__ENDPOINT=otel.example.com
    PULSE_TELEMETRY__OTLP__AUTH_TOKEN=your-token

Auto-discovers config files from:
1. PULSE_CONFIG_PATH environment variable
2. pulse.toml in current directory
3. .config/pulse.toml

Example pulse.toml:
    [service]
    name = "my-service"
    version = "1.0.0"

    [telemetry.otlp]
    endpoint = "otel.example.com"
    auth_token = "your-token"
"""

from pathlib import Path
from typing import Optional, Dict, Any
from dynaconf import Dynaconf, Validator


# Auto-discover config files
def _find_config_files() -> list[str]:
    """Find config files in order of priority."""
    config_files = []

    # Check for pulse.toml, pulse.yaml, pulse.json in current directory
    for ext in ["toml", "yaml", "yml", "json"]:
        path = Path.cwd() / f"pulse.{ext}"
        if path.exists():
            config_files.append(str(path))
            break

    # Check .config directory
    if not config_files:
        for ext in ["toml", "yaml", "yml", "json"]:
            path = Path.cwd() / ".config" / f"pulse.{ext}"
            if path.exists():
                config_files.append(str(path))
                break

    return config_files


# Initialize Dynaconf settings
settings = Dynaconf(
    envvar_prefix="PULSE",
    settings_files=_find_config_files(),
    environments=False,  # Don't use [development], [production] sections
    load_dotenv=True,
    merge_enabled=True,
    validators=[
        # Service validators
        Validator("service.name", default="unnamed-service"),
        Validator("service.version", default="1.0.0"),
        Validator("service.environment", default="development"),
        Validator("service.description", default=""),
        # Telemetry validators
        Validator("telemetry.enabled", default=True),
        # OTLP validators
        Validator("telemetry.otlp.endpoint", default="localhost:4317"),
        Validator("telemetry.otlp.auth_token", default=""),
        Validator("telemetry.otlp.secure", default=False),
        Validator("telemetry.otlp.use_http", default=False),
        # Metrics validators
        Validator("telemetry.metrics.export_interval_seconds", default=10),
        # Logging validators
        Validator("logging.log.report_caller", default=True),
        Validator("logging.log.report_timestamp", default=True),
        # Foxglove validators
        Validator("foxglove.enabled", default=False),
        Validator("foxglove.file_path", default=""),
        # Profiling validators
        Validator("profiling.enabled", default=False),
        Validator("profiling.server_address", default="http://localhost:4040"),
        # Tracing validators
        Validator("tracing.enabled", default=True),
    ],
)


def load_config(config_path: Optional[str] = None) -> Dynaconf:
    """Load configuration from file and environment variables.

    Args:
        config_path: Optional path to config file. If not provided,
                    auto-discovers pulse.toml/yaml/json.

    Returns:
        Dynaconf settings object with loaded configuration.
    """
    if config_path:
        # Load from specific path
        return Dynaconf(
            envvar_prefix="PULSE",
            settings_files=[config_path],
            environments=False,
            load_dotenv=True,
            merge_enabled=True,
        )
    return settings


def get_service_config() -> Dict[str, Any]:
    """Get service configuration as a dictionary."""
    return {
        "name": settings.get("service.name", "unnamed-service"),
        "version": settings.get("service.version", "1.0.0"),
        "environment": settings.get("service.environment", "development"),
        "description": settings.get("service.description", ""),
        "attributes": dict(settings.get("service.attributes", {})),
    }


def get_telemetry_config() -> Dict[str, Any]:
    """Get telemetry configuration as a dictionary."""
    return {
        "enabled": settings.get("telemetry.enabled", True),
        "otlp": {
            "endpoint": settings.get("telemetry.otlp.endpoint", "localhost:4317"),
            "auth_token": settings.get("telemetry.otlp.auth_token", ""),
            "secure": settings.get("telemetry.otlp.secure", False),
            "use_http": settings.get("telemetry.otlp.use_http", False),
        },
        "metrics": {
            "export_interval_seconds": settings.get(
                "telemetry.metrics.export_interval_seconds", 10
            ),
        },
    }


def get_foxglove_config() -> Dict[str, Any]:
    """Get Foxglove/MCAP configuration as a dictionary."""
    return {
        "enabled": settings.get("foxglove.enabled", False),
        "file_path": settings.get("foxglove.file_path", ""),
    }


def get_logging_config() -> Dict[str, Any]:
    """Get logging configuration as a dictionary."""
    return {
        "report_caller": settings.get("logging.log.report_caller", True),
        "report_timestamp": settings.get("logging.log.report_timestamp", True),
    }


def get_tracing_config() -> Dict[str, Any]:
    """Get tracing configuration as a dictionary."""
    return {
        "enabled": settings.get("tracing.enabled", True),
    }


def get_profiling_config() -> Dict[str, Any]:
    """Get profiling configuration as a dictionary."""
    return {
        "enabled": settings.get("profiling.enabled", False),
        "server_address": settings.get(
            "profiling.server_address", "http://localhost:4040"
        ),
    }
