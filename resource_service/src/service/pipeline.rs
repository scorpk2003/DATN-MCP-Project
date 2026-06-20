use serde_json::json;

use crate::{
    AppResult,
    chunker::chunk_document,
    extractor::extract_document,
    models::{
        CreateResourceRequest, CreateResourceVersionRequest, ProcessFetchArtifactRequest,
        ProcessFetchArtifactResponse,
    },
};

use super::ResourceService;

impl ResourceService {
    pub async fn process_fetch_artifact(
        &self,
        request: ProcessFetchArtifactRequest,
    ) -> AppResult<ProcessFetchArtifactResponse> {
        let artifact = self
            .repository
            .get_fetch_artifact_content(request.fetch_artifact_id)
            .await?;
        let extracted = extract_document(
            &artifact.url,
            artifact.final_url.as_deref(),
            artifact.content_type.as_deref(),
            &artifact.raw_body,
        )?;
        let create_resource = CreateResourceRequest {
            title: extracted.title.clone(),
            canonical_url: extracted.canonical_url.clone(),
            source_site_id: artifact.source_id,
            language: Some(extracted.language.clone()),
            resource_type: Some("article".to_string()),
            resource_format: Some(resource_format_from_content_type(
                artifact.content_type.as_deref(),
            )),
            summary: None,
            description: extracted.description.clone(),
            metadata: Some(json!({
                "provenance": {
                    "fetchArtifactId": artifact.id,
                    "sourceUrl": extracted.source_url,
                    "finalUrl": extracted.final_url,
                    "artifactMetadata": artifact.metadata
                },
                "extractor": extracted.metadata
            })),
        };
        let resource = self.repository.create_resource(&create_resource).await?;
        let version_request = CreateResourceVersionRequest {
            title: Some(extracted.title.clone()),
            content: extracted.content.clone(),
            markdown: Some(extracted.content),
            fetch_artifact_id: Some(artifact.id),
            metadata: Some(json!({
                "extractorVersion": "resource_service_basic_extractor_v1",
                "chunkingVersion": "section_aware_v1"
            })),
        };
        let chunks = chunk_document(&version_request.content);
        let version = self
            .repository
            .create_resource_version(resource.resource_id, &version_request, &chunks)
            .await?;

        if request.activate_resource.unwrap_or(true) {
            self.repository
                .activate_resource(resource.resource_id)
                .await?;
        }

        Ok(ProcessFetchArtifactResponse {
            fetch_artifact_id: artifact.id,
            resource_id: resource.resource_id,
            version_id: version.version_id,
            chunk_count: version.chunk_count,
            action: "created_or_updated_version".to_string(),
            title: create_resource.title,
            canonical_url: create_resource.canonical_url,
        })
    }
}

fn resource_format_from_content_type(content_type: Option<&str>) -> String {
    match content_type
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .unwrap_or("text/plain")
        .to_ascii_lowercase()
        .as_str()
    {
        "text/html" => "html",
        "text/markdown" => "markdown",
        "application/json" => "plain_text",
        "text/plain" => "plain_text",
        _ => "other",
    }
    .to_string()
}
