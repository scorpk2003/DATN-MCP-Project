use serde_json::json;

use crate::domain::{
    LessonCompleteSessionParam, LessonCompletionResult, ProgressPayload, SessionSummary,
};

pub fn complete_session(param: LessonCompleteSessionParam) -> LessonCompletionResult {
    let mastery_score = mastery_score(&param.session_summary);
    let completed = mastery_score >= 0.8;
    let completion_status = if completed {
        "completed"
    } else {
        "needs_review"
    }
    .to_string();

    LessonCompletionResult {
        status: completion_status.clone(),
        mastery_score,
        progress_payload: ProgressPayload {
            lesson_id: param.lesson_id,
            session_id: param.session_id,
            user_id: param.user_id,
            completion_status,
            mastery_score,
            weak_topics: weak_topics(&param.session_summary, mastery_score),
            completed_at: None,
        },
        next_action: json!({
            "type": if completed { "next_roadmap_node" } else if mastery_score >= 0.5 { "extra_practice" } else { "review" },
            "reason": if completed {
                "Mastery score meets completion threshold."
            } else if mastery_score >= 0.5 {
                "Mastery score is partial; learner should practice weak areas before moving on."
            } else {
                "Mastery score is below review threshold."
            }
        }),
    }
}

pub fn mastery_score(summary: &SessionSummary) -> f32 {
    let exercise_average = average_score(
        &summary
            .exercise_scores
            .iter()
            .map(|score| score.score)
            .collect::<Vec<_>>(),
    );
    let quiz_score = summary.quiz_score;
    let checkpoint_score = summary.checkpoint_score;
    let completion_ratio = if summary.completed_blocks.is_empty() {
        Some(0.0)
    } else {
        Some(1.0)
    };

    weighted_average(&[
        (exercise_average, 0.4),
        (quiz_score, 0.3),
        (checkpoint_score, 0.2),
        (completion_ratio, 0.1),
    ])
}

fn weighted_average(parts: &[(Option<f32>, f32)]) -> f32 {
    let available_weight = parts
        .iter()
        .filter(|(score, _)| score.is_some())
        .map(|(_, weight)| weight)
        .sum::<f32>();

    if available_weight <= f32::EPSILON {
        return 0.0;
    }

    parts
        .iter()
        .filter_map(|(score, weight)| score.map(|score| score.clamp(0.0, 1.0) * weight))
        .sum::<f32>()
        / available_weight
}

fn average_score(scores: &[f32]) -> Option<f32> {
    if scores.is_empty() {
        return None;
    }

    Some(
        scores
            .iter()
            .map(|score| score.clamp(0.0, 1.0))
            .sum::<f32>()
            / scores.len() as f32,
    )
}

fn weak_topics(summary: &SessionSummary, mastery_score: f32) -> Vec<String> {
    let mut topics = Vec::new();

    if summary
        .exercise_scores
        .iter()
        .any(|score| score.score < 0.8)
    {
        topics.push("exercise_application".to_string());
    }
    if summary.quiz_score.map(|score| score < 0.8).unwrap_or(false) {
        topics.push("quiz_concepts".to_string());
    }
    if summary
        .checkpoint_score
        .map(|score| score < 0.8)
        .unwrap_or(false)
    {
        topics.push("checkpoint_reasoning".to_string());
    }
    if mastery_score < 0.5 {
        topics.push("lesson_foundation".to_string());
    }

    topics
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{ExerciseScore, SessionSummary};

    #[test]
    fn computes_completion_when_scores_are_high() {
        let summary = SessionSummary {
            completed_blocks: vec!["block".to_string()],
            exercise_scores: vec![ExerciseScore {
                exercise_id: "exercise".to_string(),
                score: 0.9,
            }],
            quiz_score: Some(0.8),
            checkpoint_score: Some(0.85),
            time_spent_minutes: Some(30),
        };

        assert!(mastery_score(&summary) >= 0.8);
    }

    #[test]
    fn redistributes_weight_when_quiz_is_missing() {
        let summary = SessionSummary {
            completed_blocks: vec!["block".to_string()],
            exercise_scores: vec![ExerciseScore {
                exercise_id: "exercise".to_string(),
                score: 0.9,
            }],
            quiz_score: None,
            checkpoint_score: None,
            time_spent_minutes: None,
        };

        assert!(mastery_score(&summary) > 0.9);
    }
}
