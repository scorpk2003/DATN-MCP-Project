use std::{collections::HashSet, env};

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct ServerConfig {
    pub name: String,
    pub host: String,
    pub port: usize,
    pub description: String,
    pub url: String,
    pub required: bool,
    pub aliases: Vec<String>,
}

impl ServerConfig {
    pub fn new(server: &str, required: bool) -> Self {
        let key = server.to_uppercase();
        let aliases = legacy_aliases(server);
        let name = env_value(&key, "NAME", &aliases).unwrap_or_else(|_| server.to_string());
        let host = env_value(&key, "HOST", &aliases).unwrap_or_else(|_| "127.0.0.1".to_string());
        let port = env_value(&key, "PORT", &aliases)
            .unwrap_or_else(|_| "3001".to_string())
            .parse::<usize>()
            .unwrap_or(3001);
        let description = env_value(&key, "DESCRIPTION", &aliases)
            .unwrap_or_else(|_| "no description".to_string());
        let url = env_value(&key, "URL", &aliases)
            .unwrap_or_else(|_| format!("http://{host}:{port}/mcp"));
        Self {
            name,
            host,
            port,
            description,
            url,
            required,
            aliases,
        }
    }

    pub fn required_servers_from_env() -> Vec<Self> {
        parse_server_list(
            "ORCHESTRATOR_MCP_SERVERS",
            "database,resource,roadmap,lesson",
        )
        .into_iter()
        .map(|name| Self::new(&name, true))
        .collect()
    }

    pub fn optional_servers_from_env() -> Vec<Self> {
        parse_server_list("ORCHESTRATOR_OPTIONAL_MCP_SERVERS", "")
            .into_iter()
            .map(|name| Self::new(&name, false))
            .collect()
    }

    pub fn all_from_env() -> Vec<Self> {
        let mut seen = HashSet::new();
        Self::required_servers_from_env()
            .into_iter()
            .chain(Self::optional_servers_from_env())
            .filter(|server| seen.insert(server.name.clone()))
            .collect()
    }
}

fn parse_server_list(env_key: &str, default: &str) -> Vec<String> {
    env::var(env_key)
        .unwrap_or_else(|_| default.to_string())
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase())
        .collect()
}

fn env_value(server_key: &str, field: &str, aliases: &[String]) -> Result<String, env::VarError> {
    let canonical = format!("SERVER_{}_{}", server_key, field);
    env::var(canonical).or_else(|_| {
        aliases
            .iter()
            .find_map(|alias| env::var(format!("SERVER_{}_{}", alias, field)).ok())
            .ok_or(env::VarError::NotPresent)
    })
}

fn legacy_aliases(server: &str) -> Vec<String> {
    match server {
        "resource" => vec!["RESOURCE_MCP".to_string()],
        _ => Vec::new(),
    }
}
