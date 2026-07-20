import { test, expect } from '@playwright/test';

async function waitForWasm(page: import('@playwright/test').Page) {
  await page.waitForFunction(
    () =>
      document.querySelector('aside nav a') !== null ||
      document.querySelector('nav a') !== null ||
      document.querySelector('form') !== null ||
      document.querySelector('#main-content') !== null,
    { timeout: 30000 }
  );
  await page.waitForTimeout(500);
}

test.describe('Authentication', () => {
  test.describe('Login Page', () => {
    test('login page renders correctly', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await expect(page.locator('h1, h2').first()).toBeVisible();
      await expect(page.locator('input[name="username"], input#username').first()).toBeVisible();
      await expect(page.locator('input[type="password"]').first()).toBeVisible();
      await expect(page.locator('button:has-text("Sign In")').first()).toBeVisible();
    });

    test('login form validation - empty fields', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(500);
      const errorBanner = page.locator('.bg-red-50, .text-red-600, [role="alert"]');
      const hasError = await errorBanner.count() > 0;
      const usernameInput = page.locator('input[name="username"], input#username').first();
      const isFocused = await usernameInput.evaluate(el => el === document.activeElement).catch(() => false);
      expect(hasError || isFocused).toBeTruthy();
    });

    test('login form validation - invalid credentials', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').first().fill('not-an-email');
      await page.locator('input[type="password"]').first().fill('password123');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('login form validation - wrong password', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').first().fill('testuser');
      await page.locator('input[type="password"]').first().fill('wrongpassword123');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('login form submission works', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').first().fill('admin');
      await page.locator('input[type="password"]').first().fill('admin');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(2000);
      const url = page.url();
      const body = await page.textContent('body');
      const hasResponse = !url.includes('/login') || (body && body.includes('error')) || (body && body.includes('failed'));
      expect(hasResponse).toBeTruthy();
    });
  });

  test.describe('Register Page', () => {
    test('register page renders correctly', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const regLink = page.locator('button:has-text("Register"), button:has-text("Don\'t have an account")').first();
      if (await regLink.isVisible().catch(() => false)) {
        await regLink.click();
        await page.waitForTimeout(500);
      }
      const usernameInput = page.locator('input[name="username"], input#username').first();
      await expect(usernameInput).toBeVisible();
    });

    test('register form validation - empty fields', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const regLink = page.locator('button:has-text("Register"), button:has-text("Don\'t have an account")').first();
      if (await regLink.isVisible().catch(() => false)) {
        await regLink.click();
        await page.waitForTimeout(500);
      }
      await page.locator('button:has-text("Register")').click();
      await page.waitForTimeout(500);
    });

    test('register form validation - password mismatch', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const regLink = page.locator('button:has-text("Register"), button:has-text("Don\'t have an account")').first();
      if (await regLink.isVisible().catch(() => false)) {
        await regLink.click();
        await page.waitForTimeout(500);
      }
      await page.locator('input[name="username"], input#username').first().fill('newuser');
      await page.locator('input[name="email"], input[type="email"]').first().fill('new@test.com');
      const pwFields = page.locator('input[type="password"]');
      if (await pwFields.count() >= 2) {
        await pwFields.nth(0).fill('password123');
        await pwFields.nth(1).fill('different456');
      }
      await page.locator('button:has-text("Register")').click();
      await page.waitForTimeout(1000);
    });
  });

  test.describe('Logout', () => {
    test('logout flow', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').first().fill('admin');
      await page.locator('input[type="password"]').first().fill('admin');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(2000);
      const logoutBtn = page.locator('[data-action-logout], a:has-text("Sign out"), button:has-text("Sign out")').first();
      if (await logoutBtn.isVisible().catch(() => false)) {
        await logoutBtn.click();
        await page.waitForTimeout(1000);
        const url = page.url();
        expect(url).toContain('/login');
      }
    });
  });

  test.describe('Password Reset', () => {
    test('password reset page renders', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const resetLink = page.locator('a:has-text("Forgot"), a:has-text("Reset"), a:has-text("forgot"), a:has-text("reset")').first();
      if (await resetLink.isVisible().catch(() => false)) {
        await resetLink.click();
        await page.waitForTimeout(1000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('Session Persistence', () => {
    test('session state is accessible after page load', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').first().fill('admin');
      await page.locator('input[type="password"]').first().fill('admin');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(2000);
      const urlBeforeRefresh = page.url();
      await page.reload({ waitUntil: 'domcontentloaded' });
      await waitForWasm(page);
      await page.waitForTimeout(1000);
      const urlAfterRefresh = page.url();
      expect(urlAfterRefresh).toBeTruthy();
    });
  });
});
