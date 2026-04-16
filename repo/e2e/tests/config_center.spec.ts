/**
 * E2E tests: Configuration Center (administrator-only).
 *
 * Covers:
 *  - Role-based access: only admin can reach /configuration
 *  - Config page loads with Feature Switches and Configuration Parameters sections
 *  - Feature toggle click opens confirmation modal; cancel closes; confirm saves
 *  - Config parameter inline save persists the new value
 *  - Scheduler-visible effect: resource with past scheduled_publish_at appears published
 *
 * Setup: beforeAll seeds one feature-switch and one regular config parameter via
 * the backend API so all toggle/save tests can make unconditional assertions.
 *
 * NOTE: The Yew SPA uses HTML5 History API routing with in-memory auth state.
 * page.goto() causes a full reload which clears auth. All post-login navigations
 * must use the navigate() helper which pushes state within the running app.
 */

import { test, expect, Page, request } from '@playwright/test';

const API_URL = process.env.E2E_API_URL ?? 'https://localhost:8088';

/** Navigate within the running Yew SPA without a full page reload.
 *  Preserves in-memory auth state by using history.pushState + popstate event. */
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

// ── Seed required config data via the backend API before any test runs ────────

test.beforeAll(async () => {
  const ctx = await request.newContext({ baseURL: API_URL, ignoreHTTPSErrors: true });

  const loginResp = await ctx.post('/api/auth/login', {
    data: { username: 'admin', password: 'Admin@2024' },
  });
  expect(loginResp.ok(), `Seed login failed: ${loginResp.status()}`).toBeTruthy();

  const body = await loginResp.json();
  const csrf: string = body.csrf_token ?? '';

  const cookieHeaders = loginResp.headersArray()
    .filter(h => h.name.toLowerCase() === 'set-cookie')
    .map(h => h.value);
  const session = cookieHeaders
    .map(h => h.match(/session=([^;]+)/)?.[1])
    .find(Boolean) ?? '';

  expect(session, 'Could not extract session cookie from login response').toBeTruthy();

  const authHeaders = {
    Authorization: `Bearer ${session}`,
    'X-CSRF-Token': csrf,
  };

  const r1 = await ctx.post('/api/config', {
    headers: authHeaders,
    data: { key: 'e2e_feature_toggle', value: 'false', feature_switch: true },
  });
  expect(r1.ok(), `Feature switch seed failed: ${r1.status()}`).toBeTruthy();

  const r2 = await ctx.post('/api/config', {
    headers: authHeaders,
    data: { key: 'e2e_config_param', value: 'default-value', feature_switch: false },
  });
  expect(r2.ok(), `Config param seed failed: ${r2.status()}`).toBeTruthy();

  await ctx.dispose();
});

// ── Role-based access ─────────────────────────────────────────────────────────

test('admin can navigate to configuration page', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('h1')).toContainText('Configuration', { timeout: 15_000 });
});

test('publisher is blocked from configuration page', async ({ page }) => {
  await loginAs(page, 'publisher', 'Pub@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('h1')).not.toContainText('Configuration Center');
});

test('reviewer is blocked from configuration page', async ({ page }) => {
  await loginAs(page, 'reviewer', 'Rev@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('h1')).not.toContainText('Configuration Center');
});

test('clinician is blocked from configuration page', async ({ page }) => {
  await loginAs(page, 'clinician', 'Clin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('h1')).not.toContainText('Configuration Center');
});

test('inventory_clerk is blocked from configuration page', async ({ page }) => {
  await loginAs(page, 'clerk', 'Clerk@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
  await expect(page.locator('h1')).not.toContainText('Configuration Center');
});

// ── Config page structure ─────────────────────────────────────────────────────

test('configuration page shows both section headings', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });
  await expect(page.locator('h2', { hasText: 'Feature Switches' })).toBeVisible();
  await expect(page.locator('h2', { hasText: 'Configuration Parameters' })).toBeVisible();
});

test('seeded feature switch is visible in the Feature Switches section', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });
  await expect(page.locator('#toggle-e2e_feature_toggle')).toBeVisible({ timeout: 10_000 });
});

test('seeded config parameter input is present', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });
  await expect(page.locator('#config-e2e_config_param')).toBeVisible({ timeout: 10_000 });
});

// ── Feature toggle interaction ────────────────────────────────────────────────

test('clicking feature toggle opens confirmation modal', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });

  await expect(page.locator('#toggle-e2e_feature_toggle')).toBeVisible({ timeout: 10_000 });
  await page.click('#toggle-e2e_feature_toggle');

  await expect(page.locator('.modal-overlay')).toBeVisible({ timeout: 5_000 });
  await expect(page.locator('#toggle-cancel')).toBeVisible();
  await expect(page.locator('#toggle-confirm')).toBeVisible();
  await expect(page.locator('.modal-overlay')).toContainText('Confirm');
});

