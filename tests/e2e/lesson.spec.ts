import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';
import { ROUTES } from './utils/env';

test('user can open a lesson page', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);

  await page.goto(ROUTES.lesson);

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByRole('heading').first()).toBeVisible();
  await expect(page.getByText(/lesson|content|overview|objective/i).first()).toBeVisible({ timeout: 30_000 });

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
