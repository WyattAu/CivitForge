# CivitForge Manual Test Procedure
# Generated: 2026-06-14

## How to Run

### Option A: Production Server (Recommended)
Open browser to: http://192.168.1.191:9200
- Full API + WASM stack running in Docker
- Latest code deployed and verified

### Option B: Tauri Desktop App
From the project root:
```bash
cd crates/civit-desktop
../target/release/civit-desktop
```
Or build fresh:
```bash
cd crates/civit-desktop
cargo tauri dev
```
Note: Requires display server (X11/Wayland). Binary at:
- Release: crates/civit-desktop/target/release/civit-desktop
- Debug: crates/civit-desktop/target/debug/civit-desktop

### Option C: Local WASM Dev Server
```bash
cd crates/civit-ui
trunk serve --address 127.0.0.1 --port 9092
```
Note: WASM compilation takes ~4-5 minutes. Requires API backend running separately.

---

## Test Credentials
- Admin: username=admin, password=(check server database)
- Or register a new account at /register

---

## Test Procedure

### PHASE 1: Authentication (Priority: Critical)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 1.1 | Navigate to /register | "Create Account" form visible with username, display name, email, password fields | [ ] |
| 1.2 | Fill registration form with valid data | All fields accept input | [ ] |
| 1.3 | Submit registration | Redirects to /repos or shows success | [ ] |
| 1.4 | Navigate to /login | "Sign In" form visible | [ ] |
| 1.5 | Login with valid credentials | Redirects to /, token stored in localStorage | [ ] |
| 1.6 | Login with invalid credentials | Shows error message, stays on login page | [ ] |
| 1.7 | Check sidebar after login | Shows authenticated nav (Repos, Activity, Settings, etc.) | [ ] |
| 1.8 | Click "Sign Out" (if visible) | Logs out, redirects to / | [ ] |
| 1.9 | Navigate to /settings when logged out | Shows "Sign in required" with login button | [ ] |
| 1.10 | Navigate to /profile when logged out | Shows "Sign in required" with login button | [ ] |

### PHASE 2: Home Page (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 2.1 | Navigate to / | "Welcome to CivitForge" heading | [ ] |
| 2.2 | Check sidebar navigation | Links: Home, Repositories, Activity, Explore, Organizations, Search | [ ] |
| 2.3 | Check footer | Copyright, Documentation, API, Status links, language selector | [ ] |
| 2.4 | Click "Get Started" button | Navigates to /register or /repos | [ ] |
| 2.5 | Click "Explore Repos" button | Navigates to /explore | [ ] |
| 2.6 | Check feature cards | Git Hosting, Issue Tracking, Wiki sections visible | [ ] |
| 2.7 | Test keyboard shortcut (?) | Keyboard shortcuts help modal appears | [ ] |

### PHASE 3: Repository Management (Priority: Critical)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 3.1 | Navigate to /repos | Repository list page loads | [ ] |
| 3.2 | Click "New Repository" | /new-repo form appears | [ ] |
| 3.3 | Fill repo name | Input accepted, no special chars validation | [ ] |
| 3.4 | Fill description | Textarea accepts input | [ ] |
| 3.5 | Select visibility (Public/Internal/Private) | Radio buttons work correctly | [ ] |
| 3.6 | Submit create repo | Repo created, redirects to repo page | [ ] |
| 3.7 | Navigate to /explore | Public repos listed with search | [ ] |
| 3.8 | Search repos in /explore | Filter works, results update | [ ] |
| 3.9 | Click a repo card | Navigates to repo detail page | [ ] |
| 3.10 | Check repo header | Owner/name, visibility badge, star/watch/fork/clone buttons | [ ] |
| 3.11 | Check repo tabs | Code, Issues, Pull Requests, Boards, Pipelines, Wiki, Settings | [ ] |

### PHASE 4: Code Browser (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 4.1 | Click "Code" tab on a repo | Code browser loads with file tree | [ ] |
| 4.2 | Check file tree | Files and folders listed correctly | [ ] |
| 4.3 | Click a folder | Navigates into folder, shows contents | [ ] |
| 4.4 | Click a file | File content displayed with syntax highlighting | [ ] |
| 4.5 | Check breadcrumbs | Path navigation works (owner / repo / path) | [ ] |
| 4.6 | Click breadcrumb to go up | Navigates to parent directory | [ ] |

