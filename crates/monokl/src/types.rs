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

/// TypeScript-specific analysis output (AC-017) -- the only one of the 5
/// per-language data structs with real fields. Of its 3 Vec fields, all
/// carry #[serde(default)] (load-bearing missing-key leniency, per
/// DependencyRecord.bindings), but only unresolved_aliases also carries
/// skip_serializing_if = "Vec::is_empty": jsx_elements and
/// type_only_imports always serialize their key even when empty.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsData {
    #[serde(default)]
    pub jsx_elements: Vec<JsxElementEntry>,
    #[serde(default)]
    pub type_only_imports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_aliases: Vec<String>,
}

/// Post-research correction (AC-015): Go/Java replace an earlier CSharp
/// variant -- the language roadmap dropped C# and added Go and Java
/// instead (docs/spec/05-research-and-decisions.md §6). GoData/JavaData
/// are empty placeholders today, same as PythonData -- populated once
/// those analyzers land. LangData's own #[non_exhaustive] protects only
/// enum-level exhaustive-match compatibility; it does not extend to
/// GoData/JavaData's own fields -- adding real fields to those structs
/// later is a separate, unresolved compatibility question.
///
/// Per-language analysis output (AC-012, AC-013, AC-014). Adjacently
/// tagged (tag = "language", content = "data"): the tag and the variant's
/// payload stay in two separate keys of the same object (e.g. Ts
/// serializes as {"language":"typescript","data":{...}}) -- a
/// structurally different JSON shape from DependencyTarget's internal
/// tagging, which flattens variant fields alongside the tag key with no
/// separate content wrapper. Each variant carries its own explicit
/// #[serde(rename = ...)] tag value; only Ts's rename to "typescript" is
/// functionally necessary (the default transform would otherwise give
/// "ts") -- Rust/Python/Go/Java's renames are redundant with what the
/// default transform already produces. No #[serde(other)] fallback; an
/// unrecognized "language" value fails with an unknown-variant error.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "language", content = "data")]
#[non_exhaustive]
pub enum LangData {
    #[serde(rename = "typescript")]
    Ts(TsData),
    #[serde(rename = "rust")]
    Rust(RustData),
    #[serde(rename = "python")]
    Python(PythonData),
    #[serde(rename = "go")]
    Go(GoData),
    #[serde(rename = "java")]
    Java(JavaData),
}

impl LangData {
    /// Returns `Some(&TsData)` iff `self` is the `Ts` variant, `None`
    /// otherwise. Never panics (AC-016).
    pub fn ts(&self) -> Option<&TsData> {
        if let LangData::Ts(ts) = self { Some(ts) } else { None }
    }

    /// Delegates to `ts()`: `TsData.jsx_elements` as a slice when `self` is
    /// `Ts`, an empty slice otherwise. Never panics (AC-016).
    pub fn jsx_elements(&self) -> &[JsxElementEntry] {
        self.ts().map_or(&[], |ts| ts.jsx_elements.as_slice())
    }
}


/// monokl's per-match result record from the ranking pipeline (AC-001,
/// AC-002, AC-003, AC-004). Exactly 8 fields in this declaration order.
/// Derives `Debug`, `Clone`, `Serialize`, `Deserialize` only -- no
/// `Copy`/`PartialEq`/`Eq`, no `#[non_exhaustive]`. `node_kind: SymbolKind`
/// references the enum locked in SPEC-003; this track does not re-lock
/// SymbolKind's own shape. `symbol_signature` carries no field-level
/// attribute -- its leniency comes from `Option<T>`'s own `Deserialize`
/// impl, and it always emits a JSON null key when `None`.
/// `matched_lines`/`matched_keywords` each carry `#[serde(default,
/// skip_serializing_if = "Vec::is_empty")]`. Neither line_start <=
/// line_end nor the matched_lines range is enforced by this struct's own
/// Deserialize impl (AC-004) -- both are structurally possible to
/// violate. The overlaps_significantly comment
/// (01-core-architecture.md:1217-1219) already flags this gap in prose
/// and defends against it internally via saturating_sub rather than a
/// type-level guarantee.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeBlock {
    pub file: Utf8PathBuf,
    pub line_start: usize,
    pub line_end: usize,
    pub node_kind: SymbolKind,
    pub code: String,
    pub symbol_signature: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_lines: Vec<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub matched_keywords: Vec<String>,
}

