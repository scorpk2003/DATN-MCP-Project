use deadpool_postgres::{Config, ManagerConfig, Pool, RecyclingMethod, Runtime};
use tokio_postgres::NoTls;

use crate::{AppConfig, AppResult};

pub fn create_pool(config: &AppConfig) -> AppResult<Pool> {
    let mut pool_config = Config::new();
    pool_config.host = Some(config.database.host.clone());
    pool_config.port = Some(config.database.port);
    pool_config.dbname = Some(config.database.name.clone());
    pool_config.user = Some(config.database.user.clone());
    pool_config.password = Some(config.database.password.clone());
    pool_config.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    pool_config.pool = Some(deadpool_postgres::PoolConfig {
        max_size: config.database.max_pool_size,
        ..Default::default()
    });

    Ok(pool_config.create_pool(Some(Runtime::Tokio1), NoTls)?)
}
