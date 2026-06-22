use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use rmcp::{
    ErrorData, ServerHandler,
    handler::server::{tool::ToolRouter, wrapper::Parameters},
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
    },
    tool, tool_handler, tool_router,
    transport::{
        StreamableHttpServerConfig, StreamableHttpService,
        streamable_http_server::session::local::LocalSessionManager,
    },
};
use serde_json::{Map, Value, json};
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tracing::{error, info};

use crate::{
    client::{ResourceApiClient, ResourceApiError},
    contracts,
    server::{
        config::ServerConfig,
        schema::{
            DiscoverGitHubCandidatesParam, GetResourceChunksParam, RecommendResourcesParam,
            ReportResourceGapParam, RequestResearchParam, ResourceIdParam, SearchResourcesParam,
            TopicCoverageParam,
        },
    },
    validation::{
        clamp_limit, priority_value, validate_level, validate_priority, validate_text,
        validation_error,
    },
};

#[derive(Debug, Clone)]
pub struct ResourceMcpServer {
    pub config: ServerConfig,
    pub client: Result<ResourceApiClient, Arc<ResourceApiError>>,
    #[allow(dead_code)]
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl ResourceMcpServer {
    pub fn new() -> Self {
        let config = ServerConfig::default();
        let client = ResourceApiClient::new(config.resource_service_base_url.clone())
            .map(|client| client.with_token(config.resource_service_mcp_token.clone()))
            .map_err(Arc::new);

        Self {
            config,
            client,
            tool_router: Self::tool_router(),
        }
    }

    pub async fn run(self) -> Result<()> {
        let addr = format!("{}:{}", self.config.host.clone(), self.config.port.clone());
        info!("\tStarting Resource MCP Server at: {addr}");

        let config =
            StreamableHttpServerConfig::default().with_cancellation_token(CancellationToken::new());
        let service = StreamableHttpService::new(
            move || Ok(self.clone()),
            Arc::new(LocalSessionManager::default()),
            config,
        );

        let app = Router::new().nest_service("/mcp", service);
        let listener = TcpListener::bind(&addr).await?;

        info!("\tResource MCP Server Endpoint: http://{}/mcp", addr);
        axum::serve(listener, app).await?;
        Ok(())
    }

    fn json_success(value: Value) -> CallToolResult {
        match serde_json::to_string(&value) {
            Ok(text) => CallToolResult::success(vec![Content::text(text)]),
            Err(e) => CallToolResult::error(vec![Content::text(format!(
                "Serialize resource result failed: {e}"
            ))]),
        }
    }

    async fn post_resource(&self, path: &str, body: Value) -> Value {
        match &self.client {
            Ok(client) => match client.post(path, body).await {
                Ok(value) => value,
                Err(e) => {
                    error!("\tResource API call failed: {e}");
                    e.normalized()
                }
            },
            Err(e) => e.normalized(),
        }
    }

    async fn get_resource(&self, path: &str) -> Value {
        match &self.client {
            Ok(client) => match client.get(path).await {
                Ok(value) => value,
                Err(e) => {
                    error!("\tResource API call failed: {e}");
                    e.normalized()
                }
            },
            Err(e) => e.normalized(),
        }
    }

    #[tool(description = "Search safe learning resources and chunks through Resource Service.")]
    pub async fn search_resources(
        &self,
        Parameters(param): Parameters<SearchResourcesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [SEARCH RESOURCES]");
        if let Err(error) = validate_text("query", &param.query, 300) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_level(&param.level) {
            return Ok(Self::json_success(error));
        }

        let limit = clamp_limit(param.limit, 10, 20);
        let mut filters = Map::new();
        if let Some(level) = param.level {
            filters.insert("difficulty".to_string(), Value::String(level));
        }
        if let Some(language) = param.language {
            filters.insert("language".to_string(), Value::String(language));
        }
        if let Some(types) = param.source_types {
            filters.insert(
                "resourceTypes".to_string(),
                Value::Array(types.into_iter().map(Value::String).collect()),
            );
        }

        let body = json!({
            "query": param.query,
            "limit": limit,
            "maxChunksPerResource": 2,
            "includeCoverage": true,
            "createGapOnLowConfidence": false,
            "filters": Value::Object(filters),
        });

        let data = self.post_resource("/search/resources", body).await;
        Ok(Self::json_success(project_search_response(data)))
    }

    #[tool(description = "Get a resource detail by resourceId.")]
    pub async fn get_resource_detail(
        &self,
        Parameters(param): Parameters<ResourceIdParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET RESOURCE DETAIL]");
        if param.resource_id.trim().is_empty() {
            return Ok(Self::json_success(validation_error(
                "resourceId is required.",
            )));
        }

        let data = self
            .get_resource(&format!("/resources/{}", param.resource_id))
            .await;
        Ok(Self::json_success(project_detail_response(data)))
    }

