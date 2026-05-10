# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-20-implement-session-visibility/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-card shared surface: SKILL.md, session-context.sh, progress.md schema); also previous REQUEST_CHANGES cycle with v1.1 → v1.1 delta list | 3 |
| 3 — Adversarial | not triggered | — |

---

## Findings

### [MEDIUM] ac-08(e) couples to a private implementation marker in card 0009 that 0009's spec does not guarantee
**Category:** assumption
**Pass:** 2
**Description:** ac-08 verification (e) asserts `grep -c 'inside ## Acceptance Criteria' plugins/orb/scripts/session-context.sh` MUST be 0 after merge. This is framed as proof that the inlined awk block has been removed. However, the canonical string `inside ## Acceptance Criteria` is not declared anywhere in card 0009's spec (see `.orbit/specs/2026-04-20-mission-resilience/spec.yaml` ac-09 and the parser-discipline constraint — neither pins that literal marker text). The test therefore depends on an implementation artefact of 0009's as-yet-unmerged code, not on a contract 0009's spec fixes.
**Evidence:** spec.yaml ac-08 verification item (e): "`grep -c 'inside ## Acceptance Criteria' plugins/orb/scripts/session-context.sh` MUST be 0". 0009 spec.yaml ac-09 merely requires "the parser ... ignores ## Detours content when determining AC status". No literal string `inside ## Acceptance Criteria` is a declared 0009 contract.
**Recommendation:** Replace the brittle marker-grep with a structural assertion: e.g. "`grep -nE 'awk|sed' plugins/orb/scripts/session-context.sh` returns zero hits inside the next-AC-surfacing code path, and the script sources `parse-progress.sh`." Alternatively, negotiate a specific marker comment into 0009's spec so this card can grep for a 0009-contracted token rather than an incidental one. Either way, the test should assert (i) the helper is sourced, (ii) no awk/inline parsing remains in the relevant function — not a specific prose comment 0009 never promised.

### [MEDIUM] First-failure checkpoint (ac-06) has no defined non-interactive behaviour
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-06 mandates that on the first Monitor failure line the agent "MUST pause ... and call AskUserQuestion with exactly two options". Card 0009 ac-02 (whose schema this card depends on) already established that AskUserQuestion is not available under `/orb:drive` or autonomous harnesses (detected via TTY + `ORBIT_NONINTERACTIVE=1`) and defined a non-interactive branch (emit to stderr, exit 1). This card's first-failure rule does not mention the non-interactive path — under `/orb:drive`, what does the agent do on a Monitor failure? If it calls AskUserQuestion anyway, it collides with 0009's established precedent; if it falls through silently, the "react mid-run" value is lost; if it halts, that needs to be said. Ambiguity here is load-bearing because drive is the primary autonomous consumer of `/orb:implement`.
**Evidence:** spec.yaml ac-06 description: "the agent MUST pause mid-run, acknowledge the failure, and call AskUserQuestion with exactly two options". No non-interactive branch described. Contrast 0009 spec.yaml constraint on ac-02: "ac-02's AskUserQuestion drift-acknowledgement path is only invoked when an interactive responder is available ... In non-interactive runs ... the skill MUST NOT call AskUserQuestion." This card inherits 0009's schema but not its interactivity discipline for the new AskUserQuestion surface it introduces.
**Recommendation:** Add a constraint mirroring 0009's interactivity rule for the first-failure checkpoint. Concretely: in non-interactive mode (no TTY on stdin OR `ORBIT_NONINTERACTIVE=1`), skip AskUserQuestion, emit the failure line plus a canonical non-interactive marker to stderr, and halt with a defined exit status (so drive can route it to a checkpoint). Extend ac-06 verification with a non-interactive fixture asserting AskUserQuestion was NOT called and the agent halted cleanly.

### [LOW] ac-07 "byte-identical" claim weakened to line-count check in verification
**Category:** test-gap
**Pass:** 2
**Description:** Constraint #1 and ac-07 description both assert that §1, §2, §3, §4a, §4b, §4c "remain byte-identical" to the post-0009-merge baseline. But ac-07 verification (d) only specifies "a line-count check on the unchanged sections". A line-count check is strictly weaker than a byte-identity check — subtle re-wordings that preserve line count would pass the test while violating the stated constraint.
**Evidence:** spec.yaml constraint #1: "must continue to work byte-identically. Any change to those paths is a regression." ac-07 description: "§1–§3 presentation flow and §4a–§4c template remain byte-identical to the shipped skill". ac-07 verification (d): "Assert via a line-count check on the unchanged sections relative to the post-0009-merge baseline."
**Recommendation:** Either (a) use a byte-identity assertion (e.g. `diff` or `sha256sum` of the extracted section ranges against the post-0009 baseline, asserting exact equality), or (b) explicitly relax the contract to "no semantic change to §1–§4c" and document the line-count check as the deliberately weaker gate. Byte-identity for a prose document is a strong commitment that the chosen test cannot defend; pick a test that matches the claim.

---

## Honest Assessment

This is a mature spec on its second cycle. The v1.0 → v1.1 changes (shared-parser ownership pulled into this card, §4d sequencing pinned, `TaskUpdate status: cancelled` disposal primitive, canonical warning constant, ac-01 verification hardened) all land cleanly, and the depends_on contract against 0009 is internally consistent with 0009's current v1.2 spec. The biggest residual risk is cross-card fragility: ac-08(e) tests for a string that is an artefact of 0009's implementation rather than a 0009 contract, and ac-06 inherits an interaction model (AskUserQuestion under drive) that 0009 already gated with an interactivity escape hatch this card forgot to mirror. Neither is a design error — both are stitching gaps where two rally cards meet. The byte-identity vs line-count mismatch is cosmetic but worth tightening so the test defends what the spec claims. None of this rises to BLOCK; the plan is implementable, but let's close the seams before the implement loop starts.
