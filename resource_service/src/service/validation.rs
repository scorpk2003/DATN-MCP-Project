use crate::{
    AppError, AppResult,
    models::{CreateResourceRequest, ManualIngestRequest, SearchRequest},
};

pub(crate) fn validate_resource_create(request: &CreateResourceRequest) -> AppResult<()> {
    if request.title.trim().is_empty() {
        return Err(AppError::Validation("title is required".to_string()));
    }
    validate_http_url(&request.canonical_url, "canonicalUrl")
}

pub(crate) fn validate_ingest_request(request: &ManualIngestRequest) -> AppResult<()> {
    if request.title.trim().is_empty() {
        return Err(AppError::Validation("title is required".to_string()));
    }
    if request.content.trim().len() < 20 {
        return Err(AppError::Validation(
            "content must contain at least 20 non-whitespace characters".to_string(),
        ));
    }
    validate_http_url(&request.canonical_url, "canonical_url")
}

pub(crate) fn validate_search_request(request: &SearchRequest) -> AppResult<()> {
    if request.query.trim().is_empty() {
        return Err(AppError::Validation("query is required".to_string()));
    }
    Ok(())
}

pub(crate) fn validate_http_url(raw_url: &str, field: &str) -> AppResult<()> {
    let url = url::Url::parse(raw_url)
        .map_err(|_| AppError::Validation(format!("{field} must be a valid URL")))?;
    if !matches!(url.scheme(), "http" | "https") {
        return Err(AppError::Validation(format!(
            "{field} must use http or https"
        )));
    }
    Ok(())
}
