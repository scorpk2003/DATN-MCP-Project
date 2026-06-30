# Codex Agent Instructions

This repository is a full-stack application. For UI, gateway, backend, MCP server, database, roadmap, lesson, or orchestrator changes, Codex must validate behavior through the browser, not only through unit tests.

## Primary workflow

1. Start the full E2E stack.
2. Reset and seed E2E data.
3. Run Playwright tests.
4. Inspect screenshots, traces, browser console errors, failed network requests, and Docker logs on failure.
5. Apply the smallest root-cause fix.
6. Add or update a regression test.
7. Re-run the failed test.

## Commands

Start stack:

```bash
bash scripts/e2e-up.sh
```

Seed data:

```bash
bash scripts/seed-e2e-db.sh
```

Run E2E tests:

```bash
npx playwright test
```

Run one file:

```bash
npx playwright test tests/e2e/auth.spec.ts
```

View report:

```bash
npx playwright show-report
```

Stop stack:

```bash
bash scripts/e2e-down.sh
```

## Default E2E account

```txt
email: test@example.com
password: password123
```

## Important rules

- Do not use brittle CSS selectors if a role, label, text, or test id can be used.
- Prefer user-visible assertions.
- Do not weaken assertions just to make tests pass.
- Do not claim a UI bug is fixed without browser verification.
- If a test fails, inspect network and console evidence before editing code.
- If the orchestrator flow fails, inspect gateway, orchestrator, MCP, and database logs.

## Skill files

See `.agents/skills/` for detailed operating procedures.
