use serde_json::json;

use crate::{
    domain::{
        CurrentLevel, RoadmapGenerationRequest, RoadmapRequestValidationOutput, SaveMode,
        ValidationIssue,
    },
    services::goal_normalizer::{compact_whitespace, normalize_goal},
};

pub fn validate_roadmap_request(
    request: RoadmapGenerationRequest,
) -> RoadmapRequestValidationOutput {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if request.learning_goal.trim().is_empty() {
        errors.push(issue(
            "GOAL_REQUIRED",
            "learningGoal is required.",
            Some("learningGoal"),
        ));
    }
    if request.learning_goal.chars().count() > 500 {
        errors.push(issue(
            "GOAL_TOO_LONG",
            "learningGoal must be 500 characters or fewer.",
            Some("learningGoal"),
        ));
    }

    validate_time_budget(&request, &mut errors, &mut warnings);
    validate_constraints(&request, &mut errors, &mut warnings);

    if !errors.is_empty() {
        return RoadmapRequestValidationOutput {
            valid: false,
            normalized_request: None,
            goal_profile: None,
            validation_errors: errors,
            warnings,
        };
    }

    let normalized_request = normalize_request(request, &mut warnings);
    let goal_profile = normalize_goal(&normalized_request);

    for warning in &goal_profile.warnings {
        warnings.push(issue(
            "GOAL_NORMALIZATION_WARNING",
            warning,
            Some("learningGoal"),
        ));
    }

    RoadmapRequestValidationOutput {
        valid: true,
        normalized_request: Some(normalized_request),
        goal_profile: Some(goal_profile),
        validation_errors: errors,
        warnings,
    }
}

fn normalize_request(
    mut request: RoadmapGenerationRequest,
    warnings: &mut Vec<ValidationIssue>,
) -> RoadmapGenerationRequest {
    request.learning_goal = compact_whitespace(&request.learning_goal);
    if request.current_level.is_none() {
        request.current_level = Some(CurrentLevel::Unknown);
        warnings.push(issue(
            "DEFAULT_CURRENT_LEVEL",
            "currentLevel was not supplied; defaulted to unknown.",
            Some("currentLevel"),
        ));
    }
    if request
        .preferred_language
        .as_deref()
        .unwrap_or("")
        .trim()
        .is_empty()
    {
        request.preferred_language = Some("en".to_string());
        warnings.push(issue(
            "DEFAULT_LANGUAGE",
            "preferredLanguage was not supplied; defaulted to en.",
            Some("preferredLanguage"),
        ));
    }
    if request.save_mode.is_none() {
        request.save_mode = Some(SaveMode::Draft);
    }

    request
}

fn validate_time_budget(
    request: &RoadmapGenerationRequest,
    errors: &mut Vec<ValidationIssue>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let Some(time_budget) = &request.time_budget else {
        warnings.push(issue(
            "NO_TIME_BUDGET",
            "No strict time budget supplied.",
            Some("timeBudget"),
        ));
        return;
    };

    if matches!(time_budget.hours_per_week, Some(0 | 81..)) {
        errors.push(issue(
            "INVALID_HOURS_PER_WEEK",
            "hoursPerWeek must be between 1 and 80.",
            Some("timeBudget.hoursPerWeek"),
        ));
    }
    if matches!(time_budget.target_weeks, Some(0 | 105..)) {
        errors.push(issue(
            "INVALID_TARGET_WEEKS",
            "targetWeeks must be between 1 and 104.",
            Some("timeBudget.targetWeeks"),
        ));
    }
    if matches!(time_budget.max_total_hours, Some(0 | 2001..)) {
        errors.push(issue(
            "INVALID_MAX_TOTAL_HOURS",
            "maxTotalHours must be between 1 and 2000.",
            Some("timeBudget.maxTotalHours"),
        ));
    }

    if let (Some(hours_per_week), Some(target_weeks), Some(max_total_hours)) = (
        time_budget.hours_per_week,
        time_budget.target_weeks,
        time_budget.max_total_hours,
    ) {
        let total = hours_per_week.saturating_mul(target_weeks);
        if total > max_total_hours {
            errors.push(ValidationIssue {
                code: "CONTRADICTORY_TIME_BUDGET".to_string(),
                message: "hoursPerWeek multiplied by targetWeeks exceeds maxTotalHours."
                    .to_string(),
                field: Some("timeBudget".to_string()),
                details: Some(json!({
                    "computedTotalHours": total,
                    "maxTotalHours": max_total_hours,
                })),
            });
        }
    }
}

