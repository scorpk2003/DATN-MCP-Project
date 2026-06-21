use std::sync::Arc;

use resource_service::{
    AppConfig, ResourceService, create_pool,
    models::{
        AdminResourceActionRequest, CandidateRequest, CrawlJobRequest, EnrichResourceRequest,
        FetchArtifactRequest, ManualIngestRequest, PageQuery, ProcessFetchArtifactRequest,
        RecommendRequest, ReportGapRequest, ResearchTaskRequest, SearchRequest, SourceRequest,
        TopicCoverageRequest,
    },
    worker::run_embedding_once,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn integration_contract_suite() {
    if std::env::var("RUN_RESOURCE_INTEGRATION").ok().as_deref() != Some("1") {
        eprintln!("skipping integration suite; set RUN_RESOURCE_INTEGRATION=1 to enable");
        return;
    }

    dotenv::from_path("../.env").ok();

    let service = test_service().await;
    let marker = Uuid::new_v4().simple().to_string();

    let manual = service
        .ingest_manual(ManualIngestRequest {
            canonical_url: format!("http://integration-{marker}.test/docs/postgresql-indexing"),
            title: format!("PostgreSQL Indexing Integration {marker}"),
            content: format!(
                "# PostgreSQL Indexing {marker}\n\nThis official guide explains PostgreSQL b-tree index performance, query planner behavior, transaction isolation, and practice exercises for backend services."
            ),
            summary: Some("PostgreSQL indexing integration resource".to_string()),
            description: Some("Integration suite seed resource".to_string()),
            kind: Some("docs".to_string()),
            format: Some("markdown".to_string()),
            language_code: Some("en".to_string()),
            primary_domain: None,
            is_official: Some(true),
            source_id: None,
            metadata: Some(json!({"test": "integration_suite", "marker": marker})),
        })
        .await
        .expect("manual ingest should create active resource/version/chunks");
    assert!(manual.chunk_count > 0);

    let detail = service
        .get_resource_detail(manual.resource_id)
        .await
        .expect("resource detail should load");
    assert_eq!(detail.resource.id, manual.resource_id);
    assert!(detail.chunk_count > 0);

    let versions = service
        .list_versions(manual.resource_id)
        .await
        .expect("resource versions should list");
    assert!(!versions.is_empty());

    let chunks = service
        .get_resource_chunks(manual.resource_id, Some(manual.version_id), Some(10))
        .await
        .expect("resource chunks should list");
    assert!(!chunks.is_empty());

    let enrichment = service
        .enrich_resource(
            manual.resource_id,
            EnrichResourceRequest {
                resource_version_id: Some(manual.version_id),
            },
        )
        .await
        .expect("enrichment should classify resource topics");
    assert!(
        enrichment
            .topics
            .iter()
            .any(|topic| topic.slug == "postgresql")
    );

    let embedded_count = run_embedding_once(service.clone())
        .await
        .expect("embedding worker should persist vectors");
    assert!(embedded_count >= 1);

    let search = service
        .search_chunks(SearchRequest {
            query: format!("PostgreSQL b-tree index query planner {marker}"),
            filters: None,
            limit: Some(5),
            max_chunks_per_resource: Some(2),
            include_coverage: Some(true),
            create_gap_on_low_confidence: Some(false),
        })
        .await
        .expect("search should return ingested resource");
    assert!(
        search
            .items
            .iter()
            .any(|item| item.resource_id == manual.resource_id)
    );

    let recommendation = service
        .recommend(RecommendRequest {
            topic: "PostgreSQL".to_string(),
            level: None,
            goal: Some("learn indexing".to_string()),
            required_types: Some(vec!["primary_learning".to_string()]),
            max_resources: Some(5),
            include_chunks: Some(true),
        })
        .await
        .expect("recommend should use enriched topics");
    assert!(!recommendation.resources.is_empty());

    let coverage = service
        .topic_coverage(TopicCoverageRequest {
            topic: "PostgreSQL".to_string(),
            level: None,
            required_types: Some(vec!["primary_learning".to_string()]),
        })
        .await
        .expect("coverage should be computed from recommendations");
    assert!(coverage.coverage.result_count > 0);

    let gap = service
        .report_gap(ReportGapRequest {
            topic: format!("rare integration topic {marker}"),
            level: Some("advanced".to_string()),
            missing_types: Some(vec!["official_reference".to_string()]),
            reason: Some("integration test gap".to_string()),
        })
        .await
        .expect("gap report should create or reuse gap");
    assert!(gap.gap_id.is_some() || !gap.created);

    let task = service
        .create_research_task(ResearchTaskRequest {
            topic: format!("React useEffect integration {marker}"),
            gap_id: gap.gap_id,
            language: Some("en".to_string()),
            priority: Some(100),
            target_resource_types: Some(vec!["official_reference".to_string()]),
        })
        .await
        .expect("research task should be created");

    let candidate = service
        .create_candidate(CandidateRequest {
            research_task_id: task.id,
            url: format!("https://react.dev/reference/react/useEffect?marker={marker}"),
            title: Some("React useEffect Reference".to_string()),
            snippet: Some("Official React useEffect reference and cleanup guide".to_string()),
            metadata: Some(json!({"test": "integration_suite"})),
        })
        .await
        .expect("candidate should be scored and stored");
    assert!(candidate.score > 0.0);

    let rejected = service
        .reject_candidate(candidate.id, "integration test reject path".to_string())
        .await
        .expect("candidate reject should persist reason");
    assert!(rejected.reject_reason.is_some());
    assert!(
        service
            .count_outbox_events("research_candidate.rejected", Some(candidate.id))
            .await
            .expect("reject audit event should be countable")
            >= 1
    );

    let action = service
        .mark_resource_needs_review(
            manual.resource_id,
            AdminResourceActionRequest {
                reason: Some("integration suite".to_string()),
                actor_id: Some("integration-test".to_string()),
            },
        )
        .await
        .expect("admin action should update resource review state");
    assert_eq!(action.resource_id, manual.resource_id);
    assert!(
        service
            .count_outbox_events("admin.resource.needs_review", Some(manual.resource_id))
            .await
            .expect("admin audit event should be countable")
            >= 1
    );

    let _boost = service
        .boost_resource_quality(
            manual.resource_id,
            AdminResourceActionRequest {
                reason: Some("integration boost".to_string()),
                actor_id: Some("integration-test".to_string()),
            },
        )
        .await
        .expect("boost should update quality");
    let _deboost = service
        .deboost_resource_quality(
            manual.resource_id,
            AdminResourceActionRequest {
                reason: Some("integration deboost".to_string()),
                actor_id: Some("integration-test".to_string()),
            },
        )
        .await
        .expect("deboost should update quality");
    assert!(
        service
            .count_outbox_events("admin.resource.quality_boost", Some(manual.resource_id))
            .await
            .expect("boost audit event should be countable")
            >= 1
    );
    assert!(
        service
            .count_outbox_events("admin.resource.quality_deboost", Some(manual.resource_id))
            .await
            .expect("deboost audit event should be countable")
            >= 1
    );

    let source = service
        .create_source(SourceRequest {
            name: format!("Integration Source {marker}"),
            kind: Some("official_docs".to_string()),
            base_url: format!("https://integration-{marker}.test"),
            trust_tier: Some(1),
            language_hint: Some("en".to_string()),
            enabled: Some(true),
            is_official: Some(true),
            crawl_policy: Some(json!({"maxDepth": 1})),
            allowed_paths: Some(vec!["/docs".to_string()]),
            blocked_paths: Some(vec!["/private".to_string()]),
            tags: Some(vec!["integration".to_string()]),
            notes: Some("integration source".to_string()),
        })
        .await
        .expect("source should be created");

    let crawl_job = service
        .create_crawl_job(CrawlJobRequest {
            source_site_id: Some(source.id),
            seed_id: None,
            run_id: None,
            url: format!("https://integration-{marker}.test/docs/article"),
            priority: Some(50),
            depth: Some(0),
            metadata: Some(json!({"test": "integration_suite"})),
        })
        .await
        .expect("crawl job should be enqueued");
    let retried = service
        .retry_crawl_job(crawl_job.id)
        .await
        .expect("retry should create audit event");
    assert_eq!(retried.id, crawl_job.id);
    assert!(
        service
            .count_outbox_events("crawl_job.retry", Some(crawl_job.id))
            .await
            .expect("retry audit event should be countable")
            >= 1
    );

    let artifact = service
        .create_fetch_artifact(FetchArtifactRequest {
            crawl_job_id: crawl_job.id,
            source_site_id: Some(source.id),
            url: crawl_job.url.clone(),
            final_url: Some(crawl_job.url),
            http_status: Some(200),
            content_type: Some("text/html".to_string()),
            content_length: Some(128),
            etag: None,
            raw_object_key: None,
            raw_body: Some(format!(
                "<html><head><title>Integration Fetch {marker}</title></head><body><h1>Backend API Integration</h1><p>HTTP service backend API integration content {marker}</p></body></html>"
            )),
            metadata: Some(json!({"test": "integration_suite"})),
        })
        .await
        .expect("fetch artifact should store and complete job");

    let processed = service
        .process_fetch_artifact(ProcessFetchArtifactRequest {
            fetch_artifact_id: artifact.id,
            activate_resource: Some(true),
        })
        .await
        .expect("fetch artifact should extract");
    assert!(processed.chunk_count > 0);

    let dashboard = service
        .admin_dashboard_summary()
        .await
        .expect("admin dashboard should load");
    assert!(dashboard.open_gaps >= 0);

    let resources = service
        .list_resources(PageQuery {
            limit: Some(5),
            offset: Some(0),
            sort_by: None,
            sort_order: None,
        })
        .await
        .expect("resource pagination should load");
    assert!(resources.pagination.limit <= 100);

    let metrics = service
        .metrics_snapshot()
        .await
        .expect("metrics snapshot should load");
    assert!(
        metrics["fetchArtifactsCreated"]
            .as_i64()
            .unwrap_or_default()
            > 0
    );
    assert!(metrics["chunksCreatedTotal"].as_i64().unwrap_or_default() > 0);

    let lifecycle_gap = service
        .report_gap(ReportGapRequest {
            topic: format!("approval lifecycle gap {marker}"),
            level: Some("advanced".to_string()),
            missing_types: Some(vec!["official_reference".to_string()]),
            reason: Some("integration approve lifecycle".to_string()),
        })
        .await
        .expect("explicit gap report should create lifecycle gap");
    let gap_id = lifecycle_gap
        .gap_id
        .expect("explicit gap report should attach gap id");
    let running_gap = service
        .reopen_gap(gap_id)
        .await
        .expect("gap reopen should update lifecycle");
    assert_eq!(running_gap.status, "pending");
    let ignored_gap = service
        .ignore_gap(gap_id)
        .await
        .expect("gap ignore should update lifecycle");
    assert_eq!(ignored_gap.status, "cancelled");
    let reopened_gap = service
        .reopen_gap(gap_id)
        .await
        .expect("gap reopen should update lifecycle again");
    assert_eq!(reopened_gap.status, "pending");
    assert!(
        service
            .count_outbox_events("gap.pending", Some(gap_id))
            .await
            .expect("gap lifecycle event should be countable")
            >= 1
    );

    let approve_task = service
        .create_research_task(ResearchTaskRequest {
            topic: format!("HTTP service backend approval {marker}"),
            gap_id: Some(gap_id),
            language: Some("en".to_string()),
            priority: Some(150),
            target_resource_types: Some(vec!["official_reference".to_string()]),
        })
        .await
        .expect("approve flow research task should be created");
    let approve_candidate = service
        .create_candidate(CandidateRequest {
            research_task_id: approve_task.id,
            url: format!("https://integration-approve-{marker}.test/docs/backend-api"),
            title: Some("Backend API Approval Candidate".to_string()),
            snippet: Some("Official backend API service guide".to_string()),
            metadata: Some(json!({"test": "integration_approve"})),
        })
        .await
        .expect("approve flow candidate should be created");
    let approved = service
        .approve_candidate(approve_candidate.id)
        .await
        .expect("candidate approve should create crawl seed and job");
    let approved_job_id = approved
        .created_crawl_job_id
        .expect("approve should create crawl job");
    assert!(
        service
            .count_outbox_events("research_candidate.approved", Some(approve_candidate.id))
            .await
            .expect("approve audit event should be countable")
            >= 1
    );

    let approved_job = service
        .get_crawl_job(approved_job_id)
        .await
        .expect("approved candidate job should load");
    let approved_body = format!(
        "<html><head><title>Approved Backend API {marker}</title></head><body><h1>Backend API Approval</h1><p>HTTP service backend API official approval content {marker}</p></body></html>"
    );
    let approved_artifact = service
        .create_fetch_artifact(FetchArtifactRequest {
            crawl_job_id: approved_job.id,
            source_site_id: approved_job.source_id,
            url: approved_job.url.clone(),
            final_url: Some(approved_job.url),
            http_status: Some(200),
            content_type: Some("text/html".to_string()),
            content_length: Some(approved_body.len() as i64),
            etag: None,
            raw_object_key: None,
            raw_body: Some(approved_body),
            metadata: Some(json!({"test": "integration_approve"})),
        })
        .await
        .expect("approved candidate artifact should store");
    let approved_processed = service
        .process_fetch_artifact(ProcessFetchArtifactRequest {
            fetch_artifact_id: approved_artifact.id,
            activate_resource: Some(true),
        })
        .await
        .expect("approved candidate artifact should extract");
    assert!(approved_processed.chunk_count > 0);
    let _ = run_embedding_once(service.clone())
        .await
        .expect("embedding worker should handle approved candidate chunks");
    let post_approve_search = service
        .search_chunks(SearchRequest {
            query: format!("backend API approval {marker}"),
            filters: None,
            limit: Some(5),
            max_chunks_per_resource: Some(2),
            include_coverage: Some(true),
            create_gap_on_low_confidence: Some(false),
        })
        .await
        .expect("search should find approved candidate resource");
    assert!(
        post_approve_search
            .items
            .iter()
            .any(|item| item.resource_id == approved_processed.resource_id)
    );
    let resolved_gap = service
        .resolve_gap(gap_id)
        .await
        .expect("gap resolve should update lifecycle");
    assert_eq!(resolved_gap.status, "succeeded");
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
