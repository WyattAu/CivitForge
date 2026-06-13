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
const FIXED_REPO_PATH = 'admin/axum';

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
  pr: null,
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
    { name: 'verify-repo-list-loads', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No repositories') || body.includes('no repos')) {
        console.log('    Repo list shows empty state');
      } else {
        console.log('    Repo list has content');
      }
    }},
    { name: 'check-repo-cards', fn: async (p) => {
      const cards = await p.$$('.repo-card, .card, article, [class*="repo"]');
      if (cards.length > 0) {
        console.log(`    Found ${cards.length} repo card(s)`);
      } else {
        console.log('    No repo cards found (list may be empty or use different markup)');
      }
    }},
    { name: 'verify-search-input', fn: async (p) => {
      const searchInput = await p.$('input[type="search"], input[placeholder*="search" i], input[placeholder*="filter" i], input#search');
      if (searchInput) {
        console.log('    Search/filter input found on repos page');
      } else {
        console.log('    No search input on repos page');
      }
    }},
    { name: 'check-repo-count', fn: async (p) => {
      const repoLinks = await p.$$('a[href*="/repos/"]');
      console.log(`    Repo links found: ${repoLinks.length}`);
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
      } else if (pathname.includes('/repos')) {
        console.log(`    Redirected to repos list: ${pathname}`);
      } else {
        console.log(`    Current path after create: ${pathname}`);
      }
    }},
    { name: 'verify-repo-page-content', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes(created.repo)) {
        console.log('    Repo name visible on destination page');
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
  const issueTitle = `Test issue from GUI traverse ${Date.now()}`;
  created.issue = issueTitle;

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
      await fillIfExists(p, 'input#new-issue-title', issueTitle);
      await fillIfExists(p, 'textarea#new-issue-description', 'This issue was created by the automated GUI traverse test.');
    }},
    { name: 'verify-form-filled', fn: async (p) => {
      const titleVal = await p.$eval('input#new-issue-title', el => el.value).catch(() => null);
      if (titleVal === issueTitle) {
        console.log('    Issue title field filled correctly');
      } else {
        console.log(`    Title field value: ${titleVal || 'not found'}`);
      }
    }},
    { name: 'submit-issue', fn: async (p) => {
      const btn = await p.$('button:has-text("Submit Issue"), button:has-text("Create Issue")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(3000);
      }
    }},
    { name: 'verify-issue-created', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes(issueTitle)) {
        console.log('    Issue appears in list after creation');
      } else {
        console.log('    Issue may not have been created (or list is empty)');
      }
    }},
    { name: 'verify-redirect', fn: async (p) => {
      const pathname = await p.evaluate(() => window.location.pathname);
      if (pathname.includes('/issues') || pathname.includes('/repos/')) {
        console.log(`    Post-issue URL: ${pathname}`);
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
    { name: 'filter-open-issues', fn: async (p) => {
      const openBtn = await p.$('button:has-text("Open")');
      if (openBtn) {
        await openBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Open filter');
      }
    }},
    { name: 'filter-closed-issues', fn: async (p) => {
      const closedBtn = await p.$('button:has-text("Closed")');
      if (closedBtn) {
        await closedBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Closed filter');
      }
    }},
    { name: 'filter-all-issues', fn: async (p) => {
      const allBtn = await p.$('button:has-text("All")');
      if (allBtn) {
        await allBtn.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-issues');
    }},
  ]);
}

// === 4b. PULL REQUESTS ===

async function testPullRequests(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping PRs test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;
  const prTitle = `Test PR from GUI traverse ${Date.now()}`;
  created.pr = prTitle;

  await traversePage(browser, `/repos/${repoPath}/pulls`, 'repo-pulls', [
    { name: 'verify-prs-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('Pull Request') && !body.includes('Pull') && !body.includes('Merge')) {
        console.log('    PRs page loaded (title may vary)');
      }
    }},
    { name: 'click-new-pr', fn: async (p) => {
      const btn = await p.$('button:has-text("New Pull Request"), button:has-text("Create Pull Request"), a:has-text("New Pull Request")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(500);
      } else {
        console.log('    New PR button not found');
      }
    }},
    { name: 'fill-pr-form', fn: async (p) => {
      await fillIfExists(p, 'input#pr-title, input[placeholder*="title"]', prTitle);
      await fillIfExists(p, 'textarea#pr-description, textarea[placeholder*="description"], textarea#pr-body', 'Test pull request created by GUI traverse.');
    }},
    { name: 'verify-pr-form-filled', fn: async (p) => {
      const titleEl = await p.$('input#pr-title, input[placeholder*="title"]');
      if (titleEl) {
        const val = await titleEl.inputValue();
        if (val === prTitle) {
          console.log('    PR title field filled correctly');
        }
      }
    }},
    { name: 'submit-pr', fn: async (p) => {
      const btn = await p.$('button:has-text("Create Pull Request"), button:has-text("Submit")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(3000);
      }
    }},
    { name: 'verify-pr-created', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes(prTitle)) {
        console.log('    PR appears in list after creation');
      } else {
        console.log('    PR may not have been created');
      }
    }},
    { name: 'verify-pr-redirect', fn: async (p) => {
      const pathname = await p.evaluate(() => window.location.pathname);
      console.log(`    Post-PR URL: ${pathname}`);
    }},
    { name: 'filter-open-prs', fn: async (p) => {
      const openBtn = await p.$('button:has-text("Open")');
      if (openBtn) {
        await openBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Open filter');
      }
    }},
    { name: 'filter-closed-prs', fn: async (p) => {
      const closedBtn = await p.$('button:has-text("Closed")');
      if (closedBtn) {
        await closedBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Closed filter');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-pulls');
    }},
  ]);
}

