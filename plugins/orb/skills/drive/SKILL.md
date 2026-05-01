---
name: drive
description: Drive a card or bead through the orbit pipeline — promote → review-spec → implement → review-pr — at a declared autonomy level
---

# /orb:drive

Take a card and an autonomy level. Drive the pipeline (promote →
review-spec → implement → review-pr) as a single session. State lives
in beads; resumption reads bead metadata.

## Migration Note

The previous `/orb:drive` orchestrated a Design → Spec → Review-Spec →
Implement → Review-PR pipeline with state in `drive.yaml` and per-
iteration spec dirs (`drive-v2`, `-v3`). All of that is now subsumed by
beads:

- Design + Spec → `promote.sh card→bead` (one step, no interview/spec
  artefacts produced by drive)
- Drive state machine → bead `status` + bead metadata fields
  (`drive_stage`, `drive_iteration`, `drive_review_*_cycle`,
  `drive_review_*_date`, `drive_card`, `drive_autonomy`)
- Per-iteration spec dirs → new bead per iteration linked by
  `discovered-from` edge
- File-presence stage detection → `drive_stage` metadata read on resume
- `drive.yaml` resumption refusal → removed; drives initialised under
  the prior version finish there or restart from the card

Cold-fork review architecture (decision 0011 D2) is preserved — the
fork reads the bead directly via `bd show <bead-id> --json` and
`plugins/orb/scripts/parse-acceptance.sh acs <bead-id>`, with no
intermediate spec.yaml or rendered snapshot artefact.

## Usage

```
/orb:drive <card_path> [full|guided|supervised]   # fresh drive from card
/orb:drive <bead-id>                              # resume an in-flight drive
/orb:drive                                        # resume the unique in-progress drive bead, if any
```

### Autonomy Levels

| Level | Behaviour |
|-------|-----------|
| **full** | Agent self-answers in promote / review gates. All stages run without human interaction. Pauses only for PR merge. Requires ≥3 card scenarios. |
| **guided** | Promote runs autonomously. All stages run without intermediate pauses — the reviews ARE the quality gates. Author approves the final review-pr verdict before PR creation. Default. |
| **supervised** | Author greenlights after each stage before proceeding. |

## Input contract

The skill operates on exactly one drive bead per session. Resolution
proceeds in three branches:

1. **Card path provided** (`/orb:drive <card_path> [autonomy]`). Run
   the pre-flight thin-card refusal (below), then promote (§Promote).

2. **Bead-id provided** (`/orb:drive <bead-id>`). Validate that the
   bead has `drive_stage` metadata; if not, halt and instruct the
   agent to re-invoke with a card path. Otherwise, resume from the
   stage named in `drive_stage`.

3. **No argument** — query for in-progress drive beads:

   ```bash
   bd list --status in_progress --json \
     | python3 -c "import sys,json; print('\n'.join(b['id'] for b in json.load(sys.stdin) if b.get('metadata',{}).get('drive_stage')))"
   ```

   - **Single match** → resume it.
   - **Zero matches** → halt with usage.
   - **Multiple matches** → halt and instruct the agent to pass the
     bead id explicitly, listing the candidates.

## Pre-flight (card path branch only)

**Thin cards block full autonomy.** Read the card and count
`scenarios`. If fewer than 3:

- If autonomy is `full`: **REFUSE.** Output:
  ```
  BLOCKED: Card has N scenario(s) — full autonomy requires ≥3.
  Missing coverage areas to consider:
  - <suggest what scenarios are absent based on the card's goal>
  Add scenarios with /orb:card and retry.
  ```
  Do not proceed. Do not silently downgrade to guided.

- If autonomy is `guided` or `supervised`: proceed (the human is in
  the loop to compensate for thin requirements).

## Promote stage

Promote replaces the old Design + Spec stages.

```bash
BEAD_ID=$(plugins/orb/scripts/promote.sh "<card_path>")
```

Then write orchestration metadata in a single batch:

