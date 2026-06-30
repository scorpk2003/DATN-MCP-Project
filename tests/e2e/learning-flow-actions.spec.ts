import { expect, test, type APIRequestContext, type Page, type Response } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

const databaseMcpUrl = process.env.E2E_DATABASE_MCP_URL ?? 'http://127.0.0.1:3101';

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

async function seedRoadmap(request: APIRequestContext, title: string) {
  const response = await request.post(`${databaseMcpUrl}/test/roadmaps/seed`, {
    data: {
      userId: 'dev-learner',
      title,
    },
  });
  expect(response.ok()).toBeTruthy();
}

async function seedLearningData(request: APIRequestContext, title: string) {
  const response = await request.post(`${databaseMcpUrl}/test/learning-data/seed`, {
    data: {
      userId: 'dev-learner',
      title,
    },
  });
  expect(response.ok()).toBeTruthy();
}

async function expectLearningFlowRequest(page: Page, action: () => Promise<void>, expectedUrl: RegExp = /\/$/) {
  const sessionResponsePromise = page.waitForResponse(isCreateSessionResponse);
  const intentResponsePromise = page.waitForResponse(isSendIntentResponse);

  await action();

  const sessionResponse = await sessionResponsePromise;
  const intentResponse = await intentResponsePromise;
  expect(sessionResponse.ok()).toBeTruthy();
  expect(intentResponse.ok()).toBeTruthy();
  await expect(page).toHaveURL(expectedUrl);
}

function isCreateSessionResponse(response: Response) {
  const url = new URL(response.url());
  return response.request().method() === 'POST' && /\/sessions$/.test(url.pathname);
}

function isSendIntentResponse(response: Response) {
  const url = new URL(response.url());
  return response.request().method() === 'POST' && /\/sessions\/[^/]+\/intents$/.test(url.pathname);
}

test('roadmap task action starts a learning session through the gateway', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Action Roadmap ${Date.now()}`;

  await seedRoadmap(request, title);
  await page.goto('/roadmap');

  await expect(page.getByText(title)).toBeVisible();
  await expect(page.getByText('Database task appears on roadmap', { exact: true })).toBeVisible();

  await expectLearningFlowRequest(page, async () => {
    await page.getByRole('button', { name: 'Học' }).first().click();
  });

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});

test('roadmap schedule update stays on the roadmap page', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Schedule Roadmap ${Date.now()}`;

  await seedRoadmap(request, title);
  await page.goto('/roadmap');

  await expect(page.getByText(title)).toBeVisible();
  await expectLearningFlowRequest(page, async () => {
    await page.getByRole('button', { name: 'Update schedule' }).click();
  }, /\/roadmap$/);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});

test('notes review action starts a learning session through the gateway', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Action Notes ${Date.now()}`;

  await seedLearningData(request, title);
  await page.goto('/notes');

  await expect(page.getByText(`${title} note`).first()).toBeVisible();
  await page.getByRole('button', { name: new RegExp(`${escapeRegExp(title)} note`) }).click();

  await expectLearningFlowRequest(page, async () => {
    await page.getByRole('button', { name: 'Tạo review' }).click();
  });

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});

test('review queue action starts a learning session through the gateway', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Action Review ${Date.now()}`;

  await seedLearningData(request, title);
  await page.goto('/review');

  await expect(page.getByText(`${title} review task`)).toBeVisible();

  await expectLearningFlowRequest(page, async () => {
    await page.getByRole('button', { name: 'Bắt đầu ôn' }).first().click();
  });

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
