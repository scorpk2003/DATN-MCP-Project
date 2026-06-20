use serde::Serialize;
use serde_json::{Value, json};

const DEFAULT_MAX_TOKENS: i32 = 700;
const DEFAULT_MIN_TOKENS: i32 = 120;

#[derive(Debug, Clone, Serialize)]
pub struct Chunk {
    pub chunk_index: i32,
    pub heading_path: Vec<String>,
    pub heading_level: Option<i32>,
    pub content: String,
    pub content_tokens: Option<i32>,
    pub start_char: Option<i32>,
    pub end_char: Option<i32>,
    pub code_language: Option<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone)]
struct Section {
    heading_path: Vec<String>,
    heading_level: Option<i32>,
    start_char: usize,
    content: String,
}

pub fn chunk_document(content: &str) -> Vec<Chunk> {
    let normalized = normalize_content(content);
    if normalized.is_empty() {
        return Vec::new();
    }

    let sections = split_sections(&normalized);
    let mut chunks = Vec::new();
    for section in sections {
        for piece in split_section_preserving_code(&section, DEFAULT_MAX_TOKENS) {
            if estimate_tokens(&piece.content) < DEFAULT_MIN_TOKENS {
                maybe_merge_or_push(&mut chunks, piece);
            } else {
                push_chunk(&mut chunks, piece);
            }
        }
    }
    reindex(chunks)
}

fn split_sections(content: &str) -> Vec<Section> {
    let mut sections = Vec::new();
    let mut current_heading_path = Vec::<String>::new();
    let mut current_heading_level = None;
    let mut current_start = 0usize;
    let mut current = String::new();
    let mut offset = 0usize;

    for line in content.lines() {
        if let Some((level, title)) = parse_markdown_heading(line) {
            if !current.trim().is_empty() {
                sections.push(Section {
                    heading_path: current_heading_path.clone(),
                    heading_level: current_heading_level,
                    start_char: current_start,
                    content: current.trim().to_string(),
                });
                current.clear();
            }
            update_heading_path(&mut current_heading_path, level, title);
            current_heading_level = Some(level as i32);
            current_start = offset;
        }

        current.push_str(line);
        current.push('\n');
        offset += line.len() + 1;
    }

    if !current.trim().is_empty() {
        sections.push(Section {
            heading_path: current_heading_path,
            heading_level: current_heading_level,
            start_char: current_start,
            content: current.trim().to_string(),
        });
    }
    sections
}

fn split_section_preserving_code(section: &Section, max_tokens: i32) -> Vec<Section> {
    if estimate_tokens(&section.content) <= max_tokens {
        return vec![section.clone()];
    }

    let blocks = split_blocks(&section.content);
    let mut pieces = Vec::new();
    let mut current = String::new();
    let mut current_start = section.start_char;
    let mut relative_offset = 0usize;

    for block in blocks {
        let candidate = join_blocks(&current, &block);
        if !current.is_empty() && estimate_tokens(&candidate) > max_tokens {
            pieces.push(Section {
                heading_path: section.heading_path.clone(),
                heading_level: section.heading_level,
                start_char: current_start,
                content: current.trim().to_string(),
            });
            current_start = section.start_char + relative_offset;
            current.clear();
        }
        current = join_blocks(&current, &block);
        relative_offset += block.len();
    }

    if !current.trim().is_empty() {
        pieces.push(Section {
            heading_path: section.heading_path.clone(),
            heading_level: section.heading_level,
            start_char: current_start,
            content: current.trim().to_string(),
        });
    }
    pieces
}

fn split_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut current = String::new();
    let mut in_code = false;

    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
        }
        if !in_code && line.trim().is_empty() && !current.trim().is_empty() {
            blocks.push(current.clone());
            current.clear();
            continue;
        }
        current.push_str(line);
        current.push('\n');
    }

    if !current.trim().is_empty() {
        blocks.push(current);
    }
    blocks
}

