import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

const resourceApiUrl = process.env.E2E_RESOURCE_API_URL ?? 'http://127.0.0.1:3200';

test('resources page renders seeded resource service data without fallback resources', async ({ page, request }, testInfo) => {
  const diagnostics = attachDiagnostics(page);
  const title = `Seeded PostgreSQL indexing guide ${Date.now()}`;
  const canonicalUrl = `http://127.0.0.1:5174/resource-open-${Date.now()}`;

  const seedResponse = await request.post(`${resourceApiUrl}/resources`, {
    data: {
      canonicalUrl,
      title,
      summary: 'Seeded resource from the resource service.',
      description: 'Seeded resource from the resource service.',
      resourceType: 'docs',
      resourceFormat: 'html',
      language: 'en',
    },
  });
  expect(seedResponse.ok()).toBeTruthy();

  await page.goto('/resources');

  await expect(page).not.toHaveURL(/login/);
  await expect(page.getByText(title).first()).toBeVisible();
  await expect(page.getByText('RAG evaluation checklist')).toHaveCount(0);
  await expect(page.getByText('Not Found')).toHaveCount(0);

  const popupPromise = page.waitForEvent('popup');
  await page.locator('aside button').last().click();
  const popup = await popupPromise;
  await expect(popup).toHaveURL(canonicalUrl);
  await popup.close();

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
