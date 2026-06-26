pub mod config;
pub mod server;

pub use server::LessonServer;

#[cfg(test)]
mod mcp_client_tests;
