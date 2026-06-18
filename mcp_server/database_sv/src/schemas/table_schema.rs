use rmcp::schemars::{self, JsonSchema};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetTableSchema {
    #[schemars(description = "table name to retrieve schema for")]
    pub table_name: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
pub struct GetMultiTableSchema {
    #[schemars(description = "list table names to retrieve schemas for")]
    pub table_names: Vec<String>,
}
