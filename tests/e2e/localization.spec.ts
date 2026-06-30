import { expect, test } from '@playwright/test';
import { attachDiagnostics, attachDiagnosticsToTestInfo } from './utils/diagnostics';

const mojibakePattern = /Ã|áº|Ä|Æ|»|¼|½|¾/;

test('Vietnamese UI labels render with valid UTF-8 text', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);

  await page.goto('/');

  await expect(page.getByRole('button', { name: 'Lộ trình mới' })).toBeVisible();
  await expect(page.getByRole('navigation', { name: 'Điều hướng chính' })).toBeVisible();
  await expect(page.getByRole('heading', { name: 'Gần đây' })).toBeVisible();
  await expect(page.getByRole('button', { name: 'Đăng xuất' })).toBeVisible();
  await expect(page.getByText(mojibakePattern)).toHaveCount(0);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});

test('learning data pages render Vietnamese labels without mojibake', async ({ page }, testInfo) => {
  const diagnostics = attachDiagnostics(page);

  await page.goto('/notes');
  await expect(page.getByRole('heading', { name: 'Ghi chú học tập' })).toBeVisible();
  await expect(page.getByPlaceholder('Tìm ghi chú')).toBeVisible();
  await expect(page.getByText(mojibakePattern)).toHaveCount(0);

  await page.goto('/review');
  await expect(page.getByRole('heading', { name: 'Ôn tập hôm nay' })).toBeVisible();
  await expect(page.getByText(mojibakePattern)).toHaveCount(0);

  await page.goto('/resources');
  await expect(page.getByRole('heading', { name: 'Kho tài liệu' }).first()).toBeVisible();
  await expect(page.getByPlaceholder('Tìm tài liệu')).toBeVisible();
  await expect(page.getByText(mojibakePattern)).toHaveCount(0);

  await attachDiagnosticsToTestInfo(testInfo, diagnostics);
});
