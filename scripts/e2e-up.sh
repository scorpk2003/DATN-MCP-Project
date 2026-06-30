#!/usr/bin/env bash
set -euo pipefail

docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d --build

echo "E2E stack started. Current services:"
docker compose -f docker-compose.yml -f docker-compose.e2e.yml ps
