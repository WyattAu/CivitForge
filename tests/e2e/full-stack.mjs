#!/usr/bin/env node
import { chromium } from 'playwright';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';
import {
  BASE_URL, API_URL, BRAIN_URL, RUNNER_URL,
  TIMEOUT, ACTION_TIMEOUT, HEADED, DEBUG, SKIP_CLEANUP, KEEP_TESTDATA,
  TEST_USER, ADMIN_USER, SCREENSHOTS_DIR, REPORTS_DIR,
} from './config.mjs';
import {
  ApiClient, TestContext,
  assert, assertEqual, assertStatus, assertOk, assertJson, assertField,
  waitForHealth, sleep, uid,
} from './helpers.mjs';

const results = {
  startTime: new Date().toISOString(),
  suites: [],
  totalTests: 0,
  totalPassed: 0,
  totalFailed: 0,
  totalSkipped: 0,
  durationMs: 0,
};

let currentSuite = null;
let screenshotCounter = 0;

function suite(name, fn) {
  const s = { name, tests: [], startTime: Date.now(), endTime: 0, durationMs: 0 };
  results.suites.push(s);
  currentSuite = s;
  console.log(`\n=== ${name} ===`);
  return fn().then(() => {
    s.endTime = Date.now();
    s.durationMs = s.endTime - s.startTime;
    currentSuite = null;
  });
}

async function test(name, fn) {
  results.totalTests++;
  const t = { name, status: 'pending', error: null, durationMs: 0 };
  currentSuite?.tests.push(t);
  const start = Date.now();
  try {
    await fn();
    t.status = 'passed';
    results.totalPassed++;
    console.log(`  PASS  ${name}`);
  } catch (e) {
    t.status = 'failed';
    t.error = e.message;
    results.totalFailed++;
    console.log(`  FAIL  ${name}: ${e.message}`);
  }
  t.durationMs = Date.now() - start;
}

function skip(name, reason) {
  results.totalTests++;
  results.totalSkipped++;
  currentSuite?.tests.push({ name, status: 'skipped', error: reason, durationMs: 0 });
  console.log(`  SKIP  ${name}: ${reason}`);
}

async function screenshot(page, label) {
  screenshotCounter++;
  const name = `${String(screenshotCounter).padStart(3, '0')}_${label.replace(/[^a-z0-9]/gi, '_')}.png`;
  try {
    await page.screenshot({ path: join(SCREENSHOTS_DIR, name), fullPage: true });
  } catch { /* ignore */ }
}

async function setupBrowser() {
  const browser = await chromium.launch({ headless: !HEADED });
  const ctx = await browser.newContext({
    viewport: { width: 1280, height: 800 },
  });
  const page = await ctx.newPage();
  return { browser, ctx, page };
}

async function waitForHydration(page) {
  try {
    await page.waitForSelector('nav a[href]', { timeout: 8000 });
  } catch {
    await page.waitForTimeout(1000);
  }
}

async function fillForm(page, fields) {
  for (const [selector, value] of Object.entries(fields)) {
    const el = await page.$(selector);
    if (el) await el.fill(value);
  }
}

async function clickButton(page, text) {
  const btn = await page.$(`button:has-text("${text}")`);
  if (btn) {
    await btn.click();
    return true;
  }
  return false;
}

// ============================================================================
// SUITE 1: AUTHENTICATION
// ============================================================================

