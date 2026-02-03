"""
Tracing example using decorators - cleaner API without manual event tracking.

This example uses the new Pulse.new() builder API and shows how to use
the @traced decorator and TracedOperation context manager for automatic
event tracking.

Run with:
    uv run python -m examples.tracing.simple_example
"""

import pulse
from pulse import Pulse, TracedOperation
from pydantic import BaseModel
import time


class ProcessingRequest(BaseModel):
    """Request for processing"""
    request_id: str
    data: str


class ProcessingResponse(BaseModel):
    """Response from processing"""
    request_id: str
    result: str
    processing_time_ms: float


@pulse.trace("processing", auto_events=True)
def random_function():
    """Example function with @pulse.trace decorator"""
    time.sleep(0.1)
    return "processed"


def main():
    """Run tracing example with decorators"""
    # Uses pulse.toml config for OTLP endpoint and service info
    with Pulse.new().build() as p:
        p.logger.info("=== Decorator-Based Tracing Example ===")
        
        # Example 0: Using @pulse.trace decorator
        p.logger.info("Calling function with @pulse.trace decorator...")
        result = random_function()
        p.logger.info(f"Function result: {result}")
        
        # Example 1: Using TracedOperation context manager
        with TracedOperation(p.tracing, "data_pipeline", {"pipeline.version": "1.0"}) as op:
            op.step("loading_data")
            time.sleep(0.05)
            
            op.step("validating_data")
            time.sleep(0.03)
            
            op.step("transforming_data")
            time.sleep(0.08)
            
            op.step("saving_results")
            time.sleep(0.04)
        
        p.logger.info("✅ Pipeline completed with automatic event tracking!")
        
        # Example 2: Nested operations
        with TracedOperation(p.tracing, "ml_inference", {"model": "gpt-4"}) as op:
            op.step("loading_model")
            time.sleep(0.02)
            
            op.step("preprocessing_input")
            with TracedOperation(p.tracing, "tokenization") as tokenize_op:
                tokenize_op.step("splitting_text")
                time.sleep(0.01)
                tokenize_op.step("encoding_tokens")
                time.sleep(0.015)
            
            op.step("running_inference")
            time.sleep(0.15)
            
            op.step("postprocessing_output")
            time.sleep(0.02)
        
        p.logger.info("✅ ML inference completed with nested tracing!")
        
        # Give time for spans to export
        time.sleep(2)


if __name__ == "__main__":
    main()
