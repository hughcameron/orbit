# Richer reconcile rules — brownfield drift beyond field renames

**Date:** 2026-05-16
**Triggered by:** Spec 2026-05-16-ac-taxonomy ac-12 (pre-release brownfield dry-run)

## What surfaced

The `canonicalise --reconcile` dry-run against `meridian-online/finetype` (325 ACs across 54 spec.yaml files) caught a class of brownfield drift the v2 reconcile registry doesn't cover. Two failure modes dominated:

1. **Missing required top-level fields** — pre-orbit-state specs lack `id` (~36 specs). The canonical `Spec` schema requires `id` (no default); the file fails to round-trip through `value_to_canonical::<Spec>`. Reconcile mode can quarantine UNKNOWN fields but cannot SYNTHESISE missing required ones.

2. **Scalar AC entries** — pre-orbit-state ACs were authored as plain strings:
   ```yaml
   acceptance_criteria:
     - "Data pipeline reads 508 CSVs from data/csvs/, runs FineType profile..."
     - "AC1: Zero broken links — all markdown URLs return 200"
   ```
   instead of structured mappings (`{id, description, gate, checked, verification, ac_type}`). The typed parse rejects scalar entries with `invalid type: string ..., expected struct AcceptanceCriterion`.

A third (smaller) class — `meridian-online/arcform` uses `orbit/` (no leading dot) instead of `.orbit/`, so the binary is blind to the entire substrate. This isn't drift inside a file; it's substrate-discovery drift.

## Why the existing reconcile mode can't fix this

The v2 reconcile pipeline is:

1. Permissive YAML parse to `serde_yaml::Value` (always succeeds for valid YAML).
2. `walk_and_classify` mutates the Value: rename / drop / quarantine / Transform per field.
3. `value_to_canonical::<T>` re-parses the mutated Value through the typed schema.

Step 2 only operates on **fields** — adding a missing field, transforming a scalar into a mapping, or remapping the entire AC list shape is outside its mental model. Step 3 fails for any structural drift step 2 doesn't reach.

## What richer rules would look like

Three discrete additions:

- **Filename-derived `id`**: when a Spec is missing `id`, derive it from the filename stem (`2026-05-04-foo` from `.orbit/specs/2026-05-04-foo/spec.yaml`). This is already the established convention; the canonical writer enforces it; reconcile just needs to backfill.
- **Scalar-AC wrapping**: when an `acceptance_criteria` entry is a string scalar instead of a mapping, wrap it as `{id: ac-NN, description: <string>, gate: false, checked: false, ac_type: code}`. The id is positional (`ac-01`, `ac-02`, ...). Wraps without losing semantic content.
- **Layout discrimination** (lower priority): detect `orbit/` vs `.orbit/` at substrate-discovery time and either auto-migrate or surface a clear error pointing at `migrate spec-layout`.

Each of the three is a discrete piece of work with clear acceptance criteria.

## Why this isn't part of 2026-05-16-ac-taxonomy

The typed-AC migration's load-bearing deliverables shipped:
- AcType enum + `ac_type` field on AcceptanceCriterion (ac-01)
- spec.close two-band rule via `blocks_close()` (ac-02)
- Schema-version 0.2 → 0.3 migration with auto-repair (ac-03 + ac-04)
- Disposition::Transform variant + the typed-AC routing handler (ac-05 + ac-06)
- SKILL.md / METHOD.md wiring (ac-07 through ac-10)

The brownfield drift this memo describes is a SEPARATE design problem (richer reconcile mode, possibly a v3 of card 0032). Folding it into this spec would have grown the spec from 15 ACs to ~25 and pushed the Transform shipping out by another review cycle. Cleaner to ship the typed-AC layer now and treat the broader reconcile work as its own card.

## Suggested next move

Distill this memo into a card under `.orbit/cards/` — likely a sibling or extension of card 0032-brownfield-spec-migration. The card's goal is "canonicalise --reconcile handles richer brownfield drift (filename-derived id, scalar-AC wrapping, optional layout discrimination)." The card spawns its own design + spec.

The dry-run validation that ac-12 was meant to provide can re-run as part of that future spec — the same `target/release/orbit canonicalise --reconcile --dry-run --root <brownfield>` command, with the new richer rules in place.
