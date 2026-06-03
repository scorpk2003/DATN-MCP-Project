use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentContext {
    pub session_id: String,
    pub goal: Option<String>,
    pub topic: Option<String>,
    pub roadmap: Option<Value>,
    pub skill_graph: Option<Value>,
    pub lesson: Option<Value>,
    pub quizz: Option<Value>,
    pub user: Option<Value>,
    pub user_confirmed: bool,
    pub scratchpad: Map<String, Value>,
}

impl Default for AgentContext {
    fn default() -> Self {
        Self {
            session_id: String::new(),
            goal: None,
            topic: None,
            roadmap: None,
            skill_graph: None,
            lesson: None,
            quizz: None,
            user: None,
            user_confirmed: false,
            scratchpad: Map::new(),
        }
    }
}
impl AgentContext {
    pub fn write_obs(&mut self, step_id: String, obs: &Value) {
        self.scratchpad.insert("last_obs".into(), obs.clone());
        self.scratchpad.insert(format!("debug:step_{step_id}"), obs.clone());
    }

    pub fn last_obs(&self) -> Option<&Value> {
        self.scratchpad.get("last_obs")
    }
}