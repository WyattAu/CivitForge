#!/usr/bin/env bash
# =============================================================================
# CivitForge Health Check & Performance Monitoring Dashboard
# =============================================================================
# Checks all service endpoints, measures response times, verifies database
# and Redis connectivity, and reports system resource usage.
#
# Usage:
#   scripts/monitoring/health_check.sh              # full report (stdout JSON)
#   scripts/monitoring/health_check.sh --compact     # one-line summary
#   scripts/monitoring/health_check.sh --watch 5     # re-run every N seconds
#
# Prerequisites: curl, jq, df, free, pg_isready (optional), redis-cli (optional)
# =============================================================================

set -euo pipefail

# ── Configuration ────────────────────────────────────────────────────────────

BASE_URL="${CIVIT_BASE_URL:-http://127.0.0.1:9091}"
DB_HOST="${DB_HOST:-127.0.0.1}"
DB_PORT="${DB_PORT:-5432}"
DB_USER="${DB_USER:-civit}"
REDIS_HOST="${REDIS_HOST:-127.0.0.1}"
REDIS_PORT="${REDIS_PORT:-6379}"
REDIS_PASSWORD="${REDIS_PASSWORD:-}"
TIMEOUT_SECONDS="${HEALTH_TIMEOUT:-5}"
COMPACT=false
WATCH_INTERVAL=""

for arg in "$@"; do
    case "$arg" in
        --compact) COMPACT=true ;;
        --watch) WATCH_INTERVAL="next" ;;
        --help|-h)
            echo "Usage: $0 [--compact] [--watch <seconds>]"
            exit 0
            ;;
    esac
    if [[ "$WATCH_INTERVAL" == "next" ]]; then
        WATCH_INTERVAL="$arg"
    fi
done

# ── Helpers ──────────────────────────────────────────────────────────────────

now_iso() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

measure_endpoint() {
    local url="$1" name="$2"
    local start_ms end_ms duration_ms http_code body
    start_ms=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))' 2>/dev/null || echo 0)
    body=$(curl -s -o /tmp/civit-hc-body -w '%{http_code}' --max-time "$TIMEOUT_SECONDS" "$url" 2>/dev/null || echo "000")
    end_ms=$(date +%s%3N 2>/dev/null || python3 -c 'import time; print(int(time.time()*1000))' 2>/dev/null || echo 0)
    http_code="$body"
    if [[ "$start_ms" -gt 0 && "$end_ms" -gt 0 ]]; then
        duration_ms=$(( end_ms - start_ms ))
    else
        duration_ms=0
    fi
    printf '{"name":"%s","url":"%s","status_code":%s,"response_ms":%s}' "$name" "$url" "$http_code" "$duration_ms"
}

check_db() {
    if command -v pg_isready &>/dev/null; then
        if pg_isready -h "$DB_HOST" -p "$DB_PORT" -U "$DB_USER" -q 2>/dev/null; then
            echo '{"reachable":true,"latency_ms":0}'
        else
            echo '{"reachable":false,"latency_ms":0}'
        fi
    else
        echo '{"reachable":null,"latency_ms":0,"note":"pg_isready not installed"}'
    fi
}

check_redis() {
    if command -v redis-cli &>/dev/null; then
        local auth_args=()
        if [[ -n "$REDIS_PASSWORD" ]]; then
            auth_args=(-a "$REDIS_PASSWORD")
        fi
        local start_ms end_ms duration_ms
        start_ms=$(date +%s%3N 2>/dev/null || echo 0)
        local output
        output=$(redis-cli -h "$REDIS_HOST" -p "$REDIS_PORT" "${auth_args[@]}" ping 2>/dev/null || echo "ERR")
        end_ms=$(date +%s%3N 2>/dev/null || echo 0)
        if [[ "$start_ms" -gt 0 && "$end_ms" -gt 0 ]]; then
            duration_ms=$(( end_ms - start_ms ))
        else
            duration_ms=0
        fi
        if [[ "$output" == "PONG" ]]; then
            printf '{"reachable":true,"latency_ms":%s}' "$duration_ms"
        else
            printf '{"reachable":false,"latency_ms":%s,"error":"%s"}' "$duration_ms" "$output"
        fi
    else
        echo '{"reachable":null,"latency_ms":0,"note":"redis-cli not installed"}'
    fi
}

