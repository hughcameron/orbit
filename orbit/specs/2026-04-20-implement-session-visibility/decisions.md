# Decision Pack — implement session visibility (0003, rally UX uplift)

**Card:** `orbit/cards/0003-implement.yaml` (`maturity: established`)
**Rally:** orbit UX uplift via Claude Code tools (shared with 0005, 0006, 0009)
**Scope:** four new scenarios extending the shipped pre-flight skill with Claude Code tool primitives — TaskCreate on checklist generation, TaskUpdate on AC completion, SessionStart rebuild from progress.md, Monitor-backed test streaming. The shipped behaviours (read spec, write progress.md, SessionStart constraint surfacing) are out of scope.

**Shared surfaces with 0009 (mission-resilience):**
- `plugins/orb/skills/implement/SKILL.md`
- `plugins/orb/scripts/session-context.sh`
- `progress.md` schema (0009 adds `Detours`, `Return to:`, `current AC`, gate AC semantics; this card consumes those fields but does not define them)

0009 is the likely schema owner. Decisions below flag shared-schema dependencies rather than resolving them.

---

## D1 — Task ↔ AC/constraint mapping granularity

**Context.** The new scenario "Pre-flight checklist populates the session task list" is explicit: `each AC is written as a task via TaskCreate with status "pending", and each hard constraint is a task with status "pending"`. The shipped progress.md already separates `## Hard Constraints` from `## Acceptance Criteria`, and the scenario language demands both kinds be promoted. What remains is how to encode ordering and dependencies.

**Options.**
- **A. Flat, one task per item (AC + constraint) — no dependency edges.** Each AC and each constraint becomes a pending task. No `addBlockedBy` wiring. Task list mirrors progress.md sections 1:1.
- **B. Flat tasks with constraint → AC dependency edges.** Every constraint task `addBlocks` every AC task, so the task list surfaces constraints-must-hold semantics via the scheduler.
- **C. Flat tasks + gate edges reserved for 0009.** AC and constraint tasks are flat; leave `addBlockedBy` wiring for 0009 to populate when it introduces gate ACs. This card only creates nodes; 0009 adds edges.

**Trade-offs.**
- A: simplest and exactly what the scenario text prescribes; loses any mechanical enforcement of "constraint blocks AC".
- B: surfaces constraints as blockers visually, but overstates their semantics — constraints are global invariants, not prerequisite work items. Completing a constraint task is meaningless in that framing, and 0009's real gate ACs would then compete with this fake-gate pattern.
- C: keeps this card's surface narrow (nodes only), gives 0009 a clean slot to add gate edges without reshaping what implement emits. Matches the rally coordination rule "flag shared-schema concerns, not resolve them."

**Recommendation: C.** The card text (`scenarios[Pre-flight checklist populates the session task list]`) only mandates nodes; dependency semantics belong to 0009's gate AC work. Emit flat tasks with no edges, and document the seam so 0009 can layer dependencies in without reworking task creation.

---

## D2 — When TaskCreate fires

**Context.** The scenario ties TaskCreate to "the pre-flight checklist is generated". SKILL.md §3 (Present the Checklist) and §4 (Write the Progress File) are currently adjacent steps. There is a choice about where in that sequence TaskCreate runs.

**Options.**
- **A. At checklist presentation (§3), before progress.md is written.** Tasks appear as the human sees the checklist.
- **B. Immediately after progress.md is written (§4), using progress.md as the source of truth.** Tasks derive from the file, not from the in-memory extraction.
- **C. Lazy — defer TaskCreate to the first progress.md update.** Skip task emission until the agent actually makes progress.

**Trade-offs.**
- A: earliest possible visibility; risk of drift if the checklist text the human sees diverges from what gets written to progress.md (two serialisations of the same data).
- B: single source of truth — progress.md. The task list and progress.md are built from the same parse; they cannot disagree on initial content. Slight delay (one step) before tasks appear. Matches the SessionStart-resume decision (D4) which will also read progress.md — both entry paths use the same parser.
- C: violates the scenario's explicit "when the pre-flight checklist is generated, then each AC is written as a task" wording. Rejected.

**Recommendation: B.** The shipped skill already treats progress.md as the durable record; reusing it as the TaskCreate source keeps the pre-flight and resume paths symmetric (see D4) and eliminates a second serialisation. The scenario's `when` is satisfied because progress.md is written as part of checklist generation.

---

## D3 — TaskUpdate trigger

