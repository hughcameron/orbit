---
name: rally
description: Coordinate multiple independent cards through the orbit pipeline as a single multi-card delivery — proposal → queued design decisions → consolidated design review → implementation → stacked or batched review. State lives in beads; the dependency graph IS the rally.
---

# /orb:rally

Drive **multiple** independent cards through the orbit pipeline as a
coordinated rally. Rally creates an **epic bead** at proposal-approval
time and links each card-bead under it as a child. The dependency
graph between child beads encodes the implementation order. `bd ready
--type task --parent <epic>` is the queue. Individual cards run
`/orb:drive <bead-id>` in full autonomy (either serially in the main
checkout or in parallel inside isolated worktrees).

Rally exists because serial `/orb:drive` invocations accumulate human
touchpoints — every card pauses the author for design, spec, review,
and PR. A rally packs the human work into two high-signal gates
(ideation and assurance) and lets the agent work between them with
maximum clarity based on the best available evidence.

## Migration Note

The previous `/orb:rally` orchestrated multi-card delivery via
`rally.yaml` (per-rally durable state on main) and `drive.yaml`
(per-card sub-stage state inside each worktree). Resumption used a
two-layer scan with a worktree path-resolution rule. Implementation
visibility used `TaskCreate`. Park reasons were stored in a
`parked_constraint` field on each card row.

All of that is now subsumed by beads:

- `rally.yaml` durable state → epic bead + child task beads + epic
  metadata fields (`rally_phase`, `rally_autonomy`, `rally_started`)
- Per-card status row → child bead `status` + child metadata
  (`rally_card_phase`, `rally_branch`, `rally_worktree`,
  `rally_spec_dir`)
- Implementation order → `bd dep add <later> <earlier>` edges between
  child beads (graph topology IS the order)
- In-session visibility → `bd ready --type task --parent <epic>` (no
  `TaskCreate`)
- Resumption scan → `bd list --type epic --status in_progress` with
  rally_phase metadata filter
- `parked_constraint` field → embedded in `bd close --reason "PARKED:
  [<label>] <reason>"`
- Two-layer state model → one layer: the bead graph
- §11 worktree path-resolution rule → removed; drive resumes from bead
  metadata, not drive.yaml

There is no auto-migration. In-flight rally.yaml rallies finish under
the prior version or restart.

## Usage

```
/orb:rally <goal_string> [guided|supervised]   # fresh rally from a goal
/orb:rally <epic-id>                            # resume an existing rally epic
/orb:rally                                      # resume the unique in-progress rally epic, if any
```

- `goal_string` — a short description of the subsystem, theme, or
  objective binding the cards together (e.g. `"pipeline runtime
  readiness"`, `"review workflow hardening"`)
- Autonomy defaults to **guided** if omitted

### Autonomy Levels

| Level | Behaviour |
|-------|-----------|
| **guided** | Proposal and consolidated decision gate are interactive. Reviews serve as quality gates — no intermediate supervision between design and implementation. Default. |
| **supervised** | Same as guided plus explicit pauses after each rally phase (design, review, implementation, PR) for author greenlight. |

`full` autonomy is **not offered** — rally's value comes from sharper
human gates, not fewer.

**Rally-level vs drive-level autonomy.** Rally-level autonomy
(guided | supervised) governs pauses between rally phases — proposal,
consolidated decision gate, consolidated design review, batched diff
review. Drive-level autonomy inside a rally is **always full**, both
for parallel sub-agents running in worktrees and for serial cards
running in the main checkout.

## Input contract

The skill operates on exactly one rally epic per session. Resolution
proceeds in three branches:

1. **Goal string provided** (`/orb:rally <goal_string> [autonomy]`).
   Run §pre-flight (scan-for-active-rally + thin-card guard), then
   §Stage 1 (Proposal).

2. **Epic-id provided** (`/orb:rally <epic-id>`). Validate that the
   epic has `rally_phase` metadata set and not equal to `complete`;
   if not, halt and instruct the agent to start a fresh rally.
   Otherwise, resume from the phase named in `rally_phase`.

3. **No argument** — query for in-progress rally epics:

   ```bash
   bd list --type epic --status in_progress --json \
     | python3 -c "import sys,json; print('\n'.join(b['id'] for b in json.load(sys.stdin) if b.get('metadata',{}).get('rally_phase') and b['metadata']['rally_phase'] != 'complete'))"
   ```

   - **Single match** → resume it.
   - **Zero matches** → halt with usage (a goal string is required to
     start a fresh rally).
   - **Multiple matches** → halt and instruct the agent to pass the
     epic id explicitly, listing the candidates.