```bash
bd update "$BEAD_ID" \
  --set-metadata "drive_card=<absolute card_path>" \
  --set-metadata "drive_autonomy=<full|guided|supervised>" \
  --set-metadata "drive_iteration=1" \
  --set-metadata "drive_stage=review-spec" \
  --set-metadata "drive_review_spec_cycle=0" \
  --set-metadata "drive_review_pr_cycle=0"
```

Claim the bead atomically:

```bash
bd update "$BEAD_ID" --claim
```

No `interview.md`, no `spec.yaml`, no `drive.yaml`, no per-iteration
spec directory. The card → bead promotion IS the spec.

After promote, schedule the heartbeat (full autonomy only — see below)
and proceed to Stage 1.

## Heartbeat (full autonomy only)

Skip this section entirely when `drive_autonomy != full`.

**Reconciliation, not re-creation.** Drive uses CronList-first
idempotent reconciliation. Drive never delete-then-recreates the
heartbeat — that would defeat Claude Code's built-in `--resume` task
restoration.

1. `CronList` — enumerate active cron tasks in the session.
2. If a task with ID `drive-checkin-<bead-id>` already exists, no-op
   (it was restored by the harness or created earlier).
3. Otherwise, `CronCreate` a recurring task with:
   - **ID:** `drive-checkin-<bead-id>`.
   - **Interval:** 5 minutes, recurring. **Hardcoded.**
   - **Prompt body:** the exact text below.

**Heartbeat prompt body (verbatim):**

```
This is a drive heartbeat. Run `bd show <bead-id> --json` and read its
`status` and `metadata.drive_stage`.

If status is `closed` (drive_stage `complete` or `escalated`), call
CronDelete with ID `drive-checkin-<bead-id>` and emit:

  drive: heartbeat stopped (stage=<drive_stage>)

Then stop. Do not emit a heartbeat line.

Otherwise: run `plugins/orb/scripts/parse-acceptance.sh next-ac
<bead-id>` to find the current AC (if it returns nothing or drive_stage
is not `implement`, use `-`). Compute elapsed as mm:ss since the bead's
`started_at` field. Emit exactly one line in the format:

  drive: bead=<bead-id> stage=<drive_stage> ac=<id|-> elapsed=<mm:ss>

Do not modify the bead. Do not launch any Agent. Emit the single
heartbeat line and stop.
```

**Self-termination.** When the bead transitions to `closed`
(complete or escalated), the heartbeat calls `CronDelete` on itself as
a defence-in-depth backstop — primary cleanup remains in §Completion
and §Escalation.

**Non-fatal CronCreate.** If CronCreate fails (harness doesn't
support cron, rate limit, transient error), drive logs one line
`heartbeat unavailable: <reason>` and continues the pipeline. The
heartbeat is an observability affordance; its absence must not block
any stage.

## Stage 1: Review-Spec

Review-spec runs as a **forked Agent** via the Agent tool — fresh
context, no shared conversation history.

### 1.1 Compute the cycle-specific verdict path

Read `drive_review_spec_cycle` from bead metadata. Let N = that value
+ 1 (the cycle ordinal for this fork — 1-indexed).

Capture or reuse the date token:

- If `drive_review_spec_date` is unset, set it to today's ISO date and
  write it back to bead metadata. This is cycle 1's date.
- Otherwise, reuse the stored value. The date is fixed at cycle 1 for
  the whole stage so long-running drives don't split cycle files
  across date boundaries.

Compute the output path:

- **Cycle 1:** `orbit/reviews/<bead-id>/review-spec-<date>.md`
- **Cycle 2:** `orbit/reviews/<bead-id>/review-spec-<date>-v2.md`
- **Cycle 3:** `orbit/reviews/<bead-id>/review-spec-<date>-v3.md`

### 1.2 Idempotent resumption check

Before launching any fork, check whether a valid review already exists
at the cycle-specific path. If the file exists AND contains a line
matching the canonical verdict regex (§1.4), parse that verdict and
proceed to §1.5 verdict handling **without launching any Agent**.
Otherwise, continue to §1.3.

### 1.3 Launch the forked review

