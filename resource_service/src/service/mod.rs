mod admin;
mod embedding;
mod enrichment;
mod pipeline;
mod research;
mod resources;
mod search;
mod source_crawl;
mod validation;

use deadpool_postgres::Pool;

use crate::{AppConfig, repository::ResourceRepository};

#[derive(Clone)]
pub struct ResourceService {
    pub(crate) config: AppConfig,
    pub(crate) repository: ResourceRepository,
}

impl ResourceService {
    pub fn new(pool: Pool, config: AppConfig) -> Self {
        Self {
            config,
            repository: ResourceRepository::new(pool),
        }
    }
}
