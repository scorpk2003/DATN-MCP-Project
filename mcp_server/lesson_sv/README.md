# Lesson MCP Server

Evidence-based Lesson MCP server for the self-learn system.

Endpoint:

```txt
http://<LESSON_MCP_HOST>:<LESSON_MCP_PORT>/mcp
```

Defaults:

```txt
LESSON_MCP_HOST=127.0.0.1
LESSON_MCP_PORT=3400
RESOURCE_MCP_URL=http://127.0.0.1:3300/mcp
DATABASE_MCP_URL=http://127.0.0.1:3000/mcp
```

Implemented v0.1 tools:

```txt
get_lesson_contract
get_lesson_integration_contract
lesson_health
lesson_readiness
lesson_analyze_node
lesson_create_draft
lesson_validate_draft
lesson_finalize
lesson_grade_answer
lesson_generate_remediation
lesson_complete_session
```

Guardrails:

```txt
No direct database writes.
No web crawling or raw resource indexing.
No lesson content generation without Resource MCP evidence.
User-scoped tool calls require verified authContext.
No free-form tool outputs; responses use structured JSON envelopes.
```

Observability:

```txt
lesson_health and lesson_readiness expose in-process counters:
totalToolCalls
totalToolSuccesses
totalToolErrors
toolCalls
toolErrors
errorCodes
```

Readiness now includes structured dependency checks for Resource MCP,
Database MCP, internal token configuration, telemetry status, and v0.2
hardening completion.

Auth context:

```json
{
  "authContext": {
    "userId": "user_123",
    "verified": true,
    "scope": ["roadmap:read", "lesson:write", "lesson:evaluate", "lesson:progress"],
    "verifiedBy": "database_mcp",
    "verifiedAt": "2026-06-26T00:00:00Z"
  }
}
```

Required scopes:

```txt
lesson_analyze_node: roadmap:read
lesson_create_draft: lesson:write
lesson_validate_draft: lesson:write
lesson_finalize: lesson:write
lesson_grade_answer: lesson:evaluate
lesson_generate_remediation: lesson:evaluate
lesson_complete_session: lesson:progress
```

Orchestrator-managed v0.1 flow:

```txt
1. Database MCP fetches roadmap node and learner context.
2. Lesson MCP runs lesson_analyze_node.
3. Orchestrator calls Resource MCP using returned resource queries.
4. Lesson MCP runs lesson_create_draft with resource candidates.
5. Lesson MCP runs lesson_validate_draft.
6. Lesson MCP runs lesson_finalize.
7. Orchestrator executes returned Database MCP calls.
8. Lesson MCP grades answers and completes session progress payloads.
```

Database contract status:

```txt
status: missing_database_tools
mapping: mcp_server/lesson_sv/docs/database_mcp_contract_mapping.md
```

`lesson_finalize` now returns a transaction-shaped call plan with idempotency key,
rollback policy, expected outputs, and a contract warning. The current Database MCP
does not yet expose lesson-specific tools such as `create_lesson` or
`create_lesson_block`, so Orchestrator must treat the plan as not executable until
Database MCP adds those tools or provides an official alternative mapping.

Verification:

```txt
cargo check -p lesson_sv
cargo test -p lesson_sv
```
