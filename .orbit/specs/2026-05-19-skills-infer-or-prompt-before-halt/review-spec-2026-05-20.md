# Spec Review

**Date:** 2026-05-20
**Reviewer:** Context-separated agent (rally sub-drive, fresh fork unavailable; performed as inline structural review against the spec.yaml + interview.md)
**Spec:** 2026-05-19-skills-infer-or-prompt-before-halt
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals: skill-prose surface + CLI substrate surface | 1 LOW |
| 3 — Adversarial | not triggered | — |

## Findings

### [LOW] D4 `goal_first_line` shape is left under-specified in the spec
**Category:** missing-requirement
**Pass:** 2
**Description:** The interview's D4 says "extend `spec.list`'s response schema to include `goal_first_line` (or truncated `goal`) so the resolver returns prompt-ready labels." The spec.yaml acceptance criteria do not pin the shape. The open-items list in interview.md acknowledges this as "implementation detail, picked at coding time. The contract is 'the menu describes itself.'" — so this is recorded LOW because it is a deliberate latitude, not an omission. Implementation choice should be visible in the verb's docstring and test names.
**Evidence:** interview.md §D4 paragraph; `acceptance_criteria` field has no field-shape constraint.
**Recommendation:** Pick the simplest shape (use `Spec.goal`'s first newline-bounded line via the existing `first_line` helper in `main.rs:1568`) and surface the choice in a docstring on `SpecResolveResult` / `SpecSummary`. No spec change needed — the open-item already records the latitude.

---

## Honest Assessment

The decision pack is unusually well-formed because it landed via rally Stage 3 consolidated review — the five decisions (D1–D5) are deterministic, the disjointness map names files and symbols, and the halt-message templates are fixed strings the implementation must emit verbatim. ACs 01–03 map directly to the three-step recovery (infer / prompt / halt); ac-04 is the uniformity-across-skills check that becomes "all five affected skills call `orbit spec resolve` and emit only the two canonical halt templates." The biggest live risk is symbol collision with sibling drive 0037 in `verbs.rs` / `schema.rs` / `cli/main.rs`, mitigated by the named symbols in the rally brief (`spec_resolve`, `SpecResolveArgs`, `SpecResolveResult`, `goal_first_line`). Ready to implement.
