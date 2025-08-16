pub mod adapter;
pub mod batch_processor;
pub mod client;
pub mod protected_client;

pub use adapter::MeilisearchAdapter;
pub use client::MeilisearchClient;
pub use protected_client::ProtectedMeilisearchClient;
