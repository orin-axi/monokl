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

    /// AC-011: 8 raw, non-test call sites in 3 categories -- cache-deserialize
    /// (1 site, `persist::load_cache`), cache-serialize (3 sites, all in
    /// `persist.rs`), CLI/output-serialize (4 sites: `output::render_json`,
    /// `output::render_json_compact`, `projection::project_inspect_result`,
    /// `projection::project_search_response`). Display text corrected from
    /// "json serialization failed" to the text below, since the
    /// cache-deserialize site is a parse failure the old text
    /// mischaracterized as a serialization failure.
    #[error("json parsing or serialization failed")]
    Json(#[from] serde_json::Error),

    /// AC-010: fired by the `dependents` command when the canonicalized
    /// target file does not start with the canonicalized root argument.
    #[error("path outside workspace root: {path}")]
    PathOutsideRoot { path: Utf8PathBuf },

    /// AC-013: returned by `persist::load_cache` when
    /// `cache.version != CACHE_VERSION || cache.config_hash != config_hash`.
    /// `load_cache` has exactly two callers: `persist::init`, whose match
    /// arm intercepts `Err(MonoklError::StaleDiskCache)` only (logs
    /// AC-020B's `tracing::warn!` text below and substitutes a fresh empty
    /// `CacheFile`, never returning `StaleDiskCache` to its own caller) --
    /// every other `Err(e)`, including `Json` and `Io`, propagates
    /// unmodified; and AC-013B: `persist::flush`'s
    /// `state.cache_file.is_none()` fallback branch, which calls
    /// `load_cache(..)?` directly with no match arm and no interception of
    /// any kind -- any `Err`, including `StaleDiskCache` or `Json`,
    /// propagates unmodified to `flush`'s own caller. In every call chain
    /// shown in the spec corpus, `flush`'s sole caller
    /// (`WorkspaceIndex::build`) calls `persist::init(..)?` first, so this
    /// fallback branch is not exercised today -- but `flush` is
    /// `pub(crate)`, not gated behind `init` at the type level, so nothing
    /// in the source prevents a future in-crate caller from invoking it
    /// first. This asymmetry is documented, not resolved here (non_goals):
    /// fixing it means editing `persist.rs`'s verbatim `flush()` body, out
    /// of scope for this error-contract track.
    ///
    /// AC-020B: `persist::init`'s interception arm additionally logs, on
    /// the self-heal path only (never returned to any caller), this exact
    /// text: "on-disk cache is stale (version or config_hash mismatch); starting fresh".
    #[error("disk cache is stale (version or config hash mismatch)")]
    StaleDiskCache,

    /// AC-012: sole construction site `git_scope::validate_git_ref`.
    /// Exactly three `reason` literals are constructed anywhere in the spec
    /// corpus: "ref is empty" (empty input), "starts with '-' — refusing
    /// as it could be parsed as a git option" (git-option-injection guard),
    /// and "contains characters outside the safe set" (any char outside
    /// A-Za-z0-9_/.@{}^~:+-).
    #[error("invalid git ref {ref_:?}: {reason}")]
    InvalidGitRef { ref_: String, reason: &'static str },

    /// AC-015: exactly 2 of `git_scope.rs`'s construction sites spell out a
    /// literal `operation` value in the spec corpus --
    /// `effective_pr_range`'s merge_base call (`operation: "merge-base"`)
    /// and `blob_at_ref` (`operation: "show"`). `collect_changes` contains
    /// 3 additional fallible-git-operation sites whose `map_err` closures
    /// are elided in the source as shown; their operation literals (and,
    /// for 2 of the 3, even whether the constructed variant is `Git` at
    /// all) are undetermined by the spec corpus and are not invented here
    /// (non_goals).
    #[error("git {operation} failed: {message}")]
    Git {
        operation: &'static str,
        message: String,
    },

    /// AC-016: sole construction site `io_safety::read_to_string_capped`,
    /// fired when `symlink_metadata(path).file_type().is_symlink()` is
    /// true, checked before the size check (see `FileTooLarge`).
    /// `read_to_string_capped` has no call site anywhere in the spec
    /// corpus despite prose claiming it guards
    /// search/inspect/extract/refs/definition -- refuted by
    /// `07-edge-cases-and-failure-modes.md` Part 3 findings #2 and #16,
    /// which show `extract` and the refs/definition path use raw
    /// `fs::read_to_string` instead. This variant is therefore currently
    /// unconstructible via any shown command path; wiring
    /// `read_to_string_capped` in is `io_safety.rs` implementation work,
    /// out of scope here (non_goals).
    #[error("refusing to read symlink: {path}")]
    SymlinkRejected { path: Utf8PathBuf },

    /// AC-017: same sole construction site as `SymlinkRejected`, fired when
    /// `meta.len() > MAX_INSPECTABLE_FILE_SIZE` (50 * 1024 * 1024 =
    /// 52,428,800 bytes). Shares `SymlinkRejected`'s total unreachability
    /// in the current corpus.
    #[error("file too large: {path} is {size} bytes, cap is {cap}")]
    FileTooLarge {
        path: Utf8PathBuf,
        size: u64,
        cap: u64,
    },

    /// AC-018: deliberately NOT auto-recovered via
    /// `PoisonError::into_inner()` -- recovering would launder possibly-torn
    /// shared state back into use. Two confirmed construction sites:
    /// `analysis::persist::lock_state()`
    /// (`context: "analysis::persist::CacheState"`) and
    /// `pipeline::session::WorkspaceSession::standard_index`
    /// (`context: "pipeline::session::WorkspaceSession::standard_index"`).
    #[error("internal lock poisoned in {context} — a prior panic left shared state possibly inconsistent; this is a bug, please report")]
    LockPoisoned { context: &'static str },
}

/// AC-019: crate-wide alias, immediately after the enum definition.
///
/// AC-003: every `pub`/`pub(crate)` function in the spec corpus's Part 1
/// and Part 3 verbatim code blocks whose own signature returns some
/// `Result<_, _>` returns exactly this alias for its own `T`, with exactly
/// one named exception: `git_scope::blob_at_ref_in_repo(repo, git_ref,
/// path) -> std::result::Result<String, String>`, a `pub(crate)` helper
/// whose bare-`String` error is mapped into `MonoklError::Git { operation:
/// "show", .. }` at its sole call site, `git_scope::blob_at_ref`.
///
/// AC-021: within the scope of variants and call sites this spec resolves,
/// `MonoklError`'s default posture is propagate-to-caller via `?` (or an
/// explicit `map_err`/`Into::into` at the construction boundary, for the
/// AC-003 exception). The sole documented exception is `StaleDiskCache`,
/// and only when `load_cache` is reached via `persist::init` specifically
/// -- not via `persist::flush`'s fallback, where the identical variant
/// propagates uncaught (AC-013B).
pub type Result<T> = std::result::Result<T, MonoklError>;
