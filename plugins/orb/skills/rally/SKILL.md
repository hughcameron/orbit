---
name: rally
description: Coordinate multiple independent cards through the orbit pipeline as a single multi-card delivery — proposal → queued design decisions → consolidated design review → implementation → stacked or batched review
---

# /orb:rally

Drive **multiple** independent cards through the orbit pipeline as a coordinated rally. The rally skill owns multi-card orchestration; individual cards run `/orb:drive` in full autonomy (either serially in the main checkout or in parallel inside isolated worktrees).

Rally exists because serial `/orb:drive` invocations accumulate human touchpoints — every card pauses the author for design, spec, review, and PR. A rally packs the human work into two high-signal gates (ideation and assurance) and lets the agent work between them with maximum clarity based on the best available evidence.

## Usage

```
/orb:rally <goal_string> [guided|supervised]
```

- `goal_string` — a short description of the subsystem, theme, or objective binding the cards together (e.g. `"pipeline runtime readiness"`, `"review workflow hardening"`)
- Autonomy defaults to **guided** if omitted

### Autonomy Levels

| Level | Behaviour |
|-------|-----------|
| **guided** | Proposal and consolidated decision gate are interactive. Reviews serve as quality gates — no intermediate supervision between design and implementation. Default. |
| **supervised** | Same as guided plus explicit pauses after each rally phase (design, review, implementation, PR) for author greenlight. |

`full` autonomy is **not offered** — rally's value comes from sharper human gates, not fewer.

**Rally-level vs drive-level autonomy.** Rally-level autonomy (guided | supervised) governs pauses between rally phases — proposal, consolidated decision gate, consolidated design review, batched diff review. Drive-level autonomy inside a rally is **always full**, both for parallel sub-agents running in worktrees and for serial cards running in the main checkout. A thin card run outside a rally via individual `/orb:drive` can choose any drive autonomy, but once inside a rally, drive runs full regardless of serial-or-parallel.

## Why This Exists

Observed pattern (from parallel design across 3 cards):

- **Improvised output paths** — sub-agents wrote files to invented locations
- **Serial surprise** — designs that looked independent revealed shared trait changes only once designs were written
- **No durable plan** — the session died and the coordination state was lost

Rally resolves each with a declared artefact path, a definitive post-design disjointness check, and `specs/rally.yaml` as durable state.

> **Principle:** The goal of rally is to have the highest quality interactions at ideation and assertion. This means maximum clarity based on the best available evidence. Agent work between gates exists to make the next gate sharper — not just faster.

> **Honesty principle:** Rally describes what Claude Code's primitives actually provide. Path discipline for design sub-agents is **trust + post-verify** — convention imposed by the brief, verified by the lead on return. No tool-level path guard is claimed, and no PreToolUse hook is registered. Where the primitive is real (`Agent tool`, `run_in_background`, `git worktree add`, `git status --porcelain`, `git commit`, `git checkout`), it is named inline.

## Instructions

### 1. Pre-Flight: Check for an Active Rally

Before anything else, check whether a rally is already in flight.

```
if specs/rally.yaml exists:
  read and validate it (see §11 Resumption + §12 Validation)
  if phase != "complete":
    resume from the recorded phase — do not start a new rally
  if phase == "complete":
    offer to archive the old rally.yaml and start a new one
```

**One active rally at a time** is non-negotiable. A new rally on top of an active one would scatter sub-agent briefs across overlapping spec directories and make resumption ambiguous.

### 2. Propose the Rally

Parse the goal string from `$ARGUMENTS[0]` and autonomy from `$ARGUMENTS[1]` (default `guided`).

**Scan `cards/` for candidate cards:**

1. Read every `cards/*.yaml` (ignore `cards/memos/`)
2. For each card, score relevance to the goal string using the card's `feature`, `goal`, `scenarios`, and `references`
3. Surface the top candidates (usually 3–6) with a one-line rationale per card

#### 2a. Thin-card guard (refuse at proposal)

Before the proposal is presented to the author, check the scenario count on every candidate card. If any candidate has fewer than **3 scenarios**, the proposal refuses to proceed:

```
Rally cannot proceed — the following candidate card is too thin:

  cards/0017-<slug>.yaml — 2 scenarios

Thicken this card via `/orb:card cards/0017-<slug>.yaml` or remove it from
the rally list before continuing.
```

The author may then:

