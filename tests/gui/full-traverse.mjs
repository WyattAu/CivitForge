#!/usr/bin/env node
import { chromium } from 'playwright';
import { mkdirSync, writeFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const { DebugCapture } = await import('./debug-capture.mjs');

const __dirname = dirname(fileURLToPath(import.meta.url));
const SCREENSHOTS_DIR = join(__dirname, 'gui-screenshots');
const REPORTS_DIR = join(__dirname, 'gui-reports');

mkdirSync(SCREENSHOTS_DIR, { recursive: true });
mkdirSync(REPORTS_DIR, { recursive: true });

const BASE_URL = process.env.CIVITFORGE_URL || 'http://localhost:9091';
const HEADED = process.argv.includes('--headed');
const DEBUG = process.argv.includes('--debug');
const TIMEOUT = 20000;
const ACTION_TIMEOUT = 5000;

const TEST_USER = {
  email: `gui-traverse-${Date.now()}@example.com`,
  username: `guitest-${Date.now() % 100000}`,
  display_name: 'GUI Test User',
  password: `GuiTest${Date.now()}!`,
};

const created = {
  repo: null,
  org: null,
  issue: null,
  wikiSlug: null,
  token: null,
  userId: null,
};

const results = {
  startTime: new Date().toISOString(),
  pages: [],
  totalActions: 0,
  totalErrors: 0,
  errors: { console: [], page: [], network: [], responses: [] },
};

const capture = new DebugCapture({ screenshotDir: SCREENSHOTS_DIR, reportDir: REPORTS_DIR });

function pageUrl(path) {
  return `${BASE_URL}${path}`;
}

async function clickIfExists(page, selector, timeout = ACTION_TIMEOUT) {
  const el = await page.$(selector);
  if (el) {
    await el.click({ timeout });
    await page.waitForTimeout(300);
    return true;
  }
  return false;
}

async function fillIfExists(page, selector, value) {
  const el = await page.$(selector);
  if (el) {
    await el.fill(value);
    return true;
  }
  return false;
}

async function countElements(page, selector) {
  return (await page.$$(selector)).length;
}

async function waitForContent(page, timeout = ACTION_TIMEOUT) {
  await page.waitForLoadState('networkidle', { timeout }).catch(() => {});
  await page.waitForTimeout(500);
}

async function takeScreenshot(page, name) {
  await capture.screenshot(page, name);
}

function recordAction(pageResult, actionName, success = true, error = null) {
  pageResult.actionsRun++;
  results.totalActions++;
  if (!success) {
    const err = { action: actionName, error: error || 'unknown' };
    pageResult.errors.push(err);
  }
}

async function navigatePath(page, path) {
  await page.evaluate((p) => {
    window.location.href = p;
  }, path);
  await page.waitForTimeout(800);
  await page.waitForLoadState('networkidle', { timeout: TIMEOUT }).catch(() => {});
}

async function traversePage(browser, path, name, actions) {
  const page = await browser.newPage();
  await page.setViewportSize({ width: 1280, height: 800 });
  capture.reset();
  await capture.attachAll(page);

  const pageResult = {
    name,
    path,
    url: pageUrl(path),
    status: 'pending',
    loadTimeMs: null,
    actionsRun: 0,
    errors: [],
    screenshots: [],
  };

  console.log(`  Traversing: ${name} (${path})`);

  const start = Date.now();
  try {
    await page.goto(pageUrl(path), { waitUntil: 'networkidle', timeout: TIMEOUT });
    // Wait for WASM/Leptos CSR hydration. The WASM module takes 3-6s on
    // cold start to compile and hydrate. We detect hydration by checking
    // for the sidebar navigation (rendered by all routes after hydration).
    // networkidle fires before WASM finishes, so we poll until hydrated.
    try {
      await page.waitForSelector('nav a[href]', { timeout: 8000 });
    } catch {
      await page.waitForTimeout(1000);
    }
    pageResult.loadTimeMs = Date.now() - start;
    await takeScreenshot(page, `${name}-initial`);

    for (const action of actions) {
      try {
        await action.fn(page, pageResult);
        recordAction(pageResult, action.name);
      } catch (e) {
        recordAction(pageResult, action.name, false, e.message);
        try {
          const ss = await takeScreenshot(page, `${name}-${action.name}`);
          if (ss) pageResult.screenshots.push(ss);
        } catch {
          // ignore
        }
      }
    }

    pageResult.status = 'passed';
  } catch (e) {
    pageResult.loadTimeMs = Date.now() - start;
    pageResult.status = 'failed';
    recordAction(pageResult, 'navigation', false, e.message);
    try {
      await takeScreenshot(page, `${name}-nav-error`);
    } catch {
      // ignore
    }
  }

  const snap = capture.snapshot();
  for (const e of snap.errors) {
    pageResult.errors.push({ action: 'capture', error: `[${e.source}] ${e.error}` });
  }
  for (const n of snap.networkErrors) {
    results.errors.network.push(n);
  }

  results.totalErrors += pageResult.errors.filter(e => e.action !== 'capture').length;
  results.totalErrors += pageResult.errors.filter(e => e.action === 'capture' && e.error.includes('[console]')).length;
  results.pages.push(pageResult);

  await page.close();
}

// === 1. AUTHENTICATION FLOW ===

async function testRegister(browser) {
  await traversePage(browser, '/register', 'auth-register', [
    { name: 'verify-register-page', fn: async (p, pr) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 heading on register page');
      const text = await h1.textContent();
      if (!text.includes('Create Account') && !text.includes('Sign In')) {
        throw new Error(`Unexpected heading: ${text}`);
      }
    }},
    { name: 'switch-to-register', fn: async (p) => {
      const regBtn = await p.$('button:has-text("Register"), button:has-text("Don\'t have an account")');
      if (regBtn) {
        await regBtn.click();
        await p.waitForTimeout(500);
      }
      // Also try the link
      const link = await p.$('button:has-text("Register")');
      if (link) {
        await link.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'fill-register-form', fn: async (p) => {
      await fillIfExists(p, 'input#username', TEST_USER.username);
      await fillIfExists(p, 'input#display_name', TEST_USER.display_name);
      await fillIfExists(p, 'input#email', TEST_USER.email);
      await fillIfExists(p, 'input#password', TEST_USER.password);
      await fillIfExists(p, 'input#confirm_password', TEST_USER.password);
    }},
    { name: 'submit-register', fn: async (p) => {
      // Leptos Button component renders <button> without type="submit" — use Enter key
      await p.press('input#confirm_password', 'Enter');
      await p.waitForTimeout(2000);
    }},
    { name: 'check-register-success', fn: async (p) => {
      // After successful register, the app navigates to /repos
      const pathname = await p.evaluate(() => window.location.pathname);
      const bodyText = await p.textContent('body');
      // Check if token is stored
      const token = await p.evaluate(() => localStorage.getItem('civitforge_token'));
      if (token) {
        created.token = token;
      }
      // Check for error banners
      if (bodyText.includes('Failed') || bodyText.includes('already exists')) {
        // Registration may have failed or user exists - try login instead
        console.log('    Register may have failed, will try login...');
      }
    }},
  ]);
}

async function testLogin(browser) {
  await traversePage(browser, '/login', 'auth-login', [
    { name: 'verify-login-page', fn: async (p) => {
      await waitForContent(p);
      const userInput = await p.$('input#username');
      if (!userInput) throw new Error('Username input not found on login page');
    }},
    { name: 'switch-to-login', fn: async (p) => {
      const signInBtn = await p.$('button:has-text("Sign In"), button:has-text("Already have an account")');
      if (signInBtn) {
        await signInBtn.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'fill-login-form', fn: async (p) => {
      await fillIfExists(p, 'input#username', TEST_USER.username);
      await fillIfExists(p, 'input#password', TEST_USER.password);
    }},
    { name: 'submit-login', fn: async (p) => {
      // Leptos Button component renders <button> without type="submit" — use Enter key
      await p.press('input#password', 'Enter');
      await p.waitForTimeout(2000);
    }},
    { name: 'verify-token-stored', fn: async (p) => {
      const token = await p.evaluate(() => localStorage.getItem('civitforge_token'));
      if (token) {
        created.token = token;
        console.log('    Token stored in localStorage');
      } else {
        console.log('    No token found (login may require password auth)');
      }
    }},
    { name: 'verify-auth-me', fn: async (p) => {
      const token = created.token;
      if (!token) return;
      try {
        const resp = await p.evaluate(async (t) => {
          const r = await fetch('/api/v1/auth/me', {
            headers: { 'Authorization': `Bearer ${t}` },
          });
          return { status: r.status, body: await r.text() };
        }, token);
        if (resp.status === 200) {
          const user = JSON.parse(resp.body);
          created.userId = user.user_id || user.id;
          console.log(`    Authenticated as: ${user.username} (${created.userId})`);
        }
      } catch {
        // Auth endpoint may not be available
      }
    }},
  ]);
}

// === 2. HOME PAGE ===

async function testHomePage(browser) {
  await traversePage(browser, '/', 'home', [
    { name: 'verify-page-loads', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-nav-links', fn: async (p) => {
      const sidebar = await p.$('aside');
      if (!sidebar) throw new Error('Sidebar not found');
      const navLinks = await p.$$('aside a[href]');
      if (navLinks.length < 3) throw new Error(`Expected at least 3 nav links, found ${navLinks.length}`);
      const labels = [];
      for (const link of navLinks) {
        labels.push(await link.textContent());
      }
      console.log(`    Nav links found: ${labels.join(', ')}`);
    }},
    { name: 'check-brand', fn: async (p) => {
      const brand = await p.$('aside a[href="/"]');
      if (brand) {
        const text = await brand.textContent();
        if (!text.includes('CivitForge')) throw new Error('Brand link missing CivitForge text');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'home-nav');
    }},
  ]);
}

// === 3. REPOSITORIES ===

async function testReposPage(browser) {
  await traversePage(browser, '/repos', 'repos-list', [
    { name: 'verify-repos-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on repos page');
      const text = await h1.textContent();
      if (!text.includes('Repositor')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'check-new-repo-link', fn: async (p) => {
      const link = await p.$('a:has-text("New Repository")');
      if (!link) throw new Error('New Repository link not found');
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repos-list');
    }},
  ]);
}

async function testNewRepo(browser) {
  await traversePage(browser, '/new-repo', 'new-repo', [
    { name: 'verify-form', fn: async (p) => {
      await waitForContent(p);
      const nameInput = await p.$('input#repo-name');
      if (!nameInput) throw new Error('Repo name input not found');
    }},
    { name: 'fill-repo-form', fn: async (p) => {
      const repoName = `gui-test-repo-${Date.now() % 100000}`;
      await fillIfExists(p, 'input#repo-name', repoName);
      await fillIfExists(p, 'textarea#repo-description', 'Created by GUI traverse test');
      created.repo = repoName;
    }},
    { name: 'check-visibility-radios', fn: async (p) => {
      const radios = await p.$$('input[type="radio"]');
      if (radios.length < 2) throw new Error(`Expected at least 2 visibility radios, found ${radios.length}`);
    }},
    { name: 'screenshot-form', fn: async (p) => {
      await takeScreenshot(p, 'new-repo-filled');
    }},
    { name: 'submit-create-repo', fn: async (p) => {
      // Leptos Button has no type="submit" — use Enter on last input or click by text
      const submitBtn = await p.$('button:has-text("Create Repository")');
      if (submitBtn) {
        await submitBtn.click();
      } else {
        await p.press('input', 'Enter');
      }
      await p.waitForTimeout(3000);
    }},
    { name: 'verify-redirect', fn: async (p) => {
      const pathname = await p.evaluate(() => window.location.pathname);
      if (pathname.includes('/repos/')) {
        console.log(`    Redirected to: ${pathname}`);
      }
    }},
    { name: 'screenshot-result', fn: async (p) => {
      await takeScreenshot(p, 'new-repo-result');
    }},
  ]);
}

// === 4. ISSUES ===

async function testIssues(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping issues test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/issues`, 'repo-issues', [
    { name: 'verify-issues-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on issues page');
      const text = await h1.textContent();
      if (!text.includes('Issues')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'click-new-issue', fn: async (p) => {
      const btn = await p.$('button:has-text("New Issue"), .btn-new-issue');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'fill-issue-form', fn: async (p) => {
      await fillIfExists(p, 'input#new-issue-title', `Test issue from GUI traverse ${Date.now()}`);
      await fillIfExists(p, 'textarea#new-issue-description', 'This issue was created by the automated GUI traverse test.');
    }},
    { name: 'submit-issue', fn: async (p) => {
      const btn = await p.$('button:has-text("Submit Issue")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(2000);
      }
    }},
    { name: 'verify-issue-created', fn: async (p) => {
      const body = await p.textContent('body');
      if (!body.includes('Test issue from GUI')) {
        console.log('    Issue may not have been created (or list is empty)');
      } else {
        console.log('    Issue appears in list');
      }
    }},
    { name: 'check-filter-tabs', fn: async (p) => {
      const tabs = await p.$$('button');
      const tabTexts = [];
      for (const tab of tabs) {
        const t = await tab.textContent();
        if (['All', 'Open', 'In Progress', 'Closed'].includes(t)) {
          tabTexts.push(t);
        }
      }
      console.log(`    Filter tabs: ${tabTexts.join(', ') || 'none found'}`);
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-issues');
    }},
  ]);
}

// === 5. WIKI ===

async function testWiki(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping wiki test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;
  const wikiSlug = `getting-started-${Date.now() % 10000}`;
  created.wikiSlug = wikiSlug;

  await traversePage(browser, `/repos/${repoPath}/wiki`, 'repo-wiki', [
    { name: 'verify-wiki-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on wiki page');
      const text = await h1.textContent();
      if (!text.includes('Wiki')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'click-new-page', fn: async (p) => {
      const btn = await p.$('button:has-text("New Page")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'fill-wiki-form', fn: async (p) => {
      await fillIfExists(p, 'input#wiki-new-slug', wikiSlug);
      await fillIfExists(p, 'input#wiki-new-title', 'Getting Started');
      await fillIfExists(p, 'textarea#wiki-new-content', '# Welcome\n\nThis is the getting started guide for this repository.\n\n## Quick Start\n\n1. Clone the repo\n2. Build the project\n3. Run tests');
    }},
    { name: 'submit-wiki-page', fn: async (p) => {
      const btn = await p.$('button:has-text("Create Page")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(2000);
      }
    }},
    { name: 'verify-wiki-page-created', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Getting Started')) {
        console.log('    Wiki page appears in sidebar');
      }
    }},
    { name: 'navigate-to-wiki-page', fn: async (p) => {
      const pageBtn = await p.$(`button:has-text("Getting Started")`);
      if (pageBtn) {
        await pageBtn.click();
        await p.waitForTimeout(1500);
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-wiki');
    }},
  ]);
}

// === 6. CODE BROWSER ===

async function testCodeBrowser(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping code browser test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/code`, 'repo-code', [
    { name: 'verify-code-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-content', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('no files') || body.includes('empty') || body.includes('README')) {
        console.log('    Code browser shows placeholder/content');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-code');
    }},
  ]);
}

// === 7. PIPELINES ===

async function testPipelines(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping pipelines test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/pipelines`, 'repo-pipelines', [
    { name: 'verify-pipelines-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-pipeline-state', fn: async (p) => {
      const body = await p.textContent('body');
      const hasContent = body.includes('Pipeline') || body.includes('No pipeline') || body.includes('Run');
      if (!hasContent) {
        console.log('    Pipelines page loaded (content uncertain)');
      } else {
        console.log('    Pipelines content detected');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-pipelines');
    }},
  ]);
}

// === 8. REPO DETAIL ===

async function testRepoDetail(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping repo detail test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}`, 'repo-detail', [
    { name: 'verify-repo-detail', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes(created.repo)) {
        throw new Error(`Repo name "${created.repo}" not found on detail page`);
      }
    }},
    { name: 'check-repo-tabs', fn: async (p) => {
      const tabs = await p.$$('a');
      const tabTexts = [];
      for (const tab of tabs) {
        const t = await tab.textContent();
        if (['Code', 'Issues', 'Wiki', 'Pipelines', 'Settings'].includes(t)) {
          tabTexts.push(t);
        }
      }
      console.log(`    Repo tabs: ${tabTexts.join(', ') || 'none found'}`);
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-detail');
    }},
  ]);
}

