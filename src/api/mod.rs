/// SyncThing API client implementation.
pub mod client;
/// Unit tests for configuration and error management.
#[cfg(test)]
pub mod config_error_tests;
/// Unit tests for configuration and error management client methods.
#[cfg(test)]
pub mod config_sync_error_client_tests;
/// Unit tests for synchronization operation models.
#[cfg(test)]
pub mod sync_ops_models_tests;
/// Unit tests for event parsing and summaries.
#[cfg(test)]
pub mod event_tests;
/// Unit tests for health checks.
#[cfg(test)]
pub mod health_tests;
/// Data models for SyncThing API responses and configurations.
pub mod models;
/// Unit tests for the API client.
#[cfg(test)]
pub mod tests;

pub use client::SyncThingClient;
pub use models::*;
