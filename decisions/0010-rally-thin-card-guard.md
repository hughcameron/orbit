---
status: accepted
date-created: 2026-04-20
date-modified: 2026-04-20
---
# 0010. Rally Thin-Card Guard — Refuse at Proposal

## Context and Problem Statement

A "thin" card (fewer than 3 scenarios) lacks the scenario coverage a design sub-agent needs to produce a 4–6-decision pack. In rally v2.0 the proposal gate did not check scenario counts: thin cards could enter the rally and degrade the design stage — either sub-agents produced shallow decision packs, or the author had to thicken the card mid-rally, which broke the rally's two-gate model.

The question: when should rally refuse a thin card?

## Considered Options

- **Option A: Refuse at proposal — unconditional** — the proposal gate checks scenario counts before the disjointness check; any card with <3 scenarios refuses the rally until thickened or removed.
- **Option B: Warn at proposal, allow in serial only** — thin cards are allowed if the rally turns out serial (one-at-a-time execution gives the author room to recover). Rejected: serial-or-parallel is decided post-design (§6c disjointness check), so the guard cannot wait for it.
- **Option C: Check at design sub-agent launch** — refuse when the design sub-agent starts if the card is thin. Rejected: by that point the author has approved the rally, and rejecting mid-flight breaks the proposal→approval contract.
- **Option D: Allow, produce a thinner decision pack** — sub-agent adapts to thin cards. Rejected: produces low-quality decision packs and leaves the author worse-off at the consolidated decision gate.

## Decision Outcome

Chosen option: **Option A — Refuse at proposal, unconditional on the eventual serial-or-parallel outcome**, because:

- The proposal gate is the single human approval for the rally; adding the thin-card check there preserves the two-gate model.
- The guard runs before the disjointness check (which decides serial-or-parallel), so the guard cannot depend on that outcome.
- Thin cards are a cards/ hygiene issue — the author can thicken via `/orb:card` or run the card individually via `/orb:drive`. Neither is a rally problem.
- Refusing unconditionally is simpler to reason about than a conditional gate that depends on post-design state.

The guard fires with a specific message naming the card and scenario count. The author can thicken the card (re-invoke rally), remove it (re-invoke rally), or run it individually (rally remains unproposed).

### Consequences

- **Good:** Rally design sub-agents always receive cards with ≥3 scenarios, so decision packs stay substantive.
- **Good:** Simple and inspectable — one gate at §2a, before any delegation.
- **Good:** The author keeps full flexibility for thin cards outside rally (individual `/orb:drive guided` or `supervised`).
- **Bad:** Authors with many thin cards may hit the guard repeatedly while learning. Mitigated by the guard's message explicitly naming `/orb:card` as the remedy.
- **Bad:** "3 scenarios" is a heuristic — edge cases exist (a card with 3 very shallow scenarios may still be thin in spirit). Accepted trade-off; the heuristic is simple and the author can always choose the individual-drive path.

## Related

- specs/2026-04-19-rally-subagent-model/spec.yaml — constraint #3, ac-01
- specs/2026-04-19-rally-subagent-model/interview.md — Q3 (2026-04-19 discovery)
- plugins/orb/skills/rally/SKILL.md — §2a Thin-card guard
