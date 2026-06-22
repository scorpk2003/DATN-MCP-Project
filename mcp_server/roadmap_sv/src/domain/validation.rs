use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::{GoalProfile, RoadmapGenerationRequest};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationIssue {
    pub code: String,
    pub message: String,
    pub field: Option<String>,
    pub details: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
    #[serde(rename = "qualityScore")]
    pub quality_score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapRequestValidationOutput {
    pub valid: bool,
    #[serde(rename = "normalizedRequest")]
    pub normalized_request: Option<RoadmapGenerationRequest>,
    #[serde(rename = "goalProfile")]
    pub goal_profile: Option<GoalProfile>,
    #[serde(rename = "validationErrors")]
    pub validation_errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapError {
    pub code: String,
    pub message: String,
    pub details: Option<Value>,
    pub retryable: bool,
}
