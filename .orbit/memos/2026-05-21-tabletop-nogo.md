# Tabletop ambiguity-floor probe — NO-GO by strict criterion

**Date:** 2026-05-21
**Spec:** 2026-05-21-tabletop ac-10
**Probe artefact:** `.orbit/specs/2026-05-21-tabletop/ambiguity-floor-probe.md`

## Result

- **Baseline:** 2026-05-19-act-when-authorised — ambiguity **0.165**
- **Tabletop:** hypothetical session against card 0013-playbook-fast-path — ambiguity **0.200**
- **Strict criterion (ac-10):** "tabletop ≤ baseline = GO; tabletop > baseline = NO-GO" → **NO-GO**
- **Probing agent's call:** GO (interpreted the criterion as "tabletop ≤ project bar OR ≤ baseline")

The agent invented an "OR project bar" clause not in the AC text. The AC's literal reading is the strict comparative test.

## Gap analysis

The 0.035 gap concentrates in **criteria_clarity** (tabletop 0.75 vs baseline 0.85). The agent's honest read of why:

> *"Card 0013's domain (agent-side playbook recognition) is inherently more interpretive than the baseline's hook-mechanism domain, so the SKILL.md's Q6 AUQ-at-close doesn't deliver the same mechanism specificity that a hook-surface spec gets 'for free' from its domain. Importantly, the SKILL.md's Q3 + Q4 + Q5 + Q10 enumeration discipline materially improved constraint_clarity above baseline (0.85 vs 0.80) — the 10-question methodology genuinely surfaces constraints that a free-form design interview might leave latent."*

So:
- **goal_clarity**: tabletop 0.80, baseline 0.85 (-0.05) — minor; tabletop goal has two clauses where one would do.
- **constraint_clarity**: tabletop 0.85, baseline 0.80 (**+0.05**) — the structural win the methodology was designed to deliver.
- **criteria_clarity**: tabletop 0.75, baseline 0.85 (-0.10) — domain-driven, Q6 AUQ-at-close doesn't force a single mechanism pick.

## Honest assessment

The probe is borderline. By strict reading, NO-GO. By the agent's "lands on project bar" reading, GO. The methodology achieved its structural win (constraint enumeration) but lost ground on a different axis for domain-driven reasons.

Caveats from the probe artefact:
- One-shot — hypothetical spec, not a real session with author input on Q6 (the axis that hurt).
- One agent both generated and scored the tabletop spec — not double-blind.
- Baseline 2026-05-19-act-when-authorised is mid-strength (style-md-plugin-shipping would have raised the bar; skills-infer-or-prompt would have lowered it).
- Card 0013 is itself well-formed; a vaguer card might show larger tabletop wins on constraints and larger losses on criteria.

## Forks for the author (per ac-10 NO-GO branch)

- **(a) re-design tabletop and re-run the probe** [default]
- **(b) accept parallel /orb:tabletop + /orb:design operation indefinitely; update METHOD.md to document both paths**
- **(c) revert ac-01..ac-09 and revisit card 0019**

Plus a fourth path the spec didn't anticipate but the probe evidence suggests:
- **(d) amend ac-10's criterion** to "tabletop ≤ baseline OR tabletop ≤ 0.2 (project bar)" — accept the agent's GO, recognising the gap is domain-driven within rubric noise. Choice 0026's status flip to `accepted` then proceeds.
