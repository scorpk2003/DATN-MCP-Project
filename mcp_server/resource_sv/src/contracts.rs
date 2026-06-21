use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatusKind {
    Good,
    Partial,
    Poor,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ResourceRole {
    OfficialReference,
    PrimaryLearning,
    Practice,
    Project,
    DeepDive,
    Troubleshooting,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoverageStatusContract {
    pub status: CoverageStatusKind,
    #[serde(rename = "lowConfidence")]
    pub low_confidence: bool,
    #[serde(rename = "missingTypes")]
    pub missing_types: Vec<String>,
    #[serde(rename = "resultCount")]
    pub result_count: u32,
    #[serde(rename = "bestScore")]
    pub best_score: f64,
    #[serde(rename = "gapId")]
    pub gap_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RoadmapResourceRef {
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    #[serde(rename = "chunkIds")]
    pub chunk_ids: Option<Vec<String>>,
    pub role: ResourceRole,
    pub reason: String,
    pub score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LessonContextRequest {
    pub topic: String,
    pub concept: Option<String>,
    pub level: String,
    #[serde(rename = "resourceRefs")]
    pub resource_refs: Vec<RoadmapResourceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LessonChunk {
    #[serde(rename = "chunkId")]
    pub chunk_id: String,
    #[serde(rename = "headingPath")]
    pub heading_path: Vec<String>,
    pub content: String,
    #[serde(rename = "contentKind")]
    pub content_kind: String,
    #[serde(rename = "tokenCount")]
    pub token_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct LessonChunkContext {
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    pub chunks: Vec<LessonChunk>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResourceTelemetryEvent {
    #[serde(rename = "eventType")]
    pub event_type: String,
    #[serde(rename = "resourceId")]
    pub resource_id: Option<String>,
    #[serde(rename = "chunkId")]
    pub chunk_id: Option<String>,
    pub topic: Option<String>,
    pub outcome: Option<String>,
}

pub fn integration_contract() -> Value {
    let coverage = CoverageStatusContract {
        status: CoverageStatusKind::Partial,
        low_confidence: true,
        missing_types: vec!["practice".to_string()],
        result_count: 3,
        best_score: 0.74,
        gap_id: Some("gap_uuid".to_string()),
    };
    let resource_ref = RoadmapResourceRef {
        resource_id: "resource_uuid".to_string(),
        chunk_ids: Some(vec!["chunk_uuid".to_string()]),
        role: ResourceRole::PrimaryLearning,
        reason: "Grounds this module in an indexed learning resource.".to_string(),
        score: 0.82,
    };
    let lesson_request = LessonContextRequest {
        topic: "React useEffect cleanup".to_string(),
        concept: Some("effect cleanup".to_string()),
        level: "beginner".to_string(),
        resource_refs: vec![resource_ref.clone()],
    };
    let lesson_context = LessonChunkContext {
        resource_id: resource_ref.resource_id.clone(),
        chunks: vec![LessonChunk {
            chunk_id: "chunk_uuid".to_string(),
            heading_path: vec!["Effects".to_string(), "Cleanup".to_string()],
            content: "Bounded chunk content returned only to Lesson MCP context.".to_string(),
            content_kind: "concept".to_string(),
            token_count: 128,
        }],
    };
    let telemetry = ResourceTelemetryEvent {
        event_type: "lesson_resource_used".to_string(),
        resource_id: Some(resource_ref.resource_id.clone()),
        chunk_id: Some("chunk_uuid".to_string()),
        topic: Some("React useEffect cleanup".to_string()),
        outcome: Some("completed".to_string()),
    };

    json!({
        "roadmapTools": [
            "recommend_resources_for_topic",
            "get_topic_coverage",
            "request_research_for_topic"
        ],
        "lessonTools": [
            "get_resource_chunks",
            "search_resources",
            "get_resource_detail",
            "recommend_resources_for_topic"
        ],
        "coverageBehavior": {
            "good": "Build normally and attach resource references.",
            "partial": "Build with resourceCoverage=partial and request research for important missing types.",
            "poor": "Return a partial roadmap or lesson plan, request research, and avoid hallucinated resources."
        },
        "contracts": {
            "coverage": coverage,
            "roadmapResourceRef": resource_ref,
            "lessonContextRequest": lesson_request,
            "lessonChunkContext": lesson_context,
            "telemetryEvent": telemetry
        },
        "fallback": {
            "resourceServiceUnavailable": "Return ok=false with RESOURCE_API_UNAVAILABLE and keep roadmap or lesson status partial.",
            "lowConfidence": "Propagate coverage.lowConfidence to orchestrator and user-facing planning state."
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roadmap_resource_ref_serializes_as_reference_only() {
        let value = serde_json::to_value(RoadmapResourceRef {
            resource_id: "res_1".to_string(),
            chunk_ids: Some(vec!["chk_1".to_string()]),
            role: ResourceRole::OfficialReference,
            reason: "Correctness reference.".to_string(),
            score: 0.91,
        })
        .unwrap();

        assert_eq!(value["resourceId"], "res_1");
        assert!(value.get("content").is_none());
    }
}