**Pre-flight: verify Agent tool availability.** Run `ToolSearch
select:Agent` to load the Agent schema. If ToolSearch returns no
result, do NOT fall back to inline review — escalate immediately:

- Set `drive_stage=escalated` on the bead.
- Output: `Agent tool unavailable — cannot launch cold-fork review for review-spec`
- Stop. Inline review violates the cold-fork separation contract.

Invoke the Agent tool with:

- `subagent_type: general-purpose`
- A brief containing **only**:
  - The bead-id whose acceptance the reviewer must read
  - The absolute path where the review must be written (§1.1)
  - The instruction to read the bead via `bd show <bead-id> --json`,
    parse ACs via `plugins/orb/scripts/parse-acceptance.sh acs <bead-id>`,
    follow the `/orb:review-spec` skill, and write the verdict to the
    specified path using the canonical verdict line format

Example brief:

```
Run /orb:review-spec on bead <bead-id>. Read the bead via `bd show <bead-id> --json`
and parse ACs via `plugins/orb/scripts/parse-acceptance.sh acs <bead-id>` —
the bead acceptance field is the authoritative spec for this review. Write the
review to exactly <absolute output path> (this path takes precedence
over the default path in the skill). Use the canonical verdict line
format `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`.
```

The brief must NOT include any conversation context, iteration counter,
cycle number, or pointers to prior review files.

### 1.4 Parse the verdict from the file

After the fork returns, read the file at the cycle-specific output
path. Locate the first line matching:

```
^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$
```

Match is case-sensitive on the verdict token. No fuzzy matching. If
zero matches or the file is missing, fall through to the retry.

**Retry on missing verdict (budget: 1).** Launch one retry fork with a
fresh brief identical to the original. The retry overwrites the same
path. Retry does NOT increment `drive_review_spec_cycle`. If the retry
also produces no parseable verdict, drive escalates with
`drive_stage=escalated` and the message `review could not be completed
after 2 forked attempts at review-spec`.

### 1.5 Verdict handling

- **APPROVE:** Set `drive_stage=implement` on the bead. Proceed to
  Stage 2.

- **REQUEST_CHANGES:**
  - Increment `drive_review_spec_cycle` on the bead.
  - Check the budget (§1.6).
  - If the budget allows another cycle: address the findings (edit the
    bead's acceptance field via `bd update --acceptance` or the bead's
    description via `bd update --description`), then return to §1.1
    to recompute the cycle-specific output path and re-fork.

- **BLOCK:** Jump to §NO-GO Handling. The block reason becomes the
  NO-GO constraint.

### 1.6 REQUEST_CHANGES budget & synthetic BLOCK

Each stage (review-spec, review-pr) has an **independent budget of 3
REQUEST_CHANGES cycles per top-level iteration**. The counters live in
bead metadata (`drive_review_spec_cycle`, `drive_review_pr_cycle`) and
reset to 0 when a new iteration's bead is created (§NO-GO).

After incrementing the counter on a REQUEST_CHANGES verdict:

- If the new value is **< 3**: the stage has budget remaining. Address
  the findings and launch the next cycle.
- If the new value **== 3**: this was the 3rd real REQUEST_CHANGES on
  the stage in this iteration. The budget is exhausted. Do NOT launch
  a 4th fork. Synthesise a BLOCK with the canonical constraint string:

  > `review converged on REQUEST_CHANGES after 3 iterations; findings have not been addressable within budget`

  This string is fixed and **byte-identical** with the spec ac-05
  verification target. Do not paraphrase. The synthetic BLOCK consumes
  a top-level iteration the same way a real BLOCK does — jump to §NO-GO.

**Resumption case:** If drive resumes with `drive_review_<stage>_cycle ==
3` and the synthetic BLOCK was not yet written (session died between
the counter increment and the NO-GO write), synthesise the BLOCK on
resume — do not launch a 4th fork.

### 1.7 Supervised mode gate (review-spec)

If autonomy is `supervised` AND the verdict was APPROVE, pause here.
**Severity dispatch (see §Four-option verdict prompt):**