## Pre-flight

### 1. Scan for an active rally

Before launching a fresh rally, the input-contract resolution above
already ensures no in-progress rally epic exists. The skill never
launches a second rally over a first. If a stale rally is the
problem, close it explicitly via `bd close <epic-id> --reason "..."`
or set `rally_phase=complete` on its metadata before starting a
fresh rally.

**One active rally at a time** is non-negotiable. Bead-graph
orchestration loses its meaning if two epics overlap on cards.

### 2. Thin-card guard (refuse at proposal)

Before the proposal is presented to the author, check the scenario
count on every candidate card. If any candidate has fewer than
**3 scenarios**, the proposal refuses to proceed:

```
Rally cannot proceed — the following candidate card is too thin:

  orbit/cards/0017-<slug>.yaml — 2 scenarios

Thicken this card via `/orb:card orbit/cards/0017-<slug>.yaml` or remove it from
the rally list before continuing.
```

The author may then:

- Run `/orb:card` on the thin card to thicken it, re-invoke rally
- Remove the thin card from the list and re-invoke rally
- Run the thin card individually via `/orb:drive <card> guided` or
  `supervised` (rally is not the venue for thin cards)

The thin-card refusal is **unconditional on the eventual serial-or-
parallel outcome**. The guard runs before the proposal is shown and
before the post-design disjointness check (§Stage 4) — it is a
pre-qualification gate, not a runtime decision. **No silent
downgrade.**

## Stage 1: Proposal

Parse the goal string from `$ARGUMENTS[0]` and autonomy from
`$ARGUMENTS[1]` (default `guided`).

**Scan `orbit/cards/` for candidate cards:**

1. Read every `orbit/cards/*.yaml` (ignore `orbit/cards/memos/`)
2. For each card, score relevance to the goal string using the card's
   `feature`, `goal`, `scenarios`, and `references`
3. Surface the top candidates (usually 3–6) with a one-line rationale
   per card

Run the §thin-card guard against the candidate list before showing
anything to the author. If any candidate is thin, halt per the guard.

### Present the proposal using AskUserQuestion

The proposal surface has two strict halves: a **markdown preview
block** that carries the evidence (per-card rationale), and an
**AskUserQuestion** that carries the decision (three canonical, terse
options). They are not collapsed — the preview block scales with N
cards while the AskUserQuestion stays short and action-focused.

**Preview block (markdown, above the AskUserQuestion) — owns per-card
rationale:**

```
## Rally Proposal — <goal string>

Candidate cards:
  1. orbit/cards/<id>-<slug>.yaml — <feature line>
     Rationale: <why this card fits the goal>
  2. orbit/cards/<id>-<slug>.yaml — <feature line>
     Rationale: <why this card fits the goal>
  ...

Autonomy: <guided|supervised>
```

**AskUserQuestion — owns the decision.** Exactly three canonical
options in this order. The `description` field for each option is a
one-line **action summary** — it describes the action, not the cards.
Per-card rationale must not appear in these descriptions (the preview
block already owns it).

- **`approve-all`** — `Proceed with all N candidates`
- **`modify-list`** — `Add or remove cards before proceeding`
- **`decline`** — `Abort the rally; offer individual drive as alternative`

**On `approve-all`:** proceed to §Create the epic (below).

**On `decline`:** abort the rally and offer individual `/orb:drive`
invocations as the alternative.

**On `modify-list`:** the lead issues exactly one follow-up
AskUserQuestion with **no pre-populated options** (free-form only).
The prompt text reads:

> *Name cards to add (by path, e.g. `orbit/cards/0019-foo.yaml`) or
> remove (by number, e.g. `2`). Empty response cancels the modification
> and returns to the approval prompt.*

An empty response cancels the modification and re-presents the
unchanged candidate list with the same three canonical options. A
non-empty response is interpreted as modification instructions.

**Modify loop — sequence per iteration:**

1. **Apply** the requested additions and removals to the candidate
   list.
2. **Re-run the §thin-card guard** against the revised list. The
   guard's rules live in §pre-flight and are not restated here; only
   the re-run behaviour is named.
3. **Re-present** the revised preview block plus the AskUserQuestion
   with the same three canonical labels.

**Invariant:** no candidate list is shown to the author unless it has
passed the thin-card guard in the current loop iteration. The author
never decides against a list that cannot fly.

Guard re-runs inside the modify loop are **pre-qualification retries**
— they are not rally-level strikes and do not count against any
escalation budget.