### PHASE 5: Issues (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 5.1 | Click "Issues" tab | Issues list page loads | [ ] |
| 5.2 | Check filter tabs | All, Open, In Progress, Closed tabs visible | [ ] |
| 5.3 | Click "Open" filter | Shows only open issues | [ ] |
| 5.4 | Click "New Issue" button | New issue form appears | [ ] |
| 5.5 | Fill issue title and description | Fields accept input | [ ] |
| 5.6 | Submit issue | Issue created, appears in list | [ ] |
| 5.7 | Click an issue | Issue detail page with comments | [ ] |
| 5.8 | Add a comment | Comment appears in issue thread | [ ] |
| 5.9 | Check issue labels | Labels visible if assigned | [ ] |

### PHASE 6: Pull Requests (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 6.1 | Click "Pull Requests" tab | PR list page loads | [ ] |
| 6.2 | Check filter tabs | All, Open, Closed, Merged | [ ] |
| 6.3 | Click "New Pull Request" | PR creation form appears | [ ] |
| 6.4 | Fill PR title and description | Fields accept input | [ ] |
| 6.5 | Submit PR | PR created, appears in list | [ ] |
| 6.6 | Click a PR | PR detail page with diff view | [ ] |
| 6.7 | Check PR status | Open/Closed/Merged badge visible | [ ] |

### PHASE 7: Wiki (Priority: Medium)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 7.1 | Click "Wiki" tab | Wiki page loads | [ ] |
| 7.2 | Check pages sidebar | Page list visible or "No pages" message | [ ] |
| 7.3 | Click "New Page" | Wiki creation form appears | [ ] |
| 7.4 | Fill page slug, title, content | Fields accept input | [ ] |
| 7.5 | Submit page | Page created, appears in sidebar | [ ] |
| 7.6 | Click a page | Content rendered with Markdown | [ ] |
| 7.7 | Search wiki | Search input filters pages | [ ] |

### PHASE 8: Pipelines (Priority: Medium)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 8.1 | Click "Pipelines" tab | Pipeline page loads | [ ] |
| 8.2 | Check sub-tabs | Runs, Schedules, Secrets, Caches, Variables | [ ] |
| 8.3 | Check empty state | "No pipeline runs yet" message | [ ] |

### PHASE 9: Boards (Priority: Medium)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 9.1 | Click "Boards" tab | Boards page loads | [ ] |
| 9.2 | Check empty state | "No boards yet" with create button | [ ] |
| 9.3 | Click "New Board" | Board creation form appears | [ ] |
| 9.4 | Create a board | Board created with columns (Todo, In Progress, Done) | [ ] |

### PHASE 10: Settings (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 10.1 | Click "Settings" in sidebar | User settings page loads | [ ] |
| 10.2 | Check Profile section | Display name, bio fields | [ ] |
| 10.3 | Check SSH Keys section | Key list or "No SSH keys configured" | [ ] |
| 10.4 | Click "Add SSH Key" | SSH key input form appears | [ ] |
| 10.5 | Check Change Password section | Current/new/confirm password fields | [ ] |
| 10.6 | Check Danger Zone | Delete Account button with warning | [ ] |
| 10.7 | Test repo settings | Navigate to repo > Settings > General | [ ] |
| 10.8 | Check repo visibility settings | Public/Internal/Private radio buttons | [ ] |
| 10.9 | Check repo Danger Zone | Archive/Delete repository buttons | [ ] |

### PHASE 11: Admin Panel (Priority: High - Admin Only)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 11.1 | Click "Admin" in sidebar (if admin) | Admin panel loads | [ ] |
| 11.2 | Check Users tab | User list with search | [ ] |
| 11.3 | Check Repos tab | Repository list with search | [ ] |
| 11.4 | Check Audit Log tab | Audit events with filters (action, resource, actor, date) | [ ] |
| 11.5 | Check System tab | System info displayed | [ ] |
| 11.6 | Navigate to /admin/site-settings | Site settings page loads | [ ] |
| 11.7 | Non-admin access | Shows "Admin access required" | [ ] |

### PHASE 12: Navigation & Routing (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 12.1 | Click all sidebar links | Each navigates to correct page | [ ] |
| 12.2 | Click breadcrumb links | Navigate up the hierarchy | [ ] |
| 12.3 | Browser back button | Returns to previous page | [ ] |
| 12.4 | Browser forward button | Returns to next page | [ ] |
| 12.5 | Direct URL navigation | /repos, /explore, /search all load | [ ] |
| 12.6 | 404 page | /nonexistent shows "Page not found" with Go Home button | [ ] |

