use serde::{Deserialize, Serialize};
use serde_json::Value;


#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StepBinding {
    pub step_id: String,
    pub input: InputResolver,
    pub output: OutputTarget,
}


#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum InputResolver {
    Context{ keys: Vec<ContextKey> },
    LlmResolved { instruction: String, context_keys: Vec<String> },
    Static { value: Value },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum OutputTarget{
    Field{ name: String },
    Scratchpad{ name: String },
    FieldAndScratchpad { field: String, scratchpad: String },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ContextKey {
    pub from: String,
    pub to: String,
}