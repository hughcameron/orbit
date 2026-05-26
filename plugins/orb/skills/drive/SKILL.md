---
name: drive
description: Drive a card or spec through the orbit pipeline — promote → review-spec → implement → review-pr — at a declared autonomy level
argument-hint: "[card_path | spec-id] [full|guided|supervised]"
disable-model-invocation: true
allowed-tools: Bash Read Edit Write Agent AskUserQuestion CronCreate CronList CronDelete TaskCreate TaskUpdate
---

# /orb:drive

Take a card and an autonomy level. Drive the pipeline (promote →
review-spec → implement → review-pr) as a single session. Drive state
lives in `.orbit/specs/<spec-id>/drive.yaml` (sidecar layout — see
`.orbit/conventions/spec-layout.md`). AC state lives in the spec's
`acceptance_criteria` field.

## Usage

```
/orb:drive <card_path> [full|guided|supervised]   # fresh drive from card
/orb:drive <spec-id>                              # resume an in-flight drive
/orb:drive                                        # resume the unique in-progress drive, if any
```

### Autonomy Levels

| Level | Behaviour |
|-------|-----------|
| **full** | Agent self-answers in promote / review gates. APPROVE auto-merges via `gh pr merge --auto`. Requires ≥3 card scenarios. |
| **guided** | Promote runs autonomously. The reviews ARE the quality gates; author approves the final review-pr verdict before PR creation. Default. |
| **supervised** | Author greenlights after each stage. |

## Input contract

Verify the working tree carries an initialised `.orbit/` substrate via
the resolver; halt if absent. The skill operates on exactly one drive
per session. Resolution branches:

1. **Card path provided** — pre-flight thin-card refusal, then promote.
2. **Spec-id provided** — validate `<spec-id>/drive.yaml` exists; if not,
   halt and instruct the agent to re-invoke with a card path. Otherwise
   resume from the sidecar's `stage` field.
3. **No argument** — call the canonical resolver and filter for specs
   that have a drive sidecar:

   ```bash
   orbit --json spec resolve --skill drive
   ```

   Apply the three-step recovery from spec
   `2026-05-19-skills-infer-or-prompt-before-halt`:

   - `outcome=resolved` → if `.orbit/specs/<id>/drive.yaml` exists, resume; otherwise halt (open spec without drive = promote candidate).
   - `outcome=prompt` → narrow `candidates[]` to those with a drive sidecar. Zero → halt; single → resume; multiple → AskUserQuestion with `id` + `goal_first_line` per candidate.
   - Non-zero exit with `spec.resolve: unavailable: ...` → surface verbatim.

## Pre-flight (card-path branch only)

Thin cards block full autonomy. Count `scenarios`; if < 3:

- `full` → **REFUSE** with:
  ```
  BLOCKED: Card has N scenario(s) — full autonomy requires ≥3.
  Missing coverage areas to consider:
  - <suggest what scenarios are absent based on the card's goal>
  Add scenarios with /orb:card and retry.
  ```
  Do not silently downgrade.
- `guided` / `supervised` → proceed.

## Promote

```bash
SPEC_ID=$(orbit spec promote "<card_path>")
```

`orbit spec promote` materialises the spec (orbit-state v0.1) seeded
from the card's scenarios as ACs. Write the drive sidecar:

```bash
cat > ".orbit/specs/$SPEC_ID/drive.yaml" <<EOF
spec_id: $SPEC_ID
card_path: <absolute card_path>
autonomy: <full|guided|supervised>
iteration: 1
stage: review-spec
review_spec_cycle: 0
review_spec_date: null
review_pr_cycle: 0
review_pr_date: null
iteration_history: []
EOF
```

Schedule the heartbeat (full only) and proceed to Stage 1.

## Heartbeat (full autonomy only)

Skip when `autonomy != full`. CronList-first idempotent reconciliation
(never delete-then-recreate, preserving `--resume` task restoration):

