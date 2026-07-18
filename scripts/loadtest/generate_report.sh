#!/usr/bin/env bash
# =============================================================================
# CivitForge Load Test Report Generator
# =============================================================================
# Runs all load test scenarios, collects results, and generates an HTML report.
#
# Usage:
#   scripts/loadtest/generate_report.sh [--base-url URL] [--token TOKEN]
#
# Prerequisites: k6, jq
# =============================================================================

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
RESULTS_DIR="$REPO_ROOT/scripts/loadtest/results"
REPORT_DIR="$REPO_ROOT/.reports"
REPORT_FILE="$REPORT_DIR/loadtest-report-$(date +%Y%m%d-%H%M%S).html"
SCENARIOS_DIR="$REPO_ROOT/scripts/loadtest/scenarios"

BASE_URL="${BASE_URL:-http://localhost:8080}"
AUTH_TOKEN="${AUTH_TOKEN:-}"

mkdir -p "$RESULTS_DIR" "$REPORT_DIR"

log() { echo -e "\033[1;34m[REPORT]\033[0m $*"; }
ok()  { echo -e "\033[1;32m[PASS]\033[0m $*"; }
fail() { echo -e "\033[1;31m[FAIL]\033[0m $*"; }

# Parse args
while [[ $# -gt 0 ]]; do
  case $1 in
    --base-url) BASE_URL="$2"; shift 2 ;;
    --token) AUTH_TOKEN="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 [--base-url URL] [--token TOKEN]"
      exit 0
      ;;
    *) shift ;;
  esac
done

