#!/usr/bin/env node
// CivitForge Load Test
// Simulates concurrent users performing various operations against the CivitForge API.
//
// Usage:
//   CIVITFORGE_URL=http://localhost:9091 node load-test.mjs
//   node load-test.mjs --duration 30   (custom duration in seconds)
//
// Output: tests/gui/load-test-results.json

import { chromium } from 'playwright';
import { writeFileSync, mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
const DURATION_SEC = parseInt(process.argv.find((_, i, a) => a[i - 1] === '--duration') || '60', 10);

const CONCURRENT_BROWSERS = 10;
const CONCURRENT_ISSUE_CREATORS = 5;
const CONCURRENT_PR_CREATORS = 3;
const CONCURRENT_PUSH_USERS = 2;

const NAV_TIMEOUT = 15000;

// ── Utility ──

function timestamp() {
  return new Date().toISOString();
}

async function sleep(ms) {
  return new Promise(r => setTimeout(r, ms));
}

function percentile(sorted, p) {
  if (sorted.length === 0) return 0;
  const idx = Math.ceil((p / 100) * sorted.length) - 1;
  return sorted[Math.max(0, idx)];
}

// ── Results collector ──

class ResultsCollector {
  constructor() {
    this.requests = [];
    this.errors = [];
    this.startTime = null;
    this.endTime = null;
  }

  recordRequest(operation, startMs, endMs, success, httpStatus = null) {
    const durationMs = endMs - startMs;
    this.requests.push({
      operation,
      durationMs,
      success,
      httpStatus,
      timestamp: new Date(startMs).toISOString(),
    });
    if (!success) {
      this.errors.push({ operation, durationMs, httpStatus, timestamp: new Date(startMs).toISOString() });
    }
  }

  report() {
    const durations = this.requests.map(r => r.durationMs).sort((a, b) => a - b);
    const totalDurationSec = (this.endTime - this.startTime) / 1000;
    const totalRequests = this.requests.length;
    const errorCount = this.errors.length;
    const successCount = totalRequests - errorCount;
    const requestsPerSec = totalRequests / totalDurationSec;
    const avgResponseTime = durations.length > 0 ? durations.reduce((a, b) => a + b, 0) / durations.length : 0;
    const errorRate = totalRequests > 0 ? (errorCount / totalRequests) * 100 : 0;

    const ops = {};
    for (const r of this.requests) {
      if (!ops[r.operation]) ops[r.operation] = { count: 0, errors: 0, durations: [] };
      ops[r.operation].count++;
      if (!r.success) ops[r.operation].errors++;
      ops[r.operation].durations.push(r.durationMs);
    }
    for (const op of Object.values(ops)) {
      op.durations.sort((a, b) => a - b);
      op.avgMs = op.durations.reduce((a, b) => a + b, 0) / op.durations.length;
      op.p95Ms = percentile(op.durations, 95);
      op.p99Ms = percentile(op.durations, 99);
      delete op.durations;
    }

    return {
      timestamp: timestamp(),
      baseUrl: BASE_URL,
      config: {
        durationSec: DURATION_SEC,
        concurrentBrowsers: CONCURRENT_BROWSERS,
        concurrentIssueCreators: CONCURRENT_ISSUE_CREATORS,
        concurrentPrCreators: CONCURRENT_PR_CREATORS,
        concurrentPushUsers: CONCURRENT_PUSH_USERS,
      },
      summary: {
        totalRequests,
        successCount,
        errorCount,
        requestsPerSecond: Math.round(requestsPerSec * 100) / 100,
        avgResponseTimeMs: Math.round(avgResponseTime * 100) / 100,
        p50ResponseTimeMs: percentile(durations, 50),
        p95ResponseTimeMs: percentile(durations, 95),
        p99ResponseTimeMs: percentile(durations, 99),
        errorRate: Math.round(errorRate * 100) / 100,
      },
      byOperation: ops,
      errors: this.errors.slice(0, 100),
    };
  }
}

// ── Scenario: Browse repos ──

async function browseReposScenario(collector, stopTime) {
  const pages = ['/', '/explore', '/repos', '/search'];
  const browser = await chromium.launch({ headless: true });

  while (Date.now() < stopTime) {
    const page = await browser.newPage();
    const path = pages[Math.floor(Math.random() * pages.length)];
    const op = `browse:${path}`;
    const start = performance.now();
    try {
      const resp = await page.goto(`${BASE_URL}${path}`, {
        timeout: NAV_TIMEOUT,
        waitUntil: 'domcontentloaded',
      });
      await page.waitForTimeout(200);
      const status = resp ? resp.status() : 0;
      collector.recordRequest(op, start, performance.now(), status >= 200 && status < 400, status);
    } catch (e) {
      collector.recordRequest(op, start, performance.now(), false);
    }
    await page.close();
    await sleep(100 + Math.random() * 300);
  }

  await browser.close();
}

// ── Scenario: Create issues ──

async function createIssuesScenario(collector, stopTime) {
  const browser = await chromium.launch({ headless: true });
  const repos = ['admin/axum'];
  const issueBodies = [
    { title: 'Load test: intermittent failure', description: 'Auto-generated issue from load test.' },
    { title: 'Load test: performance regression', description: 'Response times degraded.' },
    { title: 'Load test: UI glitch', description: 'Layout broken on mobile.' },
  ];

  while (Date.now() < stopTime) {
    const page = await browser.newPage();
    const repo = repos[Math.floor(Math.random() * repos.length)];
    const issueBody = issueBodies[Math.floor(Math.random() * issueBodies.length)];
    const op = `create_issue`;
    const start = performance.now();
    try {
      // Try to create issue via API
      const resp = await page.goto(
        `${BASE_URL}/api/v1/repos/${repo}/issues`,
        { timeout: NAV_TIMEOUT, waitUntil: 'domcontentloaded' }
      );
      // Use page.evaluate to POST
      const result = await page.evaluate(async ({ baseUrl, repo, body }) => {
        try {
          const r = await fetch(`${baseUrl}/api/v1/repos/${repo}/issues`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
          });
          return { status: r.status };
        } catch (e) {
          return { status: 0 };
        }
      }, { baseUrl: BASE_URL, repo, body: issueBody });
      collector.recordRequest(op, start, performance.now(), result.status >= 200 && result.status < 400, result.status);
    } catch (e) {
      collector.recordRequest(op, start, performance.now(), false);
    }
    await page.close();
    await sleep(200 + Math.random() * 500);
  }

  await browser.close();
}