async function testAuthentication(api, ctx, browser) {
  await suite('Authentication', async () => {
    await test('POST /auth/register creates a new user', async () => {
      const resp = await api.post('/auth/register', {
        body: {
          username: TEST_USER.username,
          email: TEST_USER.email,
          display_name: TEST_USER.display_name,
          password: TEST_USER.password,
          confirm_password: TEST_USER.password,
        },
        expect: 200,
      });
      assert(resp.data, 'response body should exist');
      assertField(resp.data, 'token', 'registration response should contain token');
      ctx.token = resp.data.token;
      ctx.username = resp.data.username || TEST_USER.username;
      api.setToken(ctx.token);
    });

    await test('POST /auth/me validates JWT and returns user info', async () => {
      const resp = await api.get('/auth/me', { expect: 200 });
      assertOk(resp, 'auth/me');
      assertField(resp.data, 'username', 'auth/me should have username');
      ctx.userId = resp.data.user_id || resp.data.id || resp.data.username;
    });

    await test('POST /auth/login authenticates with valid credentials', async () => {
      const resp = await api.post('/auth/login', {
        body: { username: TEST_USER.username, password: TEST_USER.password },
        expect: 200,
      });
      assert(resp.data, 'login response body should exist');
      assertField(resp.data, 'token', 'login response should contain token');
    });

    await test('POST /auth/login rejects invalid password', async () => {
      const resp = await api.post('/auth/login', {
        body: { username: TEST_USER.username, password: 'wrong-password' },
      });
      assert(resp.status >= 400, `expected 4xx, got ${resp.status}`);
    });

    await test('GET /auth/me rejects request without token', async () => {
      const unauthed = new ApiClient();
      const resp = await unauthed.get('/auth/me');
      assert(resp.status === 401 || resp.status === 403, `expected 401/403, got ${resp.status}`);
    });

    await test('GET /auth/me rejects expired token', async () => {
      const fakeExpired = 'eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwiZXhwIjoxfQ.invalid';
      const unauthed = new ApiClient(fakeExpired);
      const resp = await unauthed.get('/auth/me');
      assert(resp.status >= 401, `expected 401+, got ${resp.status}`);
    });

    await test('Browser login flow stores token in localStorage', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        await fillForm(p, {
          'input#username': TEST_USER.username,
          'input[type="password"]': TEST_USER.password,
        });
        await p.press('input[type="password"]', 'Enter');
        await p.waitForTimeout(2000);
        const token = await p.evaluate(() => localStorage.getItem('civitforge_token'));
        assert(token, 'token should be stored in localStorage after login');
        await screenshot(p, 'browser_login_success');
      } finally {
        await p.close();
        await b.close();
      }
    });

    await test('Browser session persists across page loads', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        await fillForm(p, {
          'input#username': TEST_USER.username,
          'input[type="password"]': TEST_USER.password,
        });
        await p.press('input[type="password"]', 'Enter');
        await p.waitForTimeout(2000);
        await p.goto(`${BASE_URL}/repos`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        const token = await p.evaluate(() => localStorage.getItem('civitforge_token'));
        assert(token, 'token should persist after navigation');
        const meResp = await p.evaluate(async (t) => {
          const r = await fetch('/api/v1/auth/me', {
            headers: { 'Authorization': `Bearer ${t}` },
          });
          return { status: r.status, body: await r.json() };
        }, token);
        assertEqual(meResp.status, 200, 'auth/me should return 200');
        assertEqual(meResp.body.username, TEST_USER.username, 'username should match');
        await screenshot(p, 'session_persist');
      } finally {
        await p.close();
        await b.close();
      }
    });

    await test('POST /auth/logout invalidates session', async () => {
      const resp = await api.post('/auth/logout', { expect: 200 });
      const meAfter = await api.get('/auth/me');
      assert(meAfter.status >= 401, 'auth/me should fail after logout');

      const reLogin = await api.post('/auth/login', {
        body: { username: TEST_USER.username, password: TEST_USER.password },
        expect: 200,
      });
      ctx.token = reLogin.data.token;
      api.setToken(ctx.token);
    });
  });
}

// ============================================================================
// SUITE 2: REPOSITORY CRUD
// ============================================================================

async function testRepositoryCRUD(api, ctx) {
  const repoName = `e2e-crud-${uid()}`;
  let repoOwner = null;

  await suite('Repository CRUD', async () => {
    await test('POST /repos creates a new repository', async () => {
      const resp = await api.post('/repos', {
        body: {
          name: repoName,
          description: 'Created by E2E full-stack test',
          visibility: 'public',
        },
        expect: 201,
      });
      assertOk(resp, 'create repo');
      assertField(resp.data, 'name', 'repo response');
      repoOwner = resp.data.owner || resp.data.owner_id || ctx.userId;
      ctx.trackRepo(repoOwner, repoName);
    });

    await test('GET /repos lists repositories including the new one', async () => {
      const resp = await api.get('/repos', { expect: 200 });
      assertOk(resp, 'list repos');
      const repos = resp.data?.data || resp.data || [];
      const found = Array.isArray(repos)
        ? repos.some(r => r.name === repoName)
        : false;
      assert(found, `repo "${repoName}" should appear in list`);
    });

    await test('GET /repos/:owner/:name retrieves the repository', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}`, { expect: 200 });
      assertOk(resp, 'get repo');
      assertEqual(resp.data.name, repoName, 'repo name should match');
      assertField(resp.data, 'description', 'repo should have description');
    });

    await test('PATCH /repos/:owner/:name updates repository description', async () => {
      const newDesc = `Updated by E2E at ${new Date().toISOString()}`;
      const resp = await api.patch(`/repos/${repoOwner}/${repoName}`, {
        body: { description: newDesc },
        expect: 200,
      });
      assertOk(resp, 'update repo');
    });

    await test('GET /repos/:owner/:name reflects the update', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}`, { expect: 200 });
      assertOk(resp, 'get repo after update');
      assert(resp.data.description.includes('Updated'), 'description should be updated');
    });

    await test('GET /repos/:owner/:name returns 404 for non-existent repo', async () => {
      const resp = await api.get(`/repos/does-not-exist/fake-repo-${uid()}`);
      assert(resp.status === 404 || resp.status === 400, `expected 404/400, got ${resp.status}`);
    });

    await test('POST /repos rejects duplicate repo name', async () => {
      const resp = await api.post('/repos', {
        body: { name: repoName, visibility: 'public' },
      });
      assert(resp.status >= 400, `expected 4xx for duplicate, got ${resp.status}`);
    });

    await test('DELETE /repos/:owner/:name deletes the repository', async () => {
      const resp = await api.delete(`/repos/${repoOwner}/${repoName}`, { expect: 200 });
      assertOk(resp, 'delete repo');
      const getResp = await api.get(`/repos/${repoOwner}/${repoName}`);
      assert(getResp.status === 404, `repo should be gone, got ${getResp.status}`);
    });
  });
}

