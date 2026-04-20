# Discovery: Rally Sub-Agent Enforcement & Parallel Execution Model

**Date:** 2026-04-19
**Interviewer:** Nightingale
**Card:** cards/0006-rally.yaml
**Mode:** discovery
**Triggered by:** review-pr findings F-03 and F-05 on spec `specs/2026-04-19-rally/spec.yaml` v2.0

---

## Context

The first-cut rally SKILL.md made two claims that do not survive a cold reading:

**F-03 (ac-06)** — "Constrain sub-agent writes via the tools allow-list." Claude Code's Agent `tools` parameter is a tool-name allow-list (`Read`, `Edit`, `Write`, …), not a per-path scope. There is no native primitive that restricts `Write` to a specific directory. The sub-agent is told via prompt to write only to its assigned `spec_dir` — that is trust, not enforcement. ac-06's verification step ("verify the write is blocked") cannot be satisfied by the named mechanism.

**F-05 (§7 parallel implementation)** — "Parallel sub-agents follow drive's stage logic inline." Drive's guided and supervised modes both use `AskUserQuestion` for human gates. Sub-agents launched via the Agent tool cannot present interactive questions to the user mid-flight — they return a single final response. Rally simultaneously refuses `full` autonomy at the rally level, leaving no viable mode for a parallel sub-agent to actually run drive under.

Both findings are structural, not incidental. This session resolves the honest model.

## Q&A

### Q1: Path enforcement for design sub-agents
**Q:** How should rally enforce that a design sub-agent only writes to its assigned spec_dir, given Claude Code's `tools` parameter is a tool-name allow-list, not a path scope?
**A:** **Trust + post-verify.** The brief names the assigned path; the lead agent verifies after return that `interview.md` exists at the expected path and rejects any writes observed outside. No mid-flight enforcement is attempted. This is honest about what Claude Code actually provides. ac-06 changes from "writes are blocked outside spec_dir" to "writes outside spec_dir are detected and rejected on return."

### Q2: Parallel sub-agents and drive's interactive gates
**Q:** How should parallel cards actually run, given sub-agents can't present interactive gates?
**A:** **Sub-agents run drive in `full` autonomy internally.** The contradiction in the original spec dissolves once you realise sub-agents can themselves spawn forked Agents for review-spec and review-pr — the same context-separation pattern drive uses at the top level, just nested one level deeper. Drive's `full` mode is entirely non-interactive (self-answers design questions, review verdicts handled programmatically, APPROVE proceeds directly to completion). That's exactly the mode a sub-agent can run. Rally-level gates (proposal, consolidated decision gate, consolidated design review, batched diff review) remain interactive with the lead. Rally-level `supervised` autonomy means "pause between rally phases," not "pause inside each card's drive."

### Q3: Thin-card guard
**Q:** Drive-full refuses cards with fewer than 3 scenarios and explicitly forbids silent downgrade. What should rally do when a parallel-eligible rally contains a thin card?
**A:** **Refuse at proposal.** The rally's proposal gate checks scenario counts. If any proposed card has <3 scenarios, the proposal flags it and asks the author to either thicken the card with `/orb:card` or remove it from the rally. Earliest possible failure, cleanest mental model. No mid-rally execution-model switching.

### Q4: rally.yaml ownership across parallel worktrees
**Q:** Sub-agents run in separate worktrees, but rally.yaml is in the main checkout. Who owns writes?
**A:** **Lead owns all writes.** rally.yaml lives only in the lead's checkout. Sub-agents are briefed with card, interview.md, and spec_dir — they never see or write rally.yaml. Sub-agents report progress by returning messages; the lead serialises all rally.yaml writes on completion events. One source of truth, no races, worktrees stay focused on code.

**Why this works without locks:** combined with Q5's fire-and-forget model, each sub-agent triggers exactly one rally-level status transition (implementing → complete or implementing → parked) at return time. The lead receives returns serially and writes rally.yaml N times total, never concurrently. Per-card progress files and file locking both solve races that do not exist in this architecture.

**Ontology tweak required:** Each card in rally.yaml needs a `worktree` field (absolute or repo-relative path) so the lead can find each sub-agent's checkout on resumption — the lead reads rally.yaml for coordination, then each worktree's drive.yaml for sub-stage. The current ontology has `branch` but not `worktree`.