- Run `/orb:card` on the thin card to thicken it, re-invoke rally
- Remove the thin card from the list and re-invoke rally
- Run the thin card individually via `/orb:drive <card> guided` or `supervised` (rally is not the venue for thin cards)

The thin-card refusal is **unconditional on the eventual serial-or-parallel outcome**. The guard runs before the proposal is shown and before the post-design disjointness check (§6c) — it is a pre-qualification gate, not a runtime decision. (ac-01)

#### 2b. Present the proposal using AskUserQuestion

```
## Rally Proposal — <goal string>

Candidate cards:
  1. cards/<id>-<slug>.yaml — <feature line>
     Rationale: <why this card fits the goal>
  2. cards/<id>-<slug>.yaml — <feature line>
     Rationale: <why this card fits the goal>
  ...

Autonomy: <guided|supervised>
```

Offer the author these choices:
- **Approve as-is** — proceed with the proposed list
- **Modify the list** — author names cards to add or remove (free-form response)
- **Reject the rally** — abort, offer alternatives (e.g. individual drives)

If the author adds a card not in the scan's top-N, include it — then re-run the thin-card guard against the new candidate. If the author removes a card, drop it. Loop on "modify" until the author approves or rejects.

**The proposal gate is the only pre-design independence check.** The agent's scan proposes; the author's approval qualifies. Do not attempt a lightweight heuristic disjointness check — the definitive check happens after designs exist (§6).

### 3. Initialise Rally State

**rally.yaml lives only on main.** The lead writes rally.yaml while checked out to main. Before delegating to a card's rally branch (serial) or its worktree (parallel), the lead returns to main to write rally.yaml, then checks out again. Rally branches never contain rally.yaml.

On approval, create `specs/rally.yaml` while on main:

```yaml
rally: "<goal string>"
autonomy: <guided|supervised>
phase: approved
started: <ISO-8601 timestamp>
completed: null
cards:
  - path: cards/0015-<slug>.yaml
    status: proposed
    spec_dir: specs/YYYY-MM-DD-<slug>/
    branch: rally/<slug>
    worktree: null          # null until launch; 'main' for serial, absolute path for parallel
    parked_constraint: null
  - path: cards/0016-<slug>.yaml
    status: proposed
    spec_dir: specs/YYYY-MM-DD-<slug>/
    branch: rally/<slug>
    worktree: null
    parked_constraint: null
implementation_order: null
order_rationale: null
```

For each card, derive `spec_dir` from today's date plus the card slug, and create the directory. Assign a branch name (`rally/<slug>`) for later stacking.

Update `phase: designing` once sub-agents launch (§4).

