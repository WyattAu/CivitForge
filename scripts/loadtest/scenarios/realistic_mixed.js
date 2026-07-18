import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter, Gauge } from 'k6/metrics';

const config = {
  baseUrl: __ENV.BASE_URL || 'http://localhost:8080',
  authToken: __ENV.AUTH_TOKEN || '',
};

const errorRate = new Rate('mixed_errors');
const apiDuration = new Trend('mixed_api_duration', true);
const totalRequests = new Counter('mixed_total_requests');
const successfulRequests = new Counter('mixed_successful_requests');
const activeUsers = new Gauge('mixed_active_users');

export const options = {
  scenarios: {
    realistic_mixed: {
      executor: 'constant-vus',
      vus: 300,
      duration: '30m',
      exec: 'realisticMixedScenario',
      tags: { scenario: 'realistic_mixed' },
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<300'],
    mixed_errors: ['rate<0.01'],
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

function thinkTime(base) {
  const jitter = (Math.random() - 0.5) * base * 0.5;
  return Math.max(0.1, base + jitter);
}

const operations = [
  { name: 'list_repos', method: 'GET', path: '/api/v1/repos', weight: 20 },
  { name: 'search_code', method: 'GET', path: '/api/v1/search?q=function+impl', weight: 15 },
  { name: 'get_issues', method: 'GET', path: '/api/v1/repos/test-owner/test-repo/issues', weight: 15 },
  { name: 'get_repo', method: 'GET', path: '/api/v1/repos/test-owner/test-repo', weight: 10 },
  { name: 'get_commits', method: 'GET', path: '/api/v1/repos/test-owner/test-repo/commits', weight: 8 },
  { name: 'list_users', method: 'GET', path: '/api/v1/users', weight: 5 },
  { name: 'get_sla_dashboard', method: 'GET', path: '/api/v1/sla/dashboard', weight: 3 },
  { name: 'get_pipeline', method: 'GET', path: '/api/v1/repos/test-owner/test-repo/pipelines', weight: 4 },
  { name: 'create_issue', method: 'POST', path: '/api/v1/repos/test-owner/test-repo/issues', weight: 8 },
  { name: 'update_issue', method: 'PATCH', path: '/api/v1/repos/test-owner/test-repo/issues/1', weight: 5 },
  { name: 'create_comment', method: 'POST', path: '/api/v1/repos/test-owner/test-repo/issues/1/comments', weight: 4 },
  { name: 'create_release', method: 'POST', path: '/api/v1/repos/test-owner/test-repo/releases', weight: 2 },
  { name: 'run_pipeline', method: 'POST', path: '/api/v1/repos/test-owner/test-repo/pipelines/run', weight: 1 },
];

function simulateUserSession() {
  const sessionType = Math.random();
  let ops;

  if (sessionType < 0.5) {
    ops = [
      operations.filter(o => o.method === 'GET')[Math.floor(Math.random() * 8)],
      operations.filter(o => o.method === 'GET')[Math.floor(Math.random() * 8)],
      operations.filter(o => o.method === 'POST' && o.name === 'create_issue')[0],
    ];
  } else if (sessionType < 0.8) {
    ops = [
      operations.filter(o => o.method === 'GET')[Math.floor(Math.random() * 8)],
      operations.filter(o => o.method === 'PATCH')[0],
      operations.filter(o => o.method === 'POST' && o.name === 'create_comment')[0],
    ];
  } else {
    ops = [
      operations.filter(o => o.method === 'GET')[Math.floor(Math.random() * 8)],
      operations.filter(o => o.method === 'GET')[Math.floor(Math.random() * 8)],
      operations.filter(o => o.method === 'GET')[Math.floor(Math.random() * 8)],
    ];
  }

  return ops.filter(Boolean);
}

export function realisticMixedScenario() {
  const headers = getHeaders();
  activeUsers.add(1);

  const session = simulateUserSession();

  for (const op of session) {
    let res;
    const url = `${config.baseUrl}${op.path}`;

    if (op.method === 'GET') {
      res = http.get(url, { headers, tags: { operation: op.name } });
    } else if (op.method === 'POST') {
      const payload = JSON.stringify({
        title: `Test ${op.name} ${Date.now()}`,
        body: 'Realistic mixed test',
        labels: ['test'],
      });
      res = http.post(url, payload, { headers, tags: { operation: op.name } });
    } else {
      const payload = JSON.stringify({ body: `Update ${Date.now()}` });
      res = http.patch(url, payload, { headers, tags: { operation: op.name } });
    }

    totalRequests.add(1);

    const passed = check(res, {
      [`${op.name}_status_ok`]: (r) => r.status >= 200 && r.status < 400,
      [`${op.name}_latency_ok`]: (r) => r.timings.duration < 300,
    });

    if (passed) {
      successfulRequests.add(1);
    } else {
      errorRate.add(1);
    }
    apiDuration.add(res.timings.duration);

    sleep(thinkTime(1.5));
  }

  activeUsers.add(-1);
  sleep(thinkTime(2));
}

export function handleSummary(data) {
  const p95 = data.metrics.http_req_duration?.values?.['p(95)'] ?? 0;
  const failRate = data.metrics.http_req_failed?.values?.rate ?? 0;

  return {
    stdout: JSON.stringify({
      scenario: 'realistic_mixed',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 300 && failRate < 0.01,
    }, null, 2),
    'scripts/loadtest/results/realistic_mixed.json': JSON.stringify({
      scenario: 'realistic_mixed',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 300 && failRate < 0.01,
    }, null, 2),
  };
}
