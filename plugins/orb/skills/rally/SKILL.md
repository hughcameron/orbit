---
name: rally
description: Coordinate multiple independent cards through the orbit pipeline as a single multi-card delivery — proposal → queued design decisions → consolidated design review → implementation → stacked or batched review
---

# /orb:rally

Drive **multiple** independent cards through the orbit pipeline as a coordinated rally. The rally skill owns multi-card orchestration; individual card stage execution delegates to drive's stage logic.

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

## Why This Exists

Observed pattern (from parallel design across 3 cards):

- **Improvised output paths** — sub-agents wrote files to invented locations
- **Serial surprise** — designs that looked independent revealed shared trait changes only once designs were written
- **No durable plan** — the session died and the coordination state was lost

Rally resolves each with a declared artefact path, a definitive post-design disjointness check, and `specs/rally.yaml` as durable state.

> **Principle:** The goal of rally is to have the highest quality interactions at ideation and assertion. This means maximum clarity based on the best available evidence. Agent work between gates exists to make the next gate sharper — not just faster.

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

**Present the proposal using AskUserQuestion:**

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

If the author adds a card not in the scan's top-N, include it. If the author removes a card, drop it. Loop on "modify" until the author approves or rejects.

**The proposal gate is the only pre-design independence check.** The agent's scan proposes; the author's approval qualifies. Do not attempt a lightweight heuristic disjointness check — the definitive check happens after designs exist (§6).

### 3. Initialise Rally State

On approval, create `specs/rally.yaml`:

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
    parked_constraint: null
  - path: cards/0016-<slug>.yaml
    status: proposed
    spec_dir: specs/YYYY-MM-DD-<slug>/
    branch: rally/<slug>
    parked_constraint: null
implementation_order: null
order_rationale: null
```

For each card, derive `spec_dir` from today's date plus the card slug, and create the directory. Assign a branch name (`rally/<slug>`) for later stacking.

Update `phase: designing` once sub-agents launch (§4).

**Write rally.yaml at every phase transition and per-card status change** — proposing → approved → designing → design-review → implementing → complete. Writes happen at transitions, not mid-phase.

### 4. Stage: Design — Queued Decision Packs

This is the rally's central innovation. The goal: present the author with executive-ready decisions in a single consolidated gate, with options + trade-offs + recommendations drawn from the best available evidence — not raw questions they lack context to answer.

**4a. Launch N design sub-agents in parallel.**

Using the Agent tool (one call per card, all in the same message for parallelism), send each sub-agent a self-contained brief:

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
Do not write anywhere else. Do not read files outside cards/, specs/, and the source tree.

When done, return a one-line summary: "<N> decisions produced at <spec_dir>/decisions.md"
```

**Constrain sub-agent writes via the tools allow-list.** Configure each sub-agent's tools parameter so Write/Edit is scoped to the card's `spec_dir`. Claude Code's tools allow-list is the native mechanism — do not add a separate hook.

**4b. Wait for all sub-agents to return.** Verify each card's `<spec_dir>/decisions.md` exists. If any sub-agent failed, retry once; on second failure, treat as NO-GO and park the card (§7).

Update rally.yaml: each card's status → `designing` (during) → `designed` (on decision pack return). Phase → `design-review` once all packs are in.

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

**6a. Produce interview.md per card.** Re-launch each sub-agent (or have the lead agent write directly — whichever is cheaper) with its decisions + the author's approvals/overrides. The sub-agent writes `<spec_dir>/interview.md` following the design skill's interview record format, reflecting the approved decisions.

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

Use AskUserQuestion to confirm or modify the order. Update `implementation_order` and `order_rationale` in rally.yaml.

**If no shared symbols are found:**

```
No shared symbols detected — parallel implementation is safe.
Rationale: Each design names disjoint files and types.
```

Update rally.yaml: `implementation_order: null`, `order_rationale: "Designs are disjoint — <one-line summary>"`. Phase → `implementing`.

**Supervised mode gate:** If autonomy is `supervised`, pause for greenlight before implementation.

### 7. Stage: Implementation

Create a task per card using TaskCreate, with dependencies reflecting `implementation_order`. The task list provides live in-session visibility; rally.yaml provides durable cross-session state — **both are maintained**.

#### Serial Implementation (shared symbols)

Cards are implemented in `implementation_order`. For each card N:

1. Mark card status `implementing` in rally.yaml and update TaskList
2. Create branch `rally/<slug>` from the **previous non-parked card's branch** (or main for the first card)
3. Invoke drive's stage logic **inline** for this card: spec → review-spec → implement → review-pr
   - Read `plugins/orb/skills/drive/SKILL.md` and follow §4–§7, but using the card's already-complete interview.md from §6a
   - Do NOT invoke `/orb:drive` as a skill call — follow its instructions directly
4. On NO-GO at any stage (§8 in this skill), park and continue
5. On APPROVE, create the card's PR targeting the previous non-parked card's branch (§8 Stacked PRs)
6. Mark card status `complete`, update TaskList, move to N+1

Card N+1 begins **only after** card N reaches `complete` or `parked`.

#### Parallel Implementation (no shared symbols)

Launch N implementation sub-agents concurrently, each in its own git worktree:

