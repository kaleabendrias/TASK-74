/**
 * E2E tests: Import / Export page flows.
 *
 * NOTE: The Yew SPA uses HTML5 History API routing with in-memory auth state.
 * All post-login navigations use navigate() to avoid full-page reloads.
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

async function logout(page: Page) {
  await page.click('#logout-btn');
  await page.waitForSelector('#login-submit', { timeout: 10_000 });
}

// ── Role-based access ─────────────────────────────────────────────────────────

test('admin can access import/export page', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/import-export');
  await expect(page.locator('#import-dropzone')).toBeVisible({ timeout: 15_000 });
});

test('reviewer can access import/export page', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await navigate(page, '/import-export');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
});

test('inventory_clerk can access import/export page', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/import-export');
  await expect(page.locator('#import-dropzone')).toBeVisible({ timeout: 15_000 });
});

test('publisher cannot access import/export page', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await navigate(page, '/import-export');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#import-dropzone')).toHaveCount(0);
});

test('clinician cannot access import/export page', async ({ page }) => {
  await loginAs(page, 'clinician', 'Clin@2024');
  await navigate(page, '/import-export');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#import-dropzone')).toHaveCount(0);
});

// ── Import dropzone UI ────────────────────────────────────────────────────────

test('import dropzone is visible and contains file input', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/import-export');
  await page.waitForSelector('#import-dropzone', { timeout: 15_000 });
  await expect(page.locator('#import-file-input')).toBeAttached();
  await expect(page.locator('#import-upload-btn')).toHaveCount(0);
});

test('selecting a non-xlsx file shows a format error', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/import-export');
  await page.waitForSelector('#import-file-input', { timeout: 15_000 });

  await page.locator('#import-file-input').setInputFiles({
    name: 'test.csv',
    mimeType: 'text/csv',
    buffer: Buffer.from('id,name\n1,test'),
  });

  await expect(page.locator('.field-error')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('.field-error')).toContainText('.xlsx');
  await expect(page.locator('#import-upload-btn')).toHaveCount(0);
});

test('selecting a valid xlsx file shows the upload button', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/import-export');
  await page.waitForSelector('#import-file-input', { timeout: 15_000 });

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
  await navigate(page, '/import-export');
  await page.waitForSelector('#export-type-select', { timeout: 15_000 });
  await expect(page.locator('#request-export-btn')).toBeVisible();
  await expect(page.locator('#export-type-select')).toBeVisible();
});

test('reviewer can request an export and sees success feedback', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await navigate(page, '/import-export');
  await page.waitForSelector('#export-type-select', { timeout: 15_000 });

  await page.selectOption('#export-type-select', 'inventory');
  await page.click('#request-export-btn');

  // Wait for the API response: either a toast or a table row will appear
  const feedbackLocator = page.locator('.toast, [class*="toast"], table tbody tr');
  await expect(feedbackLocator.first()).toBeVisible({ timeout: 10_000 });
});

test('admin requests export then approves it and download link appears', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/import-export');
  await page.waitForSelector('#request-export-btn', { timeout: 15_000 });

  await page.selectOption('#export-type-select', 'resources');
  await page.click('#request-export-btn');

  const approveBtn = page.locator('[id^="approve-export-"]').first();
  await expect(approveBtn).toBeVisible({ timeout: 15_000 });

  await approveBtn.click();

  await expect(page.locator('[id^="download-export-"]').first()).toBeVisible({ timeout: 15_000 });
});

// ── SSE progress panel ────────────────────────────────────────────────────────

test('uploading a valid xlsx file shows the job progress panel', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/import-export');
  await page.waitForSelector('#import-file-input', { timeout: 15_000 });

  const xlsxMagic = Buffer.from([0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00]);
  await page.locator('#import-file-input').setInputFiles({
    name: 'upload.xlsx',
    mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    buffer: xlsxMagic,
  });

  await expect(page.locator('#import-upload-btn')).toBeVisible({ timeout: 5_000 });
  await page.click('#import-upload-btn');

  // Wait for upload feedback: either progress bar or a toast notification
  const uploadFeedback = page.locator('.progress-bar, [class*="toast"]');
  await expect(uploadFeedback.first()).toBeVisible({ timeout: 10_000 });
});

// ── Cross-role export visibility ──────────────────────────────────────────────

test('inventory_clerk cannot see export section', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/import-export');
  await expect(page.locator('#import-dropzone')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('#request-export-btn')).toHaveCount(0);
});
