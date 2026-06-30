import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';
import { ROUTES } from './utils/env';

test('user can ask orchestrator for a learning plan', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const prompt = 'Create a beginner roadmap for learning PostgreSQL indexing.';

  await page.goto(ROUTES.orchestrator);

  const messageBox = page.getByRole('textbox').or(page.getByLabel(/message|prompt|ask/i));
  await expect(messageBox).toBeVisible();
  await messageBox.fill(prompt);

  await page.getByRole('button', { name: 'Run agent' }).click();

  await expect(page.getByRole('heading', { name: /action required/i })).toBeVisible({ timeout: 30_000 });
  await page.getByRole('button', { name: /approve/i }).click();
  await expect(page.getByText(/CCNA standard learner roadmap/i)).toBeVisible({ timeout: 90_000 });

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
