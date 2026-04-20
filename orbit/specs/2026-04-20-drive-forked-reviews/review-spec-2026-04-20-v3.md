# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** /home/hugh/github/hughcameron/orbit/specs/2026-04-20-drive-forked-reviews/spec.yaml
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 structural, content signals present |
| 2 — Assumption & failure | content signals (cross-system boundaries, ontology/schema change, migration discipline) | 2 (LOW) |
| 3 — Adversarial | not triggered — Pass 2 revealed no structural concerns | — |

---

## Findings

### [LOW] Cycle-ordinal path leaks iteration count to forked reviewer
**Category:** assumption
**Pass:** 2
**Description:** ac-08 requires re-review briefs to be "functionally identical to the first forked review's brief" — no path to prior review files, no iteration counter, no summary of prior findings. However, ac-19 requires drive to write cycles 2 and 3 to `review-<stage>-<date>-v2.md` and `-v3.md` respectively, and that output path is part of the brief drive hands to the fork (ac-04/ac-05). An attentive forked reviewer reading its own output-path argument can infer "I am the 3rd cycle" from the `-v3` suffix. In strict terms this contradicts the cold-re-review intent of Q3 / ac-08.
**Evidence:** spec.yaml ac-08 ("The brief is functionally identical to the first forked review's brief"); ac-19 ("Drive computes the target path deterministically … the Nth review for a stage (1-indexed) uses suffix `-v<N>` for N>1"); interview Q3 ("Each re-review is an independent context-separated read. The new reviewer doesn't know it's pass 2 or 3").
**Recommendation:** Either (a) accept this as a design trade-off and note explicitly in ac-08 that the path suffix is the one permitted leak (justified by the on-disk disambiguation requirement and the impracticality of forked-agent-blind output paths), or (b) have drive pass a fixed path to the fork (e.g. `review-<stage>-<date>.pending.md`) and rename after verdict parse, keeping the cycle suffix invisible to the reviewer. Option (a) is the lower-friction choice; a one-line acknowledgement in ac-08's description would close the gap honestly.

### [LOW] `review_cycle_dates` write atomicity not specified
**Category:** failure-mode
**Pass:** 2
**Description:** The ontology defines `review_cycle_dates.<stage>` as "written when drive enters review-spec for the first time in the iteration; not updated on subsequent cycles" (ac-19). The spec does not state whether `review_cycles.<stage>` and `review_cycle_dates.<stage>` are written in the same drive.yaml update, nor what drive should do on resumption if one is present and the other is missing (e.g. crash mid-write). The happy-path resumption logic (ac-07, ac-11, ac-22) assumes both fields are present and consistent.
**Evidence:** spec.yaml ontology_schema review_cycle_dates description; ac-19 ("drive persists the captured date in drive.yaml alongside `review_cycles`"). No AC covers the partial-write recovery case.
**Recommendation:** Add a short clause to ac-19 or ac-22 stating that drive writes `review_cycles.<stage>` and `review_cycle_dates.<stage>` atomically (single drive.yaml write), and that on resumption, if `review_cycles.<stage> > 0` but `review_cycle_dates.<stage>` is absent, drive treats the state as corrupt and exits with a clear recovery message (same class as ac-20's refusal). This is a small rough-edge closure, not a structural concern.

---

## Honest Assessment

This is a carefully-constructed spec with unusually tight traceability between interview decisions, constraints, ACs, and exit conditions. The verdict contract is pinned to a byte-level regex, the fork-invocation brief discipline is explicit, the REQUEST_CHANGES budget is persisted to survive session death, and the migration strategy is deliberately simple (refuse-on-absence rather than auto-upgrade). Every Q from the interview maps to concrete ACs; every AC has a reproducible verification method. The two findings are minor rough edges, not structural problems — one is a philosophical leak in the cold-re-review contract (the `-v<N>` suffix is visible to the reviewer), the other is a missing clause about atomic drive.yaml writes for the `review_cycles` + `review_cycle_dates` pair. Both can be addressed during implementation without touching the design. The biggest real risk is that the Agent tool's behaviour with a skill's `context: fork` frontmatter will not match what the spec assumes at implementation time — but that risk sits outside the spec and is verifiable at ac-04/ac-05 inspection. Ready to implement.
