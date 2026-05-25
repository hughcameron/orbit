# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-port-promote-sh
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (workspace version bump, cross-system PR-sequencing, new core verb on the substrate) | 3 |
| 3 — Adversarial | not triggered — Pass 2 surfaced no cascade or rollback concerns | — |

## What changed since v2

The v2 review (REQUEST_CHANGES, one HIGH + two LOWs) resolved cleanly in this rev:

- **v2 HIGH (AC-11 version pin assumed PR #32 had merged)** — resolved. AC-11 now reads the working-tree `orbit-state/Cargo.toml` version at implement-stage start and bumps by one patch, enumerating both branches (0.4.33 → 0.4.34 if rebased on main, 0.4.34 → 0.4.35 if PR #32 lands first), with an explicit collision clause: "If the workspace version collides with an already-published version on main at PR-creation time, the rebase resolves it". Dynamic-version language matches the v2 review's option (b); the precondition is no longer load-bearing.
- **v2 LOW (AC-08 grep catches prose mentions)** — resolved by author intent. AC-08 explicitly states "six call sites per pre-flight grep: drive 3, card 2, rally 1", matching live `grep -rn 'promote\.sh' plugins/orb/skills/` (6 matches across those three files). The author has chosen the "rewrite prose too" reading; the AC is now internally consistent with the verification grep.
- **v2 LOW (AC-07 path-choice provenance)** — acknowledged as optional/non-blocking in v2; no change required. Letting review-pr read the diff is fine.

The pillar-of-this-rev — AC-11's dynamic version language — is the right shape and removes the live-environment dependency that drove the v2 HIGH.

## Pass 1 gate-AC text check (deterministic rules)

Three gate ACs: `ac-01`, `ac-06`, `ac-08`. All three descriptions are non-empty, not in the placeholder-token set (`TBD/TODO/FIXME/PLACEHOLDER/XXX/???`), and well above the 20-char floor (~700, ~440, ~330 chars respectively). No MEDIUM finding from rule 5.

## Findings

### [MEDIUM] AC-05 names the rejects but not the validation strategy
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-05 specifies that paths containing `..` or pointing outside the layout root are rejected as `Error::malformed`. Two implementation strategies satisfy that text:

1. **Literal `..` substring check** on the raw input string before any filesystem resolution.
2. **Canonicalise then containment check** — resolve the path (following symlinks, collapsing `..`), then verify the result lives under `layout.cards_dir()`.

The two diverge on legitimate inputs like `./.orbit/cards/0005-drive.yaml` (passes both), `.orbit/cards/../cards/0005-drive.yaml` (rejected by 1, accepted by 2 after collapse), and symlinks pointing outside the layout root (accepted by 1, rejected by 2). The shim does `cd "$(dirname "$card_path")" && pwd` — closer to strategy 2 but without the containment check.

The CLI/MCP parity tests in AC-06 will exercise one strategy and bake it in; if review-pr then reads the spec and expects the other, the test asserts a different invariant than the AC text claims.

**Evidence:** AC-05 text says "paths containing `..` or pointing outside the layout root are rejected" but doesn't say *how* the check runs. Shim implementation at `plugins/orb/scripts/promote.sh` lines 71–72 (`card_abs=$(cd "$(dirname "$card_path")" && pwd)/$(basename "$card_path")`) does path resolution but no containment check — neither strategy alone matches the shim verbatim.

**Recommendation:** Pick one and name it in AC-05. Suggested wording:

> ...rejected with `Error::malformed`. Validation: canonicalise the input path via `std::fs::canonicalize`, then verify the resolved path starts with `layout.cards_dir()`. Literal `..` substring rejection is acceptable as a cheaper pre-check but the containment check is the load-bearing one (catches symlinks pointing outside the layout root).

Strategy 2 (canonicalise + containment) is the safer default — it catches symlink escapes which strategy 1 misses. Either is fine; the spec just needs to name one.

### [LOW] AC-04's conflict error verb name will leak `spec.create` unless the implementer wraps
**Category:** test-gap
**Pass:** 2
**Description:** AC-04 specifies the conflict error: `Error::conflict("spec.promote", "spec '<id>' already exists; promote produces fresh specs")`. The natural implementation of `spec.promote` is: derive id → build `SpecCreateArgs` → call `spec_create(layout, &args)`. Today's `spec_create` returns its own conflict shaped as `Error::conflict("spec.create", format!("spec already exists at {}", path.display()))` — verb name `spec.create`, message wording differs.

Two patterns satisfy AC-04 verbatim:

1. **Pre-check**: before calling `spec_create`, run `if layout.spec_file(&id).exists() { return Err(Error::conflict("spec.promote", ...)) }`. Inner `spec_create` never sees the conflict path.
2. **Catch-and-rewrap**: call `spec_create`; on `ErrorCode::Conflict`, rewrap with `verb = "spec.promote"` and the message AC-04 specifies.

Pattern 1 is the cleaner shape (matches the pre-check pattern used elsewhere in the codebase, e.g. `spec_close`) but introduces a TOCTOU window between the `.exists()` check and `spec_create`'s own write. Pattern 2 closes the race but adds error-rewriting boilerplate.

Without guidance, the implementer may pick pattern 2 by default ("don't duplicate the check") and skip the message rewording. AC-06's parity tests assert envelope shape but the AC text on the error message ("spec '<id>' already exists; promote produces fresh specs") is more specific than the inner `spec_create`'s message — the test will catch it but the implementer wastes a round trip.

**Evidence:** `orbit-state/crates/core/src/verbs.rs:1830-1834` (current `spec_create` conflict shape):

```rust
return Err(Error::conflict(
    VERB,                                              // "spec.create"
    format!("spec already exists at {}", path.display()),
));
```

AC-04 specifies the verb as `"spec.promote"` and the message as the literal `"spec '<id>' already exists; promote produces fresh specs"`.

**Recommendation:** Add one sentence to AC-04 naming the pattern. Suggested addition:

> Implementation note: pre-check via `layout.spec_file(&id).exists()` before delegating to `spec_create`, so the conflict error carries `verb = "spec.promote"` directly. Lock acquisition inside the verb closes the TOCTOU window.

Or accept the wrap-and-rewrite path — either works, but name it.

### [LOW] AC-06 doesn't enumerate the dry-run-over-existing-target parity case
**Category:** test-gap
**Pass:** 2
**Description:** AC-04 second sentence carries a sharp behavioural contract: "The `--dry-run` path stays read-only and succeeds even when the target already exists (it reports what WOULD be written, not what is)." That is a divergence from the non-dry-run path's behaviour (which errors on conflict per AC-04 first sentence).

AC-06's parity tests assert (a) byte-equal envelope for a fresh fixture and (b) dry-run identical-envelope and no-side-effect for the same fixture. Neither test exercises dry-run when the target spec already exists. A buggy implementation could:

- Check existence unconditionally and error in dry-run too (violates AC-04 second sentence).
- Skip the existence check in dry-run but emit a different envelope than non-dry-run would have (envelope mismatch).
- Succeed silently in dry-run while leaving a partial write behind on the existence-check path (side-effect leak).

None of these are caught by AC-06 as written.

**Evidence:** AC-06 text — "byte-equal envelope assertion ... against a shared fixture card (mixed gate + non-gate scenarios), plus a separate `--dry-run` parity test asserting no-side-effect (layout snapshot equality before/after) and identical envelope between dry-run and non-dry-run paths for the same fixture". No mention of pre-existing-target case.

**Recommendation:** Extend AC-06 with one more parity-test case. Suggested addition:

> Plus a third parity test: against a fixture where a spec at the derived id already exists, assert (a) dry-run succeeds with the planned envelope, (b) layout snapshot is byte-equal before and after dry-run, and (c) non-dry-run errors with `Error::conflict("spec.promote", ...)` per AC-04.

This is the smallest test that pins AC-04's second sentence to the wire.

---

## Honest Assessment

This rev is one structural-clarification away from APPROVE. The version-sequencing HIGH that dominated v2 is genuinely resolved — the new AC-11 reads the working-tree version at implement-time and bumps from there, with the collision clause naming the rebase as the resolver. That removes the live-environment dependency that made v2 unimplementable safely.

What remains is a single MEDIUM (AC-05's validation strategy is undernamed — pick canonicalise-plus-containment, or pick literal-`..`-reject, but pick one) and two LOWs that tighten the test surface (the `spec.create` → `spec.promote` error-verb leak and the missing dry-run-over-existing-target parity case). All three are pure spec-text edits, no design rework needed.

Everything else — the verb surface, the dry-run contract, the wrapper-or-delete choice in AC-07, the deferred-relations posture in the goal, the CLI bare-id-stdout contract preserving drive's shell-capture shape — is well-formed and matches the port-acceptance-shim precedent.

The cheapest path to APPROVE: pick a validation strategy for AC-05, add the implementation-note sentence to AC-04, append the third parity-test case to AC-06. Three small edits, no scope change.
