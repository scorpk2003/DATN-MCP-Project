use std::collections::HashSet;

use crate::domain::{
    EvidenceCoverage, LessonLevel, LessonRequirement, LessonResource, PackedLessonEvidence,
    PackedSelectedChunk, ResourceCandidateInput, SourceType,
};

const DEFAULT_MIN_QUALITY: f32 = 0.65;
const MAX_SELECTED_CHUNKS: usize = 8;

pub fn pack_resources(
    requirement: &LessonRequirement,
    resources: &[ResourceCandidateInput],
) -> PackedLessonEvidence {
    let mut seen = HashSet::new();
    let mut usable = resources
        .iter()
        .filter(|resource| resource.quality_score.unwrap_or(0.0) >= DEFAULT_MIN_QUALITY)
        .filter(|resource| {
            let key = dedupe_key(resource);
            seen.insert(key)
        })
        .cloned()
        .collect::<Vec<_>>();

    usable.sort_by(|left, right| {
        score_resource(right)
            .partial_cmp(&score_resource(left))
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let selected_chunks = select_chunks(&usable);
    let coverage = calculate_coverage(requirement, &usable, &selected_chunks);

    PackedLessonEvidence {
        primary_sources: usable
            .iter()
            .filter(|resource| {
                matches!(resource.source_type, SourceType::Docs | SourceType::Course)
            })
            .take(3)
            .map(|resource| {
                to_lesson_resource(resource, &requirement.level, "Primary explanation source")
            })
            .collect(),
        supporting_sources: usable
            .iter()
            .filter(|resource| {
                matches!(
                    resource.source_type,
                    SourceType::Article | SourceType::Book | SourceType::Paper | SourceType::Video
                )
            })
            .take(4)
            .map(|resource| {
                to_lesson_resource(
                    resource,
                    &requirement.level,
                    "Supporting source for deeper explanation",
                )
            })
            .collect(),
        code_sources: usable
            .iter()
            .filter(|resource| matches!(resource.source_type, SourceType::Github))
            .take(3)
            .map(|resource| to_lesson_resource(resource, &requirement.level, "Code/example source"))
            .collect(),
        selected_chunks,
        coverage,
    }
}

pub fn has_sufficient_evidence(evidence: &PackedLessonEvidence) -> bool {
    let source_count = evidence.primary_sources.len()
        + evidence.supporting_sources.len()
        + evidence.code_sources.len();

    source_count > 0
        && !evidence.selected_chunks.is_empty()
        && evidence.coverage.coverage_score >= 0.5
}

fn select_chunks(resources: &[ResourceCandidateInput]) -> Vec<PackedSelectedChunk> {
    let mut chunks = resources
        .iter()
        .flat_map(|resource| {
            resource
                .chunks
                .clone()
                .unwrap_or_default()
                .into_iter()
                .map(|chunk| PackedSelectedChunk {
                    resource_id: resource.id.clone(),
                    chunk_id: chunk.chunk_id,
                    text: chunk.text,
                    relevance_score: chunk
                        .score
                        .unwrap_or_else(|| resource.relevance_score.unwrap_or(0.0)),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    chunks.sort_by(|left, right| {
        right
            .relevance_score
            .partial_cmp(&left.relevance_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    chunks.truncate(MAX_SELECTED_CHUNKS);
    chunks
}

fn calculate_coverage(
    requirement: &LessonRequirement,
    resources: &[ResourceCandidateInput],
    selected_chunks: &[PackedSelectedChunk],
) -> EvidenceCoverage {
    let evidence_text = resources
        .iter()
        .map(|resource| {
            format!(
                "{} {} {}",
                resource.title,
                resource.summary.clone().unwrap_or_default(),
                selected_chunks
                    .iter()
                    .filter(|chunk| chunk.resource_id == resource.id)
                    .map(|chunk| chunk.text.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase();

    let mut objectives_covered = Vec::new();
    let mut objectives_missing = Vec::new();

    for objective in &requirement.objectives {
        if objective_is_covered(objective, &evidence_text) {
            objectives_covered.push(objective.clone());
        } else {
            objectives_missing.push(objective.clone());
        }
    }

    let total = requirement.objectives.len().max(1) as f32;
    let coverage_score = objectives_covered.len() as f32 / total;

    EvidenceCoverage {
        objectives_covered,
        objectives_missing,
        coverage_score,
    }
}

fn objective_is_covered(objective: &str, evidence_text: &str) -> bool {
    objective
        .split(|character: char| !character.is_alphanumeric())
        .map(str::trim)
        .filter(|word| word.len() >= 4)
        .take(6)
        .any(|word| evidence_text.contains(&word.to_lowercase()))
}

fn to_lesson_resource(
    resource: &ResourceCandidateInput,
    level: &LessonLevel,
    reason_selected: &str,
) -> LessonResource {
    LessonResource {
        id: resource.id.clone(),
        title: resource.title.clone(),
        url: resource.url.clone(),
        source_type: resource.source_type.clone(),
        difficulty: level.clone(),
        relevance_score: resource.relevance_score.unwrap_or(0.0),
        quality_score: resource.quality_score,
        reason_selected: reason_selected.to_string(),
    }
}

fn score_resource(resource: &ResourceCandidateInput) -> f32 {
    0.6 * resource.quality_score.unwrap_or(0.0) + 0.4 * resource.relevance_score.unwrap_or(0.0)
}

fn dedupe_key(resource: &ResourceCandidateInput) -> String {
    resource
        .url
        .clone()
        .unwrap_or_else(|| resource.title.clone())
        .trim()
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ExerciseType, ResourceChunkInput};

    #[test]
    fn filters_dedupes_and_selects_chunks() {
        let requirement = LessonRequirement {
            topic: "SQL joins".to_string(),
            level: LessonLevel::Beginner,
            objectives: vec!["Write inner join queries".to_string()],
            prerequisite_gaps: vec![],
            resource_queries: vec![],
            recommended_exercise_types: vec![ExerciseType::Coding],
            estimated_minutes: 35,
        };
        let resources = vec![
            ResourceCandidateInput {
                id: "a".to_string(),
                title: "SQL joins docs".to_string(),
                url: Some("https://example.com/sql-joins".to_string()),
                source_type: SourceType::Docs,
                summary: Some("Inner join queries".to_string()),
                chunks: Some(vec![ResourceChunkInput {
                    chunk_id: "c1".to_string(),
                    text: "Use INNER JOIN to write queries across tables.".to_string(),
                    score: Some(0.9),
                }]),
                quality_score: Some(0.9),
                relevance_score: Some(0.8),
            },
            ResourceCandidateInput {
                id: "dup".to_string(),
                title: "Duplicate".to_string(),
                url: Some("https://example.com/sql-joins".to_string()),
                source_type: SourceType::Article,
                summary: None,
                chunks: None,
                quality_score: Some(0.9),
                relevance_score: Some(0.8),
            },
            ResourceCandidateInput {
                id: "bad".to_string(),
                title: "Low quality".to_string(),
                url: None,
                source_type: SourceType::Article,
                summary: None,
                chunks: None,
                quality_score: Some(0.2),
                relevance_score: Some(0.9),
            },
        ];

        let packed = pack_resources(&requirement, &resources);
        assert_eq!(packed.primary_sources.len(), 1);
        assert_eq!(packed.selected_chunks.len(), 1);
        assert!(has_sufficient_evidence(&packed));
    }
}
