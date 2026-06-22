pub mod readiness;
pub mod seed_manifest;
pub mod source_catalog;
pub mod triage;

use std::{collections::HashSet, fs, path::Path};

use serde::{Deserialize, Serialize};
use url::Url;

use crate::{AppError, AppResult};

pub use readiness::{CorpusReadinessReport, TopicReadiness, build_readiness_report};
pub use seed_manifest::{OfficialSeed, OfficialSeedManifest};
pub use source_catalog::{OfficialSource, OfficialSourceCatalog};
pub use triage::{FailureTriage, classify_triage};

const KNOWN_RESOURCE_TYPES: &[&str] = &[
    "official_reference",
    "primary_learning",
    "practice",
    "project",
    "deep_dive",
    "troubleshooting",
];

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvaluationTopicManifest {
    pub topics: Vec<EvaluationTopic>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EvaluationTopic {
    pub topic_id: String,
    pub topic_name: String,
    pub aliases: Vec<String>,
    pub level: String,
    pub required_resource_types: Vec<String>,
    pub expected_official_domains: Vec<String>,
    pub expected_min_resources: i64,
    pub expected_min_chunks: i64,
    pub expected_coverage: String,
    pub expected_official_in_top_k: i64,
    pub allow_partial_if_missing_types: bool,
    pub notes: String,
}

pub fn load_source_catalog(path: impl AsRef<Path>) -> AppResult<OfficialSourceCatalog> {
    load_yaml(path)
}

pub fn load_seed_manifest(path: impl AsRef<Path>) -> AppResult<OfficialSeedManifest> {
    load_yaml(path)
}

pub fn load_evaluation_manifest(path: impl AsRef<Path>) -> AppResult<EvaluationTopicManifest> {
    load_yaml(path)
}

pub fn load_validated_manifests(
    source_catalog_path: impl AsRef<Path>,
    seed_manifest_path: impl AsRef<Path>,
    topic_manifest_path: impl AsRef<Path>,
) -> AppResult<(
    OfficialSourceCatalog,
    OfficialSeedManifest,
    EvaluationTopicManifest,
)> {
    let source_catalog = load_source_catalog(source_catalog_path)?;
    let seed_manifest = load_seed_manifest(seed_manifest_path)?;
    let topic_manifest = load_evaluation_manifest(topic_manifest_path)?;
    validate_manifests(&source_catalog, &seed_manifest, &topic_manifest)?;
    Ok((source_catalog, seed_manifest, topic_manifest))
}

pub fn validate_manifests(
    source_catalog: &OfficialSourceCatalog,
    seed_manifest: &OfficialSeedManifest,
    topic_manifest: &EvaluationTopicManifest,
) -> AppResult<()> {
    if source_catalog.sources.is_empty() {
        return validation_error("official source catalog must contain at least one source");
    }
    if topic_manifest.topics.len() != 30 {
        return validation_error(format!(
            "evaluation topic manifest must contain exactly 30 topics, got {}",
            topic_manifest.topics.len()
        ));
    }

    let mut source_ids = HashSet::new();
    for source in &source_catalog.sources {
        require(&source.id, "source.id")?;
        require(&source.name, "source.name")?;
        require(&source.kind, "source.kind")?;
        require(&source.base_url, "source.base_url")?;
        if source.kind != "official_docs" {
            return validation_error(format!("source {} must be official_docs", source.id));
        }
        if !source.is_official {
            return validation_error(format!("source {} must be official", source.id));
        }
        if source.trust_tier != 1 {
            return validation_error(format!("source {} must have trust_tier 1", source.id));
        }
        if !KNOWN_RESOURCE_TYPES.contains(&source.default_resource_kind.as_str()) {
            return validation_error(format!(
                "source {} has unknown default_resource_kind {}",
                source.id, source.default_resource_kind
            ));
        }
        Url::parse(&source.base_url).map_err(|_| {
            AppError::Validation(format!("source {} has invalid base_url", source.id))
        })?;
        if !source_ids.insert(source.id.as_str()) {
            return validation_error(format!("duplicate source id {}", source.id));
        }
    }

    let mut topic_ids = HashSet::new();
    for topic in &topic_manifest.topics {
        require(&topic.topic_id, "topic.topic_id")?;
        require(&topic.topic_name, "topic.topic_name")?;
        if topic.expected_official_domains.is_empty() {
            return validation_error(format!(
                "topic {} missing expected_official_domains",
                topic.topic_id
            ));
        }
        if topic.aliases.is_empty() {
            return validation_error(format!("topic {} missing aliases", topic.topic_id));
        }
        if topic.expected_min_resources < 1 || topic.expected_min_chunks < 1 {
            return validation_error(format!(
                "topic {} must require resources and chunks",
                topic.topic_id
            ));
        }
        if !["beginner", "intermediate", "advanced"].contains(&topic.level.as_str()) {
            return validation_error(format!("topic {} has invalid level", topic.topic_id));
        }
        if !["good", "partial", "poor"].contains(&topic.expected_coverage.as_str()) {
            return validation_error(format!(
                "topic {} has invalid expected_coverage",
                topic.topic_id
            ));
        }
        for required in &topic.required_resource_types {
            if !KNOWN_RESOURCE_TYPES.contains(&required.as_str()) {
                return validation_error(format!(
                    "topic {} has unknown required_resource_type {}",
                    topic.topic_id, required
                ));
            }
        }
        if !topic_ids.insert(topic.topic_id.as_str()) {
            return validation_error(format!("duplicate topic id {}", topic.topic_id));
        }
    }

    let source_map = source_catalog
        .sources
        .iter()
        .map(|source| (source.id.as_str(), source))
        .collect::<std::collections::HashMap<_, _>>();
    let mut seed_ids = HashSet::new();
    for seed in &seed_manifest.seeds {
        require(&seed.seed_id, "seed.seed_id")?;
        if !seed_ids.insert(seed.seed_id.as_str()) {
            return validation_error(format!("duplicate seed id {}", seed.seed_id));
        }
        let Some(source) = source_map.get(seed.source_id.as_str()) else {
            return validation_error(format!(
                "seed {} references invalid source_id {}",
                seed.seed_id, seed.source_id
            ));
        };
        if seed.topic_ids.is_empty() {
            return validation_error(format!("seed {} missing topic_ids", seed.seed_id));
        }
        for topic_id in &seed.topic_ids {
            if !topic_ids.contains(topic_id.as_str()) {
                return validation_error(format!(
                    "seed {} references invalid topic_id {}",
                    seed.seed_id, topic_id
                ));
            }
        }
        if !KNOWN_RESOURCE_TYPES.contains(&seed.expected_resource_kind.as_str()) {
            return validation_error(format!(
                "seed {} has unknown expected_resource_kind {}",
                seed.seed_id, seed.expected_resource_kind
            ));
        }
        validate_seed_url(source, &seed.url).map_err(|err| {
            AppError::Validation(format!("seed {} URL policy error: {err}", seed.seed_id))
        })?;
    }

    Ok(())
}

pub fn validate_seed_url(source: &OfficialSource, raw_url: &str) -> AppResult<()> {
    let source_url = Url::parse(&source.base_url).map_err(|_| {
        AppError::Validation(format!("invalid source base_url {}", source.base_url))
    })?;
    let url = Url::parse(raw_url)
        .map_err(|_| AppError::Validation(format!("invalid seed url {raw_url}")))?;
    if url.scheme() != source_url.scheme() || url.host_str() != source_url.host_str() {
        return validation_error(format!("{raw_url} is outside {}", source.base_url));
    }
    let path = url.path();
    if !source.allowed_paths.is_empty()
        && !source
            .allowed_paths
            .iter()
            .any(|allowed| allowed == "/" || path.starts_with(allowed))
    {
        return validation_error(format!("{raw_url} is outside allowed_paths"));
    }
    if source
        .blocked_paths
        .iter()
        .any(|blocked| path.starts_with(blocked))
    {
        return validation_error(format!("{raw_url} is blocked by source policy"));
    }
    Ok(())
}

pub fn infer_resource_role(url: &str, is_official: bool) -> String {
    let lower = url.to_ascii_lowercase();
    if contains_any(
        &lower,
        &[
            "/examples",
            "/tutorials",
            "/tasks",
            "/kubernetes-basics",
            "/gettingstarted",
        ],
    ) {
        return "practice".to_string();
    }
    if contains_any(
        &lower,
        &[
            "/learn",
            "/tutorial",
            "/book",
            "/getting-started",
            "/get-started",
            "/guide",
            "/guides",
        ],
    ) {
        return "primary_learning".to_string();
    }
    if contains_any(
        &lower,
        &[
            "/reference",
            "/api",
            "/docs/current/sql",
            "/library",
            "/std",
            "/reference",
        ],
    ) {
        return "official_reference".to_string();
    }
    if is_official {
        "official_reference".to_string()
    } else {
        "primary_learning".to_string()
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn load_yaml<T: for<'de> Deserialize<'de>>(path: impl AsRef<Path>) -> AppResult<T> {
    let path = path.as_ref();
    let raw = fs::read_to_string(path)
        .map_err(|err| AppError::Validation(format!("failed to read {}: {err}", path.display())))?;
    serde_yaml::from_str(&raw)
        .map_err(|err| AppError::Validation(format!("failed to parse {}: {err}", path.display())))
}

fn require(value: &str, field: &str) -> AppResult<()> {
    if value.trim().is_empty() {
        validation_error(format!("{field} is required"))
    } else {
        Ok(())
    }
}

fn validation_error<T>(message: impl Into<String>) -> AppResult<T> {
    Err(AppError::Validation(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source_catalog() -> OfficialSourceCatalog {
        serde_yaml::from_str(
            r#"
sources:
  - id: react_docs
    name: React Docs
    kind: official_docs
    base_url: https://react.dev
    trust_tier: 1
    language_hint: en
    is_official: true
    enabled: true
    allowed_paths: ["/learn", "/reference"]
    blocked_paths: ["/blog"]
    default_resource_kind: primary_learning
    crawl_policy:
      max_depth_default: 1
      rate_limit_per_minute: 30
      respect_robots_txt: true
      user_agent: test
"#,
        )
        .unwrap()
    }

    fn topics() -> EvaluationTopicManifest {
        EvaluationTopicManifest {
            topics: (0..30)
                .map(|idx| EvaluationTopic {
                    topic_id: format!("topic_{idx}"),
                    topic_name: format!("Topic {idx}"),
                    aliases: vec![format!("alias {idx}")],
                    level: "beginner".to_string(),
                    required_resource_types: vec!["primary_learning".to_string()],
                    expected_official_domains: vec!["react.dev".to_string()],
                    expected_min_resources: 1,
                    expected_min_chunks: 1,
                    expected_coverage: "good".to_string(),
                    expected_official_in_top_k: 3,
                    allow_partial_if_missing_types: true,
                    notes: "test".to_string(),
                })
                .collect(),
        }
    }

    fn seed(source_id: &str, topic_id: &str, url: &str) -> OfficialSeedManifest {
        OfficialSeedManifest {
            seeds: vec![OfficialSeed {
                seed_id: "seed".to_string(),
                topic_ids: vec![topic_id.to_string()],
                source_id: source_id.to_string(),
                url: url.to_string(),
                seed_type: "url".to_string(),
                max_depth: 0,
                priority: 10,
                enabled: true,
                expected_resource_kind: "primary_learning".to_string(),
                notes: "test".to_string(),
            }],
        }
    }

    #[test]
    fn manifest_validation_rejects_invalid_source_id() {
        let err = validate_manifests(
            &source_catalog(),
            &seed("missing", "topic_0", "https://react.dev/learn/a"),
            &topics(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid source_id"));
    }

    #[test]
    fn manifest_validation_rejects_invalid_topic_id() {
        let err = validate_manifests(
            &source_catalog(),
            &seed("react_docs", "missing", "https://react.dev/learn/a"),
            &topics(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("invalid topic_id"));
    }

    #[test]
    fn manifest_validation_rejects_seed_url_outside_base_url() {
        let err = validate_manifests(
            &source_catalog(),
            &seed("react_docs", "topic_0", "https://example.com/learn/a"),
            &topics(),
        )
        .unwrap_err();
        assert!(err.to_string().contains("outside"));
    }

    #[test]
    fn manifest_validation_rejects_missing_expected_domains() {
        let mut topics = topics();
        topics.topics[0].expected_official_domains.clear();
        let err = validate_manifests(
            &source_catalog(),
            &seed("react_docs", "topic_0", "https://react.dev/learn/a"),
            &topics,
        )
        .unwrap_err();
        assert!(err.to_string().contains("expected_official_domains"));
    }

    #[test]
    fn manifest_validation_accepts_official_v0_2_1_files() {
        let catalog = load_source_catalog("config/official_source_catalog.yaml").unwrap();
        let seeds = load_seed_manifest("config/official_topic_seed_manifest.yaml").unwrap();
        let topics = load_evaluation_manifest("config/evaluation_topic_manifest.yaml").unwrap();

        validate_manifests(&catalog, &seeds, &topics).unwrap();
    }
}