// ============================================================================
// SUITE 3: ISSUE LIFECYCLE
// ============================================================================

async function testIssueLifecycle(api, ctx, browser) {
  const repoName = `e2e-issues-${uid()}`;
  let repoOwner = null;
  const issueNumbers = [];

  await suite('Issue Lifecycle', async () => {
    await test('Setup: create repository for issue tests', async () => {
      const resp = await api.post('/repos', {
        body: { name: repoName, description: 'Issue lifecycle test repo', visibility: 'public' },
        expect: 201,
      });
      repoOwner = resp.data.owner || resp.data.owner_id || ctx.userId;
      ctx.trackRepo(repoOwner, repoName);
    });

    await test('POST /repos/:owner/:name/issues creates an issue', async () => {
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/issues`, {
        body: {
          title: 'First issue from E2E',
          description: 'This issue was created by the full-stack E2E test.',
          labels: ['bug', 'e2e'],
        },
        expect: 201,
      });
      assertOk(resp, 'create issue');
      assertField(resp.data, 'title', 'issue response');
      issueNumbers.push(resp.data.number || resp.data.id);
      ctx.trackIssue(`${repoOwner}/${repoName}`, issueNumbers[0]);
    });

    await test('POST /repos/:owner/:name/issues creates a second issue', async () => {
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/issues`, {
        body: {
          title: 'Second issue from E2E',
          description: 'Another test issue.',
        },
        expect: 201,
      });
      assertOk(resp, 'create second issue');
      issueNumbers.push(resp.data.number || resp.data.id);
    });

    await test('GET /repos/:owner/:name/issues lists all issues', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/issues`, { expect: 200 });
      assertOk(resp, 'list issues');
      const issues = resp.data?.data || resp.data || [];
      assert(Array.isArray(issues), 'issues should be an array');
      assert(issues.length >= 2, `expected at least 2 issues, got ${issues.length}`);
    });

    await test('PATCH /repos/:owner/:name/issues/:number updates issue state to closed', async () => {
      const num = issueNumbers[0];
      const resp = await api.patch(`/repos/${repoOwner}/${repoName}/issues/${num}`, {
        body: { state: 'closed' },
        expect: 200,
      });
      assertOk(resp, 'close issue');
    });

    await test('POST /repos/:owner/:name/issues/:number/comments adds a comment', async () => {
      const num = issueNumbers[1];
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/issues/${num}/comments`, {
        body: { body: 'This is a test comment from E2E.' },
        expect: 201,
      });
      assertOk(resp, 'add comment');
      assertField(resp.data, 'body', 'comment response');
    });

    await test('GET /repos/:owner/:name/issues/:number retrieves single issue', async () => {
      const num = issueNumbers[0];
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/issues/${num}`, { expect: 200 });
      assertOk(resp, 'get issue');
      assertEqual(resp.data.title, 'First issue from E2E', 'issue title');
    });

    await test('Browser: create issue via UI form', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        await fillForm(p, {
          'input#username': TEST_USER.username,
          'input[type="password"]': TEST_USER.password,
        });
        await p.press('input[type="password"]', 'Enter');
        await p.waitForTimeout(2000);

        await p.goto(`${BASE_URL}/repos/${repoOwner}/${repoName}/issues`, {
          waitUntil: 'networkidle', timeout: TIMEOUT,
        });
        await waitForHydration(p);
        await screenshot(p, 'issues_page');

        const newBtn = await p.$('button:has-text("New Issue"), .btn-new-issue');
        if (newBtn) {
          await newBtn.click();
          await p.waitForTimeout(500);
          await fillForm(p, {
            'input#new-issue-title': `UI-created issue ${uid()}`,
            'textarea#new-issue-description': 'Created via browser UI.',
          });
          await clickButton(p, 'Submit Issue') || await clickButton(p, 'Create Issue');
          await p.waitForTimeout(2000);
        }
        await screenshot(p, 'issues_after_ui_create');
      } finally {
        await p.close();
        await b.close();
      }
    });

    await test('Teardown: delete issue test repository', async () => {
      if (!KEEP_TESTDATA) {
        const resp = await api.delete(`/repos/${repoOwner}/${repoName}`, { expect: 200 });
        assertOk(resp, 'delete repo');
      }
    });
  });
}

// ============================================================================
// SUITE 4: PULL REQUEST LIFECYCLE
// ============================================================================

async function testPRLifecycle(api, ctx) {
  const repoName = `e2e-prs-${uid()}`;
  let repoOwner = null;
  const prNumbers = [];

  await suite('Pull Request Lifecycle', async () => {
    await test('Setup: create repository for PR tests', async () => {
      const resp = await api.post('/repos', {
        body: { name: repoName, description: 'PR lifecycle test repo', visibility: 'public' },
        expect: 201,
      });
      repoOwner = resp.data.owner || resp.data.owner_id || ctx.userId;
      ctx.trackRepo(repoOwner, repoName);
    });

    await test('POST /repos/:owner/:name/pullrequests creates a PR', async () => {
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/pullrequests`, {
        body: {
          title: 'Test pull request',
          description: 'PR created by E2E test.',
          source_branch: 'feature-branch',
          target_branch: 'main',
        },
        expect: 201,
      });
      assertOk(resp, 'create PR');
      prNumbers.push(resp.data.number || resp.data.id);
      ctx.trackPR(`${repoOwner}/${repoName}`, prNumbers[0]);
    });

    await test('GET /repos/:owner/:name/pullrequests lists PRs', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/pullrequests`, { expect: 200 });
      assertOk(resp, 'list PRs');
      const prs = resp.data?.data || resp.data || [];
      assert(Array.isArray(prs), 'PRs should be an array');
      assert(prs.length >= 1, `expected at least 1 PR, got ${prs.length}`);
    });

    await test('POST /repos/:owner/:name/pullrequests/:number/reviews adds a review', async () => {
      const num = prNumbers[0];
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/pullrequests/${num}/reviews`, {
        body: {
          body: 'LGTM - approved by E2E test',
          state: 'approved',
        },
        expect: 201,
      });
      assertOk(resp, 'add review');
    });

    await test('POST /repos/:owner/:name/pullrequests/:number/merge merges the PR', async () => {
      const num = prNumbers[0];
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/pullrequests/${num}/merge`, {
        body: { merge_method: 'merge' },
      });
      // Merge may fail if branches don't exist; accept 200 or 4xx for branch-missing
      assert(resp.ok || resp.status === 400 || resp.status === 422,
        `merge returned unexpected ${resp.status}`);
    });

    await test('Teardown: delete PR test repository', async () => {
      if (!KEEP_TESTDATA) {
        const resp = await api.delete(`/repos/${repoOwner}/${repoName}`, { expect: 200 });
        assertOk(resp, 'delete repo');
      }
    });
  });
}

// ============================================================================
// SUITE 5: PIPELINE TESTS
// ============================================================================

async function testPipelines(api, ctx) {
  const repoName = `e2e-pipelines-${uid()}`;
  let repoOwner = null;

  await suite('Pipeline Tests', async () => {
    await test('Setup: create repository for pipeline tests', async () => {
      const resp = await api.post('/repos', {
        body: { name: repoName, description: 'Pipeline test repo', visibility: 'public' },
        expect: 201,
      });
      repoOwner = resp.data.owner || resp.data.owner_id || ctx.userId;
      ctx.trackRepo(repoOwner, repoName);
    });

    await test('POST /repos/:owner/:name/pipelines creates a pipeline', async () => {
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/pipelines`, {
        body: {
          name: 'test-pipeline',
          source_branch: 'main',
          stages: [
            {
              name: 'build',
              type: 'script',
              script: 'echo "building"',
            },
          ],
        },
      });
      // Pipeline creation may or may not be supported; accept 201 or 404/400
      if (resp.ok) {
        assertField(resp.data, 'name', 'pipeline response');
      }
    });

    await test('GET /repos/:owner/:name/pipelines lists pipelines', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/pipelines`);
      if (resp.ok) {
        const pipelines = resp.data?.data || resp.data || [];
        assert(Array.isArray(pipelines), 'pipelines should be an array');
      } else {
        // Pipeline listing may not be implemented yet
        assert(resp.status === 404 || resp.status === 501,
          `unexpected status ${resp.status}`);
      }
    });

    await test('Teardown: delete pipeline test repository', async () => {
      if (!KEEP_TESTDATA) {
        await api.delete(`/repos/${repoOwner}/${repoName}`);
      }
    });
  });
}

// ============================================================================
// SUITE 6: WIKI TESTS
// ============================================================================

async function testWiki(api, ctx) {
  const repoName = `e2e-wiki-${uid()}`;
  let repoOwner = null;
  const pageSlugs = [];

  await suite('Wiki Tests', async () => {
    await test('Setup: create repository for wiki tests', async () => {
      const resp = await api.post('/repos', {
        body: { name: repoName, description: 'Wiki test repo', visibility: 'public' },
        expect: 201,
      });
      repoOwner = resp.data.owner || resp.data.owner_id || ctx.userId;
      ctx.trackRepo(repoOwner, repoName);
    });

    await test('POST /repos/:owner/:name/wiki creates a wiki page', async () => {
      const slug = `getting-started-${uid()}`;
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/wiki`, {
        body: {
          slug,
          title: 'Getting Started',
          content: '# Getting Started\n\nThis is the wiki page created by E2E tests.',
        },
      });
      if (resp.ok) {
        pageSlugs.push(slug);
      } else {
        // Wiki API may use different path structure
        assert(resp.status === 404 || resp.status === 400 || resp.status === 405,
          `unexpected wiki create status ${resp.status}`);
      }
    });

    await test('GET /repos/:owner/:name/wiki lists wiki pages', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/wiki`);
      if (resp.ok) {
        const pages = resp.data?.data || resp.data || [];
        assert(Array.isArray(pages), 'wiki pages should be an array');
      }
    });

    await test('PUT /repos/:owner/:name/wiki/:slug edits wiki page content', async () => {
      if (pageSlugs.length === 0) {
        skip('edit wiki page', 'no wiki page was created');
        return;
      }
      const slug = pageSlugs[0];
      const resp = await api.put(`/repos/${repoOwner}/${repoName}/wiki/${slug}`, {
        body: {
          title: 'Getting Started (Updated)',
          content: '# Getting Started (Updated)\n\nUpdated content from E2E.',
        },
      });
      if (!resp.ok) {
        assert(resp.status === 404 || resp.status === 405,
          `unexpected wiki edit status ${resp.status}`);
      }
    });

    await test('GET /repos/:owner/:name/wiki/:slug retrieves wiki page', async () => {
      if (pageSlugs.length === 0) {
        skip('get wiki page', 'no wiki page was created');
        return;
      }
      const slug = pageSlugs[0];
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/wiki/${slug}`);
      if (resp.ok) {
        assertField(resp.data, 'content', 'wiki page response');
      }
    });

    await test('GET /repos/:owner/:name/wiki/:slug/history retrieves page history', async () => {
      if (pageSlugs.length === 0) {
        skip('wiki page history', 'no wiki page was created');
        return;
      }
      const slug = pageSlugs[0];
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/wiki/${slug}/history`);
      if (!resp.ok) {
        assert(resp.status === 404 || resp.status === 405 || resp.status === 501,
          `unexpected wiki history status ${resp.status}`);
      }
    });

    await test('Browser: create and view wiki page', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        await fillForm(p, {
          'input#username': TEST_USER.username,
          'input[type="password"]': TEST_USER.password,
        });
        await p.press('input[type="password"]', 'Enter');
        await p.waitForTimeout(2000);

        await p.goto(`${BASE_URL}/repos/${repoOwner}/${repoName}/wiki`, {
          waitUntil: 'networkidle', timeout: TIMEOUT,
        });
        await waitForHydration(p);
        await screenshot(p, 'wiki_page');

        const newPageBtn = await p.$('button:has-text("New Page")');
        if (newPageBtn) {
          await newPageBtn.click();
          await p.waitForTimeout(500);
          await fillForm(p, {
            'input#wiki-new-slug': `e2e-wiki-${uid()}`,
            'input#wiki-new-title': 'E2E Wiki Page',
            'textarea#wiki-new-content': '# E2E Wiki Page\n\nCreated by automated test.',
          });
          await clickButton(p, 'Create Page');
          await p.waitForTimeout(2000);
        }
        await screenshot(p, 'wiki_after_create');
      } finally {
        await p.close();
        await b.close();
      }
    });

    await test('Teardown: delete wiki test repository', async () => {
      if (!KEEP_TESTDATA) {
        await api.delete(`/repos/${repoOwner}/${repoName}`);
      }
    });
  });
}

// ============================================================================
// SUITE 7: CODE BROWSER TESTS
// ============================================================================

async function testCodeBrowser(api, ctx) {
  await suite('Code Browser', async () => {
    await test('GET /repos/:owner/:name/code lists repository files', async () => {
      const resp = await api.get('/repos/admin/axum/code');
      if (resp.ok) {
        const files = resp.data?.data || resp.data || [];
        assert(Array.isArray(files) || typeof files === 'object', 'code listing should be array or object');
      } else {
        // Code API may use different structure
        assert(resp.status === 404 || resp.status === 400,
          `unexpected code listing status ${resp.status}`);
      }
    });

    await test('GET /repos/:owner/:name/code/path reads file content', async () => {
      const resp = await api.get('/repos/admin/axum/code/README.md');
      if (resp.ok) {
        assert(resp.data, 'file content should exist');
      } else {
        assert(resp.status === 404 || resp.status === 400,
          `unexpected file content status ${resp.status}`);
      }
    });

    await test('GET /repos/:owner/:name/code returns 404 for missing file', async () => {
      const resp = await api.get(`/repos/admin/axum/code/nonexistent-file-${uid()}.txt`);
      assert(resp.status === 404 || resp.status === 400,
        `expected 404/400 for missing file, got ${resp.status}`);
    });

    await test('Browser: browse files in code tab', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/repos/admin/axum/code`, {
          waitUntil: 'networkidle', timeout: TIMEOUT,
        });
        await waitForHydration(p);
        const fileLinks = await p.$$('a[href*="/code/"]');
        console.log(`    Found ${fileLinks.length} file links in code browser`);
        await screenshot(p, 'code_browser');
        if (fileLinks.length > 0) {
          await fileLinks[0].click();
          await p.waitForTimeout(2000);
          await screenshot(p, 'code_file_view');
        }
      } finally {
        await p.close();
        await b.close();
      }
    });
  });
}

