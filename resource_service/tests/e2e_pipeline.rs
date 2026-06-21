use std::sync::Arc;

use resource_service::{
    AppConfig, ResourceService, create_pool,
    models::{
        CrawlJobRequest, FetchArtifactRequest, ProcessFetchArtifactRequest, SearchRequest,
        SourceRequest,
    },
    worker::run_embedding_once,
};
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn e2e_artifact_extract_embed_search_pipeline() {
    if std::env::var("RUN_RESOURCE_E2E").ok().as_deref() != Some("1") {
        eprintln!("skipping DB e2e; set RUN_RESOURCE_E2E=1 to enable");
        return;
    }

    dotenv::from_path("../.env").ok();

    let config = AppConfig::from_env();
    let pool = create_pool(&config).expect("resource postgres pool should be created");
    let service = Arc::new(ResourceService::new(pool, config));
    service
        .migrate()
        .await
        .expect("schema migration should pass");

    let marker = Uuid::new_v4().simple().to_string();
    let host = format!("e2e-{marker}.test");
    let source = service
        .create_source(SourceRequest {
            name: format!("E2E Docs {marker}"),
            kind: Some("official_docs".to_string()),
            base_url: format!("http://{host}"),
            trust_tier: Some(1),
            language_hint: Some("en".to_string()),
            enabled: Some(true),
            is_official: Some(true),
            crawl_policy: Some(json!({"maxDepth": 1, "rate_limit_per_minute": 60})),
            allowed_paths: Some(vec!["/docs".to_string()]),
            blocked_paths: Some(vec!["/private".to_string()]),
            tags: Some(vec!["e2e".to_string()]),
            notes: Some("resource service e2e".to_string()),
        })
        .await
        .expect("source should be created");

    let url = format!("http://{host}/docs/rust-ownership-{marker}");
    let job = service
        .create_crawl_job(CrawlJobRequest {
            source_site_id: Some(source.id),
            seed_id: None,
            run_id: None,
            url: url.clone(),
            priority: Some(100),
            depth: Some(0),
            metadata: Some(json!({"test": "e2e"})),
        })
        .await
        .expect("crawl job should be created");

    let raw_body = format!(
        "<html lang=\"en\"><head><title>Rust Ownership {marker}</title><link rel=\"canonical\" href=\"/docs/rust-ownership-{marker}\"></head><body><nav>Navigation</nav><h1>Rust Ownership {marker}</h1><p>Rust ownership borrowing lifetimes unique marker {marker}</p><pre><code>let owner = String::from(\"resource e2e\");</code></pre></body></html>"
    );
    let artifact = service
        .create_fetch_artifact(FetchArtifactRequest {
            crawl_job_id: job.id,
            source_site_id: Some(source.id),
            url: url.clone(),
            final_url: Some(url.clone()),
            http_status: Some(200),
            content_type: Some("text/html; charset=utf-8".to_string()),
            content_length: Some(raw_body.len() as i64),
            etag: Some(format!("e2e-{marker}")),
            raw_object_key: None,
            raw_body: Some(raw_body),
            metadata: Some(json!({"test": "e2e"})),
        })
        .await
        .expect("fetch artifact should be stored");

    let extracted = service
        .process_fetch_artifact(ProcessFetchArtifactRequest {
            fetch_artifact_id: artifact.id,
            activate_resource: Some(true),
        })
        .await
        .expect("artifact should extract into an active resource");
    assert!(extracted.chunk_count > 0);

    let embedded_count = run_embedding_once(service.clone())
        .await
        .expect("embedding worker should persist pending chunk vectors");
    assert!(embedded_count > 0);

    let search = service
        .search_chunks(SearchRequest {
            query: format!("Rust ownership borrowing {marker}"),
            filters: None,
            limit: Some(5),
            max_chunks_per_resource: Some(2),
            include_coverage: Some(true),
            create_gap_on_low_confidence: Some(false),
        })
        .await
        .expect("search should return indexed chunks");

    assert!(
        search
            .items
            .iter()
            .any(|item| item.resource_id == extracted.resource_id)
    );
    assert!(search.query_info.strategy.starts_with("hybrid_"));
}
