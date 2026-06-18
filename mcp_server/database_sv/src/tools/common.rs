use serde_json::{Value, json};
use tokio_postgres::Row;
use uuid::Uuid;

pub fn deleted(count: u64) -> Value {
    json!({ "deleted": count > 0, "affected_rows": count })
}

pub fn uuid_to_string(value: Uuid) -> String {
    value.to_string()
}

pub fn user(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "firebase_uid": row.get::<_, String>("firebase_uid"),
        "display_name": row.get::<_, Option<String>>("display_name"),
        "email": row.get::<_, Option<String>>("email"),
        "created_at": row.get::<_, Option<String>>("created_at"),
        "updated_at": row.get::<_, Option<String>>("updated_at"),
    })
}

pub fn project(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "user_id": uuid_to_string(row.get("user_id")),
        "title": row.get::<_, String>("title"),
        "description": row.get::<_, Option<String>>("description"),
        "status": row.get::<_, String>("status"),
        "created_at": row.get::<_, Option<String>>("created_at"),
        "updated_at": row.get::<_, Option<String>>("updated_at"),
    })
}

pub fn roadmap(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "project_id": uuid_to_string(row.get("project_id")),
        "version": row.get::<_, i32>("version"),
        "title": row.get::<_, Option<String>>("title"),
        "generated_by": row.get::<_, Option<String>>("generated_by"),
        "created_at": row.get::<_, Option<String>>("created_at"),
    })
}

pub fn phase(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "roadmap_id": uuid_to_string(row.get("roadmap_id")),
        "phase_order": row.get::<_, i32>("phase_order"),
        "title": row.get::<_, String>("title"),
        "description": row.get::<_, Option<String>>("description"),
        "estimated_days": row.get::<_, Option<i32>>("estimated_days"),
    })
}

pub fn milestone(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "phase_id": uuid_to_string(row.get("phase_id")),
        "milestone_order": row.get::<_, i32>("milestone_order"),
        "title": row.get::<_, String>("title"),
        "description": row.get::<_, Option<String>>("description"),
    })
}

pub fn task(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "milestone_id": uuid_to_string(row.get("milestone_id")),
        "task_order": row.get::<_, i32>("task_order"),
        "title": row.get::<_, String>("title"),
        "description": row.get::<_, Option<String>>("description"),
        "estimated_hours": row.get::<_, Option<i32>>("estimated_hours"),
        "difficulty": row.get::<_, Option<String>>("difficulty"),
        "status": row.get::<_, Option<String>>("status"),
    })
}

pub fn progress(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "user_id": uuid_to_string(row.get("user_id")),
        "task_id": uuid_to_string(row.get("task_id")),
        "status": row.get::<_, String>("status"),
        "progress_percent": row.get::<_, Option<i32>>("progress_percent"),
        "started_at": row.get::<_, Option<String>>("started_at"),
        "completed_at": row.get::<_, Option<String>>("completed_at"),
    })
}

pub fn resource(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "task_id": uuid_to_string(row.get("task_id")),
        "resource_type": row.get::<_, Option<String>>("resource_type"),
        "title": row.get::<_, Option<String>>("title"),
        "url": row.get::<_, Option<String>>("url"),
        "description": row.get::<_, Option<String>>("description"),
    })
}

pub fn note(row: Row) -> Value {
    json!({
        "id": uuid_to_string(row.get("id")),
        "user_id": uuid_to_string(row.get("user_id")),
        "task_id": row.get::<_, Option<Uuid>>("task_id").map(uuid_to_string),
        "content": row.get::<_, String>("content"),
        "created_at": row.get::<_, Option<String>>("created_at"),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn deleted_reports_zero_rows() {
        assert_eq!(deleted(0), json!({ "deleted": false, "affected_rows": 0 }));
    }

    #[test]
    fn deleted_reports_affected_rows() {
        assert_eq!(deleted(1), json!({ "deleted": true, "affected_rows": 1 }));
    }
}
