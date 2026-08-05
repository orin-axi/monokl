# Agent guidance

Guidance for AI coding agents working in this repo. Vendor-neutral — see [CLAUDE.md](CLAUDE.md) for Claude Code-specific additions.

## Read this first

1. [`docs/spec/README.md`](docs/spec/README.md) — index of the full spec, tells you which file covers what.
2. [`docs/spec/05-research-and-decisions.md`](docs/spec/05-research-and-decisions.md) — the changelog. If something in files 01-04 looks inconsistent, check here before "fixing" it; it's probably already addressed, and the reasoning is recorded.
3. [PRINCIPLES.md](PRINCIPLES.md) — non-goals in particular. Don't propose an embedding-based reranker, a persistent trigram index, or a tree-sitter TS/JS parser — all three were considered and explicitly rejected, with reasons on record.

## Current state

No code exists yet — this is spec-stage. There's no `Cargo.toml`, no build, no test suite to run. If you're asked to implement something, you're implementing _from_ the spec, not extending existing code. Don't invent APIs, types, or command flags that aren't in the spec without flagging that you're doing so — the spec is meant to be reconstructed verbatim, and silent deviation defeats that.

## Code conventions (apply from the first line of code)

- No `unwrap()`/`expect()`/`panic!()`/`unreachable!()`/`unimplemented!()` outside `#[cfg(test)]` — denied by the workspace's own clippy lint table.
- One `thiserror` error enum per crate. Compose new failure modes via `#[from]`/`#[source]`, don't add a `String`-message variant when a structured one is possible.
- `LanguageAnalyzer` implementations must return fully owned data — no arena-borrowed AST nodes, no lifetimes tied to the parser's `Allocator`, ever, in any field of `FileAnalysis`.
- `camino::Utf8Path`/`Utf8PathBuf` everywhere paths appear — not `std::path::Path`.
- `rustc-hash`'s `FxHashMap`/`FxHashSet` for hot-path internal maps; `BTreeMap` for anything that gets serialized to JSON output (stable key order is load-bearing for determinism, see [PRINCIPLES.md](PRINCIPLES.md)).

## When you find a real bug in the spec

Fix it in the spec file, and add an entry to [`docs/spec/05-research-and-decisions.md`](docs/spec/05-research-and-decisions.md) explaining what was wrong and why the fix is correct — the same pattern already used for the two compile-breaking gaps found there. Don't fix it only in code without updating the spec; the spec is the reconstruction source, and a silent divergence between the two is worse than either one being wrong on its own.

## Testing

`cargo nextest` once tests exist. `insta` for snapshots — accept a snapshot change only after confirming the new output is actually correct, not just different.
