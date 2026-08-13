//! monokl's core symbol-representation types.
//!
//! Per SPEC-003 (`.claude/specs/SPEC-003.json`), `SymbolKind`, `Visibility`,
//! and `SymbolEntry` are the canonical, locked shapes of monokl's
//! symbol-classification and per-symbol output types, verbatim per
//! `docs/spec/01-core-architecture.md:263-310`.
use serde::{Deserialize, Serialize};

/// monokl's universal symbol-classification enum (AC-001, AC-002). 12
/// universal variants usable by any language analyzer, 2 Rust-specific
/// variants (`Impl`, `Macro`), then 1 `Other` catch-all -- 15 total, in
/// this declaration order. No `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SymbolKind {
    Function,
    Method,
    Constructor,
    Class,
    Struct,
    Enum,
    Interface,
    TypeAlias,
    Property,
    Field,
    Variable,
    Module,
    Impl,
    Macro,
    Other,
}

/// monokl's symbol-visibility enum (AC-003). Exactly 4 variants -- no
/// `Super` variant exists. `docs/spec/04-analysis-fidelity.md`'s
/// `rust_visibility(node)` (line 62) describes an implementation that, if
/// written as literally specified there (mapping `pub`, `pub(crate)`,
/// `pub(super)`, `pub(self)` to `Visibility::{Public, Crate, Super,
/// Module}`, a fifth identifier with no corresponding variant here), would
/// fail to compile against this enum -- a conditional claim about code
/// that does not exist yet, not an assertion that the prose is presently
/// broken. Any Rust code actually written against this contract must map
/// `pub(super)` (and `pub(in ...)`) to `Visibility::Module`, per
/// `docs/spec/02-inspection-and-analysis.md:1727-1729`'s
/// `parse_visibility`, which predates `rust_visibility` and already
/// handles this case correctly. See
/// `docs/spec/07-edge-cases-and-failure-modes.md` Part 4 finding #2 (line
/// 139) (AC-008).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Visibility {
    Public,
    Crate,
    Module,
    Private,
}
