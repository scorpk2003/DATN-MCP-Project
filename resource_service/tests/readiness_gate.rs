use std::{fs, sync::Arc};

use resource_service::{
    AppConfig, ResourceService, create_pool,
    models::{
        CandidateRequest, FetchArtifactRequest, ManualIngestRequest, ProcessFetchArtifactRequest,
        RecommendRequest, ReportGapRequest, ResearchTaskRequest, SearchRequest, SourceRequest,
        TopicCoverageRequest,
    },
    worker::run_embedding_once,
};
use serde::Serialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct ReadinessSummary {
    status: String,
    checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Serialize)]
struct ReadinessCheck {
    name: String,
    passed: bool,
    details: serde_json::Value,
}

#[tokio::test]
async fn resource_service_readiness_gate() {
    if std::env::var("RUN_RESOURCE_READINESS").ok().as_deref() != Some("1") {
        eprintln!("skipping readiness gate; set RUN_RESOURCE_READINESS=1 to enable");
        return;
    }

    dotenv::from_path("../.env").ok();

    let service = test_service().await;
    let marker = Uuid::new_v4().simple().to_string();
    let mut checks = Vec::new();

    let ready = service
        .health_check()
        .await
        .expect("health check should run");
    checks.push(check(
        "resource_service_ready",
        ready.status == "ready",
        json!({"status": ready.status, "checks": ready.checks}),
    ));

    let manual = service
        .ingest_manual(ManualIngestRequest {
            canonical_url: format!("https://readiness-{marker}.test/docs/postgresql-indexing"),
            title: format!("Readiness PostgreSQL Indexing {marker}"),
            content: format!(
                "# Readiness PostgreSQL Indexing {marker}\n\nPostgreSQL indexing, b-tree, query planner, backend API service, and transaction isolation content for readiness."
            ),
            summary: Some("Readiness resource".to_string()),
            description: Some("Readiness gate seed".to_string()),
            kind: Some("docs".to_string()),
            format: Some("markdown".to_string()),
            language_code: Some("en".to_string()),
            primary_domain: None,
            is_official: Some(true),
            source_id: None,
            metadata: Some(json!({"readiness": true, "marker": marker})),
        })
        .await
        .expect("manual ingest should work");
    checks.push(check(
        "manual_ingest",
        manual.chunk_count > 0,
        json!({"resourceId": manual.resource_id, "chunkCount": manual.chunk_count}),
    ));

    let enriched = service
        .enrich_resource(
            manual.resource_id,
            resource_service::models::EnrichResourceRequest {
                resource_version_id: Some(manual.version_id),
            },
        )
        .await
        .expect("enrichment should run");
    checks.push(check(
        "enrichment",
        enriched
            .topics
            .iter()
            .any(|topic| topic.slug == "postgresql"),
        json!({"topics": enriched.topics}),
    ));

    let embedded = run_embedding_once(service.clone())
        .await
        .expect("embedding worker should run");
    checks.push(check(
        "embedding_worker",
        embedded > 0,
        json!({"embeddedChunks": embedded}),
    ));

    let search = service
        .search_chunks(SearchRequest {
            query: format!("PostgreSQL indexing query planner {marker}"),
            filters: None,
            limit: Some(5),
            max_chunks_per_resource: Some(2),
            include_coverage: Some(true),
            create_gap_on_low_confidence: Some(false),
        })
        .await
        .expect("search should run");
    checks.push(check(
        "search_resources",
        search
            .items
            .iter()
            .any(|item| item.resource_id == manual.resource_id),
        json!({"resultCount": search.items.len(), "strategy": search.query_info.strategy}),
    ));

    let detail = service
        .get_resource_detail(manual.resource_id)
        .await
        .expect("resource detail should load");
    let chunks = service
        .get_resource_chunks(manual.resource_id, Some(manual.version_id), Some(5))
        .await
        .expect("resource chunks should load");
    checks.push(check(
        "resource_read_contract",
        detail.chunk_count > 0 && !chunks.is_empty(),
        json!({"chunkCount": detail.chunk_count, "returnedChunks": chunks.len()}),
    ));

    let recommendation = service
        .recommend(RecommendRequest {
            topic: "PostgreSQL".to_string(),
            level: None,
            goal: Some("readiness".to_string()),
            required_types: Some(vec!["primary_learning".to_string()]),
            max_resources: Some(5),
            include_chunks: Some(true),
        })
        .await
        .expect("recommend should run");
    let coverage = service
        .topic_coverage(TopicCoverageRequest {
            topic: "PostgreSQL".to_string(),
            level: None,
            required_types: Some(vec!["primary_learning".to_string()]),
        })
        .await
        .expect("coverage should run");
    checks.push(check(
        "recommend_and_coverage",
        !recommendation.resources.is_empty() && coverage.coverage.result_count > 0,
        json!({
            "recommendations": recommendation.resources.len(),
            "coverage": coverage.coverage
        }),
    ));

    let source = service
        .create_source(SourceRequest {
            name: format!("Readiness Source {marker}"),
            kind: Some("official_docs".to_string()),
            base_url: format!("https://readiness-source-{marker}.test"),
            trust_tier: Some(1),
            language_hint: Some("en".to_string()),
            enabled: Some(true),
            is_official: Some(true),
            crawl_policy: Some(json!({"maxDepth": 1})),
            allowed_paths: Some(vec!["/docs".to_string()]),
            blocked_paths: Some(vec!["/private".to_string()]),
            tags: Some(vec!["readiness".to_string()]),
            notes: Some("readiness source".to_string()),
        })
        .await
        .expect("source should create");
    let crawl_job = service
        .create_crawl_job(resource_service::models::CrawlJobRequest {
            source_site_id: Some(source.id),
            seed_id: None,
            run_id: None,
            url: format!("https://readiness-source-{marker}.test/docs/backend-api"),
            priority: Some(100),
            depth: Some(0),
            metadata: Some(json!({"readiness": true})),
        })
        .await
        .expect("crawl job should create");
    let raw_body = format!(
        "<html><head><title>Readiness Backend API {marker}</title></head><body><h1>Readiness Backend API</h1><p>Backend API service readiness content {marker}</p></body></html>"
    );
    let artifact = service
        .create_fetch_artifact(FetchArtifactRequest {
            crawl_job_id: crawl_job.id,
            source_site_id: Some(source.id),
            url: crawl_job.url.clone(),
            final_url: Some(crawl_job.url),
            http_status: Some(200),
            content_type: Some("text/html".to_string()),
            content_length: Some(raw_body.len() as i64),
            etag: None,
            raw_object_key: None,
            raw_body: Some(raw_body),
            metadata: Some(json!({"readiness": true})),
        })
        .await
        .expect("artifact should store");
    let processed = service
        .process_fetch_artifact(ProcessFetchArtifactRequest {
            fetch_artifact_id: artifact.id,
            activate_resource: Some(true),
        })
        .await
        .expect("artifact should process");
    checks.push(check(
        "fetch_extract_pipeline",
        processed.chunk_count > 0,
        json!({"resourceId": processed.resource_id, "chunkCount": processed.chunk_count}),
    ));

    let gap = service
        .report_gap(ReportGapRequest {
            topic: format!("Readiness missing topic {marker}"),
            level: Some("advanced".to_string()),
            missing_types: Some(vec!["official_reference".to_string()]),
            reason: Some("readiness gate".to_string()),
        })
        .await
        .expect("gap report should work");
    let task = service
        .create_research_task(ResearchTaskRequest {
            topic: format!("Readiness research {marker}"),
            gap_id: gap.gap_id,
            language: Some("en".to_string()),
            priority: Some(100),
            target_resource_types: Some(vec!["official_reference".to_string()]),
        })
        .await
        .expect("research task should create");
    let candidate = service
        .create_candidate(CandidateRequest {
            research_task_id: task.id,
            url: format!("https://readiness-candidate-{marker}.test/docs/resource"),
            title: Some("Readiness Candidate".to_string()),
            snippet: Some("Official readiness candidate".to_string()),
            metadata: Some(json!({"readiness": true})),
        })
        .await
        .expect("candidate should create");
    let approved = service
        .approve_candidate(candidate.id)
        .await
        .expect("candidate approve should work");
    checks.push(check(
        "gap_research_candidate_flow",
        gap.gap_id.is_some()
            && approved.created_crawl_seed_id.is_some()
            && approved.created_crawl_job_id.is_some(),
        json!({
            "gapId": gap.gap_id,
            "researchTaskId": task.id,
            "candidateId": candidate.id,
            "crawlJobId": approved.created_crawl_job_id
        }),
    ));

    let metrics = service
        .metrics_snapshot()
        .await
        .expect("metrics should load");
    checks.push(check(
        "metrics_snapshot",
        metrics["fetchArtifactsCreated"]
            .as_i64()
            .unwrap_or_default()
            > 0
            && metrics["chunksCreatedTotal"].as_i64().unwrap_or_default() > 0,
        metrics,
    ));

    let status = if checks.iter().all(|check| check.passed) {
        "ready"
    } else {
        "not_ready"
    };
    let summary = ReadinessSummary {
        status: status.to_string(),
        checks,
    };
    let output = serde_json::to_string_pretty(&summary).expect("summary should serialize");
    fs::write("/tmp/resource_readiness_summary.json", &output)
        .expect("readiness summary should write");
    println!("{output}");
    assert_eq!(
        summary.status, "ready",
        "see /tmp/resource_readiness_summary.json"
    );
}

fn check(name: &str, passed: bool, details: serde_json::Value) -> ReadinessCheck {
    ReadinessCheck {
        name: name.to_string(),
        passed,
        details,
    }
}

async fn test_service() -> Arc<ResourceService> {
    let config = AppConfig::from_env();
    let pool = create_pool(&config).expect("resource postgres pool should be created");
    let service = Arc::new(ResourceService::new(pool, config));
    service
        .migrate()
        .await
        .expect("schema migration should pass");
    service
}
