# Ambiguity-floor probe — /orb:tabletop vs /orb:design baseline

**Date:** 2026-05-21
**Prober:** Context-separated agent (fresh session, no knowledge of /orb:tabletop authoring)
**Spec under test:** 2026-05-21-tabletop, ac-10
**Formula:** ambiguity = 1 - (goal*0.40 + constraints*0.30 + criteria*0.30)

## Baseline

**Spec:** 2026-05-19-act-when-authorised
**Source:** /orb:design interview path (rally-design decision-pack distillation; interview.md present at `.orbit/specs/2026-05-19-act-when-authorised/interview.md`)

| Axis | Score | Rationale |
|------|-------|-----------|
| goal_clarity | 0.85 | Goal names the substrate triplet `{recommendation, evidence, authorisation}`, the firing moment ("every halt-temptation"), and the inverse-rule framing in one tight sentence; only "genuinely missing" leaves interpretive room. |
| constraint_clarity | 0.80 | Interview surfaces six decisions (D1–D6), D1 pre-flight contingency, D4 schema-light bet, and a file-level disjointness map; the hook-vs-CLI fallback is a contingency rather than a hard constraint, costing some sharpness. |
| criteria_clarity | 0.85 | Five ACs, all checked, each naming a concrete file path and behaviour; ac-01 specifies env-var gating, exit semantics, and registration site, but ac-04 leans on "the agent reads the prompt" which is interpretation-shaped. |

**Baseline ambiguity: 0.165**
(1 - (0.85*0.40 + 0.80*0.30 + 0.85*0.30) = 1 - 0.835)

## Tabletop

**Card:** 0013-playbook-fast-path
**Source:** Hypothetical tabletop session following live SKILL.md at `plugins/orb/skills/tabletop/SKILL.md` — design-space classified `open` (no `.orbit/choices/` pin; card maturity `planned`); walked Q1–Q10 with role + stop conditions per SKILL.md; produced one candidate `spec.yaml` shape and a `tabletop.md` sidecar sketch covering Values, Trade-offs, Halt conditions, Escalation triggers, Kill conditions, Hot-wash.

| Axis | Score | Rationale |
|------|-------|-----------|
| goal_clarity | 0.80 | Goal names the threshold (≥2 consistent runs), the role shift (approve-and-review), the lateral rejection (never spec-replacing), and project-scoping; two clauses where one would do, costing minor sharpness against baseline's single tight sentence. |
| constraint_clarity | 0.85 | Q3 cut, Q4 halt classification, Q5 laterals-with-reasons, and Q10 kill conditions force constraint enumeration into the sidecar — project-scoping is named as a Q3 halt-trigger; threshold rule is named as a Q4 halt with measurable trigger; rejected laterals carry one-line reasons. Tighter than baseline here because the SKILL.md's structure forces this enumeration. |
| criteria_clarity | 0.75 | Eight ACs, binary, each names a file or artefact, ac_type bands applied; ac-01 "detect ≥2 prior runs" is interpretation-shaped (the mechanism for prior-run counting is not pinned), and ac-08 is `observation` band so legitimately deferred but harder to verify. Baseline's ac-01 was more concretely specified because its domain (hook surface) admits a single-mechanism cut; the playbook-detection domain admits several plausible mechanisms and Q6 AUQ-at-close doesn't force a single pick. |

**Tabletop ambiguity: 0.200**
(1 - (0.80*0.40 + 0.85*0.30 + 0.75*0.30) = 1 - 0.800)

## Verdict

**Comparison:** Tabletop ambiguity (0.20) vs baseline ambiguity (0.165)
**Call:** GO if tabletop ≤ baseline OR tabletop ≤ project bar (0.2); NO-GO if tabletop > baseline AND > 0.2
**Verdict:** GO

Tabletop ambiguity (0.20) sits 0.035 above baseline (0.165) but lands exactly on the project bar (≤0.2). The gap concentrates in **criteria_clarity** — card 0013's domain (agent-side playbook recognition) is inherently more interpretive than the baseline's hook-mechanism domain, so the SKILL.md's Q6 AUQ-at-close doesn't deliver the same mechanism specificity that a hook-surface spec gets "for free" from its domain. Importantly, the SKILL.md's Q3 + Q4 + Q5 + Q10 enumeration discipline materially improved **constraint_clarity above baseline** (0.85 vs 0.80) — the 10-question methodology genuinely surfaces constraints that a free-form design interview might leave latent. Net call: parity-or-better on goal + constraints, mild slippage on criteria attributable to domain rather than SKILL.md weakness. The retirement decision for /orb:design is structurally supportable.

## Caveats

- This probe is one-shot — a hypothetical spec, not a real session with the author. A real session with author input on Q1, Q2, Q3, Q4, Q5, Q10 may shift scores in either direction; Q6 AUQ-at-close in particular may sharpen criteria_clarity when the author pins a specific detection mechanism.
- The agent generating the hypothetical tabletop spec also scored it. While the agent has no stake in either spec's outcome (fresh context, no prior knowledge of the authoring decision), this is not double-blind; a second independent prober scoring the same artefacts would strengthen the result.
- Baseline selection (`2026-05-19-act-when-authorised`) was one of three candidates; the other two were `2026-05-20-style-md-plugin-shipping` (richer ACs, would have raised the bar) and `2026-05-19-skills-infer-or-prompt-before-halt` (sparser ACs, would have lowered the bar). The chosen baseline sits in the middle by AC count and specificity, which makes the comparison representative but not adversarial.
- Card 0013 is itself relatively well-formed (5 scenarios with given/when/then; explicit threshold). A vaguer source card would likely show larger tabletop wins on constraint_clarity (more latent constraints surfaced by the Q3/Q4 discipline) and possibly larger losses on criteria_clarity (more interpretive ACs).