/// A symbol's enclosing context at the point a `CodeBlock` match occurred
/// (AC-009). Exactly 3 fields in this declaration order. Derives `Debug`,
/// `Clone`, `Serialize` only -- no `Deserialize`, no `Copy`/`PartialEq`/
/// `Eq`. Like `RankedBlock`, this type can never be parsed from JSON: it
/// has no deserialize-side contract of any kind, so no missing-field or
/// leniency framing applies to it the way it does to `CodeBlock`. No
/// field carries a field-level attribute; all 3 are unconditionally
/// present in serialize output. `kind: SymbolKind` references the enum
/// locked in SPEC-003.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentContext {
    pub kind: SymbolKind,
    pub name: String,
    pub line: usize,
}

/// A `CodeBlock` after ranking-pipeline scoring (AC-005, AC-006, AC-007,
/// AC-008). Derives `Debug`, `Clone`, `Serialize` only -- no
/// `Deserialize`, no `Copy`/`PartialEq`/`Eq`: this type can never be
/// parsed from JSON. Exactly 7 fields in this declaration order. `block:
/// CodeBlock` carries `#[serde(flatten)]`, merging CodeBlock's own JSON
/// keys directly into this struct's serialized object, with no
/// "block" wrapper key. None of this struct's own 6 field names
/// collides with any of CodeBlock's 8 field names (verified in
/// tests_types.rs). Flatten does not alter CodeBlock's own field-level
/// serialize behavior. The four f64 scoring fields (bm25_score,
/// coverage_boost, node_type_boost, final_score) carry no field-level
/// attribute and are always present; serde_json serializes a non-finite
/// f64 (NAN, INFINITY, NEG_INFINITY) as JSON null rather than failing.
/// `parent_context` carries no field-level attribute -- `#[serde(flatten)]`
/// on `block` is the sole field-level attribute anywhere on this struct --
/// so a None here still emits its "parentContext" key with a JSON null
/// value.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedBlock {
    #[serde(flatten)]
    pub block: CodeBlock,
    pub bm25_score: f64,
    pub coverage_boost: f64,
    pub node_type_boost: f64,
    pub final_score: f64,
    pub rank: usize,
    pub parent_context: Option<ParentContext>,
}

/// The kind of a single diagnostic emitted by monokl's ranking pipeline
/// or a command's own analysis (AC-012, AC-013, AC-014). Exactly 3
/// variants in this declaration order. Derives `Debug`, `Clone`, `Copy`,
/// `PartialEq`, `Eq`, `Serialize`, `Deserialize` -- the same derive list
/// as `SymbolKind`/`Visibility`/`BindingKind` (fieldless enum). No
/// `#[non_exhaustive]`, no `#[serde(other)]` fallback. Carries
/// `#[serde(rename_all = "lowercase")]` -- "lowercase", not "camelCase",
/// the rule used by every other enum/struct locked so far in this
/// project. "lowercase" lowercases a variant name's every character
/// uniformly, with no word-boundary handling of any kind -- a
/// categorically different transform from "camelCase". For this enum's 3
/// actual single-word variants, "lowercase" and "camelCase" happen to
/// produce identical output (degraded/skipped/warning) -- a coincidence
/// of these variant names, not evidence the two rules are
/// interchangeable in general.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticKind {
    Degraded,
    Skipped,
    Warning,
}

