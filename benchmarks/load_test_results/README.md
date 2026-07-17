# Load Test Results

Baseline load test results for CivitForge performance tracking.

## Files

- `baseline_results.toml` — Baseline metrics from k6 load tests

## Running Load Tests

### Prerequisites

```bash
# Install k6
# macOS: brew install k6
# Linux: sudo snap install k6
# Or: docker pull grafana/k6
```

### Execute Tests

```bash
# Full load test suite (25 minutes)
k6 run scripts/loadtest/k6_api_loadtest.js

# Quick smoke test (modify scripts/loadtest/k6_api_loadtest.js options)
# Reduce VUs and duration for fast iteration
```

### Against a specific target

```bash
CIVIT_BASE_URL=http://your-server:8080 k6 run scripts/loadtest/k6_api_loadtest.js
```

## Interpreting Results

### Key Metrics

| Metric | Baseline | Warning | Critical |
|--------|----------|---------|----------|
| p95 response time | 120ms | >500ms | >1000ms |
| Error rate | 0.5% | >1% | >5% |
| p99 response time | 220ms | >800ms | >2000ms |

### Thresholds (enforced in k6 scripts)

- **http_req_duration p(95)**: Must be < 500ms
- **http_req_failed rate**: Must be < 1%
- **errors rate**: Must be < 1%

### Scenario Descriptions

| Scenario | VUs | Duration | Purpose |
|----------|-----|----------|---------|
| `api_read` | 100 | 5m | Read-heavy workload (repos, issues, search) |
| `api_write` | 50 | 5m | Write-heavy workload (create issues, push) |
| `mixed_usage` | 100 | 10m | Realistic weighted mix |
| `spike_test` | 0→1000 | 1.5m | Sudden traffic surge resilience |

### Reading the TOML

Each scenario section contains:

- **Config**: VUs and duration used
- **Results**: Aggregated metrics from the test run

The `summary` section provides overall totals.

## Updating Baselines

After infrastructure changes or performance improvements:

1. Run the full load test suite
2. Review results for regressions
3. Update `baseline_results.toml` with new values
4. Commit with description of what changed

```bash
# Compare against previous baseline
diff <(tomlq -r '.scenarios' baseline_results.toml) \
     <(tomlq -r '.scenarios' new_results.toml)
```

## Environment Requirements

For reproducible results, run against:

- **CPU**: 4+ cores
- **RAM**: 8GB+
- **Disk**: SSD
- **Network**: Low latency to target (<10ms)
- **Services**: PostgreSQL 17, Redis 7, CivitForge server
