#!/usr/bin/env bash
# =============================================================================
# CivitForge DR Validation Test
# =============================================================================
# Validates disaster recovery by creating backups, simulating corruption,
# restoring, and measuring RTO/RPO.
#
# Usage:
#   scripts/dr_validate.sh [--compose-file FILE] [--db-name NAME]
#
# Prerequisites: docker compose, pg_dump, pg_restore, psql, jq
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
COMPOSE_FILE="${COMPOSE_FILE:-$REPO_ROOT/docker-compose.yml}"
DB_NAME="${DB_NAME:-civit}"
DB_USER="${DB_USER:-civit}"
BACKUP_DIR="/tmp/civit-dr-backups"
TEST_TABLE="dr_test_markers"
PASS=0
FAIL=0
TOTAL=0

for arg in "$@"; do
  case "$arg" in
    --compose-file) COMPOSE_FILE="$2"; shift 2 ;;
    --db-name) DB_NAME="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 [--compose-file FILE] [--db-name NAME]"
      exit 0
      ;;
  esac
done

log()  { echo -e "\033[1;34m[DR-VAL]\033[0m $*"; }
ok()   { echo -e "\033[1;32m[PASS]\033[0m $*"; PASS=$((PASS + 1)); TOTAL=$((TOTAL + 1)); }
fail() { echo -e "\033[1;31m[FAIL]\033[0m $*"; FAIL=$((FAIL + 1)); TOTAL=$((TOTAL + 1)); }
skip() { echo -e "\033[1;33m[SKIP]\033[0m $*"; TOTAL=$((TOTAL + 1)); }

db_exec() {
  docker compose -f "$COMPOSE_FILE" exec -T postgres psql -U "$DB_USER" -d "$DB_NAME" -c "$1" 2>/dev/null
}

db_is_ready() {
  docker compose -f "$COMPOSE_FILE" exec -T postgres pg_isready -U "$DB_USER" -q 2>/dev/null
}

assert_healthy() {
  local retries="${1:-5}"
  local delay="${2:-2}"
  for i in $(seq 1 "$retries"); do
    if db_is_ready; then
      return 0
    fi
    sleep "$delay"
  done
  return 1
}

echo "========================================="
echo " CivitForge DR Validation Test"
echo " Compose: $COMPOSE_FILE"
echo " Database: $DB_NAME"
echo "========================================="
echo ""

mkdir -p "$BACKUP_DIR"

# Phase 1: Verify database is accessible
log "Phase 1: Verifying database accessibility..."
if assert_healthy 10 2; then
  ok "database accessible"
else
  fail "database not reachable"
  echo ""
  log "Cannot continue without database. Exiting."
  exit 1
fi
echo ""

# Phase 2: Create test data for corruption simulation
log "Phase 2: Creating test data markers..."
MARKER_COUNT=100
db_exec "CREATE TABLE IF NOT EXISTS $TEST_TABLE (id SERIAL PRIMARY KEY, marker TEXT NOT NULL, created_at TIMESTAMPTZ DEFAULT NOW());"
for i in $(seq 1 "$MARKER_COUNT"); do
  db_exec "INSERT INTO $TEST_TABLE (marker) VALUES ('dr-test-$i-$RANDOM');" 2>/dev/null || true
done

ACTUAL_COUNT=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE;" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")
if [[ "$ACTUAL_COUNT" -ge "$MARKER_COUNT" ]]; then
  ok "test data created ($ACTUAL_COUNT markers)"
else
  fail "test data incomplete (got $ACTUAL_COUNT, expected $MARKER_COUNT)"
fi
echo ""

# Phase 3: Create backup
log "Phase 3: Creating database backup..."
BACKUP_FILE="$BACKUP_DIR/civit-dr-test-$(date +%Y%m%d-%H%M%S).dump"
BACKUP_START=$(date +%s%N)

if docker compose -f "$COMPOSE_FILE" exec -T postgres \
    pg_dump -U "$DB_USER" -Fc -d "$DB_NAME" > "$BACKUP_FILE" 2>/dev/null; then
  BACKUP_END=$(date +%s%N)
  BACKUP_SIZE=$(stat -f%z "$BACKUP_FILE" 2>/dev/null || stat --format=%s "$BACKUP_FILE" 2>/dev/null || echo "0")
  BACKUP_DURATION_MS=$(( (BACKUP_END - BACKUP_START) / 1000000 ))
  ok "backup created ($(( BACKUP_SIZE / 1024 ))KB in ${BACKUP_DURATION_MS}ms)"
else
  fail "backup creation failed"
  BACKUP_FILE=""
fi
echo ""

