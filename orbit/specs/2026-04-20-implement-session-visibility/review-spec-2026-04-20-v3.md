# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** orbit/specs/2026-04-20-implement-session-visibility/spec.yaml
**Verdict:** APPROVE

---

## Review Depth

```
| Pass                         | Triggered by                                                                 | Findings |
|------------------------------|------------------------------------------------------------------------------|----------|
| 1 — Structural scan          | always                                                                       | 0        |
| 2 — Assumption & failure     | content signals: cross-system boundaries (shared parser, session-context.sh co-owned with card 0009), backwards compatibility (depends_on 0009), byte-identity claims | 0 actionable |
| 3 — Adversarial              | not triggered (no structural concerns surfaced in Pass 2)                    | —        |
```

---

## Findings

No findings at MEDIUM or higher severity. A few low-severity observations are recorded below as non-blocking notes for the implementer; none require spec changes before implementation begins.

### [LOW — NOTE ONLY] TaskCreate failure mid-loop in §4d
**Category:** failure-mode
**Pass:** 2
**Description:** The spec does not state what happens if a TaskCreate call errors partway through the §4d emission loop (e.g., TaskCreate for constraints 1–2 succeeds, TaskCreate for constraint 3 fails). The partial task list would be inconsistent with progress.md until the next resume reconcile corrects it.
**Evidence:** Constraint #12 specifies sequencing but not partial-failure recovery. ac-01 verification assumes success of all 7 TaskCreate calls.
**Recommendation:** Not blocking — resume reconcile (ac-03) would detect and rebuild on the next session start. Implementer should ensure the loop surfaces an error rather than silently continuing with a partial task list, but spec-level guarantees are adequate as written.

### [LOW — NOTE ONLY] Multi-AC checkbox flips in a single Edit
**Category:** test-gap
**Pass:** 2
**Description:** ac-02 rule text is phrased per-item ("when you mark an item `- [x]`"), which covers the common single-flip case. If the agent batches multiple `- [x]` flips in a single Edit tool call, the rule implies N TaskUpdates in the same turn, but this is not made explicit. Verification (b) only tests the single-flip case.
**Evidence:** ac-02 description; verification (b) uses "marking ac-02 complete" (singular).
**Recommendation:** Not blocking — the per-item phrasing is consistent, and the common case is the expected operational pattern. Implementer should confirm the SKILL.md §5 text reads naturally for the multi-flip case.

### [LOW — NOTE ONLY] AskUserQuestion during an active Monitor stream
**Category:** assumption
**Pass:** 2
**Description:** ac-06 interactive path calls AskUserQuestion in response to a streamed Monitor failure line. The spec does not explicitly address whether pausing for AskUserQuestion while Monitor continues emitting events is supported without deadlock or event loss.
**Evidence:** ac-06 description; constraint #9.
**Recommendation:** Not blocking — this mirrors card 0009 ac-02's AskUserQuestion discipline which has presumably been vetted. Implementer should verify by running the non-interactive path first (which is fully mechanical) and interactive path second.

---

## Honest Assessment

This plan is ready. v1.2 addresses every finding from v1.0 and v1.1 with traceable change notes and concrete contract strengthening (byte-identity sha256 assertions, structural grep for awk/sed elimination, interactive/non-interactive split mirroring 0009 ac-02, single-source canonical constants for both the resume-rebuild warning and the non-interactive first-failure marker). The coupling to card 0009 is handled explicitly — depends_on pinned, shared parser ownership moved into this card (ac-08) with a first-party refactor deliverable, §4d sequencing pinned relative to 0009's pre-AC check sequence (constraint #12/#13), and the non-interactive halt integration wired through ac-03(d).

The biggest remaining risk is execution-surface, not spec-surface: the §4d task-emission loop + resume reconcile + shared-parser refactor all land in files co-owned with card 0009, and they must merge after 0009 without reintroducing the inlined awk block. The structural assertion in ac-08(e) (`grep -nE '\bawk\b|\bsed\b'` returning zero hits in both reconcile and next-AC-surfacing blocks) is a strong check that catches a regression, and the byte-identity sha256 check in ac-07(d) defends the "byte-identical §1–§4c" wording. Both are the right-shaped guards.

Three low-severity notes above are implementation-level quality observations, not spec defects — they do not require changes before implementation and can be addressed inside the skill text and tests as the implementer encounters them.