    #[tool(description = "Get bounded resource chunks for Lesson MCP context.")]
    pub async fn get_resource_chunks(
        &self,
        Parameters(param): Parameters<GetResourceChunksParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET RESOURCE CHUNKS]");
        if param.resource_id.trim().is_empty() {
            return Ok(Self::json_success(validation_error(
                "resourceId is required.",
            )));
        }

        let max_chunks = clamp_limit(param.max_chunks, 8, 20);
        let data = self
            .get_resource(&format!(
                "/resources/{}/chunks?maxChunks={}",
                param.resource_id, max_chunks
            ))
            .await;
        Ok(Self::json_success(project_chunks_response(
            &param.resource_id,
            param.chunk_ids,
            data,
        )))
    }

    #[tool(description = "Recommend resources for a roadmap or lesson topic.")]
    pub async fn recommend_resources_for_topic(
        &self,
        Parameters(param): Parameters<RecommendResourcesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [RECOMMEND RESOURCES FOR TOPIC]");
        if let Err(error) = validate_text("topic", &param.topic, 200) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_level(&param.level) {
            return Ok(Self::json_success(error));
        }

        let body = json!({
            "topic": param.topic,
            "level": param.level,
            "goal": param.goal,
            "requiredTypes": param.required_types,
            "maxResources": clamp_limit(param.max_resources, 6, 15),
            "includeChunks": true,
        });

        let data = self.post_resource("/recommend/resources", body).await;
        Ok(Self::json_success(data))
    }

    #[tool(description = "Check whether a topic has enough indexed resource coverage.")]
    pub async fn get_topic_coverage(
        &self,
        Parameters(param): Parameters<TopicCoverageParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET TOPIC COVERAGE]");
        if let Err(error) = validate_text("topic", &param.topic, 200) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_level(&param.level) {
            return Ok(Self::json_success(error));
        }

        let body = json!({
            "topic": param.topic,
            "level": param.level,
            "requiredTypes": param.required_types,
        });

        let data = self.post_resource("/coverage/topic", body).await;
        Ok(Self::json_success(data))
    }

    #[tool(description = "Report a missing resource gap without creating crawler jobs directly.")]
    pub async fn report_resource_gap(
        &self,
        Parameters(param): Parameters<ReportResourceGapParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [REPORT RESOURCE GAP]");
        if let Err(error) = validate_text("topic", &param.topic, 200) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_text("reason", &param.reason, 500) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_level(&param.level) {
            return Ok(Self::json_success(error));
        }

        let body = json!({
            "topic": param.topic,
            "level": param.level,
            "missingTypes": param.missing_types,
            "reason": param.reason,
        });

        let data = self.post_resource("/gaps", body).await;
        Ok(Self::json_success(data))
    }

    #[tool(description = "Queue a safe research task for a topic through Resource Service.")]
    pub async fn request_research_for_topic(
        &self,
        Parameters(param): Parameters<RequestResearchParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [REQUEST RESEARCH FOR TOPIC]");
        if let Err(error) = validate_text("topic", &param.topic, 200) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_level(&param.level) {
            return Ok(Self::json_success(error));
        }
        if let Err(error) = validate_priority(&param.priority) {
            return Ok(Self::json_success(error));
        }

        let body = json!({
            "topic": param.topic,
            "language": null,
            "priority": priority_value(&param.priority),
            "targetResourceTypes": param.target_resource_types,
        });

        let data = self.post_resource("/research/tasks", body).await;
        Ok(Self::json_success(data))
    }

    #[tool(
        description = "Discover GitHub repository candidates for an existing Resource research task. Creates pending candidates only; Resource Service still scores, reviews, and approves trust."
    )]
    pub async fn discover_github_candidates(
        &self,
        Parameters(param): Parameters<DiscoverGitHubCandidatesParam>,
    ) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [DISCOVER GITHUB CANDIDATES]");
        if param.research_task_id.trim().is_empty() {
            return Ok(Self::json_success(validation_error(
                "researchTaskId is required.",
            )));
        }
        if let Some(query) = &param.query {
            if let Err(error) = validate_text("query", query, 200) {
                return Ok(Self::json_success(error));
            }
        }

        let body = json!({
            "query": param.query,
            "language": param.language,
            "minStars": param.min_stars,
            "limit": clamp_limit(param.limit, 5, 10),
        });
        let data = self
            .post_resource(
                &format!("/research/tasks/{}/discover/github", param.research_task_id),
                body,
            )
            .await;
        Ok(Self::json_success(project_candidate_discovery_response(
            data,
        )))
    }

    #[tool(description = "Return the Roadmap and Lesson MCP integration contract.")]
    pub async fn get_integration_contract(&self) -> Result<CallToolResult, ErrorData> {
        info!("\tCALL TOOL: [GET INTEGRATION CONTRACT]");
        Ok(Self::json_success(contracts::integration_contract()))
    }
}

