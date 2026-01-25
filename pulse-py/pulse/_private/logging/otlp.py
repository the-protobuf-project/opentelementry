"""
OpenTelemetry OTLP logging integration.

This module provides OTLP (OpenTelemetry Protocol) logging export functionality,
allowing logs to be sent to OpenTelemetry collectors for centralized observability.

The OTLPLogger class bridges Python's standard logging module with OpenTelemetry's
logging SDK, enabling automatic export of logs with service metadata and structured
attributes.

Typical usage example:

    otlp_logger = OTLPLogger(
        service_name="my-service",
        service_version="1.0.0",
        service_environment="production",
        otlp_host="localhost",
        otlp_port=4317,
        log_level="INFO"
    )
    otlp_logger.write_log("INFO", "User logged in", {"user_id": "123"})
"""

from typing import Dict, Any, Optional
from opentelemetry.sdk._logs import LoggerProvider, LoggingHandler
from opentelemetry.sdk._logs.export import BatchLogRecordProcessor
from opentelemetry.exporter.otlp.proto.grpc._log_exporter import OTLPLogExporter
from opentelemetry.sdk.resources import Resource
from opentelemetry._logs import set_logger_provider
import logging


class OTLPLogger:
    """OTLP logging handler for OpenTelemetry.
    
    Configures and manages OpenTelemetry logging export to an OTLP collector.
    Attaches a LoggingHandler to Python's root logger to capture all log records
    and export them with service metadata.
    
    Attributes:
        logger: OpenTelemetry logger instance.
        service_name: Name of the service.
        service_version: Version of the service.
        service_environment: Deployment environment.
    """
    
    def __init__(self, service_name: str, service_version: str, service_environment: str,
                 otlp_host: str, otlp_port: int, log_level: str):
        """Initialize the OTLP logger.
        
        Sets up OpenTelemetry logging with OTLP exporter and attaches a handler
        to Python's root logger.
        
        Args:
            service_name: Name of the service for resource attributes.
            service_version: Version of the service.
            service_environment: Deployment environment (e.g., "production").
            otlp_host: Hostname or IP of the OTLP collector.
            otlp_port: Port number of the OTLP collector (typically 4317).
            log_level: Minimum log level to export (DEBUG, INFO, WARNING, ERROR, CRITICAL).
        """
        resource = Resource.create({
            "service.name": service_name,
            "service.version": service_version,
            "service.environment": service_environment,
        })
        
        logger_provider = LoggerProvider(resource=resource)
        
        otlp_exporter = OTLPLogExporter(
            endpoint=f"{otlp_host}:{otlp_port}",
            insecure=True,
        )
        
        logger_provider.add_log_record_processor(
            BatchLogRecordProcessor(otlp_exporter)
        )
        
        set_logger_provider(logger_provider)
        
        # Setup Python logging bridge to OTEL
        handler = LoggingHandler(
            level=getattr(logging, log_level),
            logger_provider=logger_provider,
        )
        
        logging.getLogger().addHandler(handler)
        self.logger = logger_provider.get_logger(service_name)
        self.service_name = service_name
        self.service_version = service_version
        self.service_environment = service_environment
    
    def write_log(self, level: str, message: str, data: Optional[Dict[str, Any]],
                  caller_file: str = "", caller_line: int = 0):
        """Write a log entry to OTLP.
        
        Emits a log record to Python's standard logging, which is then captured
        by the OpenTelemetry LoggingHandler and exported to the OTLP collector.
        
        Args:
            level: Log level string (DEBUG, INFO, WARNING, ERROR, CRITICAL).
            message: Log message text.
            data: Optional dictionary of structured data to include as attributes.
            caller_file: Source file path where the log was called.
            caller_line: Line number where the log was called.
            
        Note:
            Structured data is added both as a JSON string in the 'data' attribute
            and as individual fields for easier querying. Reserved LogRecord keys
            are prefixed with 'field.' to avoid conflicts.
        """
        import logging as std_logging
        import json
        
        # Map logbook levels to standard logging levels
        level_map = {
            "DEBUG": std_logging.DEBUG,
            "INFO": std_logging.INFO,
            "WARNING": std_logging.WARNING,
            "ERROR": std_logging.ERROR,
            "CRITICAL": std_logging.CRITICAL,
        }
        
        # Build extra attributes with service metadata and code location
        extra_attrs = {
            "service.name": self.service_name,
            "service.version": self.service_version,
            "service.environment": self.service_environment,
            "code.filepath": caller_file,
            "code.lineno": caller_line,
        }
        
        # Add structured data as a nested 'data' attribute
        if data:
            extra_attrs["data"] = json.dumps(data)
            
            # Also add individual fields for easier querying
            reserved_keys = {
                'name', 'msg', 'args', 'created', 'filename', 'funcName', 
                'levelname', 'levelno', 'lineno', 'module', 'msecs', 'message', 
                'pathname', 'process', 'processName', 'relativeCreated', 'thread', 'threadName'
            }
            
            for key, value in data.items():
                attr_key = f"field.{key}" if key in reserved_keys else key
                
                if isinstance(value, (dict, list)):
                    extra_attrs[attr_key] = json.dumps(value)
                else:
                    extra_attrs[attr_key] = value
        
        # Emit to standard logging
        std_level = level_map.get(level, std_logging.INFO)
        std_logging.log(std_level, message, extra=extra_attrs)
