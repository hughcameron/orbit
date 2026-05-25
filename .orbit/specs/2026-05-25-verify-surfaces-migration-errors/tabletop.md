# Tabletop — Verify surfaces unrunnable schema-version errors

**Date:** 2026-05-25
**Facilitator + domain expert:** Hugh Cameron
**Scribe + driver:** Claude (Opus 4.7)
**Cards in scope:** .orbit/cards/0020-orbit-state.yaml
**Methodology:** Card 0019 — 10-question methodology; choice 0017 — output is contract, not solution
**Output spec:** .orbit/specs/2026-05-25-verify-surfaces-migration-errors/spec.yaml
**Source memo:** .orbit/memos/2026-05-16-verify-swallows-future-version.md

---

## Values

**Load-bearing value: diagnostic honesty.** `orbit verify`'s job is to tell the operator whether the binary can operate on this tree. A green light on an unrunnable tree is the failure of the entire diagnostic surface — every downstream "verify-clean" gate is conditioned on the same lie.

Substrate integrity sits on top of diagnostic honesty: you can't trust the binary to enforce integrity when verify lies about the binary's relationship to the tree.

## Trade-offs

- **Synthetic `RoundTripFailure::ParseFailed` for migration-runner errors** — acceptable. Reuses the existing diagnostics channel; no caller-contract change. Code that already inspects `outcome.round_trip_failures` and exits non-zero keeps its shape.
- **Synthetic error doesn't fully describe the failure** — acceptable. The wrapped message carries the migration runner's underlying error; that's enough for the operator to drill in.
- **A dedicated `VerifyOutcome.migration_failures` field** — rejected as expensive. One edge case doesn't justify surface bloat. Held in reserve as the Q10 pivot.

The cut is the simplest cut that holds diagnostic honesty.

## Halt conditions

- **Regression on known-older-version migration walk.** Trigger: any pre-existing test exercising `verify_all` on a tree with a known-older `schema-version` file turns red. Revert path: `git restore .` on `verify.rs` and re-attempt with a narrower guard (intercept only "unknown version" errors, not generic migration failures).

## Escalation triggers

- **Existing callers grep for specific error-message shapes.** Condition: the new synthetic `RoundTripFailure::ParseFailed` carries a message string that diverges from what an existing CI/script caller expects. Surface: caller name (file:line) + expected vs actual message text + the migration-runner error being wrapped. Action: AUQ author — (a) shape the wrapper message to match existing expectations, (b) document the new shape in the release notes and proceed.

## Kill conditions

- **K1: synthetic-error claim.** If the synthetic `RoundTripFailure::ParseFailed` breaks caller assumptions about what lands in the round-trip-failures channel, the wrapper approach is dead. Pivot: add a dedicated `VerifyOutcome.migration_failures: Vec<MigrationFailure>` field; existing callers continue to inspect `round_trip_failures` only.

## Hot-wash

- **recurred**: memo carried a fix sketch — the tabletop reduced to verifying scope and producing the halt/escalation/kill contract, not exploring shape.
- **surprised**: the trivial-skip advisory should fire here on the letter of the skill; the call to run tabletop anyway is partly substrate hygiene (close the conformance finding via the canonical path) and partly about producing the contract artefact for review-spec.
- **friction**: design-space classification has no "trivial-but-genuine-bug-fix" branch — open/partial/closed all overshoot when the memo already carries the fix sketch.
- **meta-patterns-for-future-tabletops**: when a memo carries a fix sketch and there's no live shape question, the tabletop should default to a compact sidecar (1–3 lines per Q, hot-wash brief) rather than the full methodology walk. Worth a tabletop SKILL.md tightening — a fourth design-space mode "memo-pinned" sitting between closed and partial.
