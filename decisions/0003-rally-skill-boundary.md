---
status: accepted
date-created: 2026-04-19
date-modified: 2026-04-19
---
# 0003. Rally as a Separate Skill from Drive

## Context and Problem Statement

Orbit's `/orb:drive` skill delivers one card through the full pipeline (design -> spec -> review-spec -> implement -> review-pr). The observed "rally" pattern -- driving multiple independent cards as a coordinated group -- requires different pre-flight (card selection, disjointness check), different state (`rally.yaml` vs `drive.yaml`), and different gates (consolidated design review, parallel-to-serial decision). The question is whether rally should extend drive or be a separate skill.

## Considered Options

- **Option A: Extend `/orb:drive` to accept multiple cards.** Drive detects whether it received one or many cards and branches accordingly. Keeps one skill but adds conditionals for batch state, consolidated review, and implementation ordering.
- **Option B: New `/orb:rally` skill.** Rally owns multi-card coordination (card selection, parallel design fan-out, consolidated review, implementation ordering, assurance scaling). Delegates individual card stage execution to drive's existing logic.

## Decision Outcome

Chosen option: "Option B -- New `/orb:rally` skill", because drive is already 329 lines with well-tested single-card semantics. Rally has fundamentally different orchestration concerns: N-way fan-out, cross-design dependency scanning, a higher-level state file, and consolidated human gates. Extending drive would bloat it with conditionals and risk regressions in the single-card path.

Rally delegates downward: it coordinates the batch, but each card's design -> spec -> implement -> review stages follow drive's existing stage logic. This keeps drive stable and gives rally a clean boundary.

### Consequences

- Good, because drive remains simple and well-tested for the single-card case
- Good, because rally can evolve independently (e.g. adding parallel implementation) without touching drive
- Good, because the state files are distinct -- `rally.yaml` tracks the batch, `drive.yaml` tracks individual card progress within a rally
- Bad, because some stage logic may need to be extracted from drive into shared utilities so rally can reuse it without duplicating
- Mitigation: extract reusable stage helpers incrementally as rally development reveals which pieces need sharing
