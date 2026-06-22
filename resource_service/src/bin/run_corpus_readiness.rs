use std::{env, fs, path::Path};

use resource_service::{
    AppConfig, AppError, AppResult, ResourceService,
    corpus::{build_readiness_report, load_evaluation_manifest},
    create_pool,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let args = Args::parse()?;
    let manifest = load_evaluation_manifest(&args.manifest)?;
    let config = AppConfig::from_env();
    let pool = create_pool(&config)?;
    let service = ResourceService::new(pool, config);
    let report = build_readiness_report(&service, &manifest).await?;
    let output = serde_json::to_string_pretty(&report).expect("report serializes");
    ensure_parent(&args.out)?;
    fs::write(&args.out, &output)
        .map_err(|err| AppError::Internal(format!("failed to write {}: {err}", args.out)))?;
    println!("{output}");
    Ok(())
}

struct Args {
    manifest: String,
    out: String,
}

impl Args {
    fn parse() -> AppResult<Self> {
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            manifest: "config/evaluation_topic_manifest.yaml".to_string(),
            out: "reports/corpus_readiness_report.json".to_string(),
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => {
                    parsed.manifest = args.next().ok_or_else(|| {
                        AppError::Validation("--manifest requires a value".to_string())
                    })?
                }
                "--out" => {
                    parsed.out = args
                        .next()
                        .ok_or_else(|| AppError::Validation("--out requires a value".to_string()))?
                }
                "--base-url" => {
                    let _ = args.next();
                }
                other => return Err(AppError::Validation(format!("unknown argument {other}"))),
            }
        }
        Ok(parsed)
    }
}

fn ensure_parent(path: &str) -> AppResult<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|err| {
            AppError::Internal(format!("failed to create {}: {err}", parent.display()))
        })?;
    }
    Ok(())
}
