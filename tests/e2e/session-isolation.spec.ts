import { expect, test, type APIRequestContext } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

async function createGatewaySession(request: APIRequestContext, userId: string, title: string) {
  const response = await request.post('/api/agent-gateway/sessions', {
    headers: {
      'X-User-ID': userId,
    },
    data: { title },
  });
  expect(response.ok()).toBeTruthy();
}

test('sidebar recent chats are scoped to the authenticated user', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const ownTitle = `Own session ${Date.now()}`;
  const otherTitle = `Other session ${Date.now()}`;

  await createGatewaySession(request, 'dev-learner', ownTitle);
  await createGatewaySession(request, 'other-user', otherTitle);

  await page.goto('/');

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByText(ownTitle)).toBeVisible();
  await expect(page.getByText(otherTitle)).toHaveCount(0);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
