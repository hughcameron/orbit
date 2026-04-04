# Spec Review

**Date:** 2026-04-04
**Reviewer:** Context-separated agent (fresh session)
**Spec:** specs/2026-04-04-distill/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Findings

### [HIGH] No specification of the "edit" flow mechanics
**Category:** missing-requirement
**Description:** AC-05 says "Choose edit on a candidate, modify the feature name. Verify the saved YAML reflects the edited name." The spec never defines what the edit interaction looks like. Does the user get the full YAML to rewrite? A re-interview on specific fields? A free-text instruction like "change the feature name to X"? The SKILL.md deliverable has no guidance on this, and each interpretation leads to very different implementations.
**Evidence:** Constraint 5 says "approve/edit/reject" but only approve and reject have obvious semantics. AC-05's verification tests only a feature name change, which is the simplest case -- it doesn't cover editing scenarios, adding references, or restructuring.
**Recommendation:** Add a constraint or AC clarifying the edit UX. Suggested: present the card YAML, let the user provide free-text instructions, re-present the modified card for a second approve/edit/reject cycle. Define whether edit loops are bounded or infinite.

### [HIGH] AC-08 verification is subjective and non-deterministic
**Category:** test-gap
**Description:** AC-08 ("Scenarios in extracted cards are grounded in the source material") has a verification method of "Compare extracted card scenarios against the source document. Every scenario should trace to a specific passage in the source." This is a manual, subjective judgment. There is no programmatic check, no traceability format, and no definition of what "trace to a specific passage" means.
**Evidence:** Spec line 77: "Every scenario should trace to a specific passage in the source." This directly contradicts the CLAUDE.md engineering principle: "LLMs for parsing, programmatic checks for validation." The verification relies on a human (or LLM) judgment call with no deterministic gate.
**Recommendation:** Either (a) require distill to emit a `source_passage` field per scenario that quotes or cites the originating text, making traceability mechanically verifiable, or (b) accept that this is a human review gate and rewrite the verification accordingly. Option (a) is strongly preferred.

### [MEDIUM] No error handling for invalid or empty input
**Category:** missing-requirement
**Description:** The spec does not address what happens when the input file does not exist, is empty, is binary, or contains no identifiable features. There are no ACs covering any error path.
**Evidence:** The card's scenarios (lines 7-25) all assume valid, feature-rich input. The spec's ACs (ac-01 through ac-08) all assume the happy path. Zero error paths are specified.
**Recommendation:** Add at least two ACs: (1) file not found or unreadable produces a clear error message; (2) file with no identifiable features produces a "no features found" message rather than hallucinated cards.

### [MEDIUM] Dependency on memos spec is declared but not verified
**Category:** assumption
**Description:** The spec declares a dependency on `specs/2026-04-04-memos/spec.yaml` (line 103-106), but the memos spec is risk-tier SKIP with no review gate. If memos changes its directory convention (e.g., from `cards/memos/` to something else), distill silently breaks. There is also a circular coupling: memos ac-04 references distill, and distill depends on memos.
**Evidence:** Memos spec line 14-15 defines the convention. Distill spec line 61 hardcodes `cards/memos/2026-04-04-search-ux.md` in AC-06's verification. If memos is not yet implemented when distill is built, the verification cannot run.
**Recommendation:** Clarify implementation ordering. Add a precondition to the exit conditions: "memos deliverables must exist before distill verification begins." Consider whether the circular reference between the two specs should be broken.

### [MEDIUM] Card numbering race condition unaddressed
**Category:** failure-mode
**Description:** AC-03 specifies sequential numbering (0004, 0005). The spec says cards are presented one-by-one. If the user approves card 1 (written as 0004), then during the review of card 2, another process or session creates 0005, distill would create a duplicate 0005.
**Evidence:** AC-03 verification (line 40-41) assumes exclusive access to the cards/ directory. The `/orb:card` skill (SKILL.md line 26) says "Find the highest existing NNNN-*.yaml number and increment by 1" -- this is a read-then-write pattern with no locking.
**Recommendation:** Either (a) document this as a known limitation (single-user workflow, acceptable), or (b) specify that numbering is determined at write time, not at extraction time. Given this is a personal workflow tool, (a) is probably fine, but it should be stated.

### [MEDIUM] No specification of what "AskUserQuestion" options look like for approve/edit/reject
**Category:** missing-requirement
**Description:** AC-02 says "each showing one card and offering approve/edit/reject." The spec doesn't define the presentation format. Should it show the raw YAML? A formatted summary? The card skill uses an interactive interview; distill needs a different presentation pattern (showing a proposed card, not asking questions). This is a novel UX pattern for orbit and the SKILL.md needs to define it.
**Evidence:** Existing skills (card, design, discovery) use AskUserQuestion for interviews. Distill uses it for review/approval -- a different modality. No existing skill provides a template for this pattern.
**Recommendation:** Add a constraint or note in the spec defining: (1) cards are presented as formatted YAML blocks, (2) AskUserQuestion suggested answers are ["approve", "edit", "reject"], (3) on "edit", the user's next response is interpreted as modification instructions.

### [LOW] Single deliverable may be insufficient
**Category:** missing-requirement
**Description:** The spec lists one deliverable: `plugins/orb/skills/distill/SKILL.md`. But orbit skills also need a frontmatter `name` and `description` field (per the SKILL.md files examined), and may need registration in a manifest or similar. No guidance on how the skill becomes discoverable by the `/orb:distill` command.
**Evidence:** Every existing SKILL.md starts with YAML frontmatter (`name`, `description`). The spec doesn't mention this. The skill is already listed in the system reminder skills list, so there may be auto-discovery, but the spec should be explicit.
**Recommendation:** Minor: add a constraint that the SKILL.md must include standard frontmatter. This is likely obvious to an implementer familiar with orbit, but a cold-context implementer would miss it.

### [LOW] Spec missing ontology_schema
**Category:** missing-requirement
**Description:** The spec format (per `/orb:spec` SKILL.md line 63-68) includes an `ontology_schema` section. The distill spec omits it. While possibly intentional (distill outputs cards, not domain models), the omission is not explained.
**Evidence:** Spec format template includes `ontology_schema`. Distill spec has no such section.
**Recommendation:** Either add a minimal ontology (e.g., modeling the "candidate card" intermediate object) or document the intentional omission. Low priority since the schema is likely optional.

### [LOW] Ambiguity score of 0.10 may be optimistic
**Category:** assumption
**Description:** The spec self-rates ambiguity at 0.10 (very low). Given the findings above -- unspecified edit flow, subjective AC-08, missing error paths -- a score closer to 0.20-0.25 seems more honest.
**Evidence:** Three of eight ACs have verification gaps (ac-05 edit mechanics, ac-07 "produces valid cards" without defining validity checks, ac-08 subjective traceability). The edit flow is entirely unspecified.
**Recommendation:** Re-score after addressing the above findings. Not blocking, but intellectual honesty matters for the ambiguity framework to stay calibrated.

---

## Honest Assessment

This is a well-structured spec for a genuinely useful skill. The goal is clear, the constraint about not inventing scenarios is the right call, and the one-by-one approval flow shows good judgment about human control. However, the spec has a significant gap in the edit interaction flow (which is the most complex user-facing path) and a problematic reliance on subjective verification for its most important quality criterion (extraction fidelity, weighted at 0.4). The error handling omission is typical of first-pass specs but should be addressed before implementation to avoid the "works on happy path, breaks on first real use" pattern. I recommend addressing the two HIGH findings and at least the error-handling MEDIUM finding before proceeding to implementation. The remaining findings can be resolved during implementation without spec changes if the implementer is aware of them.
