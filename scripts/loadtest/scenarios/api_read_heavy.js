import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

const config = {
  baseUrl: __ENV.BASE_URL || 'http://localhost:8080',
  authToken: __ENV.AUTH_TOKEN || '',
};

const errorRate = new Rate('read_errors');
const apiDuration = new Trend('read_api_duration', true);
const totalRequests = new Counter('read_total_requests');
const successfulRequests = new Counter('read_successful_requests');

export const options = {
  scenarios: {
    read_heavy: {
      executor: 'constant-vus',
      vus: 200,
      duration: '10m',
      exec: 'readHeavyScenario',
      tags: { scenario: 'read_heavy' },
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<200'],
    read_errors: ['rate<0.02'],
  },
};

function getHeaders() {
  const headers = { 'Content-Type': 'application/json' };
  if (config.authToken) {
    headers['Authorization'] = `Bearer ${config.authToken}`;
  }
  return headers;
}

function weightedRandom(items) {
  const total = items.reduce((sum, item) => sum + item.weight, 0);
  let r = Math.random() * total;
  for (const item of items) {
    if (r < item.weight) return item;
    r -= item.weight;
  }
  return items[items.length - 1];
}

export function readHeavyScenario() {
  const headers = getHeaders();
  const readOps = [
    { name: 'list_repos', method: 'GET', path: '/api/v1/repos', weight: 35 },
    { name: 'search_code', method: 'GET', path: '/api/v1/search?q=test+query&limit=20', weight: 25 },
    { name: 'get_issues', method: 'GET', path: '/api/v1/repos/test-owner/test-repo/issues', weight: 20 },
    { name: 'get_repo', method: 'GET', path: '/api/v1/repos/test-owner/test-repo', weight: 10 },
    { name: 'get_commits', method: 'GET', path: '/api/v1/repos/test-owner/test-repo/commits', weight: 5 },
    { name: 'list_users', method: 'GET', path: '/api/v1/users', weight: 5 },
  ];

  const op = weightedRandom(readOps);
  const res = http.get(`${config.baseUrl}${op.path}`, { headers, tags: { operation: op.name } });
  totalRequests.add(1);

  const passed = check(res, {
    [`${op.name}_status_ok`]: (r) => r.status >= 200 && r.status < 400,
    [`${op.name}_latency_ok`]: (r) => r.timings.duration < 200,
  });

  if (passed) {
    successfulRequests.add(1);
  } else {
    errorRate.add(1);
  }
  apiDuration.add(res.timings.duration);

  sleep(Math.random() * 2 + 0.5);
}

export function handleSummary(data) {
  const p95 = data.metrics.http_req_duration?.values?.['p(95)'] ?? 0;
  const failRate = data.metrics.http_req_failed?.values?.rate ?? 0;

  return {
    stdout: JSON.stringify({
      scenario: 'api_read_heavy',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 200 && failRate < 0.02,
    }, null, 2),
    'scripts/loadtest/results/api_read_heavy.json': JSON.stringify({
      scenario: 'api_read_heavy',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 200 && failRate < 0.02,
    }, null, 2),
  };
}