The loop continues — verdict → (modify instructions → apply →
re-guard → re-present) → verdict → … — until the author returns
`approve-all` or `decline`.

**The proposal gate is the only pre-design independence check.** The
agent's scan proposes; the author's approval qualifies. Do not attempt
a lightweight heuristic disjointness check — the definitive check
happens after designs exist (§Stage 4).

### Create the epic and child beads

On `approve-all`:

```bash
# 1. Derive a rally slug from the goal string
SLUG=$(echo "<goal string>" | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9]\+/-/g; s/^-//; s/-$//' | cut -c1-40)

# 2. Create the rally folder for design artefacts (decisions.md, interview.md)
RALLY_DIR="orbit/specs/$(date -I)-${SLUG}-rally"
mkdir -p "$RALLY_DIR"

# 3. Create the epic bead
EPIC=$(bd create "<goal string>" -t epic -p 1 \
  --description "$(cat <<EOF
Rally: <goal string>
Cards: <comma-separated card paths>
Autonomy: <guided|supervised>
Folder: $RALLY_DIR
EOF
)" --silent)

# 4. Seed epic metadata
bd update "$EPIC" \
  --set-metadata "rally_phase=approved" \
  --set-metadata "rally_autonomy=<guided|supervised>" \
  --set-metadata "rally_started=$(date -Iseconds)" \
  --set-metadata "rally_folder=$RALLY_DIR"

# 5. Promote each card and link as child
for CARD in "${APPROVED_CARDS[@]}"; do
  CARD_SLUG=$(basename "$CARD" .yaml)
  SPEC_DIR="orbit/specs/$(date -I)-${CARD_SLUG#*-}"
  mkdir -p "$SPEC_DIR"
  CHILD=$(plugins/orb/scripts/promote.sh "$CARD")
  bd update "$CHILD" \
    --parent "$EPIC" \
    --set-metadata "rally_card_phase=proposed" \
    --set-metadata "rally_branch=rally/${CARD_SLUG#*-}" \
    --set-metadata "rally_spec_dir=$SPEC_DIR"
done
```

The epic's `rally_phase` advances through: `approved` → `designing`
→ `design-review` → `implementing` → `complete`. Each transition is
a single `bd update <epic> --set-metadata "rally_phase=<next>"` write.

`bd children <epic>` lists every card-bead in the rally; `bd ready
--type task --parent <epic>` (post-disjointness wiring) lists the
next claimable cards respecting dependency edges.

## Stage 2: Decision packs — queued design

This is the rally's central innovation. The goal: present the author
with executive-ready decisions in a single consolidated gate, with
options + trade-offs + recommendations drawn from the best available
evidence — not raw questions they lack context to answer.

> **Principle:** The goal of rally is to have the highest quality
> interactions at ideation and assertion. This means maximum clarity
> based on the best available evidence. Agent work between gates
> exists to make the next gate sharper — not just faster.

### 2a. Set the phase

```bash
bd update "$EPIC" --set-metadata "rally_phase=designing"
for CHILD in $(bd children "$EPIC" --json | python3 -c "import sys,json; print(' '.join(b['id'] for b in json.load(sys.stdin)))"); do
  bd update "$CHILD" --set-metadata "rally_card_phase=designing"
done
```

### 2b. Launch N design sub-agents in parallel

Design sub-agents write to the main checkout. Using the Agent tool,
one call per card, all in the same message for parallelism. Each
sub-agent receives a self-contained brief:

```
You are a design analyst for card <card_path> (bead <child-bead-id>).
Produce a decision pack.

Read your bead's metadata first to learn your <spec_dir>:
  bd show <child-bead-id> --json   # extract metadata.rally_spec_dir

Your job:
1. Read the card (<card_path>) and its references
2. Read prior specs in the card's `specs` array (if any)
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

Do NOT read or write any rally-coordination state. The lead owns the rally epic and child metadata exclusively.

When done, return a JSON object with this shape (and nothing else):
  { "files": ["<spec_dir>/decisions.md", ...any other paths you wrote...] }
```

**Path discipline is trust + post-verify.** The brief names the target
directory as a convention the sub-agent is expected to honour. The
lead verifies on return via three primitives (§2c). Claude Code does
not provide a tool-level path prefix guard, so the lead takes
responsibility for the check; the brief takes responsibility for the
contract.

### 2c. Verify on return — three primitives (snapshot-diff discipline)

Before launching each design sub-agent, the lead captures a
**pre-snapshot** of the main checkout:

```bash
git status --porcelain > /tmp/rally-pre-<card-slug>.snap
```

After the sub-agent returns, the lead runs all three checks:

