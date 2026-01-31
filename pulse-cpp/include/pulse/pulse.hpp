#pragma once

#include "pulse/config.hpp"
#include "pulse/logging/logger.hpp"
#include "pulse/mcap/writer.hpp"
#include "pulse/metrics/metrics.hpp"
#include "pulse/tracing/tracer.hpp"

#if PULSE_USE_OTEL
#include "pulse/otel/exporter.hpp"
#endif

#include <memory>
#include <string>
#include <optional>

namespace pulse {

class PulseBuilder;

class Pulse {
public:
    static PulseBuilder builder(const std::string& name, const std::string& version);

    Pulse(const ServiceOptions& service_opts, const PulseOptions& pulse_opts);
    ~Pulse();

    Pulse(const Pulse&) = delete;
    Pulse& operator=(const Pulse&) = delete;
    Pulse(Pulse&&) noexcept;
    Pulse& operator=(Pulse&&) noexcept;

    logging::Logger& logger() { return *logger_; }
    metrics::Metrics& metrics() { return *metrics_; }
    tracing::Tracer& tracer() { return *tracer_; }

    std::shared_ptr<mcap::McapWriter> mcap_writer() { return mcap_writer_; }

#if PULSE_USE_OTEL
    otel::OtelExporter* otel_exporter() { return otel_exporter_.get(); }
#endif

    void flush();
    void close();

private:
    std::unique_ptr<logging::Logger> logger_;
    std::unique_ptr<metrics::Metrics> metrics_;
    std::unique_ptr<tracing::Tracer> tracer_;
    std::shared_ptr<mcap::McapWriter> mcap_writer_;
#if PULSE_USE_OTEL
    std::unique_ptr<otel::OtelExporter> otel_exporter_;
#endif
    bool closed_ = false;
};

class PulseBuilder {
public:
    PulseBuilder(const std::string& name, const std::string& version);

    PulseBuilder& description(const std::string& desc);
    PulseBuilder& environment(Environment env);
    PulseBuilder& with_otlp(const std::string& host, uint16_t port);
    PulseBuilder& with_mcap(const std::string& path);

    Pulse build();

private:
    std::string name_;
    std::string version_;
    std::optional<std::string> description_;
    Environment environment_ = Environment::Development;
    std::optional<std::string> otlp_host_;
    std::optional<uint16_t> otlp_port_;
    std::optional<std::string> mcap_path_;
};

}  // namespace pulse
