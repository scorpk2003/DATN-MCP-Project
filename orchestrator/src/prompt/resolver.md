# Binding Strategy

## Purpose

Create one executable binding for the current `PlanStep`.

Binding maps existing `AgentContext` data into tool parameters or reasoning inputs. Binding also declares where the step output should be written after execution.

Do not change the plan. Do not call tools. Do not invent tool outputs.

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

`LlmResolved` is a last resort. Avoid it unless no `Context` or `Static` binding can express the required input. Do not use `LlmResolved` for simple fields such as `goal`, `topic`, `user_id`, `session_id`, `roadmap`, `lesson`, or `auth_context`.

If `LlmResolved` is unavoidable, keep it narrow:

```json
{
  "type": "LlmResolved",
  "instruction": "Fill only the missing tool parameters from the listed context keys.",
  "context_keys": ["goal", "scratchpad.last_obs"]
}
```

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

Do not add any extra field beyond the selected output target shape.

## Valid Context Roots

You may read only these root paths:

- `session_id`
- `user_id`
- `auth_context`
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

Do not use any root path outside the list above; unknown roots do not exist in `AgentContext`.

## Binding Rules

- Match `binding.step_id` to the current plan step id.
- For `ToolCall`, build only params accepted by the selected tool input schema.
- Do not wrap params under names such as `params`, `input`, or `arguments` unless the selected tool schema explicitly requires that wrapper.
- Required schema fields must be present in the final params object.
- Unknown fields should be omitted unless the schema allows them.
- Prefer copying existing values with `Context`.
- Use `Static` for literal params and simple objects that are already known.
- Use `LlmResolved` only for genuinely ambiguous transformations that cannot be represented with `Context` or `Static`.
- Never create trusted auth data. If a tool requires auth, the Rust runtime injects trusted auth after binding.
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

Last-resort LLM filling:

```json
{
  "binding": {
    "step_id": "step 4",
    "input": {
      "type": "LlmResolved",
      "instruction": "Create only the missing search fields from the learner goal and the latest observation.",
      "context_keys": ["goal", "scratchpad.last_obs"]
    },
    "output": {
      "type": "Scratchpad",
      "name": "resource_search"
    },
    "expected_schema": null
  }
}
```