// === 9. EXPLORE ===

async function testExplore(browser) {
  await traversePage(browser, '/explore', 'explore', [
    { name: 'verify-explore-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-content', fn: async (p) => {
      const body = await p.textContent('body');
      const hasContent = body.includes('Explore') || body.includes('repo') || body.includes('popular');
      if (hasContent) {
        console.log('    Explore page has content');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'explore');
    }},
  ]);
}

// === 10. SEARCH ===

async function testSearch(browser) {
  await traversePage(browser, '/search', 'search', [
    { name: 'verify-search-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on search page');
      const text = await h1.textContent();
      if (!text.includes('Search')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'check-search-input', fn: async (p) => {
      const input = await p.$('input#search-input');
      if (!input) throw new Error('Search input not found');
    }},
    { name: 'fill-search', fn: async (p) => {
      await fillIfExists(p, 'input#search-input', 'test');
    }},
    { name: 'submit-search', fn: async (p) => {
      // Try form submit
      const form = await p.$('form');
      if (form) {
        await form.evaluate((f) => f.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })));
        await p.waitForTimeout(1000);
      }
      // Also try search button click
      await clickIfExists(p, 'button:has-text("Search")');
      await p.waitForTimeout(2000);
    }},
    { name: 'check-results', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No results found')) {
        console.log('    Search returned no results (expected for "test" query)');
      } else if (body.includes('Code Search Results')) {
        console.log('    Search returned results');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'search-results');
    }},
  ]);
}

