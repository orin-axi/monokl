# CLAUDE.md

See [AGENTS.md](AGENTS.md) for the full guidance — it applies here unchanged. This file only adds what's specific to working in Claude Code.

## Context

monokl is one of four tools in the **orin-axi** suite (alongside Firkin, Lumen, Pulse), all meant to consume [`michi`](https://github.com/orin-axi/michi) for agent-facing output rendering (TOON-formatted lists, structured errors, hints) rather than reinventing it. When implementation starts, depend on `michi` directly for CLI/agent output — don't hand-roll formatting that michi already provides.

## Skills

Once implementation starts:

- **`superpowers:test-driven-development`** before writing implementation code for any command or module — the spec's own fixtures (`tests/fixtures/small-ts/`) exist specifically to drive this.
- **`superpowers:systematic-debugging`** before proposing a fix for any test failure or unexpected behavior — don't guess-and-check against an 8000-line spec.
- **`superpowers:brainstorming`** before any design change not already covered by `docs/spec/05-research-and-decisions.md` — this repo has already been through one exhaustive research pass; a new design question deserves the same rigor, not an off-the-cuff call.

## Verification

Before claiming a command or module works: run it against the fixtures in `docs/spec/01-core-architecture.md`'s "Test fixtures" section, don't just confirm it compiles.
