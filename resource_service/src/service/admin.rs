use uuid::Uuid;

use crate::{
    AppResult,
    models::{AdminDashboardSummary, AdminResourceActionRequest, AdminResourceActionResponse},
};

use super::ResourceService;

impl ResourceService {
    pub async fn admin_dashboard_summary(&self) -> AppResult<AdminDashboardSummary> {
        self.repository.admin_dashboard_summary().await
    }

    pub async fn mark_resource_outdated(
        &self,
        id: Uuid,
        request: AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.repository.mark_resource_outdated(id, &request).await
    }

    pub async fn mark_resource_needs_review(
        &self,
        id: Uuid,
        request: AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.repository
            .mark_resource_needs_review(id, &request)
            .await
    }

    pub async fn boost_resource_quality(
        &self,
        id: Uuid,
        request: AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.repository.boost_resource_quality(id, &request).await
    }

    pub async fn deboost_resource_quality(
        &self,
        id: Uuid,
        request: AdminResourceActionRequest,
    ) -> AppResult<AdminResourceActionResponse> {
        self.repository.deboost_resource_quality(id, &request).await
    }
}
