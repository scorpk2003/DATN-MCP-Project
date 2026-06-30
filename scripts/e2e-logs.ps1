$ErrorActionPreference = "Stop"

docker compose -f docker-compose.yml -f docker-compose.e2e.yml logs --tail=200 @args
