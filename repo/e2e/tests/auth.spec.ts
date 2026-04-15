/**
 * E2E tests: Authentication flows
 * Covers login success/failure, field validation, and logout.
 */

import { test, expect, Page } from '@playwright/test';

const ADMIN_USER = 'admin';
const ADMIN_PASS = 'Admin@2024';

async function gotoLogin(page: Page) {
  await page.goto('/');
  // Should land on login page (redirect or direct)
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
}

async function login(page: Page, username: string, password: string) {
  await page.fill('#login-username', username);
  await page.fill('#login-password', password);
  await page.click('#login-submit');
}

// ── Happy path ────────────────────────────────────────────────────────────────

test('login with valid admin credentials shows sidebar', async ({ page }) => {
  await gotoLogin(page);
  await login(page, ADMIN_USER, ADMIN_PASS);
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#logout-btn')).toBeVisible();
});

test('login with publisher credentials succeeds', async ({ page }) => {
  await gotoLogin(page);
  await login(page, 'publisher', 'Pub@2024');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
});

test('login with reviewer credentials succeeds', async ({ page }) => {
  await gotoLogin(page);
  await login(page, 'reviewer', 'Rev@2024');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
});

// ── Logout ────────────────────────────────────────────────────────────────────

test('logout returns user to login page', async ({ page }) => {
  await gotoLogin(page);
  await login(page, ADMIN_USER, ADMIN_PASS);
  await expect(page.locator('#logout-btn')).toBeVisible({ timeout: 15_000 });
  await page.click('#logout-btn');
  await expect(page.locator('#login-submit')).toBeVisible({ timeout: 10_000 });
});

// ── Client-side validation ────────────────────────────────────────────────────

test('submitting empty form shows username required error', async ({ page }) => {
  await gotoLogin(page);
  await page.click('#login-submit');
  await expect(page.locator('#login-username-error')).toBeVisible();
});

test('empty password shows password required error', async ({ page }) => {
  await gotoLogin(page);
  await page.fill('#login-username', 'admin');
  await page.click('#login-submit');
  await expect(page.locator('#login-password-error')).toBeVisible();
});

test('password shorter than 4 chars shows length error', async ({ page }) => {
  await gotoLogin(page);
  await page.fill('#login-username', 'admin');
  await page.fill('#login-password', 'abc');
  await page.click('#login-submit');
  await expect(page.locator('#login-password-error')).toContainText('4 characters');
});

// ── Wrong credentials ─────────────────────────────────────────────────────────

test('wrong password shows login error banner', async ({ page }) => {
  await gotoLogin(page);
  await login(page, ADMIN_USER, 'wrongpassword');
  await expect(page.locator('#login-error')).toBeVisible({ timeout: 10_000 });
});

test('non-existent user shows login error banner', async ({ page }) => {
  await gotoLogin(page);
  await login(page, 'ghost_user', 'Password1234');
  await expect(page.locator('#login-error')).toBeVisible({ timeout: 10_000 });
});

// ── Role-based sidebar sections ───────────────────────────────────────────────

test('admin sees System section in sidebar', async ({ page }) => {
  await gotoLogin(page);
  await login(page, ADMIN_USER, ADMIN_PASS);
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  // Admin should see Configuration link (System section)
  await expect(page.locator('#sidebar a[href*="configuration"], #sidebar [data-section="System"]')).toBeVisible();
});

test('publisher does not see System section in sidebar', async ({ page }) => {
  await gotoLogin(page);
  await login(page, 'publisher', 'Pub@2024');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  // Publisher should not see Configuration link
  await expect(page.locator('#sidebar a[href*="configuration"]')).toHaveCount(0);
});