// ── Scenario: Create PRs ──

async function createPrsScenario(collector, stopTime) {
  const browser = await chromium.launch({ headless: true });
  const repos = ['admin/axum'];
  const prBodies = [
    { title: 'Load test: Fix typo', source_branch: 'fix/typo-1', target_branch: 'main' },
    { title: 'Load test: Add test', source_branch: 'test/add-1', target_branch: 'main' },
    { title: 'Load test: Update docs', source_branch: 'docs/update-1', target_branch: 'main' },
  ];

  while (Date.now() < stopTime) {
    const page = await browser.newPage();
    const repo = repos[Math.floor(Math.random() * repos.length)];
    const prBody = prBodies[Math.floor(Math.random() * prBodies.length)];
    const op = `create_pr`;
    const start = performance.now();
    try {
      const result = await page.evaluate(async ({ baseUrl, repo, body }) => {
        try {
          const r = await fetch(`${baseUrl}/api/v1/repos/${repo}/pulls`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(body),
          });
          return { status: r.status };
        } catch (e) {
          return { status: 0 };
        }
      }, { baseUrl: BASE_URL, repo, body: prBody });
      collector.recordRequest(op, start, performance.now(), result.status >= 200 && result.status < 400, result.status);
    } catch (e) {
      collector.recordRequest(op, start, performance.now(), false);
    }
    await page.close();
    await sleep(300 + Math.random() * 700);
  }

  await browser.close();
}

// ── Scenario: Push code (simulated via API) ──

