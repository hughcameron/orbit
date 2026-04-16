---
status: accepted
date-created: 2026-04-16
date-modified: 2026-04-16
---
# 0001. Progressive Spec Review Replaces Tier-Based Gating

## Context and Problem Statement

The spec skill classified work into risk tiers (HIGH/STANDARD/SKIP) to decide whether spec review was needed. In practice, agents consistently classified work as STANDARD — even for specs touching ground truth corrections and model training data — because the heuristic only measured infrastructure risk, not epistemic risk (work where errors compound downstream). The author ended up reviewing anyway because reviews are valuable regardless of tier.

Trigger incident: Nightingale classified a v12 data quality audit (23 GT corrections feeding v13 training) as STANDARD because it had "no infrastructure or deployment changes."

## Considered Options

- **Option A: Expand tier definitions** — Add "epistemic risk" categories (GT, training data, cross-system). Still requires upfront classification; agents may still misjudge.
- **Option B: Review everything, fixed depth** — Run the full 5-check review on every spec. Wastes time on simple specs; doesn't scale.
- **Option C: Progressive review** — Every spec gets reviewed. Depth scales with findings, not upfront classification. Inspired by decision 0023 (tiered context loading) from the ops repo.

## Decision Outcome

Chosen option: "Option C — Progressive review", because it eliminates the classification failure mode entirely. The review's own findings drive its depth — content signals (touches GT? deployment? cross-system?) trigger deeper passes automatically.

### The Pattern

**Pass 1 — Structural Scan (always runs, fast):**
AC testability, constraint conflicts, scope vs goal, obvious gaps. Plus content signal detection (training data, deployment, cross-system boundaries, security). If clean and no signals: APPROVE and stop.

**Pass 2 — Assumption & Failure Analysis (triggered):**
Fires when Pass 1 finds issues OR content signals are present. Full assumption audit, failure mode analysis, test adequacy.

**Pass 3 — Adversarial Review (triggered):**
Fires when Pass 2 reveals structural concerns. Simultaneous failure, cascade analysis, rollback feasibility, impact radius.

### What Changed

- `spec/SKILL.md`: Removed §6 "Assess Risk Tier" and conditional review guidance
- `review-spec/SKILL.md`: Restructured into 3 progressive passes with trigger conditions
- `implement/SKILL.md`: Removed HIGH-tier reference
- `drive/SKILL.md`: Already runs review-spec as mandatory stage (no change needed)

### Consequences

- Good, because every spec gets reviewed — no classification to get wrong
- Good, because simple specs get a fast review (Pass 1 only) — no wasted time
- Good, because complex specs automatically deepen — content signals catch what tier heuristics missed
- Good, because the pattern is self-documenting — the review output shows which passes ran and why
- Bad, because Pass 1 content signal list needs maintenance as new risk categories emerge
- Mitigation: when a content signal miss causes an incident, add it to the Pass 1 signal list