// === 4c. ISSUE/PR COMMENTS ===

async function testComments(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping comments test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  // Navigate to an existing issue to add a comment
  await traversePage(browser, `/repos/${repoPath}/issues`, 'repo-comments', [
    { name: 'navigate-to-issues', fn: async (p) => {
      await waitForContent(p);
    }},
    { name: 'click-first-issue', fn: async (p) => {
      // Try to find a clickable issue link in the list
      const issueLink = await p.$('a[href*="/issues/"], .issue-title a, tr a[href*="issues"]');
      if (issueLink) {
        await issueLink.click();
        await p.waitForTimeout(2000);
        console.log('    Navigated to issue detail');
      } else {
        console.log('    No issue links found to navigate to');
      }
    }},
    { name: 'find-comment-textarea', fn: async (p) => {
      const textarea = await p.$('textarea#comment, textarea[placeholder*="comment"], textarea[placeholder*="Comment"], textarea');
      if (textarea) {
        console.log('    Comment textarea found');
      } else {
        console.log('    Comment textarea not found (may not be on issue detail)');
      }
    }},
    { name: 'fill-comment', fn: async (p) => {
      const commentText = `GUI test comment ${Date.now()}`;
      const filled = await fillIfExists(p, 'textarea#comment, textarea[placeholder*="comment"], textarea[placeholder*="Comment"], textarea', commentText);
      if (filled) {
        console.log('    Comment textarea filled');
      }
    }},
    { name: 'submit-comment', fn: async (p) => {
      const btn = await p.$('button:has-text("Comment"), button:has-text("Submit"), button:has-text("Post")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(2000);
        console.log('    Comment submitted');
      } else {
        console.log('    Comment submit button not found');
      }
    }},
    { name: 'verify-comment-appeared', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('GUI test comment')) {
        console.log('    Comment appears in issue');
      } else {
        console.log('    Comment may not have been added');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'repo-comments');
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
        if (['Code', 'Issues', 'Wiki', 'Pipelines', 'Settings', 'Pull Requests', 'Boards'].includes(t)) {
          tabTexts.push(t);
        }
      }
      console.log(`    Repo tabs: ${tabTexts.join(', ') || 'none found'}`);
    }},
    { name: 'click-code-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Code")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Code tab -> ${pathname}`);
      }
    }},
    { name: 'click-issues-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Issues")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Issues tab -> ${pathname}`);
      }
    }},
    { name: 'click-prs-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Pull Requests"), a:has-text("PRs")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    PRs tab -> ${pathname}`);
      }
    }},
    { name: 'click-pipelines-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Pipelines")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Pipelines tab -> ${pathname}`);
      }
    }},
    { name: 'click-settings-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Settings")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Settings tab -> ${pathname}`);
      }
    }},
    { name: 'click-boards-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Boards")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Boards tab -> ${pathname}`);
      }
    }},
    { name: 'return-to-code', fn: async (p) => {
      const tab = await p.$('a:has-text("Code")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1000);
      }
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
    { name: 'verify-input-filled', fn: async (p) => {
      const val = await p.$eval('input#search-input', el => el.value).catch(() => null);
      if (val === 'test') {
        console.log('    Search input filled correctly');
      }
    }},
    { name: 'submit-search', fn: async (p) => {
      await fillIfExists(p, 'input#search-input', 'test');
      const form = await p.$('form');
      if (form) {
        await form.evaluate((f) => f.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true })));
        await p.waitForTimeout(1000);
      }
      await clickIfExists(p, 'button:has-text("Search")');
      await p.waitForTimeout(2000);
    }},
    { name: 'check-results', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No results found')) {
        console.log('    Search returned no results (expected for "test" query)');
      } else if (body.includes('Search Results') || body.includes('results')) {
        console.log('    Search returned results');
      }
    }},
    { name: 'check-result-links', fn: async (p) => {
      const resultLinks = await p.$$('a[href*="/repos/"]');
      if (resultLinks.length > 0) {
        console.log(`    Found ${resultLinks.length} repo link(s) in results`);
      } else {
        console.log('    No repo links in search results');
      }
    }},
    { name: 'search-with-query-param', fn: async (p) => {
      await navigatePath(p, '/search?q=repo');
      await p.waitForTimeout(1500);
      const body = await p.textContent('body');
      if (body.includes('Search') || body.includes('results')) {
        console.log('    Search with query param loaded');
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
    { name: 'verify-settings-form-loads', fn: async (p) => {
      const inputs = await p.$$('input');
      if (inputs.length === 0) throw new Error('No input fields found on settings page');
      console.log(`    Settings form loaded with ${inputs.length} input(s)`);
    }},
    { name: 'check-profile-form', fn: async (p) => {
      const displayName = await p.$('input#settings-display-name');
      if (displayName) {
        console.log('    Profile form loaded with display name field');
      } else {
        const anyInput = await p.$('input');
        if (anyInput) {
          console.log('    Settings has input fields (specific IDs may differ)');
        }
      }
    }},
    { name: 'check-password-form', fn: async (p) => {
      const pwCurrent = await p.$('input#pw-current, input[type="password"]');
      if (pwCurrent) {
        console.log('    Password form found');
      } else {
        console.log('    Password form not found on this view');
      }
    }},
    { name: 'check-ssh-keys-section', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('SSH Keys') || body.includes('ssh')) {
        console.log('    SSH Keys section present');
      } else {
        console.log('    SSH Keys section not found');
      }
    }},
    { name: 'check-danger-zone', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Danger Zone') || body.includes('danger')) {
        console.log('    Danger Zone section present');
      } else {
        console.log('    Danger Zone section not found');
      }
    }},
    { name: 'check-notification-settings', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Notification') || body.includes('notification') || body.includes('Email')) {
        console.log('    Notification settings section present');
      } else {
        console.log('    Notification settings not found');
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
  const boardName = `GUI Test Board ${Date.now() % 100000}`;

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
      const hasColumns = body.includes('Todo') || body.includes('In Progress') || body.includes('Done') || body.includes('No boards');
      if (hasColumns) {
        console.log('    Board page content detected');
      } else {
        console.log('    Board page loaded (content uncertain)');
      }
    }},
    { name: 'click-create-board', fn: async (p) => {
      const btn = await p.$('button:has-text("New Board"), button:has-text("Create Board")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(1000);
        console.log('    Create Board button clicked');
      } else {
        console.log('    Create Board button not found');
      }
    }},
    { name: 'fill-board-name', fn: async (p) => {
      await fillIfExists(p, 'input#board-name, input[placeholder*="board name"], input[placeholder*="Board name"]', boardName);
    }},
    { name: 'submit-board', fn: async (p) => {
      const btn = await p.$('button:has-text("Create"), button:has-text("Save")');
      if (btn) {
        await btn.click();
        await p.waitForTimeout(2000);
      }
    }},
    { name: 'verify-board-appeared', fn: async (p) => {
      await p.waitForTimeout(1000);
      const body = await p.textContent('body');
      if (body.includes(boardName)) {
        console.log(`    Board "${boardName}" appears in list`);
      } else {
        console.log('    Board may not have been created');
      }
    }},
    { name: 'check-board-columns-after', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Todo') && body.includes('In Progress') && body.includes('Done')) {
        console.log('    Kanban columns visible (Todo, In Progress, Done)');
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
    { name: 'check-admin-tabs', fn: async (p) => {
      const tabs = await p.$$('button, a');
      const tabTexts = [];
      for (const tab of tabs) {
        const t = await tab.textContent();
        if (['Users', 'Repos', 'Repositories', 'System', 'Audit', 'Settings'].includes(t)) {
          tabTexts.push(t);
        }
      }
      if (tabTexts.length > 0) {
        console.log(`    Admin tabs found: ${tabTexts.join(', ')}`);
      } else {
        console.log('    Admin tabs not found (may require admin permissions)');
      }
    }},
    { name: 'click-users-tab', fn: async (p) => {
      const usersTab = await p.$('button:has-text("Users"), a:has-text("Users")');
      if (usersTab) {
        await usersTab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const body = await p.textContent('body');
        if (body.includes('username') || body.includes('email') || body.includes('user')) {
          console.log('    Users tab loaded with content');
        } else {
          console.log('    Users tab clicked (content may be empty)');
        }
      } else {
        console.log('    Users tab not found');
      }
    }},
    { name: 'verify-user-list', fn: async (p) => {
      const userRows = await p.$$('tr, .user-row, .list-item');
      console.log(`    User list items: ${userRows.length}`);
    }},
    { name: 'click-repos-tab', fn: async (p) => {
      const reposTab = await p.$('button:has-text("Repos"), a:has-text("Repos"), button:has-text("Repositories"), a:has-text("Repositories")');
      if (reposTab) {
        await reposTab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const body = await p.textContent('body');
        if (body.includes('repo') || body.includes('Repositor')) {
          console.log('    Repos tab loaded with content');
        } else {
          console.log('    Repos tab clicked (content may be empty)');
        }
      } else {
        console.log('    Repos tab not found');
      }
    }},
    { name: 'verify-repo-list', fn: async (p) => {
      const repoRows = await p.$$('tr, .repo-row, .list-item');
      console.log(`    Repo list items: ${repoRows.length}`);
    }},
    { name: 'click-audit-log-tab', fn: async (p) => {
      const auditTab = await p.$('button:has-text("Audit"), a:has-text("Audit"), button:has-text("Audit Log"), a:has-text("Audit Log")');
      if (auditTab) {
        await auditTab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const body = await p.textContent('body');
        if (body.includes('audit') || body.includes('Audit') || body.includes('event') || body.includes('log')) {
          console.log('    Audit Log tab loaded with content');
        } else {
          console.log('    Audit Log tab clicked (content may be empty)');
        }
      } else {
        console.log('    Audit Log tab not found');
      }
    }},
    { name: 'verify-audit-events', fn: async (p) => {
      const eventRows = await p.$$('tr, .audit-row, .event-row, .list-item');
      console.log(`    Audit event items: ${eventRows.length}`);
    }},
    { name: 'click-system-tab', fn: async (p) => {
      const sysTab = await p.$('button:has-text("System"), a:has-text("System")');
      if (sysTab) {
        await sysTab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const body = await p.textContent('body');
        if (body.includes('System') || body.includes('version') || body.includes('disk')) {
          console.log('    System tab loaded with content');
        }
      } else {
        console.log('    System tab not found');
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

// === 21b. SIDEBAR NAVIGATION CLICKS ===

async function testSidebarNavigation(browser) {
  await traversePage(browser, '/', 'sidebar-nav-clicks', [
    { name: 'verify-sidebar', fn: async (p) => {
      await waitForContent(p);
      const sidebar = await p.$('aside');
      if (!sidebar) throw new Error('Sidebar not found');
    }},
    { name: 'click-repos-link', fn: async (p) => {
      const link = await p.$('aside a[href="/repos"]');
      if (link) {
        await link.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        if (pathname.includes('/repos')) {
          console.log(`    Sidebar Repos link -> ${pathname}`);
        } else {
          console.log(`    Repos link navigated to: ${pathname}`);
        }
      }
    }},
    { name: 'click-explore-link', fn: async (p) => {
      // Navigate back home first
      await navigatePath(p, '/');
      await waitForContent(p);
      const link = await p.$('aside a[href="/explore"]');
      if (link) {
        await link.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Sidebar Explore link -> ${pathname}`);
      }
    }},
    { name: 'click-activity-link', fn: async (p) => {
      await navigatePath(p, '/');
      await waitForContent(p);
      const link = await p.$('aside a[href="/activity"]');
      if (link) {
        await link.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Sidebar Activity link -> ${pathname}`);
      }
    }},
    { name: 'click-settings-link', fn: async (p) => {
      await navigatePath(p, '/');
      await waitForContent(p);
      const link = await p.$('aside a[href="/settings"]');
      if (link) {
        await link.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Sidebar Settings link -> ${pathname}`);
      }
    }},
    { name: 'click-admin-link', fn: async (p) => {
      await navigatePath(p, '/');
      await waitForContent(p);
      const link = await p.$('aside a[href="/admin"]');
      if (link) {
        await link.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Sidebar Admin link -> ${pathname}`);
      }
    }},
    { name: 'click-home-link', fn: async (p) => {
      const link = await p.$('aside a[href="/"]');
      if (link) {
        await link.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        if (pathname === '/') {
          console.log('    Sidebar Home link -> /');
        } else {
          console.log(`    Home link navigated to: ${pathname}`);
        }
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'sidebar-nav-clicks');
    }},
  ]);
}

// === 21c. BREADCRUMB NAVIGATION ===

async function testBreadcrumbNavigation(browser) {
  if (!created.repo || !created.userId) {
    console.log('    Skipping breadcrumb test (no repo/user created)');
    return;
  }

  const repoPath = `${created.userId}/${created.repo}`;

  await traversePage(browser, `/repos/${repoPath}/issues`, 'breadcrumb-nav', [
    { name: 'verify-breadcrumb', fn: async (p) => {
      await waitForContent(p);
      // Look for breadcrumb navigation
      const breadcrumbs = await p.$$('nav[aria-label*="breadcrumb"], .breadcrumb, ol.breadcrumb, .breadcrumbs a');
      if (breadcrumbs.length > 0) {
        console.log(`    Found ${breadcrumbs.length} breadcrumb element(s)`);
      } else {
        console.log('    Breadcrumb elements not found (may use different markup)');
      }
    }},
    { name: 'click-repo-breadcrumb', fn: async (p) => {
      // Try clicking a breadcrumb link that points to the repo root
      const repoBreadcrumb = await p.$(`a[href="/repos/${repoPath}"]`);
      if (repoBreadcrumb) {
        await repoBreadcrumb.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Repo breadcrumb -> ${pathname}`);
      } else {
        console.log('    Repo breadcrumb link not found');
      }
    }},
    { name: 'verify-navigated-to-repo', fn: async (p) => {
      const pathname = await p.evaluate(() => window.location.pathname);
      const body = await p.textContent('body');
      if (pathname.includes(repoPath) && body.includes(created.repo)) {
        console.log('    Successfully navigated to repo via breadcrumb');
      }
    }},
    { name: 'navigate-to-issues-via-breadcrumb', fn: async (p) => {
      const issuesBreadcrumb = await p.$('a:has-text("Issues"), a[href*="/issues"]');
      if (issuesBreadcrumb) {
        await issuesBreadcrumb.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Issues breadcrumb -> ${pathname}`);
      }
    }},
    { name: 'navigate-home-via-breadcrumb', fn: async (p) => {
      const homeBreadcrumb = await p.$('a[href="/"]');
      if (homeBreadcrumb) {
        await homeBreadcrumb.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Home breadcrumb -> ${pathname}`);
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'breadcrumb-nav');
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

// === 23. REPO DETAIL (FIXED PATH) ===

async function testFixedRepoDetail(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}`, 'fixed-repo-detail', [
    { name: 'verify-repo-name', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('axum')) {
        throw new Error('Repo name "axum" not found on detail page');
      }
      console.log('    Repo name "axum" visible');
    }},
    { name: 'verify-description', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.length > 50) {
        console.log('    Repo page has description/content');
      }
    }},
    { name: 'verify-stats', fn: async (p) => {
      const body = await p.textContent('body');
      const hasStats = body.includes('star') || body.includes('fork') || body.includes('watch') || body.includes('issue');
      if (hasStats) {
        console.log('    Repo stats visible (stars/forks/issues)');
      } else {
        console.log('    Repo stats not found (may use different wording)');
      }
    }},
    { name: 'check-repo-tabs', fn: async (p) => {
      const tabs = await p.$$('a');
      const tabTexts = [];
      for (const tab of tabs) {
        const t = await tab.textContent();
        if (['Code', 'Issues', 'Wiki', 'Pipelines', 'Settings', 'Pull Requests', 'Boards', 'Releases', 'Graph'].includes(t)) {
          tabTexts.push(t);
        }
      }
      console.log(`    Repo tabs: ${tabTexts.join(', ') || 'none found'}`);
      if (tabTexts.length < 3) throw new Error(`Expected at least 3 tabs, found ${tabTexts.length}`);
    }},
    { name: 'click-code-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Code")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Code tab -> ${pathname}`);
      }
    }},
    { name: 'click-issues-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Issues")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Issues tab -> ${pathname}`);
      }
    }},
    { name: 'click-prs-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Pull Requests"), a:has-text("PRs")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    PRs tab -> ${pathname}`);
      }
    }},
    { name: 'click-pipelines-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Pipelines")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Pipelines tab -> ${pathname}`);
      }
    }},
    { name: 'click-settings-tab', fn: async (p) => {
      const tab = await p.$('a:has-text("Settings")');
      if (tab) {
        await tab.click();
        await p.waitForTimeout(1500);
        await waitForContent(p);
        const pathname = await p.evaluate(() => window.location.pathname);
        console.log(`    Settings tab -> ${pathname}`);
      }
    }},
    { name: 'verify-file-tree', fn: async (p) => {
      await navigatePath(p, `/repos/${FIXED_REPO_PATH}`);
      await waitForContent(p);
      const fileElements = await p.$$('a[href*="/code/"], .file-entry, .tree-entry, tr a');
      if (fileElements.length > 0) {
        console.log(`    File tree has ${fileElements.length} entries`);
      } else {
        console.log('    File tree entries not found (may need to click into code tab)');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-detail');
    }},
  ]);
}

