#include "pulse/pulse.hpp"

#include <chrono>
#include <thread>
#include <cstdlib>

std::pair<std::string, uint16_t> get_otel_endpoint() {
    const char* env = std::getenv("OTEL_EXPORTER_OTLP_ENDPOINT");
    if (env) {
        std::string endpoint(env);
        auto pos = endpoint.find(':');
        if (pos != std::string::npos) {
            return {endpoint.substr(0, pos), 
                    static_cast<uint16_t>(std::stoi(endpoint.substr(pos + 1)))};
        }
        return {endpoint, 4317};
    }
    return {"localhost", 4317};
}

void simple_operation(pulse::tracing::Tracer& tracer) {
    PULSE_SPAN(tracer, "simple_operation");
    PULSE_LOG_INFO("This is a simple traced operation");
    std::this_thread::sleep_for(std::chrono::milliseconds(100));
}

int main() {
    auto [otel_host, otel_port] = get_otel_endpoint();
    
    auto pulse = pulse::Pulse::builder("simple-trace-test", "1.0.0")
        .environment(pulse::Environment::Development)
        .with_otlp(otel_host, otel_port)
        .build();

    PULSE_LOG_INFO("=== SIMPLE TRACE TEST ===");
    PULSE_LOG_INFO("Service: simple-trace-test");
    PULSE_LOG_INFO("Sending ONE span to OTLP...");

    simple_operation(pulse.tracer());

    PULSE_LOG_INFO("Span created. Waiting 5 seconds for export...");
    std::this_thread::sleep_for(std::chrono::seconds(5));

    PULSE_LOG_INFO("Done! Check Tempo for service: simple-trace-test");
    PULSE_LOG_INFO("Look for span: simple_operation");

    return 0;
}
