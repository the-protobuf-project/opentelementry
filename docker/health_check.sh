#!/bin/bash
set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

PASS=0
FAIL=0
grafana_ready=0
otelcol_ready=0
loki_ready=0
tempo_ready=0
prometheus_ready_local=0
alertmanager_ready=0
envoy_ready=0

METRICS_PUSH_URL="${HEALTH_METRICS_PUSHGATEWAY_URL:-}"
METRICS_JOB="${HEALTH_METRICS_JOB:-opentelemetry-health}"
METRICS_INSTANCE="${HEALTH_METRICS_INSTANCE:-${HOSTNAME:-opentelemetry-stack}}"
METRICS_PUSH_STRICT="${HEALTH_METRICS_PUSH_STRICT:-false}"
ROBOT_ID="${ROBOT_ID:-unknown}"
FLEET_ID="${FLEET_ID:-unknown}"
REGION="${REGION:-unknown}"

check() {
    local name="$1"
    local cmd="$2"
    local metric_var="$3"

    if eval "$cmd" >/dev/null 2>&1; then
        printf "${GREEN}✓${NC} %-20s %s\n" "$name" "healthy"
        PASS=$((PASS + 1))
        eval "$metric_var=1"
    else
        printf "${RED}✗${NC} %-20s %s\n" "$name" "unhealthy"
        FAIL=$((FAIL + 1))
        eval "$metric_var=0"
    fi
}

echo "─── Health Check ───────────────────────────"

# Required order:
# grafana -> otel collector -> loki -> tempo -> prometheus -> alertmanager
check "grafana"       "curl -sf http://localhost:3000/api/health"   "grafana_ready"
check "otelcol"       "curl -sf http://localhost:13133/"            "otelcol_ready"
check "loki"          "curl -sf http://localhost:3101/ready"        "loki_ready"
check "tempo"         "curl -sf http://localhost:3201/ready"        "tempo_ready"
check "prometheus"    "curl -sf http://localhost:9090/-/healthy"    "prometheus_ready_local"
check "alertmanager"  "curl -sf http://localhost:9093/-/healthy"    "alertmanager_ready"

if [ "${ENABLE_ENVOY:-false}" = "true" ]; then
    check "envoy"     "curl -sf http://localhost:9901/ready"         "envoy_ready"
fi

push_metrics() {
    [ -z "$METRICS_PUSH_URL" ] && return 0

    local stack_ready=0
    if [ "$FAIL" -eq 0 ]; then
        stack_ready=1
    fi

    local payload
    payload="$(cat <<EOF
# TYPE thirdparty_opentelemetry_stack_ready gauge
thirdparty_opentelemetry_stack_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $stack_ready
# TYPE thirdparty_opentelemetry_grafana_ready gauge
thirdparty_opentelemetry_grafana_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $grafana_ready
# TYPE thirdparty_opentelemetry_otelcol_ready gauge
thirdparty_opentelemetry_otelcol_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $otelcol_ready
# TYPE thirdparty_opentelemetry_loki_ready gauge
thirdparty_opentelemetry_loki_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $loki_ready
# TYPE thirdparty_opentelemetry_tempo_ready gauge
thirdparty_opentelemetry_tempo_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $tempo_ready
# TYPE thirdparty_opentelemetry_prometheus_ready gauge
thirdparty_opentelemetry_prometheus_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $prometheus_ready_local
# TYPE thirdparty_opentelemetry_alertmanager_ready gauge
thirdparty_opentelemetry_alertmanager_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $alertmanager_ready
# TYPE thirdparty_opentelemetry_envoy_ready gauge
thirdparty_opentelemetry_envoy_ready{robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $envoy_ready
# TYPE thirdparty_service_ready gauge
thirdparty_service_ready{service="opentelemetry",robot_id="$ROBOT_ID",fleet_id="$FLEET_ID",region="$REGION"} $stack_ready
EOF
)"

    if ! curl -fsS --data-binary "${payload}"$'\n' \
        "${METRICS_PUSH_URL%/}/metrics/job/${METRICS_JOB}/instance/${METRICS_INSTANCE}" >/dev/null; then
        echo "opentelemetry health metrics push failed: ${METRICS_PUSH_URL}" >&2
        if [ "$METRICS_PUSH_STRICT" = "true" ] || [ "$METRICS_PUSH_STRICT" = "1" ]; then
            return 1
        fi
    fi

    return 0
}

echo "─────────────────────────────────────────────"
printf "Total: %d passed, %d failed\n" "$PASS" "$FAIL"

push_metrics

[ "$FAIL" -eq 0 ] && exit 0 || exit 1