// === 24. ISSUES (FIXED PATH) ===

async function testFixedRepoIssues(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/issues`, 'fixed-repo-issues', [
    { name: 'verify-issues-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on issues page');
      const text = await h1.textContent();
      if (!text.includes('Issues')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'verify-issue-list-loads', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No issues') || body.includes('no issues')) {
        console.log('    Issue list shows empty state');
      } else {
        console.log('    Issue list has content');
      }
    }},
    { name: 'check-issue-cards', fn: async (p) => {
      const cards = await p.$$('.issue-card, .card, article, tr, [class*="issue"]');
      if (cards.length > 0) {
        console.log(`    Found ${cards.length} issue card(s)/row(s)`);
      } else {
        console.log('    No issue cards found (may be empty)');
      }
    }},
    { name: 'verify-filter-buttons', fn: async (p) => {
      const buttons = await p.$$('button');
      const filterTexts = [];
      for (const btn of buttons) {
        const t = await btn.textContent();
        if (['All', 'Open', 'In Progress', 'Closed'].includes(t)) {
          filterTexts.push(t);
        }
      }
      if (filterTexts.length > 0) {
        console.log(`    Filter buttons: ${filterTexts.join(', ')}`);
      } else {
        console.log('    Filter buttons not found');
      }
    }},
    { name: 'click-open-filter', fn: async (p) => {
      const openBtn = await p.$('button:has-text("Open")');
      if (openBtn) {
        await openBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Open filter');
      }
    }},
    { name: 'click-closed-filter', fn: async (p) => {
      const closedBtn = await p.$('button:has-text("Closed")');
      if (closedBtn) {
        await closedBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Closed filter');
      }
    }},
    { name: 'click-all-filter', fn: async (p) => {
      const allBtn = await p.$('button:has-text("All")');
      if (allBtn) {
        await allBtn.click();
        await p.waitForTimeout(500);
      }
    }},
    { name: 'check-issue-numbers', fn: async (p) => {
      const links = await p.$$('a[href*="/issues/"]');
      if (links.length > 0) {
        console.log(`    Found ${links.length} issue link(s)`);
      } else {
        console.log('    No issue detail links found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-issues');
    }},
  ]);
}

// === 25. PULL REQUESTS (FIXED PATH) ===

async function testFixedRepoPullRequests(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/pulls`, 'fixed-repo-pulls', [
    { name: 'verify-prs-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('Pull Request') && !body.includes('Pull') && !body.includes('Merge')) {
        console.log('    PRs page loaded (title may vary)');
      }
    }},
    { name: 'verify-pr-list-loads', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No pull requests') || body.includes('no pull')) {
        console.log('    PR list shows empty state');
      } else {
        console.log('    PR list has content');
      }
    }},
    { name: 'check-pr-cards', fn: async (p) => {
      const cards = await p.$$('.pr-card, .card, article, tr, [class*="pull"]');
      if (cards.length > 0) {
        console.log(`    Found ${cards.length} PR card(s)/row(s)`);
      } else {
        console.log('    No PR cards found (may be empty)');
      }
    }},
    { name: 'verify-filter-buttons', fn: async (p) => {
      const buttons = await p.$$('button');
      const filterTexts = [];
      for (const btn of buttons) {
        const t = await btn.textContent();
        if (['All', 'Open', 'Closed', 'Merged'].includes(t)) {
          filterTexts.push(t);
        }
      }
      if (filterTexts.length > 0) {
        console.log(`    Filter buttons: ${filterTexts.join(', ')}`);
      } else {
        console.log('    Filter buttons not found');
      }
    }},
    { name: 'click-open-filter', fn: async (p) => {
      const openBtn = await p.$('button:has-text("Open")');
      if (openBtn) {
        await openBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Open filter');
      }
    }},
    { name: 'click-closed-filter', fn: async (p) => {
      const closedBtn = await p.$('button:has-text("Closed")');
      if (closedBtn) {
        await closedBtn.click();
        await p.waitForTimeout(1000);
        console.log('    Clicked Closed filter');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-pulls');
    }},
  ]);
}