1. `CronList` — enumerate active cron tasks.
2. If `drive-checkin-<spec-id>` exists, no-op.
3. Otherwise `CronCreate` recurring with ID `drive-checkin-<spec-id>`,
   interval 5 minutes, prompt body verbatim:

```
This is a drive heartbeat. Run `orbit --json spec show <spec-id>` and
read its `status`. Read `.orbit/specs/<spec-id>/drive.yaml` and read
its `stage`.

If status is `closed` (drive stage `complete` or `escalated`), call
CronDelete with ID `drive-checkin-<spec-id>` and emit:

  drive: heartbeat stopped (stage=<drive_stage>)

Then stop. Do not emit a heartbeat line.

Otherwise: run `orbit spec next-ac
<spec-id>` to find the current AC (if it returns nothing or stage is
not `implement`, use `-`). Compute elapsed as mm:ss since the drive
sidecar's `started_at` field (set on the first heartbeat tick if
absent). Emit exactly one line in the format:

  drive: spec=<spec-id> stage=<stage> ac=<id|-> elapsed=<mm:ss>

Do not modify the spec. Do not launch any Agent. Emit the single
heartbeat line and stop.
```

If `CronCreate` fails, log `heartbeat unavailable: <reason>` and
continue — observability, not a gate.

## Stage 1: Review-Spec

Forked Agent in cold context.

### 1.1 Cycle-specific verdict path

Let N = `review_spec_cycle + 1`. If `review_spec_date` is null, set it
to today's ISO date and write back. Output path:

- Cycle 1: `.orbit/specs/<spec-id>/review-spec-<date>.md`
- Cycle 2: `.orbit/specs/<spec-id>/review-spec-<date>-v2.md`
- Cycle 3: `.orbit/specs/<spec-id>/review-spec-<date>-v3.md`

### 1.2 Idempotent resumption

If the cycle-specific path exists AND contains a line matching the
verdict regex (§1.4), parse and proceed to §1.5 without launching any
Agent. Otherwise continue.

### 1.3 Launch the forked review

Run `ToolSearch select:Agent` to load the schema. If unavailable, set
`stage: escalated` in `drive.yaml`, output `Agent tool unavailable —
cannot launch cold-fork review for review-spec`, stop. No inline
fallback.

Invoke Agent with `subagent_type: general-purpose` and a brief
containing only: spec-id, absolute output path, instruction to read via
`orbit --json spec show <spec-id>` + `orbit spec acs <spec-id>`, follow
`/orb:review-spec`, and write the canonical verdict line.

Example brief:

```
Run /orb:review-spec on spec <spec-id>. Read the spec via `orbit --json
spec show <spec-id>` and parse ACs via `orbit spec acs <spec-id>` —
the spec's acceptance_criteria field is the authoritative spec for
this review. Write the review to exactly <absolute output path> (this
path takes precedence over the default path in the skill). Use the
canonical verdict line format `**Verdict:** APPROVE | REQUEST_CHANGES
| BLOCK`.
```

The brief must NOT include conversation context, iteration counter,
cycle number, or prior-review pointers.

### 1.4 Parse the verdict from the file

Locate the first line matching:

```
^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$
```

Case-sensitive on the token. If zero matches or file missing, retry
once with a fresh identical brief (overwriting the same path; does NOT
increment `review_spec_cycle`). If retry also fails, escalate with
`stage: escalated` and message `review could not be completed after 2
forked attempts at review-spec`.

### 1.5 Verdict handling

Under guided/full autonomy, APPROVE advances; supervised pauses once
after review-pr APPROVE with the canonical four-option prompt.

- **APPROVE** — set `stage: implement`, proceed to Stage 2.
- **REQUEST_CHANGES** — increment `review_spec_cycle`, check budget
  (§1.6). If budget remains, address findings (`orbit spec update
  --goal "..."`, `--ac-check / --ac-uncheck`, or `orbit spec note`),
  return to §1.1.
- **BLOCK** — jump to §NO-GO. The block reason is the constraint.

### 1.6 REQUEST_CHANGES budget & synthetic BLOCK

