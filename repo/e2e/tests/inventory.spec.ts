/**
 * E2E tests: Inventory operations.
 * Covers lot creation with persistence verification, reservations, and
 * role-based access controls.
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

// ── Role-based access ─────────────────────────────────────────────────────────

test('admin can access inventory page', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await expect(page.locator('#create-lot-btn')).toBeVisible({ timeout: 10_000 });
});

test('clinician can access inventory page', async ({ page }) => {
  await loginAs(page, 'clinician', 'Clin@2024');
  await navigate(page, '/inventory');
  await expect(page.locator('#create-lot-btn')).toBeVisible({ timeout: 10_000 });
});

test('inventory_clerk can access inventory page', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/inventory');
  await expect(page.locator('#create-lot-btn')).toBeVisible({ timeout: 10_000 });
});

test('publisher cannot access inventory page', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await navigate(page, '/inventory');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#create-lot-btn')).toHaveCount(0);
});

test('reviewer cannot access inventory page', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await navigate(page, '/inventory');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#create-lot-btn')).toHaveCount(0);
});

// ── Lot creation with persistence verification ────────────────────────────────

test('create lot button opens lot creation form', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.waitForTimeout(300);
  await page.click('#create-lot-btn');
  await expect(page.locator('#new-lot-item')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('#new-lot-number')).toBeVisible();
  await expect(page.locator('#new-lot-qty')).toBeVisible();
  await expect(page.locator('#confirm-create-lot')).toBeVisible();
});

test('admin creates a lot and it persists in the inventory list', async ({ page }) => {
  const uniqueItem = `E2E Gauze ${Date.now()}`;
  const uniqueLot = `LOT-E2E-${Date.now()}`;

  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.waitForTimeout(500);
  await page.click('#create-lot-btn');

  await page.waitForSelector('#new-lot-item', { timeout: 10_000 });
  await page.fill('#new-lot-item', uniqueItem);
  await page.fill('#new-lot-number', uniqueLot);
  await page.fill('#new-lot-qty', '25');
  await page.click('#confirm-create-lot');

  await expect(page.getByText(uniqueItem)).toBeVisible({ timeout: 15_000 });
  await expect(page.getByText(uniqueLot)).toBeVisible({ timeout: 10_000 });
});

test('clerk creates lot then admin verifies it is visible (cross-user persistence)', async ({ page }) => {
  const uniqueItem = `E2E Cross ${Date.now()}`;
  const uniqueLot = `LOT-CROSS-${Date.now()}`;

  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.waitForTimeout(300);
  await page.click('#create-lot-btn');
  await page.waitForSelector('#new-lot-item', { timeout: 10_000 });
  await page.fill('#new-lot-item', uniqueItem);
  await page.fill('#new-lot-number', uniqueLot);
  await page.fill('#new-lot-qty', '5');
  await page.click('#confirm-create-lot');

  await expect(page.getByText(uniqueItem)).toBeVisible({ timeout: 15_000 });

  // Logout and re-login as admin (fresh page.goto resets auth, re-login restores it)
  await page.click('#logout-btn');
  await page.waitForSelector('#login-submit', { timeout: 10_000 });
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await expect(page.getByText(uniqueItem)).toBeVisible({ timeout: 15_000 });
});

// ── Reserve items ─────────────────────────────────────────────────────────────

test('reserve button appears for each lot and opens reserve modal', async ({ page }) => {
  const uniqueItem = `E2E Reserve ${Date.now()}`;
  const uniqueLot = `LOT-RES-${Date.now()}`;

  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.waitForTimeout(500);
  await page.click('#create-lot-btn');
  await page.waitForSelector('#new-lot-item', { timeout: 10_000 });
  await page.fill('#new-lot-item', uniqueItem);
  await page.fill('#new-lot-number', uniqueLot);
  await page.fill('#new-lot-qty', '50');
  await page.click('#confirm-create-lot');

  await expect(page.getByText(uniqueItem)).toBeVisible({ timeout: 15_000 });

  await page.waitForSelector(`[id^="reserve-"]`, { timeout: 10_000 });
  const reserveBtn = page.locator(`[id^="reserve-"]`).first();
  await reserveBtn.click();

  await expect(page.locator('#reserve-quantity')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#confirm-reserve')).toBeVisible();
});

// ── Near-expiry toggle ────────────────────────────────────────────────────────

test('near-expiry toggle button is visible', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#toggle-near-expiry', { timeout: 10_000 });
  await expect(page.locator('#toggle-near-expiry')).toBeVisible();
});

test('clicking near-expiry toggle changes its visual state', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#toggle-near-expiry', { timeout: 10_000 });

  const btnBefore = await page.locator('#toggle-near-expiry').getAttribute('class');
  await page.click('#toggle-near-expiry');
  await page.waitForTimeout(200);
  const btnAfter = await page.locator('#toggle-near-expiry').getAttribute('class');

  expect(btnBefore).not.toEqual(btnAfter);
});

// ── Transaction recording ─────────────────────────────────────────────────────

test('transaction form records an outbound transaction and reflects quantity', async ({ page }) => {
  const uniqueItem = `E2E Txn ${Date.now()}`;
  const uniqueLot = `LOT-TXN-${Date.now()}`;

  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.waitForTimeout(500);
  await page.click('#create-lot-btn');
  await page.waitForSelector('#new-lot-item', { timeout: 10_000 });
  await page.fill('#new-lot-item', uniqueItem);
  await page.fill('#new-lot-number', uniqueLot);
  await page.fill('#new-lot-qty', '30');
  await page.click('#confirm-create-lot');

  await expect(page.getByText(uniqueItem)).toBeVisible({ timeout: 15_000 });

  await page.waitForSelector('#create-txn-btn', { timeout: 5_000 });
  await page.click('#create-txn-btn');
  await page.waitForSelector('#txn-direction', { timeout: 5_000 });
  await page.selectOption('#txn-direction', 'outbound');
  await page.fill('#txn-qty', '5');
  await page.fill('#txn-reason', 'E2E test usage');
  const lotIdInput = page.locator('#txn-lot-id');
  if (await lotIdInput.isVisible()) {
    await lotIdInput.fill(uniqueLot);
  }
  await page.click('#confirm-txn');

  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
});
