# Lesson MCP Server

Lesson MCP Server tao, validate, finalize va danh gia bai hoc dua tren resource evidence. Server nay nam trong flow do Orchestrator quan ly: Orchestrator lay roadmap node, goi Resource MCP de lay evidence, dua evidence vao Lesson MCP, sau do tu thuc thi Database MCP call plan neu can luu.

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

## Muc dich

```txt
Analyze roadmap node thanh lesson requirement.
Pack resource evidence va chi tao lesson khi evidence du chat luong.
Sinh lesson draft gom content blocks, exercise, quiz, rubric va source references.
Validate lesson theo objective/resource/exercise/quiz/content policy.
Finalize lesson thanh payload/call plan san sang cho Database MCP.
Grade answer, tao remediation va tinh progress payload sau khi hoan thanh session.
```

## Tools

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

## Y nghia cua nhom tool

```txt
Contract/readiness
  get_lesson_contract, get_lesson_integration_contract, lesson_health, lesson_readiness.

Lesson creation
  lesson_analyze_node, lesson_create_draft, lesson_validate_draft, lesson_finalize.

Learner evaluation
  lesson_grade_answer, lesson_generate_remediation, lesson_complete_session.
```

## Services noi bo

```txt
node_analyzer     Chuan hoa objective, prerequisite gap, resource query va exercise type.
resource_packer   Loc, dedupe, chon chunks va tinh coverage tu resource candidates.
lesson_generator  Tao lesson draft co cau truc tu packed evidence.
lesson_validator  Kiem tra chat luong lesson truoc khi persist.
finalizer         Doi lesson draft thanh Database MCP call descriptors.
grading           Cham cau tra loi theo rubric.
remediation       Tao goi on tap/sua loi dua tren grading result va resource refs.
progress_policy   Tinh mastery, completion status va progress payload.
request_guard     Chan input thieu/sai/qua lon.
access_policy     Yeu cau authContext verified va scope dung.
observability     Dem tool call, success, error va exposed qua health/readiness.
```

## Guardrails

```txt
No direct database writes.
No web crawling or raw resource indexing.
No lesson content generation without Resource MCP evidence.
User-scoped tool calls require verified authContext.
No free-form tool outputs; responses use structured JSON envelopes.
```

## Observability

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
