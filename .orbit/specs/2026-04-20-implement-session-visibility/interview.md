# Design: Implement Session Visibility

**Date:** 2026-04-20
**Interviewer:** nightingale (rally sub-agent)
**Card:** .orbit/cards/0003-implement.yaml

---

## Context

Card: *Pre-flight spec check before implementation* — 11 scenarios, goal: ensure hard constraints and acceptance criteria from the spec are surfaced as a tracked checklist before any code is written, and remain visible across sessions.
Prior specs: 1 — `.orbit/specs/2026-04-04-implement/spec.yaml` shipped the pre-flight skill (read spec, present checklist, write progress.md, SessionStart hard-constraint surfacing).
Gap: the shipped skill relies on progress.md alone for in-session visibility. Four new scenarios extend it with Claude Code tool primitives — TaskCreate on checklist generation, TaskUpdate on AC completion, SessionStart rebuild from progress.md, and Monitor-backed test streaming — so the checklist is a first-class session artefact rather than a markdown file the agent has to remember to read. This card consumes the progress.md schema extensions that card 0009 (mission-resilience) owns (`Detours`, `Return to:`, `Current AC:`, gate AC semantics).

## Q&A

### Q1: Task ↔ AC/constraint mapping granularity

**Q:** How should the pre-flight checklist map onto the session task list — flat nodes only, flat nodes with constraint→AC dependency edges, or flat nodes with dependency wiring reserved for card 0009's gate AC work?

**A:** Flat tasks, no edges. Emit one pending task per AC and one pending task per hard constraint; do not wire `addBlockedBy` or `addBlocks` relationships. Rationale: the card scenario only mandates nodes; constraints are global invariants, not prerequisite work items, so treating them as blockers would overstate their semantics and collide with 0009's real gate AC edges. Reserving the dependency slot for 0009 keeps this card's surface narrow (node creation only) and matches the rally coordination rule "flag shared-schema concerns, not resolve them." 0009 will layer gate dependencies on top without reshaping what implement emits.

### Q2: When TaskCreate fires

**Q:** At what point in the pre-flight sequence does TaskCreate run — at checklist presentation (§3, before progress.md is written), immediately after progress.md is written (§4, using the file as source of truth), or lazily on the first progress.md update?

**A:** Immediately after progress.md is written (§4), using progress.md as the source of truth. Rationale: progress.md is already the durable record of the pre-flight checklist. Deriving tasks from the parsed file — rather than from an in-memory extraction presented in §3 — eliminates a second serialisation, guarantees the task list and progress.md cannot disagree on initial content, and keeps the pre-flight path symmetric with the resume path (Q4), which also reads progress.md. The scenario's `when the pre-flight checklist is generated` is satisfied because progress.md is produced as part of checklist generation.

### Q3: TaskUpdate trigger

**Q:** How is TaskUpdate triggered when an AC is marked complete — as a skill rule (agent invokes TaskUpdate in the same turn as the progress.md edit), as a post-write hook watching progress.md, or only at checkpoint boundaries?

**A:** Skill rule. SKILL.md §5 is extended: "when you mark an item `- [x]` in progress.md, call TaskUpdate with status `completed` in the same tool-call turn." Rationale: the scenario's `in the same tool-call turn` phrasing reads as an agent-authored batch, not an automated hook. The shipped skill is already agent-driven discipline (§5 "State which AC(s) were addressed"); a skill rule is low-infrastructure, matches that style, and avoids adding a `PostToolUse`/file-watch hook that would collide with 0009's concurrent hook modifications. The discipline risk (agent forgets) is acknowledged — revisit with a hook only if empirically warranted.

### Q4: Session resume — rebuild vs reconcile

**Q:** When a session resumes mid-implementation, how should the SessionStart hook reconcile the existing task list with progress.md — unconditional rebuild, full reconcile by AC-ID matching, or reconcile-or-skip (match on AC ID; fall back to rebuild on drift)?

