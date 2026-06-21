mod admin;
mod embedding;
mod enrichment;
mod health;
mod research;
mod resources;
mod search;
mod source_crawl;

use std::sync::Arc;

use axum::{
    Router,
    routing::{get, post},
};

use crate::ResourceService;

pub fn router(service: Arc<ResourceService>) -> Router {
    Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route("/metrics", get(health::metrics))
        .route("/admin/migrate", post(health::migrate))
        .route(
            "/resources",
            post(resources::create_resource).get(resources::list_resources),
        )
        .route("/resources/manual-ingest", post(resources::ingest_manual))
        .route(
            "/resources/{id}",
            get(resources::get_resource_detail).patch(resources::update_resource),
        )
        .route(
            "/resources/{id}/versions",
            get(resources::list_versions).post(resources::create_resource_version),
        )
        .route(
            "/resources/{id}/chunks",
            get(resources::get_resource_chunks),
        )
        .route(
            "/worker/enrichment/resources/{id}",
            post(enrichment::enrich_resource),
        )
        .route("/search/chunks", post(search::search_chunks))
        .route("/search/resources", post(search::search_resources))
        .route("/recommend/resources", post(search::recommend_resources))
        .route("/coverage/topic", post(search::topic_coverage))
        .route(
            "/embedding/models",
            get(embedding::list_embedding_models).post(embedding::create_embedding_model),
        )
        .route(
            "/worker/embedding/chunks/pending",
            get(embedding::list_pending_embedding_chunks),
        )
        .route(
            "/sources",
            post(source_crawl::create_source).get(source_crawl::list_sources),
        )
        .route(
            "/sources/{id}",
            get(source_crawl::get_source).patch(source_crawl::patch_source),
        )
        .route(
            "/crawl/seeds",
            post(source_crawl::create_crawl_seed).get(source_crawl::list_crawl_seeds),
        )
        .route("/crawl/jobs", post(source_crawl::create_crawl_job))
        .route("/crawl/jobs/{id}", get(source_crawl::get_crawl_job))
        .route(
            "/worker/crawl/jobs/claim",
            post(source_crawl::claim_crawl_jobs),
        )
        .route("/worker/crawl/schedule", post(source_crawl::schedule_crawl))
        .route(
            "/worker/crawl/jobs/{id}/complete",
            post(source_crawl::complete_crawl_job),
        )
        .route(
            "/worker/fetch/artifacts",
            post(source_crawl::create_fetch_artifact),
        )
        .route(
            "/worker/extract/process",
            post(source_crawl::process_fetch_artifact),
        )
        .route("/gaps", get(research::list_gaps).post(research::report_gap))
        .route("/gaps/{id}", get(research::get_gap))
        .route("/gaps/{id}/resolve", post(research::resolve_gap))
        .route(
            "/research/tasks",
            get(research::list_research_tasks).post(research::create_research_task),
        )
        .route("/research/tasks/{id}", get(research::get_research_task))
        .route(
            "/research/candidates",
            get(research::list_candidates).post(research::create_candidate),
        )
        .route("/research/candidates/{id}", get(research::get_candidate))
        .route(
            "/research/candidates/{id}/approve",
            post(research::approve_candidate),
        )
        .route(
            "/research/candidates/{id}/reject",
            post(research::reject_candidate),
        )
        .route(
            "/admin/sources",
            post(source_crawl::create_source).get(source_crawl::list_sources),
        )
        .route(
            "/admin/sources/{id}",
            get(source_crawl::get_source).patch(source_crawl::patch_source),
        )
        .route("/admin/dashboard", get(admin::dashboard_summary))
        .route("/admin/crawl/jobs", get(admin::list_crawl_jobs))
        .route(
            "/admin/resources/{id}/enrich",
            post(enrichment::enrich_resource),
        )
        .route(
            "/admin/resources/{id}/mark-outdated",
            post(admin::mark_resource_outdated),
        )
        .route(
            "/admin/resources/{id}/mark-needs-review",
            post(admin::mark_resource_needs_review),
        )
        .route(
            "/admin/resources/{id}/boost",
            post(admin::boost_resource_quality),
        )
        .route(
            "/admin/resources/{id}/deboost",
            post(admin::deboost_resource_quality),
        )
        .route("/admin/crawl/schedule", post(source_crawl::schedule_crawl))
        .route("/admin/crawl/jobs/{id}/retry", post(admin::retry_crawl_job))
        .route(
            "/admin/crawl/jobs/{id}/cancel",
            post(admin::cancel_crawl_job),
        )
        .route("/admin/gaps", get(research::list_gaps))
        .route("/admin/gaps/{id}", get(research::get_gap))
        .route("/admin/gaps/{id}/ignore", post(research::ignore_gap))
        .route("/admin/gaps/{id}/reopen", post(research::reopen_gap))
        .route("/admin/research/candidates", get(research::list_candidates))
        .route("/admin/research/tasks", get(research::list_research_tasks))
        .route(
            "/admin/research/tasks/{id}",
            get(research::get_research_task),
        )
        .route(
            "/admin/research/candidates/{id}",
            get(research::get_candidate),
        )
        .route(
            "/admin/research/candidates/{id}/approve",
            post(research::approve_candidate),
        )
        .route(
            "/admin/research/candidates/{id}/reject",
            post(research::reject_candidate),
        )
        .with_state(service)
}
