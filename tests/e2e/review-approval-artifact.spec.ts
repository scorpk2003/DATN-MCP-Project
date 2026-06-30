import { expect, test, type APIRequestContext } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

const databaseMcpUrl = process.env.E2E_DATABASE_MCP_URL ?? 'http://127.0.0.1:3101';

async function seedLearningData(request: APIRequestContext, title: string) {
  const response = await request.post(`${databaseMcpUrl}/test/learning-data/seed`, {
    data: {
      userId: 'dev-learner',
      title,
    },
  });
  expect(response.ok()).toBeTruthy();
}

test('review approval renders the resumed lesson artifact', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Review Approval ${Date.now()}`;

  await seedLearningData(request, title);
  await page.goto('/review');

  await expect(page.getByText(`${title} review task`)).toBeVisible();
  await page.getByRole('main').getByRole('button').first().click();

  await expect(page).toHaveURL(/\/$/);
  await expect(page.getByRole('heading', { name: /action required/i })).toBeVisible({ timeout: 30_000 });
  await page.getByRole('button', { name: /approve/i }).click();

  await expect(page.getByText(`${title} review task review`)).toBeVisible({ timeout: 30_000 });
  await expect(page.getByText(/did not match a supported UI artifact/i)).toHaveCount(0);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