fn project_candidate_discovery_response(data: Value) -> Value {
    if is_error(&data) {
        return data;
    }

    let candidates = data
        .get("candidates")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    json!({
                        "candidateId": item.get("candidateId").cloned().unwrap_or(Value::Null),
                        "researchTaskId": item.get("researchTaskId").cloned().unwrap_or(Value::Null),
                        "url": item.get("canonicalUrl").cloned().unwrap_or_else(|| item.get("url").cloned().unwrap_or(Value::Null)),
                        "title": item.get("title").cloned().unwrap_or(Value::Null),
                        "candidateType": item.get("candidateType").cloned().unwrap_or(Value::Null),
                        "score": item.get("score").cloned().unwrap_or(Value::Null),
                        "selected": item.get("selected").cloned().unwrap_or(Value::Null),
                        "provider": item.pointer("/metadata/provider").cloned().unwrap_or(Value::Null),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "provider": data.get("provider").cloned().unwrap_or(Value::Null),
        "query": data.get("query").cloned().unwrap_or(Value::Null),
        "createdCandidateCount": data.get("createdCandidateCount").cloned().unwrap_or(Value::Null),
        "candidates": candidates,
        "policy": "Candidates are discovery inputs only. Resource Service approval and trust policy decide whether they become crawl jobs/resources.",
    })
}

impl Drop for ResourceMcpServer {
    fn drop(&mut self) {
        info!("\tShutting down Resource MCP Server");
    }
}

#[tool_handler]
impl ServerHandler for ResourceMcpServer {
    fn get_info(&self) -> ServerInfo {
        info!("\tServer info requested...");
        let instructions = [
            "Use this server to search, recommend, inspect, and report gaps for learning resources.",
            "Do not ask this server to run SQL, crawl arbitrary URLs, delete resources, or bypass review policy.",
            "Roadmap MCP should propagate partial or poor coverage instead of inventing resources.",
            "Lesson MCP should ground lessons in chunk references returned by get_resource_chunks.",
        ]
        .join(" ");

        let capabilities = ServerCapabilities::builder()
            .enable_tools()
            .enable_resources()
            .build();
        let implementation =
            Implementation::new("Resource MCP Server".to_string(), "1.0".to_string())
                .with_website_url(self.config.url.clone())
                .with_description(
                    "Safe Resource Service wrapper for Roadmap and Lesson MCP integrations."
                        .to_string(),
                )
                .with_title("Resource MCP Server".to_string());

        ServerInfo::new(capabilities)
            .with_instructions(instructions)
            .with_protocol_version(ProtocolVersion::LATEST)
            .with_server_info(implementation)
    }
}

