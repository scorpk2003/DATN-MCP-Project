use crate::{
    AppError, AppResult,
    models::{
        EmbeddingModelRequest, EmbeddingModelSummary, PendingEmbeddingChunk,
        PendingEmbeddingChunksQuery,
    },
};

use super::ResourceService;

impl ResourceService {
    pub async fn create_embedding_model(
        &self,
        request: EmbeddingModelRequest,
    ) -> AppResult<EmbeddingModelSummary> {
        if request.provider.trim().is_empty() {
            return Err(AppError::Validation("provider is required".to_string()));
        }
        if request.name.trim().is_empty() {
            return Err(AppError::Validation("name is required".to_string()));
        }
        if request.dimensions <= 0 {
            return Err(AppError::Validation(
                "dimensions must be greater than zero".to_string(),
            ));
        }
        if let Some(metric) = &request.metric {
            if !matches!(metric.as_str(), "cosine" | "l2" | "ip") {
                return Err(AppError::Validation(
                    "metric must be cosine, l2, or ip".to_string(),
                ));
            }
        }
        self.repository.create_embedding_model(&request).await
    }

    pub async fn list_embedding_models(&self) -> AppResult<Vec<EmbeddingModelSummary>> {
        self.repository.list_embedding_models().await
    }

    pub async fn list_pending_embedding_chunks(
        &self,
        query: PendingEmbeddingChunksQuery,
    ) -> AppResult<Vec<PendingEmbeddingChunk>> {
        self.repository.list_pending_embedding_chunks(&query).await
    }
}