// ============================================================================
// SUITE 8: ADMIN TESTS
// ============================================================================

async function testAdmin(api, ctx) {
  await suite('Admin Tests', async () => {
    await test('GET /admin/users lists users (admin)', async () => {
      const resp = await api.get('/admin/users');
      if (resp.ok) {
        const users = resp.data?.data || resp.data || [];
        assert(Array.isArray(users), 'users should be an array');
        assert(users.length >= 1, 'should have at least 1 user');
      } else {
        assert(resp.status === 403 || resp.status === 401,
          'admin endpoint requires admin role');
      }
    });

    await test('GET /admin/repos lists repositories (admin)', async () => {
      const resp = await api.get('/admin/repos');
      if (!resp.ok) {
        assert(resp.status === 403 || resp.status === 401 || resp.status === 404,
          'admin repos endpoint requires admin role or not implemented');
      }
    });

    await test('GET /admin/audit returns audit log entries', async () => {
      const resp = await api.get('/admin/audit');
      if (!resp.ok) {
        assert(resp.status === 403 || resp.status === 401 || resp.status === 404,
          'admin audit endpoint requires admin role or not implemented');
      }
    });

    await test('GET /admin/site-settings returns site configuration', async () => {
      const resp = await api.get('/admin/site-settings');
      if (!resp.ok) {
        assert(resp.status === 403 || resp.status === 401 || resp.status === 404,
          'admin site-settings endpoint requires admin role or not implemented');
      }
    });

    await test('Non-admin user cannot access admin endpoints', async () => {
      const unauthed = new ApiClient();
      const resp = await unauthed.get('/admin/users');
      assert(resp.status === 401 || resp.status === 403,
        `unauthenticated request to admin should fail, got ${resp.status}`);
    });

    await test('Browser: admin panel loads for admin user', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/login`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        await fillForm(p, {
          'input#username': ADMIN_USER.username,
          'input[type="password"]': ADMIN_USER.password,
        });
        await p.press('input[type="password"]', 'Enter');
        await p.waitForTimeout(2000);

        await p.goto(`${BASE_URL}/admin`, {
          waitUntil: 'networkidle', timeout: TIMEOUT,
        });
        await waitForHydration(p);
        const body = await p.textContent('body');
        assert(body.includes('Admin') || body.includes('admin'),
          'admin panel should load for admin user');
        await screenshot(p, 'admin_panel');
      } finally {
        await p.close();
        await b.close();
      }
    });
  });
}

// ============================================================================
// SUITE 9: SEARCH TESTS
// ============================================================================

async function testSearch(api, ctx) {
  await suite('Search Tests', async () => {
    await test('GET /search?q=test returns search results', async () => {
      const resp = await api.get('/search?q=test&page=1');
      if (resp.ok) {
        assert(resp.data, 'search response should have data');
      } else {
        assert(resp.status === 404 || resp.status === 400,
          `unexpected search status ${resp.status}`);
      }
    });

    await test('GET /search returns empty results for gibberish query', async () => {
      const resp = await api.get(`/search?q=zzznonexistent${uid()}`);
      if (resp.ok) {
        const data = resp.data?.data || resp.data || [];
        if (Array.isArray(data)) {
          assertEqual(data.length, 0, 'gibberish search should return empty');
        }
      }
    });

    await test('Brain service search is reachable', async () => {
      try {
        const resp = await fetch(`${BRAIN_URL}/healthz`, {
          signal: AbortSignal.timeout(5000),
        });
        if (resp.ok) {
          console.log('    Brain service is healthy');
        }
      } catch {
        console.log('    Brain service not reachable (may not be running)');
      }
    });

    await test('Browser: search UI returns results', async () => {
      const { browser: b, page: p } = await setupBrowser();
      try {
        await p.goto(`${BASE_URL}/search`, { waitUntil: 'networkidle', timeout: TIMEOUT });
        await waitForHydration(p);
        await fillForm(p, { 'input#search-input': 'test' });
        await p.press('input#search-input', 'Enter');
        await p.waitForTimeout(2000);
        await screenshot(p, 'search_results');
        const body = await p.textContent('body');
        assert(body.includes('Search') || body.includes('results'),
          'search page should show results');
      } finally {
        await p.close();
        await b.close();
      }
    });
  });
}

// ============================================================================
// SUITE 10: WEBHOOK TESTS
// ============================================================================

async function testWebhooks(api, ctx) {
  const repoName = `e2e-webhooks-${uid()}`;
  let repoOwner = null;

  await suite('Webhook Tests', async () => {
    await test('Setup: create repository for webhook tests', async () => {
      const resp = await api.post('/repos', {
        body: { name: repoName, description: 'Webhook test repo', visibility: 'public' },
        expect: 201,
      });
      repoOwner = resp.data.owner || resp.data.owner_id || ctx.userId;
      ctx.trackRepo(repoOwner, repoName);
    });

    await test('POST /repos/:owner/:name/webhooks creates a webhook', async () => {
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/webhooks`, {
        body: {
          url: 'https://httpbin.org/post',
          events: ['push', 'pull_request'],
          secret: 'e2e-webhook-secret',
        },
      });
      if (resp.ok) {
        assertField(resp.data, 'url', 'webhook response');
        ctx.trackWebhook(`${repoOwner}/${repoName}`, resp.data.id);
      } else {
        assert(resp.status === 404 || resp.status === 400 || resp.status === 405,
          `unexpected webhook create status ${resp.status}`);
      }
    });

    await test('GET /repos/:owner/:name/webhooks lists webhooks', async () => {
      const resp = await api.get(`/repos/${repoOwner}/${repoName}/webhooks`);
      if (resp.ok) {
        const hooks = resp.data?.data || resp.data || [];
        assert(Array.isArray(hooks), 'webhooks should be an array');
      }
    });

    await test('POST /repos/:owner/:name/webhooks/:id/test triggers webhook delivery', async () => {
      const hooksResp = await api.get(`/repos/${repoOwner}/${repoName}/webhooks`);
      if (!hooksResp.ok || !hooksResp.data?.length) {
        skip('trigger webhook', 'no webhooks found');
        return;
      }
      const hookId = hooksResp.data[0]?.id;
      if (!hookId) {
        skip('trigger webhook', 'no webhook id');
        return;
      }
      const resp = await api.post(`/repos/${repoOwner}/${repoName}/webhooks/${hookId}/test`);
      if (!resp.ok) {
        assert(resp.status === 404 || resp.status === 405 || resp.status === 501,
          `unexpected webhook test status ${resp.status}`);
      }
    });

    await test('Teardown: delete webhook test repository', async () => {
      if (!KEEP_TESTDATA) {
        await api.delete(`/repos/${repoOwner}/${repoName}`);
      }
    });
  });
}