fn maybe_merge_or_push(chunks: &mut Vec<Chunk>, section: Section) {
    if let Some(last) = chunks.last_mut() {
        let same_path = last.heading_path == section.heading_path;
        let merged_tokens = last.content_tokens.unwrap_or(0) + estimate_tokens(&section.content);
        if same_path && merged_tokens <= DEFAULT_MAX_TOKENS {
            last.content.push_str("\n\n");
            last.content.push_str(&section.content);
            last.content_tokens = Some(merged_tokens);
            last.end_char = Some(section.start_char as i32 + section.content.len() as i32);
            last.metadata = chunk_metadata(&last.content);
            return;
        }
    }
    push_chunk(chunks, section);
}

fn push_chunk(chunks: &mut Vec<Chunk>, section: Section) {
    let content = section.content.trim().to_string();
    chunks.push(Chunk {
        chunk_index: chunks.len() as i32,
        heading_path: section.heading_path,
        heading_level: section.heading_level,
        content_tokens: Some(estimate_tokens(&content)),
        start_char: Some(section.start_char as i32),
        end_char: Some(section.start_char as i32 + content.len() as i32),
        code_language: dominant_code_language(&content),
        metadata: chunk_metadata(&content),
        content,
    });
}

fn reindex(mut chunks: Vec<Chunk>) -> Vec<Chunk> {
    for (idx, chunk) in chunks.iter_mut().enumerate() {
        chunk.chunk_index = idx as i32;
    }
    chunks
}

fn chunk_metadata(content: &str) -> Value {
    json!({
        "chunker": "section_aware_v1",
        "content_kind": classify_content_kind(content),
    })
}

fn classify_content_kind(content: &str) -> &'static str {
    let lower = content.to_ascii_lowercase();
    let code_fence_count = lower.matches("```").count();
    if lower.contains("exercise") || lower.contains("practice") {
        "exercise"
    } else if lower.contains("example") || code_fence_count >= 2 {
        "example"
    } else if lower.contains("api") || lower.contains("reference") {
        "reference"
    } else if code_fence_count > 0 {
        "code"
    } else if lower.contains("concept") || lower.contains("introduction") {
        "concept"
    } else {
        "mixed"
    }
}

fn dominant_code_language(content: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let trimmed = line.trim();
        trimmed
            .strip_prefix("```")
            .filter(|lang| !lang.trim().is_empty())
            .map(|lang| lang.trim().to_string())
    })
}

fn normalize_content(content: &str) -> String {
    content
        .lines()
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

fn parse_markdown_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start();
    let level = trimmed.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let title = trimmed[level..].trim();
    (!title.is_empty()).then(|| (level, title.to_string()))
}

fn update_heading_path(path: &mut Vec<String>, level: usize, title: String) {
    let idx = level.saturating_sub(1);
    if path.len() > idx {
        path.truncate(idx);
    }
    path.push(title);
}

fn join_blocks(current: &str, block: &str) -> String {
    if current.is_empty() {
        block.to_string()
    } else {
        format!("{}\n{}", current.trim_end(), block.trim_start())
    }
}

fn estimate_tokens(text: &str) -> i32 {
    text.split_whitespace().count() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_content_returns_no_chunks() {
        assert!(chunk_document(" \n\t ").is_empty());
    }

    #[test]
    fn long_content_is_split_with_stable_indexes() {
        let content = format!(
            "# Ownership\n\n{}",
            "Rust ownership prevents data races. ".repeat(180)
        );
        let chunks = chunk_document(&content);

        assert!(chunks.len() > 1);
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert!(chunks.iter().all(|chunk| !chunk.content.trim().is_empty()));
    }

    #[test]
    fn markdown_heading_is_kept_as_heading_path() {
        let chunks = chunk_document("# Transactions\nPostgreSQL rollback protects data.");

        assert_eq!(chunks[0].heading_path, vec!["Transactions"]);
    }

    #[test]
    fn code_fence_sets_content_kind() {
        let chunks = chunk_document("# Example\n```rust\nfn main() {}\n```");

        assert_eq!(chunks[0].metadata["content_kind"], "example");
        assert_eq!(chunks[0].code_language, Some("rust".to_string()));
    }
}
