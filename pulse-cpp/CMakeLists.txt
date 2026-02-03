cmake_minimum_required(VERSION 3.14)
project(pulse-cpp VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(PULSE_BUILD_EXAMPLES "Build examples" ON)
option(PULSE_BUILD_TESTS "Build tests" OFF)
option(PULSE_FREERTOS "Build for FreeRTOS" OFF)
option(PULSE_USE_OTEL "Enable OpenTelemetry support" ON)

include(FetchContent)

if(NOT PULSE_FREERTOS)
    FetchContent_Declare(
        spdlog
        GIT_REPOSITORY https://github.com/gabime/spdlog.git
        GIT_TAG v1.14.1
    )
    FetchContent_MakeAvailable(spdlog)

    FetchContent_Declare(
        mcap
        GIT_REPOSITORY https://github.com/foxglove/mcap.git
        GIT_TAG releases/cpp/v1.4.1
    )
    FetchContent_MakeAvailable(mcap)

    add_library(mcap INTERFACE)
    target_include_directories(mcap INTERFACE
        $<BUILD_INTERFACE:${mcap_SOURCE_DIR}/cpp/mcap/include>
        $<INSTALL_INTERFACE:include>
    )
endif()

set(PULSE_SOURCES
    src/pulse.cpp
    src/logging/logger.cpp
    src/mcap/writer.cpp
    src/metrics/metrics.cpp
    src/tracing/tracer.cpp
    src/otel/exporter.cpp
)

set(PULSE_HEADERS
    include/pulse/pulse.hpp
    include/pulse/config.hpp
    include/pulse/platform.hpp
    include/pulse/logging/logger.hpp
    include/pulse/logging/formatter.hpp
    include/pulse/mcap/writer.hpp
    include/pulse/mcap/schemas.hpp
    include/pulse/metrics/metrics.hpp
    include/pulse/tracing/tracer.hpp
    include/pulse/otel/exporter.hpp
)

add_library(pulse ${PULSE_SOURCES} ${PULSE_HEADERS})
add_library(pulse::pulse ALIAS pulse)

target_include_directories(pulse
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)

if(PULSE_FREERTOS)
    target_compile_definitions(pulse PUBLIC PULSE_FREERTOS=1)
else()
    target_link_libraries(pulse
        PUBLIC
            spdlog::spdlog
            mcap
    )
endif()

if(PULSE_USE_OTEL)
    set(WITH_OTLP_HTTP ON CACHE BOOL "Enable OTLP HTTP exporter" FORCE)
    set(WITH_OTLP_GRPC OFF CACHE BOOL "Disable OTLP gRPC exporter" FORCE)
    set(BUILD_TESTING OFF CACHE BOOL "Disable OTel tests" FORCE)
    set(WITH_EXAMPLES OFF CACHE BOOL "Disable OTel examples" FORCE)
    set(WITH_BENCHMARK OFF CACHE BOOL "Disable OTel benchmarks" FORCE)

    FetchContent_Declare(
        opentelemetry-cpp
        GIT_REPOSITORY https://github.com/open-telemetry/opentelemetry-cpp.git
        GIT_TAG v1.18.0
    )
    FetchContent_MakeAvailable(opentelemetry-cpp)

    target_link_libraries(pulse
        PUBLIC
            opentelemetry_api
            opentelemetry_sdk
            opentelemetry_exporter_otlp_http
            opentelemetry_exporter_otlp_http_log
            opentelemetry_exporter_otlp_http_metric
            opentelemetry_logs
            opentelemetry_trace
            opentelemetry_metrics
    )
    target_compile_definitions(pulse PUBLIC PULSE_USE_OTEL=1)
endif()

if(PULSE_BUILD_EXAMPLES)
    add_executable(logging_example examples/logging.cpp)
    target_link_libraries(logging_example PRIVATE pulse)

    add_executable(logging_mcap_example examples/logging_mcap.cpp)
    target_link_libraries(logging_mcap_example PRIVATE pulse)

    add_executable(metrics_example examples/metrics.cpp)
    target_link_libraries(metrics_example PRIVATE pulse)

    add_executable(tracing_example examples/tracing.cpp)
    target_link_libraries(tracing_example PRIVATE pulse)
endif()

if(NOT PULSE_USE_OTEL)
    include(GNUInstallDirs)

    install(TARGETS pulse spdlog mcap
        EXPORT pulse-targets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )

    install(DIRECTORY include/pulse
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    )

    install(EXPORT pulse-targets
        FILE pulse-config.cmake
        NAMESPACE pulse::
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/pulse
    )
endif()
