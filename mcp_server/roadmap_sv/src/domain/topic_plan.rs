use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::domain::{CurrentLevel, RoadmapNodeType};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TopicPlan {
    #[serde(rename = "topicId")]
    pub topic_id: String,
    #[serde(rename = "topicName")]
    pub topic_name: String,
    pub aliases: Vec<String>,
    pub level: CurrentLevel,
    #[serde(rename = "requiredResourceTypes")]
    pub required_resource_types: Vec<String>,
    #[serde(rename = "nodeType")]
    pub node_type: RoadmapNodeType,
    #[serde(rename = "estimatedHoursHint")]
    pub estimated_hours_hint: Option<u32>,
    #[serde(rename = "prerequisiteTopics")]
    pub prerequisite_topics: Vec<String>,
    pub optional: bool,
}