fn validate_constraints(
    request: &RoadmapGenerationRequest,
    errors: &mut Vec<ValidationIssue>,
    warnings: &mut Vec<ValidationIssue>,
) {
    let Some(constraints) = &request.constraints else {
        return;
    };

    if constraints.prefer_project_based == Some(true) && constraints.include_practice == Some(false)
    {
        warnings.push(issue(
            "PROJECT_WITHOUT_PRACTICE",
            "preferProjectBased is true while includePractice is false; project nodes may be limited.",
            Some("constraints"),
        ));
    }

    if let (Some(target_stack), Some(excluded_topics)) =
        (&constraints.target_stack, &constraints.excluded_topics)
    {
        for stack_item in target_stack {
            if excluded_topics
                .iter()
                .any(|excluded| excluded.eq_ignore_ascii_case(stack_item))
            {
                errors.push(issue(
                    "CONTRADICTORY_CONSTRAINTS",
                    "A targetStack item is also listed in excludedTopics.",
                    Some("constraints"),
                ));
            }
        }
    }
}

fn issue(code: &str, message: &str, field: Option<&str>) -> ValidationIssue {
    ValidationIssue {
        code: code.to_string(),
        message: message.to_string(),
        field: field.map(str::to_string),
        details: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{RoadmapConstraints, TimeBudget};

    #[test]
    fn returns_normalized_valid_request() {
        let result = validate_roadmap_request(RoadmapGenerationRequest {
            user_id: None,
            learning_goal: "learn React basics".to_string(),
            current_level: None,
            target_role: None,
            preferred_language: None,
            time_budget: None,
            constraints: None,
            save_mode: None,
        });

        assert!(result.valid);
        assert_eq!(
            result.normalized_request.unwrap().preferred_language,
            Some("en".to_string())
        );
        assert!(result.goal_profile.is_some());
    }

    #[test]
    fn rejects_contradictory_time_budget() {
        let result = validate_roadmap_request(RoadmapGenerationRequest {
            user_id: None,
            learning_goal: "learn backend".to_string(),
            current_level: None,
            target_role: None,
            preferred_language: None,
            time_budget: Some(TimeBudget {
                hours_per_week: Some(10),
                target_weeks: Some(10),
                max_total_hours: Some(40),
            }),
            constraints: None,
            save_mode: None,
        });

        assert!(!result.valid);
        assert_eq!(
            result.validation_errors[0].code,
            "CONTRADICTORY_TIME_BUDGET"
        );
    }

    #[test]
    fn rejects_stack_excluded_topic_overlap() {
        let result = validate_roadmap_request(RoadmapGenerationRequest {
            user_id: None,
            learning_goal: "learn backend".to_string(),
            current_level: None,
            target_role: None,
            preferred_language: None,
            time_budget: None,
            constraints: Some(RoadmapConstraints {
                prefer_official_docs: None,
                prefer_project_based: None,
                include_practice: None,
                avoid_advanced_math: None,
                target_stack: Some(vec!["Node.js".to_string()]),
                excluded_topics: Some(vec!["node.js".to_string()]),
            }),
            save_mode: None,
        });

        assert!(!result.valid);
        assert_eq!(
            result.validation_errors[0].code,
            "CONTRADICTORY_CONSTRAINTS"
        );
    }
}
