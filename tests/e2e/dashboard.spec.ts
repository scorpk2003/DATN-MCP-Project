import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';
import { ROUTES } from './utils/env';

test('authenticated user can open dashboard', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);

  await page.goto(ROUTES.dashboard);

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByRole('heading').first()).toBeVisible();

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
