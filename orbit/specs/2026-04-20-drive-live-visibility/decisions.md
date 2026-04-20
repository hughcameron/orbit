# Decision Pack — drive live visibility (card 0005)

**Scope:** Four new scenarios added to `orbit/cards/0005-drive.yaml` this session that extend `/orb:drive` with Claude Code tool primitives:

1. "Drive schedules a check-in at start" — recurring CronCreate one-line status
2. "Escalation fires a one-shot ping" — one-shot CronCreate on budget exhaustion
3. "Check-ins survive resume" — `--resume` / `--continue` restoration behaviour
4. "Review verdicts route via structured choice" — AskUserQuestion with approve / request-changes / block / read-full-review

This pack covers only these four scenarios. Shipped drive behaviour (autonomy levels, NO-GO handling, forked reviews, drive.yaml lifecycle) is out of scope.

**Coordination note:** Card 0006 (rally) also introduces AskUserQuestion for approval gates. Design is independent here; the consolidated design review reconciles any collisions. The recommendation in D4 below is deliberately written so that it composes with rally's three-option pattern (approve-all / select-subset / decline) rather than conflicting with it.

---

## Evidence baseline

- Shipped drive skill: `plugins/orb/skills/drive/SKILL.md` v0.2.19 — five-stage inline orchestrator with forked reviews (§5, §7), drive.yaml as master state, budgeted RFQ_CHANGES cycles (§5a), file-presence resumption (§11).
- Shipped session hook: `plugins/orb/scripts/session-context.sh` already parses `drive.yaml` (lines 133–175) and prints `card / autonomy / iteration / status / next action` on SessionStart.
- Prior spec (shipped): `orbit/specs/2026-04-15-drive/spec.yaml` — constraints explicitly include "no sub-agent spawning for pipeline stages" (since superseded by forked reviews). No mention of cron or in-session polling.
- Existing AskUserQuestion usage in drive: §4 supervised spec gate, §5a.supervised post-APPROVE gate, §6 supervised implement gate, §7.5 guided final gate with exactly three suggested answers including a "Let me read the reviews first" escape hatch.
- CHANGELOG 0.2.16 entry: "Reviews ARE the quality gates. The only interactive pause is a rich final summary … `'Let me read the reviews first'` is an explicit option." — direct precedent for the fourth option in D4.
- Rally card (0006) scenario "Rally approval is a structured prompt" uses three options: approve-all / select-subset / decline. Different decision point (rally admission vs per-review verdict) so no exact symmetry is required, but the three-option default is a useful consistency anchor.
- Claude Code tools reference (local understanding): `CronCreate` schedules a recurring or one-shot prompt within the session; tasks are session-scoped and restored on `--resume` / `--continue` if not expired. `CronList` enumerates active scheduled tasks in the current session. `CronDelete` removes a task by ID. Each fired prompt is injected as a new user message and produces a normal model turn — so tasks consume context and tokens.

---

## D1. Check-in interval default and configurability

**Context.** The new scenario says check-ins fire "every N minutes" but does not fix N. The right default has to balance two failure modes:

- Too frequent (≤1 min): every check-in injects a user-message-equivalent turn. On a long drive with forked reviews (which already consume large chunks of context), this burns budget fast and drowns out substantive output. A 3-hour drive at 1-minute cadence = 180 status prints.
- Too infrequent (≥10 min): defeats the scenario's premise. If the author opens the terminal expecting a heartbeat and sees nothing for 8 minutes, the check-in has failed the "is this still running" test.

Full-autonomy drives are the only mode where this matters (guided and supervised already pause interactively at gates). Typical full-mode drive durations, based on the shipped pipeline (design self-answered → spec → forked review-spec → implement → forked review-pr), are 10–45 minutes per iteration; a 3-iteration budget worst-case is ~2 hours.

### Options

