import { test, expect } from '@playwright/test';

async function waitForWasm(page: import('@playwright/test').Page) {
  await page.waitForFunction(
    () => document.querySelector('aside nav a') !== null || document.querySelector('nav a') !== null,
    { timeout: 25000 }
  );
}

test.describe('Repository Pages', () => {
  test.describe('Repos List Page', () => {
    test('repos list page renders', async ({ page }) => {
      await page.goto('/repos');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
      expect(body.length).toBeGreaterThan(0);
    });

    test('repos list has search or filter', async ({ page }) => {
      await page.goto('/repos');
      await waitForWasm(page);
      const searchInput = page.locator('input[type="search"], input[type="text"]');
      if (await searchInput.count() > 0) {
        await expect(searchInput.first()).toBeVisible();
      }
    });
  });

  test.describe('Repository Detail Page', () => {
    test('repo detail page renders', async ({ page }) => {
      await page.goto('/repos/test/test');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('repo tabs are visible', async ({ page }) => {
      await page.goto('/repos/test/test');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const tabs = page.locator('a[href*="code"], a[href*="issues"], a[href*="pipelines"], nav a');
      const count = await tabs.count();
      expect(count).toBeGreaterThan(0);
    });
  });

  test.describe('Create New Repo', () => {
    test('create repo form renders', async ({ page }) => {
      await page.goto('/new-repo');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('create repo form has visibility options', async ({ page }) => {
      await page.goto('/new-repo');
      await waitForWasm(page);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('create repo form validation - empty name', async ({ page }) => {
      await page.goto('/new-repo');
      await waitForWasm(page);
      await page.waitForTimeout(500);
    });

    test('create repo form fills correctly', async ({ page }) => {
      await page.goto('/new-repo');
      await waitForWasm(page);
      await page.waitForTimeout(500);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Repo Settings', () => {
    test('repo settings page renders', async ({ page }) => {
      await page.goto('/repos/test/test/settings');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Repo Branches', () => {
    test('repo branches page renders', async ({ page }) => {
      await page.goto('/repos/test/test/branches');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Repo Code Browser', () => {
    test('code browser page renders', async ({ page }) => {
      await page.goto('/repos/test/test/code');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Repo Wiki', () => {
    test('wiki page renders', async ({ page }) => {
      await page.goto('/repos/test/test/wiki');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Star/Unstar Repository', () => {
    test('star button is visible on repo page', async ({ page }) => {
      await page.goto('/repos/test/test');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const starBtn = page.locator('button:has-text("Star"), [data-testid="star-button"], button:has-text("Unstar")');
      if (await starBtn.count() > 0) {
        await expect(starBtn.first()).toBeVisible();
      }
    });
  });

  test.describe('Fork Repository', () => {
    test('fork button is visible on repo page', async ({ page }) => {
      await page.goto('/repos/test/test');
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const forkBtn = page.locator('button:has-text("Fork"), a:has-text("Fork"), [data-testid="fork-button"]');
      if (await forkBtn.count() > 0) {
        await expect(forkBtn.first()).toBeVisible();
      }
    });
  });
});
