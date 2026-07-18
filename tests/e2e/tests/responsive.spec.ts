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
];

const viewports = [
  { name: 'Mobile', width: 375, height: 667 },
  { name: 'Tablet', width: 768, height: 1024 },
  { name: 'Desktop', width: 1280, height: 720 },
];

test.describe('Responsive Design', () => {
  for (const viewport of viewports) {
    test.describe(`${viewport.name} (${viewport.width}x${viewport.height})`, () => {
      for (const pageDef of pages) {
        test(`${pageDef.name} renders correctly`, async ({ page }) => {
          await page.setViewportSize({ width: viewport.width, height: viewport.height });
          await page.goto(pageDef.path);
          await page.waitForLoadState('networkidle');
          await page.waitForTimeout(1000);
          const body = await page.textContent('body');
          expect(body).toBeTruthy();
          expect(body.length).toBeGreaterThan(0);
        });

        test(`${pageDef.name} has no horizontal overflow`, async ({ page }) => {
          await page.setViewportSize({ width: viewport.width, height: viewport.height });
          await page.goto(pageDef.path);
          await page.waitForLoadState('networkidle');
          await page.waitForTimeout(1000);
          const hasHorizontalScroll = await page.evaluate(() => {
            return document.documentElement.scrollWidth > document.documentElement.clientWidth;
          });
          expect(hasHorizontalScroll).toBe(false);
        });

        test(`${pageDef.name} body fits within viewport`, async ({ page }) => {
          await page.setViewportSize({ width: viewport.width, height: viewport.height });
          await page.goto(pageDef.path);
          await page.waitForLoadState('networkidle');
          await page.waitForTimeout(1000);
          const bodyBox = await page.locator('body').boundingBox();
          if (bodyBox) {
            expect(bodyBox.width).toBeLessThanOrEqual(viewport.width + 20);
          }
        });
      }
    });
  }

  test.describe('Navigation Adapts to Viewport', () => {
    test('mobile navigation works', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const hamburger = page.locator('button[aria-label="Menu"], button:has-text("Menu"), .hamburger, [data-testid="menu-button"]');
      if (await hamburger.count() > 0) {
        await expect(hamburger.first()).toBeVisible();
        await hamburger.first().click();
        await page.waitForTimeout(500);
        const nav = page.locator('nav, .sidebar, [data-testid="mobile-nav"]');
        if (await nav.count() > 0) {
          await expect(nav.first()).toBeVisible();
        }
      }
    });

    test('tablet navigation renders', async ({ page }) => {
      await page.setViewportSize({ width: 768, height: 1024 });
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('desktop navigation renders full menu', async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 720 });
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const nav = page.locator('nav');
      if (await nav.count() > 0) {
        await expect(nav.first()).toBeVisible();
      }
    });
  });

  test.describe('Touch Targets', () => {
    test('interactive elements are large enough on mobile', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const buttons = page.locator('button, a, [role="button"]');
      const count = await buttons.count();
      for (let i = 0; i < Math.min(count, 10); i++) {
        const box = await buttons.nth(i).boundingBox();
        if (box && box.width > 0 && box.height > 0) {
          expect(box.height).toBeGreaterThanOrEqual(24);
        }
      }
    });
  });

  test.describe('Content Reflow', () => {
    test('content reflows on small screens without overflow', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/repos');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const hasHorizontalScroll = await page.evaluate(() => {
        return document.documentElement.scrollWidth > document.documentElement.clientWidth;
      });
      expect(hasHorizontalScroll).toBe(false);
    });

    test('text is readable without zooming', async ({ page }) => {
      await page.setViewportSize({ width: 375, height: 667 });
      await page.goto('/');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const fontSize = await page.evaluate(() => {
        const body = document.body;
        return window.getComputedStyle(body).fontSize;
      });
      const size = parseInt(fontSize);
      expect(size).toBeGreaterThanOrEqual(12);
    });
  });
});
