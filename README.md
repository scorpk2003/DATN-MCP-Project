# Codex Playwright E2E Scaffold

This scaffold adds agent skills and Playwright E2E files so Codex can test a full-stack app like a real user.

## Copy into your repo

Copy these paths into your project root:

```txt
.agents/skills/
AGENTS.md
playwright.config.ts
tests/e2e/
scripts/e2e-up.sh
scripts/e2e-down.sh
scripts/e2e-logs.sh
scripts/seed-e2e-db.sh
docker-compose.e2e.yml
```

Then install Playwright:

```bash
npm i -D @playwright/test
npx playwright install
```

Add scripts from `package.e2e.example.json` into your real `package.json`.

## Customize before first run

Update these based on your real app:

- Service names in `docker-compose.e2e.yml`.
- Ports and health endpoints.
- `VITE_AGENT_GATEWAY_URL` or equivalent frontend env vars.
- Seed commands in `scripts/seed-e2e-db.sh`.
- Route defaults in `tests/e2e/utils/env.ts`.
- Test locators if your labels/buttons use different names.

## Recommended first run

```bash
npm run e2e:up
npm run seed:e2e
npm run test:e2e
```

If tests fail:

```bash
npm run e2e:logs
npm run test:e2e:report
```

## Important note

The test files are intentionally generic. They are meant as a starting point. Replace route names, labels, and assertions with your actual UI contract.
