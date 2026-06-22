use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct FailureTriage {
    #[serde(rename = "topicId")]
    pub topic_id: String,
    pub status: String,
    #[serde(rename = "primaryFailure")]
    pub primary_failure: String,
    #[serde(rename = "secondaryFailures")]
    pub secondary_failures: Vec<String>,
    #[serde(rename = "suggestedFix")]
    pub suggested_fix: String,
}

pub fn classify_triage(topic_id: &str, reasons: &[String]) -> FailureTriage {
    let primary = if reasons.iter().any(|value| value == "missing_seed") {
        "missing_seed"
    } else if reasons
        .iter()
        .any(|value| value == "seed_invalid_or_disabled")
    {
        "seed_invalid_or_disabled"
    } else if reasons
        .iter()
        .any(|value| value == "missing_official_resource")
    {
        "crawl_failed"
    } else if reasons.iter().any(|value| value == "missing_versions") {
        "resource_created_but_no_version"
    } else if reasons.iter().any(|value| value == "missing_chunks") {
        "version_created_but_no_chunks"
    } else if reasons.iter().any(|value| value == "missing_embeddings") {
        "chunks_created_but_no_embeddings"
    } else if reasons.iter().any(|value| value == "enrichment_missing") {
        "enrichment_missing"
    } else if reasons.iter().any(|value| value == "topic_alias_missing") {
        "topic_alias_missing"
    } else {
        "expected_manifest_too_strict_or_wrong"
    };
    FailureTriage {
        topic_id: topic_id.to_string(),
        status: "not_ready".to_string(),
        primary_failure: primary.to_string(),
        secondary_failures: reasons.to_vec(),
        suggested_fix: suggested_fix(primary).to_string(),
    }
}

fn suggested_fix(primary: &str) -> &'static str {
    match primary {
        "missing_seed" => "Add targeted official seed and rerun crawl/extract/embed.",
        "seed_invalid_or_disabled" => {
            "Fix the seed URL or source allowedPaths, then rerun seeding."
        }
        "crawl_failed" => "Inspect crawl job/fetch errors and retry the affected seed.",
        "resource_created_but_no_version" => "Rerun extraction for the affected fetch artifact.",
        "version_created_but_no_chunks" => "Fix or rerun chunk creation for the latest version.",
        "chunks_created_but_no_embeddings" => {
            "Run the embedding worker until pending chunks are embedded."
        }
        "enrichment_missing" => "Run enrichment for the affected official resources.",
        "topic_alias_missing" => {
            "Attach topic aliases/concepts through seed metadata or enrichment."
        }
        _ => "Review the evaluation manifest expectations against the seeded official corpus.",
    }
}