- **No findings or LOW-only findings:** use the 2-option prompt:
  ```
  AskUserQuestion: "Spec review complete — verdict: APPROVE. <N> findings (<severities>). Review saved at <path>. Proceed to implementation?"
  Suggested answers: ["GO — proceed to implement", "NO-GO — re-enter at promote"]
  ```
- **At least one MEDIUM or HIGH finding:** use the four-option verdict
  prompt (§Four-option verdict prompt) with `approve / request changes
  / block / read full review first`.

If NO-GO or `block` → §NO-GO. If `request changes` → increment
`drive_review_spec_cycle` and return to §1.1 (budget-gated).

## Stage 2: Implement

Drive sets `drive_stage=implement` on the bead and delegates entirely
to the new beads-native `/orb:implement`:

```bash
bd update <bead-id> --set-metadata "drive_stage=implement"
# invoke /orb:implement with the bead id
```

Drive does NOT inline AC tracking, detour escalation, or progress
emission — those are owned by `/orb:implement`. When implement returns
(the bead's acceptance field has no unchecked ACs — verifiable via
`parse-acceptance.sh has-unchecked <bead-id>` exiting 1), drive sets
`drive_stage=review-pr` and proceeds to Stage 3.

**Supervised mode gate (implement):** If autonomy is `supervised`,
pause after implement returns:

```
AskUserQuestion: "Implementation complete. <N>/<total> ACs addressed. Review and greenlight to continue, or NO-GO to re-enter at promote."
Suggested answers: ["GO — proceed to review-pr", "NO-GO — re-enter at promote"]
```

If NO-GO → §NO-GO Handling.

## Stage 3: Review-PR

Mirrors Stage 1 mechanics with the diff brief. The forked reviewer
reads the post-implement bead state directly via `bd show <bead-id>
--json` and `parse-acceptance.sh acs <bead-id>` — the acceptance field
may have been edited during implement, and the live `bd` query gives
the reviewer the up-to-date state with no intermediate artefact.

### 3.1 Compute the cycle-specific verdict path

Using `drive_review_pr_cycle` and `drive_review_pr_date`:

- Cycle 1: `orbit/reviews/<bead-id>/review-pr-<date>.md`
- Cycle 2: `orbit/reviews/<bead-id>/review-pr-<date>-v2.md`
- Cycle 3: `orbit/reviews/<bead-id>/review-pr-<date>-v3.md`

### 3.2 Idempotent resumption check, fork launch, verdict parse

As §1.2 / §1.3 / §1.4, with these differences:

- The Agent brief includes the diff reference (`git diff main...HEAD`
  on the current branch) PLUS the bead-id for AC cross-reference (the
  reviewer reads the live acceptance field via `bd show` and
  `parse-acceptance.sh`).
- Output path uses `review-pr` in place of `review-spec`.
- Counter / date metadata uses `drive_review_pr_*`.
- Retry escalation message: `review could not be completed after 2
  forked attempts at review-pr`.

Example brief:

```
Run /orb:review-pr against the current branch. Implementation diff is
`git diff main...HEAD` on <branch_name>. Bead acceptance field is at
bead-id <bead-id>; read via `bd show <bead-id> --json` and
`plugins/orb/scripts/parse-acceptance.sh acs <bead-id>`. Write the
review to exactly <absolute output path> (this path takes precedence
over the default path in the skill). Use the canonical verdict line
format `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`.
```

### 3.3 Verdict handling

- **REQUEST_CHANGES:** Increment `drive_review_pr_cycle`. Check
  budget (§1.6). If budget remains, address findings (edit the
  implementation), return to §3.1 for the next cycle. If budget
  exhausted, synthesise BLOCK.

- **BLOCK (real or synthetic):** Jump to §NO-GO Handling.

