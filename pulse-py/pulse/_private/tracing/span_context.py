"""Span context manager for manual span creation.

This module provides a context manager for creating and managing trace spans
manually, as opposed to using the decorator-based approach.

Typical usage example:

    with pulse.tracing.start_span("operation", {"key": "value"}) as span:
        # Do work
        span.add_event("checkpoint")
        span.set_attribute("result", "success")
"""

import time
from typing import Any, Dict, Optional, TYPE_CHECKING

from opentelemetry.trace import Status, StatusCode

from .context import get_trace_context, set_trace_context, reset_trace_context

if TYPE_CHECKING:
    from .tracing import PulseTracing


class SpanContext:
    """Context manager for manual span creation.

    This class provides a context manager interface for creating and managing
    OpenTelemetry spans manually. It handles span lifecycle, error recording,
    and context propagation.

    Attributes:
        tracing: The PulseTracing instance.
        name: Name of the span/operation.
        attributes: Dictionary of span attributes.
        span: The OpenTelemetry span object (set after __enter__).
        trace_id: Hex string trace ID (set after __enter__).
        span_id: Hex string span ID (set after __enter__).
    """

    def __init__(
        self,
        tracing: "PulseTracing",
        name: str,
        attributes: Optional[Dict[str, Any]] = None,
    ):
        """Initialize the span context.

        Args:
            tracing: The PulseTracing instance managing this span.
            name: Name of the span/operation.
            attributes: Optional dictionary of span attributes.
        """
        self.tracing = tracing
        self.name = name
        self.attributes = attributes or {}
        self.span = None
        self.token = None
        self.trace_id = None
        self.span_id = None
        self.parent_span_id = None

    def __enter__(self):
        """Enter the span context.

        Creates and starts a new span, sets attributes, and updates the
        trace context for nested spans.

        Returns:
            Self, allowing access to span methods like add_event().
        """
        if not self.tracing.enabled or not self.tracing.tracer:
            return self

        # Get current context
        ctx = get_trace_context()
        self.parent_span_id = ctx.get("span_id")

        # Start span
        self.span = self.tracing.tracer.start_span(self.name)

        # Set attributes
        for key, value in self.attributes.items():
            self.span.set_attribute(key, value)

        # Get span context
        span_ctx = self.span.get_span_context()
        self.trace_id = format(span_ctx.trace_id, "032x")
        self.span_id = format(span_ctx.span_id, "016x")

        # Update context
        new_ctx = {
            "trace_id": self.trace_id,
            "span_id": self.span_id,
        }
        self.token = set_trace_context(new_ctx)

        # Write to MCAP
        if self.tracing.mcap_writer and not self.tracing.mcap_writer.is_closed():
            self.tracing.mcap_writer.write_trace(
                trace_id=self.trace_id,
                span_id=self.span_id,
                name=self.name,
                parent_span_id=self.parent_span_id,
                attributes=self.attributes,
                timestamp=time.time_ns(),
            )

        return self

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Exit the span context.

        Ends the span, records any exceptions, and restores the previous
        trace context.

        Args:
            exc_type: Exception type if an exception occurred.
            exc_val: Exception value if an exception occurred.
            exc_tb: Exception traceback if an exception occurred.

        Returns:
            False to propagate any exception that occurred.
        """
        if self.span:
            if exc_type:
                self.span.record_exception(exc_val)
                self.span.set_status(Status(StatusCode.ERROR, str(exc_val)))
            else:
                self.span.set_status(Status(StatusCode.OK))

            self.span.end()

        if self.token:
            reset_trace_context(self.token)

        return False

    def add_event(self, name: str, attributes: Optional[Dict[str, Any]] = None):
        """Add an event to the span.

        Events are timestamped annotations that can be added to spans to mark
        significant points in the span's lifetime.

        Args:
            name: Name of the event.
            attributes: Optional dictionary of event attributes.
        """
        if self.span:
            self.span.add_event(name, attributes=attributes or {})

    def set_attribute(self, key: str, value: Any):
        """Set an attribute on the span.

        Attributes are key-value pairs that provide additional context about
        the span.

        Args:
            key: Attribute key.
            value: Attribute value (must be a primitive type or string).
        """
        if self.span:
            self.span.set_attribute(key, value)