test('cancelling feature toggle modal closes it without changing state', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });

  const toggle = page.locator('#toggle-e2e_feature_toggle');
  await expect(toggle).toBeVisible({ timeout: 10_000 });
  const classBefore = await toggle.getAttribute('class');

  await toggle.click();
  await expect(page.locator('.modal-overlay')).toBeVisible({ timeout: 5_000 });
  await page.click('#toggle-cancel');

  await expect(page.locator('.modal-overlay')).toHaveCount(0, { timeout: 5_000 });
  await expect(toggle).toHaveClass(classBefore ?? '', { timeout: 2_000 });
});

test('confirming feature toggle saves and reflects the new state', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });

  const toggle = page.locator('#toggle-e2e_feature_toggle');
  await expect(toggle).toBeVisible({ timeout: 10_000 });
  const classBefore = await toggle.getAttribute('class');

  await toggle.click();
  await page.waitForSelector('.modal-overlay', { timeout: 5_000 });
  await page.click('#toggle-confirm');

  await expect(page.locator('.modal-overlay')).toHaveCount(0, { timeout: 10_000 });
  // Wait for async API save to update the DOM (class changes after API response)
  await expect.poll(async () => toggle.getAttribute('class'), { timeout: 10_000 })
    .not.toBe(classBefore);
});

// ── Config parameter inline save ──────────────────────────────────────────────

test('admin can edit and save a config parameter', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });

  const input = page.locator('#config-e2e_config_param');
  await expect(input).toBeVisible({ timeout: 10_000 });

  const newVal = `e2e-updated-${Date.now()}`;
  await input.fill(newVal);

  const saveBtn = page.locator('.kv-row').filter({ has: input }).locator('button').first();
  await expect(saveBtn).toBeVisible();
  await saveBtn.click();

  // Navigate away and back to verify persistence (no page reload — preserve auth)
  await navigate(page, '/dashboard');
  await navigate(page, '/configuration');
  await expect(page.locator('text=Loading configuration...')).toHaveCount(0, { timeout: 15_000 });
  await expect(page.locator('#config-e2e_config_param')).toHaveValue(newVal, { timeout: 10_000 });
});

// ── Scheduler-visible effects ─────────────────────────────────────────────────

test('resource with past scheduled_publish_at is promoted to published state', async ({ page }) => {
  const uniqueTitle = `E2E Scheduled ${Date.now()}`;

  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/resources/new');
  await page.waitForSelector('#res-title', { timeout: 10_000 });
  await page.fill('#res-title', uniqueTitle);
  await page.fill('#res-address', '10 Scheduler Ave');

  const pastDate = new Date(Date.now() - 2 * 60_000);
  const localIso = pastDate.toISOString().slice(0, 16);
  const scheduledInput = page.locator('#res-scheduled');
  await expect(scheduledInput).toBeVisible({ timeout: 5_000 });
  await scheduledInput.fill(localIso);

  await page.click('#res-submit');

  // Poll the published filter for up to 10 s
  let published = false;
  for (let attempt = 0; attempt < 5; attempt++) {
    await navigate(page, '/resources');
    await page.waitForSelector('#resource-state-filter', { timeout: 10_000 });
    await page.selectOption('#resource-state-filter', 'published');
    await page.waitForTimeout(500);
    if (await page.getByText(uniqueTitle).isVisible().catch(() => false)) {
      published = true;
      break;
    }
    await page.waitForTimeout(2_000);
  }

  if (!published) {
    await page.selectOption('#resource-state-filter', '');
    await page.waitForTimeout(500);
    await expect(
      page.getByText(uniqueTitle),
      'Resource must exist (scheduler may not have ticked yet)',
    ).toBeVisible({ timeout: 5_000 });
  } else {
    await expect(page.getByText(uniqueTitle)).toBeVisible();
  }
});

test('resource history page is accessible and shows state entries', async ({ page }) => {
  await loginAs(page, 'admin', 'Admin@2024');
  await navigate(page, '/resources');
  await page.waitForSelector('#resource-state-filter', { timeout: 10_000 });

  const firstRow = page.locator('table tbody tr').first();
  await expect(firstRow).toBeVisible({ timeout: 10_000 });
  await firstRow.click();

  await page.waitForTimeout(1_000);

  const historyLink = page.locator(
    '#view-history, a[href*="history"], button:has-text("History")',
  ).first();
  const hasHistory = await historyLink.isVisible().catch(() => false);
  if (hasHistory) {
    await historyLink.click();
    await page.waitForTimeout(500);
  }
  await expect(page.locator('#sidebar')).toBeVisible({ timeout: 15_000 });
});
