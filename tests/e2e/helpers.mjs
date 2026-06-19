import { API_URL, TIMEOUT } from './config.mjs';

export class ApiClient {
  constructor(token = null) {
    this.token = token;
    this.baseUrl = API_URL;
  }

  setToken(token) {
    this.token = token;
  }

  headers(extra = {}) {
    const h = { 'Content-Type': 'application/json', ...extra };
    if (this.token) {
      h['Authorization'] = `Bearer ${this.token}`;
    }
    return h;
  }

  async request(method, path, { body = null, headers = {}, expect = null } = {}) {
    const url = `${this.baseUrl}${path}`;
    const opts = {
      method,
      headers: this.headers(headers),
      signal: AbortSignal.timeout(TIMEOUT),
    };
    if (body !== null) {
      opts.body = typeof body === 'string' ? body : JSON.stringify(body);
    }

    const start = Date.now();
    const resp = await fetch(url, opts);
    const durationMs = Date.now() - start;

    let data = null;
    const ct = resp.headers.get('content-type') || '';
    if (ct.includes('application/json')) {
      data = await resp.json().catch(() => null);
    } else {
      data = await resp.text().catch(() => null);
    }

    const result = {
      ok: resp.ok,
      status: resp.status,
      statusText: resp.statusText,
      data,
      durationMs,
      headers: Object.fromEntries(resp.headers.entries()),
    };

    if (expect !== null && resp.status !== expect) {
      throw new Error(
        `Expected status ${expect}, got ${resp.status} ${resp.statusText} from ${method} ${path}`
      );
    }

    return result;
  }

  get(path, opts) { return this.request('GET', path, opts); }
  post(path, opts) { return this.request('POST', path, opts); }
  put(path, opts) { return this.request('PUT', path, opts); }
  patch(path, opts) { return this.request('PATCH', path, opts); }
  delete(path, opts) { return this.request('DELETE', path, opts); }
}

export class TestContext {
  constructor() {
    this.token = null;
    this.userId = null;
    this.username = null;
    this.repos = [];
    this.issues = [];
    this.prs = [];
    this.pipelines = [];
    this.webhooks = [];
    this.pages = [];
    this.orgs = [];
  }

  trackRepo(owner, name) {
    this.repos.push({ owner, name });
  }

  trackIssue(repoPath, number) {
    this.issues.push({ repoPath, number });
  }

  trackPR(repoPath, number) {
    this.prs.push({ repoPath, number });
  }

  trackWebhook(repoPath, id) {
    this.webhooks.push({ repoPath, id });
  }

  trackOrg(name, id) {
    this.orgs.push({ name, id });
  }
}

export function assert(condition, message) {
  if (!condition) {
    throw new Error(`Assertion failed: ${message}`);
  }
}

export function assertEqual(actual, expected, message) {
  if (actual !== expected) {
    throw new Error(
      `${message || 'assertEqual'}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`
    );
  }
}

export function assertStatus(resp, expected, message) {
  if (resp.status !== expected) {
    throw new Error(
      `${message || 'HTTP status'}: expected ${expected}, got ${resp.status} (${resp.statusText}). Body: ${JSON.stringify(resp.data).slice(0, 200)}`
    );
  }
}

export function assertOk(resp, message) {
  if (!resp.ok) {
    throw new Error(
      `${message || 'HTTP request'}: expected 2xx, got ${resp.status} (${resp.statusText}). Body: ${JSON.stringify(resp.data).slice(0, 200)}`
    );
  }
}

export function assertJson(resp, message) {
  const ct = resp.headers?.['content-type'] || '';
  if (!ct.includes('application/json')) {
    throw new Error(
      `${message || 'Content-Type'}: expected JSON, got ${ct || 'unknown'}`
    );
  }
}

export function assertField(obj, field, message) {
  if (!(field in (obj || {}))) {
    throw new Error(
      `${message || 'Missing field'}: "${field}" not found in ${JSON.stringify(obj).slice(0, 200)}`
    );
  }
}

export async function waitForHealth(url, timeoutMs = TIMEOUT, intervalMs = 1000) {
  const deadline = Date.now() + timeoutMs;
  let lastError = null;
  while (Date.now() < deadline) {
    try {
      const resp = await fetch(url, { signal: AbortSignal.timeout(2000) });
      if (resp.ok) return true;
      lastError = new Error(`HTTP ${resp.status}`);
    } catch (e) {
      lastError = e;
    }
    await sleep(intervalMs);
  }
  throw new Error(`Health check failed after ${timeoutMs}ms: ${lastError?.message}`);
}

export function sleep(ms) {
  return new Promise((r) => setTimeout(r, ms));
}

export function uid() {
  return Date.now().toString(36) + Math.random().toString(36).slice(2, 8);
}
