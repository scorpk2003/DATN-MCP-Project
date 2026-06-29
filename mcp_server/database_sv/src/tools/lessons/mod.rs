use std::sync::Arc;

use anyhow::Result;
use serde_json::{Value, json};
use tokio::sync::Mutex;

use crate::{
    provider::SchemaProvider,
    schemas::{
        CreateLessonBlockParam, CreateLessonExerciseParam, CreateLessonParam,
        CreateLessonQuizParam, LinkLessonResourceParam,
    },
    tools::common,
};

#[derive(Debug, Clone)]
pub struct LessonTool {
    pub provider: Arc<Mutex<SchemaProvider>>,
}

impl LessonTool {
    pub fn new(provider: Arc<Mutex<SchemaProvider>>) -> Self {
        Self { provider }
    }

    pub async fn create_lesson(&self, param: CreateLessonParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_lesson_tables(&conn).await?;
        let status = param.status.unwrap_or_else(|| "ready".to_string());
        let assessment_rubric = param
            .assessment_rubric
            .map(|value| value.to_string())
            .unwrap_or_else(|| "{}".to_string());
        let row = conn
            .query_one(
                "INSERT INTO lessons (
                    idempotency_key, title, topic, level, objectives, prerequisites,
                    estimated_minutes, status, assessment_rubric
                 )
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                 ON CONFLICT (idempotency_key) DO UPDATE SET
                    title = EXCLUDED.title,
                    topic = EXCLUDED.topic,
                    level = EXCLUDED.level,
                    objectives = EXCLUDED.objectives,
                    prerequisites = EXCLUDED.prerequisites,
                    estimated_minutes = EXCLUDED.estimated_minutes,
                    status = EXCLUDED.status,
                    assessment_rubric = EXCLUDED.assessment_rubric
                 RETURNING id",
                &[
                    &param.idempotency_key,
                    &param.title,
                    &param.topic,
                    &param.level,
                    &param.objectives,
                    &param.prerequisites,
                    &param.estimated_minutes,
                    &status,
                    &assessment_rubric,
                ],
            )
            .await?;
        Ok(json!({ "ok": true, "lessonId": common::uuid_to_string(row.get("id")) }))
    }

    pub async fn create_lesson_block(&self, param: CreateLessonBlockParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_lesson_tables(&conn).await?;
        let payload = param.block.to_string();
        let row = conn
            .query_one(
                "INSERT INTO lesson_blocks (lesson_id, block_payload)
                 VALUES ($1, $2)
                 RETURNING id",
                &[&param.lesson_id, &payload],
            )
            .await?;
        Ok(json!({ "ok": true, "blockId": common::uuid_to_string(row.get("id")) }))
    }

    pub async fn link_lesson_resource(&self, param: LinkLessonResourceParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_lesson_tables(&conn).await?;
        let payload = param.resource.to_string();
        let row = conn
            .query_one(
                "INSERT INTO lesson_resources (lesson_id, resource_payload)
                 VALUES ($1, $2)
                 RETURNING id",
                &[&param.lesson_id, &payload],
            )
            .await?;
        Ok(json!({ "ok": true, "lessonResourceId": common::uuid_to_string(row.get("id")) }))
    }

    pub async fn create_lesson_exercise(&self, param: CreateLessonExerciseParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_lesson_tables(&conn).await?;
        let payload = param.exercise.to_string();
        let row = conn
            .query_one(
                "INSERT INTO lesson_exercises (lesson_id, exercise_payload)
                 VALUES ($1, $2)
                 RETURNING id",
                &[&param.lesson_id, &payload],
            )
            .await?;
        Ok(json!({ "ok": true, "exerciseId": common::uuid_to_string(row.get("id")) }))
    }

    pub async fn create_lesson_quiz(&self, param: CreateLessonQuizParam) -> Result<Value> {
        let mut provider = self.provider.lock().await;
        let conn = provider.get_connections().await?;
        ensure_lesson_tables(&conn).await?;
        let payload = param.quiz.to_string();
        let row = conn
            .query_one(
                "INSERT INTO lesson_quizzes (lesson_id, quiz_payload)
                 VALUES ($1, $2)
                 RETURNING id",
                &[&param.lesson_id, &payload],
            )
            .await?;
        Ok(json!({ "ok": true, "quizId": common::uuid_to_string(row.get("id")) }))
    }
}

async fn ensure_lesson_tables(conn: &deadpool_postgres::Object) -> Result<()> {
    conn.batch_execute(
        "
        CREATE EXTENSION IF NOT EXISTS pgcrypto;

        CREATE TABLE IF NOT EXISTS lessons (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            idempotency_key text NOT NULL UNIQUE,
            title text NOT NULL,
            topic text NOT NULL,
            level text NOT NULL,
            objectives text[] NOT NULL DEFAULT '{}',
            prerequisites text[] NOT NULL DEFAULT '{}',
            estimated_minutes integer,
            status text NOT NULL DEFAULT 'ready',
            assessment_rubric text NOT NULL DEFAULT '{}',
            created_at timestamptz NOT NULL DEFAULT now(),
            updated_at timestamptz NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS lesson_blocks (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            lesson_id uuid NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
            block_payload text NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS lesson_resources (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            lesson_id uuid NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
            resource_payload text NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS lesson_exercises (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            lesson_id uuid NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
            exercise_payload text NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now()
        );

        CREATE TABLE IF NOT EXISTS lesson_quizzes (
            id uuid PRIMARY KEY DEFAULT gen_random_uuid(),
            lesson_id uuid NOT NULL REFERENCES lessons(id) ON DELETE CASCADE,
            quiz_payload text NOT NULL,
            created_at timestamptz NOT NULL DEFAULT now()
        );
        ",
    )
    .await?;
    Ok(())
}
