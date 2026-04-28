---
name: drive
description: Drive a card through the full orbit pipeline — design → spec → review-spec → implement → review-pr — at a declared autonomy level
---

# /orb:drive

Take a card and an autonomy level. Drive the full orbit pipeline (design → spec → review-spec → implement → review-pr) as a single session, tracking state in `drive.yaml` for resumption.

## Usage

```
/orb:drive <card_path> [full|guided|supervised]
```

- `card_path` — path to a card YAML file (e.g. `orbit/cards/0005-drive.yaml`)
- Autonomy level defaults to **guided** if omitted

### Autonomy Levels

| Level | Behaviour |
|-------|-----------|
| **full** | Agent self-answers design questions. All stages run without human interaction. Pauses only for PR merge. Requires ≥3 card scenarios. |
| **guided** | Design interview is interactive. All subsequent stages (spec → review-spec → implement → review-pr) run without intermediate pauses — the reviews ARE the quality gates. Author approves the final review-pr verdict before PR creation. Default. |
| **supervised** | Design interview is interactive. Agent pauses after each stage for author greenlight before proceeding. |

## Instructions

### 1. Pre-Flight: Validate the Card

Read the card at `$ARGUMENTS[0]` (the card path). Parse the autonomy level from `$ARGUMENTS[1]`, defaulting to `guided`.

**Quality gate — thin cards block full autonomy:**

Count the card's `scenarios`. If fewer than 3 scenarios:

- If autonomy is `full`: **REFUSE.** Output a message naming the gap:
  ```
  BLOCKED: Card has N scenario(s) — full autonomy requires ≥3.
  Missing coverage areas to consider:
  - <suggest what scenarios are absent based on the card's goal>
  Add scenarios with /orb:card and retry.
  ```
  Do not proceed. Do not silently downgrade to guided.

- If autonomy is `guided` or `supervised`: proceed (the human is in the loop to compensate for thin requirements).

### 2. Initialise Drive State

Determine the spec directory for this iteration:
- **First iteration:** `orbit/specs/YYYY-MM-DD-<card-slug>/` (derive slug from card filename, e.g. `0005-drive` → `drive`)
- **Subsequent iterations:** `orbit/specs/YYYY-MM-DD-<card-slug>-v<N>/` (e.g. `drive-v2`, `drive-v3`)

Create the directory if it doesn't exist.

**Create `drive.yaml`** in the first iteration's spec directory (this is the master state file for the entire drive):

```yaml
card: <card_path>
autonomy: <full|guided|supervised>
budget: 3
iteration: 1
current_spec: <path to current spec directory>
status: design
history: []
started: <ISO-8601 timestamp>
review_cycles:
  review_spec: 0
  review_pr: 0
review_cycle_dates:
  review_spec: null
  review_pr: null
```

The `review_cycles` and `review_cycle_dates` fields track the REQUEST_CHANGES budget and review-file cycle discipline described in §5a. They are part of the initial write on every fresh drive — their presence is the marker that distinguishes drives running under forked-review semantics from pre-change drives.

**If `drive.yaml` already exists**, read it and **resume from the recorded state** (see §11 Resumption).

**Refusal on pre-change drive.yaml.** If the existing `drive.yaml` has no `review_cycles` field — i.e. it was initialised before forked reviews shipped — do NOT auto-initialise the field and do NOT advance state. Output the refusal message and exit:

```
drive.yaml was initialised before forked reviews shipped; finish the drive under
the prior /orb:drive version, or park and restart from the card.
```

This is the honest migration path: inline-mode and forked-mode must not mix within a single drive. No Agent is launched; drive.yaml is unchanged. If the field is present (written either by a new drive on fresh creation or manually added by an operator taking explicit ownership of the bridge), proceed with the resumption.

#### 2a. Schedule the live-visibility heartbeat (full autonomy only)

After writing (or resuming) drive.yaml, if `autonomy == full`, schedule a recurring check-in that emits a one-line heartbeat every 5 minutes. The heartbeat is the author's "is this still running?" signal during long unattended drives.

**Scope note.** `session-context.sh` (the shell-side SessionStart hook) is NOT modified by this live-visibility work. All cron reconciliation lives here, in the agent-side skill body, because CronList/CronCreate/CronDelete are agent-only tools.

**Reconciliation, not re-creation.** Drive never delete-then-recreates the heartbeat on resume — that would defeat Claude Code's built-in `--resume` / `--continue` task restoration. The pattern is **idempotent CronList-first**:

