from .opentelementry import Opentelementry, OpentelementryBuilder
from .options import (
    ServiceOptions,
    OpentelementryOptions,
    Environment,
    LogLevel,
    ModuleOptions,
    LoggingOptions,
    MetricsOptions,
    TracingOptions,
    TelemetryOptions,
    FoxgloveOptions,
    OTLPOptions,
    from_config,
    from_env,
)
from ._private.metrics import (
    counter,
    histogram,
    gauge,
    metric,
    Counter,
    Histogram,
    Gauge,
    MetricsBaseModel,
)
from ._private.tracing import trace, traced, trace_step, TracedOperation

__all__ = [
    "Opentelementry",
    "OpentelementryBuilder",
    "ServiceOptions",
    "OpentelementryOptions",
    "Environment",
    "LogLevel",
    "ModuleOptions",
    "LoggingOptions",
    "MetricsOptions",
    "TracingOptions",
    "TelemetryOptions",
    "FoxgloveOptions",
    "OTLPOptions",
    "from_config",
    "from_env",
    # Metrics - lowercase (explicit names)
    "counter",
    "histogram",
    "gauge",
    "metric",
    # Metrics - capitalized (auto-inferred names)
    "Counter",
    "Histogram",
    "Gauge",
    "MetricsBaseModel",
    # Tracing
    "trace",
    "traced",
    "trace_step",
    "TracedOperation",
]
