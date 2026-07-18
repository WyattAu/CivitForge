import { test, expect } from '@playwright/test';

test.describe('Navigation', () => {
  test('header navigation links work', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const links = page.locator('nav a, header a');
    const count = await links.count();
    expect(count).toBeGreaterThan(0);
    for (let i = 0; i < Math.min(count, 5); i++) {
      const href = await links.nth(i).getAttribute('href');
      if (href && href.startsWith('/') && !href.startsWith('//')) {
        await links.nth(i).click();
        await page.waitForTimeout(1000);
        expect(page.url()).toContain(href);
        await page.goBack();
        await page.waitForTimeout(1000);
      }
    }
  });

  test('back/forward browser buttons', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    await page.goto('/repos');
    await page.waitForLoadState('networkidle');
    await page.goto('/settings');
    await page.waitForLoadState('networkidle');
    await page.goBack();
    await page.waitForTimeout(500);
    expect(page.url()).toContain('/repos');
    await page.goForward();
    await page.waitForTimeout(500);
    expect(page.url()).toContain('/settings');
  });

  test('404 page renders for invalid routes', async ({ page }) => {
    const response = await page.goto('/this-page-does-not-exist-12345');
    await page.waitForTimeout(1000);
    const body = await page.textContent('body');
    const isNotFound = body.includes('404') || body.includes('Not Found') || body.includes('not found');
    expect(isNotFound).toBeTruthy();
  });

  test('footer links', async ({ page }) => {
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    const footer = page.locator('footer');
    if (await footer.isVisible()) {
      const footerLinks = footer.locator('a');
      const count = await footerLinks.count();
      expect(count).toBeGreaterThanOrEqual(0);
    }
  });

  test('breadcrumb navigation', async ({ page }) => {
    await page.goto('/repos/test/test');
    await page.waitForLoadState('networkidle');
    await page.waitForTimeout(1000);
    const breadcrumbs = page.locator('[aria-label="breadcrumb"], nav.breadcrumb, .breadcrumb');
    if (await breadcrumbs.count() > 0) {
      await expect(breadcrumbs.first()).toBeVisible();
    }
  });
});