/// monokl's diagnostic record (AC-010, AC-011). Exactly 3 fields in this
/// declaration order. Derives `Debug`, `Clone`, `Serialize`,
/// `Deserialize` only -- no `Copy`/`PartialEq`/`Eq`, no
/// `#[non_exhaustive]`. `kind` and `message` carry no field-level
/// attribute and are required on deserialize. `path` also carries no
/// field-level attribute; as an `Option<T>` field it deserializes
/// leniently to `None` on a missing or explicitly-null "path" key, and
/// always emits its key with a JSON null on serialize when `None`. This
/// is the type whose instances would populate every `diagnostics:
/// Vec<Diagnostic>` field across monokl's result types
/// (docs/spec/07-edge-cases-and-failure-modes.md Part 2, High item 3) --
/// this track locks only this type's own shape; which command populates
/// it, and how, is each command's own implementation track. A
/// `Diagnostic` value carries no information distinguishing which
/// command or code path produced it beyond whatever text the caller puts
/// in `message` -- there is no source/origin field on this type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diagnostic {
    pub kind: DiagnosticKind,
    pub path: Option<Utf8PathBuf>,
    pub message: String,
}

/// monokl's internal grep-hit representation (AC-015) -- matches
/// `.claude/semantic-model/core-types.md`'s existing characterization of
/// LineHit as "internal grep-hit representation only". Exactly 2 fields
/// in this declaration order. Derives `Debug`, `Clone` only -- no
/// `Serialize`, no `Deserialize`, no `PartialEq`/`Eq`/`Copy`, and no
/// serde attribute macro on this struct at all. This is the only type in
/// the SPEC-006 cluster with no serde derive of any kind: LineHit has no
/// JSON representation whatsoever, and cannot be a field of any struct
/// that itself derives `Serialize`/`Deserialize` without a manual
/// implementation. Attempting either produces a compile-time failure (an
/// unsatisfied-trait-bound diagnostic, E0277, naming `LineHit` as the
/// type missing the required impl), not a runtime failure or a
/// silently-wrong JSON shape.
#[derive(Debug, Clone)]
pub struct LineHit {
    pub line_number: usize,
    pub text: String,
}

/// Search-command language filter (AC-007, AC-008, AC-009, AC-010).
/// Exactly 4 variants in this declaration order. Derives `Debug`, `Clone`,
/// `Copy`, `PartialEq`, `Eq`, `Serialize`, `Deserialize` -- the same
/// fieldless-enum pattern as `SymbolKind`/`Visibility`/`BindingKind` --
/// plus a `cli`-feature-gated `clap::ValueEnum` derive. No
/// `#[non_exhaustive]`, no `#[serde(other)]` fallback. Carries
/// `#[serde(rename_all = "lowercase")]`, unconditionally. Each variant's
/// `#[cfg_attr(feature = "cli", value(name = "..."))]` supplies clap's own,
/// independently maintained CLI value name -- stripped entirely from the
/// compiled type when the `cli` feature is disabled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "cli", derive(clap::ValueEnum))]
pub enum Language {
    #[cfg_attr(feature = "cli", value(name = "typescript"))]
    TypeScript,
    #[cfg_attr(feature = "cli", value(name = "javascript"))]
    JavaScript,
    #[cfg_attr(feature = "cli", value(name = "rust"))]
    Rust,
    #[cfg_attr(feature = "cli", value(name = "python"))]
    Python,
}

/// Search-command resource limits (AC-004, AC-005, AC-006). Exactly 4
/// fields in this declaration order. No field carries any field-level
/// serde attribute. Carries a hand-written `impl Default` -- not
/// `#[derive(Default)]` -- supplying `max_results: Some(50), max_bytes:
/// 2_097_152, max_tokens: Some(20_000), max_candidates: 1_000`. That
/// Default impl is completely inert for JSON deserialize: no
/// container-level or field-level `#[serde(default)]` appears anywhere on
/// this struct, so `max_bytes`/`max_candidates` are strictly required on
/// deserialize (missing-field error, not a fallback to the Default impl's
/// values), while `max_results`/`max_tokens` tolerate a missing or
/// explicitly-null key purely via `Option<T>`'s own independent
/// `Deserialize` leniency, defaulting to `None` -- not to `Some(50)` /
/// `Some(20_000)`, the values `SearchLimits::default()` would supply.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchLimits {
    pub max_results: Option<usize>,
    pub max_bytes: usize,
    pub max_tokens: Option<usize>,
    pub max_candidates: usize,
}