Each stage has an **independent budget of 3 REQUEST_CHANGES cycles per
top-level iteration**. Counters live in `drive.yaml`
(`review_spec_cycle`, `review_pr_cycle`) and reset to 0 on new
iteration (§NO-GO).

After increment: < 3 → another cycle. == 3 → synthesise BLOCK with the
canonical constraint string (fixed, byte-identical with spec ac-05
verification target):

> `review converged on REQUEST_CHANGES after 3 iterations; findings have not been addressable within budget`

Jump to §NO-GO. Resumption case: if drive resumes with
`review_<stage>_cycle == 3` and the synthetic BLOCK was not yet written,
synthesise on resume — do not launch a 4th fork.

## Stage 2: Implement

Drive sets `stage: implement` in `drive.yaml` and delegates to
`/orb:implement <spec-id>`. When implement returns (no unchecked ACs
via `orbit spec has-unchecked <spec-id>` exiting 1), drive sets
`stage: review-pr` and proceeds to Stage 3.

## Stage 3: Review-PR

Mirrors Stage 1 with the diff brief. Forked reviewer reads
post-implement spec state via `orbit --json spec show <spec-id>` and
`orbit spec acs <spec-id>`.

### 3.1 Cycle-specific verdict path

Using `review_pr_cycle` / `review_pr_date`:

- Cycle 1: `.orbit/specs/<spec-id>/review-pr-<date>.md`
- Cycle 2: `.orbit/specs/<spec-id>/review-pr-<date>-v2.md`
- Cycle 3: `.orbit/specs/<spec-id>/review-pr-<date>-v3.md`

### 3.2 Fork launch and verdict parse

As §1.2 / §1.3 / §1.4. Brief includes the diff reference (`git diff
main...HEAD` on current branch) plus spec-id. Counter / date fields use
`review_pr_*`. Retry escalation message: `review could not be completed
after 2 forked attempts at review-pr`.

Example brief:

```
Run /orb:review-pr against the current branch. Implementation diff is
`git diff main...HEAD` on <branch_name>. Spec acceptance is on spec-id
<spec-id>; read via `orbit --json spec show <spec-id>` and
`orbit spec acs <spec-id>`. Write the review to exactly <absolute
output path> (this path takes precedence over the default path in the
skill). Use the canonical verdict line format `**Verdict:** APPROVE |
REQUEST_CHANGES | BLOCK`.
```

### 3.3 Verdict handling

Under guided/full autonomy, APPROVE advances; supervised pauses once
here with the canonical four-option prompt. Suggested answers are
verbatim, lower-case, single spaces, no hyphens, no punctuation:
`approve` / `request changes` / `block` / `read full review first`.
`approve` advances to §Completion; `request changes` increments
`review_pr_cycle` and re-enters §1.6 budget-gated; `block` jumps to
§NO-GO with constraint `author blocked post-APPROVE at MEDIUM+ PR
review`; `read full review first` defers — drive waits for the
author's next turn and re-presents the same prompt verbatim.

- **REQUEST_CHANGES** — increment `review_pr_cycle`, check budget
  (§1.6); address findings and return to §3.1, or synthesise BLOCK.
- **BLOCK** (real or synthetic) — jump to §NO-GO.
- **APPROVE** — full: §Completion direct. Guided/supervised: present
  the four-option prompt prefaced by a one-block drive summary (card
  name, spec-id, goal, spec review verdict + findings, implementation
  AC count, PR review verdict, review file path).

## Completion

On APPROVE at review-pr. REQUEST_CHANGES / BLOCK NEVER reach here —
they route through §NO-GO.

1. **Commit implementation:** code + review files, message `feat: <spec goal>`.

2. **Propose card updates:** update `maturity` if appropriate; refine
   `goal` if implementation revealed sharper success criteria. Message
   `docs: update <card> — maturity and goal after drive`.

3. **Push** (all autonomy levels): `git push -u origin <branch>` —
   precondition for step 4, not full-only.

