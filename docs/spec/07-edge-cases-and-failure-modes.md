# Edge Cases & Failure Modes

A gap audit against Parts 1-6: every place the spec is silent, self-contradictory, or asserts a guarantee its own verbatim code doesn't deliver. Organized by severity within each part. An entry marked "inconsistent" means two sections of the spec disagree about what happens in the same scenario, not that the behavior is undefined — those are the highest-priority fixes, since they mean the spec currently permits two different implementations to both claim compliance.

Part of the [monokl spec](./README.md). Read against [01](./01-core-architecture.md)-[06](./06-daemon-architecture.md); each finding cites its exact location.

Every finding below has been through an adversarial verification pass — a second, independent read trying to refute each claim rather than confirm it. 55 of 61 findings held up exactly as originally stated; 5 were weakened or corrected in place (marked "Verified correction" inline) where the original framing overstated what the text actually supports; 1 (Part 3, finding 18) was refuted outright and is struck through rather than deleted, so the record shows what didn't hold up and why.

---

## Cross-cutting patterns

These recur across multiple parts and are worth fixing as a pattern, not one call site at a time.

**Silent data-drop with no diagnostic.** The spec's own graceful-degradation story (Part 1's `Diagnostic{kind: Degraded|Skipped|Warning}`) is applied inconsistently. Confirmed instances: `dependents` drops files that fail to analyze with only a `tracing::warn!`, never a `Diagnostic` (01#1); the Rust `InspectEntry` classifier keeps one struct/trait per file and drops the rest with no warning, unlike the identical TS multi-recipe case which does warn (02#1); `RustModuleEntry` has no field for free functions at all, so `utils.rs`-shaped files lose their contents entirely (02#2); most Part 2 commands declare a `diagnostics` field but never describe it being populated (02#3); macro-generated and `#[cfg(...)]`-gated Rust code is invisible with no capability/precision signal, unlike every other precision limitation in Part 3 (03#6); nested `mod` bodies, trait method bodies, associated consts/types, and `union` items are all silently dropped by the Part 4 fidelity fixes (04#4, #6, #7, #8).

**Contradictory handling of the identical condition within one document.** A malformed `regex:` term is a hard whole-query error in one search stage and silently dropped in the next (01#7). A non-UTF-8 path is a hard error at the workspace root but silently skipped mid-walk (01#12). A symlink hit mid-traversal is silently invisible; the same symlink passed as an explicit CLI target is a named hard error (03#16).

**Guarantees asserted in prose that the shown code doesn't implement.** Part 3 says content hash is authoritative and mtime+size is just a cheap dirty-bit, but the Tier-1 cache lookup returns on mtime+size alone with no hash check in that branch (03#1) — a same-second edit on a coarse-mtime filesystem serves stale results. The `io_safety` read-safety floor (symlink rejection, 50MB cap) is stated to guard `extract`/`refs`/`definition`, but the shown code for those paths uses raw `fs::read_to_string`, bypassing it (03#2).

**"1.0 on zero denominator" conflates "fully compliant" with "nothing to measure."** Recurs independently in `tokens_analysis`'s compliance score, `coverage`'s score, `similar`'s styling score, and `explain`'s token compliance (02#13) — a file with nothing to check reports the same perfect score as a file that's genuinely well-covered, with no sentinel distinguishing the two.

**Resource limits are opt-out by construction.** Only `max_bytes` is clamped to a hard ceiling; `max_results`, `max_tokens`, `max_candidates`, query string length, per-file hit counts, git change-set size, and `ImpactedNeighbors` expansion are all caller-controlled with no enforced cap (01#11, #16, #17; 03#11, #12).

**Naming mismatches that wouldn't compile.** Part 4 uses `impl_owner` throughout where Part 1/3's canonical `SymbolEntry` field is `owner` (04#1), and invents a `Visibility::Super` variant that doesn't exist in Part 1's four-variant enum (04#2). These aren't edge cases — they're the fidelity fixes failing to type-check against the structs they're supposed to extend.

---

## Part 1 — `01-core-architecture.md`

### Crash / data-loss / silent-wrong-answer

1. **`dependents` silently drops per-file analysis failures.** `WorkspaceIndex::build`'s rayon closure does `Err(e) => { tracing::warn!(...); None }` on any per-file failure — the file vanishes from the `ImportGraph`/`SymbolIndex` with no diagnostic captured anywhere. Verified correction: `WorkspaceIndex` itself has no `diagnostics` field at all (only `files`, `file_index`, `import_graph`, `symbol_index`), so there's no existing field silently going unpopulated — there's no mechanism at all. `search`/`symbols` surface the same failure class as a structured `Diagnostic` inline during their own per-file analysis; `dependents` routes through `WorkspaceIndex::build` and has nothing comparable to route a dropped-file signal into `DependentsResult.diagnostics`. An agent gets `dependents: []` with no signal the answer might be incomplete.
2. **No panic/stack-overflow guard around the parser.** `parse_full` runs OXC's recursive-descent parser directly inside a `rayon::par_iter()` closure with no `catch_unwind` (unlike the napi FFI boundary, which does wrap for this reason) and no recursion-depth/input-size guard. A genuine stack overflow aborts the process regardless of `catch_unwind`, and the blast radius is the entire in-flight `WorkspaceIndex::build` batch, not just the one bad file.
3. **`FileTooLarge`/`SymlinkRejected` are declared but never enforced in this file's own pipeline.** Part 1's reference `TsAnalyzer::analyze`/`WorkspaceIndex::build` read via plain `std::fs::metadata`/`read_to_string` with no size cap and no symlink check — the doc's own note says enforcement lives in Part 3's `io_safety.rs`, but a reader of Part 1 alone would assume these caps are live in the baseline pipeline.
4. **Concurrent disk-cache writes race with last-writer-wins.** The atomic temp+rename write prevents torn reads/writes but not lost writes: two concurrent `monokl` invocations each load their own cache snapshot and `rename()` over the same path independently — whichever finishes last silently discards the other's newly-cached entries. Self-healing (a lost entry just costs a re-parse), but undocumented.
5. **Corrupted `cache.json` hard-fails instead of self-healing.** `init()` explicitly catches `StaleDiskCache` and rebuilds from empty, but a genuinely malformed JSON (e.g. from a disk-full truncated write, still syntactically "complete" before the paired rename) surfaces as `MonoklError::Json` via `?` and is **not** special-cased — the whole workspace build hard-fails instead of falling back to a cold rebuild the way staleness does.
6. **`StaleDiskCache`'s error message references a CLI subcommand (`mnkl init --rebuild`) that doesn't exist**, and per this file's own `init()` logic, the variant is always intercepted internally and never actually surfaced to a caller — dead code as specified.
7. **Bad `regex:` term: hard error in one search stage, silently dropped in the next.** Stage 4 builds one combined regex from all terms and hard-fails the whole query (`?`) on any invalid pattern; stage 5 builds per-term regexes for highlighting and explicitly "silently drops regex compile failures." Same input problem, two contradictory outcomes within one command.
8. **`+regex:foo` loses more than the comment claims.** The code comment says it "silently loses modifier"; tracing the lexer shows it actually degrades all the way to an optional escaped-literal match on the string `foo` — both the required-modifier *and* the regex semantics are discarded, not just the modifier.
9. **Ranking's stated tie-break isn't implemented.** Design tenet #4 claims "tie-break on `(file, line_start)`"; `rank_blocks` is a single-key `sort_by` on `final_score` alone with no secondary comparator. Likely stable in practice via upstream deterministic ordering, but the code doesn't implement the guarantee the prose asserts.
10. **Non-deterministic candidate selection above `max_candidates`.** The file walk has no `.sort_by_file_name()` and breaks once the 1000-file default cap is hit; walk order is OS/filesystem-dependent. Which 1000 files make the candidate set — and thus the final ranked output — is not reproducible across runs or machines, directly undercutting the determinism tenet.
11. **Only `max_bytes` is clamped; `max_results`/`max_tokens`/`max_candidates` have no ceiling.** The one resource guard this system has for "query matches an enormous number of results" is opt-out by construction for three of its four knobs.

### Medium — inconsistent handling / missing validation

12. Non-UTF-8 root path is a hard `NonUtf8Path` error; the same condition mid-walk in `text_search::search_files` is a silent `continue`.
13. `symbols`' `files: Vec<Utf8PathBuf>` isn't deduplicated; a duplicate path overwrites the first result in the `BTreeMap` while `total_symbol_count` double-counts it — breaking the spec's own asserted invariant (`total_symbol_count == sum of file symbol counts`).
14. `count-tokens --stdin` combined with a non-empty `files` arg silently drops `files` with no warning; supplying neither yields a silent, misleadingly-successful "0 tokens" result indistinguishable from a caller mistake.
15. `extract` never validates `line_start <= line_end`; a swapped range silently returns an empty (not erroring) result.
16. No cap on query string length or a single lexer token's length — `LIMIT: usize = 64` bounds term count, not character length.
17. No per-file cap on collected matched lines during text search — only file count is bounded.
18. Orphaned `.monokl/cache.json.<pid>.tmp` files from crashed/killed processes are never cleaned up — a slow disk-space leak with no documented GC.
19. **Two contradictory error-output mechanisms are both defined, and only one is wired up.** `output.rs::render_error` defines JSON-on-stdout with `exit(1)`; the actual `main()` never calls it and instead uses miette's human-readable diagnostic on stderr. The spec doesn't say which governs — a real problem for a tool whose value proposition is agent-parseable output.
20. Auto pretty-vs-compact JSON selection via `stdout().is_terminal()` silently switches to multi-line pretty JSON if the calling environment allocates a pseudo-TTY (plausible for some agent-orchestration wrappers), breaking a naive line-oriented JSON parser on the caller's side.
21. Exclusion-only queries (`-foo -bar`, no positive terms) always silently return an empty response with no diagnostic explaining that this is a no-op by design.

### Low

22. `--no-default-features --features cli` (cli without lang-ts) plausibly fails to compile — `cli`'s `main.rs` unconditionally calls into the `lang-ts`-gated `pipeline` module, and the feature table doesn't forbid the combination.
23. `extract --line-start 0` isn't rejected despite the codebase's 1-indexed convention elsewhere.

---

## Part 2 — `02-inspection-and-analysis.md`

### High

1. Rust `InspectEntry` classification keeps only the first trait (or, absent a trait, one struct by a method/field-count heuristic) and silently drops every other struct/enum/trait in the file — no `Warning` diagnostic, unlike the identical TS multi-recipe case, which does warn.
2. `RustModuleEntry` has no field for free functions; a `utils.rs`-shaped file (no struct/enum/trait, just `pub fn` helpers) is classified `RustModule` and its functions are never captured anywhere in `InspectEntry` output.
3. Nearly every Part 2 command declares a `diagnostics: Vec<Diagnostic>` field but never describes it being populated — `refs`, `definition`, `diff`, `tokens`, `explain`, `coverage`, `data-flow`, `similar` all leave it silently unwired, breaking Part 1's graceful-degrade contract systemically rather than in one place.
4. `diff`'s algorithm never states what happens when `git show base:path`/`git show head:path` fails for the missing side — which is not a rare case, it's the mechanism for the two most common diff outcomes (file added or deleted).
5. `diff` never reconciles which path (old, new, or both) feeds `analyze_change` for a `Renamed` file, despite the algorithm being described as operating on a single path.
6. `refs`' line-sweep mention detector has no comment/string-literal exclusion, unlike other regex scanners in the same spec (`tokens_analysis`, Rust's `strip_block_comments`) — a symbol name in a comment or string produces a spurious reference that propagates into every command built on `refs::find_refs` (`coverage`, `explain`, `data-flow`).
7. `coverage`'s `find_refs` call truncates at `max_refs=100` with no propagation of the existing `truncated` flag into `CoverageResult` — a widely-used symbol whose only test reference falls outside the first 100 can be silently mislabeled `untested`.
8. `definition` has no cap or truncation flag on ambiguous matches, unlike `refs`'s `max_refs`/`truncated` — a generic name queried workspace-wide returns every declaration site unbounded.

### Medium

9. `inspect --kind` takes an unvalidated `String` (unlike `--language`, a real `value_enum`) — a typo silently returns zero entries, indistinguishable from a genuinely empty result.
10. `InspectResult.file_count` semantics under `--kind` filtering are unstated — pre- or post-filter — breaking any coverage-ratio computation an agent might do.
11. Re-export chain truncation (`MAX_RE_EXPORT_HOPS = 10`) and cycle-breaking are silent — no `truncated`/`complete` flag on `DefinitionSite.re_export_chain`, risking an agent treating a partial chain as the canonical origin.
12. `explain`'s `kind_confidence` is a hardcoded per-kind constant for 8 of 10 kinds (only `ReactComponent`/`ReactHook` derive from actual signal strength) — a borderline classification and an obvious one report identical confidence.
13. The "fall back to 1.0 on zero denominator" pattern recurs independently across `tokens_analysis`, `coverage`, `similar`, and `explain` — see cross-cutting section above.
14. `similar` has no field recording how many candidates were actually considered — "top 5 of 6000" and "top 3 of 3" are indistinguishable.
15. `data-flow`'s purely textual pattern matching has no diagnostic distinguishing "confirmed no inputs/outputs" from "the heuristic scanner didn't recognize this code shape" (e.g. fetch wrapped in a custom client/hook).
16. `coverage`'s `score` field name invites an agent to read it as real test coverage, when it's a static-reference heuristic that doesn't run tests. Verified correction: the command's Purpose section does state "Deterministic — does not run tests" in prose, so the caveat isn't entirely absent — it's just not attached to the `score` field itself, where an agent reading the JSON schema without the surrounding prose would miss it.

### Low / scope notes

17. `mnkl tokens` in this part is a design-token discipline audit, not the LLM token-budget counter — the boundary-condition questions about token limits belong to Part 1's `CountTokens`, not here.
18. `mnkl patterns` here is a frequency aggregator, not a pattern-matching query language — the "overlapping matches" framing doesn't map onto this command as specified.
19. `find_balanced` has no stated behavior for unbalanced/unterminated braces inside a `css()`/`cva()` call (e.g. a WIP file mid-edit).
20. `refs`/`definition`'s `from_file` narrowing has no stated behavior when `from_file` itself fails to exist or parse. Verified correction: `similar` and `coverage` don't take a `from_file` parameter at all in this spec — the finding applies only to `refs`/`definition`.

---

## Part 3 — `03-multi-language-platform.md`

### High

1. **Tier-1 cache lookup trusts mtime+size alone**, contradicting Part 1's own "content hash is authoritative" claim — no hash check in that code path. Coarse mtime granularity or clock skew can serve stale analysis with zero diagnostic.
2. **`io_safety`'s read-safety floor is bypassed by the exact commands it's claimed to guard.** `extract` and the `refs`/`definition` symbol-resolution path (`analyze_with_profile`) use raw `fs::read_to_string`, not `read_to_string_capped` — a hostile symlink or 50MB+ file isn't rejected on these paths despite the prose saying it is.
3. **`ChangedLines` scope diffs git-tree state but reads live working-tree files** — an agent iterating on its own uncommitted PR (the primary use case for this feature) can have line numbers silently shift between what was diffed and what's on disk.
4. **Shallow clones are never mentioned, and a merge-base failure is a hard `MonoklError::Git` error, not silent.** Verified correction: the original "silently produce a wrong or missing merge-base" framing overstates this — `merge_base` failures propagate loudly via `?`, they don't fail quietly. The real, still-unaddressed risk is narrower: `gix`'s merge-base walk over a shallow clone's truncated history (the common case in CI, which is this feature's primary use case) could plausibly return an *incorrect-but-present* ancestor without erroring at all — that specific scenario isn't discussed anywhere, but "silent failure" isn't the right description of what's missing.
5. **Rust workspace crate-name collisions silently resolve to one crate** — `discover_workspace_packages` keys by normalized name with no diagnostic on overwrite, so cross-crate `use` resolution can silently pick the wrong crate's directory in a multi-crate workspace.
6. **Macro-generated code and inactive `#[cfg(...)]` branches are invisible with no capability/precision signal**, unlike every other Rust precision limitation in this file — `AnalyzerCapabilities` has no field flagging it, unlike the crate-graph-resolution heuristic that does get one. `pick_main_struct`'s heuristic can arbitrarily pick a platform-dead struct as "the" struct with no signal it might not be live in the analyzed configuration. (The `include!`'d-files claim from the original pass didn't hold up on verification — `include!` isn't mentioned anywhere in this file, so that part is dropped rather than asserted as a gap.)

### Medium — hard failures where graceful degradation is implied but not delivered

7. Disk-full/permission-denied during cache persistence aborts the entire command via `?`, even though the in-memory analysis that answers the query already succeeded — contradicts the doc's own framing of persistence as a decoupled, secondary concern.
8. Unrelated-history merge-base failure hard-errors the whole command rather than degrading to `FullRepo` with a warning, the pattern used elsewhere in this file for precision degradation.
9. A repo with no commits (or no `origin` remote) is unaddressed — nothing distinguishes "no history" from "typo'd ref," no fallback.
10. `.d.ts` files get a misleading "TypeScript isn't supported in this build" diagnostic when TS is fully supported and this one file is intentionally excluded by design — an agent parsing diagnostics would draw the wrong conclusion about feature availability.

### Resource bounds (DoS-shaped, not correctness)

11. No cap on git change-set size before per-command output caps apply — a whole-repo-rewrite PR is fully enumerated and diffed before any truncation.
12. `ImpactedNeighbors` one-hop expansion has no pre-analysis cap — a widely-imported hub file can balloon the analyzed set into the hundreds before output truncation discards most of it.
13. `Vec<Diagnostic>` fields have no stated ceiling, unlike every other result list in this file (symbols: 50/file, 500 total; dependents: 200).

### DX / presentation

14. No `NO_COLOR`/`CLICOLOR`/`CLICOLOR_FORCE` support — only TTY-ness is consulted, despite real investment in `owo-colors` and a bespoke `--color` flag.
15. The Mermaid "small graphs" threshold (≤12 nodes, ≤20 edges) only gates the *auto* format decision — an explicit `--format mermaid` request above that threshold has no documented behavior.
16. Symlinks hit mid-traversal are silently invisible (default `follow_links(false)`) — solidly confirmed. The stated contrast, that an explicit symlink target instead gets a loud, named `SymlinkRejected` error, rests only on prose (§13): the same verification pass that found `io_safety` bypassed on `extract`/`refs`/`definition` (finding #2 above) also found `read_to_string_capped`/`SymlinkRejected` never actually invoked by any shown command path. So the "loud error" side of this asymmetry is asserted, not demonstrated in code — treat this as one symptom of finding #2, not an independent confirmed contrast.
17. The global-flag combination matrix (`--format`/`--profile`/`--detail`/`--color`) is only partially tested — most combinations' behavior (silent-ignore vs. warn vs. reject) is unstated.
18. ~~JSON-vs-text diagnostic parity is untested.~~ **Refuted on verification** — `migration_surface_text_output_includes_unsupported_diagnostics` explicitly asserts a diagnostic message survives into text output, directly contradicting this finding as originally stated. There's a weaker, real gap underneath (no *comprehensive* cross-command parity suite covering every diagnostic kind across every command), but the blanket claim doesn't hold and shouldn't be treated as a confirmed gap.
19. The 50MB `io_safety` cap and the 2MB search token/byte budget coexist with no explained relationship — a 49MB file passes the safety gate, gets fully parsed, then has virtually all output discarded by the budget.
20. `DependencyTarget::Namespace` is spec residue from the dropped C# roadmap item — cosmetic, not functional.

---

## Part 4 — `04-analysis-fidelity.md`

### Would not compile / direct contradiction with Parts 1 & 3

1. **`impl_owner` doesn't exist.** Part 4 uses this field name throughout (header, code identifiers, test description); Part 1's canonical `SymbolEntry` and Part 3's confirmed field list both say `owner`. Not a rename — just a different name for the same field, unreconciled.
2. **`Visibility::Super` doesn't exist.** Part 1's `Visibility` enum has exactly four variants (`Public, Crate, Module, Private`); Part 4 maps `pub(super)` to a nonexistent fifth variant. Part 2's own pre-existing visibility mapper already maps this case correctly to `Module`.
3. `normalize_signature` is claimed to be "the same normalization path" as the existing `FunctionDeclaration` handling, but Parts 1 and 2 describe that existing path (`first_line_signature`) as truncating at the first newline — materially different from whitespace-collapsing across a full multi-line span. Either Part 4 mischaracterizes existing behavior or Parts 1/2 are stale.

### Coverage gaps in the fidelity fix itself

4. `rust_signature`'s stop-token assumption (`L_CURLY` terminates fn/struct/enum/trait/mod/impl signatures) breaks for unit structs (`struct Foo;`) and tuple structs (`struct Foo(i32);`) — neither has an `L_CURLY` token, and mainline usage, not an exotic case.
5. Nested/inline modules (`mod tests { ... }`) are never recursed into — only `impl` bodies get special-cased recursion. `#[cfg(test)] mod tests` is near-universal in real Rust, so an entire class of functions is invisible.
6. `#[cfg(...)]`-gated same-named items (platform-specific impls) produce two `SymbolEntry`s with identical names and nothing distinguishing which cfg branch produced which — no disambiguation field.
7. Associated consts and associated types inside `impl` blocks are dropped — only `fn` items are extracted (pre-existing limitation in Part 3 too, but unaddressed here despite this being the fidelity-focused pass).
8. Trait body methods (required and default-impl) are never extracted as symbols at all — only `impl` bodies get per-method extraction.
9. `union` items are silently unsupported and not listed among the handled top-level kinds, unlike the TS anonymous-default-export exclusion, which does get an explicit callout.
10. Class-member arrow functions (`class Foo { handleClick = () => {} }`, the standard React-class-component/MobX pattern) are silently outside the fix's scope — not listed under "Shapes NOT covered (intentional)," despite the fix's summary line implying full callable coverage.
11. Ambient `.d.ts` declarations (`declare const Widget: (props: P) => JSX.Element`, no initializer) never trigger `variable_function_signature`, which requires `declarator.init` — a real, common pattern, also missing from the "Shapes NOT covered" list, which reads as exhaustive when it isn't.
12. TS function overloads are unaddressed anywhere in the spec — `extract_stmt` emits one `SymbolEntry` per bare `FunctionDeclaration` unconditionally, so an overload set produces multiple same-named entries with no relationship indicated. Pre-existing gap, but this is the document that should have flagged it.
13. **No mention of cache-invalidation interaction.** A warm disk cache does get invalidated on a `monokl` version bump (via `config_hash`), but Part 4 never states that shipping this fidelity fix constitutes such a bump, and never addresses the daemon-mode case (Part 6) where a long-lived in-memory process checks only `content_hash` equality, not analyzer-version equality — a live daemon could keep serving pre-fix signatures across a binary swap with no process restart.
14. The Python `__all__` re-export fix's one worked example is the simplest possible case (direct literal import, literal list) — aliased re-exports, computed `__all__` (list comprehensions over `dir()`), and multi-level barrel chains are all unaddressed. Lower severity: this section is explicitly forward-looking, not-yet-implemented work.

---

## Parts 5 & 6 — `05-research-and-decisions.md` / `06-daemon-architecture.md`

Calibrated differently from Parts 1-4: 05 is a decision log where "not yet decided" is an honest, correct state, and 06 is explicitly a not-yet-implemented future design. Findings below are gaps in *adopted* decisions or *specified* design, not complaints about acknowledged open items (those are listed at the end, explicitly not findings).

1. **`PrecisionLedger` has no reconciliation rule for multiple annotations on the same content hash.** The design explicitly allows eager, lazy, and retroactive writes to accumulate against one `ContentHash` — but nothing specifies how a reader resolves multiple entries targeting the same fact: last-write-wins, min-precision-wins, dedup-and-replace, or append-and-let-the-caller-choose are all consistent with the given struct, and each gives a different answer to "what's this symbol's precision right now." `compute_precision()` itself is a pure, deterministic function — the indeterminacy is one layer up, in storage/retrieval semantics the design doesn't specify.
2. **`Session(Owned)` resources have no orphan-detection or crash-recovery story, and no concurrent-ownership arbitration.** The "Attached" side is designed to detect absence/staleness of an externally-owned resource, but nothing addresses monokl's own owned child process being orphaned by `kill -9` of the parent (no pidfile/lockfile/reaping story), or two monokl invocations racing to own the same backend for the same workspace. The stated "concurrency contract" only covers in-process thread safety, not cross-process contention.
3. **`PrecisionUpgrader::upgrade()` failing mid-session has no defined degrade-back-to-baseline contract.** The Java row explicitly documents jdtls's hard-crash failure mode and mitigates it only at session start (a project-size gate) — nothing covers a workspace growing past that threshold *after* the gate passed and the backend crashing mid-session. No fallback-to-cached-baseline, no re-establishment story, no circuit-breaker against crash-looping.
4. **Kotlin/Scala's JVM-subprocess tier has no documented fallback when no JVM is present**, unlike the parallel C/C++ case (which explicitly falls back to the tree-sitter tier when Clang is missing). Kotlin/Scala were deliberately excluded from that same tree-sitter fallback tier, so the JVM-subprocess analyzer is each language's only rung below the deferred `Exact` daemon tier — with nothing under it. What capability gets reported with no JVM on `PATH` is unstated.
5. **The eval-harness corpus is asserted to need to be "frozen" for the harness to mean anything, but no pinning mechanism is specified.** The 12-repo list records branch-default caveats and point-in-time license/size verification, never a commit SHA, tag, fork, or mirror strategy — directly undermining the stated goal of running this as a determinism check "on every commit," since any of the 12 repos changing upstream (force-push, deletion, relicense) silently breaks that determinism.
6. **`PrecisionLedger` has no eviction or size-bound story.** Combined with finding #1, growth is unbounded on two axes (distinct content hashes × entries per hash) for a long-running session or repeated retroactive sweeps — no TTL, LRU, or max-size policy, notable given how deliberately the rest of the design bounds cost.

**Explicitly checked and not findings** (already correctly handled, or already correctly flagged as the document's own open work): Clang-absence fallback (handled); 05 §4's undecided items (Salsa, `ra_ap_hir`+daemon) staying undecided and not silently assumed elsewhere in 01-04 (verified, no leakage found); `compute_precision()`'s own internal determinism (the function is pure — see finding #1 for the actual risk one layer up); per-language lifetime table's internal consistency (checked, no contradiction); §9's own listed known gaps (owned-vs-attached representation, unconfirmed feedback mechanisms) — these are the document doing its job, not gaps in it.

---

## Using this document

Every finding above is a design decision that hasn't been made yet, not a description of current behavior — nothing here is implemented. Treat this as a backlog: the cross-cutting patterns section is the highest-leverage place to start, since fixing the underlying pattern (e.g., "when does a per-file failure get a `Diagnostic` vs. a silent `tracing::warn!`") resolves several individual findings at once rather than patching each call site separately.
