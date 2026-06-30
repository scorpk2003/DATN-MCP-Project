$ErrorActionPreference = "Stop"

$scripts = (npm run --silent) -join "`n"

if ($scripts -match "db:e2e:reset") {
  npm run db:e2e:reset
}

if ($scripts -match "db:e2e:seed") {
  npm run db:e2e:seed
} else {
  Write-Host "No global E2E seed command found. E2E specs seed their own deterministic data through test routes."
}

$databaseMcpUrl = if ($env:E2E_DATABASE_MCP_URL) { $env:E2E_DATABASE_MCP_URL } else { "http://127.0.0.1:3101" }
$userId = if ($env:E2E_USER_ID) { $env:E2E_USER_ID } else { "dev-learner" }
$body = @{ userId = $userId } | ConvertTo-Json -Compress

Invoke-RestMethod -Method Post -Uri "$databaseMcpUrl/test/reset" -ContentType "application/json" -Body $body | Out-Null
Write-Host "Reset E2E data for user '$userId'."
