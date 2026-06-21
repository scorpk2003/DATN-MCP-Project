use serde_json::json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{
        ApproveCandidateResponse, CandidateRequest, CandidateScoringDetails, CandidateSummary,
        GapSummary, Page, PageQuery, ReportGapRequest, ReportGapResponse, ResearchTaskRequest,
        ResearchTaskSummary,
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

    pub async fn ignore_gap(&self, id: Uuid) -> AppResult<GapSummary> {
        self.repository.update_gap_status(id, "cancelled").await
    }

    pub async fn reopen_gap(&self, id: Uuid) -> AppResult<GapSummary> {
        self.repository.update_gap_status(id, "pending").await
    }

    pub async fn resolve_gap(&self, id: Uuid) -> AppResult<GapSummary> {
        self.repository.update_gap_status(id, "succeeded").await
    }

    pub async fn report_gap(&self, request: ReportGapRequest) -> AppResult<ReportGapResponse> {
        if request.topic.trim().is_empty() {
            return Err(AppError::Validation("topic is required".to_string()));
        }
        let missing_types = request.missing_types.clone().unwrap_or_default();
        let gap_id = self
            .repository
            .create_gap(
                "resource_mcp",
                &request.topic,
                5,
                json!({
                    "level": request.level,
                    "missingTypes": missing_types,
                    "reason": request.reason.unwrap_or_else(|| "reported by client".to_string()),
                }),
            )
            .await?;
        let research_task = match gap_id {
            Some(gap_id) => Some(
                self.repository
                    .create_research_task_for_gap(gap_id, &request.topic, &missing_types)
                    .await?,
            ),
            None => None,
        };
        Ok(ReportGapResponse {
            gap_id,
            created: gap_id.is_some(),
            status: "pending".to_string(),
            research_task_id: research_task.map(|task| task.id),
        })
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

    pub async fn list_research_tasks(
        &self,
        query: PageQuery,
    ) -> AppResult<Page<ResearchTaskSummary>> {
        self.repository.list_research_tasks(&query).await
    }

    pub async fn get_research_task(&self, id: Uuid) -> AppResult<ResearchTaskSummary> {
        self.repository.get_research_task(id).await
    }

    pub async fn create_candidate(&self, request: CandidateRequest) -> AppResult<CandidateSummary> {
        validation::validate_http_url(&request.url, "url")?;
        let mut request = request;
        let scoring = score_candidate(&request);
        let mut metadata = request.metadata.take().unwrap_or_else(|| json!({}));
        metadata["candidateType"] = json!(candidate_type(&request));
        metadata["score"] = json!(scoring.final_score);
        metadata["scoringDetails"] = serde_json::to_value(&scoring)
            .map_err(|err| AppError::Internal(format!("serialize scoring failed: {err}")))?;
        request.metadata = Some(metadata);
        self.repository.create_candidate(&request).await
    }

    pub async fn list_candidates(&self, query: PageQuery) -> AppResult<Page<CandidateSummary>> {
        self.repository.list_candidates(&query).await
    }

    pub async fn get_candidate(&self, id: Uuid) -> AppResult<CandidateSummary> {
        self.repository.get_candidate(id).await
    }

    pub async fn approve_candidate(&self, id: Uuid) -> AppResult<ApproveCandidateResponse> {
        self.repository.approve_candidate_with_crawl(id).await
    }

    pub async fn reject_candidate(&self, id: Uuid, reason: String) -> AppResult<CandidateSummary> {
        if reason.trim().is_empty() {
            return Err(AppError::Validation("reason is required".to_string()));
        }
        self.repository.reject_candidate(id, &reason).await
    }
}

fn score_candidate(request: &CandidateRequest) -> CandidateScoringDetails {
    let url = request.url.to_ascii_lowercase();
    let text = format!(
        "{} {} {}",
        request.url,
        request.title.clone().unwrap_or_default(),
        request.snippet.clone().unwrap_or_default()
    )
    .to_ascii_lowercase();
    let authority_score = if is_official_domain(&url) { 0.95 } else { 0.55 };
    let relevance_score = if request.title.as_deref().unwrap_or("").trim().is_empty() {
        0.45
    } else {
        0.75
    };
    let content_depth_score = if text.len() > 200 { 0.75 } else { 0.50 };
    let freshness_score = 0.60;
    let duplicate_penalty = 0.0;
    let language_match_score = 1.0;
    let final_score = relevance_score * 0.35
        + authority_score * 0.25
        + freshness_score * 0.10
        + content_depth_score * 0.15
        + language_match_score * 0.15
        - duplicate_penalty;
    let mut reasons = Vec::new();
    if authority_score > 0.9 {
        reasons.push("official or high-authority domain".to_string());
    }
    if relevance_score > 0.7 {
        reasons.push("title/snippet available for relevance review".to_string());
    }
    CandidateScoringDetails {
        relevance_score,
        authority_score,
        freshness_score,
        content_depth_score,
        duplicate_penalty,
        language_match_score,
        final_score,
        reasons,
    }
}

fn candidate_type(request: &CandidateRequest) -> &'static str {
    let text = format!(
        "{} {} {}",
        request.url,
        request.title.clone().unwrap_or_default(),
        request.snippet.clone().unwrap_or_default()
    )
    .to_ascii_lowercase();
    if is_official_domain(&text) {
        "official_reference"
    } else if text.contains("exercise") || text.contains("practice") {
        "practice"
    } else if text.contains("project") {
        "project"
    } else if text.contains("internals") || text.contains("deep dive") {
        "deep_dive"
    } else {
        "primary_learning"
    }
}

fn is_official_domain(text: &str) -> bool {
    [
        "postgresql.org",
        "react.dev",
        "developer.mozilla.org",
        "kubernetes.io",
        "docs.docker.com",
        "python.org",
        "nodejs.org",
    ]
    .iter()
    .any(|domain| text.contains(domain))
}