// === 11. SETTINGS ===

async function testSettings(browser) {
  await traversePage(browser, '/settings', 'settings', [
    { name: 'verify-settings-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on settings page');
      const text = await h1.textContent();
      if (!text.includes('Settings')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'check-profile-form', fn: async (p) => {
      const displayName = await p.$('input#settings-display-name');
      if (displayName) {
        console.log('    Profile form loaded with display name field');
      }
    }},
    { name: 'check-password-form', fn: async (p) => {
      const pwCurrent = await p.$('input#pw-current');
      if (pwCurrent) {
        console.log('    Change password form found');
      }
    }},
    { name: 'check-ssh-keys-section', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('SSH Keys')) {
        console.log('    SSH Keys section present');
      }
    }},
    { name: 'check-danger-zone', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Danger Zone')) {
        console.log('    Danger Zone section present');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'settings');
    }},
  ]);
}

// === 12. ACTIVITY ===

async function testActivity(browser) {
  await traversePage(browser, '/activity', 'activity', [
    { name: 'verify-activity-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-activity-content', fn: async (p) => {
      const body = await p.textContent('body');
      const hasContent = body.includes('Activity') || body.includes('activity') || body.includes('feed');
      if (hasContent) {
        console.log('    Activity page has content');
      }
    }},
    { name: 'check-filters', fn: async (p) => {
      const buttons = await p.$$('button');
      const filterTexts = [];
      for (const btn of buttons) {
        const t = await btn.textContent();
        if (['All', 'Push', 'Open Issue', 'Merge PR', 'Create Repo'].includes(t)) {
          filterTexts.push(t);
        }
      }
      console.log(`    Filter buttons: ${filterTexts.join(', ') || 'none found'}`);
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'activity');
    }},
  ]);
}

