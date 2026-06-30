use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AuthContext {
    #[serde(rename = "userId", alias = "user_id")]
    pub user_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roles: Vec<String>,
    #[serde(rename = "scope", alias = "scopes", default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub verified: bool,
    #[serde(rename = "verifiedBy", alias = "verified_by", default)]
    pub verified_by: Option<String>,
    #[serde(rename = "verifiedAt", alias = "verified_at", default)]
    pub verified_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentContext {
    pub session_id: String,
    pub user_id: Option<String>,
    pub auth_context: Option<AuthContext>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent_context: Option<Value>,
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
            user_id: None,
            auth_context: None,
            intent_context: None,
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
    pub fn write_obs(&mut self, step_id: &str, obs: &Value) {
        self.scratchpad.insert("last_obs".into(), obs.clone());
        self.scratchpad
            .insert(format!("debug:step_{step_id}"), obs.clone());
    }

    pub fn write_field(&mut self, field_name: &str, value: &Value) {
        match field_name {
            "goal" => self.goal = value.as_str().map(|s| s.to_string()),
            "intent_context" => self.intent_context = Some(value.clone()),
            "topic" => self.topic = value.as_str().map(|s| s.to_string()),
            "roadmap" => self.roadmap = Some(value.clone()),
            "skill_graph" => self.skill_graph = Some(value.clone()),
            "lesson" => self.lesson = Some(value.clone()),
            "quizz" => self.quizz = Some(value.clone()),
            "user" => self.user = Some(value.clone()),
            "user_confirmed" => self.user_confirmed = value.as_bool().unwrap_or(false),
            _ => {
                // For any other field, we can store it in the scratchpad
                self.scratchpad
                    .insert(field_name.to_string(), value.clone());
            }
        }
    }

    pub fn last_obs(&self) -> Option<&Value> {
        self.scratchpad.get("last_obs")
    }

    pub fn get_obs(&self, step_id: &str) -> Option<&Value> {
        self.scratchpad.get(&format!("debug:step_{step_id}"))
    }
}