1. **Self-report (contract):** parse the sub-agent's returned JSON
   `files` list. If the JSON is missing or malformed, reject.
2. **Artefact assertion (completeness):** assert
   `<spec_dir>/decisions.md` exists; assert every path in the returned
   list is under `<spec_dir>`.
3. **Snapshot diff (independent verification):** capture a
   post-snapshot (`git status --porcelain`) and compute the set
   difference `post \ pre`. Any entry in that difference that is not
   under `<spec_dir>` rejects the sub-agent's output. There is no
   lead-owned allowlist beyond the spec dir — rally state lives in
   beads, not on disk. Entries present in both pre and post are
   pre-existing lead-side state and are ignored.

On the **first** violation: the lead re-briefs the same sub-agent with
an explicit path warning naming the offending entry (e.g. `your
previous return created 'plugins/orb/scratch.md' outside <spec_dir>;
do not write outside <spec_dir>`). This re-brief is a
**pre-qualification retry** — it is NOT a rally-level strike and does
not count against any drive-full escalation budget.

On the **second** violation for the same card: park the card via:

```bash
bd close <child-bead> --reason "PARKED: [tool_surface_incomplete] sub-agent violated path discipline twice"
```

The rally continues with the remainder.

### 2d. Wait for all sub-agents to return

Verify each card's `<spec_dir>/decisions.md` exists. If any sub-agent
failed the three-primitive check twice, the card is parked and the
rally continues with the remainder.

For each non-parked child:

```bash
bd update <child> --set-metadata "rally_card_phase=designed"
```

Once all packs are in (or parked), advance the epic:

```bash
bd update "$EPIC" --set-metadata "rally_phase=design-review"
```

## Stage 3: Consolidated decision gate

Read all decision packs. Present them to the author **grouped by
card**, in a single consolidated response:

```
## Consolidated Decision Gate — <N> cards

### Card: <card feature> (<card_path>)
Bead: <child-bead-id>

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

**Use AskUserQuestion per card** to capture approvals and overrides.
For each card, the author either accepts the recommendations
wholesale or names specific overrides (e.g. "card 2 decision 3:
option A instead of C"). Record every override explicitly — these
flow into the interview.

### Interview production

Re-launch each sub-agent (or have the lead agent write directly —
whichever is cheaper) with its decisions + the author's
approvals/overrides. The sub-agent writes `<spec_dir>/interview.md`
following the design skill's interview record format, reflecting the
approved decisions.

The same three-primitive verification (§2c) applies to any
re-launched sub-agent. First violation → re-brief retry. Second
violation → park via `bd close --reason "PARKED:
[tool_surface_incomplete] ..."`.

When all interviews exist, the rally moves on to Stage 4.

## Stage 4: Consolidated design review + disjointness wiring

Once all interviews exist, present them to the author in a single
session:

```
## Consolidated Design Review — <N> cards

1. Card: <name> — <spec_dir>/interview.md (bead: <child-bead-id>)
   Goal: <goal>
   Key decisions: <one-liners>

2. Card: <name> — <spec_dir>/interview.md (bead: <child-bead-id>)
   Goal: <goal>
   Key decisions: <one-liners>
```

### Run the definitive disjointness check

Extract from each interview:
- Files named in the design (e.g. `plugins/orb/skills/foo/SKILL.md`)
- Symbols named (types, traits, functions, schemas)
- Shared references (skills, scripts, hooks)

Compute the intersection. Any non-empty intersection is a hard input
to implementation ordering — **it gates, not advises.**

### If shared symbols are found — wire dep edges

```
Shared symbols detected:
  - Engine trait — referenced by <card A> and <card C>
  - orbit/specs/.../hook.sh — both <card B> and <card C> modify it

Proposed implementation order: <card A> → <card C> → <card B>
Rationale: Card A establishes the Engine trait; card C extends it; card B depends on the hook update card C ships.
```

Use AskUserQuestion to confirm or modify the order. On confirm, encode
the order via `bd dep add` between child beads — for each ordered pair
`(earlier, later)`:

```bash
bd dep add <later-bead> <earlier-bead>
```

`bd ready --type task --parent <epic>` will then surface only the head
of the chain at any moment, releasing the next card as each predecessor
closes.

### If no shared symbols are found — leave parallel

```
No shared symbols detected — parallel implementation is safe.
Rationale: Each design names disjoint files and types.
```

No `bd dep add` calls. `bd ready --type task --parent <epic>` will
surface all cards simultaneously, ready for parallel claim.

### Advance to implementation

```bash
bd update "$EPIC" --set-metadata "rally_phase=implementing"
```

