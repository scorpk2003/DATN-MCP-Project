use serde_json::{Map, Value};


pub struct PlanStep {
    pub id: String,
    pub server: Option<String>,
    pub tool: Option<String>,
    pub input_mapping: Map<String, Value>,
    pub output_mapping: Map<String, Value>,
    pub waitting: bool,
    pub re_plan: bool,
}

pub enum InputResolver {
    Context(Vec<ContextKey>),
    LlmResolved,
    Static(Value),
}

pub enum OutputTarget{
    Field(String),
    Scratchpad(String),
    FieldAndScratchpad { field: String, scratchpad: String },
}

pub struct ContextKey {
    pub from: String,
    pub to: String,
}