- **APPROVE:**
  - **In full mode:** Proceed directly to §Completion.
  - **In guided mode:** This is the **only gate in guided mode**.
    Severity dispatch:
    - **No findings or LOW-only findings:** three-option rich summary:
      ```
      AskUserQuestion: "Drive summary for <card name>:

      Bead: <bead-id> — <title>
      Spec review: <verdict>, <N> findings
      Implementation: <N>/<total> ACs addressed
      PR review: APPROVE — <one-liner>

      Review saved at <path>. Proceed to PR creation?"
      Suggested answers: ["GO — create PR", "NO-GO — re-enter at promote", "Let me read the reviews first"]
      ```
      `Let me read the reviews first` defers — wait for the author's
      next turn, then re-present the gate.
    - **At least one MEDIUM or HIGH finding:** four-option verdict
      prompt (§Four-option verdict prompt), prefaced by the same
      drive-summary block.
  - **In supervised mode:** Same gate as guided.

## Four-option verdict prompt

When a review-spec supervised-APPROVE gate or a review-pr
guided/supervised APPROVE gate dispatches to the four-option prompt,
the following rules apply uniformly.

**When the four-option prompt fires.** Only on APPROVE verdicts where
the review file reports at least one finding at MEDIUM or HIGH
severity. REQUEST_CHANGES and BLOCK verdicts route via the existing
branch-to-next-cycle and NO-GO paths — the four-option prompt never
replaces those.

**Severity-read contract.** Drive reads severity labels (LOW / MEDIUM
/ HIGH) directly from the review file's findings table. Drive does
NOT re-classify findings, and does NOT invent severities.

**The four options (exact labels).** Use these labels verbatim as
AskUserQuestion suggested answers — lower-case, single spaces, no
hyphens, no punctuation:

```
approve
request changes
block
read full review first
```

**Interpretation.**

- `approve` — terminal verdict. Drive advances to the next stage
  (implement after a spec gate; §Completion after a PR gate).
- `request changes` — treated as a post-APPROVE REQUEST_CHANGES:
  drive increments `drive_review_<stage>_cycle`, checks the §1.6
  budget, and re-enters the review cycle.
- `block` — drive jumps to §NO-GO Handling. The constraint is
  `author blocked post-APPROVE at MEDIUM+ <review-spec | PR> review`.
- `read full review first` — **deferral, not a verdict.** Drive waits
  for the author's next turn; on their next turn drive re-presents
  the **same four-option prompt verbatim**.

## Completion

On APPROVE at review-pr (interactive gates per autonomy mode passed):

1. **Stage and commit the implementation** (commit 1):
   - All code changes and the review files
   - Commit message: `feat: <bead title>`

2. **Propose card updates** (commit 2):
   - Update the card's `maturity` if appropriate
   - Refine the card's `goal` if implementation revealed more precise
     success criteria
   - Commit message: `docs: update <card> — maturity and goal after drive`

3. **Create the PR:**
   - Title: `drive: <bead title>`
   - Body references the bead-id and review files

4. **Set drive_stage=complete and close the bead:**
   ```bash
   bd update <bead-id> --set-metadata "drive_stage=complete"
   bd close <bead-id> --reason "drive completed: <one-line summary>"
   ```

5. **Heartbeat cleanup (full autonomy only).** Attempt `CronDelete
   drive-checkin-<bead-id>`. **Failure is non-fatal** — log
   `heartbeat cleanup skipped: <reason>` and continue. The bead is
   already closed; the next heartbeat tick (if any) self-terminates
   on `bead.status == closed`.

## NO-GO Handling

A NO-GO means the current iteration failed a review (real or
synthetic BLOCK) or was rejected at a supervised gate.

1. **Close the current bead:**
   ```bash
   bd close <bead-id> --reason "NO-GO: <one-line constraint>"
   ```

2. **Persist the constraint:**
   ```bash
   bd remember "drive-<card-slug>-iter<N>: <constraint>"
   ```

   The key format is stable so iteration ≥2 can list all prior
   constraints with `bd memories drive-<card-slug>`.

3. **Check budget:** Read `drive_iteration` from the closed bead's
   metadata. If `drive_iteration == 3`, jump to §Escalation.