impl Default for SearchLimits {
    fn default() -> Self {
        Self { max_results: Some(50), max_bytes: 2_097_152, max_tokens: Some(20_000), max_candidates: 1_000 }
    }
}

/// Search-command request options (AC-001, AC-002, AC-003). Exactly 7
/// fields in this declaration order. No field carries any field-level
/// serde attribute. Carries a hand-written `impl Default` -- not
/// `#[derive(Default)]` -- supplying `query: String::new(), path:
/// Utf8PathBuf::new(), allow_tests: false, no_gitignore: false, limits:
/// SearchLimits::default(), exact: false, language: None`. That Default
/// impl is completely inert for JSON deserialize by the same mechanism as
/// `SearchLimits`: no container-level or field-level `#[serde(default)]`
/// appears anywhere on this struct, so `query`, `path`, `allow_tests`,
/// `no_gitignore`, `limits`, and `exact` are strictly required on
/// deserialize. `language: Option<Language>` is the sole exception, and
/// only because `Option<T>`'s own `Deserialize` impl independently
/// tolerates a missing or explicitly-null key.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOptions {
    pub query: String,
    pub path: Utf8PathBuf,
    pub allow_tests: bool,
    pub no_gitignore: bool,
    pub limits: SearchLimits,
    pub exact: bool,
    pub language: Option<Language>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            query: String::new(),
            path: Utf8PathBuf::new(),
            allow_tests: false,
            no_gitignore: false,
            limits: SearchLimits::default(),
            exact: false,
            language: None,
        }
    }
}

/// monokl's top-level search-command response (AC-011, AC-012). Derives
/// `Debug`, `Clone`, `Serialize` only -- no `Deserialize`: this type can
/// never be parsed from JSON. Exactly 7 fields in this declaration order.
/// No field carries any field-level serde attribute -- `results` and
/// `diagnostics` always emit their key as an explicit empty JSON array
/// even when empty (no `skip_serializing_if`), and `truncation_marker`
/// always emits its key with a JSON null when `None`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub results: Vec<RankedBlock>,
    pub total_blocks_before_truncation: usize,
    pub truncated: bool,
    pub truncation_marker: Option<String>,
    pub total_bytes: usize,
    pub total_tokens: usize,
    pub diagnostics: Vec<Diagnostic>,
}

/// monokl's top-level symbols-command response (AC-013, AC-014). Derives
/// `Debug`, `Clone`, `Serialize` only -- no `Deserialize`. Exactly 4 fields
/// in this declaration order. No field carries any field-level serde
/// attribute. `files` is a `BTreeMap`, not a `HashMap`: its keys serialize
/// in the order induced by `Utf8PathBuf`'s own `Ord` implementation,
/// deterministic for a given set of file paths on every run and every
/// machine. `diagnostics` always emits its key as an explicit empty JSON
/// array even when empty.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolsResult {
    pub files: std::collections::BTreeMap<Utf8PathBuf, Vec<SymbolEntry>>,
    pub total_symbol_count: usize,
    pub truncation_marker: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

/// monokl's top-level dependents-command response (AC-015, AC-016).
/// Derives `Debug`, `Clone`, `Serialize` only -- no `Deserialize`. Exactly
/// 7 fields in this declaration order. No field carries any field-level
/// serde attribute. `diagnostics` is the same `Diagnostic`-typed field
/// documented (docs/spec/07-edge-cases-and-failure-modes.md Part 1 finding
/// #1) as currently unpopulated in practice -- this track locks only this
/// type's own shape.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DependentsResult {
    pub file: Utf8PathBuf,
    pub dependents: Vec<Utf8PathBuf>,
    pub imports: Vec<Utf8PathBuf>,
    pub total_dependent_count: usize,
    pub total_import_count: usize,
    pub truncation_marker: Option<String>,
    pub diagnostics: Vec<Diagnostic>,
}

