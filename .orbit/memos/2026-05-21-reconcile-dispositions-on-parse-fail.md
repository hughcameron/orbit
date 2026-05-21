# Reconcile drops dispositions silently when typed-reparse fails

**Date:** 2026-05-21
**Source:** Spec 2026-05-21-richer-reconcile-rules ac-05 implementation finding

## Observation

`reconcile_one` in `orbit-state/crates/core/src/reconcile.rs:421-427` pushes the file's dispositions only after `value_to_canonical` succeeds:

```rust
let reserialised = match value_to_canonical::<T>(&value) {
    Ok(s) => s,
    Err(e) => {
        report.parse_failed.push((path.to_path_buf(), e));
        return;  // <-- file_dispositions dropped here
    }
};
```

When the typed re-parse fails (because of drift the registry doesn't reach), the dispositions accumulated during `walk_and_classify` are dropped on the early return. The run summary reports the file in `parse_failed` but says "0 disposition(s)" even when the registry's rules fired during the walk.

This masks rule visibility on multi-drift trees. During the spec 2026-05-21-richer-reconcile-rules implementation, the first finetype dry-run reported `54 parse_failed, 0 dispositions` — even though the synthesise-id rule and the scalar-AC wrap rule were both firing in `walk_and_classify`. The 0-disposition count made the failure mode look like "no rules applied" when the truth was "rules applied but the file still wouldn't reparse because of other drift."

## Why it matters

Dry-run output is especially misleading. The dry-run's contract is "show me what reconcile would do" — but currently it shows only what reconcile would *successfully* do, hiding any attempt that gets blocked by unrelated drift downstream.

The pattern bites when a brownfield tree has composed drift (the common case): a file missing both `id` and `status` won't show the id-synthesis disposition because the status-missing failure happens after id has been synthesised. The agent can't see "the id rule fired" — only "this file couldn't be reparsed."

## The fix (small)

Move the disposition push to fire *before* the early-return on typed-reparse fail. About 4 lines:

```rust
let reserialised = match value_to_canonical::<T>(&value) {
    Ok(s) => s,
    Err(e) => {
        report.parse_failed.push((path.to_path_buf(), e));
        push_dispositions(&mut report.dispositions, &file_dispositions);  // <-- new
        return;
    }
};
```

This shifts the semantics of `report.dispositions` from "dispositions on rewritable files" to "all attempted dispositions". The existing test suite doesn't depend on the narrower interpretation (verified during the 2026-05-21-richer-reconcile-rules implementation — all 36 existing reconcile tests passed without modification when the broader semantics was added).

## Why it's not in 2026-05-21-richer-reconcile-rules

The scope-broadening that spec accepted (adding the status synthesiser + criterion rename) eliminated the validation set's parse_failed cases — every brownfield spec now migrates cleanly. The masking issue no longer surfaces in that validation. But the architectural quirk remains: a future brownfield project with drift beyond the registry will hit the same masking again.

## Suggested next move

Distill into a card (likely a new spec against card 0032) or fold into the next brownfield-rules spec as a small additional AC. The change is one-AC sized, mechanically symmetric to existing reconcile logic, and the regression-test surface is straightforward (a fixture with mixed drift that exceeds the registry, assert the report carries the attempted dispositions).

## Related

- `[[2026-05-16-richer-reconcile-rules]]` — the broader rule-set scope
- `.orbit/specs/2026-05-21-richer-reconcile-rules/progress.md` — captures this finding under "Findings to surface"
- `.orbit/choices/0023-reconcile-as-canonicalise-mode.yaml` — the surface decision this would extend

## Status

Memo only. Surface again in the next brownfield-rules spec.
