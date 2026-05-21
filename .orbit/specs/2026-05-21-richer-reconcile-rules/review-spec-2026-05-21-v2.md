# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-richer-reconcile-rules
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | Pass 1 finding + content signals (cross-system validation, data migration) | 1 |
| 3 — Adversarial | Pass 2 surfaced an ac-05 baseline-composition risk worth cascade-checking | 0 |

## Findings

### [LOW] ac-05 baseline composition may not be fully addressable by the two new rules

**Category:** assumption
**Pass:** 2, 3
**Description:** ac-05 demands "zero `parse_failed` entries" after `--reconcile` runs against the `meridian-online/finetype` checkout (54-spec baseline). The triggering memo (`.orbit/memos/2026-05-16-richer-reconcile-rules.md`) decomposes the 54 as "~36 specs miss `id`" and "~5 specs trip on scalar-AC" — 36 + 5 = 41, leaving ~13 specs unaccounted-for under either rule. Overlap-counting (specs that trip both rules) is one explanation; the other is that the 54 baseline contains drift shapes outside the two rules' reach (other missing required fields, malformed values, unknown enum values).

**Evidence:**
- Memo line 8: "325 ACs across 54 spec.yaml files"
- Memo line 10: "pre-orbit-state specs lack `id` (~36 specs)"
- Memo line 16: "The typed parse rejects scalar entries..." (~5 specs per memo's framing)
- ac-05 description: "zero `parse_failed` entries after the migration" — leaves no slack for residue from drift outside the spec's reach
- The structural-NO-GO path in ac-05 covers only "validation-set repo not locally reachable" — not "baseline contains drift the spec doesn't intend to cover"

**Recommendation:** Two reasonable paths, pick one — don't block on it.

(a) Soften ac-05's target from "zero `parse_failed`" to "every `parse_failed` entry whose drift shape is one of {missing-id, scalar-ac, both} reconciles cleanly; residue is recorded in `progress.md` with the drift shape that defeated reconcile and is treated as input for a follow-up spec". This makes ac-05 closable against an unknown-composition baseline while still proving the two new rules work end-to-end.

(b) Leave the target at "zero" and accept that ac-05 may surface a follow-up spec against card 0032 if residue exists. The system degrades gracefully because ac-04's breadcrumb fires on whatever residue remains — the failure mode is recoverable, not catastrophic.

The spec is implementable either way. The risk is at close-time, not implement-time.

### [LOW] Fixture-tree pattern differs from existing inline-yaml test pattern

**Category:** missing-requirement
**Pass:** 1
**Description:** ac-01, ac-02, ac-03 lock fixture paths as `orbit-state/crates/core/tests/fixtures/reconcile/<name>/spec.yaml` (e.g. `tests/fixtures/reconcile/missing-id/spec.yaml`). The current reconcile test pattern in `orbit-state/crates/core/src/reconcile.rs:885-1126` uses inline yaml strings written to a temp `OrbitLayout` via `write_spec(...)`; no `tests/fixtures/...` directory exists under `crates/core/` and no integration-test pattern reads from disk. ac-07's "(or equivalent)" hedge gives the implementing agent permission to deviate; ac-01/02/03 don't carry the same hedge.

**Evidence:**
- `find orbit-state/crates/core/tests` returns no match (no integration-test directory in the core crate)
- `grep "include_str\|include_bytes\|fixtures" reconcile.rs` returns no match
- All ~15 existing reconcile tests build yaml as `let yaml = "..."` and pass to `write_spec(...)`

**Recommendation:** Either add the "(or equivalent)" hedge to ac-01/02/03 (one-word edit per AC), or change the fixture-path framing to "fixture content under `<test path>`" so the implementing agent can keep the inline-yaml pattern and the test still satisfies the AC. Not blocking — the implementing agent will sensibly fall back to the inline pattern and the test will close the AC's intent regardless. Flag for awareness.

---

## Honest Assessment

The spec is ready. The v1 review's four findings (HIGH wrong line cite for AC-04, HIGH list-element scope, MEDIUM pre-walk synthesise phase, LOW design-note drift) have all landed cleanly: AC-04 cites `cli/src/main.rs:689-708 (run_canonicalise)` and `742-774 (run_reconcile)` — verified accurate against the current tree; ac-07 explicitly names both registry-shape extensions as named load-bearing scope; the design note implementation notes block now reads consistently with the AC text. The spec correctly identifies that ac-01 and ac-02 ride on ac-07's shape extensions and orders the dependency in prose.

The remaining risks are LOW and don't block implementation:
- ac-05's "zero parse_failed" target is hostage to baseline composition the memo only partially decomposes — but ac-04's breadcrumb absorbs the residue gracefully if it appears
- the fixture-tree pattern is novel for this crate, but the implementing agent can sensibly adapt or extend with no AC re-interpretation needed

Biggest risk: ac-05 close-time disposition if the 54 baseline contains drift outside the two rules. The author may want to land path (a) above before implementation starts — it's a 10-second AC edit and avoids a close-time arbitration. APPROVE either way: the implementing agent can read residue-on-baseline as a structural NO-GO under `/orb:drive`'s halt-temptation guard if needed, and the system breadcrumb keeps the agent on the substrate's rails.
