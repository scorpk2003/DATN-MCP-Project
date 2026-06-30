#!/usr/bin/env bash
set -euo pipefail

docker compose -f docker-compose.yml -f docker-compose.e2e.yml logs --tail=200 "$@"
