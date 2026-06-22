use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialSourceCatalog {
    pub sources: Vec<OfficialSource>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OfficialSource {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub base_url: String,
    pub trust_tier: i16,
    pub language_hint: String,
    pub is_official: bool,
    pub enabled: bool,
    pub allowed_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub default_resource_kind: String,
    pub crawl_policy: CrawlPolicy,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrawlPolicy {
    pub max_depth_default: i32,
    pub rate_limit_per_minute: i32,
    pub respect_robots_txt: bool,
    pub user_agent: String,
}
