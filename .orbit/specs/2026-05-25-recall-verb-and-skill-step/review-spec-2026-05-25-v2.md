# Spec Review — 2026-05-25-recall-verb-and-skill-step (cycle 2)

**Date:** 2026-05-25
**Reviewer:** Claude (Sonnet 4.6)
**Spec:** `.orbit/specs/2026-05-25-recall-verb-and-skill-step/spec.yaml`
**Cycle:** 2 (cycle 1 verdict: REQUEST_CHANGES — HIGH wire-verb-name + MEDIUM zero-match-normalisation)

**Verdict:** APPROVE

---

## Cycle-1 findings — resolution check

### HIGH: wire verb name `recall` → `substrate.recall` (ac-01, ac-03)

Resolved. ac-01 now names the wire verb explicitly:

> "Wire verb name (for the MCP envelope and the canonical-CLI/MCP parity test) is `substrate.recall`, following the existing namespaced-verb convention (`spec.*`, `memory.*`, `substrate.classify`, etc.)."

ac-03 carries the same name in the envelope clause:

> "verb name `substrate.recall`, ok/data/result wrapper, `result.matches[]` carries the tuples"

The CLI surface (`orbit recall <topic>`) is correctly kept separate from the wire verb. MCP routing is unambiguous. No residual gap.

### MEDIUM: zero-match normalisation unspecified (ac-01)

Resolved — and the prose exceeds the minimum. ac-01 now reads:

> "Zero-match behaviour: when a type returns no per-type matches, no tuples for that type appear in the merged result — the type is simply absent. Min-max normalisation does not divide by zero because it operates only on the populated per-type result set. A type with zero matches contributes zero rows; the overall result may have fewer than five types represented."

This eliminates the test-fixture ambiguity and gives the implementing agent the exact assertion shape for the empty-type case.

---

## New findings from the edits

None. The two amendments are narrow prose additions that do not introduce new ambiguity, contradict adjacent ACs, or expand scope.

---

## AC summary

| AC    | Gate | Type   | Status  |
|-------|------|--------|---------|
| ac-01 | yes  | code   | APPROVE |
| ac-02 | yes  | code   | APPROVE |
| ac-03 | yes  | code   | APPROVE |
| ac-04 | yes  | code   | APPROVE |
| ac-05 | yes  | doc    | APPROVE |
| ac-06 | no   | code   | APPROVE |

All six ACs are gradeable, correctly typed, and carry sufficient verification clauses. Test prefix `rcall` is clean across all code ACs.

---

## Ready for implementation

The spec is implementable as written. Implementing agent should read the tabletop sidecar (halt conditions H1–H3, escalation triggers E1–E3) before starting — in particular E1 (confusing merged-result ranking → AUQ before writing the merge step) and E2 (spec+memo search budget → AUQ if either path exceeds 200 LoC).