**Context.** The scenario: "progress.md is updated to mark the AC done ⇒ the corresponding task is updated via TaskUpdate to status 'completed' in the same tool-call turn." The shipped SKILL.md §5 already instructs the agent to update progress.md when an AC is addressed. The question is whether the agent invokes TaskUpdate directly, or whether a mechanism derives TaskUpdates from progress.md writes.

**Options.**
- **A. Skill rule — agent invokes TaskUpdate directly alongside the progress.md edit.** SKILL.md §5 is extended: "when you mark an item `- [x]` in progress.md, call TaskUpdate with status completed in the same turn." Purely prescriptive.
- **B. Post-write hook — a hook watching progress.md diff fires TaskUpdate automatically.** Requires an `after-edit` hook or equivalent.
- **C. Checkpoint trigger only — TaskUpdate fires at explicit "unit complete" moments (step §5 and §6), not on every progress.md write.** Bigger batches, less chatter.

**Trade-offs.**
- A: aligns with how the shipped skill already works (agent-driven discipline); the scenario's `in the same tool-call turn` phrasing fits an agent rule, not a hook. Risk: relies on agent discipline — the exact failure mode the skill is designed to mitigate elsewhere.
- B: fully mechanical, but orbit's current hook inventory is `SessionStart` only (`plugins/orb/hooks/hooks.json`). Adding a `PostToolUse` or file-watch hook is new infrastructure and intersects with 0009's hook work — introduces coordination cost for a behaviour already prescribed by the scenario text.
- C: loses real-time visibility, which is the stated value of the scenario ("in real time"). Rejected.

**Recommendation: A.** The scenario text ("in the same tool-call turn") reads as an agent-authored batch, not an automated hook. A skill rule is low-infrastructure, matches the shipped style (§5 "Update `progress.md`…State which AC(s) were addressed"), and avoids colliding with 0009's hook modifications. Document the discipline risk — if agents routinely forget, revisit with a hook.

**Shared-schema flag:** 0009 adds detour entries to progress.md. TaskUpdate must not mark an AC completed just because a detour entry landed near its header. The parser in D4 (and the rule here) depends on 0009's final section layout — flag for design review.

---

## D4 — Session resume: rebuild vs reconcile

**Context.** Scenario: "the SessionStart hook reads progress.md and rebuilds the session task list so the author sees the same AC-by-AC status that was visible in the prior session." The shipped `session-context.sh` already parses spec.yaml for constraints. Tasks persist across sessions in the harness task store, so on resume there is an existing task list that may or may not match progress.md.

**Options.**
- **A. Unconditional rebuild — delete existing tasks tagged as orbit-implement and recreate from progress.md.** Simple, idempotent if tasks carry a spec-path tag in metadata. Task IDs change on every resume.
- **B. Reconcile — diff existing tasks against progress.md; TaskCreate missing, TaskUpdate state mismatches, leave rest.** Preserves IDs and history, more code.
- **C. Reconcile-or-skip — if tasks exist and match, do nothing; if they don't match (count or status drift), fall back to full rebuild with a warning.** Cheap common path, safe fallback.

