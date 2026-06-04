import { chromium } from 'playwright';
import { ErrorCapture } from './debug-capture.mjs';
import { mkdirSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const SCREENSHOTS_DIR = join(__dirname, 'screenshots');
const BASE_URL = 'http://localhost:9091';
const NAV_TIMEOUT = 10000;
const ACTION_TIMEOUT = 5000;

mkdirSync(SCREENSHOTS_DIR, { recursive: true });

const headed = process.argv.includes('--headed');

const results = {
  pages: [],
  summary: { passed: 0, failed: 0, total: 0 },
  errors: { console: [], page: [], network: [], responses: [] },
  actions: [],
};

const capture = new ErrorCapture();

function log(action) {
  const entry = { timestamp: new Date().toISOString(), ...action };
  results.actions.push(entry);
  console.log(`  → ${action.description}`);
}

async function safeClick(page, selector, description, timeout = ACTION_TIMEOUT) {
  try {
    const el = page.locator(selector).first();
    await el.waitFor({ state: 'visible', timeout });
    await el.click({ timeout });
    log({ description });
    return true;
  } catch {
    log({ description: `SKIP click ${selector} (${description})` });
    return false;
  }
}

async function safeFill(page, selector, value, description) {
  try {
    const el = page.locator(selector).first();
    await el.waitFor({ state: 'visible', timeout: ACTION_TIMEOUT });
    await el.fill(value);
    log({ description });
    return true;
  } catch {
    log({ description: `SKIP fill ${selector} (${description})` });
    return false;
  }
}

async function safeScreenshot(page, name) {
  try {
    const path = join(SCREENSHOTS_DIR, `${name}.png`);
    await page.screenshot({ path, fullPage: true });
    return path;
  } catch {
    return null;
  }
}

async function traversePage(page, url, label, actionsFn) {
  const pageResult = {
    url,
    label,
    status: 'passed',
    errors: [],
    actions: [],
  };

  capture.reset();
  capture.attachToPage(page);

  try {
    await page.goto(url, { waitUntil: 'networkidle', timeout: NAV_TIMEOUT });
    pageResult.httpStatus = 'loaded';
  } catch (e) {
    pageResult.status = 'failed';
    pageResult.errors.push(`Navigation failed: ${e.message}`);
    await safeScreenshot(page, label);
    captureResults(pageResult);
    return pageResult;
  }

  try {
    await page.waitForTimeout(500);
    if (typeof actionsFn === 'function') {
      await actionsFn(page);
    }
  } catch (e) {
    pageResult.status = 'failed';
    pageResult.errors.push(`Action failed: ${e.message}`);
    await safeScreenshot(page, label);
  }

  const snap = capture.snapshot();
  if (snap.errors.length > 0) pageResult.errors.push(...snap.errors.map((e) => e.text));
  if (snap.pageErrors.length > 0) pageResult.errors.push(...snap.pageErrors.map((e) => e.message));
  if (snap.networkFailures.length > 0) pageResult.errors.push(...snap.networkFailures.map((n) => `NET: ${n.url} - ${n.failure}`));

  if (pageResult.errors.length > 0 && pageResult.status !== 'failed') {
    pageResult.status = 'errors';
  }

  await safeScreenshot(page, `${label}-final`);
  captureResults(pageResult);
  return pageResult;
}

function captureResults(pageResult) {
  results.pages.push(pageResult);
  results.summary.total++;
  if (pageResult.status === 'passed') results.summary.passed++;
  else results.summary.failed++;

  const snap = capture.snapshot();
  results.errors.console.push(...snap.errors);
  results.errors.page.push(...snap.pageErrors);
  results.errors.network.push(...snap.networkFailures);
  results.errors.responses.push(...snap.responses);
}

function printReport() {
  console.log('\n');
  console.log('══════════════════════════════════════════════════════');
  console.log('  CivitForge E2E Traversal Report');
  console.log('══════════════════════════════════════════════════════');
  console.log(`  Total Pages:  ${results.summary.total}`);
  console.log(`  Passed:       ${results.summary.passed}`);
  console.log(`  Failed:       ${results.summary.failed}`);
  console.log('──────────────────────────────────────────────────────');

  for (const p of results.pages) {
    const icon = p.status === 'passed' ? '✓' : p.status === 'failed' ? '✗' : '⚠';
    console.log(`  ${icon} [${p.status.toUpperCase()}] ${p.label} → ${p.url}`);
    if (p.errors.length > 0) {
      for (const err of p.errors) {
        console.log(`      ERROR: ${err}`);
      }
    }
  }

  if (results.errors.console.length > 0) {
    console.log('\n── Console Errors ──');
    for (const e of results.errors.console) {
      console.log(`  [${e.timestamp}] ${e.url} — ${e.text}`);
    }
  }

  if (results.errors.page.length > 0) {
    console.log('\n── Uncaught JS Exceptions ──');
    for (const e of results.errors.page) {
      console.log(`  [${e.timestamp}] ${e.url} — ${e.message}`);
    }
  }

  if (results.errors.network.length > 0) {
    console.log('\n── Network Failures ──');
    for (const n of results.errors.network) {
      console.log(`  [${n.timestamp}] ${n.method} ${n.url} — ${n.failure}`);
    }
  }

  if (results.errors.responses.length > 0) {
    console.log('\n── HTTP Error Responses ──');
    for (const r of results.errors.responses) {
      console.log(`  [${r.timestamp}] ${r.status} ${r.url}`);
    }
  }

  console.log('\n══════════════════════════════════════════════════════');
  const reportPath = join(__dirname, 'traversal-report.json');
  writeFileSync(reportPath, JSON.stringify(results, null, 2));
  console.log(`  Report saved: ${reportPath}`);
  console.log(`  Screenshots:  ${SCREENSHOTS_DIR}/`);
  console.log('══════════════════════════════════════════════════════\n');
}

async function main() {
  console.log(`Launching browser (headed: ${headed})...`);
  const browser = await chromium.launch({ headless: !headed });
  const context = await browser.newContext({
    viewport: { width: 1280, height: 720 },
    ignoreHTTPSErrors: true,
  });
  const page = await context.newPage();
  capture.attachToPage(page);

  console.log('\n═══ Traversal Starting ═══\n');

  console.log('[1/12] Home page (/)');
  await traversePage(page, `${BASE_URL}/`, 'home', async (p) => {
    await safeClick(p, 'a:has-text("Get Started")', 'Click "Get Started" link');
  });

  console.log('[2/12] Login page (/login)');
  await traversePage(page, `${BASE_URL}/login`, 'login', async (p) => {
    await safeFill(p, 'input[name="username"], input[type="text"], input[data-testid="username"]', 'e2e-test-user', 'Fill username');
    await safeFill(p, 'input[name="password"], input[type="password"]', 'e2e-test-password', 'Fill password');
    await safeClick(p, 'button:has-text("Login"), button[type="submit"]', 'Click Login button');

    await safeClick(p, 'a:has-text("Register"), button:has-text("Register"), [data-testid="register-toggle"]', 'Toggle to Register form');

    await safeFill(p, 'input[name="username"], input[type="text"], input[data-testid="register-username"]', 'e2e-new-user', 'Fill register username');
    await safeFill(p, 'input[name="email"], input[type="email"]', 'e2e@test.com', 'Fill register email');
    await safeFill(p, 'input[name="password"], input[type="password"]', 'e2e-password-123', 'Fill register password');
    await safeFill(p, 'input[name="confirmPassword"], input[name="confirm_password"]', 'e2e-password-123', 'Fill confirm password');
    await safeClick(p, 'button:has-text("Register"), button[type="submit"]', 'Click Register button');
  });

  console.log('[3/12] Repos list (/repos)');
  await traversePage(page, `${BASE_URL}/repos`, 'repos-list', async (p) => {
    await safeClick(p, 'a:has-text("New Repository"), button:has-text("New Repository")', 'Click "New Repository" button');
    await p.waitForTimeout(500);
    if (p.url().includes('new-repo')) {
      await p.goBack({ waitUntil: 'networkidle', timeout: NAV_TIMEOUT });
    }
  });

  console.log('[4/12] New repo form (/new-repo)');
  await traversePage(page, `${BASE_URL}/new-repo`, 'new-repo', async (p) => {
    await safeFill(p, 'input[name="name"], input[data-testid="repo-name"]', 'e2e-test-repo', 'Fill repo name');
    await safeFill(p, 'input[name="description"], textarea[name="description"], textarea[data-testid="repo-desc"]', 'E2E test repository', 'Fill description');

    const publicRadio = p.locator('input[value="public"], input[value="Public"]').first();
    try {
      await publicRadio.check({ timeout: ACTION_TIMEOUT });
      log({ description: 'Select public visibility' });
    } catch {
      log({ description: 'SKIP visibility radio' });
    }

    await safeClick(p, 'button:has-text("Create Repository"), button[type="submit"]', 'Click Create Repository button');
  });

  console.log('[5/12] Activity feed (/activity)');
  await traversePage(page, `${BASE_URL}/activity`, 'activity', async (p) => {
    const tabs = ['All', 'Push', 'Open Issue', 'Merge PR', 'Create Repo'];
    for (const tab of tabs) {
      await safeClick(p, `button:has-text("${tab}"), a:has-text("${tab}"), [data-testid="filter-${tab.toLowerCase().replace(/ /g, '-')}"]`, `Click filter tab "${tab}"`);
      await p.waitForTimeout(300);
    }
  });

  console.log('[6/12] Search page (/search)');
  await traversePage(page, `${BASE_URL}/search`, 'search', async (p) => {
    await safeFill(p, 'input[name="q"], input[type="search"], input[data-testid="search-input"]', 'test query', 'Fill search input');
    await safeClick(p, 'button:has-text("Search"), button[type="submit"]', 'Click Search button');
    await p.waitForTimeout(500);
  });

  console.log('[7/12] Explore page (/explore)');
  await traversePage(page, `${BASE_URL}/explore`, 'explore', async (p) => {
    await safeClick(p, 'a:has-text("Next"), button:has-text("Next"), [aria-label="Next page"]', 'Click pagination Next');
    await p.waitForTimeout(300);
    await safeClick(p, 'a:has-text("Previous"), button:has-text("Previous"), [aria-label="Previous page"]', 'Click pagination Previous');
  });

  console.log('[8/12] Organizations (/orgs)');
  await traversePage(page, `${BASE_URL}/orgs`, 'orgs', async (p) => {
    await safeClick(p, 'button:has-text("Create Organization"), a:has-text("Create Organization")', 'Click "Create Organization"');

    const modal = p.locator('[role="dialog"], .modal, [data-testid="create-org-modal"]');
    try {
      await modal.waitFor({ state: 'visible', timeout: 3000 });
      await safeFill(p, 'input[name="name"], input[data-testid="org-name"]', 'e2e-test-org', 'Fill org name');
      await safeFill(p, 'input[name="description"], textarea[data-testid="org-desc"]', 'E2E test org', 'Fill org description');
      await safeClick(p, 'button:has-text("Cancel"), [aria-label="Close"]', 'Close modal');
    } catch {
      log({ description: 'SKIP org modal (not found)' });
    }
  });

  console.log('[9/12] Settings (/settings)');
  await traversePage(page, `${BASE_URL}/settings`, 'settings', async (p) => {
    const sections = [
      { tab: 'Profile', selector: 'a:has-text("Profile"), button:has-text("Profile"), [data-testid="tab-profile"]' },
      { tab: 'SSH Keys', selector: 'a:has-text("SSH Keys"), button:has-text("SSH Keys"), [data-testid="tab-ssh"]' },
      { tab: 'Password', selector: 'a:has-text("Password"), button:has-text("Password"), [data-testid="tab-password"]' },
      { tab: 'Danger Zone', selector: 'a:has-text("Danger Zone"), button:has-text("Danger Zone"), [data-testid="tab-danger"]' },
    ];
    for (const s of sections) {
      await safeClick(p, s.selector, `Navigate to ${s.tab} section`);
      await p.waitForTimeout(300);
    }

    await safeFill(p, 'input[name="displayName"], input[name="display_name"]', 'E2E Test User', 'Fill display name');
    await safeFill(p, 'input[name="bio"], textarea[name="bio"]', 'E2E bio', 'Fill bio');
  });

  console.log('[10/12] Non-existent route (/nonexistent)');
  await traversePage(page, `${BASE_URL}/nonexistent`, '404-page', async (p) => {
    const has404 = await p.locator('text=/404|not found|page not found/i').first().isVisible().catch(() => false);
    if (!has404) {
      log({ description: 'WARNING: No 404 indicator found on non-existent route' });
    } else {
      log({ description: '404 page correctly rendered' });
    }
  });

  console.log('[11/12] Repo detail (/repos/test/test)');
  await traversePage(page, `${BASE_URL}/repos/test/test`, 'repo-detail', async (p) => {
    await p.waitForTimeout(500);
    const hasError = await p.locator('text=/not found|error|does not exist/i').first().isVisible().catch(() => false);
    if (hasError) {
      log({ description: 'Repo not found (expected for test repo)' });
    }
  });

  console.log('[12/12] Repo sub-pages (wiki, issues, code, pipelines)');
  const subPages = [
    { path: '/repos/test/test/wiki', label: 'repo-wiki' },
    { path: '/repos/test/test/issues', label: 'repo-issues' },
    { path: '/repos/test/test/code', label: 'repo-code' },
    { path: '/repos/test/test/pipelines', label: 'repo-pipelines' },
  ];
  for (const sp of subPages) {
    await traversePage(page, `${BASE_URL}${sp.path}`, sp.label, async (p) => {
      await p.waitForTimeout(500);
      log({ description: `Rendered ${sp.label}` });
    });
  }

  console.log('\n═══ Traversal Complete ═══');
  printReport();

  await browser.close();

  const hasAnyErrors =
    results.errors.console.length > 0 ||
    results.errors.page.length > 0 ||
    results.errors.network.length > 0;

  if (results.summary.failed > 0 || hasAnyErrors) {
    process.exit(1);
  }
  process.exit(0);
}

main().catch((e) => {
  console.error('Fatal traversal error:', e);
  process.exit(2);
});