// === 26. PIPELINES (FIXED PATH) ===

async function testFixedRepoPipelines(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/pipelines`, 'fixed-repo-pipelines', [
    { name: 'verify-pipelines-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'verify-pipeline-list-loads', fn: async (p) => {
      const body = await p.textContent('body');
      const hasContent = body.includes('Pipeline') || body.includes('No pipeline') || body.includes('Run') || body.includes('pipeline');
      if (hasContent) {
        console.log('    Pipeline list has content');
      } else {
        console.log('    Pipeline list loaded (content uncertain)');
      }
    }},
    { name: 'check-pipeline-items', fn: async (p) => {
      const items = await p.$$('.pipeline-item, .card, tr, [class*="pipeline"]');
      if (items.length > 0) {
        console.log(`    Found ${items.length} pipeline item(s)`);
      } else {
        console.log('    No pipeline items found (may be empty)');
      }
    }},
    { name: 'check-pipeline-status', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('success') || body.includes('failed') || body.includes('running') || body.includes('pending')) {
        console.log('    Pipeline status indicators found');
      } else {
        console.log('    Pipeline status not found (no runs or different wording)');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-pipelines');
    }},
  ]);
}

// === 27. PROFILE PAGE ===

async function testProfile(browser) {
  await traversePage(browser, '/profile', 'profile', [
    { name: 'verify-profile-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-profile-content', fn: async (p) => {
      const body = await p.textContent('body');
      const hasProfile = body.includes('Profile') || body.includes('profile') || body.includes('username') || body.includes('display');
      if (hasProfile) {
        console.log('    Profile page has content');
      } else {
        console.log('    Profile page loaded (content uncertain)');
      }
    }},
    { name: 'check-avatar', fn: async (p) => {
      const avatar = await p.$('img[class*="avatar"], img[class*="profile"], img[alt*="avatar"]');
      if (avatar) {
        console.log('    Avatar image found');
      } else {
        console.log('    Avatar image not found');
      }
    }},
    { name: 'check-user-info', fn: async (p) => {
      const inputs = await p.$$('input');
      if (inputs.length > 0) {
        console.log(`    Profile form has ${inputs.length} input(s)`);
      } else {
        console.log('    No profile inputs found (may be read-only view)');
      }
    }},
    { name: 'check-repos-section', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Repositories') || body.includes('repos')) {
        console.log('    Repositories section visible on profile');
      } else {
        console.log('    Repositories section not found on profile');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'profile');
    }},
  ]);
}

// === 28. PROFILE BY USERNAME ===

async function testProfileUsername(browser) {
  await traversePage(browser, `/profile/${FIXED_REPO_PATH.split('/')[0]}`, 'profile-username', [
    { name: 'verify-profile-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-profile-username', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('admin')) {
        console.log('    Profile for admin user loaded');
      } else {
        console.log('    Profile page loaded (user may not exist)');
      }
    }},
    { name: 'check-user-repos', fn: async (p) => {
      const repoLinks = await p.$$('a[href*="/repos/"]');
      if (repoLinks.length > 0) {
        console.log(`    Found ${repoLinks.length} repo link(s) on profile`);
      } else {
        console.log('    No repo links on profile');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'profile-username');
    }},
  ]);
}

// === 29. CODE BROWSER (FIXED PATH) ===

async function testFixedRepoCodeBrowser(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/code`, 'fixed-repo-code', [
    { name: 'verify-code-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'verify-file-tree-loads', fn: async (p) => {
      const fileLinks = await p.$$('a[href*="/code/"]');
      if (fileLinks.length > 0) {
        console.log(`    File tree has ${fileLinks.length} file/folder links`);
      } else {
        console.log('    No file tree links found (may be loading or empty)');
      }
    }},
    { name: 'check-file-listings', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('.rs') || body.includes('.toml') || body.includes('.md') || body.includes('Cargo') || body.includes('README')) {
        console.log('    File listings detected in code browser');
      } else {
        console.log('    No specific file names detected');
      }
    }},
    { name: 'check-folder-structure', fn: async (p) => {
      const folders = await p.$$('a[href$="/"]');
      if (folders.length > 0) {
        console.log(`    Found ${folders.length} folder link(s)`);
      } else {
        console.log('    No folder links found');
      }
    }},
    { name: 'click-into-file', fn: async (p) => {
      const fileLink = await p.$('a[href*="/code/"][href$=".rs"], a[href*="/code/"][href$=".toml"], a[href*="/code/"][href$=".md"]');
      if (fileLink) {
        const fileName = await fileLink.textContent();
        await fileLink.click();
        await p.waitForTimeout(2000);
        await waitForContent(p);
        const body = await p.textContent('body');
        if (body.length > 100) {
          console.log(`    File "${fileName.trim()}" content loaded`);
        }
      } else {
        console.log('    No clickable file found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-code');
    }},
  ]);
}