fn project_search_response(data: Value) -> Value {
    if is_error(&data) {
        return data;
    }

    let items = data
        .get("items")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .map(|item| {
                    json!({
                        "resourceId": item.get("resourceId").cloned().unwrap_or(Value::Null),
                        "chunkId": item.get("chunkId").cloned().unwrap_or(Value::Null),
                        "title": item.get("title").cloned().unwrap_or(Value::Null),
                        "url": item.get("url").cloned().unwrap_or(Value::Null),
                        "snippet": item.get("snippet").cloned().unwrap_or(Value::Null),
                        "headingPath": item.get("headingPath").cloned().unwrap_or(Value::Array(vec![])),
                        "score": item.pointer("/scores/final").cloned().unwrap_or(Value::Null),
                        "scores": item.get("scores").cloned().unwrap_or(Value::Null),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    json!({
        "items": items,
        "coverage": data.get("coverage").cloned().unwrap_or(Value::Null),
    })
}

fn project_detail_response(data: Value) -> Value {
    if is_error(&data) {
        return data;
    }

    let resource = data.get("resource").cloned().unwrap_or(Value::Null);
    json!({
        "resourceId": resource.get("resourceId").cloned().unwrap_or(Value::Null),
        "title": resource.get("title").cloned().unwrap_or(Value::Null),
        "url": resource.get("canonicalUrl").cloned().unwrap_or(Value::Null),
        "sourceName": data.get("sourceName").cloned().unwrap_or(Value::Null),
        "sourceType": data.get("sourceType").cloned().unwrap_or(Value::Null),
        "summary": resource.get("summary").cloned().unwrap_or(Value::Null),
        "difficulty": resource.get("difficulty").cloned().unwrap_or(Value::Null),
        "qualityScore": resource.get("qualityScore").cloned().unwrap_or(Value::Null),
        "latestVersionId": data.pointer("/latestVersion/versionId").cloned().unwrap_or(Value::Null),
        "chunkCount": data.get("chunkCount").cloned().unwrap_or(Value::Null),
    })
}

fn project_chunks_response(
    resource_id: &str,
    chunk_ids: Option<Vec<String>>,
    data: Value,
) -> Value {
    if is_error(&data) {
        return data;
    }

    let mut chunks = data.as_array().cloned().unwrap_or_default();
    if let Some(ids) = chunk_ids {
        chunks.retain(|chunk| {
            chunk
                .get("chunkId")
                .and_then(Value::as_str)
                .map(|id| ids.iter().any(|requested| requested == id))
                .unwrap_or(false)
        });
    }

    let chunks = chunks
        .into_iter()
        .map(|chunk| {
            json!({
                "chunkId": chunk.get("chunkId").cloned().unwrap_or(Value::Null),
                "headingPath": chunk.get("headingPath").cloned().unwrap_or(Value::Array(vec![])),
                "content": chunk.get("content").cloned().unwrap_or(Value::Null),
                "contentKind": chunk.get("contentKind").cloned().unwrap_or(Value::Null),
                "tokenCount": chunk.get("tokenCount").cloned().unwrap_or(Value::Null),
            })
        })
        .collect::<Vec<_>>();

    json!({
        "resourceId": resource_id,
        "chunks": chunks,
    })
}

fn is_error(value: &Value) -> bool {
    value.get("ok").and_then(Value::as_bool) == Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_search_response_to_mcp_contract() {
        let data = json!({
            "items": [{
                "resourceId": "res",
                "chunkId": "chunk",
                "title": "Title",
                "url": "https://example.com",
                "snippet": "Snippet",
                "scores": {"final": 0.88}
            }],
            "coverage": {"status": "good", "lowConfidence": false}
        });

        let projected = project_search_response(data);
        assert_eq!(projected["items"][0]["score"], 0.88);
        assert_eq!(projected["coverage"]["status"], "good");
    }
}
