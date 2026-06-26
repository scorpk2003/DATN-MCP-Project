use serde_json::json;

use crate::{
    domain::AuthContext,
    error::{LessonErrorCode, LessonToolError},
};

pub fn require_verified_access(
    auth_context: &Option<AuthContext>,
    expected_user_id: Option<&str>,
    required_scope: &str,
    resource_type: &str,
    resource_id: Option<&str>,
) -> Result<(), LessonToolError> {
    let auth_context = auth_context.as_ref().ok_or_else(|| {
        permission_error(
            "Missing verified auth context.",
            None,
            expected_user_id,
            required_scope,
            resource_type,
            resource_id,
        )
    })?;

    if !auth_context.verified {
        return Err(permission_error(
            "Auth context is not verified.",
            Some(&auth_context.user_id),
            expected_user_id,
            required_scope,
            resource_type,
            resource_id,
        ));
    }

    if let Some(expected_user_id) = expected_user_id {
        if auth_context.user_id != expected_user_id {
            return Err(permission_error(
                "Auth context user does not match request user.",
                Some(&auth_context.user_id),
                Some(expected_user_id),
                required_scope,
                resource_type,
                resource_id,
            ));
        }
    }

    if !has_scope(&auth_context.scope, required_scope) {
        return Err(permission_error(
            "Auth context does not include the required scope.",
            Some(&auth_context.user_id),
            expected_user_id,
            required_scope,
            resource_type,
            resource_id,
        ));
    }

    Ok(())
}

fn has_scope(scopes: &[String], required_scope: &str) -> bool {
    scopes.iter().any(|scope| {
        scope == "*"
            || scope == required_scope
            || scope == "lesson:*"
            || (required_scope.starts_with("roadmap:") && scope == "roadmap:*")
    })
}

fn permission_error(
    message: &str,
    auth_user_id: Option<&str>,
    expected_user_id: Option<&str>,
    required_scope: &str,
    resource_type: &str,
    resource_id: Option<&str>,
) -> LessonToolError {
    LessonToolError::new(
        LessonErrorCode::PermissionDenied,
        message,
        json!({
            "authUserId": auth_user_id,
            "expectedUserId": expected_user_id,
            "requiredScope": required_scope,
            "resourceType": resource_type,
            "resourceId": resource_id,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn auth(user_id: &str, scope: Vec<&str>) -> Option<AuthContext> {
        Some(AuthContext {
            user_id: user_id.to_string(),
            verified: true,
            scope: scope.into_iter().map(ToString::to_string).collect(),
            verified_by: Some("database_mcp".to_string()),
            verified_at: Some("2026-06-26T00:00:00Z".to_string()),
        })
    }

    #[test]
    fn allows_verified_matching_user_with_scope() {
        assert!(
            require_verified_access(
                &auth("user-a", vec!["lesson:write"]),
                Some("user-a"),
                "lesson:write",
                "lesson",
                Some("lesson-a"),
            )
            .is_ok()
        );
    }

    #[test]
    fn denies_missing_auth_context() {
        let error = require_verified_access(
            &None,
            Some("user-a"),
            "lesson:write",
            "lesson",
            Some("lesson-a"),
        )
        .unwrap_err();

        assert_eq!(error.code, LessonErrorCode::PermissionDenied);
    }

    #[test]
    fn denies_mismatched_user() {
        let error = require_verified_access(
            &auth("user-a", vec!["lesson:write"]),
            Some("user-b"),
            "lesson:write",
            "lesson",
            Some("lesson-b"),
        )
        .unwrap_err();

        assert_eq!(error.code, LessonErrorCode::PermissionDenied);
    }

    #[test]
    fn denies_missing_required_scope() {
        let error = require_verified_access(
            &auth("user-a", vec!["roadmap:read"]),
            Some("user-a"),
            "lesson:progress",
            "session",
            Some("session-a"),
        )
        .unwrap_err();

        assert_eq!(error.code, LessonErrorCode::PermissionDenied);
    }
}
