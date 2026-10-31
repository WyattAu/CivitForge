1. **Keep tests independent**; each test should set up its own state
2. **Avoid hard-coded sleeps**; prefer `waitForLoadState` or `waitForSelector`
4. **Test across browsers**; the config runs Chromium, Firefox, and WebKit
5. **Check optional elements**; use `if (await element.isVisible())` for non-guaranteed UI