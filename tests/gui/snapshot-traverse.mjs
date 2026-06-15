#!/usr/bin/env node
// CivitForge GUI Snapshot Traversal
// Routes through all SPA paths, captures DOM HTML + screenshots at desktop and
// mobile viewports, and writes a drift report comparing against the
// Spatial Materialism + Amoebic UI design standard.
//
// Usage:
//   CIVITFORGE_URL=http://localhost:9091 node snapshot-traverse.mjs
//   node snapshot-traverse.mjs --headed        (visible browser)
//   node snapshot-traverse.mjs --auth          (login before traversal)
//
// Output: /tmp/civitforge-snapshots/<timestamp>/
//   dom/<route>-dom.html         (full DOM snapshot)
//   screenshots/<route>-desktop.png
//   screenshots/<route>-mobile.png
//   report.json                  (drift analysis + route status)

import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const TIMESTAMP = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
const OUT_DIR = process.env.SNAPSHOT_DIR || join('/tmp', 'civitforge-snapshots', TIMESTAMP);
const DOM_DIR = join(OUT_DIR, 'dom');
const SHOT_DIR = join(OUT_DIR, 'screenshots');

mkdirSync(DOM_DIR, { recursive: true });
mkdirSync(SHOT_DIR, { recursive: true });

const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
const HEADED = process.argv.includes('--headed');
const DO_AUTH = process.argv.includes('--auth');
const NAV_TIMEOUT = 12000;
const RENDER_WAIT = 800;

// Emojis that must NOT appear in rendered text (design standard: no emoji).
const EMOJI_RE = /[\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}\u{2B00}-\u{2BFF}\u{FE00}-\u{FE0F}\u{2300}-\u{23FF}\u{2A00}-\u{2AFF}\u{1F319}\u{1F3E0}\u{1F4C1}\u{1F50D}\u{1F527}\u{1F6AA}]/u;

// Design-language checks.
const DESIGN_CHECKS = {
  monospaceHeadings: (dom) => {
    // Spatial Materialism: headings should use monospace font family.
    const hasMono = /font-family[^;]*monospace/i.test(dom) ||
      /font-display|font-mono/.test(dom);
    return { pass: hasMono, detail: hasMono ? 'monospace font detected' : 'no monospace font in headings' };
  },
  amoebicBlob: (dom) => {
    // Amoebic UI: organic blob border-radius should be present.
    const hasBlob = /blob-bg|border-radius:\s*\d+%\s+\d+%/i.test(dom);
    return { pass: hasBlob, detail: hasBlob ? 'organic blob shapes detected' : 'no amoebic blob shapes found' };
  },
  brutalBorders: (dom) => {
    // Spatial Materialism: hard 2px borders.
    const hasBrutal = /border-brutal|border.*2px|border-2/i.test(dom);
    return { pass: hasBrutal, detail: hasBrutal ? 'brutal borders detected' : 'no brutal border tokens found' };
  },
  noEmojiInText: (text) => {
    const match = text.match(EMOJI_RE);
    return { pass: !match, detail: match ? `emoji found: U+${match[0].codePointAt(0).toString(16)}` : 'no emoji in body text' };
  },
};

// Route manifest (matches crates/civit-ui/src/app.rs router).
const ROUTES = [
  { path: '/', name: 'home', auth: false },
  { path: '/login', name: 'login', auth: false },
  { path: '/register', name: 'register', auth: false },
  { path: '/explore', name: 'explore', auth: false },
  { path: '/search', name: 'search', auth: false },
  { path: '/repos', name: 'repos-list', auth: true },
  { path: '/new-repo', name: 'new-repo', auth: true },
  { path: '/activity', name: 'activity', auth: true },
  { path: '/settings', name: 'settings', auth: true },
  { path: '/profile', name: 'profile', auth: true },
  { path: '/orgs', name: 'orgs', auth: true },
  { path: '/admin', name: 'admin', auth: true, admin: true },
  { path: '/admin/site-settings', name: 'admin-site-settings', auth: true, admin: true },
  { path: '/repos/admin/axum', name: 'repo-detail', auth: false },
  { path: '/repos/admin/axum/code', name: 'repo-code', auth: false },
  { path: '/repos/admin/axum/issues', name: 'repo-issues', auth: false },
  { path: '/repos/admin/axum/pulls', name: 'repo-pulls', auth: false },
  { path: '/repos/admin/axum/wiki', name: 'repo-wiki', auth: false },
  { path: '/repos/admin/axum/pipelines', name: 'repo-pipelines', auth: false },
  { path: '/repos/admin/axum/graph', name: 'repo-graph', auth: false },
  { path: '/repos/admin/axum/releases', name: 'repo-releases', auth: false },
  { path: '/repos/admin/axum/boards', name: 'repo-boards', auth: false },
  { path: '/nonexistent-route-404', name: 'not-found', auth: false },
];

const report = {
  timestamp: TIMESTAMP,
  baseUrl: BASE_URL,
  outputDir: OUT_DIR,
  totalRoutes: ROUTES.length,
  captured: 0,
  errors: [],
  pages: [],
  designSummary: { passCount: 0, failCount: 0, failures: [] },
};

async function waitForHydration(page) {
  // Leptos CSR: wait for WASM to hydrate, then settle.
  try {
    await page.waitForLoadState('networkidle', { timeout: NAV_TIMEOUT });
  } catch {
    // networkidle may not fire for SPAs; continue.
  }
  await page.waitForTimeout(RENDER_WAIT);
}

