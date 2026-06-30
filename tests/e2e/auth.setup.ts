import { test as setup, expect } from '@playwright/test';
import fs from 'node:fs';
import path from 'node:path';
import { login } from './utils/auth';
import { E2E_USER, ROUTES } from './utils/env';

const authFile = 'playwright/.auth/user.json';
const databaseMcpUrl = process.env.E2E_DATABASE_MCP_URL ?? 'http://127.0.0.1:3101';

setup('authenticate E2E user', async ({ page, request }) => {
  fs.mkdirSync(path.dirname(authFile), { recursive: true });

  const resetResponse = await request.post(`${databaseMcpUrl}/test/reset`, {
    data: { userId: E2E_USER.id },
  });
  expect(resetResponse.ok()).toBeTruthy();

  await login(page);

  await expect(page).not.toHaveURL(new RegExp(`${ROUTES.login}$`));
  await page.context().storageState({ path: authFile });
});
