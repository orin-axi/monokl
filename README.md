# monokl

AST-aware semantic code search for TypeScript, JavaScript, and Rust — built for AI coding agents, not humans reading a terminal.

> **Status: spec-stage.** No code exists yet. The full design lives in [`docs/spec/`](docs/spec/README.md); this repo is where that spec gets implemented. If you're looking for something to run, there isn't one yet — see [CONTRIBUTING.md](CONTRIBUTING.md) for where things stand.

## What it does

Grep finds text. Embeddings find _vibes_. monokl parses your code with a real compiler-grade parser ([OXC](https://oxc.rs) for TS/JS, [`ra_ap_syntax`](https://crates.io/crates/ra_ap_syntax) for Rust) and answers structural questions:

```
mnkl search "validatePrice" --path src/
mnkl refs Button --root . --include-tests
mnkl definition useCart --root .
mnkl dependents src/utils/format.ts --root .
mnkl explain src/components/Button.tsx
```

Text search ranks by BM25 and node-type relevance, not string distance. `refs`/`definition` resolve real bindings, not name matches. `dependents` walks an actual import graph. Everything returns structured JSON by default, with a JSON Schema per result type — built to be a tool an agent calls, not a report a human scrolls through.

## Why

Existing code-search-for-agents tools mostly fall into two camps: fast-but-fuzzy (grep, embeddings) or precise-but-heavyweight (a full language server, a persistent daemon). monokl aims at the middle — real AST parsing without the cost of running a language server, no persistent index to keep warm, cache-friendly enough that repeat queries in one session don't re-pay the parse cost. See [ARCHITECTURE.md](ARCHITECTURE.md) for how, and [PRINCIPLES.md](PRINCIPLES.md) for why it's built this way instead of the alternatives.

## Language support

| Language | Parser | Status |
| --- | --- | --- |
| TypeScript / JavaScript | OXC | Fully specified |
| Rust | `ra_ap_syntax` | Fully specified |
| Python | tree-sitter-python | Specified, not implemented |
| Go | [`gosyn`](https://github.com/chikaku/gosyn) | Specified, not implemented |
| Java | tree-sitter-java | Specified, not implemented |

Unsupported operations degrade explicitly (a `Skipped`/`Warning` diagnostic in the response) rather than failing silently or crashing — see the capability model in [ARCHITECTURE.md](ARCHITECTURE.md).

## Docs

- [ARCHITECTURE.md](ARCHITECTURE.md) — how it's built, in prose
- [PRINCIPLES.md](PRINCIPLES.md) — the design tenets and non-goals
- [CONTRIBUTING.md](CONTRIBUTING.md) — project status, how to work on this repo
- [`docs/spec/`](docs/spec/README.md) — the full reconstruction-grade specification
- [AGENTS.md](AGENTS.md) — guidance for AI coding agents working in this repo

## Name

Monokl (MON-oh-kl) — from _monocle_, a single precision lens for close examination. CLI binary is `monokl`, short alias `mnkl`.
