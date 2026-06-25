// v0.4.11 PR3: Data Transfer Objects shared between backend API handlers
// and frontend consumers. Types live in relay_shared::models; this module
// re-exports them so handlers can write `crate::dto::X` uniformly.

pub use relay_shared::models::{SharedGroupSummary, SharedNodeSummary};
