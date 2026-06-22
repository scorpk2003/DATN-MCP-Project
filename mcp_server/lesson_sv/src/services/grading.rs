use crate::domain::{GradingMistake, GradingResult, MistakeType, NextRecommendation, RubricItem};

pub fn grade_answer(answer: &str, rubric: &[RubricItem]) -> GradingResult {
    let normalized_answer = answer.trim().to_lowercase();
    let rubric = if rubric.is_empty() {
        fallback_rubric()
    } else {
        rubric.to_vec()
    };

    let max_score = rubric
        .iter()
        .map(|item| item.max_score)
        .sum::<f32>()
        .max(1.0);
    let earned = rubric
        .iter()
        .map(|item| score_criterion(&normalized_answer, item))
        .sum::<f32>();
    let score = (earned / max_score).clamp(0.0, 1.0);
    let passed = score >= 0.8;
    let mistakes = detect_mistakes(score, &normalized_answer);
    let improvement_suggestions = improvement_suggestions(score, &mistakes);

    GradingResult {
        status: "graded".to_string(),
        score,
        passed,
        feedback: feedback(score),
        mistakes,
        improvement_suggestions,
        next_recommendation: next_recommendation(score),
    }
}

fn score_criterion(answer: &str, item: &RubricItem) -> f32 {
    let criterion = item.criterion.to_lowercase();
    let description = item.description.to_lowercase();
    let mut matched: f32 = 0.0;

    if answer_contains_keywords(answer, &criterion) {
        matched += 0.5;
    }
    if answer_contains_keywords(answer, &description) {
        matched += 0.3;
    }
    if answer.split_whitespace().count() >= 20 {
        matched += 0.2;
    }

    item.max_score * matched.min(1.0)
}

fn answer_contains_keywords(answer: &str, text: &str) -> bool {
    text.split(|character: char| !character.is_alphanumeric())
        .filter(|word| word.len() >= 5)
        .take(8)
        .any(|word| answer.contains(word))
}

fn detect_mistakes(score: f32, answer: &str) -> Vec<GradingMistake> {
    let mut mistakes = Vec::new();

    if score < 0.5 {
        mistakes.push(GradingMistake {
            mistake_type: MistakeType::Conceptual,
            message: "The answer does not show enough concept coverage for the rubric.".to_string(),
            severity: "high".to_string(),
        });
    }
    if answer.split_whitespace().count() < 20 {
        mistakes.push(GradingMistake {
            mistake_type: MistakeType::MissingDetail,
            message: "The answer is too short to justify mastery.".to_string(),
            severity: "medium".to_string(),
        });
    }
    if !answer.contains("because") && !answer.contains("therefore") && !answer.contains("so") {
        mistakes.push(GradingMistake {
            mistake_type: MistakeType::Reasoning,
            message: "The answer needs clearer reasoning, not just a final statement.".to_string(),
            severity: "medium".to_string(),
        });
    }

    mistakes
}

fn improvement_suggestions(score: f32, mistakes: &[GradingMistake]) -> Vec<String> {
    if score >= 0.8 {
        return vec!["Continue to the next activity.".to_string()];
    }

    let mut suggestions = vec!["Restate the core concept in your own words.".to_string()];
    if mistakes
        .iter()
        .any(|mistake| matches!(mistake.mistake_type, MistakeType::MissingDetail))
    {
        suggestions.push("Add a concrete example and explain why it works.".to_string());
    }
    if score < 0.5 {
        suggestions.push("Review the relevant lesson block before retrying.".to_string());
    }

    suggestions
}

fn feedback(score: f32) -> String {
    if score >= 0.8 {
        "Answer meets the rubric and can continue.".to_string()
    } else if score >= 0.5 {
        "Answer is partially correct but needs more detail and evidence.".to_string()
    } else {
        "Answer does not yet demonstrate mastery of the concept.".to_string()
    }
}

fn next_recommendation(score: f32) -> NextRecommendation {
    if score >= 0.8 {
        NextRecommendation {
            action: "continue".to_string(),
            target_id: None,
            reason: "Score meets passing threshold.".to_string(),
        }
    } else if score >= 0.5 {
        NextRecommendation {
            action: "retry_exercise".to_string(),
            target_id: None,
            reason: "Answer is close but needs revision.".to_string(),
        }
    } else {
        NextRecommendation {
            action: "review_block".to_string(),
            target_id: None,
            reason: "Score is below remediation threshold.".to_string(),
        }
    }
}

fn fallback_rubric() -> Vec<RubricItem> {
    vec![RubricItem {
        criterion: "concept_accuracy".to_string(),
        max_score: 1.0,
        description: "Uses the core concept correctly with enough explanation.".to_string(),
    }]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grades_rich_answer_higher_than_short_answer() {
        let rubric = vec![RubricItem {
            criterion: "concept accuracy".to_string(),
            max_score: 1.0,
            description: "Uses concept evidence and practical application".to_string(),
        }];

        let rich = grade_answer(
            "This answer explains concept accuracy with evidence and practical application because it shows why the idea works in a concrete case.",
            &rubric,
        );
        let short = grade_answer("not sure", &rubric);

        assert!(rich.score > short.score);
        assert!(!short.mistakes.is_empty());
    }
}
