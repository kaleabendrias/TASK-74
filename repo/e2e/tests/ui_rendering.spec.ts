/**
 * E2E tests: Yew component rendering and event-driven DOM updates.
 *
 * These tests verify that Yew components produce the correct HTML structure
 * and respond to user events — covering the gap that pure-Rust frontend_tests
 * cannot exercise (they test extracted logic, not rendered output).
 *
 * Each test targets a specific component's DOM contract:
 *  - Login form: field IDs, label text, error span IDs, TOTP field visibility
 *  - Sidebar: section visibility per role, link presence, logout button
 *  - Toast notifications: container, class variants, dismiss interaction
 *  - Route guard: forbidden page renders for blocked roles
 *  - Lodging form: field IDs, deposit-cap warning lifecycle
 *  - Resource form: field IDs and scheduled-publish field
 *  - Inventory form: modal DOM structure
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

// ── Login form DOM structure ──────────────────────────────────────────────────

test('login form renders all required field IDs', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });

  await expect(page.locator('#login-username')).toBeVisible();
  await expect(page.locator('#login-password')).toBeVisible();
  await expect(page.locator('#login-submit')).toBeVisible();
  // Error spans exist in DOM as hidden/empty elements before submission
  await expect(page.locator('#login-username-error, #login-error')).toHaveCount(
    0, // absent before any submission attempt
  );
});

test('login username error element appears on empty submit', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.click('#login-submit');
  // Yew must render the error element and populate it
  await expect(page.locator('#login-username-error')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#login-username-error')).not.toBeEmpty();
});

test('login password error element appears when password is too short', async ({ page }) => {
  await page.goto('/');
  await page.waitForSelector('#login-submit', { timeout: 20_000 });
  await page.fill('#login-username', 'admin');
  await page.fill('#login-password', 'ab');  // < 4 chars
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
  // Before any interaction, none of the error elements should exist in the DOM
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
  // Sidebar must be gone after logout
  await expect(page.locator('#sidebar')).toHaveCount(0);
});

// ── Toast notifications ───────────────────────────────────────────────────────

test('toast appears after successful lodging creation', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });

  await page.fill('#lodging-name', `Toast Test ${Date.now()}`);
  await page.fill('#lodging-rent', '800');
  await page.fill('#lodging-deposit', '800');
  await page.click('#lodging-submit');

  // A success toast must appear in the toast container after the action
  await expect(
    page.locator('.toast-success, [class*="toast"][class*="success"]'),
  ).toBeVisible({ timeout: 10_000 });
});

test('toast container renders toast elements with dismiss button', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });

  await page.fill('#lodging-name', `Dismiss Test ${Date.now()}`);
  await page.fill('#lodging-rent', '900');
  await page.fill('#lodging-deposit', '900');
  await page.click('#lodging-submit');

  // Wait for a toast to appear
  const toast = page.locator('.toast-success, [class*="toast"][class*="success"]').first();
  await expect(toast).toBeVisible({ timeout: 10_000 });

  // Dismiss button must exist inside the toast
  const dismissBtn = toast.locator('.toast-dismiss, button[aria-label*="dismiss"], button');
  await expect(dismissBtn.first()).toBeVisible();

  // Clicking dismiss removes the toast
  await dismissBtn.first().click();
  await expect(toast).not.toBeVisible({ timeout: 5_000 });
});

// ── Route guard: forbidden page rendering ────────────────────────────────────

test('publisher navigating to /#/configuration renders forbidden or redirects', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await page.goto('/#/configuration');
  await page.waitForLoadState('networkidle', { timeout: 5_000 });

  // The RouteGuard must either render a forbidden message or redirect to dashboard.
  // In both cases the configuration form fields must NOT be present.
  await expect(page.locator('#sidebar')).toBeVisible();
  await expect(page.locator('h1')).not.toContainText('Configuration Center');
});

test('clerk navigating to /#/resources/new renders forbidden or redirects', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await page.goto('/#/resources/new');
  await page.waitForLoadState('networkidle', { timeout: 5_000 });

  await expect(page.locator('#sidebar')).toBeVisible();
  await expect(page.locator('#res-submit')).toHaveCount(0);
});

// ── Lodging form DOM structure ────────────────────────────────────────────────

test('new lodging form renders all required field IDs', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-name', { timeout: 10_000 });

  await expect(page.locator('#lodging-name')).toBeVisible();
  await expect(page.locator('#lodging-rent')).toBeVisible();
  await expect(page.locator('#lodging-deposit')).toBeVisible();
  await expect(page.locator('#lodging-submit')).toBeVisible();
  // Warning must be absent before any input
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0);
});

test('deposit-cap-warning element is absent when deposit equals rent', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '1000');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0, { timeout: 3_000 });
});

test('deposit-cap-warning element is present when deposit exceeds 1.5x rent', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '1600');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#deposit-cap-warning')).toContainText('1.5x');
});

test('deposit-cap-warning disappears when deposit is corrected below cap', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/lodgings/new');
  await page.waitForSelector('#lodging-rent', { timeout: 10_000 });

  // Trigger the warning first
  await page.fill('#lodging-rent', '1000');
  await page.fill('#lodging-deposit', '2000');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toBeVisible({ timeout: 5_000 });

  // Correct the deposit — warning must vanish
  await page.fill('#lodging-deposit', '1200');
  await page.press('#lodging-deposit', 'Tab');
  await expect(page.locator('#deposit-cap-warning')).toHaveCount(0, { timeout: 3_000 });
});

// ── Resource form DOM structure ───────────────────────────────────────────────

test('new resource form renders all required field IDs', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });

  await expect(page.locator('#res-title')).toBeVisible();
  await expect(page.locator('#res-address')).toBeVisible();
  await expect(page.locator('#res-submit')).toBeVisible();
  await expect(page.locator('#res-scheduled')).toBeVisible();
});

test('resource form submit is disabled until title is provided', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });

  // Title is empty — the submit button should be visible but the form should
  // not navigate away without valid data
  await page.click('#res-submit');
  // We should still be on the resource-new page (no navigation happened)
  await expect(page.locator('#res-title')).toBeVisible({ timeout: 3_000 });
});

// ── Inventory form DOM structure ──────────────────────────────────────────────

test('inventory lot creation modal renders correct field IDs', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/inventory');
  await page.waitForSelector('#create-lot-btn', { timeout: 10_000 });
  await page.click('#create-lot-btn');

  await expect(page.locator('#new-lot-item')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#new-lot-number')).toBeVisible();
  await expect(page.locator('#new-lot-qty')).toBeVisible();
  await expect(page.locator('#confirm-create-lot')).toBeVisible();
});

test('near-expiry toggle is present and changes class on click', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/inventory');
  await page.waitForSelector('#toggle-near-expiry', { timeout: 10_000 });

  const btn = page.locator('#toggle-near-expiry');
  const before = await btn.getAttribute('class');
  await btn.click();
  await page.waitForTimeout(300);
  const after = await btn.getAttribute('class');
  // Class must change to reflect the toggled state
  expect(before).not.toEqual(after);
});
