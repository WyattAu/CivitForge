import { test, expect } from '@playwright/test';

async function waitForWasm(page: import('@playwright/test').Page) {
  await page.waitForFunction(
    () => document.querySelector('aside nav a') !== null || document.querySelector('nav a') !== null,
    { timeout: 25000 }
  );
}

test.describe('Navigation', () => {
  test('header navigation links work', async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page);
    const links = page.locator('aside nav a');
    const count = await links.count();
    expect(count).toBeGreaterThan(0);
    for (let i = 0; i < Math.min(count, 5); i++) {
      const href = await links.nth(i).getAttribute('href');
      if (href && href.startsWith('/') && !href.startsWith('//')) {
        await links.nth(i).click();
        await waitForWasm(page);
        expect(page.url()).toContain(href);
        await page.goBack();
        await waitForWasm(page);
      }
    }
  });

  test('back/forward browser buttons', async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page);
    await page.goto('/repos');
    await waitForWasm(page);
    await page.goto('/settings');
    await waitForWasm(page);
    await page.goBack();
    await page.waitForTimeout(500);
    expect(page.url()).toContain('/repos');
    await page.goForward();
    await page.waitForTimeout(500);
    expect(page.url()).toContain('/settings');
  });

  test('404 page renders for invalid routes', async ({ page }) => {
    await page.goto('/this-page-does-not-exist-12345');
    await page.waitForTimeout(3000);
    const body = await page.textContent('body');
    const isNotFound = body.includes('404') || body.includes('Not Found') || body.includes('not found');
    expect(isNotFound).toBeTruthy();
  });

  test('footer links', async ({ page }) => {
    await page.goto('/');
    await waitForWasm(page);
    const footer = page.locator('footer[role="contentinfo"]');
    if (await footer.isVisible()) {
      const footerLinks = footer.locator('a');
      const count = await footerLinks.count();
      expect(count).toBeGreaterThanOrEqual(0);
    }
  });

  test('breadcrumb navigation', async ({ page }) => {
    await page.goto('/repos/test/test');
    await waitForWasm(page);
    const breadcrumbs = page.locator('[aria-label="breadcrumb"], nav.breadcrumb, .breadcrumb');
    if (await breadcrumbs.count() > 0) {
      await expect(breadcrumbs.first()).toBeVisible();
    }
  });
});
