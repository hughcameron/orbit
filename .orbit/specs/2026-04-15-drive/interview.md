# Design: Agent-Driven Card Delivery

**Date:** 2026-04-15
**Interviewer:** Agent
**Card:** .orbit/cards/0005-drive.yaml

---

## Context

Card: *Agent-driven card delivery* — 8 scenarios covering three autonomy levels (full/guided/supervised), session persistence, NO-GO re-entry, budget exhaustion, and drive state recording.

References: Prior sprint alignment (disposition in agent definitions), ralph-loop (self-referential iteration), .orbit/cards/0003 (progress.md pattern).

## Evidence Brief

- **ralph-loop** provides stop-hook-based autonomous iteration. Session-scoped, no skill chaining or gate awareness.
- **session-context.sh** infers workflow state from file presence (interview.md → spec.yaml → progress.md → review-pr-*.md). This is the existing state machine.
- **`/orb:implement`** has a mature NO-GO protocol: record finding → update card goal → re-enter at design. Manual invocation only.
- **progress.md** as machine-readable checklist state is proven (card 0003).

Constraints carried forward: drive must use the existing file-presence state model and progress.md pattern. No parallel tracking system.

---

## Q&A

### Q1: Execution model — inline vs sub-agents
**Q:** Drive needs to chain skills (design → spec → implement → review-pr). Should it invoke them as inline instructions within a single agent session, or spawn sub-agents per stage?
**A:** Inline single session. One long-running agent reads each SKILL.md and follows its instructions in sequence. Full context preserved, uses ralph-loop pattern to survive interruptions.

### Q2: Design gate in full autonomy
**Q:** The design stage normally uses AskUserQuestion to interview the author. In full autonomy mode, who answers the design questions?
**A:** Agent answers from card scenarios. The card's scenarios, goal, and references contain enough signal. If the card has <3 scenarios, refuse full mode — tell the author what's missing and suggest what to add before retrying. Don't silently downgrade.

### Q3: Iteration budget
**Q:** What should the default iteration budget be before the agent escalates?
**A:** 3 iterations (hardcoded default). First attempt + two re-designs. Three NO-GOs = strong signal the card needs human rethinking. Matches the observed pattern where 2-3 spec cycles typically either solve the problem or produce enough evidence to reframe.

### Q4: Drive state tracking
**Q:** How should drive state be tracked for session resumption across iterations?
**A:** drive.yaml lives in the first spec directory created. It references subsequent iteration dirs. Session-context.sh reads drive.yaml to know the full chain and current position. Structure: card ref, autonomy level, budget, current iteration, history of prior iterations with results and constraints added.

### Q5: Completion behavior
**Q:** When drive completes successfully, should it auto-create the PR and stop, or also handle card updates?
**A:** PR + propose card updates. Drive creates the PR with implementation in commit 1, then proposes card changes (goal refinement, maturity progression) in commit 2 on the same branch. Author reviews both in the PR diff.

---

## Summary

### Goal
An `/orb:drive` skill that takes a card and autonomy level, then drives the full orbit pipeline (design → spec → implement → review-pr) as a single long-running agent session, with file-based state for session resumption and a 3-iteration budget before escalation.

### Constraints
- Single inline session — no sub-agent spawning for pipeline stages
- Card quality gates autonomy: ≥3 scenarios required for full mode, auto-downgrade otherwise
- 3-iteration budget (hardcoded default) before escalation
- drive.yaml in first spec directory as master state file
- Must use existing file-presence state model (interview.md, spec.yaml, progress.md, review-pr-*.md)
- Must use existing progress.md checklist pattern
- Agent self-answers design questions in full mode using card scenarios/goal/references
- Review-pr runs inline (not forked) to preserve context — trade-off accepted for autopilot benefit

### Success Criteria
- Agent can take a card from zero to PR in full mode without human interaction (except merge)
- Guided mode pauses at review gates for author approval
- Supervised mode pauses after spec for author greenlight at each step
- Session interruption + resume works via drive.yaml state
- NO-GO triggers re-design with failure as new constraint
- Budget exhaustion produces an escalation with findings summary
- Successful completion creates PR with implementation + proposed card updates

### Decisions Surfaced
- **Inline over sub-agents:** Single session preserves context across the full pipeline. Forked review-pr loses the "fresh eyes" benefit but gains accumulated implementation context. Accepted trade-off.
- **Card-quality gating:** Full autonomy is a privilege earned by card completeness (≥3 scenarios). Thin cards are refused with a clear message about what's missing — no silent downgrade. This prevents agents from running unsupervised on vague requirements.
- **Fixed budget over configurable:** 3 iterations hardcoded. Simplicity over flexibility — if we find cases needing more, we can add a parameter later.
- **drive.yaml co-located with first spec:** Keeps orchestration state near the work, discoverable by session-context.sh without a new file convention at .orbit/specs/ root.

### Open Questions
- Should session-context.sh changes be part of this spec, or a follow-up? (drive.yaml awareness)
- Should drive emit a decision record (MADR) when it completes, capturing what was learned across iterations?