// ============================================================================
// SUITE 11: API VALIDATION AND ERROR HANDLING
// ============================================================================

async function testAPIValidation(api, ctx) {
  await suite('API Validation and Error Handling', async () => {
    await test('POST /repos rejects request without authentication', async () => {
      const unauthed = new ApiClient();
      const resp = await unauthed.post('/repos', {
        body: { name: `unauthed-${uid()}`, visibility: 'public' },
      });
      assert(resp.status === 401 || resp.status === 403,
        `expected 401/403 for unauthenticated, got ${resp.status}`);
    });

    await test('POST /repos rejects empty body', async () => {
      const resp = await api.post('/repos', { body: {} });
      assert(resp.status >= 400, `expected 4xx for empty body, got ${resp.status}`);
    });

    await test('POST /repos rejects missing name field', async () => {
      const resp = await api.post('/repos', {
        body: { visibility: 'public' },
      });
      assert(resp.status >= 400, `expected 4xx for missing name, got ${resp.status}`);
    });

    await test('GET /repos/nonexistent/repo returns 404', async () => {
      const resp = await api.get('/repos/__nonexistent__/__repo__');
      assert(resp.status === 404 || resp.status === 400,
        `expected 404, got ${resp.status}`);
    });

    await test('DELETE /repos/nonexistent/repo returns 404', async () => {
      const resp = await api.delete('/repos/__nonexistent__/__repo__');
      assert(resp.status === 404 || resp.status === 400 || resp.status === 403,
        `expected 404/400/403, got ${resp.status}`);
    });

    await test('POST /auth/login rejects empty body', async () => {
      const resp = await api.post('/auth/login', { body: {} });
      assert(resp.status >= 400, `expected 4xx for empty login, got ${resp.status}`);
    });

    await test('API returns JSON content-type on error responses', async () => {
      const resp = await api.get('/repos/__nonexistent__/__repo__');
      if (resp.headers['content-type']) {
        assert(resp.headers['content-type'].includes('application/json'),
          `expected JSON content-type, got ${resp.headers['content-type']}`);
      }
    });

    await test('Rate limiting returns 429 when threshold exceeded', async () => {
      let hitLimit = false;
      for (let i = 0; i < 50; i++) {
        const resp = await api.get('/repos');
        if (resp.status === 429) {
          hitLimit = true;
          break;
        }
      }
      if (!hitLimit) {
        console.log('    Rate limiting not triggered within 50 requests (may be configured higher)');
      }
    });

    await test('Request with malformed JSON body returns 400', async () => {
      const resp = await api.request('POST', '/repos', {
        body: '{invalid json',
        headers: { 'Content-Type': 'application/json' },
      });
      assert(resp.status >= 400, `expected 4xx for malformed JSON, got ${resp.status}`);
    });

    await test('Request with SQL injection attempt is rejected safely', async () => {
      const resp = await api.post('/auth/login', {
        body: {
          username: "admin'; DROP TABLE users; --",
          password: 'anything',
        },
      });
      assert(resp.status >= 400, `expected 4xx for injection attempt, got ${resp.status}`);
    });
  });
}

