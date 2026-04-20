# Implementation Progress

**Spec:** orbit/specs/2026-04-15-drive/spec.yaml
**Started:** 2026-04-15

## Hard Constraints
- [x] Single inline session — reads each stage's SKILL.md, no sub-agent spawning — SKILL.md §3-6 all say "Read the X skill instructions from SKILL.md. Follow its instructions"
- [x] Card quality gates full autonomy — ≥3 scenarios required — SKILL.md §1 Pre-Flight with explicit REFUSE block
- [x] 3-iteration budget before escalation (hardcoded) — SKILL.md §2 sets budget: 3, §7 checks iteration > budget
- [x] drive.yaml in first spec directory as master state file — SKILL.md §2 creates it in first spec dir
- [x] Uses existing file-presence state model — no parallel tracking — SKILL.md §10 Resumption uses file-presence table
- [x] Agent self-answers design questions in full mode from card content — SKILL.md §3 full mode adaptation
- [x] Successful completion: PR with commit 1 (implementation) + commit 2 (card updates) — SKILL.md §9 Completion
- [x] session-context.sh updated to detect drive.yaml — drive detection block added before latest_spec logic

## Acceptance Criteria
- [x] ac-01: SKILL.md exists with usage, autonomy levels, and stage instructions — plugins/orb/skills/drive/SKILL.md created with usage section, autonomy table, and §3-6 stage instructions
- [x] ac-02: Full mode drives design→spec→implement→review-pr without human interaction — §3-6 with full mode: no AskUserQuestion, agent self-answers design, proceeds through all stages
- [x] ac-03: Guided mode pauses at review gates for author approval — §6 Review stage: guided mode uses AskUserQuestion at review verdict
- [x] ac-04: Supervised mode pauses after spec for author greenlight at each step — §4 spec gate, §5 implement gate, §6 review gate all have supervised mode AskUserQuestion
- [x] ac-05: Thin card (<3 scenarios) refused for full mode with message naming gap — §1 Pre-Flight: BLOCKED message with scenario count and suggestions
- [x] ac-06: drive.yaml created in first spec directory with required fields — §2 template includes card, autonomy, budget, iteration, current_spec, status, history, started
- [x] ac-07: NO-GO triggers re-entry at design with failure as new constraint — §7 records failure in history, increments iteration, re-enters at §3 with constraint carried forward
- [x] ac-08: Budget exhaustion (3 NO-GOs) produces escalation with findings summary — §8 outputs iteration history, accumulated constraints, recommendation, then stops
- [x] ac-09: Successful completion creates PR with two commits — §9 commit 1 (implementation), commit 2 (card updates), then PR creation
- [x] ac-10: session-context.sh detects drive.yaml and surfaces active drive state — drive detection block parses drive.yaml, outputs card ref, autonomy, iteration, status, and next action
- [x] ac-11: Default autonomy level is guided — §1 "defaults to guided if omitted", usage section states default
- [x] ac-12: Drive resumes from correct stage after session interruption — §10 Resumption with file-presence table and status override logic
