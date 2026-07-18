# CivitForge E2E Testing Guide

## Overview

CivitForge uses [Playwright](https://playwright.dev/) for end-to-end testing. Tests run against a live CivitForge instance and validate UI behavior across Chromium, Firefox, and WebKit.

## Prerequisites

- Node.js >= 18
- A running CivitForge instance (default: `http://localhost:9091`)
- Playwright browsers installed (`npx playwright install`)

## Quick Start

```bash
cd tests/e2e
npm install
npx playwright install
npx playwright test
```

## Configuration

Tests are configured in `playwright.config.ts`:

- **Base URL**: `http://localhost:9091` (override with `CIVITFORGE_URL` env var)
- **Timeout**: 30s per test
- **Retries**: 1
- **Browsers**: Chromium, Firefox, WebKit
- **Traces**: Recorded on first retry

### Environment Variables

| Variable | Default | Description |
|---|---|---|
| `CIVITFORGE_URL` | `http://localhost:9091` | Base URL of the CivitForge instance |

## Running Tests

```bash
# All browsers
npx playwright test

# Single browser
npx playwright test --project=chromium

# Specific test file
npx playwright test tests/auth.spec.ts

# Debug mode (headed, step-by-step)
npx playwright test --debug

# UI mode (interactive)
npx playwright test --ui

# List all tests
npx playwright test --list
```

## Test Structure

```
tests/e2e/
├── playwright.config.ts   # Playwright configuration
├── tsconfig.json          # TypeScript configuration
├── tests/
│   ├── auth.spec.ts           # Login, register, logout, session
│   ├── navigation.spec.ts     # Header, footer, breadcrumbs, 404
│   ├── repos.spec.ts          # Repository CRUD, settings, branches
│   ├── issues.spec.ts         # Issue tracking lifecycle
│   ├── pull_requests.spec.ts  # PR list, detail, review, merge
│   ├── pipelines.spec.ts      # CI/CD pipeline status, logs
│   ├── search.spec.ts         # Search and explore
│   ├── admin.spec.ts          # Admin dashboard, users, settings
│   ├── accessibility.spec.ts  # A11y checks (headings, labels, tabs)
│   └── responsive.spec.ts     # Mobile/tablet/desktop viewport tests
└── TESTING_GUIDE.md           # This file
```

## Writing Tests

All tests use `@playwright/test` and follow a consistent pattern:

```typescript
import { test, expect } from '@playwright/test';

test.describe('Feature Name', () => {
  test('does something', async ({ page }) => {
    await page.goto('/some-path');
    await page.waitForLoadState('networkidle');
    // Assert on page content
    const body = await page.textContent('body');
    expect(body).toBeTruthy();
  });
});
```

### Key Patterns

- **Relative URLs**: All `page.goto()` calls use relative paths. The `baseURL` is set in `playwright.config.ts`.
- **Wait strategies**: Tests use `waitForLoadState('networkidle')` and `waitForTimeout()` for stability.
- **Graceful degradation**: Optional UI elements are checked with `isVisible()` before interaction.
- **Selector flexibility**: Multiple selectors are used (e.g., `input[name="username"], input#username`) to handle different markup variants.

## CI Integration

E2E tests run in GitHub Actions via `.github/workflows/e2e.yml`. The workflow:

1. Builds the CivitForge server
2. Starts it with a test database
3. Runs Playwright tests against the live instance
4. Uploads HTML reports and traces as artifacts

### Manual CI Trigger

```bash
# Via GitHub CLI
gh workflow run e2e.yml
```

## Debugging

### Trace Viewer

After a failed test with `trace: 'on-first-retry'`, open the trace:

```bash
npx playwright show-trace test-results/<test-name>/trace.zip
```

### Screenshots

Failed tests automatically capture screenshots to `tests/e2e/screenshots/`.

### HTML Report

```bash
npx playwright show-report reports/playwright
```

## Best Practices

1. **Keep tests independent** — each test should set up its own state
2. **Avoid hard-coded sleeps** — prefer `waitForLoadState` or `waitForSelector`
3. **Use data-testid attributes** when adding test hooks to the app
4. **Test across browsers** — the config runs Chromium, Firefox, and WebKit
5. **Check optional elements** — use `if (await element.isVisible())` for non-guaranteed UI
