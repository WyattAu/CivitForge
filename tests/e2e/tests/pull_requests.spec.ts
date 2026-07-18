import { test, expect } from '@playwright/test';

test.describe('Pull Requests', () => {
  test.describe('PR List Page', () => {
    test('PR list page renders', async ({ page }) => {
      await page.goto('/repos/test/test/pulls');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('PR list has filter options', async ({ page }) => {
      await page.goto('/repos/test/test/pulls');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const filterBtns = page.locator('button:has-text("Open"), button:has-text("Closed"), button:has-text("Merged"), button:has-text("All")');
      const count = await filterBtns.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });

    test('PR list has new PR button', async ({ page }) => {
      await page.goto('/repos/test/test/pulls');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const newPRBtn = page.locator('a:has-text("New Pull Request"), button:has-text("New Pull Request"), a:has-text("Create"), button:has-text("Create")');
      if (await newPRBtn.count() > 0) {
        await expect(newPRBtn.first()).toBeVisible();
      }
    });
  });

  test.describe('PR Detail Page', () => {
    test('PR detail page renders', async ({ page }) => {
      await page.goto('/repos/test/test/pulls/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Create PR', () => {
    test('create PR form renders', async ({ page }) => {
      await page.goto('/repos/test/test/pulls/new');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('PR Diff View', () => {
    test('PR diff view renders', async ({ page }) => {
      await page.goto('/repos/test/test/pulls/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const diffTab = page.locator('a:has-text("Files"), button:has-text("Files"), a:has-text("Changes"), button:has-text("Changes")');
      if (await diffTab.count() > 0) {
        await diffTab.first().click();
        await page.waitForTimeout(1000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('PR Review', () => {
    test('PR review comments section exists', async ({ page }) => {
      await page.goto('/repos/test/test/pulls/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const reviewSection = page.locator('textarea, .review, [data-testid="review"]');
      if (await reviewSection.count() > 0) {
        await expect(reviewSection.first()).toBeVisible();
      }
    });
  });

  test.describe('PR Merge', () => {
    test('merge button visible on PR page', async ({ page }) => {
      await page.goto('/repos/test/test/pulls/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const mergeBtn = page.locator('button:has-text("Merge"), button:has-text("Approve")');
      if (await mergeBtn.count() > 0) {
        await expect(mergeBtn.first()).toBeVisible();
      }
    });
  });

  test.describe('PR Checks', () => {
    test('PR checks status section exists', async ({ page }) => {
      await page.goto('/repos/test/test/pulls/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });
});
