# Implementation Progress

**Spec:** .orbit/specs/2026-04-19-rally/spec.yaml
**Started:** 2026-04-19
**Test prefix:** rally

## Deliverables

- `plugins/orb/skills/rally/SKILL.md` — new skill (created)
- `plugins/orb/scripts/session-context.sh` — extended for rally.yaml detection (updated)

## Hard Constraints
- [x] Rally is a separate skill from drive (decision 0003) — `plugins/orb/skills/rally/SKILL.md` is a standalone skill; delegates to drive's stage logic inline (§7)
- [x] Qualification is human — §2 Propose the Rally (AskUserQuestion gate, author approve/modify/reject)
- [x] Design stage uses queued decision packs — §4 Stage: Design prescribes sub-agent briefs that produce decision packs, not Q&A
- [x] rally.yaml in .orbit/specs/rally.yaml is the single source of orchestration state — §3 Initialise Rally State; writes at phase transitions only
- [x] One active rally at a time — §1 Pre-Flight refuses if rally.yaml exists with non-complete phase
- [x] Single-strike NO-GO — §9 NO-GO Handling parks immediately, no retries
- [x] Disjointness check runs at consolidated design review — §6c Definitive disjointness check using extracted files/symbols from actual designs
- [x] Stacked PRs for serial implementation order — §8 Stacked PRs, parked-card gap handling specified
- [x] Parallel implementation uses individual PRs against main with batched diff review — §8 Batched Diff Review
- [x] rally.yaml is validated on read — §12 rally.yaml Validation (required fields, phase enum, status enum, parked_constraint)
- [x] When rally.yaml is active, session-context.sh presents rally state as primary — hook updated: rally detection precedes drive detection, drive block skipped when rally is active
- [x] Task list (TaskCreate/TaskUpdate) + rally.yaml both maintained — §7 Stage: Implementation opens with TaskCreate per card; "both are maintained" explicitly stated
- [x] Directory consolidation (orbit/ directory) is out of scope — rally skill uses .orbit/specs/rally.yaml and .orbit/specs/YYYY-MM-DD-<slug>/
- [x] Design skill upgrade (executive-ready decision packs) is a prerequisite — rally's §4 design stage uses a rally-specific adaptation, noted in Integration section that standard /orb:design is for single-card sessions

## Acceptance Criteria

### Invocation & proposal
- [x] ac-01: §2 scans .orbit/cards/, scores relevance against goal string, presents proposal with per-card rationale
- [x] ac-02: §2 uses AskUserQuestion with approve/modify/reject options; modify loop until approve or reject

### State initialisation
- [x] ac-03: §3 creates .orbit/specs/rally.yaml with all schema fields on approval; §1 refuses on active rally
- [x] ac-04: §12 validates YAML parse, required fields, phase enum, status enum, parked_constraint presence

### Design stage — queued decision packs
- [x] ac-05: §4a launches N parallel sub-agents via Agent tool in single message; brief names card path, output path (<spec_dir>/decisions.md), and 4–6 decisions with options/trade-offs/recommendation
- [x] ac-06: §4a uses tools allow-list scoped to spec_dir; brief forbids writes outside assigned directory
- [x] ac-07: §5 Consolidated Decision Gate groups decisions by card; AskUserQuestion per card captures approve/override
- [x] ac-08: §6a re-launches sub-agents (or lead writes directly) with approved decisions to produce interview.md at each card's canonical path

### Consolidated design review
- [x] ac-09: §6b presents all interviews in a single session with per-card summary and key decisions
- [x] ac-10: §6c extracts files and symbols from each interview; computes intersection; non-empty intersection gates ordering
- [x] ac-11: §6c on shared-symbol detection proposes serial order + rationale, updates rally.yaml, AskUserQuestion to confirm/modify
- [x] ac-12: §6c on no shared symbols records implementation_order: null with order_rationale explaining why parallel is safe

### Implementation
- [x] ac-13: §7 Serial Implementation runs drive's stage logic inline per card in declared order; N+1 waits for N to reach complete or parked
- [x] ac-14: §7 Parallel Implementation launches N sub-agents in git worktrees, each running drive's stage logic independently
- [x] ac-15: §9 NO-GO Handling parks card immediately, rally.yaml records parked_constraint, remaining cards continue

### Assurance
- [x] ac-16: §8 Stacked PRs — each PR targets previous non-parked card's branch; parked-middle handling specified
- [x] ac-17: §8 Batched Diff Review — individual PRs against main, presented together with per-PR summary

### Visibility & state
- [x] ac-18: §7 opens with TaskCreate per card with dependencies; status updates as cards progress
- [x] ac-19: §3 mandates rally.yaml writes at every phase transition and per-card status change
- [x] ac-20: Two-Layer State Model section explicit — rally.yaml for coordination, drive.yaml per card for sub-stage
- [x] ac-21: §11 Resumption validates rally.yaml, uses file-presence detection, handles completed state with archive-or-cancel prompt
- [x] ac-22: session-context.sh updated — detects .orbit/specs/rally.yaml, surfaces rally as primary with goal/phase/card counts/per-card statuses/parked constraints; drive block skipped when rally active

### Completion
- [x] ac-23: §10 Completion writes summary with duration, completed cards + PRs, parked cards + constraints, order + rationale, PR list with target branches
- [x] ac-24: §10 sets phase: complete with timestamp; file remains until next rally archives it (§1 / §11 handle archival)

## Implementation Notes

**All 24 ACs addressed by skill instructions.** The rally skill is documentation-driven (instructions that a Claude Code agent follows at runtime), so "implementation" means the SKILL.md prescribes the required behaviour at every AC. Runtime verification (actually invoking `/orb:rally` end-to-end) will occur during `/orb:review-pr` and in first real-world use.

**Hook tested with simulated rally.yaml fixtures** — active implementing phase surfaces rally + per-card statuses + parked constraints correctly; completed phase shows awaiting-archival message; absent rally.yaml falls through to drive detection as before.

**Review-pr fixes applied (2026-04-19):**
- **F-01 (CRITICAL) addressed** — card counters now use `awk '... END {print n+0}'` instead of `grep -c ... || echo 0`. The grep form concatenated `"0\n0"` into the variable on zero-match (common case: fresh rally). Awk always emits a single integer. Verified with fresh-rally fixture (2 cards, 0 complete, 0 parked) — output now clean.
- **F-02 (HIGH) addressed** — top-level field parses wrapped in `{ grep || true; } | head -1 | sed ...` so missing fields yield empty strings instead of aborting under `set -euo pipefail`. Required-field validation runs after parsing: if `rally:` or `phase:` is missing, hook prints a clear error naming the missing field(s) and falls through (does not abort). Verified with two malformed fixtures — both produce actionable errors, exit 0.

Outstanding review findings (F-03 tools allow-list, F-04 disjointness algorithm, F-05 parallel sub-agent interactivity, F-06 rally.yaml in worktrees, F-07..F-10) remain open — F-03 and F-05 are going to a discovery session because they touch the rally's core model.

**Deferred/out-of-scope (noted in spec):**
- Design skill upgrade (executive-ready decision packs) — separate card prerequisite
- Directory consolidation (orbit/ directory) — separate card

## Next Step

Run `/orb:review-pr` to verify implementation coverage against spec and surface any gaps before creating the PR.
