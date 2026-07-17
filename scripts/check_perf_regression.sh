#!/usr/bin/env bash
# check_perf_regression.sh — Run benchmarks and detect performance regressions
# against the baselines defined in benches/performance_baseline.toml.
#
# Usage:
#   scripts/check_perf_regression.sh              # run + compare
#   scripts/check_perf_regression.sh --update     # run + update baselines
#   scripts/check_perf_regression.sh --dry-run    # show what would be checked

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BASELINE_FILE="$REPO_ROOT/benches/performance_baseline.toml"
RESULTS_DIR="$REPO_ROOT/target/criterion"
UPDATE_MODE=false
DRY_RUN=false

for arg in "$@"; do
    case "$arg" in
        --update) UPDATE_MODE=true ;;
        --dry-run) DRY_RUN=true ;;
        --help|-h)
            echo "Usage: $0 [--update] [--dry-run]"
            echo "  --update    Update baselines from current results"
            echo "  --dry-run   Show what would be checked without running"
            exit 0
            ;;
    esac
done

if [[ ! -f "$BASELINE_FILE" ]]; then
    echo "ERROR: Baseline file not found: $BASELINE_FILE"
    exit 1
fi

echo "=== CivitForge Performance Regression Check ==="
echo "Baseline: $BASELINE_FILE"
echo

# ── Parse baseline thresholds from TOML ──────────────────────────────────────

declare -A BASELINE_US
declare -A THRESHOLD_PCT

current_section=""
while IFS= read -r line; do
    # Match [benchmarks.name]
    if [[ "$line" =~ ^\[benchmarks\.([a-zA-Z0-9_]+)\]$ ]]; then
        current_section="${BASH_REMATCH[1]}"
    fi
    # Match key = value
    if [[ -n "$current_section" ]]; then
        if [[ "$line" =~ ^[[:space:]]*baseline_us[[:space:]]*=[[:space:]]*([0-9]+) ]]; then
            BASELINE_US[$current_section]="${BASH_REMATCH[1]}"
        fi
        if [[ "$line" =~ ^[[:space:]]*threshold_percent[[:space:]]*=[[:space:]]*([0-9]+) ]]; then
            THRESHOLD_PCT[$current_section]="${BASH_REMATCH[1]}"
        fi
    fi
done < "$BASELINE_FILE"

