# Design: drive live visibility

**Date:** 2026-04-20
**Interviewer:** nightingale (rally sub-agent)
**Card:** orbit/cards/0005-drive.yaml

---

## Context

Card: *Agent-driven card delivery* — 13 scenarios, goal: agents deliver cards end-to-end within declared autonomy bounds
Prior specs: 1 — `orbit/specs/2026-04-15-drive/spec.yaml` shipped the five-stage inline orchestrator (design → spec → review-spec → implement → review-pr), with drive.yaml as master state, forked reviews, budgeted RFQ_CHANGES cycles (hardcoded at 3), and file-presence resumption via `session-context.sh`.
Gap: the shipped drive has no live-visibility story for long unattended runs. Four new scenarios on the card — cron check-in at start, escalation ping, check-ins surviving resume, and a 4-option verdict via AskUserQuestion — extend drive with Claude Code tool primitives (CronCreate, CronList, CronDelete, AskUserQuestion) so authors can see a heartbeat during full-autonomy drives and make informed verdict decisions at MEDIUM+ review gates. Shipped drive behaviour is out of scope here.

## Q&A

### Q1: Check-in interval default and configurability
**Q:** The new scenario says check-ins fire "every N minutes" but does not fix N. Too frequent (≤1 min) burns context on a long drive; too infrequent (≥10 min) fails the "is this still running?" test that motivates the scenario. Should the interval be 5 minutes hardcoded, 10 minutes hardcoded, or 5 minutes default with a `drive.yaml.checkin_interval_minutes` override?
**A:** 5-minute interval, hardcoded (no configuration surface). Grounded in drive's existing discipline — the iteration budget and review cycle budget are both hardcoded at 3, and *defaults are opinions; configuration is overhead until proven otherwise*. 5 minutes sits inside every author's "is it dead?" window while keeping total check-in volume under two dozen per worst-case 3-iteration drive. If real usage shows 5 min is wrong, raise it later in one edit.

### Q2: Check-in content — strict one-line vs structured
**Q:** The scenario specifies "one-line status (current stage, current AC, iteration count, time elapsed)." The AC is only well-defined during Implement; the other stages have no per-AC cursor. Should the check-in be strict one-line with `ac=-` when absent, one-line plus an optional second line on stage transitions, or a 3–5 line structured block?
**A:** Strict one-line, always. Format: `drive: iter=<N>/<budget> stage=<status> ac=<id|-> elapsed=<mm:ss>`. Dash-substitute the AC when there is none — same idiom `session-context.sh` already uses for empty values. Stage transitions appear naturally through normal drive output between check-ins, so an optional second line is redundant. The check-in is a read-only observability hook: it **must not modify drive.yaml** and **must not launch any Agent**, which keeps it safe to fire mid-forked-review.

### Q3: Cron task lifecycle across resume
**Q:** Should drive create one persistent cron task and trust Claude Code's `--resume` / `--continue` restoration, delete-and-recreate on every resume via CronList, or use an idempotent CronList-first pattern that creates only if missing?
**A:** Idempotent via CronList (option C). This matches drive's existing resumption philosophy in §11.5: *"File presence overrides drive.yaml status when they disagree"* — the harness state is authoritative, drive inspects and reconciles rather than assuming. At drive start (§2), after writing drive.yaml: `CronList` → if no task matches the drive-specific ID, `CronCreate` a recurring check-in with ID `drive-checkin-<spec-slug>`. If a task with that ID already exists (restored from a prior session), no-op. At completion or escalation (§9, §10): `CronDelete drive-checkin-<spec-slug>`; failure to delete is non-fatal — log and continue. This neatly covers the "Check-ins survive resume" scenario: the first thing drive does on a resumed session is §2, which runs the CronList check and leaves the restored task alone.

