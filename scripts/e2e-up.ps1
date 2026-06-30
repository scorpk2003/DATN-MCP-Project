$ErrorActionPreference = "Stop"

docker compose -f docker-compose.yml -f docker-compose.e2e.yml up -d --build

Write-Host "E2E stack started. Current services:"
docker compose -f docker-compose.yml -f docker-compose.e2e.yml ps