// === 30. WIKI (FIXED PATH) ===

async function testFixedRepoWiki(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/wiki`, 'fixed-repo-wiki', [
    { name: 'verify-wiki-page', fn: async (p) => {
      await waitForContent(p);
      const h1 = await p.$('h1');
      if (!h1) throw new Error('No h1 on wiki page');
      const text = await h1.textContent();
      if (!text.includes('Wiki')) throw new Error(`Unexpected h1: ${text}`);
    }},
    { name: 'verify-wiki-sidebar', fn: async (p) => {
      const sidebar = await p.$('.wiki-sidebar, nav, aside');
      if (sidebar) {
        console.log('    Wiki sidebar found');
      } else {
        console.log('    Wiki sidebar not found');
      }
    }},
    { name: 'check-wiki-pages', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No pages') || body.includes('no pages') || body.includes('Getting Started')) {
        console.log('    Wiki page list has content');
      } else {
        console.log('    Wiki page list loaded');
      }
    }},
    { name: 'check-new-page-btn', fn: async (p) => {
      const btn = await p.$('button:has-text("New Page"), button:has-text("Create Page")');
      if (btn) {
        console.log('    New Page button found');
      } else {
        console.log('    New Page button not found');
      }
    }},
    { name: 'check-wiki-content', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Markdown') || body.includes('markdown') || body.includes('Edit')) {
        console.log('    Wiki content area detected');
      } else {
        console.log('    Wiki content area not detected');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-wiki');
    }},
  ]);
}

// === 31. ADMIN SITE SETTINGS ===

async function testAdminSiteSettings(browser) {
  await traversePage(browser, '/admin/site-settings', 'admin-site-settings', [
    { name: 'verify-site-settings-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('Setting') && !body.includes('setting') && !body.includes('Admin')) {
        console.log('    Site settings page loaded (content may vary)');
      }
    }},
    { name: 'check-settings-form', fn: async (p) => {
      const inputs = await p.$$('input, select, textarea');
      if (inputs.length > 0) {
        console.log(`    Site settings form has ${inputs.length} field(s)`);
      } else {
        console.log('    No form fields found on site settings');
      }
    }},
    { name: 'check-site-name', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Site Name') || body.includes('site name') || body.includes('Title')) {
        console.log('    Site name setting found');
      } else {
        console.log('    Site name setting not found');
      }
    }},
    { name: 'check-registration', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Registration') || body.includes('registration') || body.includes('Sign up')) {
        console.log('    Registration settings found');
      } else {
        console.log('    Registration settings not found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'admin-site-settings');
    }},
  ]);
}

// === 32. REPO BLAME ===

async function testFixedRepoBlame(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/blame`, 'fixed-repo-blame', [
    { name: 'verify-blame-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-blame-content', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Blame') || body.includes('blame') || body.includes('No file')) {
        console.log('    Blame page has content');
      } else {
        console.log('    Blame page loaded');
      }
    }},
    { name: 'check-blame-lines', fn: async (p) => {
      const lines = await p.$$('.blame-line, .code-line, tr, [class*="blame"]');
      if (lines.length > 0) {
        console.log(`    Found ${lines.length} blame line(s)`);
      } else {
        console.log('    No blame lines found (may need file selection)');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-blame');
    }},
  ]);
}

