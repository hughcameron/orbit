# Topology seed pointers behave ambiguously in downstream consumer repos

**Date:** 2026-05-20
**Source:** First downstream adoption of `orbit topology setup` against an external consumer repo (post-0.4.22)

## Observation

`orbit topology setup` writes five self-describing seed entries to `.orbit/topology/<entity>.yaml` (cards, choices, specs-substrate, memories, topology itself). Each seed entry's `canonical_code`, `operational_doc`, and `test_surface` fields point at upstream plugin-internal paths — e.g. `orbit-state/crates/core/src/...` and `plugins/orb/skills/...`.

These paths resolve cleanly when the seed is written into *this* repo (orbit IS the plugin). In a downstream consumer repo, the same paths don't exist on disk. `orbit audit topology` consequently surfaces 15 `stale_pointer` items per downstream repo — three pointer fields × five seed entries — as informational `topology_drift`, not as findings.

The first downstream agent labelled this "by design — seeds describe the substrate schema, not subsystems of this project." That reading is plausible but hasn't been explicitly validated against design intent.

## Why this matters

Every downstream adoption will surface the same 15 informational items at baseline. Two interpretations live underneath:

- **(a) Demonstration shape.** Seeds are templates the operator overwrites with real subsystem entries. Audit should distinguish seeds from operator-authored entries (e.g. via a `kind: seed` field or a `--include-seeds` flag) so the consumer-repo baseline reads clean.
- **(b) Schema-anchor shape.** Seeds are persistent records of the upstream substrate schema, pointing at the *plugin's* implementation files so a curious agent can trace the schema's home. The 15 items are expected noise; consumers learn to filter.

The interpretations imply different ergonomics. Under (a), downstream UX needs a fix. Under (b), it's working as intended and just needs naming somewhere.

## Adjacent finding from the same adoption

The consumer repo also surfaced `missing_entry` items for real source directories the operator could enter via `/orb:topology` when useful. That signal is unambiguous — operator-actionable, no design question.

## Status

N=1 evidence from one downstream adoption. Worth re-evaluating after the second downstream adoption to see if the baseline-15 pattern holds and whether downstream operators report it as friction or as expected noise. If (a) is the right reading, the fix is likely a small audit-side filter rather than a seed-shape change.
