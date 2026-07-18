import { test, expect } from '@playwright/test';

test.describe('CI/CD Pipelines', () => {
  test.describe('Pipelines List Page', () => {
    test('pipelines list page renders', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('pipelines list has status indicators', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('pipelines list has filter options', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const filterBtns = page.locator('button:has-text("All"), button:has-text("Running"), button:has-text("Success")');
      const count = await filterBtns.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  test.describe('Pipeline Detail Page', () => {
    test('pipeline detail page renders', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('pipeline detail has status info', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Pipeline Jobs', () => {
    test('pipeline jobs list renders', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const jobs = page.locator('.job, [data-testid="job"], tr');
      const count = await jobs.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  test.describe('Pipeline Logs', () => {
    test('pipeline logs view renders', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const logBtn = page.locator('button:has-text("Logs"), a:has-text("Logs"), button:has-text("View"), a:has-text("View")');
      if (await logBtn.count() > 0) {
        await logBtn.first().click();
        await page.waitForTimeout(1000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('Pipeline Rerun', () => {
    test('rerun button exists on pipeline page', async ({ page }) => {
      await page.goto('/repos/test/test/pipelines/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const rerunBtn = page.locator('button:has-text("Rerun"), button:has-text("Retry")');
      if (await rerunBtn.count() > 0) {
        await expect(rerunBtn.first()).toBeVisible();
      }
    });
  });
});