**Write rally.yaml at every phase transition and per-card status change** — proposing → approved → designing → design-review → implementing → complete. Writes happen at transitions, not mid-phase. Every rally.yaml write is performed while checked out to main (`git checkout main` → edit → `git commit` → `git checkout <rally/slug>` or enter worktree). (ac-09, constraint #12)

### 4. Stage: Design — Queued Decision Packs

This is the rally's central innovation. The goal: present the author with executive-ready decisions in a single consolidated gate, with options + trade-offs + recommendations drawn from the best available evidence — not raw questions they lack context to answer.

**4a. Launch N design sub-agents in parallel.**

Design sub-agents write to the main checkout (not worktrees — worktrees appear later, at parallel implementation launch). Using the Agent tool, one call per card, all in the same message for parallelism. Each sub-agent receives a self-contained brief:

```
You are a design analyst for card <path>. Produce a decision pack.

Your job:
1. Read the card (<path>) and its references
2. Read prior specs in the card's `specs` array (if any) and their progress.md
3. Run a keyword scan on the codebase using terms from the card's goal and scenarios
4. Identify the 4–6 design decisions that this card's implementation requires

For each decision, produce:
  - Title (one line, describes the choice)
  - Context (1–2 sentences — why this decision exists)
  - Options (2–3 concrete alternatives)
  - Trade-offs (what each option gains, what it loses — grounded in evidence from the card, prior specs, or codebase)
  - Recommendation (which option and why, citing the evidence)

Do NOT run interactive Q&A. Do NOT call AskUserQuestion. You produce a written decision pack that the lead agent will present to the author.

Write your decision pack to: <spec_dir>/decisions.md
Do NOT write outside <spec_dir>.

When done, return a JSON object with this shape (and nothing else):
  { "files": ["<spec_dir>/decisions.md", ...any other paths you wrote...] }
```

**Path discipline is trust + post-verify.** The brief names the target directory as a convention the sub-agent is expected to honour. The lead verifies on return via three primitives (§4b). Claude Code does not provide a tool-level path prefix guard, so the lead takes responsibility for the check; the brief takes responsibility for the contract. (ac-03, constraint #1)

**4b. Verify on return — three primitives (snapshot-diff discipline).**

Before launching each design sub-agent, the lead captures a **pre-snapshot** of the main checkout:

```bash
git status --porcelain > /tmp/rally-pre-<card-slug>.snap
```

After the sub-agent returns, the lead runs all three checks:

1. **Self-report (contract):** parse the sub-agent's returned JSON `files` list. If the JSON is missing or malformed, reject.
2. **Artefact assertion (completeness):** assert `<spec_dir>/decisions.md` exists; assert every path in the returned list is under `<spec_dir>`.
3. **Snapshot diff (independent verification):** capture a post-snapshot (`git status --porcelain`) and compute the set difference `post \ pre`. Any entry in that difference that is neither under `<spec_dir>` nor on the fixed lead-owned allowlist (`specs/rally.yaml`) rejects the sub-agent's output. Entries present in both pre and post are pre-existing lead-side state and are ignored. (constraint #9, ac-04)

On the **first** violation: the lead re-briefs the same sub-agent with an explicit path warning naming the offending entry (e.g. `your previous return created 'plugins/orb/scratch.md' outside <spec_dir>; do not write outside <spec_dir>`). This re-brief is a **pre-qualification retry** — it is NOT a rally-level strike and does not count against any drive-full escalation budget.

On the **second** violation for the same card: park the card with `parked_constraint: "sub-agent violated path discipline"`. (ac-04)

**4c. Wait for all sub-agents to return.** Verify each card's `<spec_dir>/decisions.md` exists. If any sub-agent failed the three-primitive check twice, the card is parked and the rally continues with the remainder.

Update rally.yaml (while on main — see §3): each card's status → `designing` (during) → `designed` (on decision pack return). Phase → `design-review` once all packs are in.

### 5. Consolidated Decision Gate

Read all decision packs. Present them to the author **grouped by card**, in a single consolidated response:

```
## Consolidated Decision Gate — <N> cards

### Card: <card feature> (<path>)

#### Decision 1: <title>
Context: <context>
Options:
  A. <option>
  B. <option>
  C. <option>
Trade-offs:
  A: gains <x>, loses <y>
  B: gains <x>, loses <y>
  C: gains <x>, loses <y>
Recommendation: B — <rationale>

#### Decision 2: <title>
...

### Card: <next card>
...
```

**Use AskUserQuestion per card** to capture approvals and overrides. For each card, the author either accepts the recommendations wholesale or names specific overrides (e.g. "card 2 decision 3: option A instead of C"). Record every override explicitly — these flow into the interview.

### 6. Interview Production & Consolidated Design Review

**6a. Produce interview.md per card.** Re-launch each sub-agent (or have the lead agent write directly — whichever is cheaper) with its decisions + the author's approvals/overrides. The sub-agent writes `<spec_dir>/interview.md` following the design skill's interview record format, reflecting the approved decisions. The same three-primitive verification (§4b) applies to any re-launched sub-agent.

**6b. Consolidated design review.** Once all interviews exist, present them to the author in a single session:

```
## Consolidated Design Review — <N> cards

1. Card: <name> — <spec_dir>/interview.md
   Goal: <goal>
   Key decisions: <one-liners>

2. Card: <name> — <spec_dir>/interview.md
   Goal: <goal>
   Key decisions: <one-liners>
```

**6c. Run the definitive disjointness check.** Extract from each interview:
- Files named in the design (e.g. `plugins/orb/skills/foo/SKILL.md`)
- Symbols named (types, traits, functions, schemas)
- Shared references (skills, scripts, hooks)

Compute the intersection. Any non-empty intersection is a hard input to implementation ordering — **it gates, not advises.**

**If shared symbols are found:**

```
Shared symbols detected:
  - Engine trait — referenced by <card A> and <card C>
  - specs/.../hook.sh — both <card B> and <card C> modify it

Proposed implementation order: <card A> → <card C> → <card B>
Rationale: Card A establishes the Engine trait; card C extends it; card B depends on the hook update card C ships.
```

Use AskUserQuestion to confirm or modify the order. Update `implementation_order` and `order_rationale` in rally.yaml (on main).

**If no shared symbols are found:**

```
No shared symbols detected — parallel implementation is safe.
Rationale: Each design names disjoint files and types.
```

Update rally.yaml: `implementation_order: null`, `order_rationale: "Designs are disjoint — <one-line summary>"`. Phase → `implementing`.

**Supervised mode gate:** If autonomy is `supervised`, pause for greenlight before implementation.

### 7. Stage: Implementation

Create a task per card using TaskCreate, with dependencies reflecting `implementation_order`. The task list provides live in-session visibility; rally.yaml provides durable cross-session state — **both are maintained**.

#### 7a. Commit-before-delegation (serial and parallel)

Before delegating any card to drive (serial or parallel), the lead commits the approved `interview.md` to the card's rally branch:

```bash
# on main
git checkout -b rally/<slug>            # or: git checkout rally/<slug>
git add specs/<spec_dir>/interview.md
git commit -m "rally/<slug>: approved design"
```

This gives the rally branch a clean first commit that tells the card's story ("approved design → spec → implementation → tests") in chronological order. Drive-full's §11 file-presence resumption then fires on `interview.md` and starts at the spec stage — no design stage re-entry, no drive contract change. (constraint #10, ac-16)

#### 7b. Serial Implementation (shared symbols)

Cards are implemented in `implementation_order`. Each serial card runs **drive-full against the rally branch in the main checkout** — rally-level autonomy does not reduce drive's internal autonomy. (constraint #11)

For each card N:

1. Lead is on main. Read rally.yaml, set card status → `implementing`, set `worktree: "main"`. Commit rally.yaml on main.
2. `git checkout rally/<slug>` (create from previous non-parked card's branch if not already present, or from main for the first card)
3. Run drive-full inline for this card: spec → review-spec → implement → review-pr, using the already-committed `interview.md`. Drive's §11 file-presence resumption skips design and starts at spec.
4. On NO-GO at any stage (§8), `git checkout main`, park the card in rally.yaml, commit, checkout back or move on.
5. On APPROVE, create the card's PR targeting the previous non-parked card's branch (§8 Stacked PRs)
6. `git checkout main`, mark card status `complete` in rally.yaml, commit. Move to N+1.

Card N+1 begins **only after** card N reaches `complete` or `parked`.

Every rally.yaml write happens on main (constraint #12); every card-level edit happens on the rally branch.

#### 7c. Parallel Implementation (no shared symbols)

Launch N implementation sub-agents concurrently, each in its own git worktree.

**Recursive context separation.** Each parallel sub-agent runs `/orb:drive <card> full` inside its worktree. Drive-full's review-spec and review-pr stages themselves run as nested forked Agents — the same context-separation pattern drive uses at its top level. This means a rally sub-agent (Agent tool, `general-purpose`) spawns its own forked reviewers (Agent tool, `general-purpose`) for review-spec and review-pr. Rally does not invoke reviewers directly; drive does, once per stage per cycle.

The nested-fork contract drive honours inside each sub-agent:

- **Verdict shape.** Drive parses a single canonical line `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK` from the review file — see decision `0004-drive-verdict-contract` (Drive's Verdict Contract: Strict Canonical Markdown Line).
- **Authoritative source.** Drive reads the review file on disk; the forked Agent's chat return is informational — see decision `0005-drive-review-artefact-contract` (Drive's Review Artefact Contract: File-on-Disk Authoritative).
- **Re-review context.** On REQUEST_CHANGES, drive re-forks with a fully cold brief — no pointer to prior review, no iteration counter — see decision `0006-drive-cold-re-review` (Drive's Re-Review Context: Fully Cold).
- **Request-changes budget.** Each review stage has 3 REQUEST_CHANGES cycles per drive iteration; on the 4th would-be cycle drive synthesises BLOCK with the fixed constraint string and enters NO-GO handling — see decision `0007-drive-rerequest-budget` (Drive's REQUEST_CHANGES Budget: 3 Cycles Per Stage).

These four decisions are drive-internal contracts. Rally does not implement or override them — it relies on drive to honour them inside each sub-agent. Rally's only contract with the sub-agent is the final JSON verdict (below). (ac-07)

For each card:

1. Lead is on main. Read rally.yaml, commit interview.md to `rally/<slug>` per §7a.
2. `git checkout main` (return to main before rally.yaml writes).
3. Create the worktree from the rally branch:
   ```bash
   git worktree add ../<repo>-rally-<slug> rally/<slug>
   ```
   Record the absolute path.
4. Write rally.yaml: card status → `implementing`, `worktree: <absolute path>`. Commit on main.

Then, in a single message, spawn all N sub-agents via the Agent tool with `run_in_background: true`:

```
# Sub-agent brief (parallel implementation)

You are an implementation agent for card <path>. Your working directory is
<worktree path>. Run `/orb:drive <card> full` inside that worktree. Drive's
§11 file-presence resumption will detect the already-committed
<spec_dir>/interview.md on this branch and start at the spec stage.

Do NOT read rally.yaml. Do NOT write rally.yaml. rally.yaml is owned by the
lead agent and only exists on main — it is not present on your rally branch.

When drive-full completes (APPROVE at review-pr), return a JSON object:
  { "verdict": "complete", "pr": "<pr-number-or-url>", "spec_dir": "<spec_dir>" }

If drive-full escalates, return:
  { "verdict": "parked", "reason_label": "<label>", "reason": "<one-line>",
    "spec_dir": "<spec_dir>" }

where `reason_label` is one of the five fixed tokens (see §9):
  budget | recurring_failure | contradicted_hypothesis | diminishing_signal | review_converged

Do not attempt rally-level retries — your internal drive iterations are the
strike. (ac-08)
```

The Agent tool is invoked with `run_in_background: true` and `subagent_type: "general-purpose"`; every call is in the same message so the harness dispatches all N in parallel. (ac-06)

**Parallel completion handling — Agent-return await (no polling, no sentinels).**

The lead awaits each sub-agent's completion via the Agent tool's built-in background-completion notification. The harness surfaces the sub-agent's final message as the lead's next turn event — there is no `sleep`, no polling loop, no `Monitor` call, no sentinel file. "Reacts to completion events" means: the notification arrives; the lead handles it. (constraint #5, ac-09)

On each completion:

1. Parse the sub-agent's JSON verdict.
2. `git checkout main` (if not already there).
3. Update rally.yaml for that card: `complete` on APPROVE, `parked` on escalation (with `parked_constraint` constructed per ac-14 once PR #6 is live).
4. `git commit` on main.

N sub-agent completions produce exactly N rally.yaml commits on main, in the order completions are surfaced by the harness.

### 8. Assurance — PR Strategy

#### Stacked PRs (serial)

Each card's PR targets the previous non-parked card's branch:

```
main
 └── rally/card-a         [PR #101 → main]
      └── rally/card-c    [PR #102 → rally/card-a]
           └── rally/card-b   [PR #103 → rally/card-c]
```

**If a middle card is parked**, subsequent PRs target the **last non-parked** card's branch. E.g. if card C is parked in the stack above, card B's PR targets `rally/card-a`, not `rally/card-c`.

Present the stack to the author bottom-up for review.

#### Batched Diff Review (parallel)

Each sub-agent creates an individual PR against main. The lead presents them together:

```
## Rally PR Review — <N> PRs ready

PR #201 — <card A feature>
  Spec: <spec_dir>/spec.yaml (<N> ACs)
  Files changed: <count>
  Review verdict: APPROVE — <one-line honest assessment>

PR #202 — <card B feature>
  ...
```

Author reviews in a single session.

### 9. NO-GO Handling — Single-Strike Park

A NO-GO verdict at **any** stage (spec review BLOCK, supervised gate NO-GO, PR review BLOCK, or drive-full escalation from a parallel sub-agent) parks the card immediately. **No iteration retries within the rally.** Rally is about throughput; retrying one card while others wait defeats the purpose.

```yaml
# rally.yaml update (on main)
cards:
  - path: cards/0016-<slug>.yaml
    status: parked
    spec_dir: specs/YYYY-MM-DD-<slug>/
    branch: rally/<slug>
    worktree: <path-or-'main'-at-time-of-park>
    parked_constraint: "[<label>] <one-line constraint from the NO-GO verdict>"
```

For drive-full escalations inside a parallel sub-agent, rally's single-strike absorbs all five triggers. Drive inside the sub-agent emits a `reason_label` token; rally prepends it in brackets to the escalation reason and writes the combined string to `parked_constraint`. The mapping is fixed: (ac-14)

```
Drive escalation trigger                     reason_label              parked_constraint prefix
---------------------------------------------+-------------------------+-------------------------
Budget exhausted (3 NO-GO iterations)        budget                    [budget]
Recurring failure mode                       recurring_failure         [recurring_failure]
Contradicted hypothesis                      contradicted_hypothesis   [contradicted_hypothesis]
Diminishing signal                           diminishing_signal        [diminishing_signal]
Synthetic BLOCK after 3× REQUEST_CHANGES     review_converged          [review_converged]
  (decision 0007-drive-rerequest-budget)
```

Rally does not retry at its level; the sub-agent's internal iterations (drive's 3-iteration NO-GO budget plus each stage's 3-cycle REQUEST_CHANGES budget) are the strike. An unrecognised or missing `reason_label` in the sub-agent's JSON return parks the card with the literal string `[unknown]` prefixed — the card is still parked, and the label drift becomes visible in rally.yaml for later investigation.

The parked card can be driven individually later with `/orb:drive`, where its full 3-iteration budget applies. The rally continues with remaining cards.

### 10. Completion

When all cards are in `complete` or `parked` status:

1. **Write completion summary:**

   ```
   ## Rally Complete — <goal string>

   Duration: <started> → <completed>
   Autonomy: <level>

   Completed: <N> card(s)
     - <card feature> — PR #<n>
     - <card feature> — PR #<n>

   Parked: <N> card(s)
     - <card feature> — constraint: <parked_constraint>

   Implementation order: <serial order OR "parallel">
   Rationale: <order_rationale>

   PRs:
     - #<n>: <title> (<target branch>)
     - #<n>: <title> (<target branch>)
   ```

2. **Update rally.yaml (on main):** `phase: complete`, `completed: <ISO-8601 timestamp>`.

3. **Do not archive yet.** The file remains at `specs/rally.yaml`. Archival happens when the **next** rally begins (§1 / §11).

### 11. Resumption

When `/orb:rally` is invoked and `specs/rally.yaml` exists:

1. Read and validate rally.yaml on main (§12)
2. Detect the current phase and per-card statuses
3. For each implementing card, resolve its drive.yaml path using the **worktree path-resolution rule** (ac-11):
   - If `worktree == "main"`: resolve as `<main-checkout-root>/specs/<spec_dir>/drive.yaml` (no worktree prefix)
   - If `worktree` is an absolute path: resolve as `<worktree>/specs/<spec_dir>/drive.yaml`
   - If `worktree` is null and status is `implementing`: validation error (§12)
4. Read each resolved drive.yaml to determine per-card sub-stage (spec, review-spec, implement, review-pr)
5. Announce:
   ```
   Resuming rally: <goal>
   Phase: <phase>
   Cards: <N> proposed, <N> complete, <N> parked, <N> in progress
   Per-card resume points:
     - card A (worktree: main)         → at implement
     - card B (worktree: /…/rally-b)   → at review-pr
   ```
6. Resume from each card's correct sub-stage

The path-resolution rule is the only mechanism distinguishing serial (main-checkout) from parallel (worktree) cards at resume time. Implementers must use it; do not concatenate worktree strings naively (e.g. `${worktree}/specs/...` produces `main/specs/...` for serial cards, which fails). (ac-13)

**Completion handling:** If the existing rally's `phase == complete`:
```
Rally "<goal>" completed on <completed>. Start a new rally?

Options:
  - Archive: move specs/rally.yaml → specs/archive/rally-<timestamp>.yaml, then propose new rally
  - Cancel: keep the old rally.yaml and exit
```

On archive, move the file (on main), commit, then restart from §1 with the new goal.

### 12. rally.yaml Validation

On every read, validate rally.yaml before trusting it. Required checks:

- YAML parses cleanly (malformed YAML → clear error naming the line)
- Required top-level fields present: `rally`, `autonomy`, `phase`, `started`, `cards`
- `phase` is one of: `proposing | approved | designing | design-review | implementing | complete`
- `autonomy` is one of: `guided | supervised`
- Each card has `path`, `status`, `spec_dir`, `branch`, `worktree`
- Each card `status` is one of: `proposed | designing | designed | implementing | complete | parked`
- If any card has `status: parked`, `parked_constraint` is non-null
- **Worktree field (ac-12):**
  - If `worktree == "main"` and status is `implementing`: valid (serial-launched card). No filesystem check.
  - If `worktree` is an absolute path and status is `implementing`: valid. If the path does not exist on disk, set an in-memory `worktree_missing` flag on the card (not a validation error). The lead emits ONE user-visible warning naming the card at rally resumption or the next rally.yaml-writing transition, then suppresses subsequent warnings while the worktree remains missing.
  - If `worktree` is `null` and status is `implementing`: validation error — the field is required for launched cards.
  - If `worktree` is present before launch (status is `proposed | designing | designed | design-review`): not an error — may be null or absent, populated at launch.

On validation failure:

```
rally.yaml at specs/rally.yaml is invalid: <specific problem, e.g. "card 2 has status 'implementing' but worktree is null">

Fix the file or remove it to start fresh. Rally will not proceed with corrupt state.
```

Never silently repair. Never half-resume.

## Two-Layer State Model

| Layer | File | Scope | Location |
|-------|------|-------|----------|
| Rally coordination | `specs/rally.yaml` | Per-card coordination status, implementation order, phase, goal, timestamps | **main only** |
| Card sub-stage | `<spec_dir>/drive.yaml` (per card) | Per-card drive stage (spec/review-spec/implement/review-pr), iteration history | Card's worktree, or main for serial cards |

Each implementing card runs drive-full and maintains its own `drive.yaml`. Rally reads drive.yaml on resumption via the ac-11 path-resolution rule to know **which sub-stage** a card is in; rally.yaml on main tells it **which card**.

On resumption, read both layers: rally.yaml on main for coordination, each active card's drive.yaml via the resolver for sub-stage.

## Integration with Other Skills

- **`/orb:drive`** — rally delegates every card to `/orb:drive` in full autonomy. Serial cards run drive-full in the main checkout against the rally branch; parallel cards run drive-full inside an isolated worktree. Rally never duplicates drive's stage logic — it delegates.
- **`/orb:design`** — rally's decision-pack model is a rally-specific adaptation. The standard `/orb:design` skill is for single-card interactive sessions.
- **`/orb:review-spec`** and **`/orb:review-pr`** — each card's drive-full forks these stages into nested Agents (recursive context separation). A drive-full inside a rally sub-agent spawns forked reviewers independent of rally's own context. The verdict contract, review-artefact contract, cold re-review, and request-changes budget are drive's — see decisions `0004-drive-verdict-contract` through `0007-drive-rerequest-budget`. Rally relies on these contracts but does not re-specify them.
- **SessionStart hook (`session-context.sh`)** — surfaces active rally state as primary, with per-card drive states subordinated.

## Critical Rules

- **One active rally at a time.** A new rally on top of an active one is refused.
- **Never skip the proposal gate.** The author qualifies the rally; the agent never launches unprompted.
- **Thin-card guard runs at proposal.** Any candidate card with fewer than 3 scenarios refuses the rally before the proposal is shown. Thickening the card via `/orb:card` or removing it is the only way forward.
- **rally.yaml lives only on main and is written only by the lead.** Every rally.yaml write happens on main; sub-agents never read or write rally.yaml.
- **Sub-agent path discipline is trust + post-verify.** The brief names the target directory; the lead verifies on return via self-report, artefact assertion, and pre-vs-post `git status --porcelain` snapshot diff. Claude Code does not provide tool-level path enforcement, and this skill does not claim it does.
- **Drive autonomy inside a rally is always full.** Rally-level autonomy (guided | supervised) governs rally-phase pauses only.
- **Commit before delegation.** Before any sub-agent or drive is handed a card, the lead commits `interview.md` to the rally branch. Drive-full's §11 resumption then fires cleanly.
- **Sub-agents produce decision packs, not questions.** The decision gate is executive-ready — options + trade-offs + recommendations grounded in evidence.
- **The disjointness check gates ordering.** Any non-empty intersection triggers serial ordering; the author can modify but not skip.
- **Single-strike NO-GO.** A card that fails any review is parked. No retries within the rally. Drive-full escalations inside a parallel sub-agent are absorbed by this rule.
- **Stacked PRs skip parked cards.** Gaps in the stack are handled by targeting the last non-parked branch.
- **Task list and rally.yaml are both maintained.** TaskList for live visibility, rally.yaml for durable state. Neither replaces the other.
- **Validate rally.yaml on every read.** Never silently repair corrupt state.

---

**Next step:** After completion, review all PRs in the order recommended by the assurance strategy (stacked bottom-up or batched together).
