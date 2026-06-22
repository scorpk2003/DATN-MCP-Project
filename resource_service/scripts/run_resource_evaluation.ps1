param(
  [string]$BaseUrl = "http://127.0.0.1:3200",
  [string]$Manifest = "config/evaluation_topic_manifest.yaml",
  [string]$Out = "reports/resource_eval_report.json",
  [switch]$AllowNotReady
)

$cargoArgs = @(
  "run",
  "--bin",
  "run_resource_evaluation",
  "--",
  "--base-url",
  $BaseUrl,
  "--manifest",
  $Manifest,
  "--out",
  $Out
)
if ($AllowNotReady) {
  $cargoArgs += "--allow-not-ready"
}

& cargo @cargoArgs
