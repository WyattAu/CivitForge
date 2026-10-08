// Real OFREP interop check: the community OpenFeature provider against a
// running CivitForge.
//
// Why this exists: our /ofrep endpoints are unit tested against shapes we
// wrote, which proves internal consistency and nothing about interoperability.
// This drives them with `@openfeature/ofrep-provider` — an implementation we
// did not write — so the reason codes, error codes, and the FLAG_NOT_FOUND
// contract are validated by a third party.
//
// Usage:
//   CIVITFORGE_URL=http://127.0.0.1:9091 node scripts/ofrep_interop.mjs
//
// Requires the flag fixtures the script creates itself, so it is safe to run
// against a dev instance: it creates what it needs and reports what it found.

import { OpenFeature } from '@openfeature/server-sdk';
import { OFREPProvider as OfrepProvider } from '@openfeature/ofrep-provider';

const base = process.env.CIVITFORGE_URL || 'http://127.0.0.1:9091';
const token = process.env.CIVIT_TOKEN || '';
const adminToken = process.env.CIVIT_ADMIN_TOKEN || token;

const log = (...a) => console.log(...a);

async function api(path, options = {}) {
  const res = await fetch(`${base}${path}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...(options.headers || {}),
    },
  });
  const text = await res.text();
  let body;
  try {
    body = text ? JSON.parse(text) : null;
  } catch {
    body = text;
  }
  return { status: res.status, body, etag: res.headers.get('etag') };
}

// ---------------------------------------------------------------------------
// Fixtures. Names are unique per run so repeated runs are idempotent.
// ---------------------------------------------------------------------------
const RUN = Date.now().toString(36);
const FIXTURES = [
  { name: `ofrep_off_${RUN}`, percentage: 0, expect: false },
  { name: `ofrep_full_${RUN}`, percentage: 100, expect: true },
  { name: `ofrep_partial_${RUN}`, percentage: 50, expect: 'boolean' },
];

async function seed() {
  if (!adminToken) {
    log('no admin token: skipping seeding, will only run read checks');
    return;
  }
  for (const f of FIXTURES) {
    const res = await api('/api/v1/admin/feature-flags', {
      method: 'POST',
      headers: { Authorization: `Bearer ${adminToken}` },
      body: JSON.stringify({
        name: f.name,
        description: 'OFREP interop fixture',
        kind: 'release',
        owner: 'interop',
        enabled: true,
        enabled_for_percentage: f.percentage,
      }),
    });
    if (res.status === 201) {
      log(`seeded ${f.name} at ${f.percentage}%`);
    } else if (res.status === 409) {
      log(`${f.name} already exists`);
    } else {
      log(`WARN could not seed ${f.name}: ${res.status} ${JSON.stringify(res.body)}`);
    }
  }
}

// ---------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------
const results = [];
async function check(name, fn, expect) {
  try {
    const got = await fn();
    const ok = expect(got);
    results.push({ name, got, ok });
    log(`${ok ? 'PASS' : 'FAIL'}  ${name} -> ${JSON.stringify(got)}`);
  } catch (e) {
    results.push({ name, error: String(e), ok: false });
    log(`FAIL  ${name} -> threw ${e}`);
  }
}

// The provider appends /ofrep/v1/evaluate/flags itself, so baseUrl is the
// server root. headers is an array of tuples, not an object.
const provider = new OfrepProvider({
  baseUrl: base,
  ...(token ? { headers: [['Authorization', `Bearer ${token}`]] } : {}),
});
await OpenFeature.setProviderAndWait(provider);
OpenFeature.setContext({ targetingKey: process.env.TARGETING_KEY || 'interop-user' });

await seed();

// A missing flag must fall back to the caller's default. This only works if
// FLAG_NOT_FOUND is distinguishable from an evaluation failure — the single
// most interop-significant detail in the spec.
await check(
  'missing flag falls back to the code default',
  () => OpenFeature.getClient().getBooleanValue(`no_such_flag_${RUN}`, false),
  (v) => v === false,
);

// The default is the opposite of the expected value, so a passthrough of the
// default (provider error) reads as a failure rather than coincidentally
// matching.
for (const f of FIXTURES) {
  const fallback = f.expect === false;
  await check(
    `${f.name} resolves as expected (not the fallback)`,
    () => OpenFeature.getClient().getBooleanValue(f.name, fallback),
    (v) => (f.expect === 'boolean' ? typeof v === 'boolean' : v === f.expect),
  );
}

// The partial rollout is subject-dependent, so the honest check is not "some
// boolean" but "the same value the wire returns for this exact subject".
await check(
  'partial rollout matches the wire decision for this subject',
  async () => {
    const key = FIXTURES.find((f) => f.expect === 'boolean')?.name;
    const viaSdk = await OpenFeature.getClient().getBooleanValue(key, true);
    const res = await api(`/ofrep/v1/evaluate/flags/${key}`, {
      method: 'POST',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      body: JSON.stringify({ context: { targetingKey: 'interop-user' } }),
    });
    const viaWire = res.body?.value;
    return { viaSdk, viaWire, consistent: viaSdk === viaWire };
  },
  (v) => v.consistent && typeof v.viaWire === 'boolean',
);

// Bulk endpoint, straight at the wire: the provider's own single-flag path is
// covered above, so this checks the response shape directly.
await check(
  'bulk evaluation returns one entry per flag',
  async () => {
    const res = await api('/ofrep/v1/evaluate/flags', {
      method: 'POST',
      headers: token ? { Authorization: `Bearer ${token}` } : {},
      body: JSON.stringify({ context: { targetingKey: 'interop-user' } }),
    });
    if (res.status !== 200) return { status: res.status, count: -1, etag: res.etag };
    const flags = res.body?.flags ?? [];
    return { status: res.status, count: flags.length, etag: res.etag };
  },
  (v) => v.status === 200 && v.count > 0 && typeof v.etag === 'string' && v.etag.startsWith('W/'),
);

// Conditional revalidation must actually revalidate.
await check(
  'bulk evaluation honours If-None-Match',
  async () => {
    const body = JSON.stringify({ context: { targetingKey: 'interop-user' } });
    const first = await api('/ofrep/v1/evaluate/flags', { method: 'POST', body });
    if (first.status !== 200 || !first.etag) return { first: first.status, second: 'no-etag' };
    const second = await api('/ofrep/v1/evaluate/flags', {
      method: 'POST',
      headers: { 'If-None-Match': first.etag },
      body,
    });
    return { first: first.status, second: second.status };
  },
  (v) => v.first === 200 && v.second === 304,
);

await check(
  'unknown flag returns FLAG_NOT_FOUND, not an evaluation failure',
  async () => {
    const res = await api(`/ofrep/v1/evaluate/flags/no_such_flag_${RUN}`, {
      method: 'POST',
      body: JSON.stringify({ context: { targetingKey: 'interop-user' } }),
    });
    return { status: res.status, errorCode: res.body?.errorCode, hasValue: 'value' in (res.body || {}) };
  },
  (v) => v.status === 404 && v.errorCode === 'FLAG_NOT_FOUND' && v.hasValue === false,
);

const failed = results.filter((r) => !r.ok);
log(`\n${results.length - failed.length}/${results.length} interop checks passed`);
if (failed.length) {
  for (const f of failed) log(`  failed: ${f.name}${f.error ? ` (${f.error})` : ` -> ${JSON.stringify(f.got)}`}`);
}
process.exit(failed.length === 0 ? 0 : 1);