**Supervised mode gate:** If autonomy is `supervised`, pause for
greenlight before proceeding to implementation.

## Stage 5: Implementation

The implementation queue is `bd ready --type task --parent <epic>`.
The shape of the queue (single-head chain vs flat fan-out) is
determined entirely by the dep edges from §Stage 4. Rally does not
reproduce queue logic — it consumes the bead query.

### 5a. Commit interviews to rally branches

For each non-parked child, commit `<spec_dir>/interview.md` to the
card's rally branch as a clean first commit. This is git hygiene —
the rally branch tells the card's story in chronological order — and
is independent of how `/orb:drive` resumes.

```bash
# on main
git checkout -b rally/<slug>            # or: git checkout rally/<slug>
git add <spec_dir>/interview.md
git commit -m "rally/<slug>: approved design"
git checkout main
```

### 5b. Serial implementation (chain wired by dep edges)

When dep edges exist, `bd ready --type task --parent <epic>` returns
exactly one card at a time. The lead loops:

```bash
while true; do
  NEXT=$(bd ready --type task --parent "$EPIC" --json \
    | python3 -c "import sys,json; d=json.load(sys.stdin); print(d[0]['id'] if d else '')")
  [ -z "$NEXT" ] && break
  bd update "$NEXT" --set-metadata "rally_card_phase=implementing" --set-metadata "rally_worktree=main"
  git checkout "$(bd show "$NEXT" --json | python3 -c "import sys,json; print(json.load(sys.stdin)['metadata']['rally_branch'])")"
  /orb:drive "$NEXT"     # drive resumes from drive_stage metadata
  # On APPROVE at review-pr, drive closes the child via bd close --reason "drive completed: ..."
  # On NO-GO, drive closes via bd close --reason "NO-GO: ..." — handled by §NO-GO Handling below
  git checkout main
done
```

Each serial card runs **drive-full against the rally branch in the
main checkout** — rally-level autonomy does not reduce drive's
internal autonomy. The next card is released by `bd ready` only when
its predecessor closes.

### 5c. Parallel implementation (no dep edges)

When no dep edges exist, `bd ready --type task --parent <epic>` returns
all cards. Launch N implementation sub-agents concurrently, each in
its own git worktree.

For each card:

```bash
SLUG=$(bd show <child> --json | python3 -c "import sys,json; print(json.load(sys.stdin)['metadata']['rally_branch'].split('/')[-1])")
WORKTREE_PATH="$(realpath ..)/$(basename "$(pwd)")-rally-$SLUG"
git worktree add "$WORKTREE_PATH" "rally/$SLUG"
bd update <child> --set-metadata "rally_card_phase=implementing" --set-metadata "rally_worktree=$WORKTREE_PATH"
```

Then, in a single message, spawn all N sub-agents via the Agent tool
with `run_in_background: true`:

```
# Sub-agent brief (parallel implementation)

You are an implementation agent for bead <child-bead-id>. Your working
directory is <worktree path>. Run `/orb:drive <child-bead-id>` inside
that worktree. Drive will:
  1. Resume from the bead's drive_stage metadata (or initialise it if absent)
  2. Run review-spec → implement → review-pr internally as forked Agents
  3. Close the bead with bd close --reason "drive completed: ..." on APPROVE
  4. Or escalate if iteration / review budgets exhaust

Do NOT read or write rally-coordination state. The lead owns the rally
epic and per-child rally_* metadata exclusively. You may read your own
bead via `bd show <child-bead-id> --json` and update your own
drive_stage / drive_review_*_cycle metadata as drive normally does.

When drive completes (APPROVE at review-pr), return a JSON object:
  { "verdict": "complete", "pr": "<pr-number-or-url>", "spec_dir": "<spec_dir>" }

If drive escalates, return:
  { "verdict": "parked", "reason_label": "<label>", "reason": "<one-line>",
    "spec_dir": "<spec_dir>" }

where `reason_label` is one of the six fixed tokens (see §NO-GO):
  budget | recurring_failure | contradicted_hypothesis | diminishing_signal | review_converged | tool_surface_incomplete

Do not attempt rally-level retries — your internal drive iterations are the
strike.
```

The Agent tool is invoked with `run_in_background: true` and
`subagent_type: "general-purpose"`; every call is in the same message
so the harness dispatches all N in parallel.

**Recursive context separation.** Each parallel sub-agent runs
`/orb:drive` inside its worktree. Drive's review-spec and review-pr
stages themselves run as nested forked Agents — the same context-
separation pattern drive uses at its top level. Rally does not invoke
reviewers directly; drive does, once per stage per cycle.

