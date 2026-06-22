pub mod api;
pub mod chunker;
pub mod config;
pub mod corpus;
pub mod db;
pub mod embedding_provider;
pub mod error;
pub mod extractor;
pub mod models;
pub mod repository;
pub mod service;
pub mod worker;

pub use config::AppConfig;
pub use db::create_pool;
pub use error::{AppError, AppResult};
pub use service::ResourceService;