**Trade-offs.**
- A: matches "rebuilds the session task list" literally; loses any task-level metadata (comments, owner) that accumulated in the prior session. Cheapest implementation. Churns task IDs, which may break references in the running transcript.
- B: keeps IDs and metadata; requires a stable key to match on (AC ID like `ac-01`, or constraint hash). Implementation surface is a bash-side YAML/markdown diff — nontrivial in `session-context.sh`.
- C: good default behaviour — fast when no drift, safe when there is. Still needs a matcher (shares complexity with B's matcher).

**Recommendation: C.** Scenario wording ("rebuilds") and orbit's bash-first hook style favour the simpler path, but full rebuild on every session start is aggressive given that most sessions resume without drift. A match check on AC ID is cheap; fallback to rebuild keeps the implementation bounded. Store the spec path in task `metadata` so the hook can scope to orbit-implement tasks only.

**Shared-schema flag:** 0009's "next unchecked AC" surfacing (mission-resilience scenario: *Session resume surfaces next AC*) must read the same progress.md parser. Propose one shared parser script (`parse-progress.sh` or awk block) consumed by both cards' hook changes — flag for design review to assign ownership.

---

## D5 — Monitor scope: which commands qualify, failure semantics

**Context.** Scenario: "/orb:implement runs a test suite that takes several minutes ⇒ a Monitor feeds each failing line back to the agent as it appears." The Monitor tool streams stdout lines as events; every line becomes a notification, so filters must be tight (tools-reference: "Every stdout line becomes a message…write selective filters"). "Implement runs a test suite" is underspecified — which invocations trigger Monitor, and what does "failures surface mid-run" do to the loop?

**Options.**
- **A. Opt-in marker in spec metadata.** Add an optional `metadata.test_command` (or reuse `exit_conditions`) that names the long-running command; implement uses Monitor only for that command. Short tests run via Bash as today.
- **B. Heuristic — any test command expected to exceed N seconds uses Monitor.** Skill rule: "if a test is likely to exceed ~60s or is a full-suite run, wrap it in Monitor with a failing-line filter; otherwise use Bash."
- **C. Always use Monitor for the primary test command named in deliverables.** Blanket rule.

**Trade-offs.**
- A: explicit, author-driven, composable with the existing `metadata.test_prefix` pattern. Cost: another spec field to define; risk of being forgotten. Interacts with 0009's schema — flag.
- B: no schema change. Relies on agent judgement for "likely to exceed" — the kind of gut call the implement skill discourages. Simpler to author, weaker to audit.
- C: guaranteed streaming but over-applies Monitor to fast suites where plain Bash captures everything. Also loses the "fail fast on first error" value when a suite finishes in seconds.

**Recommendation: B, with a clarifying filter rule.** Evidence: the scenario's `given` is "takes several minutes", i.e. the trigger is duration, not an explicit spec field. An opt-in field (A) adds schema churn during a rally where 0009 is already reshaping the schema. Mandate the filter in the rule: `grep --line-buffered` for FAIL/ERROR/AssertionError/traceback markers (tools-reference explicitly calls out line-buffering). Document the "fail-fast abort" behaviour in D6.

---

## D6 — Monitor failure → implement loop behaviour

**Context.** The value of streaming is to react before completion. The scenario ("failures surface mid-run rather than on completion") implies the agent should do something with streamed failures. This is not yet specified.

**Options.**
- **A. Surface + continue.** Failing lines become events the agent acknowledges, but the test run continues to completion. Agent reacts after the Monitor exits.
- **B. Surface + checkpoint on first failure.** The Monitor streams until the first FAIL event; the skill rule instructs the agent to pause and decide (fix now vs let suite finish) rather than silently continuing.
- **C. Surface + cancel after N failures.** TaskStop the Monitor once a threshold is hit; fall back to fixing.

**Trade-offs.**
- A: cheapest, minimal new rule surface. Loses the "react mid-run" value almost entirely — what does streaming gain if the agent waits anyway?
- B: delivers the stated benefit. Checkpoint fits the CLAUDE.md "Decision Checkpoints" convention — a first failure is a meaningful branch (fix vs continue). Cost: a rule addition to SKILL.md.
- C: more mechanical; requires picking N and handling noisy suites where cascading failures inflate the count.

**Recommendation: B.** The card's value ("failures surface mid-run rather than on completion") is only realised if mid-run failures change behaviour. A checkpoint-on-first-failure rule is consistent with the shipped skill's "Surface unspecced decisions" and "Assumption reversals require escalation" patterns. Add to SKILL.md §5 alongside the Monitor invocation rule from D5.

---

## Summary of shared-schema flags for consolidated design review

```
| Surface                                  | This card needs          | 0009 likely adds           | Conflict risk                      |
|------------------------------------------|--------------------------|----------------------------|------------------------------------|
| progress.md section layout               | ## Hard Constraints, ## Acceptance Criteria (shipped) | ## Detours with Return to:, current AC marker | TaskUpdate parser must ignore detour lines (D3, D4) |
| progress.md AC marker format             | `- [x] ac-NN: desc`      | gate AC annotation         | D1 defers gate edges to 0009        |
| session-context.sh progress.md parsing   | parse AC status → rebuild tasks (D4) | parse "next AC" + spec-modified detection | Propose shared parser block         |
| Task metadata                            | spec_path scope tag (D4) | (none known)               | Low                                 |
| Spec YAML schema                         | none required if D5=B     | possibly `ac.gate: true`   | D1-C reserves dependency slot       |
```

---

## Out of scope (noted for the spec)

- Changing the shipped pre-flight check or checklist format (card is `established`).
- Defining gate AC semantics or detour logging schema (0009 territory).
- Altering review-pr's consumption of progress.md beyond what the new fields require.
- Cross-card task aggregation at the rally level (mentioned in mission-resilience discovery §5 as "second-order concern").