4. **Promote a new iteration bead:**
   ```bash
   NEW_BEAD=$(plugins/orb/scripts/promote.sh "<card_path>")
   bd dep add "$NEW_BEAD" "<closed-bead-id>" --type discovered-from
   ```

5. **Inject the cumulative constraint history into the new bead's
   description:**
   ```bash
   CONSTRAINTS=$(bd memories "drive-<card-slug>" --format text)
   bd update "$NEW_BEAD" --description "$(bd show "$NEW_BEAD" --json \
     | python3 -c "import sys,json;d=json.load(sys.stdin);print((d[0] if isinstance(d,list) else d)['description'])")

## Constraints carried from prior iterations

$CONSTRAINTS"
   ```

6. **Seed the new bead's metadata and claim:**
   ```bash
   bd update "$NEW_BEAD" \
     --set-metadata "drive_card=<card_path>" \
     --set-metadata "drive_autonomy=<level>" \
     --set-metadata "drive_iteration=$((<N>+1))" \
     --set-metadata "drive_stage=review-spec" \
     --set-metadata "drive_review_spec_cycle=0" \
     --set-metadata "drive_review_pr_cycle=0"
   bd update "$NEW_BEAD" --claim
   ```

7. **Re-enter at Stage 1** with the new bead. The constraint history
   is now in its description; the cold-fork reviewer reads it as
   part of the bead's `bd show <bead-id> --json` description field.

## Escalation

