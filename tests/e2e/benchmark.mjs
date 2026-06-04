#!/usr/bin/env node
import { chromium } from 'playwright';
import fs from 'fs';
import { mkdirSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPORTS_DIR = join(__dirname, 'reports');
mkdirSync(REPORTS_DIR, { recursive: true });

const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
const HEADED = process.argv.includes('--headed');

const benchmarks = [];

async function benchmark(name, fn) {
  await fn();

  const runs = [];
  const iterations = 5;

  for (let i = 0; i < iterations; i++) {
    const start = performance.now();
    await fn();
    const end = performance.now();
    runs.push(end - start);
  }

  runs.sort((a, b) => a - b);
  const result = {
    name,
    iterations,
    min: Math.min(...runs).toFixed(2),
    max: Math.max(...runs).toFixed(2),
    mean: (runs.reduce((a, b) => a + b, 0) / runs.length).toFixed(2),
    median: runs[Math.floor(runs.length / 2)].toFixed(2),
    p95: runs[Math.ceil(runs.length * 0.95) - 1]?.toFixed(2) || runs[runs.length - 1].toFixed(2),
    runs: runs.map(r => r.toFixed(2)),
  };
  benchmarks.push(result);
  return result;
}

async function main() {
  const browser = await chromium.launch({ headless: !HEADED });
  const page = await browser.newPage();

  console.log(`\n=== CivitForge Performance Benchmarks ===`);
  console.log(`Target: ${BASE_URL}`);
  console.log(`Mode: ${HEADED ? 'headed' : 'headless'}`);
  console.log(`Iterations per benchmark: 5 (1 warmup + 5 measured)\n`);

  const pages = ['/', '/repos', '/login', '/activity', '/search', '/explore', '/orgs', '/settings'];
  for (const route of pages) {
    await benchmark(`page_load:${route}`, async () => {
      await page.goto(BASE_URL + route, { waitUntil: 'networkidle', timeout: 10000 });
    });
  }

  const apiEndpoints = [
    { name: 'api:health', url: `${BASE_URL}/health` },
    { name: 'api:repos', url: `${BASE_URL}/api/v1/repos` },
    { name: 'api:search', url: `${BASE_URL}/api/v1/search?q=rust&page=1` },
    { name: 'api:activity', url: `${BASE_URL}/api/v1/activity?limit=50` },
    { name: 'api:openapi', url: `${BASE_URL}/api/v1/openapi.json` },
  ];
  for (const endpoint of apiEndpoints) {
    await benchmark(endpoint.name, async () => {
      const resp = await page.goto(endpoint.url, { waitUntil: 'commit', timeout: 10000 });
      if (!resp.ok()) throw new Error(`${resp.status()} from ${endpoint.url}`);
    });
  }

  await benchmark('form:login-fill', async () => {
    await page.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: 10000 });
    await page.fill('input#username, input[name="username"]', 'benchuser');
    await page.fill('input[type="password"]', 'benchpassword123');
  });

  await benchmark('navigation:home-to-repos', async () => {
    await page.goto(`${BASE_URL}/`, { waitUntil: 'networkidle', timeout: 10000 });
    await Promise.all([
      page.waitForNavigation({ timeout: 5000 }).catch(() => {}),
      page.click('a[href*="/repos"]'),
    ]);
  });

  const memBefore = await page.metrics();
  for (let i = 0; i < 10; i++) {
    await page.goto(`${BASE_URL}/repos`, { waitUntil: 'networkidle', timeout: 10000 });
    await page.goto(`${BASE_URL}/activity`, { waitUntil: 'networkidle', timeout: 10000 });
    await page.goto(`${BASE_URL}/search`, { waitUntil: 'networkidle', timeout: 10000 });
  }
  const memAfter = await page.metrics();
  const memLeak = (parseFloat(memAfter.JSHeapUsedSize) - parseFloat(memBefore.JSHeapUsedSize)) / (1024 * 1024);

  console.log('\n=== Benchmark Results ===\n');
  console.log('Benchmark                                   | Min (ms) | Mean (ms) | Median | P95');
  console.log('-'.repeat(100));
  for (const b of benchmarks) {
    const name = b.name.padEnd(44);
    const min = b.min.padStart(8);
    const mean = b.mean.padStart(9);
    const median = b.median.padStart(7);
    const p95 = b.p95.padStart(6);
    console.log(`${name} | ${min} | ${mean} | ${median} | ${p95}`);
  }

  console.log(`\nMemory: ${(parseFloat(memAfter.JSHeapUsedSize) / (1024 * 1024)).toFixed(2)} MB after navigation loop`);
  console.log(`Memory delta: ${memLeak.toFixed(2)} MB after 30 page loads (leak indicator)`);

  const report = {
    timestamp: new Date().toISOString(),
    base_url: BASE_URL,
    benchmarks,
    memory: {
      finalMB: (parseFloat(memAfter.JSHeapUsedSize) / (1024 * 1024)).toFixed(2),
      leakMB: memLeak.toFixed(2),
    },
  };
  const reportFile = join(REPORTS_DIR, `benchmark-${Date.now()}.json`);
  fs.writeFileSync(reportFile, JSON.stringify(report, null, 2));
  console.log(`\nBenchmark report saved to ${reportFile}`);

  await browser.close();
}

main().catch((e) => {
  console.error('Fatal error:', e);
  process.exit(2);
});
