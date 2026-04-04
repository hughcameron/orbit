# Implementation Progress

**Spec:** specs/2026-04-04-implement/spec.yaml
**Started:** 2026-04-04

## Hard Constraints
- [x] The /orb:implement skill MUST read the spec before any code is written — SKILL.md step 1-2 enforces read-then-present before implementation
- [x] Hard constraints from in-flight specs MUST be injected at SessionStart even if the agent never invokes /orb:implement — session-context.sh constraint extraction block
- [x] The progress file MUST use ac-NN IDs that match the spec exactly — SKILL.md step 3 template uses ac-NN format
- [x] Unspecced decisions MUST surface as decision checkpoints, never silent choices — SKILL.md step 4 "Surface unspecced decisions" rule
- [x] The skill MUST NOT modify any source code — it only reads the spec and manages the progress tracker — SKILL.md only instructs spec reading and progress tracking

## Acceptance Criteria
- [x] ac-01: Reads spec and presents ACs + constraints as checklist before code — SKILL.md steps 1-2
- [x] ac-02: Writes progress.md with extracted checklist, all pending — SKILL.md step 3 + progress.md.template
- [x] ac-03: Updates progress.md to mark ACs done with notes — SKILL.md step 4 instruction
- [x] ac-04: Implements spec-prescribed patterns over codebase workarounds — SKILL.md "Spec over codebase" rule
- [x] ac-05: Surfaces unspecced decisions via AskUserQuestion — SKILL.md "Surface unspecced decisions" rule
- [x] ac-06: SessionStart hook extracts constraints from in-flight spec — session-context.sh constraint parsing block, tested against live spec
- [x] ac-07: Constraints visible even without /orb:implement invocation — session-context.sh runs unconditionally at SessionStart
- [x] ac-08: review-pr can read progress.md for cross-reference — review-pr SKILL.md updated to check for progress.md