### Q5: Mid-flight progress tracking
**Q:** How should the lead track parallel sub-agent progress during long runs?
**A:** **Fire-and-forget + completion events.** Sub-agents launched with `run_in_background: true`. Lead waits for each sub-agent's completion notification; task list shows each card as "implementing" from launch until return. On completion, the lead reads the returned verdict, updates rally.yaml and TaskList. No mid-flight stage granularity. Minimal surface area, and rally.yaml stays the single state authority — no parallel state-tracking mechanism grows up beside it.

---

## Summary

### Goal
Rally's sub-agent orchestration must honestly describe what Claude Code actually provides: no invented path-enforcement primitive, no interactive gates inside non-interactive contexts, no state ownership that races.

### Constraints (resolved model)
- Design sub-agents run under **trust + post-verify** path discipline — brief names the path, lead verifies on return
- Parallel implementation sub-agents run **drive in `full` autonomy** inside their worktree; drive's review-spec and review-pr spawn their own nested forked Agents (recursive context separation, not self-review)
- Rally refuses at proposal any rally containing a card with <3 scenarios when parallel is proposed — no silent downgrade
- rally.yaml is written **only by the lead**; sub-agents report status via return messages
- Parallel sub-agents run **fire-and-forget in background**, lead reacts to completion events; no mid-flight stage streaming
- Rally-level `supervised` means "pause between rally phases"; within a card, drive-full runs straight through

### Success Criteria
- SKILL.md §4a removes the "tools allow-list" claim and names trust + post-verify verification explicitly
- SKILL.md §7 Parallel Implementation states that sub-agents run drive-full, names the nested-fork pattern for review stages, and explains why rally-level autonomy is independent of drive-level autonomy
- SKILL.md §2 Propose the Rally adds a thin-card guard that runs before approval
- SKILL.md §3 / §7 explicitly state that sub-agents do not read or write rally.yaml
- ac-06 rewritten: "verified: writes land only at the expected spec_dir; unexpected writes are detected on return and the sub-agent's output is rejected"
- ac-14 rewritten: "parallel sub-agents run drive-full in worktrees; each sub-agent's review-spec and review-pr run as nested forked Agents; supervised/guided distinction applies only at rally-level gates"
- Ontology adds `cards[].worktree` field (path to the sub-agent's checkout, populated at parallel launch, null for serial cards)

### Decisions Surfaced
- **Path discipline is trust-based, not enforced** — chose post-verify over shipping a PreToolUse hook or dropping ac-06. Keeps orbit's surface area lean and honest about Claude Code's primitives. (→ `decisions/0004-rally-subagent-path-discipline.md`)
- **Parallel sub-agents run drive-full; reviews nest** — chose nested-fork over drive-without-reviews or serial-only. The forked-reviewer pattern is already the top-level drive's review mechanism; using it recursively preserves context separation inside sub-agents. (→ `decisions/0005-rally-parallel-drive-full.md`)
- **Rally refuses thin cards at proposal for parallel rallies** — chose refuse over mixed execution models or full-rally refusal. Mixed models create two mental models in one rally; full-rally refusal is too conservative when thin cards can be lifted out. (→ `decisions/0006-rally-thin-card-guard.md`)
- **Lead owns rally.yaml writes** — chose centralised over distributed progress files or file-locking across worktrees. rally.yaml stays single-source-of-truth; worktrees stay code-focused.
- **Fire-and-forget background sub-agents** — chose completion events over streaming monitors or sentinel files. Keeps rally.yaml the only state layer.

### Open Questions
- **Verify-on-return implementation detail (ac-06 rewrite).** After a sub-agent returns, the lead needs to scan for writes outside the assigned `spec_dir`. Options: (a) check git status across the repo and fail if unexpected paths are touched; (b) snapshot the file tree before sub-agent launch and diff on return; (c) rely on sub-agent self-report of files written. This is a spec-level detail, not a discovery decision.
- **Thin-card guard in serial rallies.** The guard is strictly necessary for parallel (drive-full needs ≥3 scenarios). Serial rallies run drive-guided, which accepts thin cards. Should rally warn but allow, or stay quiet? Probably warn — rally is ideation-to-assurance, and a thin card is weak ideation.
- **Drive's escalation path when running inside a sub-agent.** Drive-full exhausts its 3-iteration budget and writes an escalation summary. In a rally sub-agent, that summary is returned to the lead. Does the lead treat budget-exhausted escalation as a park (single-strike), or does the sub-agent's internal iteration count as "the 3 iterations already happened, park is correct"? The latter is consistent with the single-strike NO-GO policy. Should be stated explicitly in the spec update.
