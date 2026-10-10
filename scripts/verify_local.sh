#!/usr/bin/env bash
# =============================================================================
# Local verification: build, boot, and run the suites that need a live server.
#
# Exists because the manual procedure is easy to get subtly wrong (wrong port,
# missing migration, E2E pointed at a stale instance) and a suite that ran
# against the wrong server is worse than no suite.
#
# Usage:
#   ./scripts/verify_local.sh            # unit + E2E + OFREP interop
#   ./scripts/verify_local.sh --no-e2e   # skip Playwright
#
# Requires: docker, cargo, node.
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$REPO_ROOT"

PORT="${CIVIT_PORT:-9091}"
BASE_URL="http://127.0.0.1:${PORT}"
DB_CONTAINER="civit-verify-postgres"
DB_PORT="${CIVIT_VERIFY_DB_PORT:-5434}"
STORAGE_DIR="${CIVIT_VERIFY_STORAGE:-/tmp/civitforge-verify}"
RUN_E2E=1
[[ "${1:-}" == "--no-e2e" ]] && RUN_E2E=0

JWT_SECRET="verify-only-secret-32-bytes-minimum-xxxxx"
SERVER_PID=""
SERVER_LOG=""

log() { echo -e "\033[1;36m==>\033[0m $*"; }
fail() { echo -e "\033[1;31mFAIL\033[0m $*" >&2; exit 1; }

cleanup() {
  if [[ -n "$SERVER_PID" ]] && kill -0 "$SERVER_PID" 2>/dev/null; then
    log "stopping server (pid $SERVER_PID)"
    kill "$SERVER_PID" 2>/dev/null || true
    wait "$SERVER_PID" 2>/dev/null || true
  fi
  if [[ "${CIVIT_VERIFY_KEEP_DB:-0}" != "1" ]]; then
    docker rm -f "$DB_CONTAINER" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

# -----------------------------------------------------------------------------
# 1. Database
# -----------------------------------------------------------------------------
log "starting postgres on ${DB_PORT}"
docker rm -f "$DB_CONTAINER" >/dev/null 2>&1 || true
docker run -d --name "$DB_CONTAINER" \
  -e POSTGRES_USER=civit -e POSTGRES_PASSWORD=test -e POSTGRES_DB=civit_test \
  -p "${DB_PORT}:5432" postgres:16-alpine >/dev/null

for _ in $(seq 1 30); do
  if docker exec "$DB_CONTAINER" pg_isready -U civit >/dev/null 2>&1; then break; fi
  sleep 1
done
docker exec "$DB_CONTAINER" pg_isready -U civit >/dev/null 2>&1 \
  || fail "postgres did not become ready"
log "postgres ready"

# -----------------------------------------------------------------------------
# 2. Build
#
# Memory-shaped on purpose: the dev profile's default debug info plus a
# parallel link is enough to get rustc OOM-killed on a busy host, and a killed
# link looks exactly like a hung build.
# -----------------------------------------------------------------------------
log "building server (debug info off, 2 jobs)"
CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_DEV_CODEGEN_UNITS=16 \
  cargo build -p civit-core --bin civit-core -j 2

mkdir -p "$STORAGE_DIR"

# -----------------------------------------------------------------------------
# 3. Boot
# -----------------------------------------------------------------------------
log "starting server on ${PORT}"
SERVER_LOG="$(mktemp -t civitforge-verify-XXXXXX.log)"
DATABASE_URL="postgres://civit:test@127.0.0.1:${DB_PORT}/civit_test" \
JWT_SECRET="$JWT_SECRET" \
CIVIT_PORT="$PORT" \
CIVIT_HOST="127.0.0.1" \
CIVIT_STORAGE_PATH="$STORAGE_DIR" \
REDIS_URL="redis://127.0.0.1:6379" \
CIVIT_ROLLOUT_CONTROLLER="false" \
  ./target/debug/civit-core >"$SERVER_LOG" 2>&1 &
SERVER_PID=$!

for _ in $(seq 1 90); do
  code="$(curl -s -o /dev/null -w '%{http_code}' "${BASE_URL}/healthz" || true)"
  [[ "$code" == "200" ]] && break
  if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    log "server died during startup; last 40 log lines:"
    tail -40 "$SERVER_LOG"
    fail "server failed to start"
  fi
  sleep 1
done
[[ "$(curl -s -o /dev/null -w '%{http_code}' "${BASE_URL}/healthz")" == "200" ]] \
  || { tail -40 "$SERVER_LOG"; fail "server never became healthy"; }
log "server healthy: $(curl -s "${BASE_URL}/api/v1/version")"

# -----------------------------------------------------------------------------
# 4. Prometheus exposition
#
# The scrape endpoint can be present, return 200, and still serve nothing —
# that is exactly how the hand-rolled counters hid (metrics_registered stayed
# 0 forever while every request "recorded"). Drive traffic and assert the
# counter actually appears with a value.
# -----------------------------------------------------------------------------
log "Prometheus exposition"
drive_traffic() {
  for _ in $(seq 1 30); do
    curl -s -o /dev/null "${BASE_URL}/api/v1/version" &
    sleep 0.2
  done
  wait
}
drive_traffic
METRICS_BODY="$(curl -s "${BASE_URL}/api/v1/metrics/prometheus")"
if [[ "$METRICS_BODY" != *"http_server_requests_total"* ]]; then
  log "exposition missing http_server_requests_total after traffic:"
  echo "$METRICS_BODY" | head -20
  fail "Prometheus exposition carries no request counter"
fi
if ! grep -q 'service_name="civitforge"' <<<"$METRICS_BODY"; then
  fail "target_info must carry the service name so a scrape is attributable"
fi
log "exposition serves the request counter with service attribution"

# -----------------------------------------------------------------------------
# 5. OFREP interop (third-party provider)
# -----------------------------------------------------------------------------
if [[ -d node_modules/@openfeature/ofrep-provider ]]; then
  log "OFREP interop (community provider)"
  CIVITFORGE_URL="$BASE_URL" node scripts/ofrep_interop.mjs || fail "OFREP interop failed"
else
  log "SKIP OFREP interop: run 'npm install @openfeature/server-sdk @openfeature/ofrep-provider'"
fi

# -----------------------------------------------------------------------------
# 6. Playwright E2E
# -----------------------------------------------------------------------------
if [[ "$RUN_E2E" == "1" ]]; then
  log "Playwright E2E against ${BASE_URL}"
  ( cd tests/e2e && CIVITFORGE_URL="$BASE_URL" npx playwright test --project=chromium ) \
    || fail "E2E failed (full log: tests/e2e/reports/playwright)"
else
  log "skipping E2E (--no-e2e)"
fi

log "all local verification passed"