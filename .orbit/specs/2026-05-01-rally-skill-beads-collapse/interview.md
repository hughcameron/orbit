# Design notes: /orb:rally — beads collapse

**Date:** 2026-05-01
**Bead:** orbit-6da.3 (rally orbit-6da)
**Decision:** .orbit/choices/0011-beads-execution-layer.md
**Predecessors:**
- .orbit/specs/2026-05-01-implement-skill-beads-rewrite/spec.yaml (orbit-6da.1, shipped)
- .orbit/specs/2026-05-01-drive-skill-beads-rewrite/spec.yaml (orbit-6da.2, shipped)

## Charter (from bead description)

> Collapse /orb:rally into beads dependency graph. Rally becomes an
> epic bead with child task beads. bd ready --type task replaces rally
> orchestration. bd dep add mid-flight replaces disjointness→serial
> decision. Rally skill may deprecate entirely — the dependency graph
> IS the rally.

## What disappears

The current `/orb:rally` (~616 lines) layers on artefacts that beads
supplants.

| Old mechanism                                | Replaced by                                                |
|----------------------------------------------|------------------------------------------------------------|
| `rally.yaml` (durable state)                 | epic bead + child beads + epic metadata                    |
| Per-card status (proposed/designing/...)     | bead.status (open/in_progress/closed) + child metadata     |
| Phase tracking (proposing → complete)        | epic metadata `rally_phase=...`                            |
| `worktree` field per card                    | child bead metadata `rally_worktree=<path>`                |
| `implementation_order` + serial decision     | `bd dep add <later> <earlier>` — graph topology IS order   |
| `bd ready --type task --parent <epic>` queue | replaces TaskCreate / TaskList for in-session visibility   |
| §1 scan-for-rally.yaml                       | `bd list --type epic` filtered by rally_phase metadata     |
| §11 worktree path-resolution rule            | dropped — drive resumes from bead metadata, not drive.yaml |
| §12 rally.yaml validation                    | dropped — beads enforces structural validity               |
| Two-layer state model                        | one layer: the bead graph                                  |
| `parked_constraint` field                    | `bd close --reason "PARKED: [<label>] <constraint>"`       |
| `reason_label` → rally.yaml mapping table    | embedded inline in the close --reason text                 |
| `commit-before-delegation` (interview.md)    | git hygiene step, not a drive resumption requirement       |

## What survives

- **Proposal gate** — author qualifies the rally interactively. Three
  AskUserQuestion options (`approve-all`, `modify-list`, `decline`)
  preserved verbatim.
- **Thin-card refusal at proposal** — any candidate with <3 scenarios
  blocks the proposal before it is shown. Unconditional regardless of
  serial-or-parallel outcome.
- **Decision-pack queued design** — N parallel sub-agents produce
  `<spec_dir>/decisions.md`. The lead presents a consolidated decision
  gate. This is rally's value-add — preserved.
- **Three-primitive snapshot-diff verification** — pre/post
  `git status --porcelain`, returned JSON file list, artefact assertion
  under `<spec_dir>`. Trust + post-verify discipline preserved.
- **Disjointness check** — extract files / symbols / shared references
  from interviews. Non-empty intersection wires `bd dep add` between
  child beads to encode serial order.
- **Stacked PR / batched diff strategies** — git/PR mechanics unchanged;
  rally.yaml is removed but the strategies still apply.
- **Single-strike NO-GO** — a card that fails any review parks
  immediately via `bd close --reason "PARKED: ..."`. No rally-level
  retries.
- **Recursive context separation** — sub-agent runs `/orb:drive
  <bead-id>` which itself forks review-spec / review-pr Agents. Rally
  does not invoke reviewers directly.

## Rally as an epic bead

Rally creates a dedicated epic bead at proposal-approval time:

```bash
EPIC=$(bd create "<goal_string>" -t epic -p 1 \
  --description "$(cat <<'EOF'
Rally: <goal_string>
Cards: <list of card paths>
Autonomy: <guided|supervised>
Started: <ISO-8601>
EOF
)" --silent)
bd update "$EPIC" --set-metadata "rally_phase=approved" \
  --set-metadata "rally_autonomy=<guided|supervised>" \
  --set-metadata "rally_started=$(date -Iseconds)"
```

Each card becomes a child task bead via `promote.sh` followed by
`bd update <bead> --parent <EPIC>`:

```bash
for CARD in "${CANDIDATE_CARDS[@]}"; do
  CHILD=$(plugins/orb/scripts/promote.sh "$CARD")
  bd update "$CHILD" \
    --parent "$EPIC" \
    --set-metadata "rally_card_phase=proposed" \
    --set-metadata "rally_branch=rally/<slug>"
done
```

`bd children <epic-id>` lists every card-bead in the rally; `bd ready
--type task --parent <epic-id>` lists the next claimable cards
respecting dependency edges.

