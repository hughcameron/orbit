---
status: accepted
date-created: 2026-05-07
date-modified: 2026-05-07
---
# 0016. Choices, not decisions (within orbit)

## Context and Problem Statement

Decisions in MADR / ADR convention have 25 years of formal-process baggage — "Architectural Decision Record" implies infrastructure-level rulings, multi-stakeholder review, near-immutability. Orbit's pipeline operates at a faster cadence with lighter ceremony; the term "decision" overstates what we are doing most of the time. The lighter word — "choice" — captures the substance (the pick we made between options) without inheriting the ADR weight.

## Considered Options

- **A. Keep "decisions/" terminology.** Aligned with industry convention; consistent with ops repo. Carries baggage we do not want.
- **B. Rename to "choices/" within orbit; keep "decisions/" in ops.** Vocabulary delta makes the difference explicit — system-level rulings (ops) vs project-level forks (orbit). Cross-repo friction.
- **C. Use "choices" everywhere (including ops).** Fully consistent across repos. Loses the system-vs-project distinction.

## Decision Outcome

Chosen option: **B — rename to "choices/" within orbit; keep "decisions/" in ops.** The vocabulary delta is intentional, not accidental. Ops decisions are heavyweight system rulings that benefit from MADR formality. orbit choices are project-level forks made faster and revisited more often; the lighter word matches the lighter cadence.

Format moves from Markdown (MADR prose) to YAML (MADR fields with prose in literal blocks). YAML enforces metadata (status, dates, supersedes, references) via serde; literal-block prose preserves the natural shape of context, considered options, outcome, consequences. Consistent with cards (also YAML) and queryable for cross-cutting lookups ("which choices reference card 0042?").

The rename and format shift land together when orbit-state ships. Until then, current orbit/decisions/ files stay as Markdown in their existing location — this decision records the intent, not the migration.

### Consequences

- Good, because the lighter word matches the lighter cadence; choices feel revisitable, decisions feel authoritative.
- Good, because YAML format gives schema enforcement on metadata while preserving prose in literal blocks.
- Good, because the cross-repo distinction (system rulings vs project forks) is explicit, not buried in convention.
- Bad, because cross-repo friction: the same MADR shape lives under two names. Worth the friction; the words mean different things now.
- Bad, because tooling that assumes "ADR" or "decisions" needs adaptation when applied within orbit.
- Neutral, because existing orbit/decisions/ files migrate to orbit/choices/ as YAML when orbit-state ships; until then, they stay as Markdown in orbit/decisions/.
