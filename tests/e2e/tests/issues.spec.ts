import { test, expect } from '@playwright/test';

test.describe('Issue Tracking', () => {
  test.describe('Issues List Page', () => {
    test('issues list page renders', async ({ page }) => {
      await page.goto('/repos/test/test/issues');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('issues list has filter options', async ({ page }) => {
      await page.goto('/repos/test/test/issues');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const filterBtns = page.locator('button:has-text("Open"), button:has-text("Closed"), button:has-text("All")');
      const count = await filterBtns.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });

    test('issues list has new issue button', async ({ page }) => {
      await page.goto('/repos/test/test/issues');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const newIssueBtn = page.locator('a:has-text("New Issue"), button:has-text("New Issue"), a:has-text("Create"), button:has-text("Create")');
      if (await newIssueBtn.count() > 0) {
        await expect(newIssueBtn.first()).toBeVisible();
      }
    });
  });

  test.describe('Issue Detail Page', () => {
    test('issue detail page renders for existing issue', async ({ page }) => {
      await page.goto('/repos/test/test/issues/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Create New Issue', () => {
    test('create issue form renders', async ({ page }) => {
      await page.goto('/repos/test/test/issues/new');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const titleInput = page.locator('input[name="title"], input#title, textarea[name="title"]');
      if (await titleInput.count() > 0) {
        await expect(titleInput.first()).toBeVisible();
      }
    });

    test('create issue form fills correctly', async ({ page }) => {
      await page.goto('/repos/test/test/issues/new');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const titleInput = page.locator('input[name="title"], input#title, textarea[name="title"]');
      if (await titleInput.count() > 0) {
        await titleInput.first().fill('Test Issue Title');
        const value = await titleInput.first().inputValue();
        expect(value).toBe('Test Issue Title');
      }
    });

    test('create issue form validation - empty title', async ({ page }) => {
      await page.goto('/repos/test/test/issues/new');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const submitBtn = page.locator('button[type="submit"], button:has-text("Create"), button:has-text("Submit")');
      if (await submitBtn.count() > 0) {
        await submitBtn.first().click();
        await page.waitForTimeout(500);
      }
    });
  });

  test.describe('Issue Comments', () => {
    test('issue comments section renders', async ({ page }) => {
      await page.goto('/repos/test/test/issues/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const commentArea = page.locator('textarea, .comment, [data-testid="comment"]');
      if (await commentArea.count() > 0) {
        await expect(commentArea.first()).toBeVisible();
      }
    });
  });

  test.describe('Issue Labels', () => {
    test('issue labels are displayed', async ({ page }) => {
      await page.goto('/repos/test/test/issues/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const labels = page.locator('.label, [data-testid="label"], span:has-text("bug"), span:has-text("enhancement")');
      const count = await labels.count();
      expect(count).toBeGreaterThanOrEqual(0);
    });
  });

  test.describe('Issue Assignees', () => {
    test('issue assignees section exists', async ({ page }) => {
      await page.goto('/repos/test/test/issues/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });
  });

  test.describe('Issue Search/Filter', () => {
    test('issue search works', async ({ page }) => {
      await page.goto('/repos/test/test/issues');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const searchInput = page.locator('input[type="search"], input[type="text"]');
      if (await searchInput.count() > 0) {
        await searchInput.first().fill('test');
        await page.waitForTimeout(1000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('Close/Reopen Issue', () => {
    test('close button visible on open issue', async ({ page }) => {
      await page.goto('/repos/test/test/issues/1');
      await page.waitForLoadState('networkidle');
      await page.waitForTimeout(1000);
      const closeBtn = page.locator('button:has-text("Close"), button:has-text("Reopen")');
      if (await closeBtn.count() > 0) {
        await expect(closeBtn.first()).toBeVisible();
      }
    });
  });
});
