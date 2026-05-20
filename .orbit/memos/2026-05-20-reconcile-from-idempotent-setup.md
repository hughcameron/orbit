## Reconcile not offered from idempotent setup

Observed in Finetype: ran `/orb:setup` on a project already in idempotent state. Conformance audit reported `audit.drift: 321 entries` across older spec files (pre-canonical field shapes — `constraints`, `ontology_schema`, `evaluation_principles`, `exit_conditions`, top-level `metadata`).

Setup surfaced the drift but routed me to a **manual handle**:

> The drift entries on older specs are quarantine-disposition — they parse fine but carry pre-canonical field shapes. The skill notes reconcile is only offered during a brownfield migration, not from idempotent setup. If you want to clean them up, the manual handle is `orbit canonicalise --reconcile --dry-run` then `--reconcile`.

## The gap

Setup has three branches — greenfield, brownfield, idempotent. Brownfield offers `--reconcile` inline because migration is the whole point of that branch. Idempotent skips it on the assumption that an already-set-up project has nothing to reconcile.

But `audit.drift: 321` is a counter-example: idempotent projects accumulate pre-canonical specs over time as the canonical shape itself evolves. The audit detects this; the skill won't act on it.

## Question

Should §6 of `/orb:setup` offer `orbit canonicalise --reconcile --dry-run` when the conformance audit reports `audit.drift > 0`, regardless of which branch (greenfield/brownfield/idempotent) the skill took? The dry-run is cheap and the prompt-before-apply pattern matches the topology-wiring step in §6d.

Alternatively: leave setup as-is and put reconcile-offered-when-drift-present behaviour in `/orb:audit` or a dedicated `/orb:reconcile` skill — keep setup's scope narrow.

## Related

- Conformance audit per spec 2026-05-19-workflow-conformance
- Topology wiring (§6d) already follows the prompt-before-apply pattern this would mirror
- `[[richer-reconcile-rules]]` from 2026-05-16 — that memo's about *what* reconcile does; this one's about *when* it's offered
