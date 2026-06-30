import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

test('authenticated user can create a note from the notes page', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Manual note ${Date.now()}`;

  await page.goto('/notes');
  await expect(page).not.toHaveURL(/login/);

  await page.getByRole('button', { name: /Ghi chu moi/i }).click();
  await page.getByPlaceholder('Nhap noi dung ghi chu').fill(`${title}\nCreated from the notes page.`);
  await page.getByRole('button', { name: /Luu ghi chu/i }).click();

  await expect(page.getByText(title).first()).toBeVisible();
  expect(diagnostics.consoleMessages).toEqual([]);
  expect(diagnostics.networkErrors.filter((error) => error.includes('/notes'))).toEqual([]);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
