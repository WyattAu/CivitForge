#!/usr/bin/env node
import { chromium } from 'playwright';
import { mkdirSync, writeFileSync, readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const AUDIT_DIR = join(__dirname, 'audit-results');
const DOM_DIR = join(AUDIT_DIR, 'dom');
const SCREENSHOT_DIR = join(AUDIT_DIR, 'screenshots');
const REPORT_DIR = join(AUDIT_DIR, 'reports');

mkdirSync(DOM_DIR, { recursive: true });
mkdirSync(SCREENSHOT_DIR, { recursive: true });
mkdirSync(REPORT_DIR, { recursive: true });

const BASE_URL = process.env.CIVITFORGE_URL || 'http://192.168.1.191:9200';
const TIMEOUT = 25000;
const VIEWPORTS = [
  { name: 'desktop', width: 1440, height: 900 },
  { name: 'tablet', width: 768, height: 1024 },
  { name: 'mobile', width: 375, height: 812 },
];

// All routes from app.rs
const ROUTES = [
  // Public routes
  { path: '/', name: 'home', auth: false },
  { path: '/login', name: 'login', auth: false },
  { path: '/register', name: 'register', auth: false },
  { path: '/explore', name: 'explore', auth: false },
  { path: '/search', name: 'search', auth: false },
  // Auth-required routes
  { path: '/repos', name: 'repos-list', auth: true },
  { path: '/new-repo', name: 'new-repo', auth: true },
  { path: '/activity', name: 'activity', auth: true },
  { path: '/settings', name: 'settings', auth: true },
  { path: '/profile', name: 'profile', auth: true },
  // Org routes
  { path: '/orgs', name: 'orgs', auth: true },
  // Admin routes
  { path: '/admin', name: 'admin', auth: true, admin: true },
  { path: '/admin/site-settings', name: 'admin-site-settings', auth: true, admin: true },
  // Repo sub-pages (fixed repo admin/axum)
  { path: '/repos/admin/axum', name: 'repo-detail', auth: false },
  { path: '/repos/admin/axum/code', name: 'repo-code', auth: false },
  { path: '/repos/admin/axum/code/README.md', name: 'repo-code-file', auth: false },
  { path: '/repos/admin/axum/blame', name: 'repo-blame', auth: false },
  { path: '/repos/admin/axum/commits', name: 'repo-commits', auth: false },
  { path: '/repos/admin/axum/issues', name: 'repo-issues', auth: false },
  { path: '/repos/admin/axum/pulls', name: 'repo-pulls', auth: false },
  { path: '/repos/admin/axum/wiki', name: 'repo-wiki', auth: false },
  { path: '/repos/admin/axum/pipelines', name: 'repo-pipelines', auth: false },
  { path: '/repos/admin/axum/graph', name: 'repo-graph', auth: false },
  { path: '/repos/admin/axum/releases', name: 'repo-releases', auth: false },
  { path: '/repos/admin/axum/boards', name: 'repo-boards', auth: false },
  { path: '/repos/admin/axum/environments', name: 'repo-environments', auth: false },
  { path: '/repos/admin/axum/deployments', name: 'repo-deployments', auth: false },
  { path: '/repos/admin/axum/settings', name: 'repo-settings', auth: false },
  { path: '/repos/admin/axum/branch-protection', name: 'repo-branch-protection', auth: false },
  // 404 page
  { path: '/this-does-not-exist', name: 'not-found', auth: false },
];

const TEST_USER = {
  username: 'audit_user',
  password: 'AuditTest2026!',
};

const audit = {
  startTime: new Date().toISOString(),
  pages: [],
  networkErrors: [],
  consoleErrors: [],
  accessibilityIssues: [],
  brokenLinks: [],
  missingImages: [],
  performanceMetrics: [],
  domIssues: [],
  summary: {},
};

async function delay(ms) { return new Promise(r => setTimeout(r, ms)); }

async function captureDOM(page, name) {
  const dom = await page.evaluate(() => {
    const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT, null, false);
    const issues = [];
    let node;
    while (node = walker.nextNode()) {
      // Missing alt on images
      if (node.tagName === 'IMG' && !node.getAttribute('alt')) {
        issues.push({ type: 'missing-alt', tag: 'IMG', src: node.src?.substring(0, 100) });
      }
      // Missing aria-label on interactive elements
      if (['BUTTON', 'A', 'INPUT', 'SELECT', 'TEXTAREA'].includes(node.tagName)) {
        if (!node.getAttribute('aria-label') && !node.getAttribute('aria-labelledby') && !node.textContent?.trim()) {
          issues.push({ type: 'missing-aria-label', tag: node.tagName, id: node.id || '', name: node.name || '' });
        }
      }
      // Empty links
      if (node.tagName === 'A' && !node.textContent?.trim() && !node.getAttribute('aria-label') && !node.querySelector('img,svg')) {
        issues.push({ type: 'empty-link', href: node.href?.substring(0, 100) });
      }
      // Missing form labels
      if (['INPUT', 'SELECT', 'TEXTAREA'].includes(node.tagName) && node.type !== 'hidden' && node.type !== 'submit') {
        if (!node.getAttribute('aria-label') && !node.getAttribute('aria-labelledby')) {
          const id = node.id;
          const hasLabel = id && document.querySelector(`label[for="${id}"]`);
          if (!hasLabel) {
            issues.push({ type: 'missing-form-label', tag: node.tagName, id: node.id || '', name: node.name || '', type: node.type || '' });
          }
        }
      }
      // Tabindex > 0 (anti-pattern)
      if (node.getAttribute('tabindex') && parseInt(node.getAttribute('tabindex')) > 0) {
        issues.push({ type: 'positive-tabindex', tag: node.tagName, tabindex: node.getAttribute('tabindex') });
      }
      // Inline styles
      if (node.style && node.style.cssText && node.style.cssText.length > 0) {
        issues.push({ type: 'inline-style', tag: node.tagName, style: node.style.cssText.substring(0, 100) });
      }
    }
    return issues;
  });
  return dom;
}

