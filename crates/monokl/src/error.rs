//! monokl's single unified error type and crate-wide `Result` alias.
//!
//! Per SPEC-001 (`.claude/specs/SPEC-001.json`), `MonoklError` is the crate's
//! sole error enum: every fallible `pub`/`pub(crate)` function in this crate
//! returns `crate::error::Result<T>` for its own `T`, with exactly one named
//! exception: `git_scope::blob_at_ref_in_repo`, a `pub(crate)` helper that
//! returns a bare `std::result::Result<String, String>` because it is always
//! mapped into `MonoklError::Git { operation: "show", .. }` at its sole call
//! site, `git_scope::blob_at_ref` (AC-003). `git_scope`, `persist`, and
//! `io_safety` are separate, not-yet-implemented tracks; this module defines
//! only the error contract they must conform to, not their bodies.
use camino::Utf8PathBuf;

#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "miette", derive(miette::Diagnostic))]
#[non_exhaustive]
pub enum MonoklError {
    // BLOCKED (AC-004, AC-001): the `Io(#[from] FileIoError)` variant this
    // spec requires cannot be added yet. FileIoError is owned by the
    // `io-errors` crate, which does not exist on disk at `../io-errors`
    // even though crates/monokl/Cargo.toml:43 already declares a
    // mandatory, non-optional dependency on it; that same dependency line
    // also pins `registry = "orin-cargo"`, and no `.cargo/config.toml`
    // `[registries.orin-cargo]` entry exists anywhere in this workspace
    // (the same AC-008B gap SPEC-004/PLAN-004 already documented). Until
    // both (a) an `io-errors` crate exists at `../io-errors` defining
    // `FileIoError`, and (b) `.cargo/config.toml` configures the
    // `orin-cargo` registry (or the dependency is repointed at a
    // resolvable source), this variant cannot be written without either
    // inventing an undocumented stub type or leaving the crate permanently
    // unbuildable. See this plan's blocker task (T-009) for the exact
    // decision needed.
    /// AC-005: three construction sites in the spec corpus --
    /// `text_search::search_files`, `WorkspaceIndex::build`'s walk, and
    /// `source_scan`'s per-directory walk -- in all three, `path` is the
    /// root of the walk in progress, not the failing entry.
    #[error("walker error at {path}")]
    Walk {
        path: Utf8PathBuf,
        #[source]
        source: ignore::Error,
    },

    /// AC-006.
    #[error("regex compilation failed")]
    RegexBuild(#[from] grep_regex::Error),

    /// AC-007: `path` is deliberately `std::path::PathBuf`, not
    /// `Utf8PathBuf` -- the entire point of this variant is that the path
    /// is not valid UTF-8, so it cannot be represented as camino's
    /// UTF-8-guaranteed type.
    #[error("non-UTF-8 path encountered: {path:?}")]
    NonUtf8Path { path: std::path::PathBuf },

    /// AC-008.
    #[error("query has too many terms: {count} > {limit}")]
    TooManyTerms { count: usize, limit: usize },

    /// AC-009: constructed at
    /// `tiktoken_rs::o200k_base().map_err(|_| MonoklError::TokenizerInit)?`.
    #[error("tokenizer init failed")]
    TokenizerInit,
}
