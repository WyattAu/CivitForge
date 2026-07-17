import http from 'k6/http';
import { check, sleep } from 'k6';
import { Rate, Trend } from 'k6/metrics';
import { config } from './k6_config.js';

// Custom metrics
const errorRate = new Rate('errors');
const apiDuration = new Trend('api_duration', true);

// Scenarios
export const options = {
  scenarios: {
    // Scenario 1: API read operations
    api_read: {
      executor: 'constant-vus',
      vus: 100,
      duration: '5m',
      exec: 'apiReadScenario',
      tags: { scenario: 'api_read' },
    },
    
    // Scenario 2: API write operations
    api_write: {
      executor: 'constant-vus',
      vus: 50,
      duration: '5m',
      exec: 'apiWriteScenario',
      tags: { scenario: 'api_write' },
      startTime: '5m', // Start after read scenario
    },
    
    // Scenario 3: Mixed read/write (realistic usage)
    mixed_usage: {
      executor: 'constant-vus',
      vus: 100,
      duration: '10m',
      exec: 'mixedUsageScenario',
      tags: { scenario: 'mixed_usage' },
      startTime: '10m', // Start after write scenario
    },
    
    // Scenario 4: Spike test (sudden 10x traffic)
    spike_test: {
      executor: 'ramping-vus',
      startVUs: 0,
      stages: [
        { duration: '30s', target: 1000 }, // Ramp up to 1000 VUs
        { duration: '1m', target: 1000 },  // Stay at 1000 VUs
        { duration: '30s', target: 0 },    // Ramp down
      ],
      exec: 'spikeTestScenario',
      tags: { scenario: 'spike_test' },
      startTime: '20m', // Start after mixed usage scenario
    },
  },
  
  thresholds: config.thresholds,
};

// Helper function for authentication headers
function getHeaders() {
  const headers = {
    'Content-Type': 'application/json',
  };
  
  if (config.authToken) {
    headers['Authorization'] = `Bearer ${config.authToken}`;
  }
  
  return headers;
}

