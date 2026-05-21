# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-tabletop
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | Pass 1 found MEDIUM+ findings; content signals present (cross-skill cascade, METHOD.md edits, deletion of an active skill) | 3 |
| 3 — Adversarial | Pass 2 surfaced an unverifiable baseline and unstated dependencies between ACs | 2 |

## Findings

### [HIGH] ac-10's "recorded design-session score" baseline does not exist on disk

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-10 makes ac-11 (the retirement cascade) gated on a parity probe against "the recorded design-session score" for card 0043-session-start-priority-synthesis. That spec was driven on 2026-05-21 with two review-spec cycles and a clean review-pr — but neither `notes.jsonl`, the review artefacts, nor the spec.yaml contain an ambiguity score per the formula `goal * 0.4 + constraints * 0.3 + criteria * 0.3`. The current `/orb:design` SKILL.md §7 prescribes ambiguity assessment but does not require persisting the numeric score; the existing record set lists only review verdicts.
**Evidence:** `orbit spec show 2026-05-21-session-start-priority-synthesis` and inspection of `.orbit/specs/2026-05-21-session-start-priority-synthesis/{notes.jsonl,spec.yaml,review-spec-2026-05-21*.md}` — none of the four files records a numeric ambiguity score or per-axis breakdown. ac-10 treats this score as if it were already on disk.
**Recommendation:** Amend ac-10 to either (a) score the prior 2026-05-21-session-start-priority-synthesis design retroactively from the interview/spec artefacts using the spec-skill formula, and persist that baseline as part of `ambiguity-floor-probe.md`; or (b) re-run `/orb:design` against a fresh test card with explicit score capture before the probe. The baseline must be on disk before the probe can claim parity.

### [HIGH] ac-11 names a non-existent METHOD.md path

**Category:** failure-mode
**Pass:** 2
**Description:** ac-11(3) claims METHOD.md is updated "across all three copies (canonical at `plugins/orb/skills/setup/canonical/METHOD.md`, the vendored copy under `plugins/orb/skills/setup/canonical/`, and the project copy at `.orbit/METHOD.md`)". Only two copies exist on disk: `plugins/orb/skills/setup/METHOD.md` (canonical/vendored) and `.orbit/METHOD.md` (project). There is no `plugins/orb/skills/setup/canonical/` directory — the listed setup-skill contents are `METHOD.md`, `SKILL.md`, `STYLE.md` flat.
**Evidence:** `find /home/hugh/github/meridian-online/orbit -name METHOD.md` returns exactly `plugins/orb/skills/setup/METHOD.md` and `.orbit/METHOD.md`. `ls plugins/orb/skills/setup/` shows no `canonical/` subdirectory.
**Recommendation:** Rewrite ac-11(3) to name the two real copies. The AC's clarity is load-bearing because it's a gate AC and `orbit audit conformance --json` (ac-11(5)) is also a gate condition — divergence between the AC's listed paths and the substrate's actual paths makes the gate ambiguous to close.