**A:** Reconcile-or-skip. The hook scopes to orbit-implement tasks via a spec-path tag stored in task `metadata`, matches existing tasks against progress.md by AC ID (e.g. `ac-01`) and constraint hash. If the set and statuses match, do nothing. If there is drift (count or status mismatch), fall back to full rebuild and emit a warning. Rationale: most resumes happen without drift, so aggressive unconditional rebuild churns task IDs (breaking transcript references) and discards accumulated metadata. A match check is cheap; the rebuild fallback keeps the implementation bounded and preserves the scenario intent ("rebuilds the session task list" applies when drift is detected).

### Q5: Monitor scope — which commands qualify

**Q:** When should `/orb:implement` wrap a test command in Monitor rather than Bash — opt-in via a new spec metadata field, heuristic based on expected duration, or blanket rule for the primary test command?

**A:** Heuristic with a clarifying filter rule. Skill rule: "if a test is likely to exceed ~60s or is a full-suite run, wrap it in Monitor with a failing-line filter; otherwise use Bash." The filter is mandated: `grep --line-buffered` for FAIL/ERROR/AssertionError/traceback markers (the tools-reference explicitly calls out line-buffering as necessary to avoid pipe buffering swallowing events). Rationale: the scenario's `given` is "takes several minutes" — the trigger is duration, not an explicit spec field. An opt-in spec field would add schema churn during a rally where 0009 is already reshaping the progress.md schema. Blanket Monitor use loses the value when a suite finishes in seconds. Document fail-fast behaviour separately (Q6).

### Q6: Monitor failure → implement loop behaviour

**Q:** What should the agent do when Monitor surfaces a failing line mid-run — surface and continue, surface and checkpoint on first failure, or surface and cancel after N failures?

**A:** Surface and checkpoint on first failure. SKILL.md §5 gains a rule: on the first streamed FAIL/ERROR event the agent pauses, acknowledges the failure, and checkpoints with the author (fix now vs let the suite finish) before continuing. Rationale: the scenario's value ("failures surface mid-run rather than on completion") is only realised if mid-run failures change behaviour. Surface-and-continue is nearly indistinguishable from plain Bash capture. A checkpoint is consistent with the shipped skill's "Surface unspecced decisions" pattern and with CLAUDE.md's Decision Checkpoints convention — a first failure is a meaningful branch point. Cancel-after-N requires picking N and handles cascading failures poorly.

---

## Summary

### Goal

Extend the shipped pre-flight implement skill so the AC/constraint checklist is a first-class, real-time session artefact — created as tasks at pre-flight, kept in sync as the agent works, rebuilt faithfully on resume, and complemented by mid-run test visibility via Monitor — without altering the shipped behaviours (read spec, write progress.md, SessionStart constraint surfacing) and without defining any progress.md schema beyond what card 0009 owns.

### Constraints

- Shipped pre-flight behaviour is out of scope — card is `maturity: established`; do not alter the existing checklist format or the SessionStart constraint surfacing.
- progress.md schema extensions (`Current AC:`, `## Detours`, `Return to:`, `Spec hash:`, gate annotation) are defined by card 0009; this card consumes them but MUST NOT define or modify the schema.
- progress.md is the single source of truth for both TaskCreate (pre-flight) and TaskCreate/TaskUpdate-on-rebuild (resume) — no in-memory-only task emission path.
- Task emission is flat: one task per AC, one task per hard constraint, no dependency edges. Dependency wiring is reserved for card 0009.
- TaskUpdate on AC completion is an agent-side skill rule that fires in the same tool-call turn as the progress.md edit — not a hook, not a batched checkpoint.
- Session resume tasks are scoped by a spec-path tag stored in task `metadata`; the hook must not touch non-orbit-implement tasks.
- Monitor test invocations must include a line-buffered filter (`grep --line-buffered` for FAIL/ERROR/AssertionError/traceback markers). Unfiltered Monitor on a test suite is forbidden — every stdout line becomes a notification otherwise.
- Monitor is only used when the test command is expected to exceed ~60s or is a full-suite run; short tests continue to use Bash.
- On the first streamed failure the agent pauses and checkpoints with the author before continuing.
- The progress.md parser used by the resume hook must ignore `## Detours` content when determining AC status — detour lines near an AC header must not mark it complete.
- A single shared parser (`parse-progress.sh` or equivalent awk block) is proposed for both this card's hook and 0009's "next unchecked AC" surfacing; ownership sits with 0009.