async function captureA11y(page, name) {
  const a11y = await page.evaluate(() => {
    const issues = [];
    // Check heading hierarchy
    const headings = Array.from(document.querySelectorAll('h1,h2,h3,h4,h5,h6'));
    let prevLevel = 0;
    for (const h of headings) {
      const level = parseInt(h.tagName.substring(1));
      if (level > prevLevel + 1 && prevLevel > 0) {
        issues.push({ type: 'heading-skip', from: `h${prevLevel}`, to: h.tagName.toLowerCase(), text: h.textContent?.substring(0, 50) });
      }
      prevLevel = level;
    }
    // Check color contrast (basic)
    const textElements = Array.from(document.querySelectorAll('p, span, a, button, label, h1, h2, h3, h4, h5, h6'));
    const lowContrastSuspects = textElements.filter(el => {
      const style = window.getComputedStyle(el);
      const color = style.color;
      const bg = style.backgroundColor;
      // Both very light = low contrast on white bg
      return color === bg && color !== 'rgba(0, 0, 0, 0)';
    });
    if (lowContrastSuspects.length > 0) {
      issues.push({ type: 'suspected-low-contrast', count: lowContrastSuspects.length });
    }
    // Check focus management
    const focusableCount = document.querySelectorAll('a[href], button, input, select, textarea, [tabindex]').length;
    issues.push({ type: 'focusable-count', count: focusableCount });
    // Check lang attribute
    if (!document.documentElement.getAttribute('lang')) {
      issues.push({ type: 'missing-lang' });
    }
    // Check landmark regions
    const landmarks = {
      banner: document.querySelectorAll('header, [role="banner"]').length,
      nav: document.querySelectorAll('nav, [role="navigation"]').length,
      main: document.querySelectorAll('main, [role="main"]').length,
      contentinfo: document.querySelectorAll('footer, [role="contentinfo"]').length,
    };
    issues.push({ type: 'landmarks', ...landmarks });
    return issues;
  });
  return a11y;
}

async function capturePerformance(page, name) {
  const perf = await page.evaluate(() => {
    const entries = performance.getEntriesByType('navigation');
    const nav = entries[0] || {};
    return {
      domContentLoaded: nav.domContentLoadedEventEnd || 0,
      loadComplete: nav.loadEventEnd || 0,
      domInteractive: nav.domInteractive || 0,
      responseEnd: nav.responseEnd || 0,
      resourceCount: performance.getEntriesByType('resource').length,
      totalTransferSize: performance.getEntriesByType('resource').reduce((sum, r) => sum + (r.transferSize || 0), 0),
    };
  });
  return perf;
}

