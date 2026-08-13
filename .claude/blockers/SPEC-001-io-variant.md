# Blocker: MonoklError::Io variant cannot be implemented

**Spec**: SPEC-001 (`.claude/specs/SPEC-001.json`), AC-001 and AC-004.
**Blocks**: full satisfaction of AC-001 ("exactly these 14 variants"; only 13 are
present in `crates/monokl/src/error.rs` as of this plan) and AC-004 in full
(the `Io` variant's `#[error(transparent)]` behavior and `FileIoError`
pass-through cannot be written or tested).

## What's missing

1. **The `io-errors` crate does not exist on disk.** `crates/monokl/Cargo.toml`
   (line 43) already declares a mandatory, non-optional dependency on it:
   `io-errors = { path = "../io-errors", version = "0.1", registry =
   "orin-cargo", default-features = false }`. There is no `crates/io-errors/`
   directory anywhere in this workspace, so the path cannot resolve.
2. **No registry configuration for `orin-cargo` exists.** The same dependency
   line pins `registry = "orin-cargo"`. There is no `.cargo/config.toml`
   anywhere in this workspace defining `[registries.orin-cargo]`. This is the
   same gap SPEC-004's AC-008B already documented and PLAN-004 already worked
   around (via offline text-based verification instead of real `cargo`
   invocations) -- it is not new to SPEC-001, but SPEC-001's Io variant is the
   first place this gap blocks *content*, not just build tooling: even if
   `io-errors` existed on disk, `cargo metadata`/`build`/`check`/`test`
   against the real `crates/monokl` manifest still cannot resolve, because the
   `registry = "orin-cargo"` key alone fails manifest parsing with `registry
   index was not found in any configuration: `orin-cargo`` (empirically
   confirmed during SPEC-004/PLAN-004's own planning, and independently
   reconfirmed live against this workspace's real Cargo.toml -- see this
   plan's `baseline_build_status` note).
3. `FileIoError` itself (the type `Io` wraps) is explicitly out of this
   spec's scope (`non_goals`: "FileIoError itself (owned by the io-errors
   crate) -- external dependency") -- SPEC-001 does not define it and this
   blocker does not ask SPEC-001 to.

## Why this plan does not route around it

SPEC-001 is silent on how to handle this gap. Inventing a resolution here --
e.g. a local stub `FileIoError` type, a cfg-gated placeholder, or quietly
omitting the variant without saying so -- would either (a) assert a fact
(`FileIoError`'s shape) this spec explicitly puts out of scope, or (b) leave
the enum silently non-conformant with AC-001 with no record of why. Per this
plan's own instructions, the correct action is to flag the contradiction
explicitly rather than guess. Accordingly, T-009 (this task) claims ZERO
acceptance criteria in its own `covers_criteria` -- it is a documentation-only
task that implements no Rust and discharges no criterion. AC-001 and AC-004
are recorded in this plan's top-level `deferred_criteria` field instead, so
they are never silently treated as satisfied.

## What decision is needed to unblock

Both of the following, in either order:

- Create the `io-errors` crate at `../io-errors` (relative to
  `crates/monokl`), defining `FileIoError` per
  `docs/spec/01-core-architecture.md:2959-2984` (io-errors/file_io.rs
  verbatim) -- this is explicitly a separate track's job (non_goals).
- Add `.cargo/config.toml` with a `[registries.orin-cargo]` entry pointing at
  a real, resolvable registry index (or repoint
  `crates/monokl/Cargo.toml`'s `io-errors` dependency at a source that
  doesn't require one) -- tracked since SPEC-004/AC-008B, still open.

Once both are in place, add the `Io(#[from] FileIoError)` variant (with
`#[error(transparent)]`) as the first variant in `MonoklError`, per AC-001,
and extend AC-004's Display/`source()` behavior test to a real
`FileIoError` instance from the now-real `io-errors` crate.

## Current state

- `crates/monokl/src/error.rs` (as of T-008) has 13 of AC-001's 14 required
  variants, in the spec's exact order, with a placeholder comment where `Io`
  belongs.
- AC-001 and AC-004 are therefore only partially satisfied by this plan.
  This file is the record of what remains and why.