Escalation is triggered by **iteration budget exhaustion**
(`drive_iteration == 3` and current iteration NO-GO'd) OR by a
**semantic trigger** from the Disposition section (recurring failure
mode, contradicted hypothesis, diminishing signal). An honest agent
may escalate before the budget is spent.

1. **Set drive_stage and close:**
   ```bash
   bd update <bead-id> --set-metadata "drive_stage=escalated"
   bd close <bead-id> --reason "ESCALATED: <reason>"
   ```

2. **Output the escalation summary.** Iteration history is computed
   from the bead dep tree starting at iteration 1's bead:

   ```bash
   ITER1=$(bd dep list <current-bead-id> --type discovered-from --transitive --root)
   bd dep tree "$ITER1" --type discovered-from
   ```

   Format:

   ```
   DRIVE ESCALATED — <reason: budget exhausted | recurring failure | contradicted hypothesis | diminishing signal>

   Card: <card path>
   Goal: <card goal>

   Iteration history:
     1. <bead-id-iter1> — NO-GO: <constraint from bd memories>
     2. <bead-id-iter2> — NO-GO: <constraint>
     [3. <bead-id-iter3> — NO-GO: <constraint>]

   Accumulated constraints:
     - <all constraints from bd memories drive-<card-slug>>

   What would have to be true:
     <For a future attempt to succeed, what assumptions need revisiting?
      What constraints are structural vs configurational?
      What corner of the solution space was not explored?>

   Recommendation:
     <What the card needs before another drive attempt.>
   ```

3. **Heartbeat cleanup (full autonomy only).** Attempt `CronDelete
   drive-checkin-<bead-id>`. Non-fatal — failure logs `heartbeat
   cleanup skipped: <reason>` and continues. This step executes
   **before** the escalation ping so the recurring heartbeat can't
   fire between the summary output and the ping.

4. **One-shot escalation ping (full autonomy only).** Schedule
   `CronCreate` ~30 seconds out:
   - **Delay:** ~30 seconds (one-shot, not recurring).
   - **Task ID:** `drive-escalation-<bead-id>`.
   - **Prompt body (verbatim):**

     ```
     **DRIVE ESCALATED** on <card-slug> after <iterations> iterations. See prior output for findings and recommendation.
     ```

   If `CronCreate` for the ping fails, log `escalation ping skipped:
   <reason>` and continue. The escalation summary in step 2 is the
   authoritative channel; the ping is notification amplification.

5. **Stop.** The card needs human rethinking. Escalation is not giving
   up — it is the mechanism by which difficult work gets human
   judgment at the right moment.

## Resumption

When `/orb:drive` is invoked with a bead-id (or detects an in-progress
drive bead per §Input contract):

1. **Read the bead:** `bd show <bead-id> --json`. Extract:
   - `metadata.drive_stage`
   - `metadata.drive_iteration`
   - `metadata.drive_review_spec_cycle`, `drive_review_pr_cycle`
   - `metadata.drive_review_*_date`
   - `metadata.drive_card`, `drive_autonomy`

2. **Resume at the named stage.** No file-presence detection. The
   bead is the source of truth.

   | drive_stage   | Resume at                                         |
   |---------------|---------------------------------------------------|
   | `review-spec` | Stage 1 (idempotent §1.2 check skips fork if file already valid) |
   | `implement`   | Stage 2 (delegate to /orb:implement <bead-id>)    |
   | `review-pr`   | Stage 3                                           |
   | `complete`    | Already done — report status                      |
   | `escalated`   | Already escalated — report status                 |

3. **Synthetic-BLOCK resumption.** If
   `drive_review_<stage>_cycle == 3` and the bead is still
   in_progress (the synthetic BLOCK was not written before the
   session died), synthesise the BLOCK on resume per §1.6 — do not
   launch a 4th fork.

4. **Heartbeat reconciliation (full autonomy only).** Re-run the
   §Heartbeat CronList-first flow: if `drive-checkin-<bead-id>`
   exists, leave it; if absent, re-create it. Drive never
   delete-then-recreates, so a surviving task is preserved.

5. **Announce the resumption:**

   ```
   Resuming drive for <bead-id> (<bead title>)
   Card: <drive_card>
   Autonomy: <drive_autonomy>
   Iteration: <drive_iteration> of 3
   Resuming at: <drive_stage>
   Review cycles: review-spec=<N>/3, review-pr=<N>/3
   Heartbeat: <active | created | n/a (non-full autonomy) | unavailable>
   ```

## Critical Rules

- **Never skip a stage.** Promote → Review-Spec → Implement → Review-PR,
  always in order.
- **Never silently downgrade autonomy.** If full mode is requested but
  the card is thin, refuse explicitly.
- **The bead is the single source of orchestration state.** Do not
  track drive state in any file.
- **Reviews run as forked Agents.** Drive launches each review via the
  Agent tool (`subagent_type: general-purpose`) in a fresh context.
  Verify Agent availability via ToolSearch first; do not fall back to
  inline review.
- **Verdicts are read from disk only.** The review file's canonical
  verdict line (`**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`,
  matched by strict regex) is the single authoritative source. The
  fork's chat response is never parsed.
- **Re-reviews are fully cold.** When a REQUEST_CHANGES cycle re-forks
  the reviewer, the new fork's brief is functionally identical to the
  first cycle's brief — no path to prior review files, no iteration
  counter, no summary of prior findings.
- **REQUEST_CHANGES is bounded per stage.** Each stage has an
  independent 3-cycle budget. The 4th would-be cycle is converted to
  a synthetic BLOCK with the byte-identical canonical constraint
  string.
- **Iteration is bounded by 3 beads in the discovered-from chain.**
  After three NO-GOs, drive escalates. Earlier escalation is
  permitted on semantic triggers.
- **Live-visibility heartbeat is full-autonomy-only and
  self-terminating.** The cron prompt body has a read-only contract
  ("do not modify the bead, do not launch any Agent") with one
  exception: when `bead.status == closed`, the heartbeat
  `CronDelete`s itself.
- **Cron tasks are reconciled idempotently.** CronList-then-CronCreate-
  iff-absent on every initialise and every resume. Drive never
  delete-then-recreates a heartbeat task.
- **MEDIUM+ review verdicts route through the four-option
  AskUserQuestion prompt.** LOW-only or zero-finding APPROVE gates
  retain the existing shorter prompts. Drive reads severity from the
  review file directly and never re-classifies. `read full review
  first` is a deferral; drive re-presents the same prompt verbatim
  on the author's next turn.

## Disposition

The drive's job is to **find the way through, not the evidence that
closes the card.** When an iteration falls short, the first question
is *what would have to be true for this to work*, not *what does this
rule out*. Push past the first plateau. Try the approaches that look
uncomfortable. Treat negative results as constraints on the next
iteration, not as conclusions.

### Bounded by honest escalation

Disposition and escalation are the same stance, not opposing ones.
Commitment to the goal is bounded by honest reporting. Escalate —
don't push through, and don't quietly close — when any of these are
true:

- **Recurring failure mode.** The same problem has appeared across 2+
  iterations despite varied approaches to address it. The constraint
  may be structural, not configurational.
- **Contradicted hypothesis.** The accumulated evidence points to the
  card's *underlying goal* being unreachable, not just the current
  approach falling short. The call to pivot a thesis belongs to the
  author.
- **Diminishing signal.** Each iteration is producing less new
  information than the last. The drive is grinding, not learning.

These semantic triggers supplement the mechanical iteration budget. An
agent with the right disposition may escalate at iteration 2 when the
evidence warrants it, or push hard through all 3 when each iteration
genuinely narrows the search space.

## Worked example

A copy-pasteable trace for a card → bead → close happy path. Each
step is a literal command.

```bash
# 1. Validate the card (full autonomy requires ≥3 scenarios)
python3 -c "
import yaml
with open('orbit/cards/0005-drive.yaml') as f:
    card = yaml.safe_load(f)
n = len(card.get('scenarios', []))
assert n >= 3, f'BLOCKED: full autonomy requires ≥3 scenarios; have {n}'
print(f'OK: {n} scenarios')
"

# 2. Promote card → bead
BEAD=$(plugins/orb/scripts/promote.sh orbit/cards/0005-drive.yaml)
echo "Promoted: $BEAD"

# 3. Seed orchestration metadata + claim
bd update "$BEAD" \
  --set-metadata "drive_card=orbit/cards/0005-drive.yaml" \
  --set-metadata "drive_autonomy=full" \
  --set-metadata "drive_iteration=1" \
  --set-metadata "drive_stage=review-spec" \
  --set-metadata "drive_review_spec_cycle=0" \
  --set-metadata "drive_review_pr_cycle=0"
bd update "$BEAD" --claim

# 4. Schedule heartbeat (full autonomy)
# CronList → CronCreate iff absent (drive-checkin-<bead-id>, 5 min recurring)

# 5. Stage 1: fork review-spec (reads bead directly via bd show + parse-acceptance.sh)
mkdir -p "orbit/reviews/$BEAD"
bd update "$BEAD" --set-metadata "drive_review_spec_date=$(date -I)"
# Agent({ subagent_type: 'general-purpose', prompt: <brief naming $BEAD + output path; reviewer reads bd show + parse-acceptance.sh> })
# Parse verdict from orbit/reviews/$BEAD/review-spec-$(date -I).md
# APPROVE → drive_stage=implement

# 6. Stage 2: delegate to /orb:implement
bd update "$BEAD" --set-metadata "drive_stage=implement"
# (invoke /orb:implement with $BEAD)
# When parse-acceptance.sh has-unchecked $BEAD exits 1:
bd update "$BEAD" --set-metadata "drive_stage=review-pr"

# 7. Stage 3: fork review-pr
# (mirrors stage 1; brief includes git diff main...HEAD + bead-id for AC cross-reference)

# 8. APPROVE at review-pr → completion
git add -A
git commit -m "feat: $(bd show $BEAD --json | python3 -c "import sys,json;d=json.load(sys.stdin);print((d[0] if isinstance(d,list) else d)['title'])")"
gh pr create --title "drive: <bead title>" --body "<refs $BEAD and reviews>"
bd update "$BEAD" --set-metadata "drive_stage=complete"
bd close "$BEAD" --reason "drive completed: <one-line summary>"

# 9. Heartbeat cleanup (non-fatal)
# CronDelete drive-checkin-$BEAD || echo "heartbeat cleanup skipped"
```

---

**Next step:** after `bd close` at completion, the PR is ready for
human review and merge.
