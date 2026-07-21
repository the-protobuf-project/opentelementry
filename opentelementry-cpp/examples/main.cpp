#include <opentelementry/opentelementry.hpp>
#include <iostream>
#include <thread>
#include <chrono>

int main() {
    auto opentelementry = opentelementry::Opentelementry::builder("example-service", "1.0.0")
        .description("Example service demonstrating opentelementry-cpp")
        .environment(opentelementry::Environment::Development)
        .with_mcap("output.mcap")
        .build();

    OPENTELEMENTRY_LOG_INFO("Application started");
    OPENTELEMENTRY_LOG_DEBUG("Debug message with context");

    opentelementry.metrics().counter("requests_total", 1.0);
    opentelementry.metrics().histogram("request_duration_ms", 42.5);
    opentelementry.metrics().gauge("active_connections", 10.0);

    {
        auto span = opentelementry.tracer().start_span("process_request");
        span.set_attribute("user_id", "12345");
        span.set_attribute("method", "GET");
        span.add_event("started_processing");

        std::this_thread::sleep_for(std::chrono::milliseconds(100));

        span.add_event("finished_processing");
        span.set_status(opentelementry::tracing::SpanStatus::Ok);
        span.end();
    }

    {
        OPENTELEMENTRY_SPAN(opentelementry.tracer(), "nested_operation");
        opentelementry.metrics().counter("operations_total", 1.0);
        OPENTELEMENTRY_LOG_INFO("Performing nested operation");
    }

    opentelementry.logger().info("Custom logger message", __FILE__, __LINE__);

    for (int i = 0; i < 5; ++i) {
        opentelementry.metrics().counter("loop_iterations", 1.0);
        opentelementry.metrics().histogram("iteration_value", static_cast<double>(i));
    }

    OPENTELEMENTRY_LOG_INFO("Application shutting down");

    opentelementry.flush();
    opentelementry.close();

    std::cout << "Example completed. Check output.mcap in Foxglove Studio." << std::endl;
    return 0;
}
