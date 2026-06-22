use rmcp::schemars::{self, JsonSchema};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct SearchResourcesParam {
    #[schemars(description = "Search query for learning resources or chunks.")]
    #[schemars(length(min = 1, max = 300))]
    pub query: String,
    #[schemars(description = "Learner level: beginner, intermediate, or advanced.")]
    pub level: Option<String>,
    #[schemars(description = "Preferred resource language, for example en or vi.")]
    pub language: Option<String>,
    #[schemars(description = "Resource type filters such as official_reference or practice.")]
    #[serde(rename = "sourceTypes")]
    pub source_types: Option<Vec<String>>,
    #[schemars(range(min = 1, max = 20))]
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ResourceIdParam {
    #[serde(rename = "resourceId")]
    pub resource_id: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct GetResourceChunksParam {
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    #[serde(rename = "chunkIds")]
    pub chunk_ids: Option<Vec<String>>,
    #[schemars(range(min = 1, max = 20))]
    #[serde(rename = "maxChunks")]
    pub max_chunks: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RecommendResourcesParam {
    #[schemars(length(min = 1, max = 200))]
    pub topic: String,
    pub level: Option<String>,
    pub goal: Option<String>,
    #[serde(rename = "requiredTypes")]
    pub required_types: Option<Vec<String>>,
    #[schemars(range(min = 1, max = 15))]
    #[serde(rename = "maxResources")]
    pub max_resources: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct TopicCoverageParam {
    #[schemars(length(min = 1, max = 200))]
    pub topic: String,
    pub level: Option<String>,
    #[serde(rename = "requiredTypes")]
    pub required_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct ReportResourceGapParam {
    #[schemars(length(min = 1, max = 200))]
    pub topic: String,
    pub level: Option<String>,
    #[serde(rename = "missingTypes")]
    pub missing_types: Vec<String>,
    #[schemars(length(min = 1, max = 500))]
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct RequestResearchParam {
    #[schemars(length(min = 1, max = 200))]
    pub topic: String,
    pub level: Option<String>,
    #[serde(rename = "targetResourceTypes")]
    pub target_resource_types: Vec<String>,
    #[schemars(description = "Research priority: low, normal, or high.")]
    pub priority: Option<String>,
}

#[derive(Debug, Clone, Deserialize, JsonSchema)]
pub struct DiscoverGitHubCandidatesParam {
    #[serde(rename = "researchTaskId")]
    pub research_task_id: String,
    #[schemars(description = "Optional GitHub search query. Defaults to the research task query.")]
    #[schemars(length(min = 1, max = 200))]
    pub query: Option<String>,
    #[schemars(
        description = "Optional repository language filter, for example Rust or TypeScript."
    )]
    pub language: Option<String>,
    #[serde(rename = "minStars")]
    pub min_stars: Option<u32>,
    #[schemars(range(min = 1, max = 10))]
    pub limit: Option<u32>,
}