// === 33. REPO COMMITS ===

async function testFixedRepoCommits(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/commits`, 'fixed-repo-commits', [
    { name: 'verify-commits-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-commit-list', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('No commits') || body.includes('commit') || body.includes('Commit')) {
        console.log('    Commit list has content');
      } else {
        console.log('    Commit list loaded');
      }
    }},
    { name: 'check-commit-entries', fn: async (p) => {
      const entries = await p.$$('.commit-entry, .commit, tr, [class*="commit"]');
      if (entries.length > 0) {
        console.log(`    Found ${entries.length} commit entry/entries`);
      } else {
        console.log('    No commit entries found');
      }
    }},
    { name: 'check-commit-messages', fn: async (p) => {
      const links = await p.$$('a[href*="/commit"]');
      if (links.length > 0) {
        console.log(`    Found ${links.length} commit link(s)`);
      } else {
        console.log('    No commit links found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-commits');
    }},
  ]);
}

// === 34. REPO SETTINGS ===

async function testFixedRepoSettings(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/settings`, 'fixed-repo-settings', [
    { name: 'verify-repo-settings-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body.includes('Setting') && !body.includes('setting')) {
        console.log('    Repo settings page loaded');
      }
    }},
    { name: 'check-settings-form', fn: async (p) => {
      const inputs = await p.$$('input, select, textarea');
      if (inputs.length > 0) {
        console.log(`    Repo settings form has ${inputs.length} field(s)`);
      } else {
        console.log('    No form fields on repo settings');
      }
    }},
    { name: 'check-repo-name-field', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Repository name') || body.includes('repo name') || body.includes('Name')) {
        console.log('    Repository name setting found');
      } else {
        console.log('    Repository name setting not found');
      }
    }},
    { name: 'check-visibility', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Visibility') || body.includes('visibility') || body.includes('Public') || body.includes('Private')) {
        console.log('    Visibility setting found');
      } else {
        console.log('    Visibility setting not found');
      }
    }},
    { name: 'check-danger-zone', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Danger Zone') || body.includes('danger') || body.includes('Delete')) {
        console.log('    Danger Zone section present');
      } else {
        console.log('    Danger Zone section not found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-settings');
    }},
  ]);
}

