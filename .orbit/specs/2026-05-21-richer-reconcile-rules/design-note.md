# Design Note: Richer reconcile rules

**Date:** 2026-05-21
**Card:** .orbit/cards/0032-brownfield-spec-migration.yaml
**Mode:** closed
**Choice:** .orbit/choices/0023-reconcile-as-canonicalise-mode.yaml — Reconcile as a mode on `orbit canonicalise`, not a separate verb

---

## What good looks like

A downstream project whose specs were authored before the canonical schema settled — missing `id:` per AC, ACs as bare string scalars, ad-hoc top-level fields — doesn't show its agents a parse-failed wall on session prime. I run `orbit canonicalise --reconcile --dry-run`, read the disposition list, then `--reconcile`, and the substrate snaps to canonical shape in one pass. When a file is genuinely beyond the registry's reach, the error tells me *which verb to run next* rather than dropping me into raw yaml parse errors with no breadcrumb. Specifically: the 54 brownfield specs that currently report `parse_failed` against orbit 0.4.27 reach `orbit verify` clean after one `--reconcile` invocation.

## Pinned approach

- **Choice 0023** pins the surface — single verb (`orbit canonicalise --reconcile`), two hand-rolled JSON envelopes (`run_canonicalise` at `cli/src/main.rs:689-708`, `run_reconcile` at `cli/src/main.rs:742-774`), additive `dispositions` array on the reconcile envelope, ac-01 strict-parse invariant preserved on every routine path.
- **Spec 2026-05-16-ac-taxonomy** added the `Disposition::Transform` variant — value-level transforms over existing keys aren't new shape. But this spec's two new rules need TWO registry-shape extensions on top, not just new entries:
  - A **pre-walk synthesise phase** for `id`-from-filename — the existing `walk_and_classify` iterates only over keys present in the mapping; a synthesiser that creates a missing required key has to fire before that loop and needs filesystem-path context (via a `TransformContext` thread-through). Per review-spec MEDIUM finding.
  - A **list-element registry scope** for scalar-AC wrap — the existing registry has top-level-field and inner-mapping-field scopes; scalar list elements get skipped at `reconcile.rs:562-566`'s `as_mapping_mut continue`. A third scope keyed on structural-path `<field>[]` (no trailing field name) is required, positioned before `recurse_inner`'s iteration. Per review-spec HIGH finding.
- The shape extensions ship under ac-07 of the spec; ac-01 and ac-02 ride on them.

## Deferred items

- **Layout discrimination** (`orbit/` vs `.orbit/`) — different problem (substrate discovery, not field shape). Memo flagged as lower priority. Defer to a separate spec against card 0032 or card 0017.
- **Project-local registry overrides** — choice 0023 already excluded this from v1; same posture here.
- **Conformance `parse_failed_spec` finding family + session-prime failure-surface breadcrumb** — separate spec against card 0039-workflow-conformance. The chain (failure → conformance → reconcile) needs both ends to be useful, but they ship as siblings, not bundled. Rationale in memo `.orbit/memos/2026-05-21-substrate-first-under-pressure.md`.

## Implementation notes

- **`Spec` missing top-level `id`** — when permissive parse encounters a `spec.yaml` lacking `id:`, synthesise it from the parent folder's stem (`.orbit/specs/2026-05-04-foo/spec.yaml` → `2026-05-04-foo`). The canonical writer already enforces this convention via choice 0021; the registry backfills via the pre-walk synthesise phase added under ac-07 (the existing `Disposition::Transform` handler only operates on keys present in the mapping). Validation against the ~36-spec subset of the finetype tree that fails on this rule alone (subset count to be confirmed at implement time during ac-05's dry-run).

- **String-shaped `acceptance_criteria[]` entry** — when permissive parse encounters a scalar where an `AcceptanceCriterion` struct is expected, wrap as `{id: ac-NN, description: <string>, gate: false, checked: false, ac_type: code}`. The `id` is positional in source order (`ac-01`, `ac-02`, ...). Fires at the new list-element scope added under ac-07 (structural-path `Spec.acceptance_criteria[]`), positioned before `recurse_inner`'s `as_mapping_mut continue` at `reconcile.rs:562-566`. Validation against the ~5-spec subset of finetype that trips on this rule alone, plus the larger subset where both rules compose.

- **Canonicalise failure-surface breadcrumb** — when a `canonicalise` run (with or without `--reconcile`) leaves any file in `parse_failed` after the registry's full rule set has applied, both run summaries append a breadcrumb line: `N file(s) failed parse — run 'orbit audit conformance --json' for structured findings`. Both JSON envelopes — `run_canonicalise` at `cli/src/main.rs:689-708` and `run_reconcile` at `cli/src/main.rs:742-774` — gain an additive optional `next_step` field carrying the same string when `parse_failed > 0` and null otherwise. Two envelopes, two edit sites; identical text.

- **Validation set** — the 54 specs in `meridian-online/finetype` (per memo 2026-05-16) are the canonical brownfield validation set. Dry-run against that tree should report 0 `parse_failed` after the new rules ship, against the current "54 parse_failed". Cross-repo validation isn't a CI gate — it's a one-shot the implementing agent runs locally and records in the spec's progress.

- **Test discipline** — extend the existing reconcile fixture pattern at `orbit-state/crates/core/tests/fixtures/reconcile/`. Spec-shape fixtures: `missing-id/`, `scalar-ac/`, `missing-id-and-scalar-ac/` (composition), plus a `post-reconcile-parse-fail/` fixture covering the breadcrumb path. Each fixture asserts the dry-run disposition list and the round-trip canonical output. **Shape-extension fixtures** (per ac-07) are independent: a synthetic missing-top-level-key fixture and a list-with-scalar-elements fixture at `orbit-state/crates/core/tests/reconcile_shape_extensions.rs` (or equivalent) that exercise the registry-shape mechanisms directly, not through the Spec schema.

- **Per-AC `ac_type` recommendations** (per spec 2026-05-16-ac-taxonomy):

  | AC concern | Recommended type | Rationale |
  |---|---|---|
  | ac-01: filename-derived `id` rule ships and registers | `code` | Closes on fixture test asserting the transform |
  | ac-02: scalar-AC wrap rule ships and registers | `code` | Closes on fixture test asserting the transform |
  | ac-03: composition (both rules in one file) | `code` | Closes on combined fixture test |
  | ac-04: canonicalise breadcrumb on parse failure (both envelopes) | `code` | Closes on tests asserting the line in stderr / both envelopes |
  | ac-05: validation set, finetype dry-run reports 0 parse_failed | `code` | Closes on the implementing agent's recorded local run (cross-repo, one-shot, captured in progress.md) |
  | ac-06: choice 0023 references + consequences updates | `doc` | Closes on file edits |
  | ac-07: registry-shape extensions (synthesise phase + list-element scope) | `code` | Closes on independent shape-extension unit tests |