async function loginIfRequested(page) {
  if (!DO_AUTH) return false;
  console.log('  Attempting registration + login...');
  const user = `snap-${Date.now() % 100000}`;
  const email = `${user}@example.com`;
  const password = `SnapTest${Date.now()}!`;
  // Register
  await page.goto(`${BASE_URL}/register`, { timeout: NAV_TIMEOUT, waitUntil: 'domcontentloaded' });
  await waitForHydration(page);
  await page.fill('input[name="username"], input#username', user).catch(() => {});
  await page.fill('input[name="email"], input[type="email"], input#email', email).catch(() => {});
  await page.fill('input[name="password"], input[type="password"], input#password', password).catch(() => {});
  await page.click('button[type="submit"]').catch(() => {});
  await page.waitForTimeout(2000);
  return true;
}

async function captureRoute(browser, route) {
  // Hard per-route timeout: abort after 25s regardless of internal waits.
  const hardTimeoutMs = 25000;
  const page = await browser.newPage();
  const result = {
    name: route.name,
    path: route.path,
    auth: route.auth,
    status: null,
    httpStatus: null,
    domFile: null,
    screenshots: [],
    designChecks: {},
    errors: [],
  };

  const timeoutPromise = new Promise((_, reject) =>
    setTimeout(() => reject(new Error(`hard timeout ${hardTimeoutMs}ms`)), hardTimeoutMs)
  );

  try {
    await Promise.race([
      (async () => {
        // Desktop viewport.
        await page.setViewportSize({ width: 1440, height: 900 });
        const response = await page.goto(`${BASE_URL}${route.path}`, {
          timeout: NAV_TIMEOUT,
          waitUntil: 'domcontentloaded',
        });
        result.httpStatus = response ? response.status() : null;
        await waitForHydration(page);

        // Capture DOM.
        const dom = await page.content();
        const domFile = join(DOM_DIR, `${route.name}-dom.html`);
        writeFileSync(domFile, dom);
        result.domFile = domFile;

        // Desktop screenshot.
        const deskShot = join(SHOT_DIR, `${route.name}-desktop.png`);
        await page.screenshot({ path: deskShot, fullPage: false });
        result.screenshots.push(deskShot);

        // Mobile viewport screenshot.
        await page.setViewportSize({ width: 375, height: 812 });
        await page.waitForTimeout(400);
        const mobShot = join(SHOT_DIR, `${route.name}-mobile.png`);
        await page.screenshot({ path: mobShot, fullPage: false });
        result.screenshots.push(mobShot);

        // Design-language drift checks.
        const bodyText = await page.evaluate(() => document.body?.innerText || '');
        for (const [checkName, checkFn] of Object.entries(DESIGN_CHECKS)) {
          const input = checkName === 'noEmojiInText' ? bodyText : dom;
          const r = checkFn(input);
          result.designChecks[checkName] = r;
          if (r.pass) {
            report.designSummary.passCount++;
          } else {
            report.designSummary.failCount++;
            report.designSummary.failures.push({ route: route.name, check: checkName, detail: r.detail });
          }
        }
      })(),
      timeoutPromise,
    ]);
    result.status = 'ok';
    report.captured++;
  } catch (err) {
    result.status = 'error';
    result.errors.push(err.message);
    report.errors.push({ route: route.name, error: err.message });
    // Still capture an error screenshot if page is available.
    try {
      const errShot = join(SHOT_DIR, `${route.name}-error.png`);
      await page.screenshot({ path: errShot }).catch(() => {});
      result.screenshots.push(errShot);
    } catch {}
  }

  report.pages.push(result);
  await page.close();
  const tag = result.status === 'ok' ? 'OK' : 'FAIL';
  console.log(`  [${tag}] ${route.name} (${route.path})`);
  return result;
}

(async () => {
  console.log(`\nCivitForge GUI Snapshot Traversal`);
  console.log(`  Base URL:   ${BASE_URL}`);
  console.log(`  Output:     ${OUT_DIR}`);
  console.log(`  Routes:     ${ROUTES.length}`);
  console.log(`  Auth login: ${DO_AUTH}\n`);

  const browser = await chromium.launch({ headless: !HEADED });

  // Login context if requested.
  if (DO_AUTH) {
    const ctx = await browser.newContext();
    const loginPage = await ctx.newPage();
    await loginIfRequested(loginPage);
    // Reuse cookies via storage state is complex; the traversal visits public
    // routes for snapshot purposes. Auth-gated pages may redirect to login.
    await loginPage.close();
    await ctx.close();
  }

  for (const route of ROUTES) {
    await captureRoute(browser, route);
  }

  await browser.close();

  // Write report.
  const reportFile = join(OUT_DIR, 'report.json');
  writeFileSync(reportFile, JSON.stringify(report, null, 2));

  console.log(`\n=== Summary ===`);
  console.log(`  Captured:   ${report.captured}/${report.totalRoutes}`);
  console.log(`  Errors:     ${report.errors.length}`);
  console.log(`  Design:     ${report.designSummary.passCount} pass, ${report.designSummary.failCount} fail`);
  if (report.designSummary.failures.length > 0) {
    console.log(`\n  Design drift failures:`);
    for (const f of report.designSummary.failures.slice(0, 20)) {
      console.log(`    - ${f.route} / ${f.check}: ${f.detail}`);
    }
  }
  console.log(`\n  Report:     ${reportFile}`);
  console.log(`  DOM:        ${DOM_DIR}`);
  console.log(`  Screenshots: ${SHOT_DIR}\n`);

  process.exit(report.errors.length > 0 ? 1 : 0);
})();
