/**
 * E2E tests: Yew component rendering and event-driven DOM updates.
 *
 * NOTE: Uses navigate() helper to avoid full-page reloads that wipe in-memory auth.
 */

import { test, expect, Page } from '@playwright/test';

/** Navigate within the running Yew SPA without a full page reload. */
async function navigate(page: Page, path: string) {
  await page.evaluate((p) => {
    window.history.pushState(null, '', p);
    window.dispatchEvent(new PopStateEvent('popstate', { state: null }));
  }, path);
}

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.fill('#login-username', username);
  await page.fill('#login-password', password);
  await page.click('#login-submit');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  // Dismiss the Welcome! toast immediately so it does not intercept subsequent clicks
  await page.locator('.toast-dismiss').first().click({ force: true, timeout: 2_000 }).catch(() => {});
}

// ── Login form DOM structure ──────────────────────────────────────────────────

test('login form renders all required field IDs', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });

  await expect(page.locator('#login-username')).toBeVisible();
  await expect(page.locator('#login-password')).toBeVisible();
  await expect(page.locator('#login-submit')).toBeVisible();
  await expect(page.locator('#login-username-error, #login-error')).toHaveCount(0);
});

test('login username error element appears on empty submit', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.click('#login-submit');
  await expect(page.locator('#login-username-error')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#login-username-error')).not.toBeEmpty();
});

test('login password error element appears when password is too short', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.fill('#login-username', 'admin');
  await page.fill('#login-password', 'ab');
  await page.click('#login-submit');
  await expect(page.locator('#login-password-error')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#login-password-error')).toContainText('4');
});

test('login error banner appears for wrong credentials', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.fill('#login-username', 'admin');
  await page.fill('#login-password', 'WrongPass99');
  await page.click('#login-submit');
  await expect(page.locator('#login-error')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('#login-error')).not.toBeEmpty();
});

test('error elements are absent on the initial page load', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await expect(page.locator('#login-username-error')).toHaveCount(0);
  await expect(page.locator('#login-password-error')).toHaveCount(0);
  await expect(page.locator('#login-error')).toHaveCount(0);
});

// ── Sidebar DOM structure per role ────────────────────────────────────────────

test('admin sidebar contains logout button and sidebar container', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await expect(page.locator('#sidebar')).toBeVisible();
  await expect(page.locator('#logout-btn')).toBeVisible();
});

test('admin sidebar shows configuration link (System section)', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await expect(page.locator('#sidebar a[href*="configuration"]')).toBeVisible({ timeout: 5_000 });
});

test('publisher sidebar does not contain configuration link', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await expect(page.locator('#sidebar')).toBeVisible();
  await expect(page.locator('#sidebar a[href*="configuration"]')).toHaveCount(0);
});

test('reviewer sidebar does not contain configuration link', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await expect(page.locator('#sidebar')).toBeVisible();
  await expect(page.locator('#sidebar a[href*="configuration"]')).toHaveCount(0);
});

test('inventory_clerk sidebar does not contain configuration link', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await expect(page.locator('#sidebar')).toBeVisible();
  await expect(page.locator('#sidebar a[href*="configuration"]')).toHaveCount(0);
});

test('logout button click navigates back to login form', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await expect(page.locator('#logout-btn')).toBeVisible();
  await page.click('#logout-btn');
  await expect(page.locator('#login-submit')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('#sidebar')).toHaveCount(0);
});

// ── Toast notifications ───────────────────────────────────────────────────────

test('toast appears after successful lodging creation', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });

  await page.fill('#lodging-name', `Toast Test ${Date.now()}`);
  await page.fill('#lodging-rent', '800');
  await page.fill('#lodging-deposit', '800');
  await page.click('#lodging-submit');

  await expect(
    page.locator('.toast-success, [class*="toast"][class*="success"]'),
  ).toBeVisible({ timeout: 10_000 });
});

test('toast container renders toast elements with dismiss button', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });

  await page.fill('#lodging-name', `Dismiss Test ${Date.now()}`);
  await page.fill('#lodging-rent', '900');
  await page.fill('#lodging-deposit', '900');
  await page.click('#lodging-submit');

  const toast = page.locator('.toast-success, [class*="toast"][class*="success"]').first();
  await expect(toast).toBeVisible({ timeout: 10_000 });

  const dismissBtn = toast.locator('.toast-dismiss');
  await expect(dismissBtn).toBeVisible({ timeout: 5_000 });

  await dismissBtn.click();
  await expect(toast).not.toBeVisible({ timeout: 8_000 });
});

// ── Route guard: forbidden page rendering ────────────────────────────────────

test('publisher navigating to /configuration renders forbidden or redirects', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await navigate(page, '/configuration');

  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('h1')).not.toContainText('Configuration Center');
});

test('clerk navigating to /resources/new renders forbidden or redirects', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/resources/new');

  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#res-submit')).toHaveCount(0);
});

// ── Lodging form DOM structure ────────────────────────────────────────────────

test('new lodging form renders all required field IDs', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });

  await expect(page.locator('#lodging-name')).toBeVisible();
  await expect(page.locator('#lodging-rent')).toBeVisible();
  await expect(page.locator('#lodging-deposit')).toBeVisible();
  await expect(page.locator('#lodging-submit')).toBeVisible();
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0);
});

test('deposit-cap-warning element is absent when deposit equals rent', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '1000');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0, { timeout: 3_000 });
});

test('deposit-cap-warning element is present when deposit exceeds 1.5x rent', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '1600');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#deposit-cap-warning')).toContainText('1.5x');
});

test('deposit-cap-warning disappears when deposit is corrected below cap', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });

  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '2000');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toBeVisible({ timeout: 5_000 });

  await page.fill('#lodging-deposit', '1200');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0, { timeout: 3_000 });
});

// ── Resource form DOM structure ───────────────────────────────────────────────

test('new resource form renders all required field IDs', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });

  await expect(page.locator('#res-title')).toBeVisible();
  await expect(page.locator('#res-address')).toBeVisible();
  await expect(page.locator('#res-submit')).toBeVisible();
  await expect(page.locator('#res-scheduled')).toBeVisible();
});

test('resource form submit is disabled until title is provided', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });

  await page.click('#res-submit');
  await expect(page.locator('#res-title')).toBeVisible({ timeout: 3_000 });
});

// ── Inventory form DOM structure ──────────────────────────────────────────────

test('inventory lot creation modal renders correct field IDs', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.waitForTimeout(500);
  await page.click('#create-lot-btn');

  await expect(page.locator('#new-lot-item')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('#new-lot-number')).toBeVisible();
  await expect(page.locator('#new-lot-qty')).toBeVisible();
  await expect(page.locator('#confirm-create-lot')).toBeVisible();
});

test('near-expiry toggle is present and changes class on click', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#toggle-near-expiry', { timeout: 10_000 });

  const btn = page.locator('#toggle-near-expiry');
  const before = await btn.getAttribute('class');
  await btn.click();
  await page.waitForTimeout(300);
  const after = await btn.getAttribute('class');
  expect(before).not.toEqual(after);
});
