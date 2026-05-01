# Brief: beads as orbit's memory layer

## Premise

Replace orbit's drive and rally execution flows with beads as the single durable memory tool for work in flight. Cards and memos stay as product direction artefacts upstream of execution. PRs stay as milestones, but agents own them end-to-end with no human approval gate. Specs, `rally.yaml`, `progress.md` and decisions docs all stop being separate artefacts.

The goal is artefact volume reduction without losing the properties that make orbit work: durable handoffs between sessions, explicit decisions, parallel-to-serial detection, and a clear next thing to do.

## What stays, what collapses

Three things stay as-is, deliberately:

- **Memos** as rough capture for product direction (`/memo`)
- **Cards** as refined product direction, human-authored (`/card`)
- **PRs** as milestones, agent-owned end-to-end (creation and merge both)

The collapse happens below the card line, in the execution layer:

| Orbit today | Beads equivalent |
|---|---|
| Spec body (objective, context, scope) | Promoted from card into one or more beads at execution start |
| Acceptance criteria | Checklist in bead body, or sub-beads where they warrant their own work |
| Hard constraints | `AGENTS.md` or `bd remember` (auto-injected at `bd prime`) |
| Verification approach | Tests in the codebase |
| Decisions doc | `bd remember` entries, optionally tagged with bead ID |
| Progress.md | Bead status, plus first unchecked checklist item as the current AC |
| Detours | Sub-beads with a dependency back to the parent |
| Rally.yaml | The dependency graph itself |
| "What's next?" | The auto-ready query (beads with no unblocked dependencies) |
| Project objective function | Top-level bead, or a pinned `bd remember` entry |
| PR review (human gate) | Removed; agent merges on tests passing plus objective-function check |

## Drive, re-imagined

A card becomes one or more beads at execution start. The bead lifecycle is `open` → `in-progress` → `merged` → `closed`. No human review gate. The agent reads the bead at session start (via `bd prime`, which also injects relevant memories), works through the checklist, and uses `bd update` to advance status. Decisions made along the way go to `bd remember` so the next session inherits them.

Done means: tests pass, the objective function has moved (or the work is genuinely infrastructural), the PR is opened, and the agent merges it. The bead closes on merge. The PR remains as a milestone in git history, retrospectively reviewable via the commit log rather than gated upfront.

The current re-anchoring belt-and-braces (`progress.md` pointer, `session-context.sh`, implement skill rule) collapses to one thing: the first unchecked checklist item in the active bead. `bd prime` surfaces it at session start.

## Rally, re-imagined

Rally stops being a separate concept. A multi-card body of work is a parent bead (or a beads molecule, `bd mol`) with child beads, and dependencies between them expressing serial versus parallel structure. Parallel by default: any two beads without a dependency between them are parallelisable, and the auto-ready query returns the set the agent or orchestrator can fan out across. The disjointness check (paths, symbols, shared references) becomes a pre-flight on the dependency graph rather than a separate qualification step.

When parallel becomes serial mid-flight (the shared-types case), you add a dependency edge. The graph is the truth, no `rally.yaml` drift to manage.

The session-death-mid-rally failure mode is handled by the same mechanism that handles drive: beads are durable, the ready query is idempotent, and resuming means querying the graph again rather than reconstructing orchestrator state.

## What goes away

The `spec.yaml` file and its schema. The `decisions.md` file. The `progress.md` file. `rally.yaml`. The `/review-spec` and `/review-pr` commands as human gates. The mission-resilience belt-and-braces, since one mechanism replaces three.

## What still needs a home

A few things sit deliberately outside beads.

**Memos and cards** are the product direction layer, human-authored, and they stay as-is. The boundary is clear: above the card is direction, below the card is execution. Beads are below the line. The promotion step (card → bead(s)) is where direction becomes work. That's a one-time decompose action per card, not an ongoing ceremony.

**Hard constraints** that apply project-wide go in `AGENTS.md` once, not per-bead. Per-card constraints can sit in the card body and be carried into the bead description on promotion.

**The objective function** lives as a pinned `bd remember` entry, or as a top-level bead that all work beads roll up to. The `bd prime` injection means the agent sees it every session without prompting.

**Tests** remain in the codebase. They are the verification truth and, alongside the objective function, are what gates merge in the absence of a human review.

## Open questions

A few things to decide before committing:

1. **Card-to-bead promotion.** Is this a `/promote` skill the human runs, or something the agent does autonomously when a card is ready? The first keeps a human checkpoint at the entry to execution; the second extends agent autonomy further upstream. Given the move toward objective-function-driven work, the second is probably the right answer, but it's worth a deliberate decision.
2. **AC granularity.** Where is the line between checklist item and sub-bead? A reasonable rule of thumb: anything warranting its own session is a sub-bead, anything resolved in-session is a checklist item.
3. **Agent-self-review at merge.** The agent now decides when to merge. Is "tests pass plus objective function moved" sufficient, or is there value in a structured self-review step (e.g., a review-bead the agent must complete before merging)? The lighter answer is probably right initially, with a self-review step added if the no-approval flow produces obvious gaps.
4. **Schema discipline.** If a bead body is freeform, what stops the gate-versus-code AC distinction quietly drifting? Possibly nothing, and that might be acceptable given the move to objective-function-driven work, but worth being deliberate about.
5. **Skill interaction.** Orbit's execution skills (`/design`, `/implement`, and so on) probably stay, but with their inputs and outputs reframed in beads terms. The implement skill in particular needs rewriting around the auto-ready query and `bd update`.

## Trial path

Pick one recent rally, ideally one with a parallel-to-serial revelation in its history, and replay it as beads:

1. Initialise beads in a scratch repo. Author the rally's parent bead with the objective function, sourced from the original card.
2. Author the cards as child beads with dependencies matching the original rally's serial/parallel structure.
3. Use `bd remember` to seed the decisions that were made during the original work.
4. Walk the auto-ready queue, simulating each session, and check whether the next thing to do is always obvious from the queue plus `bd show`.
5. Inject the parallel-to-serial event mid-replay (add the dependency edge) and check that the rest of the work re-orders correctly.
6. At the end, simulate PR creation and auto-merge for each completed bead. Inspect the resulting git history and ask: would I be comfortable seeing only this, with no per-PR approval step?

If the replay feels lighter than the original rally without losing track of what mattered, the move is sound. If you find yourself wanting spec fields back, that tells you precisely which schema discipline was load-bearing. If the no-approval PR flow feels uncomfortable, that's a signal to add a self-review step at merge time before going wider.