# Clean previous results
rm -f "$RESULTS_DIR"/*.json

log "Running load test scenarios against $BASE_URL"
echo "========================================="

SCENARIOS=("api_read_heavy" "api_write_heavy" "realistic_mixed" "spike_test")

for scenario in "${SCENARIOS[@]}"; do
  log "Running $scenario..."
  if k6 run \
    -e "BASE_URL=$BASE_URL" \
    -e "AUTH_TOKEN=$AUTH_TOKEN" \
    --out json="$RESULTS_DIR/${scenario}_raw.json" \
    "$SCENARIOS_DIR/${scenario}.js" 2>&1 | tail -5; then
    ok "$scenario completed"
  else
    fail "$scenario failed (exit $?)"
  fi
  echo ""
done

log "Collecting results..."

# Build JSON summary
SUMMARY_FILE="$RESULTS_DIR/summary.json"
echo '{' > "$SUMMARY_FILE"
echo '  "generated_at": "'$(date -u +%Y-%m-%dT%H:%M:%SZ)'",' >> "$SUMMARY_FILE"
echo '  "base_url": "'"$BASE_URL"'",' >> "$SUMMARY_FILE"
echo '  "scenarios": {' >> "$SUMMARY_FILE"

FIRST=true
for scenario in "${SCENARIOS[@]}"; do
  FILE="$RESULTS_DIR/${scenario}.json"
  if [[ -f "$FILE" ]]; then
    if [[ "$FIRST" == "true" ]]; then
      FIRST=false
    else
      echo ',' >> "$SUMMARY_FILE"
    fi
    DATA=$(cat "$FILE" | jq -c '.' 2>/dev/null || echo '{}')
    echo "    \"$scenario\": $DATA" >> "$SUMMARY_FILE"
  fi
done

echo '' >> "$SUMMARY_FILE"
echo '  }' >> "$SUMMARY_FILE"
echo '}' >> "$SUMMARY_FILE"

log "Generating HTML report..."

# Generate HTML report
cat > "$REPORT_FILE" << 'HTMLHEAD'
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>CivitForge Load Test Report</title>
<style>
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #0f172a; color: #e2e8f0; padding: 2rem; }
  h1 { font-size: 1.5rem; margin-bottom: 0.5rem; color: #38bdf8; }
  h2 { font-size: 1.1rem; margin: 1.5rem 0 0.75rem; color: #94a3b8; }
  .meta { color: #64748b; font-size: 0.85rem; margin-bottom: 1.5rem; }
  .card { background: #1e293b; border-radius: 8px; padding: 1.25rem; margin-bottom: 1rem; border: 1px solid #334155; }
  .card-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 0.75rem; }
  .card-title { font-weight: 600; font-size: 1rem; }
  .badge { padding: 0.2rem 0.6rem; border-radius: 999px; font-size: 0.75rem; font-weight: 600; }
  .badge-pass { background: #065f46; color: #34d399; }
  .badge-fail { background: #7f1d1d; color: #f87171; }
  .metrics { display: grid; grid-template-columns: repeat(auto-fit, minmax(140px, 1fr)); gap: 0.75rem; }
  .metric { background: #0f172a; border-radius: 6px; padding: 0.75rem; }
  .metric-label { font-size: 0.75rem; color: #64748b; text-transform: uppercase; letter-spacing: 0.05em; }
  .metric-value { font-size: 1.25rem; font-weight: 700; margin-top: 0.25rem; }
  .metric-value.good { color: #34d399; }
  .metric-value.warn { color: #fbbf24; }
  .metric-value.bad { color: #f87171; }
  table { width: 100%; border-collapse: collapse; margin-top: 0.75rem; }
  th, td { padding: 0.5rem 0.75rem; text-align: left; border-bottom: 1px solid #334155; font-size: 0.85rem; }
  th { color: #94a3b8; font-weight: 600; }
  .summary-grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 1rem; margin-bottom: 1.5rem; }
  .summary-card { background: #1e293b; border-radius: 8px; padding: 1rem; border: 1px solid #334155; }
  .summary-card .label { font-size: 0.75rem; color: #64748b; text-transform: uppercase; }
  .summary-card .value { font-size: 1.5rem; font-weight: 700; margin-top: 0.25rem; }
</style>
</head>
<body>
HTMLHEAD

# Add dynamic content
cat >> "$REPORT_FILE" << HTMLBODY
<h1>CivitForge Load Test Report</h1>
<p class="meta">Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ) | Target: $BASE_URL</p>

<div class="summary-grid">
HTMLBODY

# Read results and generate summary cards
for scenario in "${SCENARIOS[@]}"; do
  FILE="$RESULTS_DIR/${scenario}.json"
  if [[ -f "$FILE" ]]; then
    P95=$(jq -r '.p95_latency_ms // "N/A"' "$FILE" 2>/dev/null || echo "N/A")
    ERR=$(jq -r '.error_rate // "N/A"' "$FILE" 2>/dev/null || echo "N/A")
    THRESH=$(jq -r '.thresholds_met // false' "$FILE" 2>/dev/null || echo "false")
    BADGE_CLASS="badge-pass"
    if [[ "$THRESH" != "true" ]]; then BADGE_CLASS="badge-fail"; fi

    cat >> "$REPORT_FILE" << HTMLCARD
<div class="summary-card">
  <div class="label">$scenario</div>
  <div class="value"><span class="badge $BADGE_CLASS">$([ "$THRESH" = "true" ] && echo "PASS" || echo "FAIL")</span></div>
  <div class="metric" style="margin-top:0.5rem">
    <div class="metric-label">p95 Latency</div>
    <div class="metric-value $(echo "$P95" | awk '{if ($1 < 200) print "good"; else if ($1 < 500) print "warn"; else print "bad"}')">${P95}ms</div>
  </div>
  <div class="metric" style="margin-top:0.5rem">
    <div class="metric-label">Error Rate</div>
    <div class="metric-value $(echo "$ERR" | awk '{if ($1 < 0.01) print "good"; else if ($1 < 0.05) print "warn"; else print "bad"}')">${ERR}</div>
  </div>
</div>
HTMLCARD
  fi
done

cat >> "$REPORT_FILE" << 'HTMLGRID'
</div>

<h2>Detailed Results</h2>
<table>
<tr><th>Scenario</th><th>p95 Latency (ms)</th><th>Error Rate</th><th>Total Requests</th><th>RPS</th><th>Status</th></tr>
HTMLGRID

for scenario in "${SCENARIOS[@]}"; do
  FILE="$RESULTS_DIR/${scenario}.json"
  if [[ -f "$FILE" ]]; then
    P95=$(jq -r '.p95_latency_ms // "N/A"' "$FILE" 2>/dev/null || echo "N/A")
    ERR=$(jq -r '.error_rate // "N/A"' "$FILE" 2>/dev/null || echo "N/A")
    REQ=$(jq -r '.total_requests // "N/A"' "$FILE" 2>/dev/null || echo "N/A")
    RPS=$(jq -r '.rps // "N/A"' "$FILE" 2>/dev/null || echo "N/A")
    THRESH=$(jq -r '.thresholds_met // false' "$FILE" 2>/dev/null || echo "false")
    STATUS="<span class=\"badge badge-pass\">PASS</span>"
    if [[ "$THRESH" != "true" ]]; then STATUS="<span class=\"badge badge-fail\">FAIL</span>"; fi

    echo "<tr><td>$scenario</td><td>$P95</td><td>$ERR</td><td>$REQ</td><td>$RPS</td><td>$STATUS</td></tr>" >> "$REPORT_FILE"
  fi
done

cat >> "$REPORT_FILE" << HTMLFOOT
</table>

<h2>Thresholds</h2>
<div class="card">
<table>
<tr><th>Scenario</th><th>p95 Target</th><th>Error Rate Target</th></tr>
<tr><td>api_read_heavy</td><td>&lt; 200ms</td><td>&lt; 2%</td></tr>
<tr><td>api_write_heavy</td><td>&lt; 500ms</td><td>&lt; 2%</td></tr>
<tr><td>realistic_mixed</td><td>&lt; 300ms</td><td>&lt; 1%</td></tr>
<tr><td>spike_test</td><td>&lt; 1000ms</td><td>&lt; 5%</td></tr>
</table>
</div>

</body>
</html>
HTMLFOOT

echo ""
log "========================================="
log "Report saved to: $REPORT_FILE"
log "Results saved to: $RESULTS_DIR/"
log "========================================="

# Determine overall pass/fail
OVERALL_PASS=true
for scenario in "${SCENARIOS[@]}"; do
  FILE="$RESULTS_DIR/${scenario}.json"
  if [[ -f "$FILE" ]]; then
    THRESH=$(jq -r '.thresholds_met // false' "$FILE" 2>/dev/null || echo "false")
    if [[ "$THRESH" != "true" ]]; then
      OVERALL_PASS=false
    fi
  fi
done

if [[ "$OVERALL_PASS" == "true" ]]; then
  ok "All scenarios passed thresholds"
  exit 0
else
  fail "One or more scenarios failed thresholds"
  exit 1
fi
