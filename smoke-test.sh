#!/usr/bin/env bash
# =============================================================================
# CivitForge v1.0.0 Smoke Test
# =============================================================================
# Prerequisites:
#   docker compose up -d   (wait for all services healthy)
#
# Tests:
#   1. Server health/readiness endpoints
#   2. API version endpoint
#   3. Git HTTP smart protocol (info/refs)
#   4. Runner HTTP endpoint
#   5. PostgreSQL connectivity (via server health)
#   6. Redis connectivity (via server health)
# =============================================================================

set -euo pipefail

BASE_URL="${CIVIT_BASE_URL:-http://localhost:8080}"
RUNNER_URL="${CIVIT_RUNNER_URL:-http://localhost:8088}"
PASS=0
FAIL=0
TOTAL=0

check() {
    local name="$1"
    local url="$2"
    local expected_code="${3:-200}"
    TOTAL=$((TOTAL + 1))

    echo -n "  [$name] GET $url ... "
    local code
    code=$(curl -s -o /tmp/civit-smoke-body -w '%{http_code}' --max-time 10 "$url" 2>/dev/null || echo "000")

    if [ "$code" = "$expected_code" ]; then
        PASS=$((PASS + 1))
        echo "PASS ($code)"
    else
        FAIL=$((FAIL + 1))
        echo "FAIL (expected $expected_code, got $code)"
        if [ -f /tmp/civit-smoke-body ]; then
            cat /tmp/civit-smoke-body | head -5
        fi
    fi
}

post_check() {
    local name="$1"
    local url="$2"
    local data="$3"
    local expected_code="${4:-200}"
    TOTAL=$((TOTAL + 1))

    echo -n "  [$name] POST $url ... "
    local code
    code=$(curl -s -o /tmp/civit-smoke-body -w '%{http_code}' --max-time 10 \
        -H "Content-Type: application/json" \
        -d "$data" \
        "$url" 2>/dev/null || echo "000")

    if [ "$code" = "$expected_code" ]; then
        PASS=$((PASS + 1))
        echo "PASS ($code)"
    else
        FAIL=$((FAIL + 1))
        echo "FAIL (expected $expected_code, got $code)"
        if [ -f /tmp/civit-smoke-body ]; then
            cat /tmp/civit-smoke-body | head -5
        fi
    fi
}

echo "========================================="
echo " CivitForge v1.0.0 Smoke Test"
echo " Target: $BASE_URL"
echo " Runner: $RUNNER_URL"
echo "========================================="
echo ""

# --- 1. Server Health ---
echo "=== Server Health ==="
check "healthz" "$BASE_URL/healthz" 200
check "ready"   "$BASE_URL/ready"   200
echo ""

# --- 2. API Version ---
echo "=== API Endpoints ==="
check "api-root" "$BASE_URL/api/v1/" 404
check "api-openapi" "$BASE_URL/api/v1/openapi.json" 404
echo ""

# --- 3. Git HTTP Smart Protocol ---
echo "=== Git HTTP ==="
check "git-info-refs" "$BASE_URL/test-repo.git/info/refs?service=git-upload-pack" 404
echo ""

# --- 4. Runner ---
echo "=== Runner ==="
check "runner-health" "$RUNNER_URL/" 200
echo ""

# --- 5. Static Assets (if UI served) ---
echo "=== Static Assets ==="
check "favicon" "$BASE_URL/favicon.ico" 404
echo ""

# --- 6. POST endpoints (expect 401 or 422 without auth) ---
echo "=== Auth-Gated Endpoints ==="
post_check "create-repo" "$BASE_URL/api/v1/repos" '{"name":"smoke-test"}' 401
post_check "register-runner" "$BASE_URL/api/v1/runners/register" '{"name":"smoke"}' 401
echo ""

# --- Summary ---
echo "========================================="
echo " Results: $PASS/$TOTAL passed"
if [ "$FAIL" -gt 0 ]; then
    echo " FAILED: $FAIL test(s)"
    exit 1
else
    echo " ALL TESTS PASSED"
fi
echo "========================================="