**Parallel completion handling — Agent-return await (no polling, no
sentinels).** The lead awaits each sub-agent's completion via the
Agent tool's built-in background-completion notification. The harness
surfaces the sub-agent's final message as the lead's next turn event
— no `sleep`, no polling loop, no `Monitor` call, no sentinel file.

On each completion:

1. Parse the sub-agent's JSON verdict.
2. On `complete`: the child bead is already closed by drive. Update
   `rally_card_phase=complete` (idempotent — no-op if already set).
3. On `parked`: handle per §NO-GO Handling.

### 5d. Mid-flight parallel→serial conversion

If parallel implementation surfaces a shared-symbol contention
mid-flight (e.g. two in-progress sub-agents about to touch the same
file), the lead serializes by adding a single dep edge:

```bash
bd dep add <later-bead> <earlier-bead>
```

`bd ready` will then withhold `<later-bead>` until `<earlier-bead>`
closes. **In-progress work continues.** The runtime change is the
queue, not the running cards. No sub-agent restart, no rally-coord
mutation, no special "convert to serial" mode — a single bd dep add
is the operation.

If a sub-agent has not yet started (still queued by the harness),
adding the edge prevents it from starting. If the sub-agent has
already started, the lead may either (a) let both finish if the
contention is mild, or (b) ask the later sub-agent to halt and
re-claim once its predecessor closes (sub-agent honours the request
via its own halt path).

## NO-GO Handling — single-strike park

A NO-GO verdict at **any** stage (drive's spec review BLOCK,
supervised gate NO-GO, drive's PR review BLOCK, or drive-full
escalation from a parallel sub-agent) parks the card immediately.
**No iteration retries within the rally.** Rally is about throughput;
retrying one card while others wait defeats the purpose.

Drive escalations inside a sub-agent surface as a JSON return:

```json
{ "verdict": "parked", "reason_label": "<label>", "reason": "<one-line>",
  "spec_dir": "<spec_dir>" }
```

Rally lead converts that into a single `bd close` invocation:

```bash
bd close <child-bead> --reason "PARKED: [<reason_label>] <reason>"
bd update <child-bead> --set-metadata "rally_card_phase=parked"
```

The reason_label vocabulary is preserved (six fixed tokens):

```
Drive escalation trigger                     reason_label
---------------------------------------------+----------------------
Budget exhausted (3 NO-GO iterations)        budget
Recurring failure mode                       recurring_failure
Contradicted hypothesis                      contradicted_hypothesis
Diminishing signal                           diminishing_signal
Synthetic BLOCK after 3× REQUEST_CHANGES     review_converged
Agent tool unavailable for cold-fork         tool_surface_incomplete
```

An unrecognised or missing `reason_label` in the sub-agent's JSON
return parks the card with the literal string `[unknown]` prefixed —
the card is still parked, and the label drift becomes visible in the
bead's close --reason for later investigation.

Rally does not retry at its level; the sub-agent's internal iterations
(drive's 3-iteration NO-GO budget plus each stage's 3-cycle
REQUEST_CHANGES budget) are the strike.

The parked card can be driven individually later with `/orb:drive
<card_path>`, where its full 3-iteration budget applies (a fresh
drive starts a new bead chain). The rally continues with remaining
cards.

## Stage 6: Assurance — PR strategy

### Stacked PRs (serial — dep edges exist)

Each card's PR targets the previous non-parked card's branch:

```
main
 └── rally/card-a         [PR #101 → main]
      └── rally/card-c    [PR #102 → rally/card-a]
           └── rally/card-b   [PR #103 → rally/card-c]
```

**If a middle card is parked**, subsequent PRs target the **last
non-parked** card's branch. E.g. if card C is parked in the stack
above, card B's PR targets `rally/card-a`, not `rally/card-c`.

The lead computes "last non-parked predecessor" by walking the dep
chain (via `bd dep tree <head-bead>`) and skipping any child whose
status is closed with a `PARKED:` reason.

Present the stack to the author bottom-up for review.

### Batched diff review (parallel — no dep edges)

Each sub-agent creates an individual PR against main. The lead
presents them together:

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

## Stage 7: Completion

When `bd children <epic-id>` shows every child bead as closed
(regardless of close reason — `drive completed: ...` for success,
`PARKED: ...` for park):

