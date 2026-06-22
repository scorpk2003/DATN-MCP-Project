use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::{CurrentLevel, TargetRole};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GoalCategory {
    Frontend,
    Backend,
    Fullstack,
    Devops,
    Database,
    ProgrammingLanguage,
    WebFoundation,
    SystemDesign,
    GeneralSoftware,
    CustomTopic,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GoalProfile {
    pub category: GoalCategory,
    pub domain: String,
    pub stack: Vec<String>,
    #[serde(rename = "targetRole")]
    pub target_role: Option<TargetRole>,
    pub level: CurrentLevel,
    #[serde(rename = "desiredOutcome")]
    pub desired_outcome: Option<String>,
    #[serde(rename = "normalizedGoal")]
    pub normalized_goal: String,
    pub warnings: Vec<String>,
}