async function traverseRoute(browser, route) {
  const pageResult = {
    name: route.name,
    path: route.path,
    url: `${BASE_URL}${route.path}`,
    status: 'pending',
    loadTimeMs: null,
    screenshots: [],
    domIssues: [],
    a11yIssues: [],
    perfMetrics: {},
    consoleErrors: [],
    networkErrors: [],
  };

  let context;
  try {
    context = await browser.newContext({
      viewport: VIEWPORTS[0], // Desktop first
      userAgent: 'CivitForge-Audit/1.0',
    });
  } catch (e) {
    pageResult.status = 'failed';
    pageResult.error = `Failed to create context: ${e.message}`;
    return pageResult;
  }

  const page = await context.newPage();

  // Capture console errors
  page.on('console', msg => {
    if (msg.type() === 'error') {
      pageResult.consoleErrors.push({ text: msg.text(), url: page.url() });
    }
  });

  // Capture network errors
  page.on('response', resp => {
    if (resp.status() >= 400) {
      pageResult.networkErrors.push({ url: resp.url(), status: resp.status(), statusText: resp.statusText() });
    }
  });

  page.on('requestfailed', req => {
    pageResult.networkErrors.push({ url: req.url(), failure: req.failure()?.errorText || 'unknown' });
  });

  console.log(`  [${route.name}] Traversing ${route.path}...`);

  const start = Date.now();
  try {
    await page.goto(`${BASE_URL}${route.path}`, { waitUntil: 'networkidle', timeout: TIMEOUT });

    // Wait for WASM hydration
    try {
      await page.waitForSelector('nav a[href], main', { timeout: 8000 });
    } catch {
      await delay(1000);
    }
    pageResult.loadTimeMs = Date.now() - start;

    // Capture DOM
    pageResult.domIssues = await captureDOM(page, route.name);

    // Capture accessibility
    pageResult.a11yIssues = await captureA11y(page, route.name);

    // Capture performance
    pageResult.perfMetrics = await capturePerformance(page, route.name);

    // Full page screenshot
    const ssName = `${route.name}-desktop`;
    const ssPath = join(SCREENSHOT_DIR, `${ssName}.png`);
    await page.screenshot({ path: ssPath, fullPage: true });
    pageResult.screenshots.push(ssName);

    // DOM snapshot
    const html = await page.content();
    const domPath = join(DOM_DIR, `${route.name}-dom.html`);
    writeFileSync(domPath, html);

    // Mobile screenshot
    try {
      await page.setViewportSize({ width: VIEWPORTS[2].width, height: VIEWPORTS[2].height });
      await delay(500);
      const mobileSsPath = join(SCREENSHOT_DIR, `${route.name}-mobile.png`);
      await page.screenshot({ path: mobileSsPath, fullPage: true });
      pageResult.screenshots.push(`${route.name}-mobile`);
    } catch {}

    // Check for broken internal links
    const links = await page.$$eval('a[href]', els =>
      els.map(el => ({ href: el.href, text: el.textContent?.trim().substring(0, 50) }))
    );
    pageResult.links = links;

    // Check for missing images
    const images = await page.$$eval('img', els =>
      els.map(el => ({ src: el.src, alt: el.alt, loaded: el.complete && el.naturalWidth > 0 }))
    );
    const brokenImages = images.filter(i => !i.loaded);
    if (brokenImages.length > 0) {
      pageResult.brokenImages = brokenImages;
    }

    pageResult.status = 'passed';
  } catch (e) {
    pageResult.loadTimeMs = Date.now() - start;
    pageResult.status = 'failed';
    pageResult.error = e.message;

    // Error screenshot
    const errSsPath = join(SCREENSHOT_DIR, `${route.name}-error.png`);
    try {
      await page.screenshot({ path: errSsPath, fullPage: true });
      pageResult.screenshots.push(`${route.name}-error`);
    } catch {}
  }

  try { await context.close(); } catch {}
  return pageResult;
}

