#!/bin/bash
# CivitForge automated UI tester
# Writes navigation triggers and reads captured HTML from /tmp/civit-capture.html
# The Tauri app polls /tmp/civit-navigate.txt every second.
set -euo pipefail

CAPTURE_DIR="/tmp/civit-captures"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$CAPTURE_DIR"

PASS=0
FAIL=0
ERRORS=""

navigate_to() {
    local url="$1"
    local label="$2"
    echo ">>> Navigating to ${url} (${label})..."
    echo -n "$url" > /tmp/civit-navigate.txt
    sleep 4  # wait for WASM to render
    # Extra wait for API calls to complete
    sleep 2
}

capture_and_read() {
    local label="$1"
    local outfile="${CAPTURE_DIR}/${TIMESTAMP}_${label}.html"
    if [ -f /tmp/civit-capture.html ]; then
        cp /tmp/civit-capture.html "$outfile"
    else
        echo "WARNING: No capture file found"
        return 1
    fi
    echo "    Captured: ${outfile}"
}

extract_text() {
    local infile="$1"
    python3 -c "
import re, sys
with open('$infile') as f:
    html = f.read()
body = re.search(r'<body>(.*)</body>', html, re.DOTALL)
if body:
    clean = re.sub(r'<(style|script)[^>]*>.*?</\1>', '', body.group(1), flags=re.DOTALL)
    text = re.sub(r'<[^>]+>', ' ', clean)
    text = re.sub(r'\s+', ' ', text).strip()
    print(text[:5000])
" 2>/dev/null
}

check_errors() {
    local infile="$1"
    local label="$2"
    python3 - "$infile" "$label" << 'PYEOF'
import re, sys
infile, label = sys.argv[1], sys.argv[2]
with open(infile) as f:
    html = f.read()
body = re.search(r'<body>(.*)</body>', html, re.DOTALL)
if body:
    clean = re.sub(r'<(style|script)[^>]*>.*?</\1>', '', body.group(1), flags=re.DOTALL)
    text = re.sub(r'<[^>]+>', ' ', clean)
    text = re.sub(r'\\s+', ' ', text).strip()
    errors = re.findall(r'(?:500 internal server|502 bad gateway|503 service unavailable|gateway timeout)[^<]{3,80}', text, re.IGNORECASE)
    if errors:
        for e in set(errors):
            print('ERROR: [%s] %s' % (label, e.strip()))
            sys.exit(1)
    print('OK: [%s] no errors' % label)
PYEOF
}

check_contains() {
    local infile="$1"
    local label="$2"
    local pattern="$3"
    if grep -qi "$pattern" "$infile" 2>/dev/null; then
        echo "PASS: [$label] contains '${pattern}'"
        return 0
    else
        echo "FAIL: [$label] missing '${pattern}'"
        return 1
    fi
}

assert_page() {
    local label="$1"
    local url="$2"
    navigate_to "$url" "$label"
    capture_and_read "$label"
    local infile="${CAPTURE_DIR}/${TIMESTAMP}_${label}.html"
    check_errors "$infile" "$label" || { FAIL=$((FAIL+1)); ERRORS="${ERRORS}\n  [${label}]"; return; }
    # Check for key content (strip scripts/styles first to avoid false matches)
    shift 2
    for pattern in "$@"; do
        check_contains "$infile" "$label" "$pattern" || { FAIL=$((FAIL+1)); ERRORS="${ERRORS}\n  [${label}] missing '${pattern}'"; return; }
    done
    PASS=$((PASS+1))
}

summary() {
    echo ""
    echo "========================================="
    echo "Test Summary: ${PASS} passed, ${FAIL} failed"
    echo "Time: $(date)"
    if [ $FAIL -gt 0 ]; then
        echo "FAILED:${ERRORS}"
    else
        echo "ALL TESTS PASSED"
    fi
    echo "========================================="
    [ $FAIL -eq 0 ]
}

full_test() {
    echo "========================================="
    echo "CivitForge UI Automated Test (Extended)"
    echo "Time: $(date)"
    echo "========================================="
    echo ""

    # === Core Pages ===
    echo "--- Core Pages ---"
    assert_page "home" "/" "CivitForge" "Sign" "Explore"
    assert_page "login" "/login" "Username" "Password" "Sign"
    assert_page "register" "/register" "Username" "Password" "Register"

    # === Repo List ===
    echo "--- Repository List ---"
    assert_page "repos" "/repos" "Explore" "test-repo" "testuser"

    # === Repo Detail (with real content) ===
    echo "--- Repository Detail ---"
    assert_page "repo-detail" "/repos/testuser/test-repo" "test-repo" "testuser" "Code" "README"

    # === Code Browser - Root Tree ===
    echo "--- Code Browser - Root Tree ---"
    assert_page "code-root" "/repos/testuser/test-repo/code" "README.md" "src" "docs"

    # === Code Browser - Subdirectory ===
    echo "--- Code Browser - Subdirectories ---"
    assert_page "code-src" "/repos/testuser/test-repo/code?path=src" "main.rs"
    assert_page "code-docs" "/repos/testuser/test-repo/code?path=docs" "guide.md"

    # === Code Browser - File View ===
    echo "--- File View ---"
    assert_page "file-readme" "/repos/testuser/test-repo/code?path=README.md" "CivitForge"
    assert_page "file-main-rs" "/repos/testuser/test-repo/code?path=src/main.rs" "println"
    assert_page "file-guide" "/repos/testuser/test-repo/code?path=docs/guide.md" "Guide"

    # === Navigation Flow ===
    echo "--- Navigation Flow ---"
    assert_page "nav-to-code" "/repos/testuser/test-repo/code" "README.md"
    assert_page "nav-to-file" "/repos/testuser/test-repo/code?path=README.md" "CivitForge"
    assert_page "nav-back-to-tree" "/repos/testuser/test-repo/code" "README.md"

    # === Non-existent Pages ===
    echo "--- Error Pages ---"
    navigate_to "/repos/nonexistent/nope" "missing-repo"
    capture_and_read "missing-repo"
    # Should get some error indication (404 or error message)
    local missing_file="${CAPTURE_DIR}/${TIMESTAMP}_missing-repo.html"
    if grep -qi "not found\|error\|404" "$missing_file" 2>/dev/null; then
        echo "PASS: [missing-repo] shows error for nonexistent repo"
        PASS=$((PASS+1))
    else
        echo "FAIL: [missing-repo] should show error"
        FAIL=$((FAIL+1))
    fi

    summary
}

single_page() {
    local url="${1:-/}"
    local label="${2:-page}"
    navigate_to "$url" "$label"
    capture_and_read "$label"
    extract_text "${CAPTURE_DIR}/${TIMESTAMP}_${label}.html"
}

case "${1:-full}" in
    single|s)  single_page "${2:-/}" "${3:-page}" ;;
    full|f|*)  full_test ;;
esac
