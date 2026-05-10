# Design: Rally — Coordinated Multi-Card Delivery

**Date:** 2026-04-19
**Interviewer:** Nightingale
**Card:** .orbit/cards/0006-rally.yaml

---

## Context

Card: *Rally — coordinated multi-card delivery* — 9 scenarios, goal: agents coordinate multi-card delivery with fewer human touchpoints per card than serial drives
Prior specs: 0 — this is the first spec for this card
Gap: The entire capability is unbuilt. The observed pattern (parallel design across 3 cards) worked ad-hoc but had three failure modes: improvised output paths, serial surprise, and no durable batch plan. The research (.orbit/discovery/rally-research.md) resolved the approach for each failure mode. This design session decides the orchestration specifics.

## Q&A

### Q1: How should the author invoke a rally?
**Q:** The card says '/orb:rally with a list of card paths', but there's an alternative: the agent could scan .orbit/cards/ and propose a rally based on shared goals or subsystems. The research strongly recommends keeping qualification human. What's the right invocation model?
**A:** Goal-scoped rally with confirmations. The author names a goal (e.g. "pipeline runtime readiness"), the agent scans .orbit/cards/ and proposes a rally — showing which cards match with rationale per card, and suggesting cards the author might not have considered. The author confirms or edits the list before anything starts. This keeps qualification human (the author approves) while leveraging the agent's ability to scan and propose.

### Q2: When should the disjointness check run?
**Q:** The disjointness check needs to know what files each card will touch — but at rally invocation time, designs haven't happened yet. When should the check run?
**A:** Both — lightweight pre-flight plus definitive post-design. A quick heuristic check at invocation (keyword overlap, shared references, card scenario analysis) flags obvious conflicts early. Then a definitive check at consolidated design review uses actual design outputs — the files and symbols each design names. This matches the observed pattern: the original 3-card session's cards *looked* independent but designs revealed shared Engine trait changes.

### Q3: How do parallel design agents stay on the artefact path?
**Q:** Should the sub-agents run as full background agents writing files directly, or return their design content to the lead agent who writes all files?
**A:** Sub-agents write directly. Each sub-agent gets a brief naming its output path (`.orbit/specs/YYYY-MM-DD-<slug>/interview.md`) and the design skill's conventions. A PreToolUse hook blocks writes outside the assigned card directory. Sub-agents are independent — they read the card and codebase, produce the interview.md, and return. The lead agent verifies files exist at the expected paths after all sub-agents return.

### Q4: What happens when one card in a rally gets a NO-GO?
**Q:** Drive has a 3-iteration budget with re-entry at design. Should rally honour that budget per card, or should a NO-GO be a single-strike park?
**A:** Single-strike park. A NO-GO parks the card immediately — no iteration retries within the rally. Rally is about throughput; retrying one card while others wait (or while context accumulates) defeats the purpose. The parked card is recorded in rally.yaml with its constraint and can be driven individually later with its full 3-iteration budget. The rally summary reports parked cards.

### Q5: Where does rally.yaml live?
**Q:** drive.yaml lives inside the first spec directory for each card. But rally.yaml coordinates across multiple cards' spec directories. Where should it live?
**A:** In `.orbit/specs/rally.yaml`, alongside the per-card spec directories it coordinates. This groups it with the artefacts it manages. Session-context.sh detects it there. Note: the author is considering consolidating all orbit artefacts under an `orbit/` directory — that's a separate card. Rally uses `.orbit/specs/` for now and will move with everything else if the consolidation happens.

### Q6: How should the assurance gate work for v1?
**Q:** The card says serial rallies use stacked PRs and parallel rallies use batched diff. Stacked PRs require tooling. Should we keep it simple for v1?
**A:** Stacked PRs from the start. Each card's PR targets the previous card's branch. The author reviews the stack bottom-up. The serial implementation order maps naturally onto a branch stack. This is the right model even if it requires the author to have stacking tooling available.

---

## Summary

### Goal
Agents coordinate multi-card delivery with fewer human touchpoints per card than serial drives. The rally skill owns multi-card orchestration; individual card stage execution delegates to drive's existing logic.

### Constraints
- Qualification is human — the agent proposes, the author approves
- Sub-agents write directly to assigned paths, constrained by PreToolUse hooks
- rally.yaml is durable state in `.orbit/specs/rally.yaml`, written by code at phase transitions
- Single-strike NO-GO policy — parked cards get their full iteration budget when driven individually later
- Disjointness check runs twice: lightweight at invocation, definitive at consolidated design review
- Stacked PRs for serial implementation order
- Directory consolidation (orbit/ directory) is a separate card — rally uses current paths for now

### Success Criteria
- `/orb:rally` accepts a goal string, scans .orbit/cards/, proposes a rally with rationale, and waits for author approval
- Parallel design sub-agents produce interview.md at correct .orbit/specs/ paths, enforced by PreToolUse hook
- Consolidated design review surfaces cross-cutting concerns and proposes serial/parallel implementation order
- rally.yaml survives session death and enables full resumption
- A single-card NO-GO parks the card and the rally continues with remaining cards
- Stacked PRs are created for serial rallies, with the author reviewing bottom-up

### Decisions Surfaced
- Goal-scoped invocation with confirmation: chose interactive proposal over explicit card lists or pure agent judgement. Agent scans and proposes; author confirms. Balances convenience with the research finding that LLM qualification is unreliable.
- Two-stage disjointness: lightweight pre-flight (heuristic) + definitive post-design (actual file/symbol analysis). Neither stage alone is sufficient — pre-flight catches obvious conflicts, post-design catches the non-obvious ones (shared traits, interfaces).
- Single-strike NO-GO: rally parks on first failure rather than honouring drive's 3-iteration budget. Rally optimises for throughput; perseverance belongs to individual drives.
- Stacked PRs from v1: no deferred complexity on the review model. Serial implementation order maps to branch stacks.
- Directory consolidation is a separate card: affects all skills, not just rally. Ship separately.

### Open Questions
- PreToolUse hook implementation: does orbit need to ship a hook script, or can the sub-agent brief include path constraints that Claude Code enforces natively via the `tools:` allow-list?
- Session-context.sh detection pattern for rally.yaml: should it show per-card status or just the rally-level phase?
- How does the lead agent brief sub-agents with design skill conventions without running `/orb:design` as a sub-skill? The brief needs to include enough of the design skill's instructions for the sub-agent to produce a well-formed interview.md.
