#!/bin/bash
# CivitForge automated UI tester
# Writes navigation triggers and reads captured HTML from /tmp/civit-capture.html
# The Tauri app polls /tmp/civit-navigate.txt every second.
set -euo pipefail

CAPTURE_DIR="/tmp/civit-captures"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
mkdir -p "$CAPTURE_DIR"

navigate_to() {
    local url="$1"
    local label="$2"
    echo ">>> Navigating to ${url} (${label})..."
    echo -n "$url" > /tmp/civit-navigate.txt
    sleep 4  # wait for WASM to render
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
    # Print text content
    print(text[:5000])
    # Print errors found
    errors = re.findall(r'(?:error|fail|401|403|404|500)[^<]{3,80}', text, re.IGNORECASE)
    if errors:
        print()
        print('=== ERRORS ===')
        for e in set(errors):
            print(f'  - {e.strip()}')
" 2>/dev/null
}

full_test() {
    echo "========================================="
    echo "CivitForge UI Automated Test"
    echo "Time: $(date)"
    echo "========================================="
    echo ""

    # Test pages
    navigate_to "/" "home"
    capture_and_read "home"
    extract_text "${CAPTURE_DIR}/${TIMESTAMP}_home.html"
    echo ""

    navigate_to "/login" "login"
    capture_and_read "login"
    extract_text "${CAPTURE_DIR}/${TIMESTAMP}_login.html"
    echo ""

    navigate_to "/repos" "repos"
    capture_and_read "repos"
    extract_text "${CAPTURE_DIR}/${TIMESTAMP}_repos.html"
    echo ""

    navigate_to "/repos/testuser/test-repo" "repo-detail"
    capture_and_read "repo-detail"
    extract_text "${CAPTURE_DIR}/${TIMESTAMP}_repo-detail.html"
    echo ""

    navigate_to "/repos/testuser/test-repo/code" "code"
    capture_and_read "code"
    extract_text "${CAPTURE_DIR}/${TIMESTAMP}_code.html"
    echo ""

    echo "========================================="
    echo "Done. Captures in ${CAPTURE_DIR}/"
    echo "========================================="
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
    full|f|*) full_test ;;
esac
