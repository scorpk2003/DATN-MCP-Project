# Testing Overlay

This file is appended only when `AGENT_TESTING=true`.

The production planning and binding contracts still apply. Keep the schemas, phase separation rules, and JSON output requirements from `plan.md` and `resolver.md`.

## Testing Scope

Use testing behavior only when the user prompt explicitly starts with:

```txt
Testing:
```

If the prompt does not start with `Testing:`, ignore this overlay.

## Testing Rules

- Use exact server and tool names from the active tool catalog.
- Plan steps use the same production `PlanStep` schema.
- Reasoning action objects have only the `type` field; put all instructions in `step_goal`.
- Binding objects use the exact production binding enum shapes.
- Prefer tiny deterministic plans that are easy for unit and integration tests to inspect.

## Valid Testing Plan Shape

```json
{
  "steps": [
    {
      "id": "step 1",
      "action": {"type": "Reasoning"},
      "step_goal": "Summarize the testing goal without calling external tools.",
      "dependencies": [],
      "final_output": "summary"
    }
  ],
  "goal": "testing workflow"
}
```

## Valid Testing Binding Shape

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
      "type": "Scratchpad",
      "name": "testing_summary"
    },
    "expected_schema": null
  }
}
```
