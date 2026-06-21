#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub log_level: String,
    pub database: DatabaseConfig,
    pub search_low_confidence_min_results: usize,
    pub auth: AuthConfig,
    pub worker: WorkerConfig,
}

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub min_pool_size: usize,
    pub max_pool_size: usize,
}

#[derive(Debug, Clone, Default)]
pub struct AuthConfig {
    pub admin_token: Option<String>,
    pub worker_token: Option<String>,
    pub mcp_token: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub worker_id: String,
    pub batch_size: i64,
    pub poll_interval_ms: u64,
    pub http_timeout_ms: u64,
    pub max_body_bytes: usize,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            host: env_or("RESOURCE_SERVICE_API_HOST", "127.0.0.1"),
            port: env_or("RESOURCE_SERVICE_API_PORT", "3200")
                .parse()
                .unwrap_or(3200),
            log_level: env_or("LOG_LEVEL", "info"),
            database: DatabaseConfig {
                host: env_or("RESOURCE_SERVICE_HOST", "127.0.0.1"),
                port: env_or("RESOURCE_SERVICE_PORT", "5433")
                    .parse()
                    .unwrap_or(5433),
                name: env_or("RESOURCE_DB", "postgres"),
                user: env_or("RESOURCE_USER", "postgres"),
                password: env_or("RESOURCE_PASS", "1234"),
                min_pool_size: env_or("RESOURCE_POSTGRES_MIN_CONNECTIONS", "1")
                    .parse()
                    .unwrap_or(1),
                max_pool_size: env_or("RESOURCE_POSTGRES_MAX_CONNECTIONS", "15")
                    .parse()
                    .unwrap_or(15),
            },
            search_low_confidence_min_results: env_or("RESOURCE_LOW_CONFIDENCE_MIN_RESULTS", "3")
                .parse()
                .unwrap_or(3),
            auth: AuthConfig {
                admin_token: env_optional("RESOURCE_SERVICE_ADMIN_TOKEN"),
                worker_token: env_optional("RESOURCE_SERVICE_WORKER_TOKEN"),
                mcp_token: env_optional("RESOURCE_SERVICE_MCP_TOKEN"),
            },
            worker: WorkerConfig {
                worker_id: env_or("RESOURCE_WORKER_ID", "resource_worker_local"),
                batch_size: env_or("RESOURCE_WORKER_BATCH_SIZE", "10")
                    .parse()
                    .unwrap_or(10),
                poll_interval_ms: env_or("RESOURCE_WORKER_POLL_INTERVAL_MS", "5000")
                    .parse()
                    .unwrap_or(5000),
                http_timeout_ms: env_or("RESOURCE_WORKER_HTTP_TIMEOUT_MS", "15000")
                    .parse()
                    .unwrap_or(15000),
                max_body_bytes: env_or("RESOURCE_WORKER_MAX_BODY_BYTES", "2097152")
                    .parse()
                    .unwrap_or(2 * 1024 * 1024),
            },
        }
    }
}

fn env_or(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn env_optional(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_usable_for_local_resource_postgres() {
        let config = AppConfig::from_env();

        assert!(!config.database.host.is_empty());
        assert!(config.database.port > 0);
        assert!(config.database.max_pool_size >= config.database.min_pool_size);
    }
}