1. `CronList` — enumerate active cron tasks in the session.
2. If a task with ID `drive-checkin-<spec-slug>` already exists, it was restored by the harness (or created earlier in this session). No-op.
3. Otherwise, `CronCreate` a recurring task with:
   - **ID:** `drive-checkin-<spec-slug>` (where `<spec-slug>` is the basename of `current_spec`, e.g. `2026-04-20-drive-live-visibility`).
   - **Interval:** 5 minutes, recurring. **Hardcoded. No configuration knob.** (Matches drive's existing discipline — iteration budget and review-cycle budget are also hardcoded at 3. *Defaults are opinions; configuration is overhead until proven otherwise.*)
   - **Prompt body:** the exact text below.

**Heartbeat prompt body (verbatim, use this string when calling CronCreate):**

```
This is a drive heartbeat. Read <current_spec>/drive.yaml for iter,
budget, status, and started.

If status is `complete` or `escalated`, call CronDelete with ID
`drive-checkin-<spec-slug>` and emit:

  drive: heartbeat stopped (status=<status>)

Then stop. Do not emit a heartbeat line.

Otherwise: if <current_spec>/progress.md exists, read it to
find the most recent `- [ ] ac-NN` entry (the current AC); if none or not in
Implement stage, use `-`. Compute elapsed as mm:ss since `started`. Emit
exactly one line in the format:

  drive: iter=<N>/<budget> stage=<status> ac=<id|-> elapsed=<mm:ss>

Do not modify drive.yaml. Do not launch any Agent. Emit the single heartbeat
line and stop.
```

The prompt body's contract is load-bearing: the heartbeat fires as a fresh user-message-equivalent turn, and without the explicit "do not modify / do not launch" instruction a helpful agent could mutate state or spawn work mid-forked-review. The contract makes the heartbeat safe to fire at any point in the pipeline, including during a review fork. The one exception is self-termination: when drive.yaml shows a terminal state (`complete` or `escalated`), the heartbeat calls `CronDelete` on itself as a defence-in-depth backstop — §10 and §9 remain the primary cleanup path.

**Heartbeat format.** The literal output format is `drive: iter=<N>/<budget> stage=<status> ac=<id|-> elapsed=<mm:ss>`. When there is no current AC cursor (any stage other than Implement, or Implement before the first AC is marked in-progress), the `ac` field renders a literal `-` character — the same dash idiom `session-context.sh` uses for absent values. Example:

```
drive: iter=1/3 stage=implement ac=ac-04 elapsed=23:17
drive: iter=2/3 stage=review-spec ac=- elapsed=01:42
```

**Non-fatal CronCreate.** If CronCreate fails at this step — harness doesn't support cron, rate limit, transient error — drive logs one line `heartbeat unavailable: <reason>` and continues the pipeline. The heartbeat is an observability affordance; its absence must not block design, spec, implement, or review. Drive never gates stage progression on the heartbeat.

**Autonomy gate.** Guided and supervised drives pause interactively at their own gates and do not schedule the heartbeat. Skip step §2a entirely when `autonomy != full`.

### 3. Stage: Design

Read the design skill instructions from `plugins/orb/skills/design/SKILL.md`. Follow its instructions with these drive-specific adaptations:

**In full mode — agent self-answers design questions:**

The design stage normally uses AskUserQuestion to interview the author. In full mode, the agent answers its own questions using:
1. The card's `scenarios` — these are the author's expressed requirements
2. The card's `goal` — the measurable objective
3. The card's `references` — prior art and evidence
4. Prior iteration history from `drive.yaml` (if this is iteration ≥2, the failure constraints from prior NO-GOs are critical input)

For each design question, write both Q and A into the interview record. The agent's answers must be grounded in card content — do not invent requirements that aren't in the card.

**In guided and supervised modes:** Use AskUserQuestion normally — the author answers.

**Output:** Save `interview.md` in the current spec directory.

Update `drive.yaml`: `status: spec`

### 4. Stage: Spec

Read the spec skill instructions from `plugins/orb/skills/spec/SKILL.md`. Follow its instructions to generate a spec from the interview.

**Output:** Save `spec.yaml` in the current spec directory. Update the card's `specs` array per the spec skill's instructions.

Update `drive.yaml`: `status: review-spec`

**Supervised mode gate:** If autonomy is `supervised`, pause here:
```
AskUserQuestion: "Spec generated at <path>. Summary: <goal from spec>, <N> ACs, <N> constraints. Review and greenlight to continue, or NO-GO to re-enter at design."
Suggested answers: ["GO — proceed to review-spec", "NO-GO — re-enter at design"]
```
If NO-GO → jump to §8 (NO-GO Handling).

### 5. Stage: Review-Spec

Review-spec runs as a **forked Agent** via the Agent tool. The component skill declares `context: fork` in its frontmatter — drive honours that contract and launches the review in a fresh context, not inline in this session.

#### 5.1 Compute the cycle-specific output path

Read `review_cycles.review_spec` from drive.yaml. Let N = that value + 1 (the cycle ordinal for this fork — 1-indexed).

Capture or reuse the date token:
- If `review_cycle_dates.review_spec` is null, set it to today's ISO date (YYYY-MM-DD) and persist drive.yaml. This is cycle 1's date.
- Otherwise, reuse the stored value. The date is fixed at cycle 1 for the whole stage so long-running drives don't split cycle files across date boundaries.

Compute the output path:
- **Cycle 1 (N=1):** `<current_spec>/review-spec-<date>.md` (no suffix — preserves the inline-convention path shape)
- **Cycle 2 (N=2):** `<current_spec>/review-spec-<date>-v2.md`
- **Cycle 3 (N=3):** `<current_spec>/review-spec-<date>-v3.md`

#### 5.2 Idempotent resumption check

Before launching any fork, check whether a valid review already exists at the cycle-specific path (a prior session may have crashed after the fork wrote the file but before drive parsed it):

- If the file exists AND contains a line matching the canonical verdict regex (§5.4), parse that verdict and proceed to §5.5 verdict handling **without launching any Agent**.
- Otherwise, continue to §5.3.

#### 5.3 Launch the forked review

**Pre-flight: verify Agent tool availability.** Run `ToolSearch select:Agent` to load the Agent tool schema. If ToolSearch returns no result, do NOT fall back to inline review — escalate immediately:
- Update `drive.yaml` `status: escalated`
- Output: `Agent tool unavailable — cannot launch cold-fork review for review-spec`
- Stop — inline review violates the cold-fork separation contract (decisions 0005, 0006).

This pre-flight is load-bearing when drive runs in nested contexts (e.g. inside a rally sub-agent) where the deferred-tool surface may not include Agent.

Invoke the Agent tool with:
- `subagent_type: general-purpose`
- A brief containing **only**:
  - The absolute path to the spec under review (`<current_spec>/spec.yaml`)
  - The absolute path where the review must be written (the cycle-specific path from §5.1)
  - The instruction to read the spec cold, follow the `/orb:review-spec` skill, and write the review to the specified path using the canonical verdict line format

The brief must NOT include:
- Any conversation context from this drive session
- The iteration counter, cycle number, or the existence of prior review files (the re-review is functionally identical to the first — see ac-08)
- drive.yaml contents or any other state

Example brief shape:

```
Run /orb:review-spec on the spec at <absolute spec path>.
Write the review to exactly <absolute output path> (this path takes precedence
over the default path in the skill). Use the canonical verdict line format
`**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`.
```

The forked Agent's chat response may contain a summary. **Drive does not parse the chat response for the verdict** — the file on disk is the only authoritative source (see §5.4).

#### 5.4 Parse the verdict from the file

After the fork returns, read the file at the cycle-specific output path. Locate the first line matching the canonical regex:

```
^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$
```

- The match is **case-sensitive** on the verdict token. `**verdict:** approve` does not match. `Verdict: APPROVE` (no bold) does not match. `The verdict is APPROVE.` does not match. No fuzzy matching, no heuristics — canonical or no-verdict.
- If multiple matches exist, use the first.
- If zero matches exist, or the file is missing entirely, treat as **no verdict** and fall through to §5.4.1 retry.

##### 5.4.1 Retry on missing verdict (budget: 1)

If no verdict was parseable from the file, launch **one** retry fork with a fresh brief identical to the original (§5.3). The retry writes to the same cycle-specific path, overwriting any partial file from the failed attempt.

**Retry does not increment `review_cycles.review_spec`** — that counter only advances on a successfully-parsed REQUEST_CHANGES verdict. The fork-retry budget is independent of the REQUEST_CHANGES budget.

If the retry also produces no parseable verdict, drive escalates:
- Update `drive.yaml` `status: escalated`
- Output: `review could not be completed after 2 forked attempts at review-spec`
- Stop — do not re-enter at design, this is a harness-level failure not a review outcome.

#### 5.5 Verdict handling

Once a verdict is parsed:

- **APPROVE:** Update `drive.yaml` `status: implement`. Proceed to §6 Implement.

- **REQUEST_CHANGES:**
  - Increment `review_cycles.review_spec` in drive.yaml.
  - Check the synthetic-BLOCK budget (§5a).
  - If the budget allows another cycle: address the specific changes in the spec (edit spec.yaml per the review's findings), then return to §5.1 for the next cycle. The next fork's brief is functionally identical to the first — the new reviewer sees only the updated spec, with no pointer to prior review files and no iteration counter.

- **BLOCK:** Jump to §8 NO-GO Handling. The block reason (the review file's findings, summarised) becomes the NO-GO constraint.

#### 5a. REQUEST_CHANGES budget & synthetic BLOCK (applies to both review stages)

Each stage (review-spec, review-pr) has an **independent budget of 3 REQUEST_CHANGES cycles per top-level iteration**. The counters live at `review_cycles.review_spec` and `review_cycles.review_pr` in drive.yaml and reset to 0 when drive enters a new top-level iteration (new spec directory after a NO-GO re-entry).

After incrementing `review_cycles.<stage>` on a REQUEST_CHANGES verdict:

- If the new counter value is **< 3**: the stage has budget remaining. Address the findings and launch the next cycle (§5.1 → §5.5 for review-spec, symmetric path for review-pr).
- If the new counter value **== 3**: this was the 3rd real REQUEST_CHANGES on the stage in this iteration. The budget is exhausted. Do NOT launch a 4th fork. Instead, synthesise a BLOCK verdict:
  - Record in `drive.yaml` `history`:
    ```yaml
    - dir: <current_spec>
      result: NO-GO
      constraint_added: "review converged on REQUEST_CHANGES after 3 iterations; findings have not been addressable within budget"
    ```
  - The synthetic BLOCK consumes a top-level iteration the same way a real BLOCK does — jump to §8 NO-GO Handling with this constraint string.

The constraint string is fixed and byte-identical across the codebase (spec ac-10 and this section). Do not paraphrase.

**Resumption case:** If drive resumes with `review_cycles.<stage> == 3` and the stage has not already triggered a NO-GO (i.e. the session died between the counter increment and the synthetic-BLOCK write), synthesise the BLOCK on resumption — do not launch a 4th fork.

**Supervised mode gate:** If autonomy is `supervised` AND the verdict was APPROVE, pause here. **Severity dispatch (see §7a for the shared contract):**

- If the review file reports **no findings** or **LOW-only findings**: use the 2-option prompt:
  ```
  AskUserQuestion: "Spec review complete — verdict: APPROVE. <N> findings (<severities>). Review saved at <path>. Proceed to implementation?"
  Suggested answers: ["GO — proceed to implement", "NO-GO — re-enter at design"]
  ```
- If the review file reports **at least one finding at MEDIUM or HIGH severity**: use the **four-option verdict prompt** per §7a (`approve`, `request changes`, `block`, `read full review first`). Interpret the author's selection:
  - `approve` → proceed to implement.
  - `request changes` → treat as a post-APPROVE REQUEST_CHANGES; increment `review_cycles.review_spec` and return to §5.1 (budget-gated per §5a).
  - `block` → jump to §8 NO-GO Handling with the block reason "author blocked post-APPROVE at MEDIUM+ review".
  - `read full review first` → wait for the author's next turn, then re-present the same four-option prompt verbatim (§7a).

If the 2-option prompt returns NO-GO → jump to §8 (NO-GO Handling). (On REQUEST_CHANGES or BLOCK from the forked review itself, handling above applies unconditionally — the supervised gate only applies on APPROVE.)

### 6. Stage: Implement

Read the implement skill instructions from `plugins/orb/skills/implement/SKILL.md`. Follow its instructions — read the spec, present the checklist, write `progress.md`, implement the deliverables.

**Output:** Implementation code + `progress.md` with AC tracking.

Update `drive.yaml`: `status: review`

**Supervised mode gate:** If autonomy is `supervised`, pause here:
```
AskUserQuestion: "Implementation complete. Progress tracked in <path>. <N>/<total> ACs addressed. Review and greenlight to continue, or NO-GO to re-enter at design."
Suggested answers: ["GO — proceed to review-pr", "NO-GO — re-enter at design"]
```
If NO-GO → jump to §8 (NO-GO Handling).

### 7. Stage: Review-PR

Review-pr runs as a **forked Agent** via the Agent tool. The component skill declares `context: fork, agent: general-purpose` in its frontmatter — drive honours that contract and launches the review in a fresh context, not inline in this session.

The mechanics mirror §5 Review-Spec exactly. The differences are:
- The brief references the current branch diff (`git diff main...HEAD`) instead of the spec path
- The output path uses `review-pr` in place of `review-spec`
- The budget counter is `review_cycles.review_pr`; the date token is `review_cycle_dates.review_pr`

#### 7.1 Compute the cycle-specific output path

As §5.1, using `review_cycles.review_pr` and `review_cycle_dates.review_pr`:
- Cycle 1: `<current_spec>/review-pr-<date>.md`
- Cycle 2: `<current_spec>/review-pr-<date>-v2.md`
- Cycle 3: `<current_spec>/review-pr-<date>-v3.md`

#### 7.2 Idempotent resumption check

As §5.2 — if a valid review file already exists at the cycle-specific path, parse it and skip the fork.

#### 7.3 Launch the forked review

**Pre-flight: verify Agent tool availability.** As §5.3 — run `ToolSearch select:Agent` before invoking the Agent tool. If unavailable, escalate with `Agent tool unavailable — cannot launch cold-fork review for review-pr`. Do not fall back to inline review.

Invoke the Agent tool with:
- `subagent_type: general-purpose`
- A brief containing **only**:
  - The branch name or the `git diff main...HEAD` reference
  - The absolute path to the spec (for AC cross-reference)
  - The absolute path where the review must be written (the cycle-specific path from §7.1)
  - The instruction to read the diff cold, follow the `/orb:review-pr` skill, and write the review to the specified path using the canonical verdict line format

Example brief shape:

```
Run /orb:review-pr against the current branch. The implementation is on
<branch_name>; diff against main. Spec is at <absolute spec path>.
Write the review to exactly <absolute output path> (this path takes precedence
over the default path in the skill). Use the canonical verdict line format
`**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`.
```

The brief must NOT include any conversation context, iteration counter, or references to prior review files. Re-reviews are functionally identical to the first.

#### 7.4 Parse the verdict from the file

As §5.4 — strict canonical regex, case-sensitive, no fuzzy matching, retry once on no-verdict per §5.4.1 (with the review-pr escalation message: `review could not be completed after 2 forked attempts at review-pr`).

#### 7.5 Verdict handling

- **REQUEST_CHANGES:**
  - Increment `review_cycles.review_pr` in drive.yaml.
  - Check the §5a budget. If 3: synthetic BLOCK per §5a.
  - Otherwise: address the findings (edit the implementation), then return to §7.1 for the next cycle. The re-reviewer sees only the updated diff, with no pointer to prior review files.

- **BLOCK (real or synthetic):** Jump to §8 NO-GO Handling.

- **APPROVE:**
  - **In full mode:** Proceed directly to §10 (Completion).
  - **In guided mode:** This is the **only gate in guided mode**. Severity dispatch (see §7a for the shared contract):
    - **No findings or LOW-only findings** — use the existing three-option rich summary:
      ```
      AskUserQuestion: "Drive summary for <card name>:

      Spec: <spec path> — <goal summary>
      Spec review: <verdict>, <N> findings
      Implementation: <N>/<total> ACs addressed
      PR review: APPROVE — <honest assessment one-liner>

      Review saved at <path>. Proceed to PR creation?"
      Suggested answers: ["GO — create PR", "NO-GO — re-enter at design", "Let me read the reviews first"]
      ```
      If "Let me read the reviews first" → wait for the author to respond after reading, then re-present the gate. If NO-GO → jump to §8 (NO-GO Handling).
    - **At least one MEDIUM or HIGH finding** — use the **four-option verdict prompt** per §7a (`approve`, `request changes`, `block`, `read full review first`), prefaced by the same drive-summary block. Interpret the selection:
      - `approve` → proceed to §10 (Completion).
      - `request changes` → increment `review_cycles.review_pr` and return to §7.1 (budget-gated per §5a).
      - `block` → jump to §8 NO-GO Handling with the block reason "author blocked post-APPROVE at MEDIUM+ PR review".
      - `read full review first` → wait for the author's next turn, then re-present the same four-option prompt verbatim (§7a).
  - **In supervised mode:** Same gate as guided.

### 7a. Four-option verdict prompt (shared contract)

When a review-spec supervised-APPROVE gate (§5a) or a review-pr guided/supervised APPROVE gate (§7.5) dispatches to the four-option prompt, the following rules apply uniformly.

**When the four-option prompt fires.** Only on APPROVE verdicts where the review file reports at least one finding at MEDIUM or HIGH severity. REQUEST_CHANGES and BLOCK verdicts are handled by the existing branch-to-next-cycle and NO-GO paths respectively — the four-option prompt never replaces those. LOW-only or zero-finding APPROVE gates retain the existing shorter prompt (two-option for review-spec supervised, three-option for review-pr guided/supervised).

**Severity-read contract.** Drive reads severity labels (LOW / MEDIUM / HIGH) directly from the review file's findings table. **Drive does not re-classify findings, and does not invent severities.** If the review file uses only LOW or is finding-free, drive uses the shorter prompt. If the file contains any `[MEDIUM]` or `[HIGH]` finding, drive routes through the four-option prompt.

**The four options (exact labels).** Use these labels verbatim as AskUserQuestion suggested answers — lower-case, single spaces, no hyphens, no punctuation:

```
approve
request changes
block
read full review first
```

**Interpretation (same at both gates).**

- `approve` — terminal verdict. Drive advances to the next stage (implement after a spec gate; §10 Completion after a PR gate).
- `request changes` — treated as a post-APPROVE REQUEST_CHANGES: drive increments `review_cycles.<stage>`, checks the §5a budget, and re-enters the review cycle (§5.1 for spec, §7.1 for PR). The author is responsible for flagging the specific changes they want in the next reviewer brief *via updating the spec or implementation first*; drive does not pass freeform text through to the forked reviewer.
- `block` — drive jumps to §8 NO-GO Handling. The constraint is `author blocked post-APPROVE at MEDIUM+ <review-spec | PR> review`.
- `read full review first` — **deferral, not a verdict.** Drive waits for the author's next turn (they may ask follow-up questions, request clarification, or signal ready). On their next turn, drive re-presents the **same four-option prompt verbatim** — same preamble, same four options, no iteration counter added. The deferral is open-ended; drive does not time out. This matches how §7.5's existing three-option prompt handles "Let me read the reviews first."

**Why four options and not three.** The scenario "Review verdicts route via structured choice" (card 0005) names exactly four options. Reading the full review is itself a valid deferred state that the terminal verbs do not express — collapsing it into "NO-GO" conflates deferral with rejection. The four-option shape preserves the signal.

**Applicability boundary.** The four-option prompt does not fire in full autonomy (full mode has no APPROVE gates at either review stage — review APPROVE flows directly to the next stage). It only fires in supervised mode for review-spec, and in guided/supervised mode for review-pr.

### 8. NO-GO Handling

A NO-GO means the current iteration failed a review (spec or PR) or was rejected at a supervised gate.

1. **Record the failure** in `drive.yaml`:
   ```yaml
   history:
     - dir: <current spec directory>
       result: NO-GO
       constraint_added: "<one-line description of what failed and why>"
   ```

2. **Check budget:** If `iteration == budget` (i.e., this was the last allowed iteration), jump to §9 (Escalation). Do not increment.

3. **Increment iteration** and reset the per-stage REQUEST_CHANGES counters for the new iteration (each iteration has its own fresh 3-cycle budget per stage):
   ```yaml
   iteration: <current + 1>
   current_spec: <new spec directory path>
   status: design
   review_cycles:
     review_spec: 0
     review_pr: 0
   review_cycle_dates:
     review_spec: null
     review_pr: null
   ```

4. **Create the new spec directory** (e.g. `orbit/specs/YYYY-MM-DD-drive-v2/`).

5. **Re-enter at design** (§3) with the failure constraint carried forward. The constraint from the NO-GO becomes a hard constraint in the new iteration's design session.

### 9. Escalation

Escalation is triggered by **budget exhaustion** (3 NO-GO iterations) OR by a **semantic trigger** from the Disposition section (recurring failure mode, contradicted hypothesis, diminishing signal). An honest agent may escalate before the budget is spent.

1. **Update drive.yaml:**
   ```yaml
   status: escalated
   ```

2. **Output an escalation summary:**
   ```
   DRIVE ESCALATED — <reason: budget exhausted | recurring failure | contradicted hypothesis | diminishing signal>

   Card: <card path>
   Goal: <card goal>

   Iteration history:
     1. <dir> — NO-GO: <constraint_added>
     2. <dir> — NO-GO: <constraint_added>
     [3. <dir> — NO-GO: <constraint_added>]

   Accumulated constraints:
     - <all constraints from all iterations>

   What would have to be true:
     <For a future attempt to succeed, what assumptions need revisiting?
      What constraints are structural vs configurational?
      What corner of the solution space was not explored?>

   Recommendation:
     <What the card needs before another drive attempt.>
   ```

3. **Clean up the recurring heartbeat (full autonomy only).** If `autonomy == full`, attempt `CronDelete drive-checkin-<spec-slug>` to stop the recurring heartbeat. If CronDelete fails (e.g. task already expired, harness error), log one line `heartbeat cleanup skipped: <reason>` and continue. **Failure is non-fatal** — it must not abort escalation. This step executes **before** scheduling the one-shot escalation ping in step 4, so the recurring heartbeat cannot fire between the summary output and the ping.

4. **Fire a one-shot escalation ping (full autonomy only).** If `autonomy == full`, schedule a one-shot `CronCreate` to fire approximately 30 seconds after the escalation summary renders. This gives the summary time to appear first so the ping's "see prior output" reference is accurate.

   - **Delay:** ~30 seconds (one-shot, not recurring).
   - **Task ID:** `drive-escalation-<spec-slug>`.
   - **Prompt body:** emit the single line below verbatim (including the bold markdown header — the scenario asks for a "prominent" message and bold is the available emphasis mechanism):

     ```
     **DRIVE ESCALATED** on <card-slug> after <iterations> iterations. See prior output for findings and recommendation.
     ```

   Drive does not nag — the ping is one-shot, not recurring. If `CronCreate` for the ping fails, log one line `escalation ping skipped: <reason>` and continue. The escalation summary emitted in step 2 is the authoritative channel; the ping is notification amplification, not the signal itself.

5. **Stop.** The card needs human rethinking. Escalation is not giving up — it is the mechanism by which difficult work gets human judgment at the right moment.

### 10. Completion

On successful review (APPROVE verdict, gates passed):

1. **Stage and commit the implementation** (commit 1):
   - All code changes, spec files, progress.md, review file
   - Commit message: `feat: <card feature name> — drive iteration <N>`

2. **Propose card updates** (commit 2):
   - Update the card's `maturity` field if appropriate (e.g. `planned` → `active`)
   - Refine the card's `goal` if the implementation revealed more precise success criteria
   - Commit message: `docs: update <card> — maturity and goal after drive`

3. **Create the PR:**
   - Title: `drive: <card feature name>`
   - Body references the spec path, drive.yaml, and iteration count
   - Both commits visible in the PR diff

4. **Update drive.yaml:**
   ```yaml
   status: complete
   ```

5. **Clean up the recurring heartbeat (full autonomy only).** If `autonomy == full`, attempt `CronDelete drive-checkin-<spec-slug>` to stop the recurring heartbeat. **Failure is non-fatal** — if CronDelete fails (task already expired, harness error), log one line `heartbeat cleanup skipped: <reason>` and continue. Completion must not abort on a cleanup failure; the PR is already created and drive.yaml already reads `complete`.

### 11. Resumption

When `/orb:drive` is invoked and `drive.yaml` already exists in the expected location:

1. **Check for the `review_cycles` field.** This is the first check on any resumption — it enforces the forked-reviews migration boundary.
   - **Absent:** drive.yaml was initialised before forked reviews shipped. Refuse to resume per §2. Output the refusal message and exit without advancing state or launching any Agent.
   - **Present:** proceed with resumption. Read `review_cycles.review_spec`, `review_cycles.review_pr`, and the corresponding `review_cycle_dates` entries into working state.

2. **Read `drive.yaml`** for the rest of the state (card, autonomy, iteration, budget, current_spec, status, history).

3. **Determine which stage to resume from** using file-presence detection:

   | drive.yaml status | Files present | Resume at |
   |-------------------|---------------|-----------|
   | `design` | no interview.md | Design (§3) |
   | `spec` | interview.md, no spec.yaml | Spec (§4) |
   | `review-spec` | spec.yaml | Review-Spec (§5) — see review_cycles handling below |
   | `implement` | review-spec-*.md (with APPROVE verdict), no progress.md | Implement (§6) |
   | `review` | progress.md | Review-PR (§7) — see review_cycles handling below |
   | `complete` | review-pr-*.md | Already done — report status |
   | `escalated` | — | Already escalated — report status |

4. **Review-stage resumption (review-spec or review-pr):**
   - If `review_cycles.<stage> == 3` and the budget-exhausted synthetic BLOCK was not yet written to history (session died between the counter increment and the §8 NO-GO write), synthesise the BLOCK on resumption per §5a — do not launch a 4th fork.
   - Otherwise, enter the stage at §5.1 / §7.1 and compute the cycle-specific path from the counter. The §5.2 / §7.2 idempotent-resumption check will parse any valid review file already on disk before launching a fork, so a session that died after the fork wrote a valid file but before drive parsed it advances cleanly on resume with no extra fork.

5. **File presence overrides drive.yaml status** when they disagree on completion. If drive.yaml says `implement` but `progress.md` already exists with completed ACs, advance to `review`. The files are ground truth for stage completion; drive.yaml may be stale from an interrupted session. This rule does NOT override the `review_cycles` migration check — that check is unconditional.

6. **Re-run §2a's heartbeat reconciliation (full autonomy only).** If `autonomy == full`, re-execute the §2a idempotent CronList-first flow: if `drive-checkin-<spec-slug>` exists (restored by the harness), leave it untouched; if absent (task expired or lost during the interrupted session), re-create it via CronCreate. This is the mechanism that satisfies the "Check-ins survive resume" scenario — drive never delete-then-recreates, so a surviving task is preserved, and an absent task is repaired.

7. **Announce the resumption:**
   ```
   Resuming drive for <card path>
   Autonomy: <level>
   Iteration: <N> of <budget>
   Resuming at: <stage>
   Review cycles: review-spec=<N>/3, review-pr=<N>/3
   Heartbeat: <active | created | n/a (non-full autonomy) | unavailable>
   ```

   The `Heartbeat` line reports the outcome of the step-6 reconciliation: `active` if the task was already present, `created` if it was re-created, `n/a` if autonomy is guided or supervised, `unavailable` if CronCreate failed (with the non-fatal log from §2a).

## Disposition

The drive's job is to **find the way through, not the evidence that closes the card.** When an iteration falls short, the first question is *what would have to be true for this to work*, not *what does this rule out*. Push past the first plateau. Try the approaches that look uncomfortable. Treat negative results as constraints on the next iteration, not as conclusions.

This disposition applies at every stage:

- **Design:** When carrying forward a NO-GO constraint, the agent's task is to find the configuration that satisfies the new constraint — not to confirm that the goal is unreachable.
- **Review-Spec:** An honest spec review catches assumptions before they become implementation debt. REQUEST_CHANGES strengthens the spec; BLOCK is evidence for re-design.
- **Implement:** When implementation hits friction, work through it. The spec was designed with the constraint in mind; honour the design.
- **Review-PR:** An honest PR review serves the disposition. A REQUEST_CHANGES verdict is an opportunity to strengthen the iteration, not a signal to give up. A BLOCK verdict is evidence for the next iteration's design.

### Bounded by honest escalation

Disposition and escalation are the same stance, not opposing ones. Commitment to the goal is bounded by honest reporting. Escalate — don't push through, and don't quietly close — when any of these are true:

- **Recurring failure mode.** The same problem has appeared across 2+ iterations despite varied approaches to address it. The constraint may be structural, not configurational.
- **Contradicted hypothesis.** The accumulated evidence points to the card's *underlying goal* being unreachable, not just the current approach falling short. The call to pivot a thesis belongs to the author.
- **Diminishing signal.** Each iteration is producing less new information than the last. The drive is grinding, not learning.

These semantic triggers supplement the mechanical budget (3 iterations). An agent with the right disposition may escalate at iteration 2 when the evidence warrants it, or push hard through all 3 when each iteration genuinely narrows the search space.

### What this means in practice

A mechanical agent runs 3 iterations by rote and escalates with "tried 3 times, didn't work." A disposed agent:

1. Treats iteration 1's failure as a constraint that sharpens iteration 2's design
2. Asks "what corner of the solution space haven't I tried?" before concluding the space is empty
3. Reports honestly when the evidence says the goal needs rethinking — and explains *why*, with the accumulated constraint history as proof
4. Includes in every escalation summary not just what failed, but what would have to be true for a future attempt to succeed

## Critical Rules

- **Never skip a stage.** Design → Spec → Review-Spec → Implement → Review-PR, always in order.
- **Never silently downgrade autonomy.** If full mode is requested but the card is thin, refuse explicitly.
- **drive.yaml is the single source of orchestration state.** Do not track drive state anywhere else.
- **Existing file-presence model is authoritative for stage completion.** drive.yaml tracks the orchestration layer; individual files (interview.md, spec.yaml, review-spec-*.md, progress.md, review-pr-*.md) prove stage completion.
- **Constraints accumulate across iterations.** Every NO-GO adds a constraint. Iteration 3 carries constraints from iterations 1 and 2.
- **Reviews run as forked Agents.** `review-spec` and `review-pr` declare `context: fork` in their frontmatter; drive honours that contract by launching each review via the Agent tool (`subagent_type: general-purpose`) in a fresh context. Drive never reads their SKILL.md files inline at runtime.
- **Verdicts are read from disk only.** The review file's canonical verdict line (`**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`, matched by strict regex) is the single authoritative source. Drive never parses the forked Agent's chat response for the verdict — if chat and file disagree, the file wins by construction.
- **Re-reviews are fully cold.** When a REQUEST_CHANGES cycle re-forks the reviewer, the new fork's brief is functionally identical to the first cycle's brief — no path to prior review files, no iteration counter, no summary of prior findings. Confirmation-bias resistance is the whole point; context bleed defeats it.
- **REQUEST_CHANGES is bounded per stage.** Each review stage has an independent 3-cycle budget (see §5a). The 4th would-be cycle is converted to a synthetic BLOCK with a fixed constraint string and consumes a top-level iteration normally.
- **No migration scaffolding.** Drives initialised before forked reviews shipped refuse to resume under the new code. Finish or park in-flight drives before upgrade. No `review_mode` field, no dual code paths, no flag day.
- **Reviews are the quality gates.** In guided mode, the spec review and PR review replace explicit go/no-go prompts. The only interactive gate is the final verdict summary before PR creation.
- **Live-visibility heartbeat is self-terminating and full-autonomy-only.** The recurring `drive-checkin-<spec-slug>` task fires only when `autonomy == full`. The cron prompt body carries a read-only contract ("do not modify drive.yaml, do not launch any Agent") with one exception: when drive.yaml shows a terminal state (`complete` or `escalated`), the heartbeat calls `CronDelete` on itself and stops. This is a defence-in-depth backstop — §10 and §9 remain the primary cleanup path, but if their `CronDelete` fails (non-fatal), the next heartbeat tick self-terminates instead of zombieing indefinitely.
- **Cron tasks are reconciled idempotently.** Drive uses CronList-then-CronCreate-iff-absent (§2a) on every initialise and every resume. Drive never delete-then-recreates a heartbeat task — doing so would defeat Claude Code's built-in task restoration on `--resume` / `--continue`.
- **MEDIUM+ review verdicts route through a four-option AskUserQuestion prompt.** At APPROVE gates where the review file reports any MEDIUM or HIGH finding, the prompt surfaces `approve / request changes / block / read full review first` (§7a). LOW-only APPROVE gates retain the existing shorter prompt. Drive reads severity from the review file directly and never re-classifies. `read full review first` is a deferral, not a verdict — drive re-presents the same four-option prompt verbatim on the author's next turn.