### Q4: Verdict AskUserQuestion — 3 options vs 4, and the "read full review" slot
**Q:** The new scenario literally names four options (`approve / request-changes / block / read-full-review`). Today's §7.5 guided gate uses three options including a "Let me read the reviews first" escape hatch; rally (card 0006) uses three options (approve-all / select-subset / decline). Should drive's verdict prompt be literal 4 options flat, 3 options with mandatory prior reading, or 4 options with an explicit re-prompt contract on the deferral?
**A:** 4 options with an explicit re-prompt contract (option C). Honour the author's requirement (the scenario names four options) while documenting one sentence in SKILL.md: "On `read-full-review`, wait for the author's next turn (they may ask follow-up questions or signal ready), then re-present the same four-option prompt verbatim." This matches the already-shipped §7.5 "Let me read the reviews first" behaviour. Labels: `approve`, `request changes`, `block`, `read full review first` — lower-case, spaces between words, consistent with other orbit prompts. Applicability scope: the four-option prompt fires only when a spec review or PR review has surfaced findings at **MEDIUM+** severity; LOW-only findings still go through the existing three-option guided APPROVE path, keeping the routine "no-issue approve" path fast.

### Q5: Escalation ping — one-shot CronCreate vs normal return output
**Q:** The shipped §9 already emits a multi-line "DRIVE ESCALATED" summary. The new scenario asks for an *additional* one-shot CronCreate ping so the author notices "even in a long session." Should drive rely on §9's existing output, fire a one-shot CronCreate with identical content immediately, or fire a short distinct ping 30–60 seconds after escalation that references the prior output?
**A:** Short distinct ping, ~30-second delay (option C). Grounded in the scenario's own wording: the emphasis is on *notice*, not *detail*. A short distinct message is more likely to surface through notification pathways (terminal bell, OS notification) than a long wall-of-text that dilutes the signal. The 30-second delay gives the full §9 output time to render first so the ping's "see prior output" reference is accurate. Message shape: `**DRIVE ESCALATED** on <card-slug> after <iterations> iterations. See prior output for findings and recommendation.` Bold heading because the scenario says "prominent" and markdown bold is the available emphasis mechanism. One-shot, not recurring — drive does not nag.

### Q6: CronList at SessionStart — inspection, reconciliation, or out-of-scope?
**Q:** Should the SessionStart hook / skill body inspect and display active drive cron tasks on resume, reconcile them against drive.yaml (e.g., warn when status=complete but a drive-checkin is still active), or leave this entirely to §2's existing reconciliation?
**A:** Out of scope for this card (option A). D3 already establishes that drive's §2 runs CronList on every initialise/resume to reconcile the check-in task against drive.yaml — that is the right layer because CronList is an agent tool and `session-context.sh` is a shell script that cannot invoke it. Adding reconciliation at the shell-hook layer or as a separate visible skill step duplicates effort for marginal observability benefit, and SessionStart output is already getting crowded (rally display, drive display, constraints). The four scenarios under review do not require a visible cron inspection step. If a future scenario asks for explicit cron observability ("show me all scheduled drive tasks"), reopen with a dedicated skill verb (e.g. `/orb:drive status`) — not this card.

---

## Summary

### Goal
Extend `/orb:drive` with live-visibility and structured-verdict primitives so that full-autonomy runs emit a lightweight heartbeat, escalations surface prominently, check-ins survive session resume, and MEDIUM+ review verdicts route through a structured four-option prompt — all without changing shipped drive behaviour.

### Constraints
- Check-in cadence is hardcoded at 5 minutes; no configuration surface.
- Check-in output is strictly one line: `drive: iter=<N>/<budget> stage=<status> ac=<id|-> elapsed=<mm:ss>`.
- Check-in prompts are read-only: must not modify drive.yaml and must not launch any Agent.
- Cron task ID is `drive-checkin-<spec-slug>`; created idempotently via CronList at §2, deleted at §9 / §10. CronDelete failure is non-fatal — log and continue.
- Four-option verdict prompt fires only when review findings are at MEDIUM+ severity; LOW-only verdicts keep the existing three-option guided APPROVE path.
- Verdict labels: `approve`, `request changes`, `block`, `read full review first` (lower-case, spaces).
- On `read full review first`, wait for the author's next turn, then re-present the same four-option prompt verbatim.
- Escalation ping is one-shot (not recurring), fires ~30 seconds after §9 output, message: `**DRIVE ESCALATED** on <card-slug> after <iterations> iterations. See prior output for findings and recommendation.`
- Session-context shell hook is unchanged by this card; all cron reconciliation lives in the agent-side skill body.
- Scope is strictly the four new scenarios; shipped drive behaviour (autonomy levels, NO-GO handling, forked reviews, drive.yaml lifecycle) is unchanged.

