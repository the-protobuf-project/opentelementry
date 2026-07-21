#pragma once

#include "opentelementry/config.hpp"
#include "opentelementry/logging/logger.hpp"
#include "opentelementry/mcap/writer.hpp"
#include "opentelementry/metrics/metrics.hpp"
#include "opentelementry/tracing/tracer.hpp"

#if OPENTELEMENTRY_USE_OTEL
#include "opentelementry/otel/exporter.hpp"
#endif

#include <memory>
#include <string>
#include <optional>

namespace opentelementry {

class OpentelementryBuilder;

class Opentelementry {
public:
    static OpentelementryBuilder builder(const std::string& name, const std::string& version);

    Opentelementry(const ServiceOptions& service_opts, const OpentelementryOptions& opentelementry_opts);
    ~Opentelementry();

    Opentelementry(const Opentelementry&) = delete;
    Opentelementry& operator=(const Opentelementry&) = delete;
    Opentelementry(Opentelementry&&) noexcept;
    Opentelementry& operator=(Opentelementry&&) noexcept;

    logging::Logger& logger() { return *logger_; }
    metrics::Metrics& metrics() { return *metrics_; }
    tracing::Tracer& tracer() { return *tracer_; }

    std::shared_ptr<mcap::McapWriter> mcap_writer() { return mcap_writer_; }

#if OPENTELEMENTRY_USE_OTEL
    otel::OtelExporter* otel_exporter() { return otel_exporter_.get(); }
#endif

    void flush();
    void close();

private:
    std::unique_ptr<logging::Logger> logger_;
    std::unique_ptr<metrics::Metrics> metrics_;
    std::unique_ptr<tracing::Tracer> tracer_;
    std::shared_ptr<mcap::McapWriter> mcap_writer_;
#if OPENTELEMENTRY_USE_OTEL
    std::unique_ptr<otel::OtelExporter> otel_exporter_;
#endif
    bool closed_ = false;
};

class OpentelementryBuilder {
public:
    OpentelementryBuilder(const std::string& name, const std::string& version);

    OpentelementryBuilder& description(const std::string& desc);
    OpentelementryBuilder& environment(Environment env);
    OpentelementryBuilder& with_otlp(const std::string& host, uint16_t port);
    OpentelementryBuilder& with_mcap(const std::string& path);

    Opentelementry build();

private:
    std::string name_;
    std::string version_;
    std::optional<std::string> description_;
    Environment environment_ = Environment::Development;
    std::optional<std::string> otlp_host_;
    std::optional<uint16_t> otlp_port_;
    std::optional<std::string> mcap_path_;
};

}  // namespace opentelementry
