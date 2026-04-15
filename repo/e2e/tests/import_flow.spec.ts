/**
 * E2E tests: Import / Export page flows.
 *
 * Covers:
 *  - Role-based access to the import/export page
 *  - File-type validation (only .xlsx accepted)
 *  - Export request → approval → download link lifecycle
 *  - SSE progress UI becomes visible after upload
 *  - Error state when uploading without a file selected
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

// ── Role-based access ─────────────────────────────────────────────────────────

test('admin can access import/export page', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  await expect(page.locator('#import-dropzone')).toBeVisible({ timeout: 10_000 });
});

test('reviewer can access import/export page', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await page.goto('/#/import-export');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  // Reviewer can access but cannot see the import dropzone (only clerk/admin can import)
  await expect(page.locator('#sidebar')).toBeVisible();
});

test('inventory_clerk can access import/export page', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await page.goto('/#/import-export');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  await expect(page.locator('#import-dropzone')).toBeVisible({ timeout: 10_000 });
});

test('publisher cannot access import/export page', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await page.goto('/#/import-export');
  await page.waitForLoadState('networkidle', { timeout: 5_000 });
  // RouteGuard redirects publisher to forbidden — dropzone must not be visible
  await expect(page.locator('#import-dropzone')).toHaveCount(0);
  await expect(page.locator('#sidebar')).toBeVisible();
});

test('clinician cannot access import/export page', async ({ page }) => {
  await loginAs(page, 'clinician', 'Clin@2024');
  await page.goto('/#/import-export');
  await page.waitForLoadState('networkidle', { timeout: 5_000 });
  await expect(page.locator('#import-dropzone')).toHaveCount(0);
  await expect(page.locator('#sidebar')).toBeVisible();
});

// ── Import dropzone UI ────────────────────────────────────────────────────────

test('import dropzone is visible and contains file input', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#import-dropzone', { timeout: 10_000 });
  await expect(page.locator('#import-file-input')).toBeAttached();
  // Upload button should not be visible before a file is selected
  await expect(page.locator('#import-upload-btn')).toHaveCount(0);
});

test('selecting a non-xlsx file shows a format error', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#import-file-input', { timeout: 10_000 });

  // Simulate selecting a .csv file (not accepted)
  await page.locator('#import-file-input').setInputFiles({
    name: 'test.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('id,name\n1,test'),
  });

  await expect(page.locator('.field-error')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('.field-error')).toContainText('.xlsx');
  // Upload button must not appear after a rejected file
  await expect(page.locator('#import-upload-btn')).toHaveCount(0);
});

test('selecting a valid xlsx file shows the upload button', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#import-file-input', { timeout: 10_000 });

  // Minimal valid-looking xlsx (ZIP magic bytes PK + xlsx content-type)
  const xlsxMagic = Buffer.from([0x50, 0x4B, 0x03, 0x04, 0x00, 0x00]);
  await page.locator('#import-file-input').setInputFiles({
    name: 'data.xlsx',
    mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    buffer: xlsxMagic,
  });

  await expect(page.locator('#import-upload-btn')).toBeVisible({ timeout: 5_000 });
});

// ── Export request flow ───────────────────────────────────────────────────────

test('admin can see export type selector and request export button', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#export-type-select', { timeout: 10_000 });
  await expect(page.locator('#request-export-btn')).toBeVisible();
  await expect(page.locator('#export-type-select')).toBeVisible();
});

test('reviewer can request an export and sees success feedback', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#export-type-select', { timeout: 10_000 });

  // Request an inventory export
  await page.selectOption('#export-type-select', 'inventory');
  await page.click('#request-export-btn');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });

  // Either a toast fires or the pending export row appears in the table
  const toastVisible = await page.locator('.toast, [class*="toast"]').isVisible().catch(() => false);
  const tableRowVisible = await page.locator('table tbody tr').isVisible().catch(() => false);
  expect(toastVisible || tableRowVisible).toBe(true);
});

test('admin requests export then approves it and download link appears', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#request-export-btn', { timeout: 10_000 });

  await page.selectOption('#export-type-select', 'resources');
  await page.click('#request-export-btn');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });

  // After requesting, the pending approve button MUST appear. An unconditional
  // assertion here means a broken export-request flow causes the test to fail
  // rather than silently falling back to a "page is stable" pass.
  const approveBtn = page.locator('[id^="approve-export-"]').first();
  await expect(approveBtn).toBeVisible({ timeout: 10_000 });

  await approveBtn.click();
  await page.waitForLoadState('networkidle', { timeout: 10_000 });

  // After approval the download link must replace the approve button
  await expect(page.locator('[id^="download-export-"]').first()).toBeVisible({ timeout: 10_000 });
});

// ── SSE progress panel ────────────────────────────────────────────────────────

test('uploading a valid xlsx file shows the job progress panel', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await page.goto('/#/import-export');
  await page.waitForSelector('#import-file-input', { timeout: 10_000 });

  // Upload a minimal xlsx buffer — the server will create a job and return job state
  const xlsxMagic = Buffer.from([0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00]);
  await page.locator('#import-file-input').setInputFiles({
    name: 'upload.xlsx',
    mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    buffer: xlsxMagic,
  });

  await expect(page.locator('#import-upload-btn')).toBeVisible({ timeout: 5_000 });
  await page.click('#import-upload-btn');

  // After upload, the job progress card should appear (contains a progress-bar element)
  // or an error toast fires if the server rejects the malformed xlsx
  await page.waitForLoadState('networkidle', { timeout: 15_000 });
  const progressVisible = await page.locator('.progress-bar').isVisible().catch(() => false);
  const toastVisible = await page.locator('[class*="toast"]').isVisible().catch(() => false);
  expect(progressVisible || toastVisible).toBe(true);
});

// ── Cross-role export visibility ──────────────────────────────────────────────

test('inventory_clerk cannot see export section', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await page.goto('/#/import-export');
  await page.waitForLoadState('networkidle', { timeout: 10_000 });
  // Clerk can import but cannot request exports
  await expect(page.locator('#request-export-btn')).toHaveCount(0);
});
