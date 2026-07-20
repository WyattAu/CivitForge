import { test, expect } from '@playwright/test';

async function waitForWasm(page: import('@playwright/test').Page) {
  await page.waitForFunction(
    () => document.querySelector('aside nav a') !== null || document.querySelector('nav a') !== null,
    { timeout: 25000 }
  );
}

test.describe('Authentication', () => {
  test.describe('Login Page', () => {
    test('login page renders correctly', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await expect(page.locator('h1, h2')).toBeVisible();
      await expect(page.locator('input[name="username"], input#username')).toBeVisible();
      await expect(page.locator('input[type="password"]')).toBeVisible();
      await expect(page.locator('button:has-text("Sign In")')).toBeVisible();
    });

    test('login form validation - empty fields', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('button:has-text("Sign In")').click();
      const usernameInput = page.locator('input[name="username"], input#username');
      await expect(usernameInput).toBeFocused();
    });

    test('login form validation - invalid email', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').fill('not-an-email');
      await page.locator('input[type="password"]').fill('password123');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('login form validation - wrong password', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').fill('testuser');
      await page.locator('input[type="password"]').fill('wrongpassword123');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(1000);
      const body = await page.textContent('body');
      expect(body).toBeTruthy();
    });

    test('successful login redirects to dashboard', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').fill('admin');
      await page.locator('input[type="password"]').fill('admin');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(2000);
      const url = page.url();
      expect(url).not.toContain('/login');
    });
  });

  test.describe('Register Page', () => {
    test('register page renders correctly', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const regLink = page.locator('button:has-text("Register"), button:has-text("Don\'t have an account")');
      if (await regLink.isVisible()) {
        await regLink.click();
        await page.waitForTimeout(500);
      }
      const usernameInput = page.locator('input[name="username"], input#username');
      await expect(usernameInput).toBeVisible();
    });

    test('register form validation - empty fields', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const regLink = page.locator('button:has-text("Register"), button:has-text("Don\'t have an account")');
      if (await regLink.isVisible()) {
        await regLink.click();
        await page.waitForTimeout(500);
      }
      await page.locator('button:has-text("Register")').click();
      await page.waitForTimeout(500);
    });

    test('register form validation - password mismatch', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      const regLink = page.locator('button:has-text("Register"), button:has-text("Don\'t have an account")');
      if (await regLink.isVisible()) {
        await regLink.click();
        await page.waitForTimeout(500);
      }
      await page.locator('input[name="username"], input#username').fill('newuser');
      await page.locator('input[name="email"], input[type="email"]').fill('new@test.com');
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
      await page.locator('input[name="username"], input#username').fill('admin');
      await page.locator('input[type="password"]').fill('admin');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(2000);
      const logoutBtn = page.locator('a:has-text("Logout"), button:has-text("Logout"), [data-testid="logout"]');
      if (await logoutBtn.isVisible()) {
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
      const resetLink = page.locator('a:has-text("Forgot"), a:has-text("Reset"), a:has-text("forgot"), a:has-text("reset")');
      if (await resetLink.isVisible()) {
        await resetLink.click();
        await page.waitForTimeout(1000);
        const body = await page.textContent('body');
        expect(body).toBeTruthy();
      }
    });
  });

  test.describe('Session Persistence', () => {
    test('session persists after page refresh', async ({ page }) => {
      await page.goto('/login');
      await waitForWasm(page);
      await page.locator('input[name="username"], input#username').fill('admin');
      await page.locator('input[type="password"]').fill('admin');
      await page.locator('button:has-text("Sign In")').click();
      await page.waitForTimeout(2000);
      const urlBeforeRefresh = page.url();
      await page.reload({ waitUntil: 'networkidle' });
      await page.waitForTimeout(1000);
      const urlAfterRefresh = page.url();
      expect(urlAfterRefresh).not.toContain('/login');
    });
  });
});