4. **Create the PR** (idempotent):
   ```bash
   gh pr view --json number,autoMergeRequest,state 2>/dev/null
   ```
   If a PR already exists for the branch (`state: OPEN`,
   `autoMergeRequest: null`), skip create and carry forward. Otherwise
   create with title `drive: <spec goal>` and body referencing spec-id
   + review files.

5. **Merge the PR** (full autonomy only): `gh pr merge --auto` with the
   repo's merge strategy (`--squash` / `--merge` / `--rebase`). If
   `gh pr merge --auto` returns non-zero, log a reason token via
   `orbit spec note` and continue to step 6; full-autonomy drives
   degrade quietly rather than halting.

6. **Close the spec:**
   ```bash
   # Edit drive.yaml: stage: complete
   orbit spec note <spec-id> "drive completed: <one-line summary>"
   orbit spec close <spec-id>
   ```

   `spec.close` transactionally appends the spec's path to every linked
   card's `specs` array. It rejects on open child tasks; resolve first.

   **AC pre-flight before close.** `spec.close` rejects when any
   non-time-gated AC remains `checked: false` (spec
   2026-05-13-spec-close-ac-preflight). The error names the offending
   AC ids and flags gate ACs separately.

   - **Reconcile first** — tick the missing AC(s) (`orbit spec check
     <spec-id> <ac-id>`) and re-invoke `spec close`.
   - **`--force` is the deliberate escape.** When ACs are genuinely
     unfinished (review NO-GO, scoped deferral, mid-pipeline halt):
     ```bash
     orbit spec note <spec-id> "force-close: ac-04, ac-07 unfinished — <reason>"
     orbit spec close --force <spec-id>
     ```
   - **Deferrable-kind ACs never block close.** `ops` / `observation`
     ACs surface in `deferrable_open` automatically.

**Post-close heartbeat cleanup (full only).** Attempt `CronDelete
drive-checkin-<spec-id>`. Non-fatal — log `heartbeat cleanup skipped:
<reason>` and continue.

## NO-GO Handling

Current iteration failed a review (real or synthetic BLOCK) or was
rejected at a supervised gate.

1. **Note and close:**
   ```bash
   orbit spec note <spec-id> "NO-GO: <one-line constraint>"
   orbit spec close <spec-id>
   ```
   If `spec close` rejects on open child tasks, `orbit task done
   <task-id>` first.

2. **Persist the constraint:**
   ```bash
   orbit memory remember drive-<card-slug>-iter<N> "<constraint>"
   ```
   Key format is stable so iteration ≥2 can list priors via
   `orbit memory search drive-<card-slug>`.

3. **Check budget:** if `iteration == 3`, jump to §Escalation.

4. **Promote new iteration spec:**
   ```bash
   NEW_SPEC=$(orbit spec promote "<card_path>")
   ```

5. **Inject cumulative constraints:**
   ```bash
   CONSTRAINTS=$(orbit --json memory search "drive-<card-slug>" \
     | jq -r '.data.result.memories[] | "- " + .body')
   orbit spec note "$NEW_SPEC" "Constraints carried from prior iterations:
   $CONSTRAINTS"
   ```

6. **Seed new drive sidecar:**
   ```bash
   cat > ".orbit/specs/$NEW_SPEC/drive.yaml" <<EOF
   spec_id: $NEW_SPEC
   card_path: <card_path>
   autonomy: <level>
   iteration: $((<N>+1))
   stage: review-spec
   review_spec_cycle: 0
   review_spec_date: null
   review_pr_cycle: 0
   review_pr_date: null
   iteration_history:
     - spec_id: <closed-spec-id>
       iteration: <N>
       outcome: NO-GO
       constraint: <one-line>
   EOF
   ```

7. **Re-enter at Stage 1** with the new spec.

## Escalation

The drive's job is to find the way through, not the evidence that
closes the card. Escalation gets human judgment at the right moment.