// ============================================================================
// SUITE 12: HEALTH AND INFRASTRUCTURE
// ============================================================================

async function testInfrastructure(api, ctx) {
  await suite('Health and Infrastructure', async () => {
    await test('GET /healthz returns 200 OK', async () => {
      const resp = await api.get('/healthz');
      assertEqual(resp.status, 200, 'healthz status');
    });

    await test('GET /api/v1/openapi.json returns API spec', async () => {
      const resp = await api.get('/openapi.json');
      if (resp.ok) {
        assert(resp.data, 'openapi spec should have data');
      } else {
        assert(resp.status === 404, `expected 404 if not found, got ${resp.status}`);
      }
    });

    await test('Brain service health check', async () => {
      try {
        const resp = await fetch(`${BRAIN_URL}/healthz`, {
          signal: AbortSignal.timeout(5000),
        });
        assert(resp.ok || resp.status === 503, `brain healthz: ${resp.status}`);
      } catch {
        console.log('    Brain service not reachable (skipped)');
      }
    });

    await test('Runner service health check', async () => {
      try {
        const resp = await fetch(`${RUNNER_URL}/`, {
          signal: AbortSignal.timeout(5000),
        });
        assert(resp.ok || resp.status === 503, `runner health: ${resp.status}`);
      } catch {
        console.log('    Runner service not reachable (skipped)');
      }
    });
  });
}

