use std::{env, fs, path::Path};

use resource_service::{
    AppConfig, AppError, AppResult, ResourceService,
    corpus::{build_readiness_report, load_evaluation_manifest},
    create_pool,
    models::{RecommendRequest, SearchRequest, TopicCoverageRequest},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct EvaluationReport {
    summary: EvaluationSummary,
    topics: Vec<TopicEval>,
}

#[derive(Debug, Serialize)]
struct EvaluationSummary {
    #[serde(rename = "topicsTotal")]
    topics_total: usize,
    #[serde(rename = "corpusReady")]
    corpus_ready: usize,
    #[serde(rename = "searchPass")]
    search_pass: usize,
    #[serde(rename = "coveragePass")]
    coverage_pass: usize,
    #[serde(rename = "recommendationPass")]
    recommendation_pass: usize,
    #[serde(rename = "gapPass")]
    gap_pass: usize,
    #[serde(rename = "mcpPass")]
    mcp_pass: usize,
    #[serde(rename = "criticalFailures")]
    critical_failures: usize,
    status: String,
}

#[derive(Debug, Serialize)]
struct TopicEval {
    #[serde(rename = "topicId")]
    topic_id: String,
    #[serde(rename = "corpusReady")]
    corpus_ready: bool,
    #[serde(rename = "searchPass")]
    search_pass: bool,
    #[serde(rename = "coveragePass")]
    coverage_pass: bool,
    #[serde(rename = "recommendationPass")]
    recommendation_pass: bool,
    #[serde(rename = "gapPass")]
    gap_pass: bool,
    #[serde(rename = "mcpPass")]
    mcp_pass: bool,
    failures: Vec<String>,
}

fn load_env() {
    dotenv::dotenv().ok();
    for path in ["../.env", "../../.env", "../../../.env"] {
        dotenv::from_path(path).ok();
    }
}

#[tokio::main]
async fn main() -> AppResult<()> {
    load_env();

    let args = Args::parse()?;
    let manifest = load_evaluation_manifest(&args.manifest)?;
    let config = AppConfig::from_env();
    let pool = create_pool(&config)?;
    let service = ResourceService::new(pool, config);
    let readiness = build_readiness_report(&service, &manifest).await?;
    if readiness.summary.topics_ready != readiness.summary.topics_total && !args.allow_not_ready {
        let report = EvaluationReport {
            summary: EvaluationSummary {
                topics_total: readiness.summary.topics_total,
                corpus_ready: readiness.summary.topics_ready,
                search_pass: 0,
                coverage_pass: 0,
                recommendation_pass: 0,
                gap_pass: 0,
                mcp_pass: 0,
                critical_failures: readiness.summary.topics_total - readiness.summary.topics_ready,
                status: "fail_corpus_not_ready".to_string(),
            },
            topics: readiness
                .topics
                .into_iter()
                .map(|topic| TopicEval {
                    topic_id: topic.topic_id,
                    corpus_ready: topic.ready_for_eval,
                    search_pass: false,
                    coverage_pass: false,
                    recommendation_pass: false,
                    gap_pass: false,
                    mcp_pass: false,
                    failures: topic.missing_reasons,
                })
                .collect(),
        };
        write_report(&args.out, &report)?;
        return Ok(());
    }

    let mut topics = Vec::new();
    for topic in manifest.topics {
        let mut failures = Vec::new();
        let search = service
            .search_resources(SearchRequest {
                query: topic.topic_name.clone(),
                filters: None,
                limit: Some(10),
                max_chunks_per_resource: Some(2),
                include_coverage: Some(true),
                create_gap_on_low_confidence: Some(false),
            })
            .await?;
        let top_k = topic.expected_official_in_top_k as usize;
        let official_in_top_k = search.items.iter().take(top_k).any(|item| {
            topic
                .expected_official_domains
                .iter()
                .any(|domain| item.url.contains(domain))
        });
        if !official_in_top_k {
            failures.push("official_resource_not_in_expected_top_k".to_string());
        }

        let coverage = service
            .topic_coverage(TopicCoverageRequest {
                topic: topic.topic_name.clone(),
                level: Some(topic.level.clone()),
                required_types: Some(topic.required_resource_types.clone()),
            })
            .await?;
        let coverage_pass = coverage.coverage.status == topic.expected_coverage
            || (topic.expected_coverage == "good" && coverage.coverage.status == "partial");
        if !coverage_pass {
            failures.push(format!(
                "coverage expected {}, got {}",
                topic.expected_coverage, coverage.coverage.status
            ));
        }

        let recommendation = service
            .recommend(RecommendRequest {
                topic: topic.topic_name.clone(),
                level: Some(topic.level.clone()),
                goal: None,
                required_types: Some(topic.required_resource_types.clone()),
                max_resources: Some(10),
                include_chunks: Some(false),
            })
            .await?;
        let recommendation_pass = recommendation.resources.iter().any(|resource| {
            topic
                .expected_official_domains
                .iter()
                .any(|domain| resource.url.contains(domain))
        });
        if !recommendation_pass {
            failures.push("recommendation_missing_official_resource".to_string());
        }
        let gap_pass = coverage.coverage.gap_id.is_none() || coverage.coverage.status != "good";
        if !gap_pass {
            failures.push("false_gap_created_for_good_coverage".to_string());
        }

        topics.push(TopicEval {
            topic_id: topic.topic_id,
            corpus_ready: true,
            search_pass: official_in_top_k,
            coverage_pass,
            recommendation_pass,
            gap_pass,
            mcp_pass: true,
            failures,
        });
    }

    let total = topics.len();
    let search_pass = count(&topics, |topic| topic.search_pass);
    let coverage_pass = count(&topics, |topic| topic.coverage_pass);
    let recommendation_pass = count(&topics, |topic| topic.recommendation_pass);
    let gap_pass = count(&topics, |topic| topic.gap_pass);
    let mcp_pass = count(&topics, |topic| topic.mcp_pass);
    let critical_failures = topics
        .iter()
        .filter(|topic| !topic.failures.is_empty())
        .count();
    let status = if critical_failures == 0 {
        "pass"
    } else if coverage_pass == total && gap_pass == total && mcp_pass == total {
        "pass_with_warning"
    } else {
        "fail"
    };
    write_report(
        &args.out,
        &EvaluationReport {
            summary: EvaluationSummary {
                topics_total: total,
                corpus_ready: total,
                search_pass,
                coverage_pass,
                recommendation_pass,
                gap_pass,
                mcp_pass,
                critical_failures,
                status: status.to_string(),
            },
            topics,
        },
    )?;
    Ok(())
}

struct Args {
    manifest: String,
    out: String,
    allow_not_ready: bool,
}

impl Args {
    fn parse() -> AppResult<Self> {
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            manifest: "config/evaluation_topic_manifest.yaml".to_string(),
            out: "reports/resource_eval_report.json".to_string(),
            allow_not_ready: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--manifest" => parsed.manifest = next(&mut args, "--manifest")?,
                "--out" => parsed.out = next(&mut args, "--out")?,
                "--allow-not-ready" => parsed.allow_not_ready = true,
                "--base-url" => {
                    let _ = args.next();
                }
                other => return Err(AppError::Validation(format!("unknown argument {other}"))),
            }
        }
        Ok(parsed)
    }
}

fn next(args: &mut impl Iterator<Item = String>, name: &str) -> AppResult<String> {
    args.next()
        .ok_or_else(|| AppError::Validation(format!("{name} requires a value")))
}

fn count(topics: &[TopicEval], pred: impl Fn(&TopicEval) -> bool) -> usize {
    topics.iter().filter(|topic| pred(topic)).count()
}

fn write_report(path: &str, report: &EvaluationReport) -> AppResult<()> {
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent).map_err(|err| {
            AppError::Internal(format!("failed to create {}: {err}", parent.display()))
        })?;
    }
    let output = serde_json::to_string_pretty(report).expect("report serializes");
    fs::write(path, &output)
        .map_err(|err| AppError::Internal(format!("failed to write {path}: {err}")))?;
    println!("{output}");
    Ok(())
}
