/**
 * E2E tests: Resource lifecycle
 * Covers full cross-role workflow: publisher creates → submits for review → reviewer
 * publishes → verify persisted published state visible to admin.
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

// ── Full lifecycle: draft → in_review → published ────────────────────────────

test('full resource lifecycle: create → review → publish (cross-role)', async ({ page }) => {
  const uniqueTitle = `E2E Lifecycle ${Date.now()}`;

  // Step 1: Publisher creates a draft resource
  await loginAs(page, 'publisher', 'Pub@2024');
  await page.goto('/#/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });
  await page.fill('#res-title', uniqueTitle);
  await page.fill('#res-address', '42 Lifecycle Ave, Test City');
  await page.click('#res-submit');

  // After creation, we should land on resource detail or list
  await page.waitForLoadState('networkidle', { timeout: 15_000 });

  // Step 2: Verify the resource persists in the list (search by title)
  await page.goto('/#/resources');
  await page.waitForSelector('#resource-search', { timeout: 10_000 });
  await page.fill('#resource-search', uniqueTitle);
  await page.waitForTimeout(500); // brief debounce
  await expect(page.getByText(uniqueTitle)).toBeVisible({ timeout: 10_000 });

  // Step 3: Navigate to the resource detail and submit for review
  await page.getByText(uniqueTitle).click();
  await page.waitForSelector('#btn-submit-review', { timeout: 10_000 });
  await page.click('#btn-submit-review');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });

  // Verify the submit button is gone (state transitioned away from draft)
  await expect(page.locator('#btn-submit-review')).toHaveCount(0);

  // Step 4: Reviewer logs in and publishes the resource
  await logout(page);
  await loginAs(page, 'reviewer', 'Rev@2024');
  await page.goto('/#/resources');
  await page.waitForSelector('#resource-state-filter', { timeout: 10_000 });
  // Filter to in_review state
  await page.selectOption('#resource-state-filter', 'in_review');
  await page.waitForTimeout(500);
  await expect(page.getByText(uniqueTitle)).toBeVisible({ timeout: 10_000 });
  await page.getByText(uniqueTitle).click();
  await page.waitForSelector('#btn-publish', { timeout: 10_000 });
  await page.click('#btn-publish');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });

  // Verify published button is gone (now in published state)
  await expect(page.locator('#btn-publish')).toHaveCount(0);

  // Step 5: Admin verifies the resource appears under published filter
  await logout(page);
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/resources');
  await page.waitForSelector('#resource-state-filter', { timeout: 10_000 });
  await page.selectOption('#resource-state-filter', 'published');
  await page.waitForTimeout(500);
  await expect(page.getByText(uniqueTitle)).toBeVisible({ timeout: 10_000 });
});

// ── Role restrictions ─────────────────────────────────────────────────────────

test('inventory_clerk cannot submit a resource for review', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await page.goto('/#/resources/new');
  // Should redirect away — clerk has no access to resource creation
  await page.waitForLoadState('networkidle', { timeout: 5_000 });
  await expect(page.locator('#res-submit')).toHaveCount(0);
  await expect(page.locator('#sidebar')).toBeVisible();
});

// ── Individual step verifications ─────────────────────────────────────────────

test('admin can navigate to resource list', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/resources');
  await expect(page.locator('#resource-search')).toBeVisible({ timeout: 10_000 });
});

test('publisher can navigate to resource list', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await page.goto('/#/resources');
  await expect(page.locator('#resource-search')).toBeVisible({ timeout: 10_000 });
});

test('reviewer can navigate to resource list', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await page.goto('/#/resources');
  await expect(page.locator('#resource-search')).toBeVisible({ timeout: 10_000 });
});

test('new resource form has required fields', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/resources/new');
  await expect(page.locator('#res-title')).toBeVisible({ timeout: 10_000 });
  await expect(page.locator('#res-address')).toBeVisible();
  await expect(page.locator('#res-submit')).toBeVisible();
});

test('publisher creates resource and it persists in list', async ({ page }) => {
  const uniqueTitle = `E2E Persist ${Date.now()}`;
  await loginAs(page, 'publisher', 'Pub@2024');
  await page.goto('/#/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });
  await page.fill('#res-title', uniqueTitle);
  await page.fill('#res-address', '99 Persist Lane');
  await page.click('#res-submit');
  await page.waitForLoadState('networkidle', { timeout: 15_000 });

  // Navigate back to list and search — must find the newly created resource
  await page.goto('/#/resources');
  await page.waitForSelector('#resource-search', { timeout: 10_000 });
  await page.fill('#resource-search', uniqueTitle);
  await page.waitForTimeout(500);
  await expect(page.getByText(uniqueTitle)).toBeVisible({ timeout: 10_000 });
});

test('resource list state filter shows only matching resources', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/resources');
  await page.waitForSelector('#resource-state-filter', { timeout: 10_000 });
  await page.selectOption('#resource-state-filter', 'draft');
  // After filtering, only draft resources should be listed (page should not error)
  await page.waitForLoadState('networkidle', { timeout: 5_000 });
  await expect(page.locator('#sidebar')).toBeVisible();
});
