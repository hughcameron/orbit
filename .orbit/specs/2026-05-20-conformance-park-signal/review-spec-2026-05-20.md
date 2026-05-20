# Spec Review

**Date:** 2026-05-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-20-conformance-park-signal
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signal: schema change with backwards-compat surface | 0 |
| 3 — Adversarial | not triggered | — |

Pass 1 deterministic gate-AC description check: all four ACs are gates; all four descriptions are non-empty, not placeholder tokens, and well over 20 characters. PASS.

Pass 2 triggered by a schema-change content signal — the spec touches `Card::FIELDS`, the canonical writer, and the on-disk shape of every card YAML. The serde `deny_unknown_fields` contract and the existing `FIELDS`-drift assertion in `schema.rs` mean the failure mode for a botched extension is loud (parse error or test failure), not silent corruption. Pass 2 surfaces no structural concerns; no Pass 3.

## Findings

### [MINOR] Empty-string validation tested asymmetrically
**Category:** test-gap
**Pass:** 1
**Description:** ac-01 mandates that both `reason` and `until` are non-empty when present. Verification (c) covers `reason: ""` → parse error, but no case covers `until: ""` → parse error. The symmetric guarantee is asserted in the description but only half-tested.
**Evidence:** ac-01 description: *"Both `reason` and `until` are non-empty when present — empty-string is a parse error"*. ac-01 verification enumerates (a)…(f) but only (c) tests empty `reason`; no analogue for `until`.
**Recommendation:** Add a (c2) case to ac-01 verification: `park: {reason: "x", until: ""}` fails to parse with an empty-string error. One-line test addition; closes the symmetry without expanding scope.

---

## Honest Assessment

This plan is ready. The spec is well-scoped — one schema field, one audit carve-out, two doc updates — and each AC ties to a specific file plus a specific test invocation. The riskiest move is the `Card::FIELDS` extension, and the existing drift-assertion infrastructure in `schema.rs` catches that class of mistake at compile/test time. The biggest residual risk is operator-visible rather than implementation-visible: free-form `until:` with no automated unpark means parked cards can outlive their hold condition silently. That's a deliberate v1 scope cut, acknowledged in the interview and the goal, and the remediation is cheap (operator edits YAML). The MINOR finding above is the one fix I'd make before implementation; everything else is solid.
