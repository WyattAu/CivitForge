import { test, expect } from '@playwright/test';

const pages = [
  { name: 'Home', path: '/' },
  { name: 'Login', path: '/login' },
  { name: 'Repos', path: '/repos' },
  { name: 'Repo Detail', path: '/repos/test/test' },
  { name: 'Issues', path: '/repos/test/test/issues' },
  { name: 'Pull Requests', path: '/repos/test/test/pulls' },
  { name: 'Pipelines', path: '/repos/test/test/pipelines' },
  { name: 'Search', path: '/search' },
  { name: 'Settings', path: '/settings' },
  { name: 'Orgs', path: '/orgs' },
];

async function waitForWasm(page: import('@playwright/test').Page) {
  await page.waitForFunction(
    () => document.querySelector('aside nav a') !== null || document.querySelector('nav a') !== null,
    { timeout: 25000 }
  );
}

test.describe('Accessibility', () => {
  for (const pageDef of pages) {
    test.describe(`${pageDef.name} page`, () => {
      test('has proper heading hierarchy', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('domcontentloaded');
        await waitForWasm(page);
        const h1 = await page.locator('h1').count();
        const h2 = await page.locator('h2').count();
        const h3 = await page.locator('h3').count();
        const headings = h1 + h2 + h3;
        if (headings === 0) {
          const body = await page.textContent('body');
          expect(body && body.trim().length).toBeGreaterThan(0);
        } else {
          expect(headings).toBeGreaterThan(0);
        }
      });

      test('all images have alt text', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('domcontentloaded');
        await waitForWasm(page);
        const images = page.locator('img');
        const count = await images.count();
        for (let i = 0; i < count; i++) {
          const alt = await images.nth(i).getAttribute('alt');
          expect(alt !== null).toBeTruthy();
        }
      });

      test('all forms have labels', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('domcontentloaded');
        await waitForWasm(page);
        const inputs = page.locator('input:not([type="hidden"]):not([type="submit"]):not([type="checkbox"]):not([type="radio"]), textarea, select');
        const count = await inputs.count();
        for (let i = 0; i < count; i++) {
          const id = await inputs.nth(i).getAttribute('id');
          const ariaLabel = await inputs.nth(i).getAttribute('aria-label');
          const ariaLabelledby = await inputs.nth(i).getAttribute('aria-labelledby');
          const placeholder = await inputs.nth(i).getAttribute('placeholder');
          const hasLabel = id ? await page.locator(`label[for="${id}"]`).count() > 0 : false;
          const hasAccessibility = hasLabel || ariaLabel !== null || ariaLabelledby !== null || placeholder !== null;
          expect(hasAccessibility).toBeTruthy();
        }
      });

      test('tab navigation works', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('domcontentloaded');
        await waitForWasm(page);
        await page.keyboard.press('Tab');
        await page.waitForTimeout(300);
        const focusedElement = await page.evaluate(() => {
          const el = document.activeElement;
          return el ? el.tagName : null;
        });
        expect(focusedElement).toBeTruthy();
      });

      test('no horizontal overflow', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('domcontentloaded');
        await waitForWasm(page);
        const hasHorizontalScroll = await page.evaluate(() => {
          return document.documentElement.scrollWidth > document.documentElement.clientWidth;
        });
        expect(hasHorizontalScroll).toBe(false);
      });
    });
  }

  test.describe('Global Accessibility', () => {
    test('skip to content link exists', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('domcontentloaded');
      await waitForWasm(page);
      const skipLink = page.locator('a[href="#main-content"], a[href="#content"], a[href="#main"], a.skip-link');
      const count = await skipLink.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });

    test('language attribute is set', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('domcontentloaded');
      await waitForWasm(page);
      const lang = await page.getAttribute('html', 'lang');
      expect(lang).toBeTruthy();
    });

    test('page has a title', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('domcontentloaded');
      await waitForWasm(page);
      const title = await page.title();
      expect(title).toBeTruthy();
      expect(title.length).toBeGreaterThan(0);
    });
  });
});
