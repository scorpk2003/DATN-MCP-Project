use anyhow::{Result, anyhow};
use serde_json::Value;

pub fn parse_llm_json_value(content: &str, label: &str) -> Result<Value> {
    let trimmed = content.trim();
    if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
        return Ok(value);
    }

    let stripped = strip_code_fence(trimmed);
    if let Ok(value) = serde_json::from_str::<Value>(stripped) {
        return Ok(value);
    }

    if let Some(json_slice) = first_balanced_json(stripped) {
        return serde_json::from_str::<Value>(json_slice).map_err(|e| {
            anyhow!(
                "{label}: invalid JSON after extracting JSON payload: {e}. preview: {}",
                preview(content)
            )
        });
    }

    Err(anyhow!(
        "{label}: no JSON object or array found. preview: {}",
        preview(content)
    ))
}

fn strip_code_fence(content: &str) -> &str {
    let Some(mut inner) = content.strip_prefix("```") else {
        return content;
    };
    if let Some(end) = inner.rfind("```") {
        inner = &inner[..end];
    }
    let inner = inner.trim();
    let json_start = inner
        .char_indices()
        .find_map(|(idx, ch)| matches!(ch, '{' | '[').then_some(idx));
    match json_start {
        Some(idx) => inner[idx..].trim(),
        None => inner,
    }
}

fn first_balanced_json(content: &str) -> Option<&str> {
    let mut start = None;
    let mut stack = Vec::new();
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in content.char_indices() {
        if start.is_none() {
            match ch {
                '{' => {
                    start = Some(idx);
                    stack.push('}');
                }
                '[' => {
                    start = Some(idx);
                    stack.push(']');
                }
                _ => continue,
            }
            continue;
        }

        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => stack.push('}'),
            '[' => stack.push(']'),
            '}' | ']' => {
                if stack.pop() != Some(ch) {
                    return None;
                }
                if stack.is_empty() {
                    let end = idx + ch.len_utf8();
                    return start.map(|start_idx| &content[start_idx..end]);
                }
            }
            _ => {}
        }
    }

    None
}

fn preview(content: &str) -> String {
    const MAX_PREVIEW_CHARS: usize = 200;
    let mut value = content.trim().replace(['\r', '\n', '\t'], " ");
    if value.chars().count() > MAX_PREVIEW_CHARS {
        value = value.chars().take(MAX_PREVIEW_CHARS).collect::<String>();
        value.push_str("...");
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_raw_json() {
        let value = parse_llm_json_value(r#"{"ok":true}"#, "TEST").unwrap();
        assert_eq!(value["ok"], true);
    }

    #[test]
    fn parses_fenced_json_with_newline() {
        let value = parse_llm_json_value("```json\n{\"ok\":true}\n```", "TEST").unwrap();
        assert_eq!(value["ok"], true);
    }

    #[test]
    fn parses_fenced_json_without_newline() {
        let value = parse_llm_json_value("```json{\"ok\":true}```", "TEST").unwrap();
        assert_eq!(value["ok"], true);
    }

    #[test]
    fn extracts_json_from_prefixed_text() {
        let value = parse_llm_json_value("content: Some(```json{\"ok\":true}```)", "TEST").unwrap();
        assert_eq!(value["ok"], true);
    }
}
