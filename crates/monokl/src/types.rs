//! monokl's core symbol-representation types.
//!
//! Per SPEC-003 (`.claude/specs/SPEC-003.json`), `SymbolKind`, `Visibility`,
//! and `SymbolEntry` are the canonical, locked shapes of monokl's
//! symbol-classification and per-symbol output types, verbatim per
//! `docs/spec/01-core-architecture.md:263-310`.
use camino::Utf8PathBuf;
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

/// The resolved target of a dependency statement (AC-005, AC-006, AC-007).
/// Internally tagged (`tag = "kind"`): each variant's own fields flatten
/// directly into the same JSON object as the "kind" tag key, not nested
/// under a separate variant-name key. The enum-level
/// `#[serde(rename_all = "camelCase")]` renames only the 3 variant tag
/// values (File->"file", RustPath->"rustPath", Namespace->"namespace") --
/// it does NOT rename these struct variants' own fields, since that
/// requires the separate `rename_all_fields` attribute, which this enum
/// does not carry: the wire keys stay snake_case (is_relative, is_static),
/// not isRelative/isStatic. No `#[serde(other)]` fallback; an unrecognized
/// "kind" value fails with an unknown-variant error. `#[non_exhaustive]`
/// here only affects Rust-level exhaustive match/construct checking outside
/// this crate -- it has no effect on serde's generated (de)serialize code.
///
/// `Namespace` is confirmed spec residue from a dropped C#-roadmap item
/// (docs/spec/07-edge-cases-and-failure-modes.md Part 3 finding #20;
/// docs/spec/05-research-and-decisions.md §6 records C# dropped from the
/// bespoke Go/Java-tier roadmap, while §10 separately adopts a not-yet-built
/// generic tree-sitter fallback tier whose launch set includes C#). No
/// analyzer in the current verbatim spec code constructs
/// DependencyTarget::Namespace -- it is cosmetic residue in today's code,
/// not a functional defect: the variant compiles and round-trips through
/// serde normally.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
#[non_exhaustive]
pub enum DependencyTarget {
    File {
        specifier: String,
        resolved: Option<Utf8PathBuf>,
        is_relative: bool,
    },
    RustPath {
        segments: Vec<String>,
        anchor: RustPathAnchor,
        resolved: Option<Utf8PathBuf>,
    },
    Namespace {
        segments: Vec<String>,
        is_static: bool,
        alias: Option<String>,
    },
}

/// A single named/local/kind binding within a `DependencyRecord` (AC-002).
/// Exactly 3 fields in this declaration order, all required on deserialize
/// -- no field-level attributes. Derives `Debug`, `Clone`, `Serialize`,
/// `Deserialize` only; no `Copy`/`PartialEq`/`Eq`, no `#[non_exhaustive]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyBinding {
    pub imported: String,
    pub local: String,
    pub kind: BindingKind,
}

/// monokl's per-dependency-statement output record (AC-001). Exactly 3
/// fields in this declaration order. Derives `Debug`, `Clone`,
/// `Serialize`, `Deserialize` only -- no `Copy`/`PartialEq`/`Eq`, no
/// `#[non_exhaustive]`. Only `bindings` carries a field-level attribute:
/// `#[serde(default)]` with no `skip_serializing_if`, so an empty
/// `bindings` vec still serializes its key ("bindings":[]) while a missing
/// "bindings" key still deserializes leniently to an empty vec -- this is
/// load-bearing, unlike an `Option<T>` field's `#[serde(default)]`: a
/// `Vec<T>` field gets no implicit missing-key deserialize leniency from
/// serde's derive macro on its own. `line` and `target` carry no
/// field-level attributes and are required.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyRecord {
    pub line: usize,
    #[serde(default)]
    pub bindings: Vec<DependencyBinding>,
    pub target: DependencyTarget,
}

/// A single named export statement (AC-009). Exactly 3 fields in this
/// declaration order, all required on deserialize -- no field-level
/// attributes. Under rename_all = "camelCase", re_export's JSON key is
/// reExport.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportRecord {
    pub name: String,
    pub line: usize,
    pub re_export: bool,
}

/// A single JSX attribute (AC-010). Exactly 4 fields in this declaration
/// order. name/is_expression/is_spread are required; string_value carries
/// no serde attribute at all yet still deserializes leniently to None on a
/// missing or explicitly-null key (Option<T>'s own implicit leniency), and
/// serializes its key with a JSON null when None (no skip_serializing_if to
/// omit it).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsxAttribute {
    pub name: String,
    pub string_value: Option<String>,
    pub is_expression: bool,
    pub is_spread: bool,
}

/// A single JSX element usage (AC-011). Exactly 4 fields in this
/// declaration order. Unlike DependencyRecord.bindings, attributes carries
/// no #[serde(default)]: Vec<T> gets no implicit missing-key leniency from
/// serde's derive macro, so all 4 fields including attributes are required
/// on deserialize.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JsxElementEntry {
    pub name: String,
    pub is_html: bool,
    pub line: usize,
    pub attributes: Vec<JsxAttribute>,
}

/// Empty per-language placeholder (AC-018) -- populated once the Rust
/// analyzer lands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RustData {}

/// Empty per-language placeholder (AC-018) -- populated once the Python
/// analyzer lands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PythonData {}

/// Empty per-language placeholder (AC-018, AC-015) -- replaces the dropped
/// CSharp variant; populated once the Go analyzer lands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GoData {}

/// Empty per-language placeholder (AC-018, AC-015) -- replaces the dropped
/// CSharp variant; populated once the Java analyzer lands.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaData {}

