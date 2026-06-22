use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialSeedManifest {
    pub seeds: Vec<OfficialSeed>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialSeed {
    pub seed_id: String,
    pub topic_ids: Vec<String>,
    pub source_id: String,
    pub url: String,
    pub seed_type: String,
    pub max_depth: i32,
    pub priority: i32,
    pub enabled: bool,
    pub expected_resource_kind: String,
    pub notes: String,
}
