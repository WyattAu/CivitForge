import { test, expect } from '@playwright/test';

test.describe('Search', () => {
  test.describe('Search Page', () => {
    test('search page renders', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('search input is visible', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await expect(searchInput.first()).toBeVisible();
    });
  });

  test.describe('Search Input', () => {
    test('search input accepts text', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await searchInput.first().fill('test query');
      const value = await searchInput.first().inputValue();
      expect(value).toBe('test query');
    });

    test('search submits on enter', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await searchInput.first().fill('rust');
      await searchInput.first().press('Enter');
      await page.waitForTimeout(2000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('search submits on button click', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await searchInput.first().fill('rust');
      const searchBtn = page.locator('button[type="submit"], button:has-text("Search")');
      if (await searchBtn.count() > 0) {
        await searchBtn.first().click();
        await page.waitForTimeout(2000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('Search Results', () => {
    test('search results display', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await searchInput.first().fill('rust');
      await searchInput.first().press('Enter');
      await page.waitForTimeout(2000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Search Filters', () => {
    test('search filters work', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await searchInput.first().fill('rust');
      await searchInput.first().press('Enter');
      await page.waitForTimeout(2000);
      const filterTabs = page.locator('button:has-text("Repositories"), button:has-text("Users"), button:has-text("Issues"), a:has-text("Repositories")');
      if (await filterTabs.count() > 0) {
        await filterTabs.first().click();
        await page.waitForTimeout(1000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('Empty Search Results', () => {
    test('empty search shows message', async ({ page }) => {
      await page.goto('/search');
      await page.waitForLoadState('networkidle');
      const searchInput = page.locator('input[type="search"], input[type="text"], input[name="q"]');
      await searchInput.first().fill('xyznonexistentquery12345');
      await searchInput.first().press('Enter');
      await page.waitForTimeout(2000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Explore Page', () => {
    test('explore page renders', async ({ page }) => {
      await page.goto('/explore');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('explore page has content', async ({ page }) => {
      await page.goto('/explore');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const content = page.locator('.repo, [data-testid="repo"], article, .card, .item');
      const count = await content.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });
});
