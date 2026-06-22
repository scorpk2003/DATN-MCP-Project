use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CurrentLevel {
    Beginner,
    Intermediate,
    Advanced,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TargetRole {
    Frontend,
    Backend,
    Fullstack,
    Devops,
    Data,
    AiMl,
    GeneralSoftware,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SaveMode {
    Draft,
    Active,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TimeBudget {
    #[serde(rename = "hoursPerWeek")]
    pub hours_per_week: Option<u32>,
    #[serde(rename = "targetWeeks")]
    pub target_weeks: Option<u32>,
    #[serde(rename = "maxTotalHours")]
    pub max_total_hours: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapConstraints {
    #[serde(rename = "preferOfficialDocs")]
    pub prefer_official_docs: Option<bool>,
    #[serde(rename = "preferProjectBased")]
    pub prefer_project_based: Option<bool>,
    #[serde(rename = "includePractice")]
    pub include_practice: Option<bool>,
    #[serde(rename = "avoidAdvancedMath")]
    pub avoid_advanced_math: Option<bool>,
    #[serde(rename = "targetStack")]
    pub target_stack: Option<Vec<String>>,
    #[serde(rename = "excludedTopics")]
    pub excluded_topics: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapGenerationRequest {
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "learningGoal")]
    pub learning_goal: String,
    #[serde(rename = "currentLevel")]
    pub current_level: Option<CurrentLevel>,
    #[serde(rename = "targetRole")]
    pub target_role: Option<TargetRole>,
    #[serde(rename = "preferredLanguage")]
    pub preferred_language: Option<String>,
    #[serde(rename = "timeBudget")]
    pub time_budget: Option<TimeBudget>,
    pub constraints: Option<RoadmapConstraints>,
    #[serde(rename = "saveMode")]
    pub save_mode: Option<SaveMode>,
}
