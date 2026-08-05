# Architecture

This is the design in prose. For verbatim code, see [`docs/spec/`](docs/spec/README.md).

## Two levels: analyze once, aggregate separately

Every file gets parsed exactly once, by a `LanguageAnalyzer`:

```
file → LanguageAnalyzer::analyze() → FileAnalysis (owned: symbols, deps, exports, code blocks)
```

`FileAnalysis` never borrows from the parser's arena — everything in it is owned data (`String`, not `&str`; `Vec<T>` of owned items). The arena drops at the end of `analyze()`. This is a hard rule, not a style preference: it's what lets `FileAnalysis` be cached, shared across threads via `Arc`, and outlive the parse call that produced it.

A second layer, `WorkspaceEnricher`, folds per-file analyses into workspace-wide structure — an import graph, a symbol index — without re-parsing anything:

```
Vec<FileAnalysis> → WorkspaceEnricher::update() × N files → WorkspaceIndex
```

Two enrichers exist today: `ImportGraphEnricher` (which files import which) and `SymbolIndexEnricher` (symbol name → declaration sites). Both are cheap bookkeeping over already-parsed data — the expensive part is parsing, done once, upstream.

Building a `WorkspaceIndex` is three phases:

1. **Parallel analyze.** Walk the directory, run `LanguageAnalyzer::analyze()` per file via `rayon`, then sort by path — sorting _after_ parallel completion, not enforcing commit order _during_ it, because import resolution is real (`oxc_resolver`, not name-matching), so there's no candidate-disambiguation-by-insertion-order problem to solve in the first place.
2. **Stream `update()`** for every (file, enricher) pair.
3. **`finalize()`** each enricher — by now the full file index exists, so an enricher can resolve `Utf8PathBuf → FileIdx` for edges it deferred during `update()`.

## The search pipeline

`search` is eight stages, each one doing strictly less work on strictly fewer candidates than the last:

1. Parse the query (`+required` / `-excluded` / bare-optional / `"phrase"` / `regex:pattern`).
2. Canonicalize the workspace root.
3. Text prefilter — `grep-searcher` (the crate ripgrep is built on) over the candidate file set.
4. Boolean evaluation — drop files missing a required term or containing an excluded one.
5. Block retrieval — analyze surviving files, keep only code blocks overlapping a matched line.
6. BM25 rank (`K1=1.5`, `B=0.5`) plus a node-type boost (functions rank higher than modules) and a coverage boost (more of the block matched = higher).
7. Dedup — drop blocks that overlap >50% with a higher-ranked block in the same file.
8. Token budget — truncate to fit, with an explicit marker so an agent knows results were cut.

Every stage is designed to fail toward "fewer, better results" rather than "more results, sort it out downstream" — the token budget in stage 8 is a last resort, not the primary filter.

## Caching: two tiers, one invariant

```
stat(path) → mtime + size match? → return cached, no file read
           → no match → read file → blake3 hash → hash match? → refresh mtime, return cached
                                                  → no match → re-parse, cache result
```

`ContentHash` (blake3) is authoritative; mtime/size is the cheap dirty-bit that avoids reading files that haven't actually changed. Two storage layers: an in-memory `DashMap` (process-local, lock-free reads) for repeat calls within one run, and a disk-persisted `.monokl/cache.json` (version- and config-hash-gated, so a monokl upgrade or a tsconfig change invalidates it automatically) for repeat calls across separate CLI invocations.

There's deliberately no persistent on-disk _search_ index (no tantivy-style inverted index, no SQLite). The cache remembers _parses_; the `WorkspaceIndex` (import graph, symbol index) is rebuilt in memory on every invocation from the cached per-file data — cheap, because it's aggregation over already-parsed structure, not re-parsing. See [`docs/spec/05-research-and-decisions.md`](docs/spec/05-research-and-decisions.md) for the numbers behind that tradeoff and when it stops holding (roughly: past ~50k files, a persistent session becomes worth its complexity — not decided yet, tracked as an open question).

## Multi-language: capability, not uniformity

Not every language gets the same precision, and the design says so explicitly instead of pretending otherwise. Every analyzer reports a `CapabilityPrecision` per operation:

```
Unsupported < Heuristic < Structural < Exact
```

- **Exact** — real resolution (e.g. TS/JS import resolution via `oxc_resolver`).
- **Structural** — real AST parsing, heuristic resolution (Rust's module-path walker; Go's `go.mod`-based cross-package resolution).
- **Heuristic** — regex/text-scan level (a language with no real parser yet).
- **Unsupported** — the operation doesn't exist for this language.

When an operation asks for more precision than an analyzer can give, the response carries a `Skipped` or `Warning` diagnostic instead of silently returning wrong answers or crashing. An `AnalyzerRegistry` dispatches each file to the first analyzer that claims it — TS/JS and Rust are registered with full native parsers; Python, Go, and Java are specified at `Structural` target precision but not yet implemented (see [README.md](README.md) for current status).

## What's explicitly out of scope

Full list and rationale in [PRINCIPLES.md](PRINCIPLES.md#non-goals) — briefly: no persistent search index, no full [Salsa](https://github.com/salsa-rs/salsa) incremental framework (yet — tracked as an open investigation, not ruled out), no embedding-based retrieval by default, no stemming, no in-memory trigram index (measured at 600MB–1.2GB at scale in an earlier design, rejected for cause).