- **A. 5-minute interval, hardcoded.** One sentence in SKILL.md: "Schedule a recurring CronCreate task at 5-minute cadence." No configuration surface.
- **B. 10-minute interval, hardcoded.** Lower context cost, still inside the "did it die?" patience window for most authors.
- **C. Default 5 minutes, configurable via `drive.yaml.checkin_interval_minutes` (integer, optional).** Same default as A, but drive.yaml can override it on first write or on resume.

### Trade-offs

```
| Option | Context cost (3-iter worst case) | Heartbeat latency | Surface area |
|--------|----------------------------------|-------------------|--------------|
| A      | ~24 check-ins, low               | 5 min             | zero config  |
| B      | ~12 check-ins, lower             | 10 min            | zero config  |
| C      | tunable                          | tunable           | +1 field     |
```

### Recommendation — **A (5-minute, hardcoded)**

Grounded in the project's configuration discipline: budget is hardcoded at 3 (shipped spec constraint line 9 "Budget is hardcoded, not configurable"), review cycle budget is hardcoded at 3 (§5a), and the escalation triggers are literal. Drive's existing design philosophy says *defaults are opinions; configuration is overhead until proven otherwise*. 5 minutes is inside every author's "is it dead?" window while keeping total check-in volume under two dozen per worst-case drive. If real usage shows 5 min is wrong, raise it later in one edit.

---

## D2. Check-in content: one-line vs structured vs both

**Context.** The scenario specifies "a one-line status (current stage, current AC, iteration count, time elapsed)." Literal reading gives us the content; the open question is *format* and *how the AC is derived mid-stage*.

Relevant facts:
- "Current AC" is only well-defined during Implement (progress.md has the checklist and clearly identifies the in-progress AC). During Design, Spec, Review-Spec, and Review-PR, there is no per-AC cursor — the whole stage is underway.
- drive.yaml carries `status`, `iteration`, `current_spec`, and `started` — all four directly readable by a cron-fired prompt.
- Check-in prompts ARE model turns: the cron task is a prompt that the model responds to. So "print a one-line status" means "the model reads drive.yaml and progress.md and emits one line."

### Options

- **A. Strict one-line, always.** Format: `drive: iter=<N>/3 stage=<status> ac=<id-or-->  elapsed=<mm:ss>` — dash when no current AC.
- **B. One-line with optional second line for notable events.** Second line only when the check-in coincides with a stage transition or a REQUEST_CHANGES (read from recent drive.yaml diff).
- **C. Structured block (3–5 lines): stage, iteration, current AC, last event, elapsed.** Richer context, always printed.

### Trade-offs

```
| Option | Readability at a glance | Context cost | Ambiguity when no AC |
|--------|-------------------------|--------------|----------------------|
| A      | high                    | lowest       | `ac=-` is clear      |
| B      | high (most), medium (transition) | low–medium | handled |
| C      | medium (wall of text)   | highest      | fewer blanks         |
```

### Recommendation — **A (strict one-line)**

The card's scenario says "one-line status." Honour it. Dash-substitute the AC when there is none (`ac=-`) — this is the same idiom session-context.sh already uses for empty values. Format: `drive: iter=<N>/<budget> stage=<status> ac=<id|-> elapsed=<mm:ss>`. Short enough to scan in a glance; unambiguous; doesn't require the check-in prompt to diff drive.yaml for "events." Stage transitions will appear naturally through the normal drive output between check-ins, so B's optional second line is redundant.

Secondary: specify that the check-in **must not modify drive.yaml** and **must not launch any Agent** — it is a read-only observability hook. This keeps it safe to fire mid-forked-review.

---

## D3. Cron task lifecycle — one persistent task, or recreated on resume?

**Context.** Two intertwined questions:

1. Does drive create one cron task per drive and leave it running until completion/escalation, or re-create on every stage transition?
2. On `--resume` / `--continue`, Claude Code restores unexpired tasks. What does drive do at that moment — trust the restored task, list existing tasks with `CronList` first, or always schedule fresh and delete any leftovers?