# Phase 4: Simulate data corruption
log "Phase 4: Simulating data corruption..."
db_exec "UPDATE $TEST_TABLE SET marker = 'CORRUPTED-' || marker WHERE id % 5 = 0;"
db_exec "DELETE FROM $TEST_TABLE WHERE id % 7 = 0;"
db_exec "INSERT INTO $TEST_TABLE (marker) VALUES ('INJECTED-FAKE-DATA-1');"
db_exec "INSERT INTO $TEST_TABLE (marker) VALUES ('INJECTED-FAKE-DATA-2');"

CORRUPTED_COUNT=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE WHERE marker LIKE 'CORRUPTED-%';" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")
DELETED_COUNT=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE WHERE id % 7 = 0;" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")
INJECTED_COUNT=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE WHERE marker LIKE 'INJECTED-%';" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")

if [[ "$CORRUPTED_COUNT" -gt 0 ]]; then
  ok "corruption simulated ($CORRUPTED_COUNT modified, $INJECTED_COUNT injected)"
else
  fail "corruption simulation failed"
fi
echo ""

# Phase 5: Restore from backup (measure RTO)
log "Phase 5: Restoring from backup (measuring RTO)..."
if [[ -n "$BACKUP_FILE" && -f "$BACKUP_FILE" ]]; then
  RTO_START=$(date +%s%N)

  docker compose -f "$COMPOSE_FILE" exec -T postgres \
    pg_restore -U "$DB_USER" -d "$DB_NAME" --clean --if-exists < "$BACKUP_FILE" 2>/dev/null || true

  RTO_END=$(date +%s%N)
  RTO_MS=$(( (RTO_END - RTO_START) / 1000000 ))

  if assert_healthy 10 2; then
    ok "restore completed (RTO: ${RTO_MS}ms)"
  else
    fail "restore failed — database not healthy after restore"
  fi
else
  skip "restore: no backup file available"
  RTO_MS=-1
fi
echo ""

# Phase 6: Verify data integrity (measure RPO)
log "Phase 6: Verifying data integrity..."
RESTORED_COUNT=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE WHERE marker LIKE 'dr-test-%';" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")
FAKE_COUNT=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE WHERE marker LIKE 'INJECTED-%';" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")
CORRUPT_REMAINING=$(db_exec "SELECT COUNT(*) FROM $TEST_TABLE WHERE marker LIKE 'CORRUPTED-%';" | tr -d '[:space:]' | grep -o '[0-9]*' | head -1 || echo "0")

if [[ "$RESTORED_COUNT" -ge "$MARKER_COUNT" ]]; then
  ok "data integrity verified ($RESTORED_COUNT original markers restored)"
else
  fail "data loss detected (expected >= $MARKER_COUNT, got $RESTORED_COUNT)"
fi

if [[ "$FAKE_COUNT" -eq 0 ]]; then
  ok "injected data removed after restore"
else
  fail "injected data persists after restore ($FAKE_COUNT rows)"
fi

if [[ "$CORRUPT_REMAINING" -eq 0 ]]; then
  ok "corrupted data removed after restore"
else
  fail "corrupted data persists after restore ($CORRUPT_REMAINING rows)"
fi
echo ""

# Phase 7: Calculate RPO
log "Phase 7: Calculating Recovery Point Objective (RPO)..."
if [[ "$ACTUAL_COUNT" -gt 0 ]]; then
  LOST_ROWS=$(( ACTUAL_COUNT - RESTORED_COUNT ))
  if [[ "$LOST_ROWS" -le 0 ]]; then
    RPO_SECONDS=0
    ok "RPO: 0s (no data loss, $RESTORED_COUNT/$ACTUAL_COUNT rows recovered)"
  else
    RPO_SECONDS=0
    fail "RPO: data loss of $LOST_ROWS rows ($RESTORED_COUNT/$ACTUAL_COUNT recovered)"
  fi
else
  RPO_SECONDS=-1
  skip "RPO: baseline count unavailable"
fi
echo ""

# Phase 8: Cleanup
log "Phase 8: Cleaning up test data..."
db_exec "DROP TABLE IF EXISTS $TEST_TABLE;" 2>/dev/null || true
rm -f "$BACKUP_FILE"
ok "cleanup completed"
echo ""

# ── Summary ──────────────────────────────────────────────────────────────────

echo "========================================="
echo " DR Validation Results: $PASS/$TOTAL passed"
echo ""
if [[ $RTO_MS -ge 0 ]]; then
  echo " Recovery Time Objective (RTO): ${RTO_MS}ms"
fi
if [[ $RPO_SECONDS -ge 0 ]]; then
  echo " Recovery Point Objective (RPO): ${RPO_SECONDS}s (data loss rows: $(( ACTUAL_COUNT - RESTORED_COUNT )))"
fi
echo ""
if [[ $FAIL -gt 0 ]]; then
  echo " FAILED: $FAIL test(s)"
  exit 1
else
  echo " ALL TESTS PASSED"
fi
echo "========================================="
