use serde::Serialize;
use serde_json::{Value, json};
use url::Url;

use crate::{AppError, AppResult};

#[derive(Debug, Clone, Serialize)]
pub struct ExtractedDocument {
    pub source_url: String,
    pub final_url: String,
    pub canonical_url: String,
    pub title: String,
    pub description: Option<String>,
    pub language: String,
    pub content: String,
    pub metadata: Value,
}

pub fn extract_document(
    source_url: &str,
    final_url: Option<&str>,
    content_type: Option<&str>,
    raw_body: &str,
) -> AppResult<ExtractedDocument> {
    let final_url = final_url.unwrap_or(source_url);
    let media_type = content_type
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .unwrap_or("text/plain")
        .to_ascii_lowercase();

    match media_type.as_str() {
        "text/html" => Ok(extract_html(source_url, final_url, raw_body)),
        "text/markdown" | "text/plain" | "application/json" => Ok(extract_text_like(
            source_url,
            final_url,
            raw_body,
            &media_type,
        )),
        _ => Err(AppError::Validation(format!(
            "unsupported content type for extraction: {media_type}"
        ))),
    }
}

fn extract_html(source_url: &str, final_url: &str, html: &str) -> ExtractedDocument {
    let title = first_non_empty(&[
        extract_between_ci(html, "<title", "</title>")
            .and_then(|value| value.split_once('>').map(|(_, text)| text.to_string())),
        extract_meta_content(html, "og:title"),
        extract_heading(html, "h1"),
    ])
    .unwrap_or_else(|| final_url.to_string());
    let description = extract_meta_content(html, "description")
        .or_else(|| extract_meta_content(html, "og:description"));
    let canonical_url = extract_canonical_url(html)
        .and_then(|value| normalize_url(final_url, &value))
        .or_else(|| normalize_url(final_url, final_url))
        .unwrap_or_else(|| final_url.to_string());
    let language = extract_html_language(html).unwrap_or_else(|| "en".to_string());
    let without_noise = remove_html_noise(html);
    let markdownish = html_to_markdownish(&without_noise);
    let content = normalize_whitespace(&decode_entities(&markdownish));

    ExtractedDocument {
        source_url: source_url.to_string(),
        final_url: final_url.to_string(),
        canonical_url: canonical_url.clone(),
        title: normalize_inline(&decode_entities(&title)),
        description: description.map(|value| normalize_inline(&decode_entities(&value))),
        language: language.clone(),
        content,
        metadata: json!({
            "extractor": "html_basic_v1",
            "contentType": "text/html",
            "language": language,
            "canonicalUrl": canonical_url
        }),
    }
}

fn extract_text_like(
    source_url: &str,
    final_url: &str,
    raw_body: &str,
    media_type: &str,
) -> ExtractedDocument {
    let content = normalize_whitespace(raw_body);
    let title = content
        .lines()
        .find_map(|line| {
            let trimmed = line.trim().trim_start_matches('#').trim();
            (!trimmed.is_empty()).then(|| trimmed.to_string())
        })
        .unwrap_or_else(|| final_url.to_string());

    ExtractedDocument {
        source_url: source_url.to_string(),
        final_url: final_url.to_string(),
        canonical_url: normalize_url(final_url, final_url).unwrap_or_else(|| final_url.to_string()),
        title,
        description: None,
        language: "en".to_string(),
        content,
        metadata: json!({
            "extractor": "text_basic_v1",
            "contentType": media_type
        }),
    }
}

fn remove_html_noise(html: &str) -> String {
    let mut text = html.to_string();
    for tag in [
        "script", "style", "noscript", "svg", "nav", "footer", "header", "aside",
    ] {
        text = remove_tag_block(&text, tag);
    }
    text
}

fn html_to_markdownish(html: &str) -> String {
    let mut text = html.to_string();
    for level in 1..=6 {
        let tag = format!("h{level}");
        text = replace_heading_tags(&text, &tag, level);
    }
    for tag in ["p", "li", "tr", "div", "section", "article", "br"] {
        text = replace_tag_ci(&text, tag, "\n");
    }
    text = replace_tag_ci(&text, "pre", "\n```\n");
    text = replace_tag_ci(&text, "code", "`");
    strip_tags(&text)
}

fn replace_heading_tags(html: &str, tag: &str, level: usize) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    let close = format!("</{tag}>");
    while let Some(start) = find_ci(rest, &format!("<{tag}")) {
        output.push_str(&rest[..start]);
        let after_start = &rest[start..];
        let Some(open_end) = after_start.find('>') else {
            output.push_str(after_start);
            return output;
        };
        let content_start = start + open_end + 1;
        let after_content = &rest[content_start..];
        let Some(end) = find_ci(after_content, &close) else {
            output.push_str(after_start);
            return output;
        };
        let heading = strip_tags(&after_content[..end]);
        output.push('\n');
        output.push_str(&"#".repeat(level));
        output.push(' ');
        output.push_str(heading.trim());
        output.push('\n');
        rest = &after_content[end + close.len()..];
    }
    output.push_str(rest);
    output
}