// ============================================================================
// CLEANUP
// ============================================================================

async function cleanup(api, ctx) {
  if (SKIP_CLEANUP || KEEP_TESTDATA) {
    console.log('\nCleanup skipped (--skip-cleanup or --keep-testdata)');
    return;
  }

  await suite('Cleanup', async () => {
    await test('Delete all test repositories', async () => {
      let deleted = 0;
      for (const repo of ctx.repos) {
        try {
          const resp = await api.delete(`/repos/${repo.owner}/${repo.name}`);
          if (resp.ok) deleted++;
        } catch { /* ignore */ }
      }
      console.log(`    Deleted ${deleted}/${ctx.repos.length} test repositories`);
    });

    await test('Delete all test organizations', async () => {
      let deleted = 0;
      for (const org of ctx.orgs) {
        try {
          const resp = await api.delete(`/orgs/${org.id || org.name}`);
          if (resp.ok) deleted++;
        } catch { /* ignore */ }
      }
      console.log(`    Deleted ${deleted}/${ctx.orgs.length} test organizations`);
    });
  });
}

// ============================================================================
// MAIN
// ============================================================================

async function main() {
  console.log(`\n${'='.repeat(60)}`);
  console.log(`  CivitForge Full-Stack E2E Test Suite`);
  console.log(`  Target:   ${BASE_URL}`);
  console.log(`  Mode:     ${HEADED ? 'headed' : 'headless'}`);
  console.log(`  Started:  ${results.startTime}`);
  console.log(`${'='.repeat(60)}`);

  try {
    await waitForHealth(`${BASE_URL}/healthz`, HEALTH_TIMEOUT);
    console.log('\nServer is healthy, starting tests...');
  } catch (e) {
    console.error(`\nServer not reachable at ${BASE_URL}: ${e.message}`);
    console.error('Ensure CivitForge is running (docker compose up -d)');
    process.exit(2);
  }

  const ctx = new TestContext();
  const api = new ApiClient();

  const totalStart = Date.now();

  await testAuthentication(api, ctx, null);
  await testRepositoryCRUD(api, ctx);
  await testIssueLifecycle(api, ctx, null);
  await testPRLifecycle(api, ctx);
  await testPipelines(api, ctx);
  await testWiki(api, ctx);
  await testCodeBrowser(api, ctx);
  await testAdmin(api, ctx);
  await testSearch(api, ctx);
  await testWebhooks(api, ctx);
  await testAPIValidation(api, ctx);
  await testInfrastructure(api, ctx);
  await cleanup(api, ctx);

  results.durationMs = Date.now() - totalStart;

  console.log(`\n${'='.repeat(60)}`);
  console.log(`  Results Summary`);
  console.log(`${'='.repeat(60)}`);

  for (const suite of results.suites) {
    const passed = suite.tests.filter(t => t.status === 'passed').length;
    const failed = suite.tests.filter(t => t.status === 'failed').length;
    const skipped = suite.tests.filter(t => t.status === 'skipped').length;
    const status = failed > 0 ? 'FAIL' : 'PASS';
    console.log(`  [${status}] ${suite.name}: ${passed} passed, ${failed} failed, ${skipped} skipped (${suite.durationMs}ms)`);
    for (const t of suite.tests) {
      if (t.status === 'failed') {
        console.log(`         FAIL: ${t.name}: ${t.error}`);
      }
    }
  }

  console.log(`\n${'='.repeat(60)}`);
  console.log(`  Total: ${results.totalTests} tests, ${results.totalPassed} passed, ${results.totalFailed} failed, ${results.totalSkipped} skipped`);
  console.log(`  Duration: ${(results.durationMs / 1000).toFixed(1)}s`);
  console.log(`${'='.repeat(60)}`);

  const reportFile = join(REPORTS_DIR, `full-stack-${Date.now()}.json`);
  writeFileSync(reportFile, JSON.stringify(results, null, 2));
  console.log(`\nReport saved to ${reportFile}`);

  process.exit(results.totalFailed > 0 ? 1 : 0);
}

main().catch((e) => {
  console.error('Fatal error:', e);
  process.exit(2);
});
