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

/// monokl's per-symbol output record (AC-004, AC-005, AC-006, AC-010,
/// AC-011, AC-012). Exactly 8 fields in this declaration order; derives
/// `Debug`, `Clone`, `Serialize`, `Deserialize` only -- notably not
/// `Copy`, not `PartialEq`, not `Eq` (unlike `SymbolKind` and
/// `Visibility`), and no `#[non_exhaustive]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolEntry {
    pub name: String,
    pub kind: SymbolKind,
    pub line: usize,
    pub signature: Option<String>,
    /// The sole canonical spelling of this field is `owner`, never
    /// `impl_owner` (AC-007). `docs/spec/04-analysis-fidelity.md`
    /// describes an implementation that, if written as literally
    /// specified there (assigning to a field named `impl_owner`), would
    /// fail to compile against this struct -- a conditional claim about
    /// code that does not exist yet, not an assertion that the prose is
    /// presently broken. `04-analysis-fidelity.md` currently uses
    /// `impl_owner` seven times (lines 3, 19, 72, 77, 95, 100, 101) with
    /// zero bare occurrences of `owner`. See
    /// `docs/spec/07-edge-cases-and-failure-modes.md` Part 4 finding #1
    /// (line 138). Any Rust code actually written against this contract
    /// must use `owner`; `impl_owner` is not a legal field name on
    /// `SymbolEntry`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trait_impl: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind_detail: Option<String>,
}

/// The kind of a single import binding within a `DependencyBinding` (AC-003,
/// AC-004). Exactly 5 variants in this declaration order. `Copy`/`PartialEq`/
/// `Eq` present (fieldless enum, same pattern as `SymbolKind`/`Visibility`).
/// No `#[non_exhaustive]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BindingKind {
    Named,
    Default,
    Namespace,
    Glob,
    NamespaceWide,
}

/// The anchor of a resolved Rust module path (AC-008). No `Copy`/`PartialEq`/
/// `Eq` (`Extern(String)` carries a payload). `#[non_exhaustive]`. Externally
/// tagged by default (no `tag`/`content` attribute on this enum) --
/// `Extern`'s associated `String` is carried as the tagged value's content,
/// not flattened as a named field. `Crate`/`Super`/`Extern` receive the
/// default camelCase transform with no override, producing tag values
/// "crate"/"super"/"extern". `Selff` alone carries an explicit
/// `#[serde(rename = "self")]` -- functionally necessary, not stylistic:
/// `self` is a reserved keyword and cannot name an enum variant, so without
/// this rename the default transform of the identifier `Selff` would
/// produce "selff", not "self".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum RustPathAnchor {
    Crate,
    Super,
    #[serde(rename = "self")]
    Selff,
    Extern(String),
}

