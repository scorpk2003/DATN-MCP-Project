# Lesson MCP to Database MCP Contract Mapping

## Current Status

**Status:** `missing_database_tools`

Lesson MCP v0.2 finalizer produces a lesson-specific persistence plan, but the current `database_sv` server does not expose lesson-specific tools yet.

Current Database MCP tools observed:

```txt
create_user
get_user_by_id
create_project
get_project
update_project
delete_project
list_projects
create_roadmap
get_roadmap
delete_roadmap
list_project_roadmap
create_phase
get_phase
update_phase
delete_phase
create_milestone
get_milestone
update_milestone
delete_milestone
create_task
get_task
update_task
delete_task
update_task_progress
get_task_progress
get_project_progress
create_resource
delete_resource
list_resources
search_projects
search_tasks
search_notes
get_user_statistics
get_learning_history
get_multi_schema
get_schema
get_health_check
```

## Required Lesson Persistence Tools

Lesson MCP finalizer currently expects these Database MCP tools:

```txt
create_lesson
create_lesson_block
link_lesson_resource
create_lesson_exercise
create_lesson_quiz
```

These tools are not present in `database_sv` yet. Until they exist, Orchestrator must not execute the lesson finalizer call plan as a verified Database MCP workflow.

## Required Call Plan Shape

Lesson MCP finalizer returns:

```json
{
  "schemaVersion": "lesson_draft_v1",
  "notPersisted": true,
  "idempotencyKey": "lesson:topic:level:block_count:status",
  "orchestratorPersistencePlan": {
    "databaseMcpCalls": [],
    "databaseCallPlan": {
      "contractStatus": "missing_database_tools",
      "transactionRequired": true,
      "idempotencyKey": "string",
      "steps": [],
      "rollbackPolicy": {
        "onFailure": "rollback_all"
      }
    }
  }
}
```

## Mapping Details

### create_lesson

Lesson MCP step:

```json
{
  "tool": "database_mcp.create_lesson",
  "args": {
    "idempotencyKey": "string",
    "title": "string",
    "topic": "string",
    "level": "beginner | intermediate | advanced",
    "objectives": ["string"],
    "prerequisites": ["string"],
    "estimatedMinutes": 45,
    "status": "ready",
    "assessmentRubric": {}
  }
}
```

Expected Database MCP response:

```json
{
  "ok": true,
  "lessonId": "string"
}
```

### create_lesson_block

Depends on `create_lesson`.

```json
{
  "tool": "database_mcp.create_lesson_block",
  "args": {
    "lessonId": "${lesson.lessonId}",
    "block": {}
  }
}
```

### link_lesson_resource

Depends on `create_lesson`.

```json
{
  "tool": "database_mcp.link_lesson_resource",
  "args": {
    "lessonId": "${lesson.lessonId}",
    "resource": {}
  }
}
```

### create_lesson_exercise

Depends on `create_lesson`.

```json
{
  "tool": "database_mcp.create_lesson_exercise",
  "args": {
    "lessonId": "${lesson.lessonId}",
    "exercise": {}
  }
}
```

### create_lesson_quiz

Depends on `create_lesson`.

```json
{
  "tool": "database_mcp.create_lesson_quiz",
  "args": {
    "lessonId": "${lesson.lessonId}",
    "quiz": {}
  }
}
```

## Required Database MCP Work

Before this contract can become `verified`, Database MCP should add lesson entities/tools or provide an official alternative mapping.

Minimum required entities:

```txt
lessons
lesson_blocks
lesson_resources
lesson_exercises
lesson_quizzes
lesson_attempts
lesson_progress
```

Minimum required semantics:

```txt
transaction_required: true
rollback_policy: rollback_all
idempotency_key: required for create_lesson
```

