import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';
import { login } from './utils/auth';
import { ROUTES } from './utils/env';

test('user can login through the browser', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);

  await login(page);

  await expect(page).not.toHaveURL(new RegExp(`${ROUTES.login}$`));
  await expect(page.getByRole('heading').first()).toBeVisible();

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
