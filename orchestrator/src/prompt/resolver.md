# Binding Strategy

## Phase Goal
- You will build input resolver - output target for step before executing. Your goal is to generate executable runtime bindings between available context and step execution..

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
- Step-Output: Enum that expect when finish step.
```json
// Write to field of Agent Context: goal, lesson, quizz, practice, ...
{"type": "Field", "name": "field_name"}

// Write to scratchdpad (debug or intermediate)
{"type": "Scratchpad", "name": "scratchpad_name"}

// Both
{"type": "FieldAndScratchpad", "field": "field_name", "scratchpad": "scratchpad_name"}
```

## Strategy Flow
1. 

## Ouput Format
- Return raw Json following this format:
```json
{
    "binding": {
        "step_id": "id_or_name_of_current_step",
        "input": "input_resolver",
        "output": "output_target",
    }
}
```

## Example
- Prompt:
```
Use tool: extrac_topic from server: Roadmap Server for step: step 03. With Dependencies: step 02 {...}.
```
-> Return:
```json
{
    "binding": {
        "step_id": "step 03",
        "input": {
            "type": "Context",
            "keys": [{
                "from": "extract_topic",
                "to": "goal"
            }]
        },
        "output": {
            "type": "Field",
            "name": "roadmap",
            "schema": {
                "topic": "array"
            }
        }
    }
}
```