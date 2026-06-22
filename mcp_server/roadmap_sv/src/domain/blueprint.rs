use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::{CurrentLevel, GoalCategory, TargetRole};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlueprintPhaseTemplate {
    #[serde(rename = "phaseId")]
    pub phase_id: String,
    pub title: String,
    pub purpose: String,
    #[serde(rename = "topicGroupIds")]
    pub topic_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlueprintTopicGroup {
    #[serde(rename = "groupId")]
    pub group_id: String,
    pub title: String,
    pub topics: Vec<String>,
    #[serde(rename = "requiredResourceTypes")]
    pub required_resource_types: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrerequisiteRule {
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EstimatedHoursRange {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapBlueprint {
    #[serde(rename = "blueprintId")]
    pub blueprint_id: String,
    pub domain: GoalCategory,
    #[serde(rename = "targetRole")]
    pub target_role: Option<TargetRole>,
    pub level: CurrentLevel,
    pub phases: Vec<BlueprintPhaseTemplate>,
    #[serde(rename = "topicGroups")]
    pub topic_groups: Vec<BlueprintTopicGroup>,
    #[serde(rename = "prerequisiteRules")]
    pub prerequisite_rules: Vec<PrerequisiteRule>,
    #[serde(rename = "defaultRequiredResourceTypes")]
    pub default_required_resource_types: Vec<String>,
    #[serde(rename = "estimatedHoursRange")]
    pub estimated_hours_range: EstimatedHoursRange,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BlueprintSelection {
    pub blueprint: RoadmapBlueprint,
    #[serde(rename = "selectionReason")]
    pub selection_reason: String,
    pub warnings: Vec<String>,
}
