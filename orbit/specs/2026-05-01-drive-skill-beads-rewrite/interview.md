# Design notes: /orb:drive — beads rewrite

**Date:** 2026-05-01
**Bead:** orbit-6da.2 (rally orbit-6da)
**Decision:** orbit/decisions/0011-beads-execution-layer.md
**Predecessor:** orbit/specs/2026-05-01-implement-skill-beads-rewrite/spec.yaml (orbit-6da.1, shipped)

## Charter (from bead description)

> Rewrite /orb:drive to use bead lifecycle. Promotion (promote.sh) replaces
> design→spec. Cold-fork review reads acceptance field instead of spec.yaml.
> Bead status replaces drive.yaml state machine. bd close --reason replaces
> drive completion flow. Agent promotes autonomously at drive start.

## What disappears

The current `/orb:drive` (~620 lines) layers on artefacts that beads supplants.

| Old mechanism                                | Replaced by                                                |
|----------------------------------------------|------------------------------------------------------------|
| `drive.yaml` (state machine)                 | bead `status` + bead metadata fields                       |
| `interview.md` (design output)               | dropped — card is the design artefact                      |
| `spec.yaml` (spec output)                    | dropped — promote.sh card→bead is the spec                 |
| Per-iteration spec dir (`drive-v2`, `-v3`)   | new bead per iteration with `discovered-from` edge         |
| File-presence stage detection                | bead status + `drive_stage` metadata                       |
| Pre-change drive.yaml refusal block          | dropped — drive.yaml is gone in this version               |
| `review_cycles.<stage>` in drive.yaml        | bead metadata `drive_review_<stage>_cycle`                 |
| `review_cycle_dates.<stage>` in drive.yaml   | bead metadata `drive_review_<stage>_date`                  |
| `history` array in drive.yaml                | `bd dep tree` over `discovered-from` edges + `bd remember` |
| `current_spec` path                          | dropped — bead-id is the handle                            |

## What survives

- **Cold-fork review architecture (decision 0011 D2).** Reviews still run
  in fresh agent sessions via the `Agent` tool. The brief points the
  fork at the bead's content, not at a `spec.yaml`.
- **REQUEST_CHANGES budget of 3 per stage.** Counter lives in bead
  metadata. The 4th would-be cycle synthesises a BLOCK with the
  byte-identical canonical constraint string from §5a of the old skill.
- **Iteration budget of 3.** Each iteration is its own bead in the
  `discovered-from` graph. Iteration count is bead metadata.
- **Heartbeat (full autonomy only).** Idempotent CronList-first
  reconciliation. Heartbeat reads bead state instead of drive.yaml +
  progress.md. Self-terminates on `bead.status == closed`.
- **Severity dispatch + four-option verdict prompt.** UX contract from
  card 0005. Preserved verbatim.
- **Thin-card refusal in full mode.** ≥3 scenarios required for full
  autonomy. No silent downgrade.
- **Honest escalation triggers.** Recurring failure mode, contradicted
  hypothesis, diminishing signal. Disposition section preserved.

## State location: bead metadata vs labels vs memory

Drive needs to persist orchestration state across resumption: autonomy
level, current stage, review-cycle counters, review-cycle dates, original
card path, iteration number.

| Mechanism            | Pros                                       | Cons                                                                |
|----------------------|--------------------------------------------|---------------------------------------------------------------------|
| Bead metadata (k=v)  | First-class field, JSON-queryable          | One write per field                                                 |
| Labels               | bd-list filterable                         | Best for taxonomy, awkward for typed values                         |
| `bd remember`        | Persistent across closures                 | Global keyspace, not scoped to bead                                 |
| Thin drive.yaml      | All-state-in-one-file, mature              | Re-introduces the very state file we're removing                    |

**Decision: bead metadata.** All orchestration state on the bead itself
via `bd update <bead> --set-metadata "drive_<key>=<value>"`. The bead
stays the single source of truth. Specific fields:

| Metadata key                   | Value                                       |
|--------------------------------|---------------------------------------------|
| `drive_card`                   | absolute card path (input)                  |
| `drive_autonomy`               | `full` / `guided` / `supervised`            |
| `drive_iteration`              | integer 1..3                                |
| `drive_stage`                  | `review-spec` / `implement` / `review-pr` / `complete` / `escalated` |
| `drive_review_spec_cycle`      | integer 0..3                                |
| `drive_review_spec_date`       | ISO date `YYYY-MM-DD` or `null`             |
| `drive_review_pr_cycle`        | integer 0..3                                |
| `drive_review_pr_date`         | ISO date `YYYY-MM-DD` or `null`             |

## Iteration model

Each iteration is its own bead. NO-GO closes the current bead with
`--reason "NO-GO: <constraint>"` and creates a new bead via promote.sh
with `--deps "discovered-from:<closed-bead>"`. The new bead's description
embeds the cumulative constraint history (extracted from bd memories
and the dep chain).

The `bd dep tree` from the original (iteration 1) bead IS the iteration
history; no separate `history` array required.

## Review file location

Cold-fork review verdicts are produced as markdown files. With no spec
dir, they need a home.

- **Chosen:** `orbit/reviews/<bead-id>/review-{spec,pr}-<date>[-vN].md`
- **Snapshot:** `orbit/reviews/<bead-id>/bead-snapshot-<date>.md` —
  static markdown render of the bead's description + acceptance field
  written at the start of each review cycle. The cold-fork's brief
  points to this snapshot rather than asking the fork to query `bd`
  (the fork can run in environments without bd configured).

## Cold-fork brief shape

The fork brief tells the reviewer to read the snapshot file and apply
the `/orb:review-spec` (or `/orb:review-pr`) skill. Snapshot replaces
spec.yaml as the input. Verdict format and regex unchanged. The
`/orb:review-spec` skill itself is NOT modified by this rewrite — it
operates on whatever spec-shaped input it's given. Drive's snapshot is
spec-shaped enough.

## Resumption

`/orb:drive` invoked with a bead-id (or no argument while a drive bead
is in_progress) reads `drive_stage` metadata and resumes there. No
file-presence detection. The bead's status + metadata is the entire
resumption surface.

If `/orb:drive` is invoked on a card that already has an in-progress
drive bead (via metadata key match `drive_card == <card_path>`), drive
resumes that bead rather than promoting a duplicate.

## Migration note

There is no migration path for in-flight drives initialised under the
prior version. Any extant `drive.yaml` files are stale. The recommended
path is to finish or park drives under the prior version, then upgrade.
The new skill will not auto-migrate `drive.yaml` content — it ignores
the file entirely.

## Out of scope

- `/orb:review-spec` and `/orb:review-pr` skill body changes —
  separate spec if needed. Drive synthesises spec-shaped snapshots so
  these skills don't have to change.
- `promote.sh` enhancements (constraint injection for iteration ≥2)
  — separate spec. For now, drive injects the constraint into the
  bead's description AFTER promote runs, via `bd update --description`.
- Multi-bead drives (one drive coordinating multiple beads) — that's
  the rally rewrite (orbit-6da.3).