if [[ ${#BASELINE_US[@]} -eq 0 ]]; then
    echo "ERROR: No baselines found in $BASELINE_FILE"
    exit 1
fi

echo "Loaded ${#BASELINE_US[@]} benchmark baselines."
echo

# ── Dry run: just show what will be checked ──────────────────────────────────

if $DRY_RUN; then
    echo "Benchmarks to check:"
    for name in "${!BASELINE_US[@]}"; do
        echo "  $name  baseline=${BASELINE_US[$name]}us  threshold=+${THRESHOLD_PCT[$name]}%"
    done
    exit 0
fi

# ── Run benchmarks ───────────────────────────────────────────────────────────

echo "Running benchmarks..."
cargo bench --workspace --locked -- --output-format bencher 2>/dev/null \
    | tee /tmp/civit-bench-results.txt

echo
echo "=== Comparing against baselines ==="
echo

# ── Parse benchmark results ──────────────────────────────────────────────────
# Bencher format: benchmark_name     time:   [lower center upper]
# We extract the center value in nanoseconds.

declare -A RESULT_NS

while IFS= read -r line; do
    # Match benchmark lines with time: [...] pattern
    if [[ "$line" =~ ^[[:space:]]*([a-zA-Z0-9_/]+)[[:space:]]+time:[[:space:]]+\[([0-9.]+)[[:space:]]+([0-9.]+)[[:space:]]+([0-9.]+)\] ]]; then
        bench_name="${BASH_REMATCH[1]}"
        center="${BASH_REMATCH[3]}"
        # Convert to microseconds based on unit suffix
        if [[ "$line" =~ ns ]]; then
            us=$(echo "$center / 1000" | bc -l 2>/dev/null || echo "$center")
        elif [[ "$line" =~ µs ]] || [[ "$line" =~ us ]]; then
            us="$center"
        elif [[ "$line" =~ ms ]]; then
            us=$(echo "$center * 1000" | bc -l 2>/dev/null || echo "$center")
        elif [[ "$line" =~ s[^e] ]]; then
            us=$(echo "$center * 1000000" | bc -l 2>/dev/null || echo "$center")
        else
            # Default: assume nanoseconds
            us=$(echo "$center / 1000" | bc -l 2>/dev/null || echo "$center")
        fi
        RESULT_NS[$bench_name]="$us"
    fi
done < /tmp/civit-bench-results.txt

if [[ ${#RESULT_NS[@]} -eq 0 ]]; then
    echo "WARNING: No benchmark results parsed. Check benchmark output format."
    echo "Raw output saved to /tmp/civit-bench-results.txt"
    exit 1
fi

echo "Parsed ${#RESULT_NS[@]} benchmark results."
echo

# ── Compare against thresholds ───────────────────────────────────────────────

REGRESSIONS=0
IMPROVEMENTS=0
NEW_BENCHMARKS=0
PASSED=0

printf "%-45s %10s %10s %10s %10s  %s\n" "Benchmark" "Baseline" "Actual" "Delta" "Threshold" "Status"
echo "--------------------------------------------------------------------------------------------------------------"

for name in $(echo "${!BASELINE_US[@]}" | tr ' ' '\n' | sort); do
    baseline="${BASELINE_US[$name]}"
    threshold="${THRESHOLD_PCT[$name]}"

    if [[ -z "${RESULT_NS[$name]:-}" ]]; then
        printf "%-45s %10s %10s %10s %10s  %s\n" "$name" "${baseline}us" "N/A" "-" "-" "MISSING"
        continue
    fi

    actual="${RESULT_NS[$name]}"
    # Calculate percentage change: ((actual - baseline) / baseline) * 100
    delta_pct=$(echo "scale=1; (($actual - $baseline) / $baseline) * 100" | bc -l 2>/dev/null || echo "0")
    exceeded=$(echo "$delta_pct > $threshold" | bc -l 2>/dev/null || echo "0")

    if [[ "$exceeded" == "1" ]]; then
        status="REGRESSION"
        REGRESSIONS=$((REGRESSIONS + 1))
    elif (( $(echo "$delta_pct < -$threshold" | bc -l 2>/dev/null || echo "0") )); then
        status="IMPROVED"
        IMPROVEMENTS=$((IMPROVEMENTS + 1))
    else
        status="OK"
        PASSED=$((PASSED + 1))
    fi

    printf "%-45s %9sus %9sus %9s%% %9s%%  %s\n" \
        "$name" "$baseline" "$actual" "$delta_pct" "$threshold" "$status"
done

echo
echo "=== Summary ==="
echo "  Passed:       $PASSED"
echo "  Regressions:  $REGRESSIONS"
echo "  Improvements: $IMPROVEMENTS"

# ── Update baselines mode ────────────────────────────────────────────────────

if $UPDATE_MODE; then
    echo
    echo "Updating baselines from current results..."
    # Rewrite the TOML with actual values
    {
        echo "[metadata]"
        echo "version = \"1.0.0\""
        echo "created_at = \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""
        echo "toolchain = \"$(rustc --version 2>/dev/null | awk '{print $2}' || echo 'unknown')\""
        echo "description = \"Performance baselines for CivitForge core benchmarks\""
        echo
        for name in $(echo "${!BASELINE_US[@]}" | tr ' ' '\n' | sort); do
            actual="${RESULT_NS[$name]:-${BASELINE_US[$name]}}"
            # Round to integer microseconds
            actual_int=$(echo "$actual" | cut -d. -f1)
            threshold="${THRESHOLD_PCT[$name]}"
            echo "[benchmarks.$name]"
            echo "baseline_us = $actual_int"
            echo "threshold_percent = $threshold"
            echo
        done
    } > "$BASELINE_FILE"
    echo "Baselines updated in $BASELINE_FILE"
fi

# ── Exit code ────────────────────────────────────────────────────────────────

if [[ $REGRESSIONS -gt 0 ]]; then
    echo
    echo "FAIL: $REGRESSIONS regression(s) detected. Threshold exceeded."
    echo "To investigate, check the specific benchmarks above."
    echo "To update baselines (if the change is intentional): $0 --update"
    exit 1
fi

echo
echo "PASS: All benchmarks within threshold."
exit 0
