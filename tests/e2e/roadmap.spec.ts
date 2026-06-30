import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';
import { ROUTES } from './utils/env';

const databaseMcpUrl = process.env.E2E_DATABASE_MCP_URL ?? 'http://127.0.0.1:3101';

test('authenticated user can inspect the persisted database roadmap', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const dataRequests: string[] = [];
  const roadmapTitle = `Persisted DB Roadmap ${Date.now()}`;

  const seedResponse = await request.post(`${databaseMcpUrl}/test/roadmaps/seed`, {
    data: {
      userId: 'dev-learner',
      title: roadmapTitle,
    },
  });
  expect(seedResponse.ok()).toBeTruthy();

  page.on('request', request => {
    const url = new URL(request.url());
    if (url.pathname.includes('roadmap')) {
      dataRequests.push(url.pathname);
    }
  });

  await page.goto(ROUTES.roadmap);

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByText('Not Found')).toHaveCount(0);
  await expect(page.getByText(roadmapTitle)).toBeVisible();
  await expect(page.getByText('Database task appears on roadmap', { exact: true })).toBeVisible();
  await expect(page.getByText(/AI Engineer/i)).toHaveCount(0);
  expect(dataRequests).toContain('/roadmap');
  expect(dataRequests).not.toContain('/api/roadmap');
  expect(diagnostics.networkErrors.filter(error => error.includes('roadmap'))).toEqual([]);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
