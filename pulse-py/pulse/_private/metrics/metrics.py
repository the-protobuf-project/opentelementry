from typing import Any, Dict, Optional
from pydantic import BaseModel
import time

from opentelemetry import metrics
from opentelemetry.sdk.metrics import MeterProvider
from opentelemetry.sdk.metrics.export import PeriodicExportingMetricReader
from opentelemetry.exporter.otlp.proto.grpc.metric_exporter import OTLPMetricExporter
from opentelemetry.sdk.resources import Resource

from ...options import ServiceOptions, MetricsOptions, OTLPOptions
from ..foxglove import UnifiedMcapWriter


class PulseMetrics:
    """
    Metrics system that integrates OpenTelemetry metrics with Pydantic models.
    
    Features:
    - Automatic metric extraction from Pydantic models using field metadata
    - Sends metrics to OTLP collector when enabled
    - Writes metrics to MCAP file when enabled
    """
    
    def __init__(
        self,
        service_opts: ServiceOptions,
        metrics_opts: MetricsOptions,
        otlp_opts: Optional[OTLPOptions] = None,
        mcap_writer: Optional[UnifiedMcapWriter] = None,
    ):
        self.service_opts = service_opts
        self.metrics_opts = metrics_opts
        self.mcap_writer = mcap_writer
        
        # Initialize OpenTelemetry metrics if enabled
        self.meter = None
        self._counters: Dict[str, Any] = {}
        self._histograms: Dict[str, Any] = {}
        self._gauges: Dict[str, Any] = {}
        
        if otlp_opts and otlp_opts.enabled:
            self._setup_otel_metrics(service_opts, metrics_opts, otlp_opts)
    
    def _setup_otel_metrics(
        self,
        service_opts: ServiceOptions,
        metrics_opts: MetricsOptions,
        otlp_opts: OTLPOptions,
    ):
        """Setup OpenTelemetry metrics with OTLP exporter"""
        resource = Resource.create({
            "service.name": service_opts.name,
            "service.version": service_opts.version,
            "service.environment": service_opts.environment.value,
        })
        
        exporter = OTLPMetricExporter(
            endpoint=f"{otlp_opts.host}:{otlp_opts.port}",
            insecure=True,
        )
        
        reader = PeriodicExportingMetricReader(
            exporter,
            export_interval_millis=metrics_opts.export_interval_seconds * 1000,
        )
        
        provider = MeterProvider(resource=resource, metric_readers=[reader])
        metrics.set_meter_provider(provider)
        
        self.meter = metrics.get_meter(service_opts.name)
    
    def record(self, model: BaseModel, labels: Optional[Dict[str, Any]] = None):
        """
        Record metrics from a Pydantic model.
        
        The model should use field json_schema_extra to specify metric types:
        
        Example:
            class MyMetrics(BaseModel):
                request_count: int = Field(json_schema_extra={"metric_type": "counter", "metric_name": "requests_total"})
                response_time: float = Field(json_schema_extra={"metric_type": "histogram", "metric_name": "response_time_ms"})
                active_users: int = Field(json_schema_extra={"metric_type": "gauge", "metric_name": "active_users"})
        """
        if not isinstance(model, BaseModel):
            raise ValueError("record() requires a Pydantic BaseModel instance")
        
        # If model is a MetricsModel, resolve metric names with service name prefix
        if hasattr(model, '_resolve_metric_names'):
            model._resolve_metric_names(service_name=self.service_opts.name)
        
        # Extract metrics from model fields
        for field_name, field_info in model.model_fields.items():
            # Check json_schema_extra for metric metadata
            json_schema_extra = field_info.json_schema_extra
            if not json_schema_extra or not isinstance(json_schema_extra, dict):
                continue
            
            if "metric_type" not in json_schema_extra:
                continue
            
            metric_type = json_schema_extra.get("metric_type")
            metric_name = json_schema_extra.get("metric_name", field_name)
            value = getattr(model, field_name)
            
            # Record based on type
            if metric_type == "counter":
                self._record_counter(metric_name, float(value), labels)
            elif metric_type == "histogram":
                self._record_histogram(metric_name, float(value), labels)
            elif metric_type == "gauge":
                self._record_gauge(metric_name, float(value), labels)
    
    def _record_counter(self, name: str, value: float, labels: Optional[Dict[str, Any]] = None):
        """Record a counter metric"""
        # OTEL counter
        if self.meter:
            if name not in self._counters:
                self._counters[name] = self.meter.create_counter(name)
            self._counters[name].add(value, attributes=labels or {})
        
        # MCAP
        if self.mcap_writer and not self.mcap_writer.is_closed():
            self.mcap_writer.write_metric(
                name=name,
                value=value,
                metric_type="counter",
                labels=labels,
                timestamp=time.time_ns(),
            )
    
    def _record_histogram(self, name: str, value: float, labels: Optional[Dict[str, Any]] = None):
        """Record a histogram metric"""
        # OTEL histogram
        if self.meter:
            if name not in self._histograms:
                self._histograms[name] = self.meter.create_histogram(name)
            self._histograms[name].record(value, attributes=labels or {})
        
        # MCAP
        if self.mcap_writer and not self.mcap_writer.is_closed():
            self.mcap_writer.write_metric(
                name=name,
                value=value,
                metric_type="histogram",
                labels=labels,
                timestamp=time.time_ns(),
            )
    
    def _record_gauge(self, name: str, value: float, labels: Optional[Dict[str, Any]] = None):
        """Record a gauge metric"""
        # OTEL gauge (using observable gauge)
        if self.meter:
            if name not in self._gauges:
                # Store the current value for the callback
                self._gauges[name] = {"value": value, "labels": labels or {}}
                
                def callback(options):
                    gauge_data = self._gauges.get(name, {})
                    yield metrics.Observation(
                        gauge_data.get("value", 0),
                        attributes=gauge_data.get("labels", {}),
                    )
                
                self.meter.create_observable_gauge(name, callbacks=[callback])
            else:
                # Update stored value
                self._gauges[name] = {"value": value, "labels": labels or {}}
        
        # MCAP
        if self.mcap_writer and not self.mcap_writer.is_closed():
            self.mcap_writer.write_metric(
                name=name,
                value=value,
                metric_type="gauge",
                labels=labels,
                timestamp=time.time_ns(),
            )
    
    def counter(self, name: str, value: float = 1.0, labels: Optional[Dict[str, Any]] = None):
        """Manually record a counter metric"""
        self._record_counter(name, value, labels)
    
    def histogram(self, name: str, value: float, labels: Optional[Dict[str, Any]] = None):
        """Manually record a histogram metric"""
        self._record_histogram(name, value, labels)
    
    def gauge(self, name: str, value: float, labels: Optional[Dict[str, Any]] = None):
        """Manually record a gauge metric"""
        self._record_gauge(name, value, labels)
    
    def close(self):
        """Close metrics system"""
        pass
