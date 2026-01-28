"""
Metrics example demonstrating automatic metric recording with Pydantic models.

This example matches the Go implementation and shows:
- LLM processing metrics (tokens, response time, active requests, cache hit rate)
- Transcription metrics (audio duration, confidence, word count)
- OTLP export and MCAP recording

Make sure you have an OTLP collector running:
  docker-compose -f ../opentelemetry/compose.yaml up
  
Then run this script and check:
1. MCAP file: examples/metrics-data.mcap
2. Open in Foxglove Studio
3. Use Gauge/Indicator/Plot panels to visualize
"""

import pulse
from pulse import (
    Pulse, ServiceOptions, PulseOptions, Environment,
    TelemetryOptions, OTLPOptions, FoxgloveOptions, MetricsBaseModel
)
import time
import random


# LLMMetrics demonstrates automatic metric recording with MetricsBaseModel
# By default, metrics will be prefixed with the service name ("metrics-example")
# You can override by specifying: class LLMMetrics(pulse.MetricsBaseModel, prefix="llm")
class LLMMetrics(MetricsBaseModel, prefix="llm"):
    """LLM processing metrics"""
    tokens_processed: int = pulse.Counter(description="Total tokens processed by LLM")
    response_time: float = pulse.Histogram(description="LLM response time in milliseconds")
    active_requests: int = pulse.Gauge(description="Number of active LLM requests")
    cache_hit_rate: float = pulse.Gauge(description="LLM cache hit rate (0.0-1.0)")


# TranscriptionMetrics for speech-to-text
class TranscriptionMetrics(MetricsBaseModel, prefix="transcription"):
    """Speech-to-text transcription metrics"""
    audio_duration: float = pulse.Histogram(description="Audio duration in seconds")
    confidence: float = pulse.Gauge(description="Transcription confidence score (0.0-1.0)")
    word_count: int = pulse.Counter(description="Total words transcribed")


def main():
    # Configure service
    service_opts = ServiceOptions(
        name="metrics-example",
        version="1.0.0",
        environment=Environment.DEVELOPMENT,
    )

    # Configure Pulse with OTLP and MCAP for metrics
    pulse_opts = PulseOptions(
        telemetry=TelemetryOptions(
            otlp=OTLPOptions(
                enabled=True,
                host="localhost",
                port=4317,
            ),
        ),
        foxglove=FoxgloveOptions(
            enabled=True,
            mcap_path="metrics-data.mcap",
        ),
    )

    with Pulse(service_opts, pulse_opts) as p:
        p.logger.info("Metrics Example Started")
        p.logger.info("Metrics will be written to OTLP and MCAP")

        # Simulate LLM processing with metrics (run for 2 minutes to generate rate data)
        for i in range(120):
            # LLM request metrics
            llm_metrics = LLMMetrics(
                tokens_processed=random.randint(100, 600),
                response_time=random.uniform(500, 2500),  # 500-2500ms
                active_requests=random.randint(1, 11),
                cache_hit_rate=random.random(),
            )

            # Record metrics automatically from field metadata
            p.metrics.record(llm_metrics)

            p.logger.info("LLM request processed", {
                "tokens": llm_metrics.tokens_processed,
                "response_time": llm_metrics.response_time,
            })

            # Transcription metrics every 3rd iteration
            if i % 3 == 0:
                trans_metrics = TranscriptionMetrics(
                    audio_duration=random.uniform(1, 11),  # 1-11 seconds
                    confidence=random.uniform(0.8, 1.0),
                    word_count=random.randint(20, 120),
                )

                p.metrics.record(trans_metrics)

                p.logger.info("Audio transcribed", {
                    "duration": trans_metrics.audio_duration,
                    "confidence": trans_metrics.confidence,
                })

            time.sleep(0.3)  # 300ms

        p.logger.info("Metrics example completed!")
        p.logger.info("Check:")
        p.logger.info("1. MCAP file: metrics-data.mcap")
        p.logger.info("2. Open in Foxglove Studio")
        p.logger.info("3. Use Gauge/Indicator/Plot panels to visualize")


if __name__ == "__main__":
    main()