## State location: epic metadata vs child metadata

| Field                  | Lives on    | Reason                                          |
|------------------------|-------------|-------------------------------------------------|
| `rally_phase`          | epic        | Whole-rally phase: approved / designing / ...  |
| `rally_autonomy`       | epic        | Guided or supervised (rally-level)              |
| `rally_started`        | epic        | Timestamp                                       |
| `rally_card_phase`     | each child  | proposed / designing / designed / ...           |
| `rally_worktree`       | each child  | Absolute path or "main"                         |
| `rally_branch`         | each child  | Git branch name                                 |
| `rally_spec_dir`       | each child  | Path to the card's spec dir                     |

`bd show <epic-id>` plus `bd children <epic-id> --json` gives the lead
everything it needs to resume.

## Implementation orchestration via bd ready

Once disjointness is checked and edges wired:

- **Serial:** `bd dep add <later> <earlier>` for each ordered pair.
  `bd ready --type task --parent <epic>` returns exactly one card at
  a time (the head of the chain), one card at a time.
- **Parallel:** no edges added. `bd ready --type task --parent <epic>`
  returns all cards simultaneously, ready for parallel claim by N
  sub-agents.

Sub-agents (in worktrees or main) claim atomically via `bd update
<bead-id> --claim`, then run `/orb:drive <bead-id>` (drive accepts a
bead-id since orbit-6da.2). Drive resumes from `drive_stage` metadata
and runs review-spec → implement → review-pr internally.

When drive completes (APPROVE at review-pr), the child bead closes
with `bd close <bead-id> --reason "drive completed: ..."`. The lead's
sub-agent then returns a JSON verdict.

## Single-strike park

Drive escalations inside a sub-agent surface as a JSON return:

```json
{ "verdict": "parked", "reason_label": "<label>", "reason": "<one-line>",
  "spec_dir": "<spec_dir>" }
```

Rally lead converts that into a `bd close` invocation:

```bash
bd close <child-bead> --reason "PARKED: [<label>] <reason>"
```

The reason_label vocabulary is preserved (six fixed tokens — budget,
recurring_failure, contradicted_hypothesis, diminishing_signal,
review_converged, tool_surface_incomplete) but lives only in the
close --reason string. No `parked_constraint` field, no separate
mapping table on disk.

## Mid-flight serial conversion

If parallel implementation surfaces a shared symbol mid-flight (e.g.
two sub-agents about to touch the same file), the lead can run:

```bash
bd dep add <later-bead> <earlier-bead>
```

`bd ready` will then withhold `<later-bead>` until `<earlier-bead>`
closes. In-progress work continues; the runtime change is the queue,
not the running cards.

## Resumption

`/orb:rally` invoked with no args:

```bash
bd list --type epic --status in_progress --json \
  | python3 -c "import sys,json; print('\n'.join(b['id'] for b in json.load(sys.stdin) if b.get('metadata',{}).get('rally_phase') and b['metadata']['rally_phase'] != 'complete'))"
```

- **Single match** → resume.
- **Zero matches** → propose new rally.
- **Multiple matches** → halt and ask for the epic-id.

`bd show <epic> --json` reveals `rally_phase`. `bd children <epic>
--json` reveals each card's `rally_card_phase` and bead status. From
that, the lead resumes at the right stage with the right per-card
state. No file-presence detection. No drive.yaml resolution rule.

## Migration

There is no migration path for in-flight `rally.yaml` rallies. They
finish under the prior version or restart. The new skill ignores
`rally.yaml` files entirely — `bd list --type epic` is the resumption
mechanism.

## Out of scope

- `promote.sh` enhancements — separate spec if needed.
- `/orb:drive` changes — already shipped in orbit-6da.2.
- `/orb:design` changes — rally's decision-pack model is rally-specific
  and remains so.
- `/orb:review-spec` and `/orb:review-pr` changes — drive forks them;
  rally never invokes them directly.
- Multi-rally coordination (rally-of-rallies) — out of scope.
- Auto-migration of in-flight rally.yaml — there is no migration.

## What deprecates entirely?

The bead description says *"Rally skill may deprecate entirely — the
dependency graph IS the rally."*

After this rewrite, rally is a thin orchestrator that adds four things
beads alone doesn't:

1. **Goal-based card scanning + proposal** (cards are an orbit concept,
   not a beads concept).
2. **Decision-pack queued design** (executive-ready gate via N parallel
   sub-agents producing decisions.md).
3. **Disjointness analysis → bd dep edges** (analytical work that
   produces the dep graph beads then orchestrates).
4. **Coordinated PR review** (stacked vs batched, presented together).

These four are why rally survives — without them, the rally would be
`bd create -t epic + bd update --parent` per card, which is too thin
to warrant a skill. Rally is the *capability* that produces these four
artefacts; beads runs the resulting graph. Rally does not deprecate
this round.
