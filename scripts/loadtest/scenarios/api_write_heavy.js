import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend, Counter } from 'k6/metrics';

const config = {
  baseUrl: __ENV.BASE_URL || 'http://localhost:8080',
  authToken: __ENV.AUTH_TOKEN || '',
};

const errorRate = new Rate('write_errors');
const apiDuration = new Trend('write_api_duration', true);
const totalRequests = new Counter('write_total_requests');
const successfulRequests = new Counter('write_successful_requests');

export const options = {
  scenarios: {
    write_heavy: {
      executor: 'constant-vus',
      vus: 100,
      duration: '10m',
      exec: 'writeHeavyScenario',
      tags: { scenario: 'write_heavy' },
    },
  },
  thresholds: {
    http_req_duration: ['p(95)<500'],
    write_errors: ['rate<0.02'],
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

function makeWriteRequest(method, path, payload) {
  const headers = getHeaders();
  const url = `${config.baseUrl}${path}`;
  const body = JSON.stringify(payload);

  switch (method) {
    case 'POST': return http.post(url, body, { headers });
    case 'PUT': return http.put(url, body, { headers });
    case 'PATCH': return http.patch(url, body, { headers });
    default: return http.post(url, body, { headers });
  }
}

export function writeHeavyScenario() {
  const writeOps = [
    {
      name: 'create_issue',
      method: 'POST',
      path: '/api/v1/repos/test-owner/test-repo/issues',
      payload: () => ({
        title: `Load Test Issue ${Date.now()}`,
        body: 'Created during write-heavy load testing',
        labels: ['load-test'],
      }),
      weight: 30,
    },
    {
      name: 'push_commit',
      method: 'POST',
      path: '/api/v1/repos/test-owner/test-repo/push',
      payload: () => ({
        ref: 'main',
        sha: `sha-${Date.now()}`,
        updates: [{ path: 'test.txt', content: `load-test-${Date.now()}` }],
      }),
      weight: 25,
    },
    {
      name: 'run_pipeline',
      method: 'POST',
      path: '/api/v1/repos/test-owner/test-repo/pipelines/run',
      payload: () => ({
        pipeline: 'test-pipeline',
        parameters: { run_id: Date.now() },
      }),
      weight: 20,
    },
    {
      name: 'create_webhook',
      method: 'POST',
      path: '/api/v1/repos/test-owner/test-repo/webhooks',
      payload: () => ({
        url: `https://hooks.example.com/${Date.now()}`,
        events: ['push', 'issues'],
        secret: 'test-secret',
      }),
      weight: 10,
    },
    {
      name: 'create_deploy_key',
      method: 'POST',
      path: '/api/v1/repos/test-owner/test-repo/deploy-keys',
      payload: () => ({
        title: `deploy-key-${Date.now()}`,
        key: 'ssh-rsa AAAA...test',
        read_only: false,
      }),
      weight: 10,
    },
    {
      name: 'create_release',
      method: 'POST',
      path: '/api/v1/repos/test-owner/test-repo/releases',
      payload: () => ({
        tag_name: `v${Date.now()}`,
        name: `Release ${Date.now()}`,
        body: 'Release created during load testing',
      }),
      weight: 5,
    },
  ];

  const op = weightedRandom(writeOps);
  const payload = op.payload();
  const res = makeWriteRequest(op.method, op.path, payload);
  totalRequests.add(1);

  const passed = check(res, {
    [`${op.name}_status_ok`]: (r) => r.status >= 200 && r.status < 400,
    [`${op.name}_latency_ok`]: (r) => r.timings.duration < 500,
  });

  if (passed) {
    successfulRequests.add(1);
  } else {
    errorRate.add(1);
  }
  apiDuration.add(res.timings.duration);

  sleep(Math.random() * 1.5 + 0.5);
}

export function handleSummary(data) {
  const p95 = data.metrics.http_req_duration?.values?.['p(95)'] ?? 0;
  const failRate = data.metrics.http_req_failed?.values?.rate ?? 0;

  return {
    stdout: JSON.stringify({
      scenario: 'api_write_heavy',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 500 && failRate < 0.02,
    }, null, 2),
    'scripts/loadtest/results/api_write_heavy.json': JSON.stringify({
      scenario: 'api_write_heavy',
      timestamp: new Date().toISOString(),
      p95_latency_ms: p95,
      error_rate: failRate,
      total_requests: data.metrics.http_reqs?.values?.count ?? 0,
      rps: data.metrics.http_reqs?.values?.rate ?? 0,
      thresholds_met: p95 < 500 && failRate < 0.02,
    }, null, 2),
  };
}