### Success Criteria

- On pre-flight, every AC in the spec becomes a pending task and every hard constraint becomes a pending task — verified by comparing `TaskList` output against `progress.md` sections.
- When the agent marks `- [x] ac-NN` in progress.md, the corresponding task transitions to `completed` in the same tool-call turn — verified by transcript inspection.
- On session resume, the rebuilt task list's AC-by-AC statuses match the `- [ ]`/`- [x]` state in progress.md at resume time.
- When progress.md and the existing task list are already in sync on resume, the hook performs no task mutations (reconcile-skip path exercised).
- A Monitor-wrapped test run streams FAIL/ERROR lines to the agent mid-run; on the first such event the agent checkpoints rather than waiting for completion — verified in an integration-style scenario.
- `/orb:review-pr` AC coverage aligns with `/orb:implement` because both skills read the same spec and the same progress.md — verified by a scenario where both are run against the same spec.

### Decisions Surfaced

- **D1 — Task granularity:** chose flat tasks (one per AC, one per constraint) with no dependency edges over flat-with-constraint-blocks-AC or flat-with-0009-reserved-edges. Rationale: scenario text mandates nodes only; constraints are global invariants, not prerequisite work items; 0009 owns gate edges.
- **D2 — TaskCreate timing:** chose "immediately after progress.md is written (§4), using progress.md as source of truth" over "at checklist presentation (§3)" or "lazy on first progress.md update". Rationale: single source of truth with the resume path, eliminates double serialisation.
- **D3 — TaskUpdate trigger:** chose agent-invoked skill rule (TaskUpdate in the same turn as the progress.md edit) over post-write hook or checkpoint-only. Rationale: matches scenario's `same tool-call turn` phrasing, low infrastructure, avoids colliding with 0009's hook work.
- **D4 — Session resume strategy:** chose reconcile-or-skip (AC-ID match; fall back to full rebuild on drift) over unconditional rebuild or full reconcile. Rationale: common-case cheap, safe fallback, preserves task IDs and metadata when possible. Spec-path stored in task `metadata` scopes the hook.
- **D5 — Monitor scope:** chose duration-based heuristic ("~60s or full-suite run" → Monitor with line-buffered FAIL/ERROR filter) over opt-in spec metadata field or blanket rule. Rationale: scenario trigger is duration, not schema; avoids schema churn during 0009's schema reshape; mandates the filter so every-stdout-line-is-a-notification doesn't swamp the agent.
- **D6 — Monitor failure behaviour:** chose surface-and-checkpoint-on-first-failure over surface-and-continue or cancel-after-N. Rationale: delivers the stated "react mid-run" value, consistent with shipped "Surface unspecced decisions" pattern and CLAUDE.md Decision Checkpoints convention.

### Open Questions

All three shared-surface items below are *resolved in favour of card 0009 as schema owner* — this card's spec MUST treat the schema as given and depend on 0009's spec landing first. They are recorded here as inter-card coordination items, not as open design questions for this card.

- **progress.md section layout and detour-line handling (D3, D4).** Card 0009 Decision 5 defines the authoritative template (`Spec hash:`, `Current AC:`, `## Detours` between Hard Constraints and Acceptance Criteria, gate annotations `- [ ] ac-01 (gate): …`). This card consumes the layout; TaskUpdate parser and the resume-hook parser must skip `## Detours` when determining AC status. Resolution: schema owned by 0009 per rally coordination note; this card references 0009's Decision 5 as a hard dependency.
- **Shared progress.md parser (`parse-progress.sh`).** Both 0003's resume hook (rebuild tasks) and 0009's session-context hook (surface "next unchecked AC") need the same markdown parse. Resolution: propose a single shared parser, owned and authored under 0009; 0003 consumes it. 0009 Decision 5 already flags schema authority sitting there.
- **Gate AC dependency semantics (D1).** This card emits flat task nodes; card 0009 Decision 2 + Decision 6 define gate semantics (blocking ACs by declaration order) and enforce them at runtime inside the implement skill. Resolution: 0009 owns gate behaviour; this card leaves `addBlockedBy` unwired so 0009 can layer it in additively without reshaping task creation.
