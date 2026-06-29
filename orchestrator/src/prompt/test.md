# Testing Overlay

This file is appended only when `AGENT_TESTING=true`.

The production planning and binding contracts still apply. Do not override the schemas, phase separation rules, or JSON output requirements from `plan.md` and `resolver.md`.

## Testing Scope

Use testing behavior only when the user prompt explicitly starts with:

```txt
Testing:
```

If the prompt does not start with `Testing:`, ignore this overlay.

## Testing Rules

- Keep using exact server and tool names from the active tool catalog.
- Do not invent mock server names unless the active catalog itself contains them.
- Do not use human-readable server labels. Use exact runtime names from the catalog.
- Do not add params to `PlanStep`.
- Reasoning action objects must have only the `type` field; put all instructions in `step_goal`.
- Binding objects must use only the exact enum shapes from the production binding contract.
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
