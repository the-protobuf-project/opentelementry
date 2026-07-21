cmake_minimum_required(VERSION 3.14)
project(opentelementry-cpp VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

option(OPENTELEMENTRY_BUILD_EXAMPLES "Build examples" ON)
option(OPENTELEMENTRY_BUILD_TESTS "Build tests" OFF)
option(OPENTELEMENTRY_FREERTOS "Build for FreeRTOS" OFF)
option(OPENTELEMENTRY_USE_OTEL "Enable OpenTelemetry support" ON)

include(FetchContent)

if(NOT OPENTELEMENTRY_FREERTOS)
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

set(OPENTELEMENTRY_SOURCES
    src/opentelementry.cpp
    src/logging/logger.cpp
    src/mcap/writer.cpp
    src/metrics/metrics.cpp
    src/tracing/tracer.cpp
    src/otel/exporter.cpp
)

set(OPENTELEMENTRY_HEADERS
    include/opentelementry/opentelementry.hpp
    include/opentelementry/config.hpp
    include/opentelementry/platform.hpp
    include/opentelementry/logging/logger.hpp
    include/opentelementry/logging/formatter.hpp
    include/opentelementry/mcap/writer.hpp
    include/opentelementry/mcap/schemas.hpp
    include/opentelementry/metrics/metrics.hpp
    include/opentelementry/tracing/tracer.hpp
    include/opentelementry/otel/exporter.hpp
)

add_library(opentelementry ${OPENTELEMENTRY_SOURCES} ${OPENTELEMENTRY_HEADERS})
add_library(opentelementry::opentelementry ALIAS opentelementry)

target_include_directories(opentelementry
    PUBLIC
        $<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>
        $<INSTALL_INTERFACE:include>
)

if(OPENTELEMENTRY_FREERTOS)
    target_compile_definitions(opentelementry PUBLIC OPENTELEMENTRY_FREERTOS=1)
else()
    target_link_libraries(opentelementry
        PUBLIC
            spdlog::spdlog
            mcap
    )
endif()

if(OPENTELEMENTRY_USE_OTEL)
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

    target_link_libraries(opentelementry
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
    target_compile_definitions(opentelementry PUBLIC OPENTELEMENTRY_USE_OTEL=1)
endif()

if(OPENTELEMENTRY_BUILD_EXAMPLES)
    add_executable(logging_example examples/logging.cpp)
    target_link_libraries(logging_example PRIVATE opentelementry)

    add_executable(logging_mcap_example examples/logging_mcap.cpp)
    target_link_libraries(logging_mcap_example PRIVATE opentelementry)

    add_executable(metrics_example examples/metrics.cpp)
    target_link_libraries(metrics_example PRIVATE opentelementry)

    add_executable(tracing_example examples/tracing.cpp)
    target_link_libraries(tracing_example PRIVATE opentelementry)
endif()

if(NOT OPENTELEMENTRY_USE_OTEL)
    include(GNUInstallDirs)

    install(TARGETS opentelementry spdlog mcap
        EXPORT opentelementry-targets
        LIBRARY DESTINATION ${CMAKE_INSTALL_LIBDIR}
        ARCHIVE DESTINATION ${CMAKE_INSTALL_LIBDIR}
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )

    install(DIRECTORY include/opentelementry
        DESTINATION ${CMAKE_INSTALL_INCLUDEDIR}
    )

    install(EXPORT opentelementry-targets
        FILE opentelementry-config.cmake
        NAMESPACE opentelementry::
        DESTINATION ${CMAKE_INSTALL_LIBDIR}/cmake/opentelementry
    )
endif()
