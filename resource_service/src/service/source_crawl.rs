use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{
        ClaimJobsRequest, CompleteJobRequest, CrawlJob, CrawlJobRequest, CrawlSeed,
        CrawlSeedRequest, FetchArtifact, FetchArtifactRequest, Page, PageQuery,
        ScheduleCrawlRequest, ScheduleCrawlResponse, SourcePatchRequest, SourceRequest, SourceSite,
    },
};

use super::{ResourceService, validation};

impl ResourceService {
    pub async fn create_source(&self, request: SourceRequest) -> AppResult<SourceSite> {
        if request.name.trim().is_empty() {
            return Err(AppError::Validation("name is required".to_string()));
        }
        validation::validate_http_url(&request.base_url, "baseUrl")?;
        self.repository.create_source(&request).await
    }

    pub async fn list_sources(&self, query: PageQuery) -> AppResult<Page<SourceSite>> {
        self.repository.list_sources(&query).await
    }

    pub async fn get_source(&self, id: Uuid) -> AppResult<SourceSite> {
        self.repository.get_source(id).await
    }

    pub async fn patch_source(
        &self,
        id: Uuid,
        request: SourcePatchRequest,
    ) -> AppResult<SourceSite> {
        self.repository.patch_source(id, &request).await
    }

    pub async fn create_crawl_seed(&self, request: CrawlSeedRequest) -> AppResult<CrawlSeed> {
        validation::validate_http_url(&request.seed_url, "seedUrl")?;
        self.repository.create_crawl_seed(&request).await
    }

    pub async fn list_crawl_seeds(&self, query: PageQuery) -> AppResult<Page<CrawlSeed>> {
        self.repository.list_crawl_seeds(&query).await
    }

    pub async fn schedule_crawl(
        &self,
        request: ScheduleCrawlRequest,
    ) -> AppResult<ScheduleCrawlResponse> {
        self.repository.schedule_crawl(&request).await
    }

    pub async fn create_crawl_job(&self, request: CrawlJobRequest) -> AppResult<CrawlJob> {
        validation::validate_http_url(&request.url, "url")?;
        self.repository.create_crawl_job(&request).await
    }

    pub async fn get_crawl_job(&self, id: Uuid) -> AppResult<CrawlJob> {
        self.repository.get_crawl_job(id).await
    }

    pub async fn list_crawl_jobs(&self, query: PageQuery) -> AppResult<Page<CrawlJob>> {
        self.repository.list_crawl_jobs(&query).await
    }

    pub async fn retry_crawl_job(&self, id: Uuid) -> AppResult<CrawlJob> {
        self.repository.retry_crawl_job(id).await
    }

    pub async fn cancel_crawl_job(&self, id: Uuid) -> AppResult<CrawlJob> {
        self.repository.cancel_crawl_job(id).await
    }

    pub async fn claim_crawl_jobs(&self, request: ClaimJobsRequest) -> AppResult<Vec<CrawlJob>> {
        if request.worker_id.trim().is_empty() {
            return Err(AppError::Validation("workerId is required".to_string()));
        }
        self.repository.claim_crawl_jobs(&request).await
    }

    pub async fn complete_crawl_job(
        &self,
        id: Uuid,
        request: CompleteJobRequest,
    ) -> AppResult<CrawlJob> {
        self.repository.complete_crawl_job(id, &request).await
    }

    pub async fn create_fetch_artifact(
        &self,
        request: FetchArtifactRequest,
    ) -> AppResult<FetchArtifact> {
        validation::validate_http_url(&request.url, "url")?;
        if let Some(final_url) = &request.final_url {
            validation::validate_http_url(final_url, "finalUrl")?;
        }
        if request.raw_body.is_none() && request.raw_object_key.is_none() {
            return Err(AppError::Validation(
                "rawBody or rawObjectKey is required".to_string(),
            ));
        }
        if let Some(content_type) = &request.content_type {
            validate_supported_content_type(content_type)?;
        }

        let artifact = self.repository.create_fetch_artifact(&request).await?;
        let succeeded = request
            .http_status
            .map(|status| (200..400).contains(&status))
            .unwrap_or(false);
        let complete = CompleteJobRequest {
            succeeded,
            http_status: request.http_status,
            error: if succeeded {
                None
            } else {
                Some("fetch artifact stored with non-success status".to_string())
            },
        };
        self.repository
            .complete_crawl_job(request.crawl_job_id, &complete)
            .await?;
        Ok(artifact)
    }
}

fn validate_supported_content_type(content_type: &str) -> AppResult<()> {
    let content_type = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase();
    let supported = matches!(
        content_type.as_str(),
        "text/html" | "text/markdown" | "text/plain" | "application/json"
    );
    if supported {
        Ok(())
    } else {
        Err(AppError::Validation(format!(
            "unsupported contentType: {content_type}"
        )))
    }
}
