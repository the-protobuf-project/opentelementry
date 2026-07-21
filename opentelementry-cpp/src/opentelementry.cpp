#include "opentelementry/opentelementry.hpp"

namespace opentelementry {

Opentelementry::Opentelementry(const ServiceOptions& service_opts, const OpentelementryOptions& opentelementry_opts) {
    if (opentelementry_opts.foxglove.enabled && !opentelementry_opts.foxglove.mcap_path.empty()) {
        mcap_writer_ = std::make_shared<mcap::McapWriter>(service_opts, opentelementry_opts.foxglove.mcap_path);
    }

#if OPENTELEMENTRY_USE_OTEL
    if (opentelementry_opts.telemetry.otlp.enabled) {
        std::string endpoint = "http://" + opentelementry_opts.telemetry.otlp.host + ":" +
                               std::to_string(opentelementry_opts.telemetry.otlp.port);
        otel_exporter_ = std::make_unique<otel::OtelExporter>(service_opts, endpoint);
    }
#endif

    logger_ = std::make_unique<logging::Logger>(
        service_opts.name,
        service_opts.version,
        environment_to_string(service_opts.environment),
        mcap_writer_
#if OPENTELEMENTRY_USE_OTEL
        , otel_exporter_.get()
#endif
    );

    metrics_ = std::make_unique<metrics::Metrics>(service_opts, mcap_writer_
#if OPENTELEMENTRY_USE_OTEL
        , otel_exporter_.get()
#endif
    );

    if (opentelementry_opts.telemetry.otlp.enabled) {
        std::string endpoint = "http://" + opentelementry_opts.telemetry.otlp.host + ":" +
                               std::to_string(opentelementry_opts.telemetry.otlp.port);
        tracer_ = std::make_unique<tracing::Tracer>(service_opts, mcap_writer_, endpoint
#if OPENTELEMENTRY_USE_OTEL
            , otel_exporter_.get()
#endif
        );
    } else {
        tracer_ = std::make_unique<tracing::Tracer>(service_opts, mcap_writer_
#if OPENTELEMENTRY_USE_OTEL
            , otel_exporter_.get()
#endif
        );
    }

    auto global_logger = std::make_unique<logging::Logger>(
        service_opts.name,
        service_opts.version,
        environment_to_string(service_opts.environment),
        mcap_writer_
#if OPENTELEMENTRY_USE_OTEL
        , otel_exporter_.get()
#endif
    );
    logging::GlobalLogger::init(std::move(global_logger));
}

Opentelementry::~Opentelementry() {
    close();
}

Opentelementry::Opentelementry(Opentelementry&& other) noexcept
    : logger_(std::move(other.logger_))
    , metrics_(std::move(other.metrics_))
    , tracer_(std::move(other.tracer_))
    , mcap_writer_(std::move(other.mcap_writer_))
#if OPENTELEMENTRY_USE_OTEL
    , otel_exporter_(std::move(other.otel_exporter_))
#endif
    , closed_(other.closed_) {
    other.closed_ = true;
}

Opentelementry& Opentelementry::operator=(Opentelementry&& other) noexcept {
    if (this != &other) {
        close();
        logger_ = std::move(other.logger_);
        metrics_ = std::move(other.metrics_);
        tracer_ = std::move(other.tracer_);
        mcap_writer_ = std::move(other.mcap_writer_);
#if OPENTELEMENTRY_USE_OTEL
        otel_exporter_ = std::move(other.otel_exporter_);
#endif
        closed_ = other.closed_;
        other.closed_ = true;
    }
    return *this;
}

OpentelementryBuilder Opentelementry::builder(const std::string& name, const std::string& version) {
    return OpentelementryBuilder(name, version);
}

void Opentelementry::flush() {
    if (mcap_writer_) {
        mcap_writer_->flush();
    }
}

void Opentelementry::close() {
    if (closed_) return;

    logging::GlobalLogger::shutdown();

#if OPENTELEMENTRY_USE_OTEL
    if (otel_exporter_) {
        otel_exporter_->shutdown();
    }
#endif

    if (mcap_writer_) {
        mcap_writer_->close();
    }

    closed_ = true;
}

OpentelementryBuilder::OpentelementryBuilder(const std::string& name, const std::string& version)
    : name_(name)
    , version_(version) {
}

OpentelementryBuilder& OpentelementryBuilder::description(const std::string& desc) {
    description_ = desc;
    return *this;
}

OpentelementryBuilder& OpentelementryBuilder::environment(Environment env) {
    environment_ = env;
    return *this;
}

OpentelementryBuilder& OpentelementryBuilder::with_otlp(const std::string& host, uint16_t port) {
    otlp_host_ = host;
    otlp_port_ = port;
    return *this;
}

OpentelementryBuilder& OpentelementryBuilder::with_mcap(const std::string& path) {
    mcap_path_ = path;
    return *this;
}

Opentelementry OpentelementryBuilder::build() {
    ServiceOptions service_opts(name_, version_);
    service_opts.with_environment(environment_);

    if (description_) {
        service_opts.with_description(*description_);
    }

    OpentelementryOptions opentelementry_opts;

    if (otlp_host_ && otlp_port_) {
        opentelementry_opts.telemetry.otlp.enabled = true;
        opentelementry_opts.telemetry.otlp.host = *otlp_host_;
        opentelementry_opts.telemetry.otlp.port = *otlp_port_;
    }

    if (mcap_path_) {
        opentelementry_opts.foxglove.enabled = true;
        opentelementry_opts.foxglove.mcap_path = *mcap_path_;
    }

    return Opentelementry(service_opts, opentelementry_opts);
}

}  // namespace opentelementry
