---
status: accepted
date-created: 2026-04-20
date-modified: 2026-04-20
---
# 0008. Rally Sub-Agent Path Discipline — Trust + Post-Verify

## Context and Problem Statement

Rally's v2.0 spec claimed that design sub-agents were path-constrained by "the sub-agent's tools allow-list" — implying Claude Code would block writes outside the assigned `spec_dir`. The first subagent-model spec review (2026-04-19) found this was not a Claude Code primitive: the tools allow-list governs which tool names a sub-agent can invoke, not where `Write`/`Edit` may land. The honesty principle requires the spec and SKILL.md to describe what Claude Code actually provides.

The question: how should design sub-agents be kept to their assigned `spec_dir` when there is no native per-path guard?

## Considered Options

- **Option A: Custom PreToolUse hook** — register a hook that intercepts Write/Edit, parses the target path, and refuses non-`spec_dir` writes. Rejected: reintroduces the "custom hook" pattern rally was trying to avoid, and the hook would need per-rally state to know which sub-agent is writing where.
- **Option B: Trust — brief-only contract** — brief instructs the sub-agent where to write, no verification. Rejected: a sub-agent that writes extra files silently passes; trust without a check is not a discipline.
- **Option C: Check — post-return scan only** — `git status --porcelain` on return; brief does not specify the target. Rejected: leaves the sub-agent no contract to follow, making violations hard to diagnose.
- **Option D: Trust + post-verify — three primitives** — brief imposes the contract (self-report a file list), lead asserts the expected artefact exists, lead runs a pre-vs-post `git status --porcelain` snapshot diff in the main checkout and rejects unexpected new entries.

## Decision Outcome

Chosen option: **Option D — Trust + post-verify**, because all three primitives map to real Claude Code mechanisms (the brief format, artefact-existence checks, `git status --porcelain`) and together they provide belt-and-suspenders honesty: the contract is real, the completeness check is real, the independent verification is real.

Snapshot-diff discipline (pre-launch snapshot → post-return snapshot → set difference) filters out false positives from pre-existing uncommitted state and any lead-side mutations that land between launch and return. The allowlist (`orbit/specs/rally.yaml`) covers the one legitimate lead-owned path.

A first violation triggers a pre-qualification re-brief — NOT a rally-level strike, does not count against any drive-full escalation budget. A second violation on the same card parks it with `parked_constraint: "sub-agent violated path discipline"`.

### Consequences

- **Good:** The honesty principle is preserved — every claim in SKILL.md maps to a real Claude Code primitive. Violations are detected by the lead rather than silently admitted.
- **Good:** No custom hooks needed; the pattern uses only tools every rally invocation already has (`Agent tool`, `git status --porcelain`).
- **Good:** The pre/post snapshot discipline makes the verification robust to concurrent lead-side activity (rally.yaml writes, editor state).
- **Bad:** The lead takes responsibility for the check — if the lead forgets, the discipline silently fails. Mitigated by making the snapshot-capture a named step in SKILL.md §4b and by the ac-15 coherence scan asserting that "trust + post-verify" appears in the relevant sections.
- **Bad:** The mechanism does not help with writes outside `orbit/specs/` — those require the unscoped scan (which the final v1.3 spec uses). The snapshot-diff version closes that gap cleanly.

## Related

- orbit/specs/2026-04-19-rally-subagent-model/spec.yaml — constraint #9, ac-03, ac-04
- orbit/specs/2026-04-19-rally/spec.yaml — ac-06 (amended per this decision)
- plugins/orb/skills/rally/SKILL.md — §4a, §4b, Honesty principle callout