1. For each card, create a worktree at `../<repo>-rally-<slug>/` on branch `rally/<slug>` from main
2. Agent tool, one per card, all in the same message for parallelism
3. Sub-agent brief: "Implement card <path> using the completed design at <spec_dir>/interview.md. Follow drive's stage logic (spec → review-spec → implement → review-pr) inline. Your working directory is <worktree path>."
4. Each sub-agent runs drive's stage logic within its worktree independently
5. On NO-GO, mark the card parked in rally.yaml and let the sub-agent return
6. On completion, each sub-agent produces its own PR against main (§8 Batched Diff Review)

Update TaskList and rally.yaml as sub-agents report status.

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

A NO-GO verdict at **any** stage (spec review BLOCK, supervised gate NO-GO, PR review BLOCK) parks the card immediately. **No iteration retries within the rally.** Rally is about throughput; retrying one card while others wait defeats the purpose.

```yaml
# rally.yaml update
cards:
  - path: cards/0016-<slug>.yaml
    status: parked
    spec_dir: specs/YYYY-MM-DD-<slug>/
    branch: rally/<slug>
    parked_constraint: "<one-line constraint from the NO-GO verdict>"
```

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

2. **Update rally.yaml:** `phase: complete`, `completed: <ISO-8601 timestamp>`.

3. **Do not archive yet.** The file remains at `specs/rally.yaml`. Archival happens when the **next** rally begins (§1 / §11).

### 11. Resumption

When `/orb:rally` is invoked and `specs/rally.yaml` exists:

1. Read and validate rally.yaml (§12)
2. Detect the current phase and per-card statuses
3. Use file-presence detection per card (interview.md, spec.yaml, review-spec-*.md, progress.md, review-pr-*.md) to find the exact resumption point
4. Announce:
   ```
   Resuming rally: <goal>
   Phase: <phase>
   Cards: <N> proposed, <N> complete, <N> parked, <N> in progress
   Resuming at: <stage>
   ```
5. Resume from the correct stage

**Completion handling:** If the existing rally's `phase == complete`:
```
Rally "<goal>" completed on <completed>. Start a new rally?

Options:
  - Archive: move specs/rally.yaml → specs/archive/rally-<timestamp>.yaml, then propose new rally
  - Cancel: keep the old rally.yaml and exit
```

On archive, move the file, then restart from §1 with the new goal.

### 12. rally.yaml Validation

On every read, validate rally.yaml before trusting it. Required checks:

- YAML parses cleanly (malformed YAML → clear error naming the line)
- Required top-level fields present: `rally`, `autonomy`, `phase`, `started`, `cards`
- `phase` is one of: `proposing | approved | designing | design-review | implementing | complete`
- `autonomy` is one of: `guided | supervised`
- Each card has `path`, `status`, `spec_dir`, `branch`
- Each card `status` is one of: `proposed | designing | designed | implementing | complete | parked`
- If any card has `status: parked`, `parked_constraint` is non-null

On validation failure:

```
rally.yaml at specs/rally.yaml is invalid: <specific problem, e.g. "card 2 has status 'parked' but no parked_constraint">

Fix the file or remove it to start fresh. Rally will not proceed with corrupt state.
```

Never silently repair. Never half-resume.

## Two-Layer State Model

| Layer | File | Scope |
|-------|------|-------|
| Rally coordination | `specs/rally.yaml` | Per-card coordination status (proposed/designing/designed/implementing/complete/parked), implementation order, phase, goal, timestamps |
| Card sub-stage | `<spec_dir>/drive.yaml` (per card) | Per-card drive stage (spec/review-spec/implement/review/complete), iteration history |

Each implementing card runs drive's stage logic and maintains its own `drive.yaml`. Rally reads the drive.yaml when resuming to know **which sub-stage** a card is in; rally.yaml tells it **which card**.

On resumption, read both layers: rally.yaml for coordination, each active card's drive.yaml for sub-stage.

## Integration with Other Skills

- **`/orb:drive`** — rally delegates to drive's stage logic for each card. Do not invoke `/orb:drive` as a skill call inside rally; follow drive's SKILL.md instructions inline.
- **`/orb:design`** — rally's decision-pack model is a rally-specific adaptation. The standard `/orb:design` skill is for single-card interactive sessions.
- **`/orb:review-spec`** and **`/orb:review-pr`** — run inline per card as part of each card's drive stage logic.
- **SessionStart hook (`session-context.sh`)** — surfaces active rally state as primary, with per-card drive states subordinated.

## Critical Rules

- **One active rally at a time.** A new rally on top of an active one is refused.
- **Never skip the proposal gate.** The author qualifies the rally; the agent never launches unprompted.
- **rally.yaml is written at phase transitions, not mid-phase.** Atomic updates only.
- **Sub-agents produce decision packs, not questions.** The decision gate is executive-ready — options + trade-offs + recommendations grounded in evidence.
- **The disjointness check gates ordering.** Any non-empty intersection triggers serial ordering; the author can modify but not skip.
- **Single-strike NO-GO.** A card that fails any review is parked. No retries within the rally.
- **Stacked PRs skip parked cards.** Gaps in the stack are handled by targeting the last non-parked branch.
- **Task list and rally.yaml are both maintained.** TaskList for live visibility, rally.yaml for durable state. Neither replaces the other.
- **Validate rally.yaml on every read.** Never silently repair corrupt state.

---

**Next step:** After completion, review all PRs in the order recommended by the assurance strategy (stacked bottom-up or batched together).