// === 13. ORGANIZATIONS ===

async function testOrgs(browser) {
  await traversePage(browser, '/orgs', 'orgs', [
    { name: 'verify-orgs-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on orgs page');
      const text = await h1.textContent();
      if (!text.includes('Organizations') && !text.includes('Organization')) {
        throw new Error(`Unexpected h1: ${text}`);
      }
    }},
    { name: 'click-new-org', fn: async (p) => {
      const btn = await p.$('button:has-text("New Organization"), .btn-new-org');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(1000);
        console.log('    Opened create org modal');
      }
    }},
    { name: 'fill-org-form', fn: async (p) => {
      const orgName = `gui-test-org-${Date.now() % 100000}`;
      await fillIfExists(p, 'input#org-name', orgName);
      await fillIfExists(p, 'input#org-display-name', 'GUI Test Org');
      await fillIfExists(p, 'textarea#org-description', 'Created by GUI traverse');
      created.org = orgName;
    }},
    { name: 'submit-org', fn: async (p) => {
      const submitBtn = await p.$('button:has-text("Create")');
      if (submitBtn) {
        await submitBtn.click();
        await p.waitForTimeout(2000);
      }
      await p.keyboard.press('Escape');
      await p.waitForTimeout(500);
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'orgs');
    }},
  ]);
}