fn remove_tag_block(html: &str, tag: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut rest = html;
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    while let Some(start) = find_ci(rest, &open) {
        output.push_str(&rest[..start]);
        let after_start = &rest[start..];
        let Some(end) = find_ci(after_start, &close) else {
            return output;
        };
        rest = &after_start[end + close.len()..];
    }
    output.push_str(rest);
    output
}

fn replace_tag_ci(html: &str, tag: &str, replacement: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut idx = 0;
    while let Some(start) = find_ci(&html[idx..], &format!("<{tag}")) {
        let absolute = idx + start;
        output.push_str(&html[idx..absolute]);
        let Some(end) = html[absolute..].find('>') else {
            output.push_str(&html[absolute..]);
            return output;
        };
        output.push_str(replacement);
        idx = absolute + end + 1;
    }
    output.push_str(&html[idx..]);
    output.replace(&format!("</{tag}>"), replacement)
}

fn strip_tags(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    output
}

fn extract_between_ci(haystack: &str, start: &str, end: &str) -> Option<String> {
    let start_idx = find_ci(haystack, start)?;
    let after_start = &haystack[start_idx..];
    let end_idx = find_ci(after_start, end)?;
    Some(after_start[..end_idx].to_string())
}

fn extract_heading(html: &str, tag: &str) -> Option<String> {
    extract_between_ci(html, &format!("<{tag}"), &format!("</{tag}>"))
        .and_then(|value| value.split_once('>').map(|(_, text)| strip_tags(text)))
}

fn extract_meta_content(html: &str, name: &str) -> Option<String> {
    for attr in ["name", "property"] {
        let needle = format!("{attr}=\"{name}\"");
        if let Some(idx) = find_ci(html, &needle) {
            let tag_start = html[..idx].rfind('<')?;
            let tag_end = html[idx..].find('>')? + idx;
            return extract_attr(&html[tag_start..=tag_end], "content");
        }
    }
    None
}

fn extract_canonical_url(html: &str) -> Option<String> {
    let mut rest = html;
    while let Some(idx) = find_ci(rest, "<link") {
        let after = &rest[idx..];
        let Some(end) = after.find('>') else {
            return None;
        };
        let tag = &after[..=end];
        let tag_lower = tag.to_lowercase();
        if tag_lower.contains("rel=") && tag_lower.contains("canonical") {
            return extract_attr(tag, "href");
        }
        rest = &after[end + 1..];
    }
    None
}

fn extract_attr(tag: &str, attr: &str) -> Option<String> {
    for quote in ['"', '\''] {
        let needle = format!("{attr}={quote}");
        if let Some(start) = find_ci(tag, &needle).map(|idx| idx + needle.len()) {
            let end = tag[start..].find(quote)? + start;
            return Some(tag[start..end].to_string());
        }
    }
    None
}

fn extract_html_language(html: &str) -> Option<String> {
    let idx = find_ci(html, "<html")?;
    let tag_end = html[idx..].find('>')? + idx;
    extract_attr(&html[idx..=tag_end], "lang")
        .map(|value| {
            value
                .split('-')
                .next()
                .unwrap_or(&value)
                .trim()
                .to_lowercase()
        })
        .filter(|value| !value.is_empty())
}

fn normalize_url(base_url: &str, candidate: &str) -> Option<String> {
    let base = Url::parse(base_url).ok()?;
    let mut url = base.join(candidate.trim()).ok()?;
    url.set_fragment(None);
    Some(url.to_string())
}

fn first_non_empty(values: &[Option<String>]) -> Option<String> {
    values
        .iter()
        .flatten()
        .map(|value| value.trim())
        .find(|value| !value.is_empty())
        .map(ToString::to_string)
}

fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    haystack.to_lowercase().find(&needle.to_lowercase())
}

fn decode_entities(text: &str) -> String {
    text.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn normalize_inline(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn normalize_whitespace(text: &str) -> String {
    let mut lines = Vec::new();
    let mut previous_blank = false;
    for line in text.lines() {
        let trimmed = line.trim_end();
        let blank = trimmed.trim().is_empty();
        if blank && previous_blank {
            continue;
        }
        lines.push(trimmed.to_string());
        previous_blank = blank;
    }
    lines.join("\n").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn html_extractor_removes_script_and_keeps_heading() {
        let doc = extract_document(
            "https://example.com/doc",
            None,
            Some("text/html"),
            "<html><head><title>Doc</title><script>x()</script></head><body><h1>Intro</h1><p>Hello</p></body></html>",
        )
        .unwrap();

        assert_eq!(doc.title, "Doc");
        assert!(doc.content.contains("# Intro"));
        assert!(!doc.content.contains("x()"));
    }

    #[test]
    fn html_extractor_resolves_relative_canonical_and_language() {
        let doc = extract_document(
            "https://example.com/docs/page?utm=1",
            Some("https://example.com/docs/page?utm=1"),
            Some("text/html"),
            "<html lang='vi-VN'><head><link rel='canonical' href='/docs/page#intro'><title>Doc</title></head><body><aside>Menu</aside><h1>Intro</h1><pre><code>let x = 1;</code></pre></body></html>",
        )
        .unwrap();

        assert_eq!(doc.canonical_url, "https://example.com/docs/page");
        assert_eq!(doc.language, "vi");
        assert!(doc.content.contains("let x = 1;"));
        assert!(!doc.content.contains("Menu"));
    }
}
