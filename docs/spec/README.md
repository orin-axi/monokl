# monokl spec

Reconstruction-grade specification for `monokl` — split by phase so no single file exceeds what a person (or an agent) can hold in working memory at once. Read in order; each part builds on the ones before it.

| File | Covers |
| --- | --- |
| [01-core-architecture.md](./01-core-architecture.md) | v0.1.0 baseline — types, error handling, query language, ranking, caching, the `LanguageAnalyzer`/`WorkspaceEnricher` design, CLI, the TS/JS analyzer |
| [02-inspection-and-analysis.md](./02-inspection-and-analysis.md) | `inspect`/`patterns`/`refs`/`definition`/`diff`/`tokens`/`explain`/`coverage`/`data-flow`/`similar`, the `InspectEntry` classifier |
| [03-multi-language-platform.md](./03-multi-language-platform.md) | `AnalyzerRegistry`, the real Rust analyzer, pipeline modularization, git-scoped queries, the capability/precision model, presentation layer |
| [04-analysis-fidelity.md](./04-analysis-fidelity.md) | Signature and symbol extraction fidelity fixes for TS and Rust |
| [05-research-and-decisions.md](./05-research-and-decisions.md) | Research findings, adopted/rejected design calls, and a Rust-idiom audit — read this to understand _why_, not just _what_ |
| [06-daemon-architecture.md](./06-daemon-architecture.md) | Research-backed design for the future daemon/session-mode tier (`PrecisionUpgrader`, per-language lifetime fit, confidence-signaling rules) — not yet implemented, not required for v0.1.0 |
| [07-edge-cases-and-failure-modes.md](./07-edge-cases-and-failure-modes.md) | Gap audit against Parts 1-6 — unhandled edge cases, self-contradictions, and guarantees the shown code doesn't actually deliver |
| [08-library-session-api.md](./08-library-session-api.md) | The embedded-library API — `Workspace`/`Snapshot`, `apply_change`, scoped opens, the batch `query`, `SymbolId`/`Provenance`/`Digest`, the `monokl-core`/`monokl-agent` split, and the benchmark the no-persistent-index bet rests on. Supersedes Part 1's free functions and caps; not yet implemented |

## Before you build from this

- **Code blocks are verbatim**, not illustrative pseudocode — what's shown is what gets written.
- **Check 05 first** when two files look inconsistent: it's the changelog for corrections made after the fact.
- **Check 07** before trusting a failure path is handled — several commands' `diagnostics` fields and error branches look complete but aren't wired up.

## Known gaps

- `benches/pipeline.rs`'s four criterion groups are still stubs — none of the latency targets asserted anywhere in this spec are measured yet, including in monokl's own suite. The ~50k-file threshold below which no persistent index is needed is an untested hypothesis, not a measurement; Part 8 §13 is the benchmark that decides it.
- The session/batch API in Part 8 is specified but unimplemented, and Part 1's free functions, `types.rs` serde derives, and pipeline caps are what it supersedes. It is a hard prerequisite for Wisp's Milestone 3 (context compiler), which embeds monokl in-process.
- Python (`lang-py`), Go (`lang-go`), and Java (`lang-java`) are specified but not implemented. Rust and TypeScript/JavaScript are the only analyzers with full verbatim code.
- See [05-research-and-decisions.md §4](./05-research-and-decisions.md) for open investigation tracks (Salsa, `ra_ap_hir` + a persistent session, ranking beyond BM25) that are recommended but not yet decided, and [06-daemon-architecture.md](./06-daemon-architecture.md) for the research-backed shape that future work should build toward once one of those tracks is picked up.
- See [07-edge-cases-and-failure-modes.md](./07-edge-cases-and-failure-modes.md) for the full list of undecided failure-mode behavior — none of it blocks starting implementation, but several findings (the `impl_owner`/`owner` and `Visibility::Super` mismatches in Part 4, in particular) should be resolved before writing the code they touch.