1. **Write completion summary:**

   ```
   ## Rally Complete — <goal string>

   Duration: <rally_started> → <now>
   Autonomy: <rally_autonomy>

   Completed: <N> card(s)
     - <card feature> — PR #<n>
     - <card feature> — PR #<n>

   Parked: <N> card(s)
     - <card feature> — close reason: PARKED: [<label>] <reason>

   Implementation order: <serial chain order OR "parallel">
   Rationale: <derived from bd dep tree, or "no shared symbols" for parallel>

   PRs:
     - #<n>: <title> (<target branch>)
     - #<n>: <title> (<target branch>)
   ```

2. **Update epic metadata + close the epic:**

   ```bash
   bd update "$EPIC" \
     --set-metadata "rally_phase=complete" \
     --set-metadata "rally_completed=$(date -Iseconds)"
   bd close "$EPIC" --reason "rally complete: <one-line summary>"
   ```

   `bd epic close-eligible` is also acceptable as a batch convenience
   when multiple completed epics are pending close.

3. **No archival step.** The rally folder (`<rally-folder>`) stays
   where it is — its decisions.md and interview.md files remain on
   disk as the design record. The epic bead remains in the bead
   database as the orchestration record. When the next rally begins,
   it creates its own folder alongside this one.

## Resumption

When `/orb:rally` is invoked with an epic-id (or detects an
in-progress rally epic per §Input contract):

1. **Read the epic:** `bd show <epic-id> --json`. Extract:
   - `metadata.rally_phase`
   - `metadata.rally_autonomy`
   - `metadata.rally_started`
   - `metadata.rally_folder`

2. **Read all children:** `bd children <epic-id> --json`. For each:
   - Bead status (open / in_progress / closed)
   - `metadata.rally_card_phase`
   - `metadata.rally_branch`, `rally_worktree`, `rally_spec_dir`

3. **Resume at the named phase.** The epic + children are the source
   of truth — there is no on-disk state to scan or reconcile.

   | rally_phase     | Resume at                                        |
   |-----------------|--------------------------------------------------|
   | `approved`      | §Stage 2 (decision packs not yet launched)       |
   | `designing`     | §Stage 2 (some decision packs may be returned)   |
   | `design-review` | §Stage 3 (decision gate / interview production)  |
   | `implementing`  | §Stage 5 (queue replay via bd ready)             |
   | `complete`      | Already done — report status                     |

4. **For implementing-phase resume:** the lead does not reconstruct
   per-card sub-stage from any side file — each implementing child
   resumes via `/orb:drive <child-bead-id>`
   which itself reads `drive_stage` metadata. Rally's job at resume
   is only to (a) re-launch sub-agents for any card whose
   `rally_card_phase=implementing` and bead status is in_progress
   (or open if its turn has come up), (b) honour the dep-edge queue
   via `bd ready --type task --parent <epic>` for serial flows, and
   (c) await completions for any sub-agent that was running before
   the session died.

5. **Announce the resumption:**

   ```
   Resuming rally: <goal>
   Epic: <epic-id>
   Phase: <rally_phase>
   Cards: <N> proposed, <N> complete, <N> parked, <N> in progress
   Per-card resume points (from bd children):
     - card A (bead <id>, worktree: main)         → in implement
     - card B (bead <id>, worktree: /…/rally-b)   → in review-pr
   ```

## Critical Rules

- **One active rally at a time.** Resolution refuses a fresh rally if
  an in-progress epic exists.
- **Never skip the proposal gate.** The author qualifies the rally;
  the agent never launches unprompted.
- **Thin-card guard runs at proposal.** Any candidate card with fewer
  than 3 scenarios refuses the rally before the proposal is shown.
  Thickening the card via `/orb:card` or removing it is the only way
  forward.
- **The bead graph is the single source of orchestration state.** No
  rally-coordination data lives on disk. Sub-agents never read or
  write rally state — they read their own bead and update their own
  drive metadata.
- **Sub-agent path discipline is trust + post-verify.** The brief
  names the target directory; the lead verifies on return via
  self-report, artefact assertion, and pre-vs-post `git status
  --porcelain` snapshot diff. Claude Code does not provide tool-level
  path enforcement, and this skill does not claim it does.
- **Drive autonomy inside a rally is always full.** Rally-level
  autonomy (guided | supervised) governs rally-phase pauses only.
- **The disjointness check gates ordering.** Any non-empty
  intersection triggers `bd dep add` edges; the author can modify but
  not skip.
- **Mid-flight serial conversion is a single bd dep add.** No special
  mode, no sub-agent restart, no rally-coord mutation.
- **Single-strike NO-GO.** A card that fails any review is parked via
  `bd close --reason "PARKED: [<label>] <reason>"`. No retries within
  the rally.
