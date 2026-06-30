#!/usr/bin/env bash
set -euo pipefail

# Customize this script for your real database/service commands.
# The command should be idempotent and safe for E2E only.

if npm run | grep -q "db:e2e:reset"; then
  npm run db:e2e:reset
fi

if npm run | grep -q "db:e2e:seed"; then
  npm run db:e2e:seed
else
  echo "No global E2E seed command found. E2E specs seed their own deterministic data through test routes."
fi

database_mcp_url="${E2E_DATABASE_MCP_URL:-http://127.0.0.1:3101}"
user_id="${E2E_USER_ID:-dev-learner}"
curl -fsS -X POST "$database_mcp_url/test/reset" \
  -H "Content-Type: application/json" \
  -d "{\"userId\":\"$user_id\"}" >/dev/null
echo "Reset E2E data for user '$user_id'."
