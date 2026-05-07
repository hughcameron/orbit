# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-20-mission-resilience/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

```
| Pass                       | Triggered by                                                                  | Findings |
|----------------------------|-------------------------------------------------------------------------------|----------|
| 1 — Structural scan        | always                                                                        | 0        |
| 2 — Assumption & failure   | content signals: deployment, cross-system (card 0003), migrations (ac-12)     | 3        |
| 3 — Adversarial            | not triggered — no structural cascades, contradictions, or untestable ACs     | —        |
```

Pass 1 found no structural issues — the spec is unusually tight. All 12 ACs have deterministic verification recipes, constraints are internally consistent, scope maps cleanly onto goal, and the v1.0 → v1.1 revision genuinely addressed each of the eight prior findings. Pass 2 was entered on content signals only (the plugin touches deployment surfaces, a cross-card schema contract, and introduces a backfill migration). Pass 2 surfaced one MEDIUM and two LOW findings, none cascading — Pass 3 was not reached.

---

## Findings

### [MEDIUM] ac-02 behaviour in non-interactive runs is unspecified
**Category:** assumption
**Pass:** 2
**Description:** ac-02 mandates that on drift, the implement skill "calls AskUserQuestion with exactly two options" and "refuses to start the AC until the author's response is received". The spec assumes an interactive author is always present. This repo also ships `/orb:drive` (explicitly a documented skill in this session's environment) whose purpose is to drive cards through the pipeline at a declared autonomy level — i.e. non-interactively or semi-autonomously. If drive (or any future automation) invokes the implement skill and drift is detected, the AskUserQuestion has no responder. Behaviour in that situation is undefined: does the skill block indefinitely, error, pick a default, halt like the Stop response? Each answer has different consequences for autonomous cycles.
**Evidence:** spec.yaml ac-02 (line 27) specifies only the interactive contract. No constraint or AC covers the non-interactive path. The broader codebase has `/orb:drive` (listed in available skills) which would reach this code path. The interview Q3 answer explicitly framed drift handling as a "REQUEST_CHANGES-style notice" without considering the autonomous invocation surface.
**Recommendation:** Either (a) add a constraint that ac-02's AskUserQuestion is only invoked when a TTY / interactive responder is present, and specify the fallback (probably: halt with non-zero exit, leave stale hash, emit drift notice to stderr) for non-interactive runs, or (b) add an AC covering the non-interactive path explicitly. Choice (a) is cheaper; either makes the contract well-defined for drive.

### [LOW] ac-02 missing-hash handling is only stated in ac-03
**Category:** test-gap
**Pass:** 2
**Description:** ac-03 explicitly says the session hook silently skips the drift check when `Spec hash` is absent. ac-12 covers backfill semantics for the implement skill. But ac-02's own description — the implement skill's pre-AC drift check — does not explicitly state what happens when the field is absent on entry. The behaviour is inferable from ac-12 (backfill runs, no drift notice), but the ac-02 contract reads as if "recomputes sha256 and compares to the Spec hash recorded in progress.md" always has a recorded hash to compare to. An implementer reading ac-02 in isolation could write `if hash != recorded: drift_notice()` which would fire a false drift on the legacy path.
**Evidence:** spec.yaml ac-02 (line 27) vs ac-03 (line 32) vs ac-12 (line 77). ac-03 has the explicit "If the Spec hash field is absent … the hook silently skips"; ac-02 does not.
**Recommendation:** Add one sentence to ac-02's description: "If the Spec hash field is absent, defer to ac-12's backfill path instead of emitting the drift notice." Or add a verification sub-case to mrl_ac02 asserting that a progress.md without a Spec hash line does NOT emit the drift notice from the ac-02 code path.

### [LOW] Order of operations between backfill (ac-12) and gate enforcement (ac-06) unspecified
**Category:** missing-requirement
**Pass:** 2
**Description:** When the implement skill encounters a legacy progress.md (no Spec hash) that also has a preceding gate AC unchecked, the spec does not say which check fires first: the ac-12 backfill, the ac-06 gate refusal, or the ac-02 drift flow. In practice these are independent operations (backfill writes a header line; gate check walks the AC list), but their output ordering affects the user-visible log. If gate refusal fires first, the user sees "cannot start ac-02, ac-01 is an open gate" without knowing the hash was just backfilled. If backfill logs first, the user sees "Backfilled Spec hash" and then the gate refusal — clearer causality. Not a correctness defect, but spec silence means two implementers could produce observably different logs and both be conformant.
**Evidence:** ac-06 (line 47) pre-AC checks include only "walks the AC list in declaration order and refuses to start". ac-12 (line 77) covers backfill on the implement skill's pre-AC check but does not sequence it against gate enforcement.
**Recommendation:** Add to ac-12 (or a new constraint) the sequence: the pre-AC check order is (1) backfill if hash missing, (2) drift check if hash present, (3) gate enforcement. This also matches intuition — you cannot decide blocking semantics on a file you have not finished writing.

---

## Honest Assessment

This spec is ready to implement with one non-trivial change (the non-interactive drift path). The v1.1 revision was disciplined — every v1.0 finding was addressed with material changes, not hand-waving, and the ambiguity score (0.10) is earned rather than self-awarded. The three-layer architecture is cleanly decomposed, constraints are sharply worded with trade-offs owned (see the sha256 normalisation constraint, which explicitly accepts cross-platform false drift), and the detour lifecycle rework from v1.0 (atomic + terminal, no open state) removes the biggest class of earlier ambiguity.

The single biggest risk is the non-interactive AskUserQuestion path (MEDIUM 1). Orbit's own `/orb:drive` exists to run cards autonomously; a spec whose drift response is "block on a human prompt" will either freeze drive runs or silently regress to defaults. This is the one finding that could bite in production and is worth pinning down before implementation, not after. The two LOW findings are polish — they make the contract easier to implement conformantly but are not load-bearing.

Two things I deliberately did not flag: (1) the fact that ac-04 and ac-07 partially test the same Current-AC-update behaviour from different angles is redundancy, not a defect — belt-and-braces matches the stated design principle. (2) Exit status 0 on deliberate Stop is a debatable convention but the author chose it explicitly in the v1.1 revision, so it is a decision, not an oversight.
