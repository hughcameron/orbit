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

- `card_path` — path to a card YAML file (e.g. `cards/0005-drive.yaml`)
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
- **First iteration:** `specs/YYYY-MM-DD-<card-slug>/` (derive slug from card filename, e.g. `0005-drive` → `drive`)
- **Subsequent iterations:** `specs/YYYY-MM-DD-<card-slug>-v<N>/` (e.g. `drive-v2`, `drive-v3`)

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
```

If `drive.yaml` already exists, read it and **resume from the recorded state** (see §11 Resumption).

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

Read the review-spec skill instructions from `plugins/orb/skills/review-spec/SKILL.md`. Follow its instructions **inline** — read the spec cold, run the assumption audit, failure mode analysis, test adequacy check, gap analysis, and constraint check.

**Important:** Do NOT invoke `/orb:review-spec` as a skill call. Read the SKILL.md file and follow its instructions directly within this session. The review runs inline to keep the drive's single-session model intact.

**Output:** Save `review-spec-<date>.md` in the current spec directory.

**Review-spec verdict handling:**

- **APPROVE:** Proceed to Implement (§6).
- **REQUEST_CHANGES:** Address the specific changes in the spec. Re-run the review. If the review now approves, proceed.
- **BLOCK:** Jump to §8 (NO-GO Handling). The spec needs rework — the block reason becomes the NO-GO constraint.

Update `drive.yaml`: `status: implement`

**Supervised mode gate:** If autonomy is `supervised`, pause here:
```
AskUserQuestion: "Spec review complete — verdict: <verdict>. <N> findings (<severities>). Review saved at <path>. Proceed to implementation?"
Suggested answers: ["GO — proceed to implement", "NO-GO — re-enter at design"]
```
If NO-GO → jump to §8 (NO-GO Handling).

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

Read the review-pr skill instructions from `plugins/orb/skills/review-pr/SKILL.md`. Follow its instructions **inline** — read the diff, check AC coverage, probe edge cases, write the review file.

**Important:** Do NOT invoke `/orb:review-pr` as a skill call. Read the SKILL.md file and follow its instructions directly within this session. The review runs inline to preserve the full implementation context.

**Output:** Save `review-pr-<date>.md` in the current spec directory.

**Review verdict handling:**

- **REQUEST_CHANGES:**
  - Address the specific changes requested in the review.
  - Re-run the review after fixes.
  - If changes are addressed and review now approves, continue to APPROVE handling below.

- **BLOCK (NO-GO):**
  - Jump to §8 (NO-GO Handling).

- **APPROVE:**
  - **In full mode:** Proceed directly to §10 (Completion).
  - **In guided mode:** This is the **only gate in guided mode**. Present a rich summary of the entire drive:
    ```
    AskUserQuestion: "Drive summary for <card name>:

    Spec: <spec path> — <goal summary>
    Spec review: <verdict>, <N> findings
    Implementation: <N>/<total> ACs addressed
    PR review: APPROVE — <honest assessment one-liner>

    Review saved at <path>. Proceed to PR creation?"
    Suggested answers: ["GO — create PR", "NO-GO — re-enter at design", "Let me read the reviews first"]
    ```
    If "Let me read the reviews first" → wait for the author to respond after reading, then re-present the gate.
    If NO-GO → jump to §8 (NO-GO Handling).
  - **In supervised mode:** Same gate as guided.

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

3. **Increment iteration:**
   ```yaml
   iteration: <current + 1>
   current_spec: <new spec directory path>
   status: design
   ```

4. **Create the new spec directory** (e.g. `specs/YYYY-MM-DD-drive-v2/`).

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

3. **Stop.** The card needs human rethinking. Escalation is not giving up — it is the mechanism by which difficult work gets human judgment at the right moment.

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

### 11. Resumption

When `/orb:drive` is invoked and `drive.yaml` already exists in the expected location:

1. **Read `drive.yaml`** to determine current state.
2. **Determine which stage to resume from** using file-presence detection:

   | drive.yaml status | Files present | Resume at |
   |-------------------|---------------|-----------|
   | `design` | no interview.md | Design (§3) |
   | `spec` | interview.md, no spec.yaml | Spec (§4) |
   | `review-spec` | spec.yaml, no review-spec-*.md | Review-Spec (§5) |
   | `implement` | review-spec-*.md, no progress.md | Implement (§6) |
   | `review` | progress.md, no review-pr-*.md | Review-PR (§7) |
   | `complete` | review-pr-*.md | Already done — report status |
   | `escalated` | — | Already escalated — report status |

3. **File presence overrides drive.yaml status** when they disagree. If drive.yaml says `implement` but `progress.md` already exists with completed ACs, advance to `review`. The files are ground truth; drive.yaml may be stale from an interrupted session.

4. **Announce the resumption:**
   ```
   Resuming drive for <card path>
   Autonomy: <level>
   Iteration: <N> of <budget>
   Resuming at: <stage>
   ```

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
- **Both reviews run inline.** Do not invoke `/orb:review-spec` or `/orb:review-pr` as skill calls — read their SKILL.md files and follow the instructions within this session.
- **Reviews are the quality gates.** In guided mode, the spec review and PR review replace explicit go/no-go prompts. The only interactive gate is the final verdict summary before PR creation.