The scenario "Check-ins survive resume" explicitly relies on Claude Code's task-restoration behaviour. We must not defeat it by aggressively deleting and re-creating.

### Options

- **A. One persistent recurring task; never recreate.** Created at drive start (§2 initialise) with a stable task ID derived from drive.yaml (e.g., `drive-checkin-<spec-slug>`). On resume, trust that the task is restored. On completion or escalation, `CronDelete` it.
- **B. Recreate on every resume; delete-then-create using CronList as inspection.** On resume: `CronList` → if a `drive-checkin-*` task exists, `CronDelete` it; then `CronCreate` a fresh one. Guarantees a running task after resume even if restoration failed.
- **C. Idempotent schedule: CronList on every SessionStart, create only if missing.** On resume: `CronList` for `drive-checkin-<slug>`; if present, no-op; if absent, `CronCreate`. Avoids duplication, repairs a missing task, never deletes a good one.

### Trade-offs

```
| Option | Dup-task risk | Repair-missing | Complexity | Trust in harness |
|--------|---------------|----------------|------------|------------------|
| A      | low (restore works) → medium (if it doesn't) | no | low | high |
| B      | very low      | yes            | medium     | low              |
| C      | very low      | yes            | medium     | calibrated       |
```

### Recommendation — **C (idempotent via CronList)**

This matches drive's existing resumption philosophy: §11.5 says "File presence overrides drive.yaml status when they disagree." The same discipline applies to cron tasks — the harness state is authoritative, drive inspects it and reconciles rather than assuming. CronList is cheap. The pattern is:

- At drive start (§2), after writing drive.yaml: `CronList` → if no task matches the drive-specific ID, `CronCreate` the recurring check-in with ID `drive-checkin-<spec-slug>`. If a task with that ID already exists (restored from a prior session), no-op.
- At completion or escalation (§9, §10): `CronDelete drive-checkin-<spec-slug>`. Failure to delete is non-fatal — log and continue.

This also neatly covers the edge case the card explicitly names ("Check-ins survive resume"): the first thing drive does on a resumed session is §2, which runs the CronList check and leaves the restored task alone.

---

## D4. AskUserQuestion for verdict routing — 3 options vs 4, and the "read full review" slot

**Context.** The new scenario says the verdict prompt uses options `approve / request-changes / block / read-full-review`. Four options. Drive today (§7.5 guided-mode gate) already uses AskUserQuestion with three options including `"Let me read the reviews first"`. Rally (card 0006) uses three options (approve-all / select-subset / decline).

Three forces are in play:
1. AskUserQuestion supports multiple choice — there is no hard cap, but more options = more cognitive load and more scrolling.
2. The card-level scenarios are the author's expressed requirements. The scenario literally names four options.
3. "Read full review" is *not a verdict* — it's a deferral. Approving / request-changes / blocking are terminal verdict routes. Conflating them in the same enum risks the author picking "read full review" and then never returning.

### Options

- **A. Literal 4 options, flat list.** `approve / request-changes / block / read-full-review`. Selecting "read full review" re-presents the same 4-option prompt after the author responds.
- **B. 3 options + mandatory prior reading.** Drive always surfaces the review path and a one-paragraph summary *before* the AskUserQuestion, and the prompt itself offers only the three terminal verdicts. No "read full review" option because reading is already expected.
- **C. 3 terminal verdicts + 1 deferral, with a clear re-prompt contract.** Same as A, but SKILL.md explicitly documents: "If the author picks read-full-review, wait for their next turn, then re-present the exact same 4-option prompt." This matches how §7.5 handles "Let me read the reviews first" today.

### Trade-offs

```
| Option | Matches scenario text | Cognitive load | Risk of "read" black hole |
|--------|-----------------------|----------------|----------------------------|
| A      | yes                   | medium         | medium (no contract)       |
| B      | no (3 not 4)          | low            | low                        |
| C      | yes                   | medium         | low (explicit re-prompt)   |
```