// === 14. RELEASES ===

async function testReleases(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping releases test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/releases`, 'repo-releases', [
    { name: 'verify-releases-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on releases page');
      const text = await h1.textContent();
      if (!text.includes('Release')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'check-create-release-btn', fn: async (p) => {
      const btn = await p.$('button:has-text("New Release"), button:has-text("Create Release")');
      if (btn) {
        console.log('    Create Release button found');
      } else {
        console.log('    Create Release button not found (may require permissions)');
      }
    }},
    { name: 'check-release-list', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No releases') || body.includes('no releases')) {
        console.log('    No releases yet (expected for new repo)');
      } else {
        console.log('    Release list has content');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-releases');
    }},
  ]);
}

// === 15. BOARDS ===

async function testBoards(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping boards test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/boards`, 'repo-boards', [
    { name: 'verify-boards-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on boards page');
      const text = await h1.textContent();
      if (!text.includes('Board')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'check-board-columns', fn: async (p) => {
      const body = await p.textContent('body');
      const hasColumns = body.includes('Todo') || body.includes('In Progress') || body.includes('Done');
      if (hasColumns) {
        console.log('    Kanban columns detected');
      } else {
        console.log('    Board may be empty or not yet created');
      }
    }},
    { name: 'check-create-board-btn', fn: async (p) => {
      const btn = await p.$('button:has-text("New Board"), button:has-text("Create Board")');
      if (btn) {
        console.log('    Create Board button found');
      } else {
        console.log('    Create Board button not found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-boards');
    }},
  ]);
}

