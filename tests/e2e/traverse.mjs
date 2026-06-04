#!/usr/bin/env node
import { chromium } from 'playwright';
import { ErrorCapture } from './debug-capture.m.js';
import { mkdirSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SCREENSHOTS_DIR = join(__dirname, 'screenshots');
const REPORTS_DIR = join(__dirname, 'reports');

const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
const HEADED = process.argv.includes('--headed');
const TIMEOUT = 15000;
const ACTION_TIMEOUT = 5000;

mkdirSync(SCREENSHOTS_DIR, { recursive: true });
mkdirSync(REPORTS_DIR, { recursive: true });

const results = {
  startTime: new Date().toISOString(),
  pages: [],
  totalActions: 0,
  totalErrors: 0,
  errors: { console: [], page: [], network: [], responses: [] },
};

const capture = new ErrorCapture();

async function clickIfExists(page, selector, name) {
  const el = await page.$(selector);
  if (el) {
    await el.click({ timeout: ACTION_TIMEOUT });
    await page.waitForTimeout(300);
  }
}

async function fillIfExists(page, selector, value, name) {
  const el = await page.$(selector);
  if (el) {
    await el.fill(value);
  }
}

async function traversePage(browser, url, name, actions) {
  const page = await browser.newPage();
  capture.reset();
  capture.attachToPage(page);

  const pageResult = {
    name,
    url,
    status: 'pending',
    loadTimeMs: null,
    actionsRun: 0,
    errors: [],
    screenshots: [],
  };

  console.log(`  Traversing: ${name} (${url})`);

  const start = Date.now();
  try {
    await page.goto(BASE_URL + url, { waitUntil: 'networkidle', timeout: TIMEOUT });
    pageResult.loadTimeMs = Date.now() - start;

    for (const action of actions) {
      try {
        await action.fn(page);
        pageResult.actionsRun++;
        results.totalActions++;
      } catch (e) {
        pageResult.errors.push({ action: action.name || 'unknown', error: e.message });
        const filename = `${name.replace(/[^a-z0-9]/gi, '_')}_${action.name || 'action'}.png`;
        try {
          await page.screenshot({ path: join(SCREENSHOTS_DIR, filename), fullPage: true });
          pageResult.screenshots.push(filename);
        } catch {
          // ignore screenshot failures
        }
      }
    }

    pageResult.status = 'passed';
  } catch (e) {
    pageResult.loadTimeMs = Date.now() - start;
    pageResult.status = 'failed';
    pageResult.errors.push({ action: 'navigation', error: e.message });
    try {
      await page.screenshot({ path: join(SCREENSHOTS_DIR, `${name.replace(/[^a-z0-9]/gi, '_')}.png`), fullPage: true });
    } catch {
      // ignore screenshot failures
    }
  }

  const snap = capture.snapshot();
  if (snap.errors.length > 0) {
    pageResult.errors.push(...snap.errors.map((e) => ({ action: 'console', error: e.text })));
  }
  if (snap.pageErrors.length > 0) {
    pageResult.errors.push(...snap.pageErrors.map((e) => ({ action: 'pageerror', error: e.message })));
  }
  if (snap.networkFailures.length > 0) {
    pageResult.errors.push(...snap.networkFailures.map((n) => ({ action: 'network', error: `NET: ${n.url} - ${n.failure}` })));
  }

  results.errors.console.push(...snap.errors);
  results.errors.page.push(...snap.pageErrors);
  results.errors.network.push(...snap.networkFailures);
  results.errors.responses.push(...snap.responses);

  results.totalErrors += pageResult.errors.length;
  results.pages.push(pageResult);

  await page.close();
}

async function main() {
  const browser = await chromium.launch({ headless: !HEADED });

  console.log(`\n=== CivitForge E2E Traversal ===`);
  console.log(`Target: ${BASE_URL}`);
  console.log(`Mode: ${HEADED ? 'headed' : 'headless'}`);
  console.log(`Started: ${results.startTime}\n`);

  await traversePage(browser, '/', 'home-logged-out', [
    { name: 'check-welcome', fn: async (p) => {
      const h1 = await p.$('h1, h2');
      if (!h1) throw new Error('No heading found on home page');
    }},
  ]);

  await traversePage(browser, '/login', 'login-page', [
    { name: 'check-form', fn: async (p) => {
      const username = await p.$('input#username, input[name="username"]');
      if (!username) throw new Error('Username input not found');
    }},
    { name: 'fill-login', fn: async (p) => {
      await fillIfExists(p, 'input#username, input[name="username"]', 'testuser');
      await fillIfExists(p, 'input[type="password"]', 'testpassword123');
    }},
    { name: 'click-login-btn', fn: async (p) => {
      await clickIfExists(p, 'button[type="submit"], button:has-text("Login")');
    }},
    { name: 'switch-to-register', fn: async (p) => {
      const regLink = await p.$('a:has-text("Register"), button:has-text("Register")');
      if (regLink) {
        await regLink.click();
        await p.waitForTimeout(500);
        const regForm = await p.$('input#username, input[name="username"]');
        if (!regForm) throw new Error('Register form not shown after click');
      }
    }},
  ]);

  await traversePage(browser, '/repos', 'repos-list', [
    { name: 'check-repo-list', fn: async (p) => {
      await p.waitForSelector('body', { timeout: ACTION_TIMEOUT });
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
  ]);

  await traversePage(browser, '/new-repo', 'new-repo-form', [
    { name: 'check-form-fields', fn: async (p) => {
      const name = await p.$('input#name, input[name="name"]');
      if (!name) throw new Error('Repo name input not found');
    }},
    { name: 'fill-form', fn: async (p) => {
      await fillIfExists(p, 'input#name, input[name="name"]', 'e2e-test-repo');
      await fillIfExists(p, 'textarea#description, textarea[name="description"]', 'Created by E2E traversal');
    }},
    { name: 'check-visibility', fn: async (p) => {
      const radios = await p.$$('input[type="radio"]');
      if (radios.length < 1) throw new Error('No visibility radios found');
    }},
    { name: 'click-create', fn: async (p) => {
      await clickIfExists(p, 'button[type="submit"], button:has-text("Create")');
    }},
  ]);

  await traversePage(browser, '/activity', 'activity-feed', [
    { name: 'check-activity-list', fn: async (p) => {
      await p.waitForTimeout(1000);
    }},
    { name: 'click-filter-all', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("All")');
      await p.waitForTimeout(500);
    }},
    { name: 'click-filter-push', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("Push")');
      await p.waitForTimeout(500);
    }},
    { name: 'click-filter-issues', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("Open Issue")');
      await p.waitForTimeout(500);
    }},
    { name: 'click-filter-pr', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("Merge PR")');
      await p.waitForTimeout(500);
    }},
    { name: 'click-filter-repos', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("Create Repo")');
      await p.waitForTimeout(500);
    }},
  ]);

  await traversePage(browser, '/search', 'search-page', [
    { name: 'check-search-input', fn: async (p) => {
      const input = await p.$('input[type="text"], input[name="q"], input#q');
      if (!input) throw new Error('Search input not found');
    }},
    { name: 'fill-search', fn: async (p) => {
      await fillIfExists(p, 'input[type="text"], input[name="q"]', 'rust');
    }},
    { name: 'click-search', fn: async (p) => {
      await clickIfExists(p, 'button[type="submit"], button:has-text("Search")');
      await p.waitForTimeout(2000);
    }},
  ]);

  await traversePage(browser, '/explore', 'explore-page', [
    { name: 'check-explore-list', fn: async (p) => {
      await p.waitForTimeout(1000);
    }},
  ]);

  await traversePage(browser, '/orgs', 'orgs-page', [
    { name: 'check-orgs-list', fn: async (p) => {
      await p.waitForTimeout(1000);
    }},
    { name: 'open-create-org-modal', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("Create Organization"), button:has-text("New")');
      await p.waitForTimeout(500);
    }},
    { name: 'fill-org-form', fn: async (p) => {
      await fillIfExists(p, 'input#name, input[name="name"]', 'e2e-test-org');
      await fillIfExists(p, 'input#display_name, input[name="display_name"]', 'E2E Test Org');
    }},
    { name: 'close-modal', fn: async (p) => {
      await p.keyboard.press('Escape');
      await p.waitForTimeout(500);
    }},
  ]);

  await traversePage(browser, '/settings', 'settings-page', [
    { name: 'check-profile-section', fn: async (p) => {
      await p.waitForTimeout(1000);
    }},
  ]);

  await traversePage(browser, '/this-page-does-not-exist', 'not-found', [
    { name: 'check-404', fn: async (p) => {
      await p.waitForTimeout(1000);
      const body = await p.textContent('body');
      if (!body.includes('404') && !body.includes('Not Found') && !body.includes('not found')) {
        throw new Error('404 page not shown');
      }
    }},
  ]);

  await traversePage(browser, '/repos/test/test', 'repo-detail', [
    { name: 'check-repo-page', fn: async (p) => {
      await p.waitForTimeout(2000);
    }},
  ]);

  await traversePage(browser, '/repos/test/test/wiki', 'wiki-page', [
    { name: 'check-wiki', fn: async (p) => {
      await p.waitForTimeout(2000);
    }},
  ]);

  await traversePage(browser, '/repos/test/test/issues', 'issues-page', [
    { name: 'check-issues', fn: async (p) => {
      await p.waitForTimeout(2000);
    }},
    { name: 'click-filter-open', fn: async (p) => {
      await clickIfExists(p, 'button:has-text("Open")');
      await p.waitForTimeout(500);
    }},
  ]);

  await traversePage(browser, '/repos/test/test/pipelines', 'pipelines-page', [
    { name: 'check-pipelines', fn: async (p) => {
      await p.waitForTimeout(2000);
    }},
  ]);

  await traversePage(browser, '/repos/test/test/code', 'code-browser', [
    { name: 'check-code', fn: async (p) => {
      await p.waitForTimeout(2000);
    }},
  ]);

  console.log('\n=== Traversal Results ===\n');
  for (const page of results.pages) {
    const status = page.status === 'passed' ? 'PASS' : 'FAIL';
    console.log(`[${status}] ${page.name} (${page.loadTimeMs}ms, ${page.actionsRun} actions${page.errors.length > 0 ? `, ${page.errors.length} errors` : ''})`);
    for (const err of page.errors) {
      console.log(`  ERROR: ${err.action}: ${err.error}`);
    }
  }

  console.log(`\n=== Summary ===`);
  console.log(`Pages: ${results.pages.length} total, ${results.pages.filter(p => p.status === 'passed').length} passed, ${results.pages.filter(p => p.status === 'failed').length} failed`);
  console.log(`Actions: ${results.totalActions}`);
  console.log(`Errors: ${results.totalErrors}`);

  const reportFile = join(REPORTS_DIR, `traverse-${Date.now()}.json`);
  writeFileSync(reportFile, JSON.stringify(results, null, 2));
  console.log(`\nReport saved to ${reportFile}`);

  await browser.close();
  process.exit(results.totalErrors > 0 ? 1 : 0);
}

main().catch((e) => {
  console.error('Fatal error:', e);
  process.exit(2);
});