async function pushCodeScenario(collector, stopTime) {
  const browser = await chromium.launch({ headless: true });
  const repos = ['admin/axum'];

  while (Date.now() < stopTime) {
    const page = await browser.newPage();
    const repo = repos[Math.floor(Math.random() * repos.length)];
    const op = 'push_code';
    const start = performance.now();
    try {
      // Simulate push by fetching the receive-pack endpoint (GET only to simulate load)
      const result = await page.evaluate(async ({ baseUrl, repo }) => {
        try {
          const r = await fetch(`${baseUrl}/api/v1/repos/${repo}/branches`);
          return { status: r.status };
        } catch (e) {
          return { status: 0 };
        }
      }, { baseUrl: BASE_URL, repo });
      collector.recordRequest(op, start, performance.now(), result.status >= 200 && result.status < 400, result.status);
    } catch (e) {
      collector.recordRequest(op, start, performance.now(), false);
    }
    await page.close();
    await sleep(500 + Math.random() * 1000);
  }

  await browser.close();
}

// ── Main ──

(async () => {
  console.log(`\nCivitForge Load Test`);
  console.log(`  Base URL:  ${BASE_URL}`);
  console.log(`  Duration:  ${DURATION_SEC}s`);
  console.log(`  Scenarios:`);
  console.log(`    - ${CONCURRENT_BROWSERS} concurrent browsers browsing repos`);
  console.log(`    - ${CONCURRENT_ISSUE_CREATORS} concurrent issue creators`);
  console.log(`    - ${CONCURRENT_PR_CREATORS} concurrent PR creators`);
  console.log(`    - ${CONCURRENT_PUSH_USERS} concurrent push users (simulated)\n`);

  const collector = new ResultsCollector();
  collector.startTime = Date.now();
  const stopTime = collector.startTime + DURATION_SEC * 1000;

  const scenarios = [];

  for (let i = 0; i < CONCURRENT_BROWSERS; i++) {
    scenarios.push(browseReposScenario(collector, stopTime));
  }
  for (let i = 0; i < CONCURRENT_ISSUE_CREATORS; i++) {
    scenarios.push(createIssuesScenario(collector, stopTime));
  }
  for (let i = 0; i < CONCURRENT_PR_CREATORS; i++) {
    scenarios.push(createPrsScenario(collector, stopTime));
  }
  for (let i = 0; i < CONCURRENT_PUSH_USERS; i++) {
    scenarios.push(pushCodeScenario(collector, stopTime));
  }

  console.log(`  Running ${scenarios.length} concurrent scenarios...`);
  await Promise.allSettled(scenarios);
  collector.endTime = Date.now();

  const report = collector.report();

  // Save results
  const resultsPath = join(__dirname, 'load-test-results.json');
  writeFileSync(resultsPath, JSON.stringify(report, null, 2));

  console.log(`\n=== Results ===`);
  console.log(`  Total requests:      ${report.summary.totalRequests}`);
  console.log(`  Success:             ${report.summary.successCount}`);
  console.log(`  Errors:              ${report.summary.errorCount}`);
  console.log(`  Requests/sec:        ${report.summary.requestsPerSecond}`);
  console.log(`  Avg response time:   ${report.summary.avgResponseTimeMs}ms`);
  console.log(`  P50 response time:   ${report.summary.p50ResponseTimeMs}ms`);
  console.log(`  P95 response time:   ${report.summary.p95ResponseTimeMs}ms`);
  console.log(`  P99 response time:   ${report.summary.p99ResponseTimeMs}ms`);
  console.log(`  Error rate:          ${report.summary.errorRate}%`);
  console.log(`\n  By operation:`);
  for (const [op, stats] of Object.entries(report.byOperation)) {
    console.log(`    ${op}: ${stats.count} reqs, ${stats.errors} errors, avg ${Math.round(stats.avgMs)}ms, p95 ${stats.p95Ms}ms`);
  }
  console.log(`\n  Results saved to: ${resultsPath}\n`);

  process.exit(report.summary.errorRate > 50 ? 1 : 0);
})();
