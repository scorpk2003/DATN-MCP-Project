use uuid::Uuid;

use crate::{
    AppError, AppResult,
    chunker::chunk_document,
    models::{
        CreateResourceRequest, CreateResourceResponse, CreateResourceVersionRequest,
        HealthResponse, IngestResourceResponse, ManualIngestRequest, Page, PageQuery,
        ResourceChunk, ResourceDetail, ResourceSummary, ResourceVersionSummary,
        UpdateResourceRequest,
    },
};

use super::{ResourceService, validation};

impl ResourceService {
    pub async fn health_check(&self) -> AppResult<HealthResponse> {
        self.repository.health_check().await
    }

    pub async fn migrate(&self) -> AppResult<()> {
        self.repository.run_schema_migration().await
    }

    pub async fn create_resource(
        &self,
        request: CreateResourceRequest,
    ) -> AppResult<CreateResourceResponse> {
        validation::validate_resource_create(&request)?;
        self.repository.create_resource(&request).await
    }

    pub async fn update_resource(
        &self,
        id: Uuid,
        request: UpdateResourceRequest,
    ) -> AppResult<ResourceDetail> {
        if let Some(score) = request.quality_score {
            if !(0.0..=1.0).contains(&score) {
                return Err(AppError::Validation(
                    "qualityScore must be between 0 and 1".to_string(),
                ));
            }
        }
        self.repository.update_resource(id, &request).await
    }

    pub async fn create_resource_version(
        &self,
        resource_id: Uuid,
        request: CreateResourceVersionRequest,
    ) -> AppResult<IngestResourceResponse> {
        if request.content.trim().len() < 20 {
            return Err(AppError::Validation(
                "content must contain at least 20 non-whitespace characters".to_string(),
            ));
        }
        let chunks = chunk_document(&request.content);
        self.repository
            .create_resource_version(resource_id, &request, &chunks)
            .await
    }

    pub async fn ingest_manual(
        &self,
        request: ManualIngestRequest,
    ) -> AppResult<IngestResourceResponse> {
        validation::validate_ingest_request(&request)?;
        let chunks = chunk_document(&request.content);
        if chunks.is_empty() {
            return Err(AppError::Validation(
                "content must produce at least one searchable chunk".to_string(),
            ));
        }
        self.repository.ingest_manual(&request, &chunks).await
    }

    pub async fn list_resources(&self, query: PageQuery) -> AppResult<Page<ResourceSummary>> {
        self.repository.list_resources(&query).await
    }

    pub async fn get_resource_detail(&self, id: Uuid) -> AppResult<ResourceDetail> {
        self.repository.get_resource_detail(id).await
    }

    pub async fn list_versions(&self, id: Uuid) -> AppResult<Vec<ResourceVersionSummary>> {
        self.repository.list_versions(id).await
    }

    pub async fn get_resource_chunks(
        &self,
        id: Uuid,
        version_id: Option<Uuid>,
        max_chunks: Option<i64>,
    ) -> AppResult<Vec<ResourceChunk>> {
        self.repository
            .get_resource_chunks(id, version_id, max_chunks.unwrap_or(50).clamp(1, 200))
            .await
    }
}
