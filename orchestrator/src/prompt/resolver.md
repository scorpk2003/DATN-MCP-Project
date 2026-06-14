# Binding Strategy

## Phase Goal
- You will build input resolver - output target for step before executing. Your goal is to generate executable runtime bindings between available context and step execution. If context path or parameters are ambiguous, prefer partial bindings and defer exact resolution to execution observations.

## Input Resolver Format
- Step-Input: Enum that keep context during flow.
```json
// Context: Build params from main context
{"type": "Context", "keys": [{
    "from": "context_path",
    "to": "param_name"
}]}

// LlmResolved: Reasoning and Generate params - Params complicated
{"type": "LlmResolved", "instruction": "instruction_to_build_prompt", "context_keys": ["context_keys"]}

// Static: Hard-code params - easy step
{"type": "Static", "value":}
```

## OutputTarget Format
- Step-Output: Enum that expect when finish step. Responsibility for WHERE data goes.
```json
// Write to field of Agent Context: goal, lesson, quizz, practice, ...
{"type": "Field", "name": "field_name"}

// Write to scratchdpad (debug or intermediate)
{"type": "Scratchpad", "name": "scratchpad_name"}

// Both
{"type": "FieldAndScratchpad", "field": "field_name", "scratchpad": "scratchpad_name"}
```

## Strategy Flow
1. Context: Knowing context of flow or at least knowing previous step - current step - next step.
2. Dependencies: Knowing all dependencies ready for build input resolver.
3. Input Resolver: Build input resolver for step, if ambigous about previous step or can't specified exact context path, params, etc... so watch Observation to building.
4. Output Target: Based on action, build expected output after execute step.
5. Schema Expect: Expected relative schema using evaluate after execution.

## Ouput Format
- Return raw Json following this format:
```json
{
    "binding": {
        "step_id": "id_or_name_of_current_step",
        "input": "input_resolver",
        "output": "output_target",
        "expected_schema": "schema_expect_after_execution", // What executor expects
    }
}
```

## Example
- Prompt:
```
Use tool: extract_topic from server: Roadmap Server for step: step 03. Dependencies: step 02 {
    "call_tool": "extract_topic",
    "output": {
        "topic": [{"name": "topic_name", "chapter 1": "chapter_01_name", ...}]
    }
}.
```
-> Return:
```json
{
    "binding": {
        "step_id": "step 03",
        "input": {
            "type": "Context",
            "keys": [{
                "from": "steps.extract_topic.output.topic",
                "to": "goal"
            }]
        },
        "output": {
            "type": "Field",
            "name": "roadmap",
            "mode": "insert", // insert | overwrite | merge
        },
        "expected_schema": {
            "topics": ["array"]
        }
    }
}
```
- Prompt:
```
Search all topics with infomation below. Sort ASC following ranking as couting number user finish 1 topic:
{
    "call_tool": {
        "name: "get_all_topic",
        "output":  {       
            "topic": [
                {
                    "id": "topic_id_01",
                    "name": "name_topic_01",
                    "course": ["course_topic_01"],
                    "chapter": "chapter_topic_01",
                    "price": "price_topic_01"
                },
                {...},
                {...}
            ]
        },
    },
    "dependencies": [
        {
            "step_id": "id_of_step",
            "type": "action_of_step", // Call Tool | LlmResolved | Static
            "server": "server_name",
            "tool_name": "name_of_tool",
            "output": [
                {
                    "id": "user_id",
                    "name": "user_name",
                    "course": {
                        "course_id": "id_course",
                        "course_name": "course_name",
                        "state": "learn_state", // ready | learning | completed
                    }
                },
                {...},
                {...}
            ]
        },
        {...},
        {...},
    ]
}
```
-> Return:
```json
{
    "binding": {
        "step_id": "step_02",
        "input": {
            "type": "LlmResolved",
            "target": ["query", "sort", "asc"],
            "instruction": "Generate optimize query before search",
            "context_keys": [
                "call_tool.output.topic",
                "dependencies.output",
            ],
        },
        "output": {
            "type": "FieldAndSratchpad",
            "Field": "topic",
            "Scratchpad": "ranking_topic",
            "mode": "insert",
        },
        "expected_schema": {
            "topics": ["topic"],
        }
    },
}
```
- Prompt:
```
Return roadmap to user
```
-> Return:
```json
{
    "binding" : {
        "input": {
            "type": "Static",
            "value": {
                "approval_required": true,
            }
        },
        "output": {
            "type": "Scratchpad",
            "name": "approve_roadmap",
            "mode": "overwrite",
        },
        "expected_schema": {
            "approval_required": true,
        }
    }
}
```

## Critical Rules
- Never hallucinate context paths.
- Only use available dependencies and context.
- Prefer Context over LlmResolved when possible.
- Binding must be executable at runtime.
- Avoid unnecessary LlmResolved.