### Recommendation — **C (4 options, explicit re-prompt contract)**

The card's scenario names exactly four options. Honour the author's requirement. But extend the contract with a single sentence in SKILL.md: "On `read-full-review`, wait for the author's next turn (they may ask follow-up questions or signal ready), then re-present the same four-option prompt verbatim." This matches the already-shipped §7.5 "Let me read the reviews first" behaviour, so it is neither new nor surprising.

**Label polish:** match casing/tone to other orbit prompts. Suggested labels: `approve`, `request changes`, `block`, `read full review first` — all lower-case, spaces between words, consistent with the rally card's `approve-all` style reading but without the hyphens (no namespace collision need).

**Applicability scope:** the scenario says "a spec review or PR review has surfaced findings at MEDIUM+ severity." This needs to be stated explicitly in SKILL.md — LOW-only findings still go through the simpler guided APPROVE path. The existing §7.5 gate already fires unconditionally on APPROVE in guided mode, so this new four-option prompt replaces the current three-option one *when any finding is MEDIUM+*; otherwise the existing three-option prompt stays. This keeps the routine "no-issue approve" path fast and reserves the richer prompt for cases where the author actually needs to make a judgement.

**Consistency with rally:** rally (0006) uses three options because its decision space is three-way (accept all / subset / decline). Drive's verdict space is four-way because reading the review is itself a valid deferred state that rally does not have an analogue for. The patterns are consistent in shape (AskUserQuestion with suggested answers, no free-text) even though the cardinalities differ.

---

## D5. Escalation ping — one-shot CronCreate vs normal return output

**Context.** The shipped escalation path (§9) already produces a multi-line output summary ("DRIVE ESCALATED — …"). The new scenario asks for an *additional* one-shot CronCreate ping so the author notices "even in a long session." This matters specifically for full-autonomy drives where the author may be looking elsewhere while the drive grinds.

Two questions:
1. Is the one-shot cron actually different in effect from the normal escalation output?
2. If yes — what's its cadence? Fire immediately? Fire once after a short delay to ensure the author sees it?

A cron-fired one-shot message is still injected as a user-message-equivalent in the same session. The author's notification channel depends on their Claude Code harness (terminal bell, OS notification, IDE integration). So "fires a prominent message" is only more prominent than a normal turn if the author's harness treats cron-fired messages specially OR if the message fires while the author is away (longer escalation output would also do this).

### Options

- **A. No cron — rely on the existing escalation summary output.** The scenario is already covered by §9's output; add nothing.
- **B. One-shot CronCreate fired immediately after §9 output, identical content.** Duplicates the message but guarantees delivery via whatever notification pathway the harness wires cron prompts to.
- **C. One-shot CronCreate with a distinctive short ping message, scheduled 30–60 seconds after escalation.** E.g. `DRIVE ESCALATED on <card> — see prior output for summary.` Short enough to surface in a notification if the harness supports it, referencing the full output for detail.

### Trade-offs

```
| Option | Honours scenario | Duplication | Notification reach |
|--------|------------------|-------------|--------------------|
| A      | no               | none        | depends on harness |
| B      | yes              | full        | doubled            |
| C      | yes              | short pointer | broader          |
```

### Recommendation — **C (short distinct ping, 30-second delay)**

Grounded in the scenario's own wording: "fires a prominent drive escalated on <spec> message so the author notices even in a long session." The emphasis is on *notice*, not *detail*. A short distinct ping is more likely to surface through notification pathways (terminal bell, OS notification) than a long wall-of-text that dilutes the signal. The 30-second delay gives the full §9 output time to render first so the ping's "see prior output" reference is accurate.

Message shape: `**DRIVE ESCALATED** on <card-slug> after <iterations> iterations. See prior output for findings and recommendation.` Bold heading because the scenario says "prominent" and markdown bold is the available emphasis mechanism.

One-shot, not recurring. Drive does not nag.

---

## D6. CronList at SessionStart — inspection, reconciliation, or out-of-scope?

