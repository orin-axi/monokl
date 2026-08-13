// Test modules use `.unwrap()` / `.expect()` liberally — conventional in tests because
// a failing assertion should panic, and the workspace-level `deny unwrap_used` / `expect_used`
// lint is intended for production code only. Scope the allow to `#[cfg(test)]` so prod code
// stays covered.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod analysis;
pub mod budget;
#[cfg(feature = "cli")]
pub mod cli;
pub mod dedup;
#[cfg(feature = "lang-ts")]
pub mod enrich;
pub mod error;
#[cfg(feature = "lang-ts")]
pub mod indices;
#[cfg(feature = "cli")]
pub mod output;
#[cfg(feature = "lang-ts")]
pub mod pipeline;
pub mod query;
pub mod rank;
pub mod text_search;
pub mod tokens;
pub mod types;

#[cfg(test)]
mod tests_types;
