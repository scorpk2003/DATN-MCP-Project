use serde_json::json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{
        CandidateRequest, CandidateSummary, GapSummary, Page, PageQuery, ReportGapRequest,
        ResearchTaskRequest, ResearchTaskSummary,
    },
};

use super::{ResourceService, validation};

impl ResourceService {
    pub async fn list_gaps(&self, query: PageQuery) -> AppResult<Page<GapSummary>> {
        self.repository.list_gaps(&query).await
    }

    pub async fn get_gap(&self, id: Uuid) -> AppResult<GapSummary> {
        self.repository.get_gap(id).await
    }

    pub async fn report_gap(&self, request: ReportGapRequest) -> AppResult<Option<Uuid>> {
        if request.topic.trim().is_empty() {
            return Err(AppError::Validation("topic is required".to_string()));
        }
        self.repository
            .create_gap_if_low_results(
                "resource_mcp",
                &request.topic,
                0,
                5,
                json!({
                    "level": request.level,
                    "missingTypes": request.missing_types.unwrap_or_default(),
                    "reason": request.reason.unwrap_or_else(|| "reported by client".to_string()),
                }),
            )
            .await
    }

    pub async fn create_research_task(
        &self,
        request: ResearchTaskRequest,
    ) -> AppResult<ResearchTaskSummary> {
        if request.topic.trim().is_empty() {
            return Err(AppError::Validation("topic is required".to_string()));
        }
        self.repository.create_research_task(&request).await
    }

    pub async fn create_candidate(&self, request: CandidateRequest) -> AppResult<CandidateSummary> {
        validation::validate_http_url(&request.url, "url")?;
        self.repository.create_candidate(&request).await
    }

    pub async fn list_candidates(&self, query: PageQuery) -> AppResult<Page<CandidateSummary>> {
        self.repository.list_candidates(&query).await
    }

    pub async fn approve_candidate(&self, id: Uuid) -> AppResult<CandidateSummary> {
        self.repository.approve_candidate(id).await
    }

    pub async fn reject_candidate(&self, id: Uuid, reason: String) -> AppResult<CandidateSummary> {
        if reason.trim().is_empty() {
            return Err(AppError::Validation("reason is required".to_string()));
        }
        self.repository.reject_candidate(id, &reason).await
    }
}
