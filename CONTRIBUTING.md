# Contributing

By participating in this project, you agree to abide by the [Code of Conduct](./CODE_OF_CONDUCT.md). Found a security issue? See [SECURITY.md](./SECURITY.md) instead of opening a public issue.

## Where things stand

monokl is spec-stage — no `Cargo.toml`, no `src/`, nothing to build or run yet. The design is fully written up in [`docs/spec/`](docs/spec/README.md). If you're picking this up to start implementation, read [`docs/spec/README.md`](docs/spec/README.md) first; it tells you which file to read for which subsystem.

Before writing code against the spec, check [`docs/spec/05-research-and-decisions.md`](docs/spec/05-research-and-decisions.md) — it's the changelog for corrections made after the fact (compile-breaking gaps, dropped languages, adopted-but-unbuilt design changes). The other four spec files are the source of truth for _what_ to build; this one is the source of truth for _why_ it looks the way it does.

## Proposing a spec change

Don't silently edit around a design decision you disagree with. Add a dated entry to `docs/spec/05-research-and-decisions.md` explaining what changed and why — same discipline that's already been applied throughout that file. If the change is substantial enough to need its own writeup, add a new file under `docs/spec/` and link it from [`docs/spec/README.md`](docs/spec/README.md).

## Once there's code

These conventions are already locked into the spec — `docs/spec/01-core-architecture.md`'s workspace lint table — and apply from the first commit, not as an aspiration to grow into:

- **No `unwrap()`, `expect()`, `panic!()`, `unreachable!()`, or `unimplemented!()` in library code.** Denied at the workspace level via `clippy`. Fine in `#[cfg(test)]` code — assertions are supposed to panic on failure there.
- **`thiserror`, one error enum per crate**, composed via `#[from]`/`#[source]` rather than proliferating string-message variants. See `MonoklError` in [`docs/spec/01-core-architecture.md`](docs/spec/01-core-architecture.md) for the pattern.
- **Arena data never escapes a per-file parse call.** If you're touching a `LanguageAnalyzer` impl, everything in the returned `FileAnalysis` must be owned — no borrowed AST nodes, no lifetimes tied to the parser's allocator.
- **`camino::Utf8Path`/`Utf8PathBuf`**, not `std::path::Path` — the whole crate assumes UTF-8 paths.
- **`cargo nextest`** for running tests, **`insta`** for snapshot tests.

## Filing an issue

Say what you expected, what happened, and — if it's a spec-vs-reality mismatch — which file in `docs/spec/` you were reading. "The spec says X but Y happens" is more actionable than "this is broken."
