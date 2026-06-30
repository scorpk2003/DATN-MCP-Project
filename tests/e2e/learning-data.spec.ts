import { expect, test, type APIRequestContext } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

const databaseMcpUrl = process.env.E2E_DATABASE_MCP_URL ?? 'http://127.0.0.1:3101';

async function seedLearningData(request: APIRequestContext, title: string) {
  const seedResponse = await request.post(`${databaseMcpUrl}/test/learning-data/seed`, {
    data: {
      userId: 'dev-learner',
      title,
    },
  });
  expect(seedResponse.ok()).toBeTruthy();
}

test('notes page renders persisted database notes', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Persisted note ${Date.now()}`;

  await seedLearningData(request, title);
  await page.goto('/notes');

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByText('Not Found')).toHaveCount(0);
  await expect(page.getByText(`${title} review task`).first()).toBeVisible();
  await expect(page.getByText(`${title} note`).first()).toBeVisible();
  await expect(page.getByText('Cosine similarity')).toHaveCount(0);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});

test('review page renders persisted database review queue', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Persisted review ${Date.now()}`;

  await seedLearningData(request, title);
  await page.goto('/review');

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByText('Not Found')).toHaveCount(0);
  await expect(page.getByText(`${title} review task`)).toBeVisible();
  await expect(page.getByText(`${title} project`)).toBeVisible();
  await expect(page.getByText('Cosine similarity')).toHaveCount(0);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
