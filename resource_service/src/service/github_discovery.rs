use std::time::Duration;

use reqwest::header::{ACCEPT, AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    AppError, AppResult,
    models::{CandidateRequest, GitHubDiscoveryRequest, GitHubDiscoveryResponse},
};

use super::ResourceService;

#[derive(Debug, Deserialize)]
struct GitHubSearchResponse {
    items: Vec<GitHubRepository>,
}

#[derive(Debug, Deserialize)]
struct GitHubRepository {
    full_name: String,
    html_url: String,
    description: Option<String>,
    stargazers_count: u32,
    forks_count: u32,
    language: Option<String>,
    topics: Option<Vec<String>>,
    updated_at: Option<String>,
    pushed_at: Option<String>,
    license: Option<GitHubLicense>,
}

#[derive(Debug, Deserialize)]
struct GitHubLicense {
    spdx_id: Option<String>,
}

impl ResourceService {
    pub async fn discover_github_candidates(
        &self,
        research_task_id: Uuid,
        request: GitHubDiscoveryRequest,
    ) -> AppResult<GitHubDiscoveryResponse> {
        let task = self.get_research_task(research_task_id).await?;
        let query_text = request
            .query
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(task.query.as_str());
        if query_text.is_empty() {
            return Err(AppError::Validation("query is required".to_string()));
        }

        let limit = request
            .limit
            .unwrap_or(5)
            .clamp(1, self.config.github.max_candidates.max(1));
        let min_stars = request.min_stars.unwrap_or(25);
        let github_query =
            build_github_repo_query(query_text, request.language.as_deref(), min_stars);
        let repositories = self
            .search_github_repositories(&github_query, limit)
            .await?;

        let mut candidates = Vec::new();
        for repository in repositories {
            let candidate = self
                .create_candidate(CandidateRequest {
                    research_task_id,
                    url: repository.html_url.clone(),
                    title: Some(repository.full_name.clone()),
                    snippet: repository.description.clone(),
                    metadata: Some(json!({
                        "provider": "github",
                        "candidateSource": "github_repository_search",
                        "preferredCandidateType": "project",
                        "github": {
                            "fullName": repository.full_name,
                            "stars": repository.stargazers_count,
                            "forks": repository.forks_count,
                            "language": repository.language,
                            "topics": repository.topics.unwrap_or_default(),
                            "updatedAt": repository.updated_at,
                            "pushedAt": repository.pushed_at,
                            "license": repository.license.and_then(|license| license.spdx_id)
                        }
                    })),
                })
                .await?;
            candidates.push(candidate);
        }

        Ok(GitHubDiscoveryResponse {
            research_task_id,
            provider: "github".to_string(),
            query: github_query,
            created_candidate_count: candidates.len(),
            candidates,
        })
    }

    async fn search_github_repositories(
        &self,
        query: &str,
        limit: u32,
    ) -> AppResult<Vec<GitHubRepository>> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_millis(self.config.github.timeout_ms))
            .build()
            .map_err(|err| AppError::Internal(format!("github client build failed: {err}")))?;

        let url = format!(
            "{}/search/repositories",
            self.config.github.api_base_url.trim_end_matches('/')
        );
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, HeaderValue::from_static("resource-service"));
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        if let Some(token) = &self.config.github.token {
            let value = HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_| AppError::Internal("invalid github token header".to_string()))?;
            headers.insert(AUTHORIZATION, value);
        }

        let response = client
            .get(url)
            .headers(headers)
            .query(&[
                ("q", query),
                ("sort", "stars"),
                ("order", "desc"),
                ("per_page", &limit.to_string()),
            ])
            .send()
            .await
            .map_err(|err| AppError::Internal(format!("github search failed: {err}")))?;

        if !response.status().is_success() {
            return Err(AppError::BadRequest(format!(
                "github search rejected request with status {}",
                response.status()
            )));
        }

        let body = response
            .json::<GitHubSearchResponse>()
            .await
            .map_err(|err| AppError::Internal(format!("github response decode failed: {err}")))?;
        Ok(body.items)
    }
}

fn build_github_repo_query(query: &str, language: Option<&str>, min_stars: u32) -> String {
    let mut parts = vec![
        query.trim().to_string(),
        "in:name,description,readme".to_string(),
        format!("stars:>={min_stars}"),
    ];
    if let Some(language) = language.map(str::trim).filter(|value| !value.is_empty()) {
        parts.push(format!("language:{language}"));
    }
    parts.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_github_search_query_with_resource_filters() {
        let query = build_github_repo_query("react hooks", Some("TypeScript"), 50);

        assert!(query.contains("react hooks"));
        assert!(query.contains("in:name,description,readme"));
        assert!(query.contains("stars:>=50"));
        assert!(query.contains("language:TypeScript"));
    }
}
