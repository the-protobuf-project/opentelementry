#include <pulse/pulse.hpp>
#include <iostream>
#include <thread>
#include <chrono>

int main() {
    auto pulse = pulse::Pulse::builder("example-service", "1.0.0")
        .description("Example service demonstrating pulse-cpp")
        .environment(pulse::Environment::Development)
        .with_mcap("output.mcap")
        .build();

    PULSE_LOG_INFO("Application started");
    PULSE_LOG_DEBUG("Debug message with context");

    pulse.metrics().counter("requests_total", 1.0);
    pulse.metrics().histogram("request_duration_ms", 42.5);
    pulse.metrics().gauge("active_connections", 10.0);

    {
        auto span = pulse.tracer().start_span("process_request");
        span.set_attribute("user_id", "12345");
        span.set_attribute("method", "GET");
        span.add_event("started_processing");
        
        std::this_thread::sleep_for(std::chrono::milliseconds(100));
        
        span.add_event("finished_processing");
        span.set_status(pulse::tracing::SpanStatus::Ok);
        span.end();
    }

    {
        PULSE_SPAN(pulse.tracer(), "nested_operation");
        pulse.metrics().counter("operations_total", 1.0);
        PULSE_LOG_INFO("Performing nested operation");
    }

    pulse.logger().info("Custom logger message", __FILE__, __LINE__);

    for (int i = 0; i < 5; ++i) {
        pulse.metrics().counter("loop_iterations", 1.0);
        pulse.metrics().histogram("iteration_value", static_cast<double>(i));
    }

    PULSE_LOG_INFO("Application shutting down");
    
    pulse.flush();
    pulse.close();

    std::cout << "Example completed. Check output.mcap in Foxglove Studio." << std::endl;
    return 0;
}
