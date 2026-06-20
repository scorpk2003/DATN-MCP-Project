mod crawl;
mod embedding;
pub(crate) mod enrichment;
mod fetch_artifacts;
mod health_check;
mod mappers;
mod recommendation;
mod research;
mod resources;
mod scheduler;
mod schema;
mod search;
mod sources;

use deadpool_postgres::Pool;

pub use search::{coverage_for_results, normalize_query};

#[derive(Clone)]
pub struct ResourceRepository {
    pub(crate) pool: Pool,
}

impl ResourceRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}
