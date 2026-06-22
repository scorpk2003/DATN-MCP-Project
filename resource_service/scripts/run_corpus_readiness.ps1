param(
  [string]$BaseUrl = "http://127.0.0.1:3200",
  [string]$Manifest = "config/evaluation_topic_manifest.yaml",
  [string]$Out = "reports/corpus_readiness_report.json"
)

cargo run --bin run_corpus_readiness -- `
  --base-url $BaseUrl `
  --manifest $Manifest `
  --out $Out
