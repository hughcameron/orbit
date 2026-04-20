---
status: accepted
date-created: 2026-04-20
date-modified: 2026-04-20
---
# 0009. Rally Parallel Implementation — Drive-Full with Nested Forked Reviews

## Context and Problem Statement

When rally's post-design disjointness check (§6c) confirms no shared symbols, rally enters parallel implementation: N cards are driven concurrently, each in an isolated git worktree. The question is how each parallel sub-agent runs drive internally — in particular, how its review-spec and review-pr stages execute, and what contracts rally relies on versus what drive owns.

Three shapes were on the table once `drive-forked-reviews` (decisions 0004–0007) shipped to main:

- Rally replicates drive's stage logic inside its sub-agent brief (inline review loops, inline verdict parsing, inline budget counters).
- Rally delegates to drive-full unchanged, inheriting drive's fork behaviour transitively. A rally sub-agent runs `/orb:drive <card> full`; drive itself spawns forked reviewers as it always does.
- Rally runs drive in a reduced-autonomy mode (e.g. drive-supervised or drive-guided) inside the sub-agent to reduce the nested-fork depth.

The choice has downstream consequences for honesty (rally's claims about recursion must match what actually happens), for maintenance (duplicated logic drifts), and for the parked_constraint contract (ac-14 depends on drive's escalation-reason labels being reachable from rally.yaml).

## Considered Options

- **Option A: Duplicate drive's stage logic in rally's sub-agent brief.** The brief contains inline review-spec and review-pr instructions, verdict parsing, and budget counters. Rejected: violates the "rally never duplicates drive's stage logic" principle in SKILL.md Integration; drift between rally's copy and drive's source is guaranteed; the coherence scan forbids "drive's stage logic inline" as a phrase for precisely this reason.
- **Option B: Delegate to drive-full unchanged; inherit nested forks transitively.** Rally's sub-agent brief says `run /orb:drive <card> full inside your worktree`. Drive-full's review stages then fork their own Agents per decisions 0004–0007. Rally observes only the sub-agent's top-level JSON return.
- **Option C: Run drive in a reduced autonomy mode (drive-supervised or drive-guided) inside the sub-agent.** Rejected: rally-level autonomy (guided | supervised) already governs rally-phase pauses; reducing drive autonomy inside a parallel sub-agent would reintroduce human touchpoints the rally exists to avoid. Rally's principle is full drive autonomy inside the rally, regardless of serial-or-parallel.

## Decision Outcome

Chosen option: **Option B — delegate to drive-full unchanged; nested forked Agents are transitive.** Rally's parallel sub-agent (Agent tool, `general-purpose`) runs `/orb:drive <card> full`; drive-full's review-spec and review-pr stages fork their own Agents per the contracts in decisions 0004–0007. Rally's only contract with the sub-agent is the final JSON verdict described in §7c.

The four drive-internal contracts rally inherits without redeclaring:

- `0004-drive-verdict-contract` (Drive's Verdict Contract: Strict Canonical Markdown Line) — every forked reviewer writes a single `**Verdict:**` line that drive parses.
- `0005-drive-review-artefact-contract` (Drive's Review Artefact Contract: File-on-Disk Authoritative) — drive reads the review file; the forked chat return is informational.
- `0006-drive-cold-re-review` (Drive's Re-Review Context: Fully Cold) — REQUEST_CHANGES triggers a fully cold re-fork.
- `0007-drive-rerequest-budget` (Drive's REQUEST_CHANGES Budget: 3 Cycles Per Stage) — the 4th would-be cycle synthesises BLOCK and enters NO-GO.

Rally's §9 single-strike park absorbs drive's escalation by reading the sub-agent's returned `reason_label` token and prepending it in brackets to `parked_constraint`. The five tokens (`budget | recurring_failure | contradicted_hypothesis | diminishing_signal | review_converged`) cover drive's four semantic triggers plus the synthetic BLOCK from decision 0007. Unrecognised labels park the card with `[unknown]` — the card parks regardless, so drift is visible rather than silently absorbed.

### Consequences

- **Good:** Rally's SKILL.md §7c is small — it points to drive for reviewer mechanics rather than reproducing them. The coherence scan's forbidden phrase `drive's stage logic inline` continues to hold.
- **Good:** Drive evolution (new verdict tokens, new budget rules) reaches rally's parallel path without a rally-side change. The sub-agent runs whatever drive is on main.
- **Good:** `parked_constraint` in rally.yaml carries drive's semantic label — the same label drive would have surfaced in an individual `/orb:drive` run. Debugging parity between rally-driven and individually-driven cards is preserved.
- **Good:** Claude Code's Agent tool supports nested invocations natively (a `general-purpose` Agent may spawn further Agents). No custom machinery is required; the primitive is real.
- **Bad:** Nested forks have compounding cost — a single rally sub-agent can launch up to 2 (stages) × 4 (1 initial + 3 retries) = 8 forked reviewer invocations before drive escalates. Accepted trade-off: rally is a multi-card mode and the cost is per card, not per rally.
- **Bad:** Failure in a nested fork (e.g. Agent tool transient error) is seen by drive, not by rally. Rally sees only the sub-agent's final verdict. If a nested fork fails in a way drive cannot recover, drive itself will escalate with `reason_label: recurring_failure` or similar, and rally absorbs it via §9. The ac-14 mapping provides this pass-through.

## Related

- orbit/specs/2026-04-19-rally-subagent-model/spec.yaml — ac-06, ac-07, ac-14
- orbit/specs/2026-04-19-rally-subagent-model/progress.md — follow-up PR scope
- plugins/orb/skills/rally/SKILL.md — §7c (parallel implementation mechanics), §9 (escalation label mapping)
- orbit/decisions/0004-drive-verdict-contract.md
- orbit/decisions/0005-drive-review-artefact-contract.md
- orbit/decisions/0006-drive-cold-re-review.md
- orbit/decisions/0007-drive-rerequest-budget.md
- orbit/decisions/0008-rally-subagent-path-discipline.md
- orbit/decisions/0010-rally-thin-card-guard.md