### Success Criteria
- A full-autonomy drive schedules a recurring `drive-checkin-<spec-slug>` cron task at §2 and emits a one-line heartbeat every 5 minutes until completion or escalation.
- The heartbeat renders the correct values for `iter`, `stage`, `ac` (or `-`), and `elapsed` at every firing, including during stages where no current AC exists.
- After `--resume` / `--continue`, the restored check-in task continues firing on schedule; §2's CronList reconciliation is a no-op when the task is already present and creates the task if it was lost.
- On budget exhaustion, §9 emits its existing escalation summary, then a one-shot `DRIVE ESCALATED` ping fires ~30 seconds later referencing the prior output.
- On completion or escalation, `CronDelete drive-checkin-<spec-slug>` is attempted; failure does not abort §10.
- When a review produces MEDIUM+ findings, the verdict prompt is an AskUserQuestion with exactly four options (`approve`, `request changes`, `block`, `read full review first`); LOW-only APPROVE paths retain the existing three-option prompt.
- Selecting `read full review first` yields the same four-option prompt verbatim on the author's next turn.

### Decisions Surfaced
- **D1 — Check-in interval:** chose 5 minutes hardcoded over 10 minutes hardcoded or a configurable override, because drive's existing discipline hardcodes budgets and *defaults are opinions; configuration is overhead until proven otherwise*.
- **D2 — Check-in content:** chose strict one-line (`drive: iter=<N>/<budget> stage=<status> ac=<id|-> elapsed=<mm:ss>`) over an optional second line on transitions or a structured multi-line block, honouring the scenario text and using the existing `session-context.sh` dash idiom for absent values.
- **D3 — Cron task lifecycle:** chose idempotent CronList-first (create only if missing, delete at completion/escalation, non-fatal on CronDelete failure) over blind persistent creation or delete-then-recreate, matching §11.5's "file presence overrides drive.yaml status" reconciliation philosophy.
- **D4 — Verdict AskUserQuestion:** chose four options with an explicit re-prompt contract on `read full review first` over collapsing to three options or accepting a deferral black hole; applicability is MEDIUM+ findings only, preserving the fast LOW-only APPROVE path.
- **D5 — Escalation ping:** chose a short distinct one-shot ping with ~30-second delay over no ping or a content-duplicating immediate ping, prioritising *notice* over *detail* and letting §9's full output render first.
- **D6 — CronList at SessionStart:** chose out-of-scope over adding a skill-body cron status block or a shell-hook nudge, because §2's agent-side reconciliation (D3) is the correct layer and SessionStart output is already dense.

### Open Questions
Three items noted for the consolidated design review (accepted, not blockers):

- **Reconciliation with rally card 0006 on AskUserQuestion patterns.** Both cards standardise on suggested-answer prompts but with different cardinalities (3 vs 4). The consolidated review should document this as a *deliberate* choice (drive's verdict has a "read first" deferral state that rally's admission decision lacks) rather than an inconsistency to flatten.
- **Interaction with forked reviews (§5, §7) when a check-in fires during a fork.** The Agent-tool call is synchronous from the parent's perspective; a cron task firing mid-fork will be serviced by the parent after the fork returns, so check-ins may skew slightly late during long review forks. Acceptable — the `elapsed` field carries the truth. Document as an expected characteristic.
- **CronDelete failure mode on completion.** If `CronDelete drive-checkin-<slug>` fails (e.g. task already expired), drive should log and continue, not abort completion. State explicitly in the §10 completion step when the spec is written.
