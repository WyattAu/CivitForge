#!/usr/bin/env bash
# =============================================================================
# CivitForge v1.1.0 Smoke Test
# =============================================================================
# Prerequisites:
#   docker compose up -d   (wait for all services healthy)
#   cargo run --release -p civit-core   (server on 127.0.0.1:9091)
#
# Tests:
#   1. Server health/readiness endpoints
#   2. Auth: login (auto-register) → token
#   3. Repos: create → get → list → delete
#   4. Search
#   5. Wiki (404 on missing)
#   6. Pipelines (empty list)
#   7. Git HTTP smart protocol (404 without repo)
#   8. Users list
# =============================================================================

set -euo pipefail

BASE_URL="${CIVIT_BASE_URL:-http://127.0.0.1:9091}"
PASS=0
FAIL=0
TOTAL=0
TOKEN=""

smoke_get() {
    local name="$1" url="$2" expected="${3:-200}" auth="${4:-no}"
    TOTAL=$((TOTAL + 1))
    local auth_args=()
    if [ "$auth" = "yes" ] && [ -n "$TOKEN" ]; then
        auth_args=(-H "Authorization: Bearer $TOKEN")
    fi
    echo -n "  [$name] GET $url ... "
    local code
    code=$(curl -s -o /tmp/civit-smoke-body -w '%{http_code}' --max-time 10 \
        "${auth_args[@]}" "$url" 2>/dev/null || echo "000")
    if [ "$code" = "$expected" ]; then
        PASS=$((PASS + 1)); echo "PASS ($code)"
    else
        FAIL=$((FAIL + 1)); echo "FAIL (expected $expected, got $code)"
        [ -f /tmp/civit-smoke-body ] && head -3 /tmp/civit-smoke-body
    fi
}

smoke_post() {
    local name="$1" url="$2" data="$3" expected="${4:-200}" auth="${5:-no}"
    TOTAL=$((TOTAL + 1))
    local auth_args=()
    if [ "$auth" = "yes" ] && [ -n "$TOKEN" ]; then
        auth_args=(-H "Authorization: Bearer $TOKEN")
    fi
    echo -n "  [$name] POST $url ... "
    local code
    code=$(curl -s -o /tmp/civit-smoke-body -w '%{http_code}' --max-time 10 \
        -H "Content-Type: application/json" \
        "${auth_args[@]}" \
        -d "$data" "$url" 2>/dev/null || echo "000")
    if [ "$code" = "$expected" ]; then
        PASS=$((PASS + 1)); echo "PASS ($code)"
    else
        FAIL=$((FAIL + 1)); echo "FAIL (expected $expected, got $code)"
        [ -f /tmp/civit-smoke-body ] && head -3 /tmp/civit-smoke-body
    fi
}

smoke_delete() {
    local name="$1" url="$2" expected="${3:-204}"
    TOTAL=$((TOTAL + 1))
    echo -n "  [$name] DELETE $url ... "
    local code
    code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 \
        -H "Authorization: Bearer $TOKEN" \
        "$url" 2>/dev/null || echo "000")
    if [ "$code" = "$expected" ]; then
        PASS=$((PASS + 1)); echo "PASS ($code)"
    else
        FAIL=$((FAIL + 1)); echo "FAIL (expected $expected, got $code)"
    fi
}

echo "========================================="
echo " CivitForge v1.1.0 Smoke Test"
echo " Target: $BASE_URL"
echo "========================================="
echo ""

# --- 1. Server Health ---
echo "=== Server Health ==="
smoke_get "healthz"     "$BASE_URL/healthz" 200
smoke_get "ready"       "$BASE_URL/ready"   200
smoke_get "api-health"  "$BASE_URL/api/v1/health" 200
echo ""

# --- 2. Auth (auto-register on login) ---
echo "=== Auth (auto-register on login) ==="
TS="$(date +%s)"
SMOKE_USER="smoke-${TS}"
SMOKE_EMAIL="smoke-${TS}@test.dev"
REPO_NAME="smoke-${TS}"

smoke_post "login" "$BASE_URL/api/v1/auth/login" \
    "{\"username\":\"$SMOKE_USER\",\"email\":\"$SMOKE_EMAIL\",\"display_name\":\"Smoke Tester\"}" 200

TOKEN=$(python3 -c "
import json,sys
try:
    d=json.load(open('/tmp/civit-smoke-body'))
    print(d.get('token',''))
except: print('')" 2>/dev/null)

if [ -n "$TOKEN" ]; then
    echo "  [auth] Token acquired ✓"
else
    echo "  [auth] WARNING: no token — auth-gated tests may fail"
fi
echo ""

# --- 3. Repos CRUD ---
echo "=== Repos (CRUD) ==="
smoke_post "create-repo" "$BASE_URL/api/v1/repos" \
    "{\"name\":\"$REPO_NAME\",\"owner\":\"$SMOKE_USER\",\"description\":\"Smoke test repo\",\"visibility\":\"private\"}" 201 yes

smoke_get "list-repos"  "$BASE_URL/api/v1/repos" 200 yes
smoke_get "get-repo"    "$BASE_URL/api/v1/repos/$SMOKE_USER/$REPO_NAME" 200 yes
smoke_get "search"      "$BASE_URL/api/v1/search?q=smoke" 200 yes
smoke_delete "delete-repo" "$BASE_URL/api/v1/repos/$SMOKE_USER/$REPO_NAME" 200
echo ""

# --- 4. Pipelines ---
echo "=== Pipelines ==="
smoke_get "pipelines-list" "$BASE_URL/api/v1/pipelines" 200 yes
echo ""

# --- 5. Wiki (deleted repo → 404) ---
echo "=== Wiki ==="
smoke_get "wiki-deleted" "$BASE_URL/api/v1/repos/$SMOKE_USER/$REPO_NAME/pages" 404 yes
echo ""

# --- 6. Git HTTP Smart Protocol ---
echo "=== Git HTTP ==="
smoke_get "git-info-refs" "$BASE_URL/test-repo.git/info/refs?service=git-upload-pack" 404
echo ""

# --- 7. Users ---
echo "=== Users ==="
smoke_get "users-list" "$BASE_URL/api/v1/users" 200 yes
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
