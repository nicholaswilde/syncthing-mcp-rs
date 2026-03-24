/// SyncThing API client implementation.
pub mod client;
/// Unit tests for event parsing and summaries.
#[cfg(test)]
pub mod event_tests;
/// Data models for SyncThing API responses and configurations.
pub mod models;
/// Unit tests for health checks.
#[cfg(test)]
pub mod health_tests;
/// Unit tests for the API client.
#[cfg(test)]
pub mod tests;

pub use client::SyncThingClient;
pub use models::*;
