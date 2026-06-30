import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

const protectedRoutes = ['/', '/roadmap', '/review', '/notes', '/resources'];

for (const route of protectedRoutes) {
  test(`sidebar renders navigation on ${route}`, async ({ page }, testInfo) => {
    const diagnostics = attachDiagnostics(page);

    await page.goto(route);

    await expect(page).not.toHaveURL(/login/);
    await expect(page.getByText('Not Found')).toHaveCount(0);
    await expect(page.locator('a[href="/"]')).toBeVisible();
    await expect(page.locator('a[href="/roadmap"]')).toBeVisible();
    await expect(page.locator('a[href="/review"]')).toBeVisible();
    await expect(page.locator('a[href="/notes"]')).toBeVisible();
    await expect(page.locator('a[href="/resources"]')).toBeVisible();

    await attachDiagnosticsToTestInfo(testInfo, diagnostics);
  });
}

