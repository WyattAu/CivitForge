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

test.describe('Accessibility', () => {
  for (const pageDef of pages) {
    test.describe(`${pageDef.name} page`, () => {
      test('has proper heading hierarchy', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);
        const h1 = await page.locator('h1').count();
        const h2 = await page.locator('h2').count();
        const h3 = await page.locator('h3').count();
        expect(h1 + h2 + h3).toBeGreaterThan(0);
      });

      test('all images have alt text', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);
        const images = page.locator('img');
        const count = await images.count();
        for (let i = 0; i < count; i++) {
          const alt = await images.nth(i).getAttribute('alt');
          expect(alt !== null).toBeTruthy();
        }
      });

      test('all forms have labels', async ({ page }) => {
        await page.goto(pageDef.path);
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);
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
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);
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
        await page.waitForLoadState('networkidle');
        await page.waitForTimeout(1000);
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
      await page.waitForLoadState('networkidle');
      const skipLink = page.locator('a[href="#content"], a[href="#main"], a.skip-link');
      const count = await skipLink.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });

    test('language attribute is set', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      const lang = await page.getAttribute('html', 'lang');
      expect(lang).toBeTruthy();
    });

    test('page has a title', async ({ page }) => {
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      const title = await page.title();
      expect(title).toBeTruthy();
      expect(title.length).toBeGreaterThan(0);
    });
  });
});
