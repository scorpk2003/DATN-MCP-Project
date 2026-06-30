import type { Page, TestInfo } from '@playwright/test';

export type Diagnostics = {
  consoleMessages: string[];
  networkErrors: string[];
};

export function attachDiagnostics(page: Page): Diagnostics {
  const diagnostics: Diagnostics = {
    consoleMessages: [],
    networkErrors: [],
  };

  page.on('console', message => {
    if (['error', 'warning'].includes(message.type())) {
      diagnostics.consoleMessages.push(`[${message.type()}] ${message.text()}`);
    }
  });

  page.on('pageerror', error => {
    diagnostics.consoleMessages.push(`[pageerror] ${error.message}`);
  });

  page.on('requestfailed', request => {
    diagnostics.networkErrors.push(
      `[requestfailed] ${request.method()} ${request.url()} :: ${request.failure()?.errorText ?? 'unknown error'}`,
    );
  });

  page.on('response', response => {
    const status = response.status();
    if (status >= 400) {
      diagnostics.networkErrors.push(`[response] ${status} ${response.url()}`);
    }
  });

  return diagnostics;
}

export async function attachDiagnosticsToTestInfo(testInfo: TestInfo, diagnostics: Diagnostics) {
  if (diagnostics.consoleMessages.length > 0) {
    await testInfo.attach('browser-console.log', {
      body: diagnostics.consoleMessages.join('\n'),
      contentType: 'text/plain',
    });
  }

  if (diagnostics.networkErrors.length > 0) {
    await testInfo.attach('network-errors.log', {
      body: diagnostics.networkErrors.join('\n'),
      contentType: 'text/plain',
    });
  }
}
