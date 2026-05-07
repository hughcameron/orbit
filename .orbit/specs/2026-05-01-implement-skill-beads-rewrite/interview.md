# Design notes: /orb:implement — beads rewrite

**Date:** 2026-05-01
**Bead:** orbit-6da.1 (rally orbit-6da)
**Decision:** .orbit/choices/0011-beads-execution-layer.md
**Foundation spec:** .orbit/specs/2026-05-01-beads-foundation/spec.yaml

## Charter (from bead description)

> Rewrite /orb:implement to read ACs from bead acceptance field via
> parse-acceptance.sh instead of spec.yaml + progress.md. Track progress
> via bd update --acceptance. Gate enforcement via parse-acceptance.sh
> next-ac. Detours become sub-beads with discovered-from dependency.
> AC checking via parse-acceptance.sh check.

## What disappears

The current `/orb:implement` skill is layered on three obsolete substrates:

| Old mechanism                     | Replaced by                              |
|-----------------------------------|------------------------------------------|
| `progress.md` (Spec hash, ACs)    | bead acceptance field (`bd show --json`) |
| `parse-progress.sh`               | `parse-acceptance.sh`                    |
| `spec.yaml` as input              | bead ID as input                         |
| `TaskCreate` / `TaskUpdate` list  | `bd ready` / `bd show` surfaces          |
| Drift detection (`Spec hash`)     | dropped — beads is atomic; the field IS the contract |
| `DRIFT_NOTICE` constant           | dropped                                  |
| Resume reconcile (cancel-rebuild) | `bd show <id>` on resume                 |
| `RESUME_REBUILD_WARNING`          | dropped                                  |
| Detour append in `progress.md`    | sub-bead via `bd create --deps discovered-from:<parent>` |
| Cards 0003 / 0009 cross-refs      | reframed as "capability now in beads"    |

## What survives

- **Test execution discipline.** Monitor heuristic for tests >60s + the
  line-buffered `grep -E 'FAIL|ERROR|AssertionError|Traceback'` filter.
  First-failure checkpoint with interactive vs non-interactive split.
  The canonical `FIRST_FAILURE_NONINTERACTIVE_MARKER` string is preserved
  byte-for-byte — it is referenced by tests and orchestrators outside this
  skill (e.g. `/orb:drive`).
- **Forward findings via memo channel** for product-direction observations
  (.orbit/cards/memos/). Sub-beads are reserved for *blocking* detours;
  follow-up work that doesn't block becomes a top-level bead; insights
  about the product become memos.
- **NO-GO outcome.** Closing with reason "NO-GO: <evidence>" plus
  `bd remember` for persistence. Card direction-layer updates remain
  the author's call.

## Open questions resolved by defaults

**Q: How does the skill find the active bead when no argument is passed?**
A: `bd list --status in_progress --assignee <agent>` returns claimed beads.
If exactly one matches, use it. If none, halt and instruct the agent to
claim from `bd ready --type task`. If multiple, halt and ask the agent to
disambiguate by passing the bead ID explicitly.

**Q: Where do constraints live in a bead?**
A: In the `description` field as prose. The skill surfaces the description
verbatim during pre-flight; no parser. Cards and specs that promote to
beads embed constraints in description text. (No new convention required;
description is markdown-flavoured prose.)

**Q: Sub-bead vs blocked-by edge for detours?**
A: Use `--parent <current>` for hierarchy and `--deps
"discovered-from:<current>"` for provenance. The agent's discipline is to
work the sub-bead first; no hard `blocks` edge needed because the agent
explicitly resumes via `bd show <parent>` after `bd close <sub-bead>`.

**Q: What about agents working multiple beads concurrently?**
A: Out of scope for this rewrite. The skill assumes one in-progress bead
per agent session — the foundation-spec design assumption.

## Constraints inherited from rally

- The skill must not introduce parallel state (no progress.md, no
  TaskList, no decisions.md fork).
- `parse-acceptance.sh` is the only AC parser; SKILL.md never parses ACs
  inline.
- Gate enforcement is delegated to `parse-acceptance.sh next-ac` —
  declaration-order blocking is an orbit convention defined in
  `.orbit/conventions/acceptance-field.md`.
- Card 0003 / 0009 mechanisms are referenced as historical only — no
  card edits in this spec.
