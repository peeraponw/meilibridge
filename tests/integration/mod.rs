// Integration tests for MeiliBridge
// These tests require external services (PostgreSQL, Meilisearch, Redis)

pub mod api;
pub mod common;
pub mod meilisearch;
pub mod postgres_cdc;
pub mod redis;
