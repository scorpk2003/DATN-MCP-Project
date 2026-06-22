param(
  [string]$BaseUrl = "http://127.0.0.1:3200",
  [string]$SourceCatalog = "config/official_source_catalog.yaml",
  [string]$SeedManifest = "config/official_topic_seed_manifest.yaml",
  [string]$TopicManifest = "config/evaluation_topic_manifest.yaml",
  [bool]$DryRun = $true,
  [bool]$VerifyUrls = $true
)

cargo run --bin seed_official_corpus -- `
  --base-url $BaseUrl `
  --source-catalog $SourceCatalog `
  --seed-manifest $SeedManifest `
  --topic-manifest $TopicManifest `
  --dry-run $DryRun `
  --verify-urls $VerifyUrls