// === 16. COMMIT GRAPH ===

async function testGraph(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping graph test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/graph`, 'repo-graph', [
    { name: 'verify-graph-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on graph page');
      const text = await h1.textContent();
      if (!text.includes('Graph') && !text.includes('Commit') && !text.includes('History')) {
        throw new Error(`Unexpected h1: ${text}`);
      }
    }},
    { name: 'check-graph-canvas', fn: async (p) => {
      const svg = await p.$('svg');
      const canvas = await p.$('canvas');
      if (svg || canvas) {
        console.log('    Graph visualization element found');
      } else {
        console.log('    No SVG/canvas graph element (may use DOM-based graph)');
      }
    }},
    { name: 'check-graph-entries', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No commits') || body.includes('no commits')) {
        console.log('    No commits yet (expected for new repo)');
      } else {
        console.log('    Commit history has entries');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-graph');
    }},
  ]);
}

// === 17. ADMIN PANEL ===

async function testAdmin(browser) {
  await traversePage(browser, '/admin', 'admin', [
    { name: 'verify-admin-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('Admin') && !body.includes('admin')) {
        throw new Error('Admin panel did not load');
      }
    }},
    { name: 'check-admin-nav', fn: async (p) => {
      const body = await p.textContent('body');
      const hasSections = body.includes('Users') || body.includes('Repos') || body.includes('System');
      if (hasSections) {
        console.log('    Admin sections detected');
      } else {
        console.log('    Admin page loaded (sections may require permissions)');
      }
    }},
    { name: 'check-user-management', fn: async (p) => {
      const usersLink = await p.$('a:has-text("Users"), button:has-text("Users")');
      if (usersLink) {
        console.log('    User management link found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'admin');
    }},
  ]);
}

// === 18. SIDEBAR LOCALE SWITCHER ===

async function testSidebarLocaleSwitcher(browser) {
  await traversePage(browser, '/', 'sidebar-locale', [
    { name: 'verify-sidebar-loads', fn: async (p) => {
      await waitForContent(p);
      const sidebar = await p.$('aside');
      if (!sidebar) throw new Error('Sidebar not found');
    }},
    { name: 'check-locale-switcher', fn: async (p) => {
      const select = await p.$('select');
      const langBtn = await p.$('button:has-text("EN"), button:has-text("Language"), button:has-text("Locale")');
      if (select) {
        const options = await select.$$('option');
        const optionTexts = [];
        for (const opt of options) {
          optionTexts.push(await opt.textContent());
        }
        console.log(`    Locale dropdown found with options: ${optionTexts.join(', ')}`);
      } else if (langBtn) {
        console.log('    Language button found in sidebar');
      } else {
        console.log('    Locale switcher not found (may not be implemented)');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'sidebar-locale');
    }},
  ]);
}

// === 19. BRANCH PROTECTION ===

async function testBranchProtection(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping branch protection test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/branch-protection`, 'repo-branch-protection', [
    { name: 'verify-branch-protection-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on branch protection page');
      const text = await h1.textContent();
      if (!text.includes('Branch') && !text.includes('Protection')) {
        throw new Error(`Unexpected h1: ${text}`);
      }
    }},
    { name: 'check-protection-rules', fn: async (p) => {
      const body = await p.textContent('body');
      const hasRules = body.includes('main') || body.includes('master') || body.includes('default branch');
      if (hasRules) {
        console.log('    Branch protection rules detected');
      } else {
        console.log('    Branch protection page loaded (may have no rules)');
      }
    }},
    { name: 'check-require-pr-toggle', fn: async (p) => {
      const toggles = await p.$$('input[type="checkbox"], button[role="switch"]');
      if (toggles.length > 0) {
        console.log(`    Found ${toggles.length} toggle(s) for protection settings`);
      } else {
        console.log('    No toggle elements found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-branch-protection');
    }},
  ]);
}

// === 20. TEAM MANAGEMENT ===

