# CivitForge E2E Traversal Tests

Full-page traversal of the CivitForge UI using Playwright. Visits every route, clicks every button, fills every form, and captures all errors.

## Prerequisites

The CivitForge UI server must be running on `http://localhost:9091` (Leptos WASM CSR).

## Install

```bash
cd tests/e2e
npm install
npx playwright install chromium
```

## Run

```bash
# Headless (default)
npm test

# Visible browser
npm run test:headed
```

## What It Tests

| Page | Route | Actions |
|------|-------|---------|
| Home | `/` | Click "Get Started" link |
| Login | `/login` | Fill credentials, click Login, toggle Register, fill register form |
| Repos | `/repos` | Click "New Repository" |
| New Repo | `/new-repo` | Fill name/description, select visibility, click Create |
| Activity | `/activity` | Click all filter tabs (All, Push, Open Issue, Merge PR, Create Repo) |
| Search | `/search` | Fill search input, submit |
| Explore | `/explore` | Click pagination |
| Orgs | `/orgs` | Open "Create Organization" modal, fill form, close |
| Settings | `/settings` | Navigate all tabs (Profile, SSH Keys, Password, Danger Zone), fill profile form |
| 404 | `/nonexistent` | Verify 404 indicator renders |
| Repo Detail | `/repos/test/test` | Check page loads or shows error |
| Repo Sub-pages | `/repos/test/test/{wiki,issues,code,pipelines}` | Verify each renders |

## Error Capture

The script captures globally:

- **Console errors/warnings** — `page.on('console')`
- **Uncaught JS exceptions** — `page.on('pageerror')`
- **Network failures** — `page.on('requestfailed')`
- **HTTP 4xx/5xx responses** — `page.on('response')`

## Results

- **Exit code 0** — all pages passed, no errors
- **Exit code 1** — any page failed or errors were captured
- **Screenshots** saved to `tests/e2e/screenshots/` (one per page, plus failure captures)
- **JSON report** saved to `tests/e2e/traversal-report.json`

## Interpreting the Report

Each page is marked:

| Status | Meaning |
|--------|---------|
| `PASSED` | Page loaded, all actions completed, no errors |
| `ERRORS` | Page loaded but console/network errors were captured |
| `FAILED` | Navigation or action threw an exception |

The summary lists all captured errors grouped by type (console, JS exceptions, network, HTTP).
