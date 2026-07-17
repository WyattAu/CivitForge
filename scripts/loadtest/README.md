# CivitForge Load Testing Scripts

This directory contains k6 load testing scripts for CivitForge API performance testing.

## Prerequisites

1. Install k6: https://k6.io/docs/get-started/installation/
2. Ensure CivitForge API is running and accessible

## Scripts

### k6_api_loadtest.js
Main load testing script with 4 scenarios:
1. **API Read**: 100 VUs for 5 minutes - Tests read operations (list repos, get issues, search)
2. **API Write**: 50 VUs for 5 minutes - Tests write operations (create issue, push code, run pipeline)
3. **Mixed Usage**: 100 VUs for 10 minutes - Realistic mixed read/write pattern
4. **Spike Test**: 1000 VUs for 2 minutes - Tests sudden traffic spikes

### k6_config.js
Shared configuration with:
- Base URL configuration
- Authentication token support
- Performance thresholds (p95 < 500ms, error rate < 1%)

## Usage

### Run all scenarios
```bash
k6 run scripts/loadtest/k6_api_loadtest.js
```

### Run with custom configuration
```bash
k6 run scripts/loadtest/k6_api_loadtest.js \
  -e BASE_URL=http://localhost:8080 \
  -e AUTH_TOKEN=your_token_here
```

### Run specific scenario
```bash
k6 run scripts/loadtest/k6_api_loadtest.js --scenario api_read
```

### Generate HTML report
```bash
k6 run scripts/loadtest/k6_api_loadtest.js \
  --out json=scripts/loadtest/results/k6_results.json
```

## Environment Variables

- `BASE_URL`: API base URL (default: http://localhost:8080)
- `AUTH_TOKEN`: Authentication token for protected endpoints
- `K6_ENVIRONMENT`: Environment tag for metrics (default: development)

## Results

Results are saved to:
- `scripts/loadtest/results/summary.json` - JSON summary
- `scripts/loadtest/results/k6_results.json` - Detailed k6 metrics (if using JSON output)

## Thresholds

The load tests enforce these performance thresholds:
- **HTTP Request Duration (p95)**: < 500ms
- **HTTP Request Failed Rate**: < 1%

If thresholds are not met, k6 will exit with a non-zero status code.