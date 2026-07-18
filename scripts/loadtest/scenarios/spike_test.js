import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

const config = {
  baseUrl: __ENV.BASE_URL || 'http://localhost:8080',
  authToken: __ENV.AUTH_TOKEN || '',
};

const errorRate = new Rate('spike_errors');
const apiDuration = new Trend('spike_api_duration', true);
const totalRequests = new Counter('spike_total_requests');
const currentVUs = new Gauge('spike_current_vus');
const recoveryTime = new Trend('spike_recovery_time', true);

export const options = {
  scenarios: {
    spike: {
      executor: 'ramping-vus',
      startVUs: 50,
      stages: [
        { duration: '30s', target: 5000 },
        { duration: '2m', target: 5000 },
        { duration: '10s', target: 50 },
      ],
      exec: 'spikeScenario',
      tags: { scenario: 'spike' },
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<1000'],
    spike_errors: ['rate<0.05'],
  },
};

function getHeaders() {
  const headers = { 'Content-Type': 'application/json' };
  if (config.authToken) {
    headers['Authorization'] = `Bearer ${config.authToken}`;
  }
  return headers;
}

export function spikeScenario() {
  const headers = getHeaders();
  currentVUs.add(1);

  const spikeOps = [
    { name: 'list_repos', path: '/api/v1/repos', weight: 40 },
    { name: 'search_code', path: '/api/v1/search?q=test', weight: 25 },
    { name: 'get_issues', path: '/api/v1/repos/test-owner/test-repo/issues', weight: 20 },
    { name: 'get_repo', path: '/api/v1/repos/test-owner/test-repo', weight: 10 },
    { name: 'health_check', path: '/healthz', weight: 5 },
  ];

  const total = spikeOps.reduce((sum, op) => sum + op.weight, 0);
  let r = Math.random() * total;
  let selected = spikeOps[0];
  for (const op of spikeOps) {
    if (r < op.weight) { selected = op; break; }
    r -= op.weight;
  }

  const res = http.get(`${config.baseUrl}${selected.path}`, {
    headers,
    tags: { operation: selected.name },
    timeout: '10s',
  });

  totalRequests.add(1);

  const passed = check(res, {
    [`${selected.name}_status_ok`]: (r) => r.status >= 200 && r.status < 500,
    [`${selected.name}_responded`]: (r) => r.timings.duration > 0,
  });

  if (!passed) {
    errorRate.add(1);
  }
  apiDuration.add(res.timings.duration);

  sleep(0.1);
}

export function handleSummary(data) {
  const p95 = data.metrics.http_req_duration?.values?.['p(95)'] ?? 0;
  const failRate = data.metrics.http_req_failed?.values?.rate ?? 0;
  const maxVUs = data.metrics.vus_max?.values?.value ?? 0;

  return {
    stdout: JSON.stringify({
      scenario: 'spike_test',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      max_vus: maxVUs,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 1000 && failRate < 0.05,
    }, null, 2),
    'scripts/loadtest/results/spike_test.json': JSON.stringify({
      scenario: 'spike_test',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      max_vus: maxVUs,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 1000 && failRate < 0.05,
    }, null, 2),
  };
}