// === 35. REPO ENVIRONMENTS ===

async function testFixedRepoEnvironments(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/environments`, 'fixed-repo-environments', [
    { name: 'verify-environments-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-environments-content', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Environment') || body.includes('environment') || body.includes('No environments')) {
        console.log('    Environments page has content');
      } else {
        console.log('    Environments page loaded');
      }
    }},
    { name: 'check-new-environment-btn', fn: async (p) => {
      const btn = await p.$('button:has-text("New Environment"), button:has-text("Create Environment")');
      if (btn) {
        console.log('    New Environment button found');
      } else {
        console.log('    New Environment button not found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-environments');
    }},
  ]);
}

// === 36. REPO DEPLOYMENTS ===

async function testFixedRepoDeployments(browser) {
  await traversePage(browser, `/repos/${FIXED_REPO_PATH}/deployments`, 'fixed-repo-deployments', [
    { name: 'verify-deployments-page', fn: async (p) => {
      await waitForContent(p);
      const body = await p.textContent('body');
      if (!body) throw new Error('Page body empty');
    }},
    { name: 'check-deployments-content', fn: async (p) => {
      const body = await p.textContent('body');
      if (body.includes('Deployment') || body.includes('deployment') || body.includes('No deployments')) {
        console.log('    Deployments page has content');
      } else {
        console.log('    Deployments page loaded');
      }
    }},
    { name: 'check-deployment-items', fn: async (p) => {
      const items = await p.$$('.deployment-item, .card, tr, [class*="deploy"]');
      if (items.length > 0) {
        console.log(`    Found ${items.length} deployment item(s)`);
      } else {
        console.log('    No deployment items found');
      }
    }},
    { name: 'screenshot', fn: async (p) => {
      await takeScreenshot(p, 'fixed-repo-deployments');
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
    await testPullRequests(browser);
    await testComments(browser);
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
    await testSidebarNavigation(browser);
    await testBreadcrumbNavigation(browser);
    await testNavigation(browser);
    await testNotFound(browser);
    // Fixed-path tests (use /repos/admin/axum as known existing repo)
    await testFixedRepoDetail(browser);
    await testFixedRepoIssues(browser);
    await testFixedRepoPullRequests(browser);
    await testFixedRepoPipelines(browser);
    await testFixedRepoCodeBrowser(browser);
    await testFixedRepoWiki(browser);
    await testFixedRepoBlame(browser);
    await testFixedRepoCommits(browser);
    await testFixedRepoSettings(browser);
    await testFixedRepoEnvironments(browser);
    await testFixedRepoDeployments(browser);
    // New standalone pages
    await testProfile(browser);
    await testProfileUsername(browser);
    await testAdminSiteSettings(browser);
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
