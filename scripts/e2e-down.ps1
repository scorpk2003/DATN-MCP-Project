$ErrorActionPreference = "Stop"

docker compose -f docker-compose.yml -f docker-compose.e2e.yml down -v
