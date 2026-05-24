# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-24-port-acceptance-shim
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-surface CLI/MCP parity, version bump, CHANGELOG entry, 31 call-site rewrites) | 0 |
| 3 — Adversarial | not triggered | — |

## Findings

None blocking. Notes on the v1 → v2 deltas and one non-blocking session-management observation follow.

---

### [INFO] v1 HIGH findings cleanly addressed

**Category:** missing-requirement
**Pass:** 1
**Description:** v1 (`review-spec-2026-05-24.md`) raised two HIGH findings and one MEDIUM. All three are resolved in the current spec text:
- **AC-05** now names `spec_check` as a first-class core verb in `orbit-state/crates/core/src/verbs.rs`, citing the exact CLI ladder it replaces (`crates/cli/src/main.rs:1037-1080`). Substrate read confirms the cited range houses the `(None, None) | (Some(_), Some(_))` mutual-exclusion guard, the AC-existence lookup, and the `already {state}` → `Error::conflict` arm. The previously open question about `--ac-uncheck` is closed: AC-05 ships symmetric `spec_uncheck` in the same change.
- **AC-06** explicitly splits the gate-axis helpers from `spec_close`'s taxonomy-axis pre-flight and pins the no-change invariant on `verbs.rs:1980-1990`. Substrate read confirms that range filters on `!ac.checked && ac.ac_type.blocks_close()` (and uses `gate` only for error-suffix annotation), exactly as the AC asserts. The "force-share" temptation that v1 warned would burn half a session is closed off in prose.
- **AC-08** now states the wrapper step "MAY be skipped" with AC-10's no-orphaned-wrapper invariant holding either way — authorising both the wrapper-first and rewrite-then-delete orderings without ambiguity.

**Evidence:**
- `orbit-state/crates/cli/src/main.rs:1037-1080` — verified the AC-check/-uncheck ladder lives at the cited range.
- `orbit-state/crates/core/src/verbs.rs:1866-1908` — verified `spec_update` is a full-list replacement primitive (no per-AC semantics), matching the gap AC-05 now fills.
- `orbit-state/crates/core/src/verbs.rs:1980-1990` — verified the `ac_type.blocks_close()` filter and confirmed `gate` participates only in the error suffix, matching AC-06's "no behaviour change to close" claim.

**Recommendation:** None — informational.

---

### [INFO] v1 LOW-1 (missing-id case) addressed

**Category:** test-gap
**Pass:** 1
**Description:** AC-01 now closes with "Missing spec id returns `Error::not_found` (same shape as `spec_show`)" — picks up v1's recommended one-liner directly. The parallel test pattern (`spec_show_missing_id_is_not_found` at `verbs.rs:6619`) is the obvious template for the new verbs' parity coverage under AC-07.

**Recommendation:** None — informational.

---

### [INFO] AC-08 one-shot path implies a single large PR — session-management note, not a spec defect

**Category:** content-signal
**Pass:** 2
**Description:** AC-08's "MAY skip wrapper" clause authorises landing verb additions + parity tests + 31 SKILL.md rewrites (across 5 files: `audit`, `drive`, `implement`, `review-pr`, `review-spec`) + shim deletion + `test-gate-ac-verification.sh` deletion + choice 0020 update + CHANGELOG entry in a single commit. That is a large diff, but every piece of it is mechanical and grep-verifiable, and the alternative (wrapper-first) is explicitly preserved. The spec correctly leaves the cut to implementer discretion. Calling this out so the implement agent goes in eyes open on commit size rather than discovering it mid-pipeline.

**Evidence:**
- 31 call sites confirmed: `rg -n 'orbit-acceptance\.sh' plugins/orb/skills/` returns 31 lines across `audit/SKILL.md` (2), `drive/SKILL.md` (9), `implement/SKILL.md` (16), `review-pr/SKILL.md` (1), `review-spec/SKILL.md` (3).
- The `review-spec/SKILL.md` line at 63 (Pass-1 gate-AC description check) cites `orbit-acceptance.sh acs` by name inside prose explaining where column data comes from — rewrite is a search-and-replace of the command, no logic change.

**Recommendation:** None blocking. If the implementer takes the one-shot path, sequence the commit as: (1) core verbs + parity tests landing green, (2) CLI sugar reroute, (3) SKILL.md sweep, (4) shim + test deletion, (5) choice 0020 update + CHANGELOG — all in one PR but ordered so a partial revert is mechanical.

---

## Gate-AC Description Check (Pass 1 deterministic rule)

Gate ACs in the spec (per parser `is_gate=1`): AC-01, AC-06, AC-07, AC-09.

| AC | Length | Placeholder? | Non-empty? | Verdict |
|----|--------|--------------|------------|---------|
| AC-01 | 384 chars | no | yes | pass |
| AC-06 | 506 chars | no | yes | pass |
| AC-07 | 196 chars | no | yes | pass |
| AC-09 | 178 chars | no | yes | pass |

No gate-AC structural violations.

---

## Honest Assessment

This is a clean v2. Both HIGH findings from v1 were addressed by editing the AC text against the substrate the v1 reviewer cited — AC-05 now names `spec_check` as a first-class core verb (with the right file and line range), and AC-06 explicitly disavows the helper-share with `spec_close` that v1 had warned would mislead the implement agent. The MEDIUM (wrapper ordering) and LOW (missing-id case) are closed too. The biggest remaining risk is purely session-management: an implementer who takes AC-08's one-shot path lands a large but mechanical PR — explicitly authorised by the spec, just worth knowing going in. Ready to drive.