**Context.** The session hook (`session-context.sh`) runs on SessionStart and today surfaces drive state from drive.yaml. The new scenarios introduce cron tasks tied to drive. A natural question: should SessionStart inspect cron tasks too?

Two sub-questions:
1. Should the hook *display* active drive cron tasks as part of SessionStart output?
2. Should the hook *reconcile* cron tasks against drive.yaml (e.g., warn if drive.yaml says `status: complete` but a drive-checkin cron is still active)?

Constraint: session-context.sh is a **shell script**, not a Claude-agent context. CronList is a Claude Code tool, invokable only by the agent. So "SessionStart inspects cron" cannot literally happen in the shell hook — it would need to happen in the skill body *after* the agent takes its first turn.

### Options

- **A. Out of scope. Skill body handles all cron reconciliation as part of §2 resumption (D3).** Session hook unchanged.
- **B. Skill body prepends a "cron status" block on resume.** After §11 resumption detection, the agent runs `CronList`, filters to `drive-checkin-*` and escalation tasks, and prints a one-line summary alongside the existing "Resuming drive for <card>" block.
- **C. Session hook flags expected cron state as a nudge.** Shell hook emits `orbit: drive active — expected cron tasks: drive-checkin-<slug>` so the author knows what to look for if something feels off; agent-side reconciliation still happens in §2.

### Trade-offs

```
| Option | Surface area added | Visibility | Redundancy w/ existing output |
|--------|--------------------|------------|-------------------------------|
| A      | none               | lowest     | none                          |
| B      | +1 skill step      | medium     | none                          |
| C      | +1 hook line       | medium     | partial                       |
```

### Recommendation — **A (out of scope for this card)**

D3 already establishes that drive's §2 runs CronList on every initialise/resume to reconcile the check-in task against drive.yaml. That is the right layer for cron reconciliation because CronList is an agent tool. Adding reconciliation at the shell-hook layer (option C) or as a separate visible skill step (option B) duplicates effort for marginal observability benefit, and the SessionStart output is already getting crowded (rally display, drive display, constraints). The four scenarios under review do not require a visible cron inspection step — only that check-ins survive resume (covered by D3) and that escalation pings fire (covered by D5).

If a future scenario asks for explicit cron observability ("show me all scheduled drive tasks"), reopen this decision with a dedicated skill verb (e.g., `/orb:drive status`). Not this card.

---

## Summary table

```
| ID | Decision                          | Recommendation                              |
|----|-----------------------------------|---------------------------------------------|
| D1 | Check-in interval                 | 5 min, hardcoded (Option A)                 |
| D2 | Check-in content                  | Strict one-line, dash for absent AC (A)     |
| D3 | Cron task lifecycle               | Idempotent via CronList (Option C)          |
| D4 | Verdict AskUserQuestion           | 4 options + explicit re-prompt (Option C)   |
| D5 | Escalation ping                   | Short distinct ping, 30s delay (Option C)   |
| D6 | CronList at SessionStart          | Out of scope for this card (Option A)       |
```

## Open items for consolidated design review

- **Reconciliation with rally card 0006 on AskUserQuestion patterns.** Both cards standardise on suggested-answer prompts but with different cardinalities (3 vs 4). Recommend the consolidated review document this as a *deliberate* choice (drive's verdict has a "read first" deferral state that rally's admission decision lacks) rather than an inconsistency to flatten.
- **Interaction with forked reviews (§5, §7) when a check-in fires during a fork.** The Agent-tool call is synchronous from the parent's perspective; a cron task firing mid-fork will be serviced by the parent after the fork returns, so check-ins may skew slightly late during long review forks. Acceptable — the elapsed field will carry the truth. Document this as an expected characteristic.
- **CronDelete failure mode on completion.** If `CronDelete drive-checkin-<slug>` fails (e.g., task already expired), drive should log and continue, not abort completion. State this explicitly in the §10 completion step when the spec is written.