async function testTeamManagement(browser) {
  if (!created.org) {
    console.log('    Skipping team management test (no org created)');
    return;
  }

  const orgId = created.org;

  await traversePage(browser, `/orgs/${orgId}/teams`, 'org-teams', [
    { name: 'verify-teams-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on teams page');
      const text = await h1.textContent();
      if (!text.includes('Team')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'check-create-team-btn', fn: async (p) => {
      const btn = await p.$('button:has-text("New Team"), button:has-text("Create Team")');
      if (btn) {
        console.log('    Create Team button found');
      } else {
        console.log('    Create Team button not found');
      }
    }},
    { name: 'check-team-list', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No teams') || body.includes('no teams')) {
        console.log('    No teams yet (expected for new org)');
      } else {
        console.log('    Team list has content');
      }
    }},
    { name: 'click-create-team', fn: async (p) => {
      const btn = await p.$('button:has-text("New Team"), button:has-text("Create Team")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'fill-team-form', fn: async (p) => {
      await fillIfExists(p, 'input#team-name', `gui-test-team-${Date.now() % 100000}`);
      await fillIfExists(p, 'textarea#team-description', 'Created by GUI traverse test');
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'org-teams');
    }},
  ]);
}

// === 21. NAVIGATION ===

async function testNavigation(browser) {
  const BASE = 'http://localhost:9091';
  await traversePage(browser, '/', 'navigation-test', [
    { name: 'navigate-home', fn: async (p) => {
      await navigatePath(p, `${BASE}/`);
      const pathname = await p.evaluate(() => window.location.pathname);
      if (pathname !== '/') throw new Error(`Expected /, got ${pathname}`);
    }},
    { name: 'navigate-repos', fn: async (p) => {
      await navigatePath(p, `${BASE}/repos`);
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('Repositor')) throw new Error('Repos page did not load');
    }},
    { name: 'navigate-explore', fn: async (p) => {
      await navigatePath(p, `${BASE}/explore`);
      await waitForContent(p);
    }},
    { name: 'navigate-search', fn: async (p) => {
      await navigatePath(p, `${BASE}/search`);
      await waitForContent(p);
    }},
    { name: 'navigate-settings', fn: async (p) => {
      await navigatePath(p, `${BASE}/settings`);
      await waitForContent(p);
    }},
    { name: 'navigate-activity', fn: async (p) => {
      await navigatePath(p, `${BASE}/activity`);
      await waitForContent(p);
    }},
    { name: 'navigate-orgs', fn: async (p) => {
      await navigatePath(p, `${BASE}/orgs`);
      await waitForContent(p);
    }},
    { name: 'navigate-login', fn: async (p) => {
      await navigatePath(p, `${BASE}/login`);
      await waitForContent(p);
    }},
    { name: 'browser-back', fn: async (p) => {
      await p.goBack({ waitUntil: 'networkidle', timeout: TIMEOUT }).catch(() => {});
      await p.waitForTimeout(500);
    }},
    { name: 'browser-forward', fn: async (p) => {
      await p.goForward({ waitUntil: 'networkidle', timeout: TIMEOUT }).catch(() => {});
      await p.waitForTimeout(500);
    }},
    { name: 'verify-path-routing', fn: async (p) => {
      await navigatePath(p, `${BASE}/repos`);
      const pathname = await p.evaluate(() => window.location.pathname);
      if (!pathname.includes('repos')) throw new Error(`Path routing broken: ${pathname}`);
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'navigation-final');
    }},
  ]);
}

// === 22. 404 PAGE ===

async function testNotFound(browser) {
  await traversePage(browser, '/this-page-does-not-exist-at-all', 'not-found', [
    { name: 'check-404', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('404') && !body.includes('Not Found') && !body.includes('not found')) {
        throw new Error('404/Not Found text not displayed on non-existent route');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'not-found');
    }},
  ]);
}

// === CLEANUP ===

async function cleanup(browser) {
  if (!created.token) {
    console.log('\n  Cleanup: No token, skipping cleanup');
    return;
  }

  const page = await browser.newPage();
  const token = created.token;

  try {
    // Delete org if created
    if (created.org) {
      try {
        const resp = await page.evaluate(async ({ token, orgName }) => {
          // Try to find the org first
          const r = await fetch(`/api/v1/orgs?per_page=100`, {
            headers: { 'Authorization': `Bearer ${token}` },
          });
          const data = await r.json();
          const org = data.data?.find((o) => o.name === orgName);
          if (org) {
            const del = await fetch(`/api/v1/orgs/${org.id}`, {
              method: 'DELETE',
              headers: { 'Authorization': `Bearer ${token}` },
            });
            return { deleted: del.ok, id: org.id };
          }
          return { deleted: false, reason: 'not found' };
        }, { token, orgName: created.org });
        if (resp.deleted) {
          console.log(`  Cleanup: Deleted org ${created.org}`);
        }
      } catch {
        console.log('  Cleanup: Failed to delete org');
      }
    }

    // Delete repo if created
    if (created.repo && created.userId) {
      try {
        const resp = await page.evaluate(async ({ token, userId, repoName }) => {
          const del = await fetch(`/api/v1/repos/${userId}/${repoName}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${token}` },
          });
          return { deleted: del.ok, status: del.status };
        }, { token, userId: created.userId, repoName: created.repo });
        if (resp.deleted) {
          console.log(`  Cleanup: Deleted repo ${created.repo}`);
        } else {
          console.log(`  Cleanup: Repo delete returned ${resp.status}`);
        }
      } catch {
        console.log('  Cleanup: Failed to delete repo');
      }
    }
  } finally {
    await page.close();
  }
}

