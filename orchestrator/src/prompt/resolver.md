# Resolver Strategy

## Phase Goal
- You will build input/output resolver for each step before execute program. Your goal is ensure that program can using tools exactly and clarify HOW data flow in step.

## Input Resolver Format
- Step-Input: Enum that keep context during flow.
```json
// Context: Build params from main context
{"type": "Context", "keys": [{
    "from": "field_name",
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

## Ouput Format