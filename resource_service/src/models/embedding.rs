use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingModelRequest {
    pub provider: String,
    pub name: String,
    pub dimensions: i32,
    pub metric: Option<String>,
    #[serde(rename = "isDefault")]
    pub is_default: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EmbeddingModelSummary {
    #[serde(rename = "embeddingModelId")]
    pub id: Uuid,
    pub provider: String,
    pub name: String,
    pub dimensions: i32,
    pub metric: String,
    #[serde(rename = "isDefault")]
    pub is_default: bool,
}

impl EmbeddingModelSummary {
    pub fn supports_inline_pgvector(&self) -> bool {
        self.dimensions == 1536
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PendingEmbeddingChunksQuery {
    #[serde(rename = "embeddingModelId")]
    pub embedding_model_id: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PendingEmbeddingChunk {
    #[serde(rename = "chunkId")]
    pub chunk_id: Uuid,
    #[serde(rename = "resourceId")]
    pub resource_id: Uuid,
    #[serde(rename = "resourceVersionId")]
    pub version_id: Uuid,
    #[serde(rename = "embeddingModelId")]
    pub embedding_model_id: Uuid,
    pub title: String,
    #[serde(rename = "headingPath")]
    pub heading_path: Vec<String>,
    #[serde(rename = "inputText")]
    pub input_text: String,
    #[serde(rename = "tokenCount")]
    pub token_count: Option<i32>,
}
