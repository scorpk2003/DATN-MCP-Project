# Binding Strategy

## Purpose

Create one executable binding for the current `PlanStep`.

Binding maps existing `AgentContext` data into tool parameters or reasoning inputs. Binding also declares where the step output should be written after execution.

Keep the existing plan unchanged. Tool execution and tool outputs are runtime-owned.

For `ToolCall` steps, the runtime provides the selected tool input schema in the user message. The produced params must validate against that schema.

## Required Output

Return only raw JSON with this exact top-level shape:

```json
{
  "binding": {
    "step_id": "step id",
    "input": {"type": "Static", "value": {}},
    "output": {"type": "Scratchpad", "name": "scratchpad_key"},
    "expected_schema": null
  }
}
```

The response must deserialize into `StepBinding`.

## Valid Input Resolver Shapes

Prefer `Context` whenever params can be copied from existing context:

```json
{
  "type": "Context",
  "keys": [
    {"from": "goal", "to": "goal"}
  ]
}
```

Use `Static` when params are literal values or can be directly assembled from information already present in the prompt:

```json
{
  "type": "Static",
  "value": {
    "approved": true
  }
}
```

Use only `Context` or `Static` in normal binding responses. If neither shape can produce valid params, use `Static` with the safest minimal object that follows the selected tool schema. The runtime has a separate repair path for validation failures.

## Valid Output Target Shapes

Use exactly one of these:

```json
{"type": "Field", "name": "roadmap"}
```

```json
{"type": "Scratchpad", "name": "step_result"}
```

```json
{"type": "FieldAndScratchpad", "field": "roadmap", "scratchpad": "roadmap_result"}
```

Use only the fields shown in the selected output target shape.

## Valid Context Roots

You may read only these root paths:

- `session_id`
- `user_id`
- `auth_context`
- `intent_context`
- `goal`
- `topic`
- `roadmap`
- `skill_graph`
- `lesson`
- `quizz`
- `user`
- `user_confirmed`
- `scratchpad.last_obs`
- `scratchpad.debug:step_<id>`

For dependency observations, use the exact scratchpad key created by the runtime:

```txt
scratchpad.debug:step_step 1
scratchpad.debug:step_step 2
```

Only the listed root paths exist in `AgentContext`.

## Binding Rules

- Match `binding.step_id` to the current plan step id.
- For `ToolCall`, build only params accepted by the selected tool input schema.
- The params object is the direct object described by the selected tool schema.
- Required schema fields must be present in the final params object.
- Required identifiers such as `roadmapId`, `roadmapNodeId`, `lessonId`, and `userId` must come from `intent_context`, trusted auth context, or previous observations.
- Unknown fields should be omitted unless the schema allows them.
- Prefer copying existing values with `Context`.
- Use `Static` for literal params and simple objects that are already known.
- Trusted auth data is injected by Rust after binding.
- Keep `expected_schema` as `null` unless a small useful expectation is obvious.
- If there is no meaningful input for a `Reasoning` or `HumanApproval` step, use `Static` with an empty object or minimal literal object.

## Output Target Guidance

- Write stable user-facing artifacts to fields: `goal`, `topic`, `roadmap`, `skill_graph`, `lesson`, `quizz`, `user`, or `user_confirmed`.
- Write intermediate outputs to `Scratchpad`.
- Use `FieldAndScratchpad` when later steps need both a typed context field and a debug/intermediate observation.

## Valid Examples

Tool call using the learner goal:

```json
{
  "binding": {
    "step_id": "step 1",
    "input": {
      "type": "Context",
      "keys": [
        {"from": "goal", "to": "goal"}
      ]
    },
    "output": {
      "type": "FieldAndScratchpad",
      "field": "roadmap",
      "scratchpad": "roadmap_generation"
    },
    "expected_schema": null
  }
}
```

Human approval gate:

```json
{
  "binding": {
    "step_id": "step 2",
    "input": {
      "type": "Static",
      "value": {}
    },
    "output": {
      "type": "Scratchpad",
      "name": "approval_gate"
    },
    "expected_schema": null
  }
}
```

Reasoning based on previous observations:

```json
{
  "binding": {
    "step_id": "step 3",
    "input": {
      "type": "Context",
      "keys": [
        {"from": "scratchpad.last_obs", "to": "last_observation"},
        {"from": "roadmap", "to": "roadmap"}
      ]
    },
    "output": {
      "type": "Scratchpad",
      "name": "final_reasoning"
    },
    "expected_schema": null
  }
}
```