// === MAIN ===

async function main() {
  console.log(`\n${'='.repeat(60)}`);
  console.log(`  CivitForge GUI Full Traverse`);
  console.log(`  Target: ${BASE_URL}`);
  console.log(`  Mode:   ${HEADED ? 'headed' : 'headless'}${DEBUG ? ' (debug)' : ''}`);
  console.log(`  Time:   ${results.startTime}`);
  console.log(`${'='.repeat(60)}\n`);

  const browser = await chromium.launch({
    headless: !HEADED,
    args: DEBUG ? ['--auto-open-devtools-for-tabs'] : [],
  });

  const totalStart = Date.now();

  try {
    await testRegister(browser);
    await testLogin(browser);
    await testHomePage(browser);
    await testReposPage(browser);
    await testNewRepo(browser);
    await testRepoDetail(browser);
    await testIssues(browser);
    await testWiki(browser);
    await testCodeBrowser(browser);
    await testPipelines(browser);
    await testReleases(browser);
    await testBoards(browser);
    await testGraph(browser);
    await testExplore(browser);
    await testSearch(browser);
    await testSettings(browser);
    await testActivity(browser);
    await testOrgs(browser);
    await testAdmin(browser);
    await testSidebarLocaleSwitcher(browser);
    await testBranchProtection(browser);
    await testTeamManagement(browser);
    await testNavigation(browser);
    await testNotFound(browser);
  } catch (e) {
    console.error(`\n  Fatal traverse error: ${e.message}`);
  }

  await cleanup(browser);

  const totalDuration = Date.now() - totalStart;

  console.log(`\n${'='.repeat(60)}`);
  console.log(`  Traversal Results`);
  console.log(`${'='.repeat(60)}\n`);

  for (const page of results.pages) {
    const status = page.status === 'passed' ? 'PASS' : 'FAIL';
    const errCount = page.errors.length;
    const extra = errCount > 0 ? `, ${errCount} errors` : '';
    console.log(`  [${status}] ${page.name} (${page.loadTimeMs != null ? `${page.loadTimeMs}ms` : 'N/A'}, ${page.actionsRun} actions${extra})`);
    for (const err of page.errors) {
      console.log(`    ERROR [${err.action}]: ${err.error}`);
    }
  }

  const passed = results.pages.filter(p => p.status === 'passed').length;
  const failed = results.pages.filter(p => p.status === 'failed').length;
  const allErrors = results.pages.reduce((sum, p) => sum + p.errors.length, 0);

  console.log(`\n${'='.repeat(60)}`);
  console.log(`  Summary`);
  console.log(`${'='.repeat(60)}`);
  console.log(`  Pages:     ${results.pages.length} total, ${passed} passed, ${failed} failed`);
  console.log(`  Actions:   ${results.totalActions}`);
  console.log(`  Errors:    ${allErrors}`);
  console.log(`  Duration:  ${(totalDuration / 1000).toFixed(1)}s`);
  console.log(`  Created:   repo=${created.repo || 'none'}, org=${created.org || 'none'}`);
  console.log(`  Auth:      token=${created.token ? 'yes' : 'no'}, userId=${created.userId || 'none'}`);
  console.log(`  Screenshots: ${capture.screenshots.length} saved to ${SCREENSHOTS_DIR}`);

  const reportFile = join(REPORTS_DIR, `traverse-${Date.now()}.json`);
  const report = {
    ...results,
    endTime: new Date().toISOString(),
    durationMs: totalDuration,
    created,
    capture: capture.getReport(),
    config: { baseUrl: BASE_URL, headed: HEADED, debug: DEBUG },
  };
  writeFileSync(reportFile, JSON.stringify(report, null, 2));
  console.log(`  Report:    ${reportFile}`);

  await capture.saveReport(`debug-${Date.now()}.json`);

  console.log();

  await browser.close();
  process.exit(allErrors > 0 ? 1 : 0);
}

main().catch((e) => {
  console.error('Fatal error:', e);
  process.exit(2);
});