Triggered by **iteration budget exhaustion** (`iteration == 3` and
current iteration NO-GO'd) OR by a **semantic trigger** — recurring
failure mode (same problem across 2+ iterations despite varied
approaches), contradicted hypothesis (card's underlying goal appears
unreachable), or diminishing signal (each iteration learning less).

### Steps

1. **Set stage and close:**
   ```bash
   # Edit drive.yaml: stage: escalated
   orbit spec note <spec-id> "ESCALATED: <reason>"
   orbit spec close <spec-id>
   ```

2. **Output escalation summary.** Walk `iteration_history` across each
   iteration's `drive.yaml`. Format:

   ```
   DRIVE ESCALATED — <reason: budget exhausted | recurring failure | contradicted hypothesis | diminishing signal>

   Card: <card path>
   Goal: <card goal>

   Iteration history:
     1. <spec-id-iter1> — NO-GO: <constraint>
     2. <spec-id-iter2> — NO-GO: <constraint>
     [3. <spec-id-iter3> — NO-GO: <constraint>]

   Accumulated constraints:
     - <all constraints from orbit memory search drive-<card-slug>>

   What would have to be true:
     <assumptions to revisit; structural vs configurational read;
      unexplored corners of the solution space>

   Recommendation:
     <What the card needs before another drive attempt.>
   ```

3. **Heartbeat cleanup (full only).** `CronDelete
   drive-checkin-<spec-id>`. Non-fatal. Runs **before** the ping so
   the recurring heartbeat can't fire between summary and ping.

4. **One-shot escalation ping (full only).** `CronCreate` ~30s out:
   - Task ID: `drive-escalation-<spec-id>`.
   - Prompt body (verbatim):
     ```
     **DRIVE ESCALATED** on <card-slug> after <iterations> iterations. See prior output for findings and recommendation.
     ```

   If `CronCreate` fails, log `escalation ping skipped: <reason>` and
   continue.

5. **Stop.**

## Critical Rules

Invariants that must always hold.

- **drive.yaml is the single source of orchestration state.** Its
  `stage` field is the source of truth for resumption.
- **Reviews run as forked Agents in cold context.** Re-reviews after
  REQUEST_CHANGES are functionally identical to first-cycle reviews.
- **Verdicts are read from disk only.** The canonical verdict line
  (§1.4 regex) is authoritative. The fork's chat response is never
  parsed.
- **REQUEST_CHANGES is bounded per stage** (3-cycle budget per
  iteration). The 4th would-be cycle synthesises BLOCK with the
  byte-identical §1.6 constraint string.
- **Iteration is bounded by 3 specs** in the `iteration_history` chain.
  Earlier escalation is permitted on semantic triggers.
- **Never silently downgrade autonomy.** Thin-card guard is a
  pre-qualification gate, not a runtime decision.
- Per hook `three-question-test.sh`, mid-autonomy AskUserQuestion calls
  are gated under `ORBIT_NONINTERACTIVE=1`; the discipline lives in the
  hook, not in skill prose.

## Resumption

When `/orb:drive` is invoked with a spec-id (or detects an in-progress
drive per §Input contract):

1. **Read `.orbit/specs/<spec-id>/drive.yaml`.** Extract `stage`,
   `iteration`, `review_spec_cycle`, `review_pr_cycle`,
   `review_spec_date`, `review_pr_date`, `card_path`, `autonomy`.

2. **Resume at the named stage.** `drive.yaml` is the source of truth.

   | stage         | Resume at                                                        |
   |---------------|------------------------------------------------------------------|
   | `review-spec` | Stage 1 (idempotent §1.2 check skips fork if file already valid) |
   | `implement`   | Stage 2 (delegate to /orb:implement <spec-id>)                   |
   | `review-pr`   | Stage 3                                                          |
   | `complete`    | Already done — report status                                     |
   | `escalated`   | Already escalated — report status                                |

3. **Synthetic-BLOCK resumption.** If `review_<stage>_cycle == 3` and
   spec still open, synthesise the BLOCK on resume per §1.6.

4. **Heartbeat reconciliation (full only).** Re-run the CronList-first
   flow: existing task stays, absent task is recreated.

5. **Announce the resumption** in one line: spec id, stage, iteration,
   review-cycle counts, heartbeat status.
