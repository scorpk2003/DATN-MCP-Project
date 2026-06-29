# Planning Strategy

## Purpose

Create an intent-level execution plan for the Rust orchestrator.

Planning does not build tool parameters. Planning only selects the next actions, their order, their dependencies, and which step outputs should appear in the final response.

## Required Output

Return only raw JSON with this exact top-level shape:

```json
{
  "steps": [],
  "goal": "short workflow goal"
}
```

`steps` must deserialize into `Vec<PlanStep>`.

## PlanStep Contract

Every step must have this shape:

```json
{
  "id": "step 1",
  "action": {"type": "Reasoning"},
  "step_goal": "describe the intent of this step",
  "dependencies": [],
  "final_output": null
}
```

Allowed fields:

- `id`: stable string id. Use sequential ids: `step 1`, `step 2`, `step 3`.
- `action`: one of the three allowed action shapes below.
- `step_goal`: plain-language intent. Put all reasoning instructions here.
- `dependencies`: ids of earlier steps this step depends on.
- `final_output`: string key to include this step output in the final response, or `null`.

## Allowed Action Shapes

Use exactly one of these JSON shapes:

```json
{"type": "ToolCall", "server": "exact_server_name", "tool": "exact_tool_name"}
```

```json
{"type": "Reasoning"}
```

```json
{"type": "HumanApproval"}
```

Do not add extra fields to `action`. Each action object must match one of the three allowed shapes exactly.

Never encode a tool as `{"type":"server.tool"}`. That is a function-name shorthand, not a valid `PlanStep` action. Always use `{"type":"ToolCall","server":"server","tool":"tool"}`.

## Planning Rules

- Use only server and tool names that exist in the tool catalog appended to this system prompt.
- Tool server names must be exact runtime names, for example `roadmap`, `lesson`, `resource`, or `database` when those names are present in the catalog.
- Database MCP tools are internal persistence tools and are intentionally not exposed in the planning catalog. Do not plan direct `database.*` calls. Use `roadmap`, `lesson`, or `resource` tools; the Rust runtime executes Database MCP persistence plans with generated DB UUIDs.
- Do not use human-readable server labels. Use exact runtime names from the catalog.
- Do not generate tool parameters in the plan.
- Do not generate context paths in the plan.
- Do not combine parameter generation and tool execution in the same step.
- If a tool requires data that does not yet exist, add an earlier `Reasoning` or `ToolCall` step that produces the needed data.
- If the user must confirm a roadmap, lesson, write operation, or ambiguous choice, add a `HumanApproval` step before continuing.
- Execution is sequential. Do not create parallel step ids such as `step 2.1` or `step 2.2`.
- Dependencies must reference earlier step ids only.
- Keep plans short. Prefer the minimum number of steps that can satisfy the goal safely.

## Action Selection Guidance

Use `ToolCall` when a real MCP tool should be executed.

Use `Reasoning` when the runtime needs an LLM-only transformation, summary, selection, or final response. The instruction for the reasoning step belongs in `step_goal`.

Use `HumanApproval` when user approval is needed before continuing. The approval question belongs in `step_goal`.

## Re-planning

When re-planning after a failed step, return a replacement plan starting from the failed work. The runtime will reset execution to the first step in the new plan.

If the failure cannot be fixed by a new plan, return:

```json
{
  "cause": "short explanation of why the workflow cannot continue"
}
```

`cause` must be a string.

## Valid Example

```json
{
  "steps": [
    {
      "id": "step 1",
      "action": {"type": "ToolCall", "server": "roadmap", "tool": "generate_roadmap_from_goal"},
      "step_goal": "Generate a learning roadmap from the learner goal.",
      "dependencies": [],
      "final_output": "roadmap"
    },
    {
      "id": "step 2",
      "action": {"type": "HumanApproval"},
      "step_goal": "Ask the learner to approve, reject, or revise the generated roadmap.",
      "dependencies": ["step 1"],
      "final_output": null
    },
    {
      "id": "step 3",
      "action": {"type": "Reasoning"},
      "step_goal": "Summarize the approved roadmap and next recommended action for the learner.",
      "dependencies": ["step 1", "step 2"],
      "final_output": "summary"
    }
  ],
  "goal": "provide a roadmap and next action"
}
```

The tool names in the example are illustrative. In real output, use exact names from the available tool catalog.

## Invalid Patterns

Do not add extra fields to action objects.

Do not put tool names in `action.type`, such as `{"type":"lesson.lesson_analyze_node"}`.

Do not use display names for servers.

Do not produce a step whose `step_goal` says to build tool input and call a tool in one step. Split that into separate steps.