- **Stacked PRs skip parked cards.** Gaps in the stack are handled by
  targeting the last non-parked branch.
- **bd ready --type task --parent is the queue.** Rally does not
  maintain a parallel in-session task list — the bead query provides
  live visibility on demand.

## Worked example

A copy-pasteable trace for a four-card rally that runs partly in
parallel and partly serial. Each step is a literal command.

```bash
# 1. Validate candidates (thin-card guard) — refuse if any has <3 scenarios
for CARD in orbit/cards/0021-foo.yaml orbit/cards/0022-bar.yaml \
            orbit/cards/0023-baz.yaml orbit/cards/0024-quux.yaml; do
  python3 -c "
import yaml
with open('$CARD') as f:
    n = len(yaml.safe_load(f).get('scenarios', []))
assert n >= 3, f'BLOCKED: $CARD has {n} scenarios; rally requires ≥3'
"
done

# 2. Create rally folder
RALLY_DIR="orbit/specs/$(date -I)-pipeline-readiness-rally"
mkdir -p "$RALLY_DIR"

# 3. Create the epic + seed metadata (after AskUserQuestion approve-all)
EPIC=$(bd create "pipeline runtime readiness" -t epic -p 1 \
  --description "Rally: pipeline runtime readiness
Cards: 0021-foo, 0022-bar, 0023-baz, 0024-quux
Autonomy: guided
Folder: $RALLY_DIR" --silent)
bd update "$EPIC" \
  --set-metadata "rally_phase=approved" \
  --set-metadata "rally_autonomy=guided" \
  --set-metadata "rally_started=$(date -Iseconds)" \
  --set-metadata "rally_folder=$RALLY_DIR"

# 4. Promote cards + link as children
for CARD in orbit/cards/0021-foo.yaml orbit/cards/0022-bar.yaml \
            orbit/cards/0023-baz.yaml orbit/cards/0024-quux.yaml; do
  CARD_SLUG=$(basename "$CARD" .yaml)
  SPEC_DIR="orbit/specs/$(date -I)-${CARD_SLUG#*-}"
  mkdir -p "$SPEC_DIR"
  CHILD=$(plugins/orb/scripts/promote.sh "$CARD")
  bd update "$CHILD" \
    --parent "$EPIC" \
    --set-metadata "rally_card_phase=proposed" \
    --set-metadata "rally_branch=rally/${CARD_SLUG#*-}" \
    --set-metadata "rally_spec_dir=$SPEC_DIR"
done

# 5. Stage 2 — launch decision-pack sub-agents in parallel
bd update "$EPIC" --set-metadata "rally_phase=designing"
# (one Agent call per child in a single message; each writes <spec_dir>/decisions.md)
# Verify each return via three-primitive snapshot diff.

# 6. Stage 3 — consolidated decision gate (AskUserQuestion per card)
# Re-launch sub-agents to write <spec_dir>/interview.md per card
bd update "$EPIC" --set-metadata "rally_phase=design-review"

# 7. Stage 4 — disjointness check
# Suppose foo + bar share a trait; baz + quux are disjoint.
bd dep add <bar-bead> <foo-bead>
# (no edges between baz and quux — they run parallel)
bd update "$EPIC" --set-metadata "rally_phase=implementing"

# 8. Stage 5 — implementation queue
# Initial state: bd ready --type task --parent $EPIC returns foo, baz, quux
# (bar is blocked by foo)
# Launch sub-agents for foo, baz, quux in parallel via Agent run_in_background.
# Each runs /orb:drive <bead-id> in a worktree.

# 9. As foo closes, bd ready surfaces bar.
# Lead launches a sub-agent for bar.

# 10. Suppose quux's drive escalates — sub-agent returns:
#   { "verdict": "parked", "reason_label": "review_converged", "reason": "...", "spec_dir": "..." }
bd close <quux-bead> --reason "PARKED: [review_converged] review converged on REQUEST_CHANGES after 3 iterations"
bd update <quux-bead> --set-metadata "rally_card_phase=parked"

# 11. Stage 6 — PR strategy
# foo + bar: stacked (rally/bar PR targets rally/foo)
# baz: standalone PR (target main)
# quux: parked, no PR

# 12. Stage 7 — completion when all children closed
bd update "$EPIC" \
  --set-metadata "rally_phase=complete" \
  --set-metadata "rally_completed=$(date -Iseconds)"
bd close "$EPIC" --reason "rally complete: pipeline runtime readiness — 3 of 4 cards merged, 1 parked"
```

---

**Next step:** After completion, review all PRs in the order
recommended by the assurance strategy (stacked bottom-up or batched
together).