// Scenario 1: API read operations
export function apiReadScenario() {
  const headers = getHeaders();
  
  // List repositories
  const listReposRes = http.get(`${config.baseUrl}/api/repos`, { headers });
  check(listReposRes, {
    'list repos status 200': (r) => r.status === 200,
    'list repos response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  apiDuration.add(listReposRes.timings.duration);
  
  sleep(1);
  
  // Get issues for a specific repo
  const getIssuesRes = http.get(`${config.baseUrl}/api/repos/test-owner/test-repo/issues`, { headers });
  check(getIssuesRes, {
    'get issues status 200': (r) => r.status === 200,
    'get issues response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  apiDuration.add(getIssuesRes.timings.duration);
  
  sleep(1);
  
  // Search code
  const searchRes = http.get(`${config.baseUrl}/api/search?q=test+query`, { headers });
  check(searchRes, {
    'search status 200': (r) => r.status === 200,
    'search response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  apiDuration.add(searchRes.timings.duration);
  
  sleep(1);
}

// Scenario 2: API write operations
export function apiWriteScenario() {
  const headers = getHeaders();
  
  // Create issue
  const createIssuePayload = JSON.stringify({
    title: 'Load Test Issue',
    body: 'This is a test issue created during load testing',
    labels: ['load-test'],
  });
  
  const createIssueRes = http.post(
    `${config.baseUrl}/api/repos/test-owner/test-repo/issues`,
    createIssuePayload,
    { headers }
  );
  check(createIssueRes, {
    'create issue status 201': (r) => r.status === 201,
    'create issue response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  apiDuration.add(createIssueRes.timings.duration);
  
  sleep(1);
  
  // Push code (simulated)
  const pushPayload = JSON.stringify({
    ref: 'main',
    sha: 'abc123',
    updates: [{ path: 'test.txt', content: 'load test content' }],
  });
  
  const pushRes = http.post(
    `${config.baseUrl}/api/repos/test-owner/test-repo/push`,
    pushPayload,
    { headers }
  );
  check(pushRes, {
    'push code status 200': (r) => r.status === 200,
    'push code response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  apiDuration.add(pushRes.timings.duration);
  
  sleep(1);
  
  // Run pipeline
  const pipelinePayload = JSON.stringify({
    pipeline: 'test-pipeline',
    parameters: { test: true },
  });
  
  const pipelineRes = http.post(
    `${config.baseUrl}/api/repos/test-owner/test-repo/pipelines/run`,
    pipelinePayload,
    { headers }
  );
  check(pipelineRes, {
    'run pipeline status 200': (r) => r.status === 200,
    'run pipeline response time < 500ms': (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  apiDuration.add(pipelineRes.timings.duration);
  
  sleep(1);
}

// Scenario 3: Mixed read/write (realistic usage pattern)
export function mixedUsageScenario() {
  const headers = getHeaders();
  
  // Simulate realistic user behavior with random mix of operations
  const operations = [
    { name: 'list_repos', method: 'GET', path: '/api/repos', weight: 30 },
    { name: 'get_issues', method: 'GET', path: '/api/repos/test-owner/test-repo/issues', weight: 25 },
    { name: 'search', method: 'GET', path: '/api/search?q=test', weight: 20 },
    { name: 'create_issue', method: 'POST', path: '/api/repos/test-owner/test-repo/issues', weight: 15 },
    { name: 'get_repo', method: 'GET', path: '/api/repos/test-owner/test-repo', weight: 10 },
  ];
  
  // Select random operation based on weights
  const totalWeight = operations.reduce((sum, op) => sum + op.weight, 0);
  let random = Math.random() * totalWeight;
  let selectedOp;
  
  for (const op of operations) {
    if (random < op.weight) {
      selectedOp = op;
      break;
    }
    random -= op.weight;
  }
  
  let res;
  const startTime = Date.now();
  
  if (selectedOp.method === 'GET') {
    res = http.get(`${config.baseUrl}${selectedOp.path}`, { headers });
  } else {
    const payload = JSON.stringify({
      title: 'Mixed Test Issue',
      body: 'Created during mixed usage testing',
    });
    res = http.post(`${config.baseUrl}${selectedOp.path}`, payload, { headers });
  }
  
  const duration = Date.now() - startTime;
  
  check(res, {
    [`${selectedOp.name} status 200`]: (r) => r.status === 200,
    [`${selectedOp.name} response time < 500ms`]: (r) => r.timings.duration < 500,
  }) || errorRate.add(1);
  
  apiDuration.add(duration);
  sleep(Math.random() * 3 + 1); // Random sleep between 1-4 seconds
}

// Scenario 4: Spike test (sudden 10x traffic)
export function spikeTestScenario() {
  const headers = getHeaders();
  
  // Simple read operation for spike test
  const res = http.get(`${config.baseUrl}/api/repos`, { headers });
  
  check(res, {
    'spike test status 200': (r) => r.status === 200,
    'spike test response time < 1000ms': (r) => r.timings.duration < 1000, // More lenient during spike
  }) || errorRate.add(1);
  
  apiDuration.add(res.timings.duration);
  sleep(0.5); // Shorter sleep during spike test
}

// Summary handler
export function handleSummary(data) {
  const summary = {
    timestamp: new Date().toISOString(),
    metrics: {
      http_req_duration: data.metrics.http_req_duration?.values,
      http_req_failed: data.metrics.http_req_failed?.values,
      errors: data.metrics.errors?.values,
      api_duration: data.metrics.api_duration?.values,
    },
    scenarios: {},
  };
  
  // Collect scenario-specific metrics
  for (const [key, value] of Object.entries(data.root_group?.checks || {})) {
    const scenarioMatch = key.match(/\[(\w+)\]/);
    if (scenarioMatch) {
      const scenario = scenarioMatch[1];
      if (!summary.scenarios[scenario]) {
        summary.scenarios[scenario] = { passed: 0, failed: 0 };
      }
      summary.scenarios[scenario][value.ok ? 'passed' : 'failed']++;
    }
  }
  
  return {
    'stdout': textSummary(data, { indent: ' ', enableColors: true }),
    'scripts/loadtest/results/summary.json': JSON.stringify(summary, null, 2),
  };
}

// Text summary helper
function textSummary(data, options = {}) {
  const lines = [];
  lines.push('📊 Load Test Results');
  lines.push('====================');
  lines.push(`Timestamp: ${new Date().toISOString()}`);
  lines.push('');
  
  lines.push('🎯 Thresholds:');
  if (data.metrics.http_req_duration) {
    const p95 = data.metrics.http_req_duration.values?.['p(95)'];
    lines.push(`  HTTP Request Duration (p95): ${p95?.toFixed(2)}ms (threshold: 500ms)`);
  }
  if (data.metrics.http_req_failed) {
    const failRate = data.metrics.http_req_failed.values?.rate;
    lines.push(`  HTTP Request Failed Rate: ${(failRate * 100)?.toFixed(2)}% (threshold: 1%)`);
  }
  lines.push('');
  
  lines.push('📈 Scenarios:');
  for (const [name, scenario] of Object.entries(data.scenarios || {})) {
    lines.push(`  ${name}:`);
    lines.push(`    VUs: ${scenario.vus}`);
    lines.push(`    Duration: ${scenario.duration}`);
  }
  
  return lines.join('\n');
}