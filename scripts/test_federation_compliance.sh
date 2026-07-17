#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
REPORT_DIR="$REPO_ROOT/.reports/federation"
CRATE="civit-federation"

mkdir -p "$REPORT_DIR"

echo "=== CivitForge Federation Compliance Test Suite ==="
echo ""

# 1. Cargo check
echo "[1/4] Cargo check..."
if cargo check -p "$CRATE" --locked 2>&1 | tail -5; then
    echo "  PASS: crate compiles"
else
    echo "  FAIL: crate compilation failed"
    exit 1
fi

# 2. Run unit tests
echo ""
echo "[2/4] Running unit tests..."
if cargo test -p "$CRATE" --lib 2>&1 | tee "$REPORT_DIR/unit_tests.log" | tail -5; then
    echo "  PASS: unit tests passed"
else
    echo "  FAIL: unit tests failed"
    exit 1
fi

# 3. Run integration tests
echo ""
echo "[3/4] Running protocol compliance integration tests..."
if cargo test -p "$CRATE" --test protocol_compliance 2>&1 | tee "$REPORT_DIR/integration_tests.log" | tail -10; then
    echo "  PASS: integration tests passed"
else
    echo "  FAIL: integration tests failed"
    exit 1
fi

# 4. Generate compliance report
echo ""
echo "[4/4] Generating compliance report..."

TEST_COUNT=$(grep -c '#\[test\]' "$REPO_ROOT/crates/$CRATE/tests/protocol_compliance.rs" 2>/dev/null || echo "0")
UNIT_COUNT=$(grep -c '#\[test\]' "$REPO_ROOT/crates/$CRATE/src/"*.rs 2>/dev/null || echo "0")

cat > "$REPORT_DIR/compliance_report.md" <<EOF
# Federation Compliance Report
Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")

## Summary
- Integration test cases: $TEST_COUNT
- Unit test cases (crate): $UNIT_COUNT
- Crate: $CRATE
- Toolchain: $(rustc --version 2>/dev/null || echo "unknown")

## Test Results
- Cargo check: PASS
- Unit tests: PASS
- Integration tests: PASS

## Protocol Coverage
- ActivityPub: Create, Update, Delete, Follow, Undo, Accept, Reject, Add, Like, Announce
- HTTP Signatures: Ed25519, RSA-SHA256, ECDSA-P256, HMAC-SHA256
- WebFinger: RFC 7033 discovery
- ForgeFed: Repository, Fork, Star, Issue, PR, Review, Comment

## Logs
- Unit tests: $REPORT_DIR/unit_tests.log
- Integration tests: $REPORT_DIR/integration_tests.log
EOF

echo "  Report written to: $REPORT_DIR/compliance_report.md"
echo ""
echo "=== All compliance tests passed ==="
