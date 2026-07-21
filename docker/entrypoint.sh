#!/bin/bash
set -e

SUPERVISOR_CONF="/etc/supervisor/conf.d/supervisord.conf"
EMBEDDED_CONFIG_DIR="/etc/opentelemetry-stack"
STACK_CONFIG_DIR="${STACK_CONFIG_DIR:-/config}"

if [ ! -e "$STACK_CONFIG_DIR" ]; then
    ln -s "$EMBEDDED_CONFIG_DIR" "$STACK_CONFIG_DIR"
fi

export STACK_CONFIG_DIR
export ENVOY_CONFIG="${ENVOY_CONFIG:-${STACK_CONFIG_DIR}/envoy.yaml}"
export GRAFANA_DOMAIN="${GRAFANA_DOMAIN:-127.0.0.1}"

ENABLE_ENVOY="${ENABLE_ENVOY:-true}"
if [ "$ENABLE_ENVOY" = "false" ] || [ "$ENABLE_ENVOY" = "0" ]; then
    sed -i '/\[program:envoy\]/,/priority=/{s/autostart=true/autostart=false/}' "$SUPERVISOR_CONF"
    echo "Envoy disabled"
else
    if ! command -v envoy >/dev/null 2>&1; then
        echo "Error: Envoy binary not found in image. Build/pull an image that includes /usr/local/bin/envoy." >&2
        exit 1
    fi
    if [ -f "${STACK_CONFIG_DIR}/otel-collector.yaml" ] && grep -Eq 'endpoint:[[:space:]]*0\.0\.0\.0:6009' "${STACK_CONFIG_DIR}/otel-collector.yaml"; then
        echo "Error: ${STACK_CONFIG_DIR}/otel-collector.yaml still binds OTLP gRPC to :6009." >&2
        echo "Fix it to :4317 (with HTTP on :4318) when ENABLE_ENVOY=true, then restart the container." >&2
        exit 1
    fi
    sed -i '/\[program:envoy\]/,/priority=/{s/autostart=false/autostart=true/}' "$SUPERVISOR_CONF"
    echo "Envoy enabled on port 6009"
fi

mkdir -p /data/loki /data/tempo /data/prometheus \
    /data/grafana /data/alertmanager /tmp/loki /tmp/tempo

echo "Starting OpenTelemetry stack via supervisord..."
exec /usr/bin/supervisord -c "$SUPERVISOR_CONF"
