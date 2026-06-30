import { expect, type Page } from '@playwright/test';
import { E2E_AUTH, E2E_USER, ROUTES } from './env';

export async function login(page: Page) {
  await page.goto(ROUTES.login);

  if (E2E_AUTH.devAuth) {
    await expect(page).not.toHaveURL(new RegExp(`${ROUTES.login}$`));
    return;
  }

  await page.getByLabel(/email/i).fill(E2E_USER.email);
  await page.getByLabel(/password/i).fill(E2E_USER.password);
  await page.getByRole('button', { name: /log in|login|sign in/i }).click();

  await expect(page).not.toHaveURL(new RegExp(`${ROUTES.login}$`));
}
