from typing import Optional
from pathlib import Path

from .options import ServiceOptions, PulseOptions
from ._private.logging import PulseLogger
from ._private.metrics import PulseMetrics, set_current_pulse_metrics, reset_current_pulse_metrics
from ._private.tracing import PulseTracing, set_current_pulse, reset_current_pulse
from ._private.foxglove import UnifiedMcapWriter

# Auto-load .env file from project root
try:
    from dotenv import load_dotenv
    # Find .env file in current directory or parent directories
    env_path = Path.cwd() / '.env'
    if env_path.exists():
        load_dotenv(env_path)
    else:
        # Try to find .env in parent directories
        current = Path.cwd()
        for parent in current.parents:
            env_path = parent / '.env'
            if env_path.exists():
                load_dotenv(env_path)
                break
except ImportError:
    pass  # python-dotenv not installed, skip auto-loading


class Pulse:
    """
    Main Pulse framework class providing unified observability.
    
    Integrates:
    - Logging (logbook + OpenTelemetry + MCAP)
    - Metrics (OpenTelemetry + MCAP with Pydantic support)
    - Tracing (OpenTelemetry + MCAP with decorator support)
    
    Example:
        pulse = Pulse(
            service_opts=ServiceOptions(
                name="my-service",
                version="1.0.0",
                environment=Environment.PRODUCTION,
            ),
            pulse_opts=PulseOptions(
                telemetry=TelemetryOptions(
                    otlp=OTLPOptions(
                        host="localhost",
                        port=4317,
                        enabled=True,
                    ),
                ),
                foxglove=FoxgloveOptions(
                    enabled=True,
                    mcap_path="/tmp/my-service.mcap",
                ),
            ),
        )
        
        # Use logging
        pulse.logger.info("Service started", {"version": "1.0.0"})
        
        # Use metrics with Pydantic
        class MyMetrics(BaseModel):
            requests: int = Field(metadata={"metric_type": "counter", "metric_name": "requests_total"})
        
        pulse.metrics.record(MyMetrics(requests=1))
        
        # Use tracing with decorator
        @pulse.tracing.trace(name="process_data")
        def process_data():
            pass
        
        # Cleanup
        pulse.close()
    """
    
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
            otlp_opts=pulse_opts.telemetry.otlp if pulse_opts.telemetry.otlp.enabled else None,
            mcap_writer=self.mcap_writer,
        )
        
        # Initialize metrics
        self.metrics = PulseMetrics(
            service_opts=service_opts,
            metrics_opts=pulse_opts.telemetry.metrics,
            otlp_opts=pulse_opts.telemetry.otlp if pulse_opts.telemetry.otlp.enabled else None,
            mcap_writer=self.mcap_writer,
        )
        
        # Initialize tracing
        self.tracing = PulseTracing(
            service_opts=service_opts,
            tracing_opts=pulse_opts.telemetry.tracing,
            otlp_opts=pulse_opts.telemetry.otlp if pulse_opts.telemetry.otlp.enabled else None,
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
