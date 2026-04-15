/**
 * E2E tests: Lodging and rent negotiation flows.
 * Covers lodging list, new lodging form, deposit cap enforcement,
 * and rent change request/response lifecycle.
 */

import { test, expect, Page } from '@playwright/test';

async function loginAs(page: Page, username: string, password: string) {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.fill('#login-username', username);
  await page.fill('#login-password', password);
  await page.click('#login-submit');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
}

async function logout(page: Page) {
  await page.click('#logout-btn');
  await page.waitForSelector('#login-submit', { timeout: 10_000 });
}

// ── Lodging creation and persistence ─────────────────────────────────────────

test('admin creates a lodging and it persists in the list', async ({ page }) => {
  const uniqueName = `E2E Lodge ${Date.now()}`;

  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });
  await page.fill('#lodging-name', uniqueName);
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '1000');
  await page.click('#lodging-submit');
  await page.waitForLoadState('networkidle', { timeout: 15_000 });

  // Verify the lodging appears in the list
  await page.goto('/#/lodgings');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  await expect(page.getByText(uniqueName)).toBeVisible({ timeout: 10_000 });
});

// ── Deposit cap enforcement ───────────────────────────────────────────────────

test('deposit cap warning appears when deposit exceeds 1.5x rent', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });
  // Set rent = 1000, deposit = 2000 (exceeds 1.5x = 1500)
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '2000');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toBeVisible({ timeout: 5_000 });
});

test('deposit cap warning absent when deposit is within 1.5x rent', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });
  // Set rent = 1000, deposit = 1500 (exactly at cap — allowed)
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '1500');
  await page.press('#lodging-deposit', 'Tab');
  // The Yew component does not render the warning element at all when within cap.
  // Assert it is absent from the DOM — a conditional check would let a broken
  // component (one that never renders the warning) silently pass.
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0, { timeout: 3_000 });
});

// ── Rent change request lifecycle ─────────────────────────────────────────────

test('admin creates lodging then requests a rent change', async ({ page }) => {
  const uniqueName = `E2E RentChg ${Date.now()}`;

  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });
  await page.fill('#lodging-name', uniqueName);
  await page.fill('#lodging-rent', '1200');
  await page.fill('#lodging-deposit', '1200');
  await page.click('#lodging-submit');
  await page.waitForLoadState('networkidle', { timeout: 15_000 });

  // Navigate to lodging list and open this lodging's detail page
  await page.goto('/#/lodgings');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  await page.getByText(uniqueName).click();
  await page.waitForLoadState('networkidle', { timeout: 10_000 });

  // The lodging detail page (is_edit=true) always shows the rent change panel
  // for Administrator and Publisher roles. Unconditional assertion proves the
  // Yew component rendered the fields — a conditional would hide regressions.
  await expect(page.locator('#proposed-rent')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('#proposed-deposit')).toBeVisible();
  await expect(page.locator('#request-rent-change')).toBeVisible();

  await page.fill('#proposed-rent', '1400');
  await page.fill('#proposed-deposit', '1400');
  await page.click('#request-rent-change');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  // Sidebar must still be visible — page did not crash after submitting
  await expect(page.locator('#sidebar')).toBeVisible();
});

// ── Role restrictions ─────────────────────────────────────────────────────────

test('inventory_clerk cannot access lodging creation', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForLoadState('networkidle', { timeout: 5_000 });
  await expect(page.locator('#lodging-name')).toHaveCount(0);
  await expect(page.locator('#sidebar')).toBeVisible();
});

// ── Lodging navigation ────────────────────────────────────────────────────────

test('clinician can view lodging list', async ({ page }) => {
  await loginAs(page, 'clinician', 'Clin@2024');
  await page.goto('/#/lodgings');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  await expect(page.locator('#sidebar')).toBeVisible();
});

test('new lodging form has all required fields', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });
  await expect(page.locator('#lodging-rent')).toBeVisible();
  await expect(page.locator('#lodging-deposit')).toBeVisible();
  await expect(page.locator('#lodging-submit')).toBeVisible();
});