async function main() {
  console.log(`\n${'='.repeat(70)}`);
  console.log(`  CivitForge Comprehensive DOM & Screenshot Audit`);
  console.log(`  Target: ${BASE_URL}`);
  console.log(`  Time:   ${audit.startTime}`);
  console.log(`  Routes: ${ROUTES.length}`);
  console.log(`  Viewports: ${VIEWPORTS.map(v => `${v.name}(${v.width}x${v.height})`).join(', ')}`);
  console.log(`${'='.repeat(70)}\n`);

  const browser = await chromium.launch({ headless: true });
  const totalStart = Date.now();

  // Phase 1: Authenticate if needed
  let authToken = null;
  console.log('Phase 1: Authentication');
  const authPage = await browser.newPage();
  try {
    await authPage.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: TIMEOUT });
    try { await authPage.waitForSelector('nav a[href]', { timeout: 8000 }); } catch { await delay(1000); }

    // Try login
    const userInput = await authPage.$('input#username');
    if (userInput) {
      await authPage.fill('input#username', TEST_USER.username);
      await authPage.fill('input#password', TEST_USER.password);
      await authPage.press('input#password', 'Enter');
      await delay(2000);
      authToken = await authPage.evaluate(() => localStorage.getItem('civitforge_token'));
      if (authToken) {
        console.log('  Authenticated successfully');
      } else {
        console.log('  No token (may not be logged in - public pages still testable)');
      }
    }
  } catch (e) {
    console.log(`  Auth failed: ${e.message}`);
  }
  await authPage.close();

  // Phase 2: Traverse all routes
  console.log('\nPhase 2: Route Traversal');
  for (const route of ROUTES) {
    const result = await traverseRoute(browser, route);
    audit.pages.push(result);

    // Collect global errors
    audit.consoleErrors.push(...result.consoleErrors.map(e => ({ ...e, page: route.name })));
    audit.networkErrors.push(...result.networkErrors.map(e => ({ ...e, page: route.name })));
    audit.domIssues.push(...result.domIssues.map(e => ({ ...e, page: route.name })));
    audit.accessibilityIssues.push(...result.a11yIssues.map(e => ({ ...e, page: route.name })));

    if (result.status === 'passed') {
      console.log(`  [PASS] ${route.name} - ${result.loadTimeMs}ms, ${result.domIssues.length} DOM issues, ${result.a11yIssues.length} a11y issues`);
    } else {
      console.log(`  [FAIL] ${route.name} - ${result.error?.substring(0, 100)}`);
    }
  }

  await browser.close();
  const totalDuration = Date.now() - totalStart;

  // Phase 3: Generate Report
  console.log('\nPhase 3: Generating Report');

  const passed = audit.pages.filter(p => p.status === 'passed').length;
  const failed = audit.pages.filter(p => p.status === 'failed').length;

  // Categorize DOM issues
  const domIssueTypes = {};
  for (const issue of audit.domIssues) {
    domIssueTypes[issue.type] = (domIssueTypes[issue.type] || 0) + 1;
  }

  // Categorize a11y issues
  const a11yIssueTypes = {};
  for (const issue of audit.accessibilityIssues) {
    const key = issue.type;
    a11yIssueTypes[key] = (a11yIssueTypes[key] || 0) + 1;
  }

  // Performance summary
  const loadTimes = audit.pages.filter(p => p.loadTimeMs).map(p => p.loadTimeMs);
  const avgLoadTime = loadTimes.length > 0 ? loadTimes.reduce((a, b) => a + b, 0) / loadTimes.length : 0;
  const maxLoadTime = Math.max(...loadTimes);
  const minLoadTime = Math.min(...loadTimes);

  audit.summary = {
    routes: ROUTES.length,
    passed,
    failed,
    totalConsoleErrors: audit.consoleErrors.length,
    totalNetworkErrors: audit.networkErrors.length,
    totalDomIssues: audit.domIssues.length,
    totalA11yIssues: audit.accessibilityIssues.length,
    domIssueTypes,
    a11yIssueTypes,
    performance: {
      avgLoadTimeMs: Math.round(avgLoadTime),
      maxLoadTimeMs: maxLoadTime,
      minLoadTimeMs: minLoadTime,
    },
    durationMs: totalDuration,
  };

  // Write report
  const reportFile = join(REPORT_DIR, `audit-${Date.now()}.json`);
  writeFileSync(reportFile, JSON.stringify(audit, null, 2));

  // Print summary
  console.log(`\n${'='.repeat(70)}`);
  console.log(`  AUDIT SUMMARY`);
  console.log(`${'='.repeat(70)}`);
  console.log(`  Routes:        ${audit.summary.routes} total, ${passed} passed, ${failed} failed`);
  console.log(`  Duration:      ${(totalDuration / 1000).toFixed(1)}s`);
  console.log(`  Console Errors: ${audit.summary.totalConsoleErrors}`);
  console.log(`  Network Errors: ${audit.summary.totalNetworkErrors}`);
  console.log(`  DOM Issues:     ${audit.summary.totalDomIssues}`);
  console.log(`  A11y Issues:    ${audit.summary.totalA11yIssues}`);
  console.log(`  Performance:    avg=${audit.summary.performance.avgLoadTimeMs}ms, max=${audit.summary.performance.maxLoadTimeMs}ms`);
  console.log(`\n  DOM Issue Types:`);
  for (const [type, count] of Object.entries(domIssueTypes).sort((a, b) => b[1] - a[1])) {
    console.log(`    ${type}: ${count}`);
  }
  console.log(`\n  A11y Issue Types:`);
  for (const [type, count] of Object.entries(a11yIssueTypes).sort((a, b) => b[1] - a[1])) {
    console.log(`    ${type}: ${count}`);
  }
  console.log(`\n  Console Errors (first 10):`);
  for (const err of audit.consoleErrors.slice(0, 10)) {
    console.log(`    [${err.page}] ${err.text?.substring(0, 120)}`);
  }
  console.log(`\n  Network Errors:`);
  for (const err of audit.networkErrors.slice(0, 10)) {
    console.log(`    [${err.page}] ${err.status || err.failure} ${err.url?.substring(0, 100)}`);
  }
  console.log(`\n  Report: ${reportFile}`);
  console.log(`  DOM snapshots: ${DOM_DIR}`);
  console.log(`  Screenshots: ${SCREENSHOT_DIR}`);
  console.log(`${'='.repeat(70)}\n`);

  process.exit(failed > 0 ? 1 : 0);
}

main().catch(e => {
  console.error('Fatal error:', e);
  process.exit(2);
});
