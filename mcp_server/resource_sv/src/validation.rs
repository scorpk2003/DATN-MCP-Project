use serde_json::{Value, json};

pub fn validate_text(name: &str, value: &str, max_len: usize) -> Result<(), Value> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(validation_error(format!("{name} is required.")));
    }
    if trimmed.len() > max_len {
        return Err(validation_error(format!("{name} is too long.")));
    }
    if contains_forbidden_intent(trimmed) {
        return Err(validation_error(
            "This Resource MCP tool does not execute SQL, crawl arbitrary URLs, or perform admin actions.",
        ));
    }
    Ok(())
}

pub fn validate_level(level: &Option<String>) -> Result<(), Value> {
    if let Some(level) = level {
        let allowed = ["beginner", "intermediate", "advanced"];
        if !allowed.contains(&level.as_str()) {
            return Err(validation_error(
                "level must be beginner, intermediate, or advanced.",
            ));
        }
    }
    Ok(())
}

pub fn validate_priority(priority: &Option<String>) -> Result<(), Value> {
    if let Some(priority) = priority {
        let allowed = ["low", "normal", "high"];
        if !allowed.contains(&priority.as_str()) {
            return Err(validation_error("priority must be low, normal, or high."));
        }
    }
    Ok(())
}

pub fn clamp_limit(limit: Option<u32>, default: u32, max: u32) -> u32 {
    limit.unwrap_or(default).clamp(1, max)
}

pub fn priority_value(priority: &Option<String>) -> i32 {
    match priority.as_deref() {
        Some("low") => 2,
        Some("high") => 10,
        _ => 5,
    }
}

pub fn validation_error(message: impl Into<String>) -> Value {
    json!({"ok": false, "error": {"code": "VALIDATION_ERROR", "message": message.into()}})
}

fn contains_forbidden_intent(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    [
        "run_sql",
        "crawl_any_url",
        "drop table",
        "truncate table",
        "delete from",
        "approve_candidate_without_policy",
        "modify_quality_score_directly",
    ]
    .iter()
    .any(|needle| lower.contains(needle))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_dangerous_tool_intent() {
        assert!(validate_text("query", "please run_sql drop table resources", 300).is_err());
    }

    #[test]
    fn clamps_tool_limits() {
        assert_eq!(clamp_limit(Some(100), 10, 20), 20);
        assert_eq!(clamp_limit(None, 10, 20), 10);
    }
}
