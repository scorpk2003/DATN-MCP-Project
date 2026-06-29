use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{CurrentLevel, ResourceBinding, ValidationResult};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoadmapStatus {
    Draft,
    Active,
    Incomplete,
    NeedsResourceBackfill,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RoadmapNodeType {
    Foundation,
    Concept,
    Skill,
    Practice,
    Project,
    Checkpoint,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Good,
    Partial,
    Poor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NodeStatus {
    Ready,
    Partial,
    Blocked,
    Placeholder,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Prerequisite,
    RecommendedBefore,
    OptionalBefore,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapGraph {
    #[serde(rename = "roadmapId")]
    pub roadmap_id: Option<String>,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "projectId")]
    pub project_id: Option<String>,
    pub status: RoadmapStatus,
    pub metadata: Value,
    pub phases: Vec<RoadmapPhase>,
    pub nodes: Vec<RoadmapNode>,
    pub edges: Vec<RoadmapEdge>,
    #[serde(rename = "coverageSummary")]
    pub coverage_summary: CoverageSummary,
    #[serde(rename = "resourceSummary")]
    pub resource_summary: Value,
    #[serde(rename = "gapWarnings")]
    pub gap_warnings: Vec<String>,
    #[serde(rename = "validationResult")]
    pub validation_result: Option<ValidationResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapPhase {
    #[serde(rename = "phaseId")]
    pub phase_id: String,
    pub title: String,
    pub purpose: String,
    #[serde(rename = "orderIndex")]
    pub order_index: u32,
    #[serde(rename = "estimatedHours")]
    pub estimated_hours: u32,
    #[serde(rename = "nodeIds")]
    pub node_ids: Vec<String>,
    #[serde(rename = "exitCriteria")]
    pub exit_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapNode {
    #[serde(rename = "nodeId")]
    pub node_id: String,
    #[serde(rename = "phaseId")]
    pub phase_id: String,
    pub title: String,
    pub topic: String,
    pub aliases: Vec<String>,
    #[serde(rename = "nodeType")]
    pub node_type: RoadmapNodeType,
    pub level: CurrentLevel,
    pub purpose: String,
    #[serde(rename = "learningOutcomes")]
    pub learning_outcomes: Vec<String>,
    pub prerequisites: Vec<String>,
    #[serde(rename = "estimatedHours")]
    pub estimated_hours: u32,
    #[serde(rename = "coverageStatus")]
    pub coverage_status: CoverageStatus,
    #[serde(rename = "resourceRefs")]
    pub resource_refs: Vec<ResourceBinding>,
    #[serde(rename = "missingResourceTypes")]
    pub missing_resource_types: Vec<String>,
    pub warnings: Vec<String>,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapEdge {
    #[serde(rename = "fromNodeId")]
    pub from_node_id: String,
    #[serde(rename = "toNodeId")]
    pub to_node_id: String,
    #[serde(rename = "edgeType")]
    pub edge_type: EdgeType,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct CoverageSummary {
    #[serde(rename = "totalTopics")]
    pub total_topics: u32,
    #[serde(rename = "coverageGood")]
    pub coverage_good: u32,
    #[serde(rename = "coveragePartial")]
    pub coverage_partial: u32,
    #[serde(rename = "coveragePoor")]
    pub coverage_poor: u32,
    #[serde(rename = "missingOfficialReferenceCount")]
    pub missing_official_reference_count: u32,
    #[serde(rename = "missingPracticeCount")]
    pub missing_practice_count: u32,
    #[serde(rename = "missingProjectCount")]
    pub missing_project_count: u32,
    #[serde(rename = "gapsCreated")]
    pub gaps_created: u32,
    #[serde(rename = "researchTasksRequested")]
    pub research_tasks_requested: u32,
    #[serde(rename = "readyForLessonGeneration")]
    pub ready_for_lesson_generation: bool,
}
