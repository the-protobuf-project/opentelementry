#include "pulse/pulse.hpp"

#include <chrono>
#include <thread>
#include <vector>
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

struct LlmMetrics : public pulse::metrics::RecordMetrics {
    uint64_t request_count = 0;
    double latency_ms = 0.0;
    double cache_hit_rate = 0.0;

    LlmMetrics() = default;
    LlmMetrics(uint64_t count, double latency, double hit_rate)
        : request_count(count), latency_ms(latency), cache_hit_rate(hit_rate) {}

    std::vector<pulse::metrics::MetricField> metric_fields() const override {
        return {
            {"llm.requests.total", pulse::metrics::MetricType::Counter,
             "Total number of LLM requests", static_cast<double>(request_count)},
            {"llm.response.latency_ms", pulse::metrics::MetricType::Histogram,
             "LLM response latency in milliseconds", latency_ms},
            {"llm.cache.hit_rate", pulse::metrics::MetricType::Gauge,
             "LLM cache hit rate percentage", cache_hit_rate}
        };
    }
};

int main() {
    auto [otel_host, otel_port] = get_otel_endpoint();

    auto pulse = pulse::Pulse::builder("metrics-example", "1.0.0")
        .description("Metrics example service")
        .environment(pulse::Environment::Development)
        .with_mcap("examples/metrics.mcap")
        .with_otlp(otel_host, otel_port)
        .build();

    PULSE_LOG_INFO("Metrics Example Started");
    PULSE_LOG_INFO("Sending metrics to OTEL collector at localhost:4317");

    LlmMetrics llm_metrics{42, 123.5, 0.85};
    pulse.metrics().record(llm_metrics);
    PULSE_LOG_INFO("Recorded LLM metrics from struct");

    PULSE_LOG_INFO("Recording metrics for 30 seconds...");

    for (int iteration = 0; iteration < 30; ++iteration) {
        for (int i = 0; i < 10; ++i) {
            pulse.metrics().counter("api.requests", 1.0);
            pulse.metrics().histogram("api.latency_ms", static_cast<double>(i) * 10.0 + 50.0);
            pulse.metrics().gauge("api.active_connections", static_cast<double>(10 - i));

            pulse.metrics().record(llm_metrics);

            PULSE_LOG_DEBUG("Recorded API metrics");
            std::this_thread::sleep_for(std::chrono::milliseconds(100));
        }
    }

    PULSE_LOG_INFO("Metrics recording completed");
    PULSE_LOG_INFO("MCAP file will be finalized automatically");

    return 0;
}
