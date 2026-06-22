use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct LearnerContext {
    #[serde(rename = "userId")]
    pub user_id: String,
    #[serde(rename = "knownSkills")]
    pub known_skills: Vec<String>,
    #[serde(rename = "weakSkills")]
    pub weak_skills: Vec<String>,
    #[serde(rename = "completedLessons")]
    pub completed_lessons: Vec<String>,
    #[serde(rename = "completedRoadmaps")]
    pub completed_roadmaps: Vec<String>,
    #[serde(rename = "preferredLearningStyle")]
    pub preferred_learning_style: Option<String>,
    #[serde(rename = "preferredLanguage")]
    pub preferred_language: Option<String>,
    #[serde(rename = "timeAvailability")]
    pub time_availability: Option<String>,
    #[serde(rename = "currentLevelByTopic")]
    pub current_level_by_topic: BTreeMap<String, String>,
}
