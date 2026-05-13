use serde_json::{Value};


pub struct PlanStep {
    pub id: String,
    pub server: Option<String>,
    pub tool: Option<String>,
    pub input_mapping: InputResolver,
    pub output_mapping: OutputTarget,
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
