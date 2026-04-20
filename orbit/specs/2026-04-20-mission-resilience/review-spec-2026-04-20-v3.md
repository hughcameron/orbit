# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** orbit/specs/2026-04-20-mission-resilience/spec.yaml
**Verdict:** APPROVE

---

## Review Depth

```
| Pass                      | Triggered by                                                            | Findings |
|---------------------------|-------------------------------------------------------------------------|----------|
| 1 — Structural scan       | always                                                                  | 0        |
| 2 — Assumption & failure  | content signals (cross-file coordination, schema ownership for card 0003)| 2        |
| 3 — Adversarial           | not triggered (no structural concerns after Pass 2)                     | —        |
```

## Findings

### [LOW] ac-08 does not cover the "gate is the last AC" edge case
**Category:** test-gap
**Pass:** 2
**Description:** ac-08 description says the hook "names the first AC after the gate (if any) that would become startable once the gate closes." The "(if any)" hedge correctly anticipates the degenerate case where a gate AC is the final AC in the list, but the `mrl_ac08_hook_surfaces_next_ac` fixture set only exercises (a) non-gate next AC, (b) gate followed by a non-gate, and (c) all checked. There is no fixture where the first unchecked AC is a gate and is ALSO the last AC in the spec — i.e. no fixture asserts what the hook prints when there is no subsequent AC to name.
**Evidence:** spec.yaml lines 58–60 (ac-08 description and verification fixtures a/b/c); interview Q1/Q5 do not pin the wording for the gate-is-last case.
**Recommendation:** Either (i) add a fixture (d) with a single gate AC [ ] and assert the hook names it as blocking without crashing/mis-printing on the "first AC after the gate" slot, or (ii) accept this as an intentional under-specification and state in the description that "if any" means the slot is silently omitted when absent. Either is acceptable; the risk of leaving it implicit is low because the guard is a simple "if next-AC-exists" branch.

### [LOW] ac-12 fixture (d) ordering regex can match false positives
**Category:** test-gap
**Pass:** 2
**Description:** The ordering assertion regex in mrl_ac12 fixture (d) is `'Backfilled.*\\n.*(?:cannot start|blocking gate|ac-01)'`. `.*` across a newline with `\n` anchoring is permissive: any later line containing "ac-01" (including the backfill line itself being re-logged, or a debug/trace line echoing the AC id) satisfies the pattern. The test proves "Backfilled... appears somewhere before some line mentioning ac-01" rather than strictly "Backfilled... appears before the ac-06 gate-refusal message."
**Evidence:** spec.yaml line 80, verification block for ac-12 fixture (d).
**Recommendation:** Tighten the regex to target the gate-refusal phrasing specifically, e.g. require `cannot start ac-02` or `blocking gate: ac-01` as the second-line match, or split into two assertions (line index of "Backfilled" < line index of gate-refusal). Not blocking — the current regex is weak but still catches the most likely regression (backfill log missing entirely).

---

## Honest Assessment

This spec is ready for implementation. It is the third cycle (v1.2) against the same topic and the metadata's `changes_since` blocks show it has absorbed eight HIGH/MEDIUM/LOW findings from v1.0 and three MEDIUM/LOW findings from v1.1 in disciplined fashion — every prior finding maps to a concrete constraint, AC-description edit, or new fixture. The three-layer architecture (progress.md / session-context.sh / implement/SKILL.md) has clean role separation, the pre-AC sequence (backfill → drift → gate) is pinned, the interactive/non-interactive branching in ac-02 is fully specified with exit-status semantics, and backwards compatibility has its own AC (ac-12) rather than being a footnote. The biggest remaining risk is test-adequacy rather than design: the ac-12 ordering regex is weaker than its prose and the ac-08 "gate-is-last" case has no fixture. Neither is load-bearing for the three-layer contract, and both can be tightened during implementation without touching the design. The spec correctly resists scope creep (rally-level AC visibility deferred, Monitor-tool watching deferred, LLM-judgement rules explicitly rejected in favour of deterministic checks). I couldn't find problems that warrant another design cycle.
