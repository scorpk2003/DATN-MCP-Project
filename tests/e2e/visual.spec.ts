import { expect, test } from '@playwright/test';
import { ROUTES } from './utils/env';

test('login route renders a stable first screen', async ({ page }, testInfo) => {
  await page.goto(ROUTES.login);
  await expect(page.getByRole('heading').first()).toBeVisible();
  await testInfo.attach('first-screen.png', {
    body: await page.screenshot({
      animations: 'disabled',
      fullPage: true,
    }),
    contentType: 'image/png',
  });
});