### [HIGH] ac-11's cascade-target list overstates the touch surface

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-11(2) names 12 SKILL.md files that "every `/orb:design` reference becomes `/orb:tabletop` or is rewritten" in: distill, spec, spec-architect, interviewer, drive, rally, implement, review-spec, review-pr, setup, release, discovery. A grep of `plugins/orb/skills/*/SKILL.md` for `/orb:design` returns only 7 of those 12 — distill, spec, drive, review-spec, setup, discovery, and card (which the AC doesn't list). The 5 listed-but-clean files (spec-architect, interviewer, rally, implement, review-pr, release) will trip a literal-reading implementing agent into either fabricating edits or marking the AC failed for missing-substrate.
**Evidence:** `grep -rln "/orb:design" plugins/orb/skills/*/SKILL.md` returns 7 paths; `plugins/orb/skills/card/SKILL.md` is present in that list but absent from ac-11; `spec-architect`, `interviewer`, `rally`, `implement`, `review-pr`, `release` are absent from the grep but present in ac-11.
**Recommendation:** Replace the explicit file list in ac-11(2) with a substrate query: *"every `/orb:design` reference in tracked SKILL.md and CLAUDE.md files becomes `/orb:tabletop` (or is rewritten for the new shape) — the canonical detection is `rg -l '/orb:design' plugins/orb/skills`, run at implement time"*. This makes the AC self-verifying via grep and absorbs additions/removals that land between spec authoring and implementation.

### [MEDIUM] No AC commits to retiring the card-0019-references substrate

**Category:** missing-requirement
**Pass:** 1
**Description:** Card 0019 references `.orbit/choices/0017-tabletop-output-is-contract.md` as load-bearing. ac-11 retires `/orb:design` but says nothing about whether choice 0017 needs updating (its rationale references the v1 `/orb:design` failure mode). Likewise, card 0031-design-session-user-language is named "design" and may need re-keying. No AC commits to a touch-list for tabletop-adjacent substrate beyond SKILL.md and METHOD.md.
**Evidence:** `grep -rln "/orb:design"` includes `.orbit/cards/0031-design-session-user-language.yaml`, `0035-ac-taxonomy.yaml`, `0034-spec-close-ac-preflight.yaml`, `0037-memory-gates-decisions.yaml`, `0004-specs-array-integrity.yaml`, `0042-act-when-authorised.yaml`, `0039-workflow-conformance.yaml`, `0028-four-pillars.yaml`, `0030-canonical-schema-and-glossary.yaml`, `0026-agent-prose-discipline.yaml`, plus several memos and convention files. ac-11 scope is "SKILL.md and METHOD.md", which leaves the cards/conventions/memos surface untouched.
**Recommendation:** Add an AC (or extend ac-11) committing to either (a) update card/convention/memo references in the same pass, or (b) explicitly declare them out of scope and accept the staleness. The current spec is silent on the choice.

### [MEDIUM] ac-09 closed-mode path conflicts with ac-02 sidecar contract

**Category:** constraint-conflict
**Pass:** 1
**Description:** ac-02 prescribes that "every tabletop session writes a `tabletop.md` sidecar per output spec at `.orbit/specs/<spec-id>/tabletop.md`" with six declared sections (values, trade-offs, halt conditions, escalation triggers, kill conditions, hot-wash). ac-09 introduces a closed-mode path producing `tabletop-note.md` instead, with a different shape (What good looks like, Pinned approach, Deferred items, Implementation notes). It is not stated whether closed-mode skips the `tabletop.md` sidecar or produces both. The phrasing "instead of running the full 10-question methodology" implies skip, but ac-02 says "every tabletop session writes a `tabletop.md` sidecar".
**Evidence:** ac-02 says "every tabletop session writes a `tabletop.md` sidecar"; ac-09 says closed-mode produces `tabletop-note.md` "instead of running the full 10-question methodology". The two are not reconcilable as written without reader interpretation.
**Recommendation:** Amend ac-02 to scope its sidecar requirement to open/partial-mode sessions, or amend ac-09 to explicitly state that closed-mode replaces the sidecar entirely (and confirm closed-mode is exempt from the six-section contract). Pick one and write it inline.

### [MEDIUM] ac-08 AUQ-prose hybrid is missing the failure-mode for AUQ refusal

**Category:** failure-mode
**Pass:** 2
**Description:** ac-08 pins specific questions to AUQ (Q6 success criteria, Q7 escalation, Q9 budget, Q10 kill conditions). If the author rejects all offered options and types a custom response that triggers re-scoping, the AC is silent on what happens — does the agent re-run the prose-phase question? Loop AUQ? Fall back to prose? The 2026-05-07 dogfood explicitly used AUQ at closing picks but did not encounter a custom-response reframe in the budget step (the inflation-guard recut was an agent-side adjustment, not an author-side reframe).
**Evidence:** ac-08 prose names question-type binding but doesn't define fallback. The dogfood file `.orbit/archive/specs/2026-05-07-orbit-state-v0.1/tabletop.md` Q9 captured the inflation-guard recut as a methodology note, not as a re-elicit.
**Recommendation:** Add one sentence to ac-08 describing the AUQ-refusal fallback: "If the author rejects all AUQ options with a custom response that reframes the question itself, the agent treats the response as a return-to-prose signal and re-walks that question's prose phase." This closes the loop the AC currently leaves open.

### [MEDIUM] ac-04 card-inference algorithm is unspecified

**Category:** test-gap
**Pass:** 2
**Description:** ac-04 prescribes that on a bare goal string ("ship X"), the agent "walks the card index, infers which cards the goal touches, presents the inferred cluster in prose with one-line rationales per card". How? Substring match on goal/feature fields? Embedding similarity? Manual scan with budget cap? With ~45 cards in `.orbit/cards/`, the agent's inference will be subjective and the AC has no verification surface — two agents handed the same goal string can plausibly infer different clusters and both pass the AC.
**Evidence:** `ls .orbit/cards/` shows >40 entries; ac-04 names no detection heuristic; ac-04 has no test that pins the inference output.
**Recommendation:** Either (a) pin a concrete heuristic in the AC ("substring match on card `feature` and `goal` fields, ranked by hit count, top-5 surfaced"), or (b) name the AUQ-confirmation step as the safety valve and accept the inference as best-effort. (b) is the lighter touch given the AUQ is already part of the flow.

### [LOW] ac-12 names choices "tabletop-replaces-design" but card 0017 already exists

**Category:** missing-requirement
**Pass:** 3
**Description:** ac-12 commits to writing `tabletop-replaces-design`. Choice 0017 (`tabletop-output-is-contract`) already exists, accepted, and is cited as load-bearing in card 0019. There is no AC committing to revising 0017 — its rationale section refers to "the v1 design failure mode" and the retirement cascade may make its framing stale, but ac-12 adds two new choices without amending the existing one.
**Evidence:** `.orbit/choices/0017-tabletop-output-is-contract.yaml` exists; card 0019 lists it under `references` as "load-bearing rule that prevents the v1 design failure mode"; ac-12 adds two new choices but does not address 0017.
**Recommendation:** Add a one-line clarification to ac-12: either (a) the two new choices supersede 0017's design-failure-mode framing and 0017 gets a `superseded-by` link added, or (b) 0017's framing stands and the two new choices cite it. Currently ambiguous.

### [LOW] Pass-3 cascade — ac-10 NO-GO leaves the spec half-shipped

**Category:** failure-mode
**Pass:** 3
**Description:** If ac-10's probe returns NO-GO, ac-11 is explicitly gated off. But ac-01 through ac-09 (the SKILL.md, sidecar shape, flows, AUQ-prose hybrid, closed-mode path) all ship regardless. This means `/orb:tabletop` lands and operates alongside `/orb:design` indefinitely. The card 0019 goal contemplates tabletop as the canonical pre-spec session — running two skills in parallel produces drift over time, especially in cascade docs (METHOD.md pipeline diagrams, distill skill refs).
**Evidence:** ac-10/ac-11 gating language: "ac-11 lands ONLY after ac-10 returns GO". No AC covers the parallel-running state.
**Recommendation:** Add a single sentence to ac-10 (or a new ac-13): "If ac-10 returns NO-GO, surface a memo at `.orbit/memos/<date>-tabletop-noogo.md` capturing the probe result, gap analysis, and one of: (a) re-design tabletop and re-run the probe, (b) accept parallel operation explicitly and update METHOD.md to document both paths, or (c) revert ac-01..ac-09 and revisit the card." This closes the NO-GO branch.

---

## Honest Assessment

The spec is well-shaped overall — the methodology contract (ac-01..ac-09) is concrete, the sidecar pattern is anchored to a real dogfood file, and the retirement gate (ac-10) is the right structural protection. The biggest risks are the three HIGH findings, all in the retirement cascade (ac-10 and ac-11): the parity baseline doesn't exist on disk, one of the named METHOD.md paths is fabricated, and the cascade-file list is wrong in both directions (missing files and naming files that don't have the reference). These are mechanical fixes — pin the baseline, drop the canonical/ path, replace the file list with a grep query — but a literal-reading implementing agent will trip on them.

The two structural-conflict findings (ac-09 vs ac-02 closed-mode sidecar handling; ac-08 AUQ-refusal fallback) and the inference-algorithm gap (ac-04) are the second tier — they will surface during implementation as the agent picks an interpretation, but the AC framing should pre-commit which one. Approve after the cascade-substrate findings are addressed; the methodology core is sound.

The choice-file decisions in ac-12 are the right shape; the only nit is whether choice 0017 needs a touch-up alongside the two new choices.