### PHASE 13: Search (Priority: Medium)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 13.1 | Click "Search" in sidebar | Search page loads | [ ] |
| 13.2 | Type in search input | Input accepts text | [ ] |
| 13.3 | Submit search | Results displayed or "No results" | [ ] |
| 13.4 | Click a result | Navigates to repo/page | [ ] |

### PHASE 14: Activity (Priority: Medium)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 14.1 | Click "Activity" in sidebar | Activity page loads | [ ] |
| 14.2 | Check filter buttons | All, Push, Open Issue, etc. | [ ] |
| 14.3 | Click a filter | Activity list filters correctly | [ ] |
| 14.4 | Check empty state | "No recent activity" with helpful message | [ ] |

### PHASE 15: Responsive Design (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 15.1 | Resize to mobile (375px) | Sidebar collapses to hamburger menu | [ ] |
| 15.2 | Click hamburger menu | Sidebar slides out | [ ] |
| 15.3 | Check home page mobile | Content stacks vertically, readable | [ ] |
| 15.4 | Check repo page mobile | Tabs scroll horizontally | [ ] |
| 15.5 | Check forms mobile | Inputs full width, buttons accessible | [ ] |
| 15.6 | Check settings mobile | Sections stack vertically | [ ] |

### PHASE 16: Accessibility (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 16.1 | Tab through home page | All interactive elements focusable in order | [ ] |
| 16.2 | Check skip navigation | "Skip to main content" link works | [ ] |
| 16.3 | Check form labels | All inputs have visible labels | [ ] |
| 16.4 | Check heading hierarchy | h1 -> h2 -> h3 (no skips) | [ ] |
| 16.5 | Check ARIA landmarks | header, nav, main, footer present | [ ] |
| 16.6 | Check color contrast | Text readable against backgrounds | [ ] |

### PHASE 17: Error Handling (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 17.1 | Submit empty login form | Validation error shown | [ ] |
| 17.2 | Submit invalid email format | Validation error shown | [ ] |
| 17.3 | Create repo with duplicate name | Error message shown | [ ] |
| 17.4 | Try to merge PR without permissions | Error or button disabled | [ ] |
| 17.5 | Network disconnect | Graceful error handling | [ ] |

### PHASE 18: Keyboard Shortcuts (Priority: Low)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 18.1 | Press ? key | Keyboard shortcuts help modal | [ ] |
| 18.2 | Press Escape | Modal closes | [ ] |
| 18.3 | Navigate with keyboard only | All features accessible | [ ] |

### PHASE 19: Theme & Locale (Priority: Low)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 19.1 | Click "Toggle Theme" | Switches between light/dark mode | [ ] |
| 19.2 | Check dark mode readability | All text readable, no contrast issues | [ ] |
| 19.3 | Change language dropdown | Language options available (EN, ZH, RU) | [ ] |
| 19.4 | Switch to another language | UI text changes (if translations exist) | [ ] |

### PHASE 20: Tauri Desktop Specific (Priority: High)

| # | Step | Expected Result | Pass |
|---|------|----------------|------|
| 20.1 | Launch Tauri app | Window opens with title "CivitForge" | [ ] |
| 20.2 | Check window size | 1280x800 default, resizable | [ ] |
| 20.3 | Check system tray | Tray icon present (if configured) | [ ] |
| 20.4 | Minimize/restore | Window minimizes and restores correctly | [ ] |
| 20.5 | Close and reopen | App state preserved | [ ] |
| 20.6 | Check devtools | Right-click > Inspect works (debug build) | [ ] |

---

## Scoring

| Phase | Total | Passed | Score |
|-------|-------|--------|-------|
| 1: Auth | 10 | | /10 |
| 2: Home | 7 | | /7 |
| 3: Repos | 11 | | /11 |
| 4: Code | 6 | | /6 |
| 5: Issues | 9 | | /9 |
| 6: PRs | 7 | | /7 |
| 7: Wiki | 7 | | /7 |
| 8: Pipelines | 3 | | /3 |
| 9: Boards | 4 | | /4 |
| 10: Settings | 9 | | /9 |
| 11: Admin | 7 | | /7 |
| 12: Navigation | 6 | | /6 |
| 13: Search | 4 | | /4 |
| 14: Activity | 4 | | /4 |
| 15: Responsive | 6 | | /6 |
| 16: A11y | 6 | | /6 |
| 17: Errors | 5 | | /5 |
| 18: Keyboard | 3 | | /3 |
| 19: Theme | 4 | | /4 |
| 20: Tauri | 6 | | /6 |
| **TOTAL** | **124** | | **/124** |