check_disk() {
    local total used avail pct mount
    read -r total used avail pct mount < <(df -BM / | awk 'NR==2 {gsub(/%/,"",$5); print $2,$3,$4,$5,$6}')
    printf '{"mount":"%s","total_mb":%s,"used_mb":%s,"avail_mb":%s,"use_percent":%s}' \
        "$mount" "$total" "$used" "$avail" "$pct"
}

check_memory() {
    local total used avail pct
    read -r total used avail pct < <(free -m | awk '/^Mem:/ {gsub(/%/,"",$3/$2*100); printf "%s %s %s %d", $2, $3, $7, $3/$2*100}')
    printf '{"total_mb":%s,"used_mb":%s,"avail_mb":%s,"use_percent":%s}' \
        "$total" "$used" "$avail" "$pct"
}

# ── Run Checks ───────────────────────────────────────────────────────────────

run_checks() {
    local timestamp
    timestamp=$(now_iso)

    # Endpoint checks
    local healthz ready api_health repos search
    healthz=$(measure_endpoint "$BASE_URL/healthz" "healthz")
    ready=$(measure_endpoint "$BASE_URL/ready" "ready")
    api_health=$(measure_endpoint "$BASE_URL/api/v1/health" "api-health")
    repos=$(measure_endpoint "$BASE_URL/api/v1/repos" "repos-list")
    search=$(measure_endpoint "$BASE_URL/api/v1/search?q=health" "search")

    # Infrastructure checks
    local db_check redis_check
    db_check=$(check_db)
    redis_check=$(check_redis)

    # System resource checks
    local disk_check mem_check
    disk_check=$(check_disk)
    mem_check=$(check_memory)

    # Determine overall status
    local overall="healthy"
    if echo "$healthz" | grep -q '"status_code":5'; then
        overall="degraded"
    fi

    cat <<EOF
{
  "timestamp": "$timestamp",
  "overall_status": "$overall",
  "base_url": "$BASE_URL",
  "endpoints": [
    $healthz,
    $ready,
    $api_health,
    $repos,
    $search
  ],
  "database": $db_check,
  "redis": $redis_check,
  "system": {
    "disk": $disk_check,
    "memory": $mem_check
  }
}
EOF
}

# ── Compact Output ───────────────────────────────────────────────────────────

compact_summary() {
    local result
    result=$(run_checks)
    local status health_ms db_ok redis_ok disk_pct mem_pct
    status=$(echo "$result" | jq -r '.overall_status')
    health_ms=$(echo "$result" | jq '.endpoints[0].response_ms')
    db_ok=$(echo "$result" | jq '.database.reachable')
    redis_ok=$(echo "$result" | jq '.redis.reachable')
    disk_pct=$(echo "$result" | jq '.system.disk.use_percent')
    mem_pct=$(echo "$result" | jq '.system.memory.use_percent')

    printf "[%s] status=%-10s health=%sms db=%-5s redis=%-5s disk=%s%% mem=%s%%\n" \
        "$(date +%H:%M:%S)" "$status" "$health_ms" "$db_ok" "$redis_ok" "$disk_pct" "$mem_pct"
}

# ── Main ─────────────────────────────────────────────────────────────────────

if [[ "$COMPACT" == "true" ]]; then
    compact_summary
elif [[ -n "$WATCH_INTERVAL" ]]; then
    while true; do
        compact_summary
        sleep "$WATCH_INTERVAL"
    done
else
    run_checks
fi
