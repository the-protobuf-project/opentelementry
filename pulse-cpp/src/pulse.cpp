#include "pulse/pulse.hpp"

namespace pulse {

Pulse::Pulse(const ServiceOptions& service_opts, const PulseOptions& pulse_opts) {
    if (pulse_opts.foxglove.enabled && !pulse_opts.foxglove.mcap_path.empty()) {
        mcap_writer_ = std::make_shared<mcap::McapWriter>(service_opts, pulse_opts.foxglove.mcap_path);
    }

#if PULSE_USE_OTEL
    if (pulse_opts.telemetry.otlp.enabled) {
        std::string endpoint = "http://" + pulse_opts.telemetry.otlp.host + ":" + 
                               std::to_string(pulse_opts.telemetry.otlp.port);
        otel_exporter_ = std::make_unique<otel::OtelExporter>(service_opts, endpoint);
    }
#endif

    logger_ = std::make_unique<logging::Logger>(
        service_opts.name,
        service_opts.version,
        environment_to_string(service_opts.environment),
        mcap_writer_
#if PULSE_USE_OTEL
        , otel_exporter_.get()
#endif
    );

    metrics_ = std::make_unique<metrics::Metrics>(service_opts, mcap_writer_
#if PULSE_USE_OTEL
        , otel_exporter_.get()
#endif
    );

    if (pulse_opts.telemetry.otlp.enabled) {
        std::string endpoint = "http://" + pulse_opts.telemetry.otlp.host + ":" + 
                               std::to_string(pulse_opts.telemetry.otlp.port);
        tracer_ = std::make_unique<tracing::Tracer>(service_opts, mcap_writer_, endpoint
#if PULSE_USE_OTEL
            , otel_exporter_.get()
#endif
        );
    } else {
        tracer_ = std::make_unique<tracing::Tracer>(service_opts, mcap_writer_
#if PULSE_USE_OTEL
            , otel_exporter_.get()
#endif
        );
    }

    auto global_logger = std::make_unique<logging::Logger>(
        service_opts.name,
        service_opts.version,
        environment_to_string(service_opts.environment),
        mcap_writer_
#if PULSE_USE_OTEL
        , otel_exporter_.get()
#endif
    );
    logging::GlobalLogger::init(std::move(global_logger));
}

Pulse::~Pulse() {
    close();
}

Pulse::Pulse(Pulse&& other) noexcept
    : logger_(std::move(other.logger_))
    , metrics_(std::move(other.metrics_))
    , tracer_(std::move(other.tracer_))
    , mcap_writer_(std::move(other.mcap_writer_))
#if PULSE_USE_OTEL
    , otel_exporter_(std::move(other.otel_exporter_))
#endif
    , closed_(other.closed_) {
    other.closed_ = true;
}

Pulse& Pulse::operator=(Pulse&& other) noexcept {
    if (this != &other) {
        close();
        logger_ = std::move(other.logger_);
        metrics_ = std::move(other.metrics_);
        tracer_ = std::move(other.tracer_);
        mcap_writer_ = std::move(other.mcap_writer_);
#if PULSE_USE_OTEL
        otel_exporter_ = std::move(other.otel_exporter_);
#endif
        closed_ = other.closed_;
        other.closed_ = true;
    }
    return *this;
}

PulseBuilder Pulse::builder(const std::string& name, const std::string& version) {
    return PulseBuilder(name, version);
}

void Pulse::flush() {
    if (mcap_writer_) {
        mcap_writer_->flush();
    }
}

void Pulse::close() {
    if (closed_) return;
    
    logging::GlobalLogger::shutdown();
    
#if PULSE_USE_OTEL
    if (otel_exporter_) {
        otel_exporter_->shutdown();
    }
#endif
    
    if (mcap_writer_) {
        mcap_writer_->close();
    }
    
    closed_ = true;
}

PulseBuilder::PulseBuilder(const std::string& name, const std::string& version)
    : name_(name)
    , version_(version) {
}

PulseBuilder& PulseBuilder::description(const std::string& desc) {
    description_ = desc;
    return *this;
}

PulseBuilder& PulseBuilder::environment(Environment env) {
    environment_ = env;
    return *this;
}

PulseBuilder& PulseBuilder::with_otlp(const std::string& host, uint16_t port) {
    otlp_host_ = host;
    otlp_port_ = port;
    return *this;
}

PulseBuilder& PulseBuilder::with_mcap(const std::string& path) {
    mcap_path_ = path;
    return *this;
}

Pulse PulseBuilder::build() {
    ServiceOptions service_opts(name_, version_);
    service_opts.with_environment(environment_);
    
    if (description_) {
        service_opts.with_description(*description_);
    }

    PulseOptions pulse_opts;

    if (otlp_host_ && otlp_port_) {
        pulse_opts.telemetry.otlp.enabled = true;
        pulse_opts.telemetry.otlp.host = *otlp_host_;
        pulse_opts.telemetry.otlp.port = *otlp_port_;
    }

    if (mcap_path_) {
        pulse_opts.foxglove.enabled = true;
        pulse_opts.foxglove.mcap_path = *mcap_path_;
    }

    return Pulse(service_opts, pulse_opts);
}

}  // namespace pulse
