use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{CoverageStatus, NodeStatus, TopicPlan};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoverageRole {
    Primary,
    Reference,
    Practice,
    Optional,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SelectedChunkRef {
    #[serde(rename = "chunkId")]
    pub chunk_id: String,
    #[serde(rename = "headingPath")]
    pub heading_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceBinding {
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    pub title: String,
    #[serde(rename = "canonicalUrl")]
    pub canonical_url: String,
    #[serde(rename = "sourceDomain")]
    pub source_domain: Option<String>,
    pub kind: String,
    pub format: Option<String>,
    #[serde(rename = "languageCode")]
    pub language_code: Option<String>,
    #[serde(rename = "isOfficial")]
    pub is_official: bool,
    #[serde(rename = "qualityScore")]
    pub quality_score: Option<f64>,
    #[serde(rename = "trustTier")]
    pub trust_tier: Option<u8>,
    #[serde(rename = "coverageRole")]
    pub coverage_role: CoverageRole,
    #[serde(rename = "selectedChunks")]
    pub selected_chunks: Option<Vec<SelectedChunkRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoverageCheckResult {
    #[serde(rename = "coverageStatus")]
    pub coverage_status: CoverageStatus,
    #[serde(rename = "availableTypes")]
    pub available_types: Vec<String>,
    #[serde(rename = "missingTypes")]
    pub missing_types: Vec<String>,
    pub confidence: Option<f64>,
    #[serde(rename = "candidateResourceCount")]
    pub candidate_resource_count: Option<u32>,
    #[serde(rename = "gapId")]
    pub gap_id: Option<String>,
    pub raw: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundTopicPlan {
    #[serde(rename = "topicPlan")]
    pub topic_plan: TopicPlan,
    pub coverage: CoverageCheckResult,
    #[serde(rename = "resourceRefs")]
    pub resource_refs: Vec<ResourceBinding>,
    #[serde(rename = "missingResourceTypes")]
    pub missing_resource_types: Vec<String>,
    pub warnings: Vec<String>,
    pub status: NodeStatus,
    #[serde(rename = "gapReported")]
    pub gap_reported: bool,
    #[serde(rename = "researchRequested")]
    pub research_requested: bool,
}
