Observations from running `/orb:prioritise` + `/orb:setup` in the arcform repo. Four improvements visible, ranked by leverage.

## 1. Conformance-finding blindness on undotted substrate

arcform had `orbit/` (no dot) containing 22 cards, 16 decisions, specs — but `orbit audit conformance` only saw the surface symptom (`.orbit/METHOD.md` + `.orbit/STYLE.md` missing, MEDIUM) and recommended `orbit setup`. The real finding should have been HIGH: "substrate in non-canonical location `orbit/` — needs migration." Running setup naïvely would have created `.orbit/` alongside `orbit/`, leaving both directories present and the substrate still tool-invisible.

The `/orb:prioritise` contract (single-verb remediation) breaks here — the verb it surfaced would have made things worse. Audit needs an undotted-substrate detector that fires before the canonical-files-missing check.

**Deepest of the four** — `/orb:prioritise`'s value rests on conformance findings pointing at the right verb. When a MEDIUM finding's "remediation" would have made things worse, the trust contract breaks.

## 2. Topology auto-seeds are conceptually wrong outside the orbit-plugin repo

Setup wrote 5 topology entries (`cards`/`choices`/`memories`/`specs-substrate`/`topology`) pointing at `orbit-state/crates/core/src/schema.rs` and similar. In arcform this generated 21 stale-pointer drift entries immediately.

But the deeper bug, surfaced by the arcform agent attempting to remediate: those entries are **orbit's substrate types**, not arcform's subsystems. Topology is designed to map project subsystems (manifest, runner, engine, registry) to canonical code/docs/tests. The seeds only make conceptual sense in the orbit-plugin repo, where those substrate types are themselves implemented as Rust crates. In any other project they're a category error — there's no arcform code that "owns" the cards schema, because cards live in the orbit plugin.

The arcform agent also found that empty `canonical_code` pointers fail validation (entries without code pointers are not load-bearing), and re-running setup is idempotent and doesn't reseed for a new orbit version. So the only honest remediation in arcform was deletion → `configured: false`.

Fix shape: setup should **not auto-seed topology entries in non-plugin projects**. Either scaffold `.orbit/topology/` empty (with a README explaining opt-in via `/orb:topology`), or detect "this is the orbit-plugin repo itself" before seeding substrate-type entries. Topology then becomes truly opt-in, authored project-by-project against real subsystems.

## 3. Reconcile fails on common brownfield `maturity` values

Five cards in arcform used `maturity: active` (for shipped work) and `maturity: in_design` (for active design) — neither in the canonical `planned/emerging/established` set. Reconcile aborted on these and required hand-edits before it would run.

These values are likely common in older orbit repos. Reconcile should auto-map `active → established` and `in_design → emerging` with a log entry, the same way it synthesises missing `id` and `status`.

## 4. `decisions/` (MADR markdown) vs `choices/` (YAML) is unmigrated

Setup migrated `orbit/ → .orbit/` but left `.orbit/decisions/` (16 MADR markdown files) untouched. `.orbit/choices/` doesn't exist, so `orbit choice list` returns empty despite the substrate being there.

Either setup should detect `decisions/` during brownfield migration and offer conversion (markdown → YAML), or the conformance audit needs to surface this as a finding so the agent knows to handle it explicitly.

## Source

arcform repo session, 2026-05-24. Full transcript: user pasted into orbit-plugin repo for capture. Both commits landed in arcform (`176fb61` layout migration, `64cdc69` substrate reconcile) before this memo was written.
