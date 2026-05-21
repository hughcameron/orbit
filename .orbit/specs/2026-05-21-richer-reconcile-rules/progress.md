# Progress — 2026-05-21-richer-reconcile-rules

## Scope broadening mid-implementation

The initial spec (ac-01 + ac-02 + ac-04 + ac-07) covered filename-derived `id` synthesis, scalar-AC wrapping, and the canonicalise breadcrumb. The first finetype dry-run surfaced an assumption reversal: the source memo's diagnosis (~36 specs missing `id`) was wrong about which rule was load-bearing. The dominant failure was missing `status:` (53 of 54 specs), with one outlier (`_template/spec.yaml`) carrying a `criterion`/`description` field rename.

Per the working rules in `/orb:implement` ("Assumption reversals require escalation"), the finding was surfaced to the author, who authorised broadening the spec inline (option 2 of three paths offered). Two ACs added: **ac-08** (`Spec.status` synthesiser defaulting to `open`) and **ac-09** (`acceptance_criteria[].criterion → description` Map rename). Both mechanically symmetric to what was already shipping — one synthesise + one map — and they ride on the same registry-shape extensions delivered under ac-07.

## ac-05 validation against the brownfield validation-set repo

Ran the dry-run + `--reconcile` + `orbit verify` sequence against the local checkout of the validation-set repo named in `.orbit/memos/2026-05-16-richer-reconcile-rules.md`.

### Baseline (before this spec's rules)

`orbit verify`: 54 spec files fail strict parse on the canonical `Spec` schema.

### Migration run (with this spec's full rule set: ac-01 + ac-02 + ac-08 + ac-09)

```
orbit canonicalise --reconcile:
  rewrote 54 file(s), 117 unchanged, 0 parse-failed, 838 disposition(s)
```

Disposition mix:
- `synthesise-id-from-filename` — fired on the specs that lacked `id`
- `synthesise-status-default-open` — fired on all 53 specs that lacked `status`
- `wrap-scalar-ac` — fired on each scalar AC entry across the brownfield specs
- `map` (criterion → description) — fired on the `_template/spec.yaml` outlier
- `quarantine` — pre-existing rules fired on `constraints`, `ontology_schema`, `evaluation_principles`, `exit_conditions`, `metadata`, `decisions`

### Post-migration verify

```
orbit verify: clean
```

Every spec parses against the canonical `Spec` schema. 54 spec.yaml files rewritten; 53 sibling `spec.legacy.yaml` sidecars created carrying quarantined prose content. The finetype tree's git state shows the migration as standard working-tree modifications — reviewable, committable, or revertable as a normal change.

### Breadcrumb verification (ac-04)

On the pre-broadening dry-run (when 54 specs still failed parse), the human-readable output ended with:

```
54 file(s) failed parse — run "orbit audit conformance --json" for structured findings
```

The JSON envelope's `next_step` field carried the same string. Post-broadening, no files fail parse, so the breadcrumb correctly does not appear and `next_step` is null. Both behaviours match ac-04's contract.

## Residual `parse_failed` enumeration

**Zero residual parse_failed entries.** All 54 specs migrated cleanly under the broadened rule set.

## Findings to surface

### Finding 1: dispositions silently dropped on typed-reparse fail

`reconcile_one` in `orbit-state/crates/core/src/reconcile.rs:421-427` pushes the file's dispositions only after `value_to_canonical` succeeds. When the typed re-parse fails (file moves into `parse_failed`), the dispositions accumulated during `walk_and_classify` are dropped. The pre-broadening dry-run reported "0 disposition(s)" despite the new rules firing — masking what reconcile attempted to do on multi-drift files.

This finding doesn't block ac-05 (the broadened scope produces 0 parse_failed, so the masking issue no longer surfaces in this validation set). But it remains an architectural quirk worth a follow-up: when a future brownfield project has drift that exceeds the current registry, the dry-run should still show what was attempted.

**Disposition:** memo at `.orbit/memos/2026-05-21-reconcile-dispositions-on-parse-fail.md` capturing the rationale + the ~4-line fix proposal.

### Finding 2: source memo's diagnosis was inverted

`.orbit/memos/2026-05-16-richer-reconcile-rules.md` named "missing id (~36 specs)" as the dominant brownfield failure. The reality was "missing status (53 of 54 specs)". The implementation didn't catch this until the first cross-repo dry-run. Implication for future memo discipline: claims about validation-set composition should be verified by a quick dry-run during the memo phase, not assumed during distill.

**Disposition:** memo at `.orbit/memos/2026-05-21-validate-memo-claims-against-substrate.md` capturing the pattern as a substrate-engagement observation.

## ACs status

All nine ACs covered by the implementation:

- ac-01 (gate) — filename-derived `id` synthesis: closed via unit test `ac_01_synthesise_id_from_filename_round_trips`
- ac-02 (gate) — scalar-AC wrap: closed via unit test `ac_02_wrap_scalar_ac_round_trips`
- ac-03 — composition + idempotency: closed via unit test `ac_03_composition_missing_id_and_scalar_ac_round_trips_idempotently`
- ac-04 (gate) — canonicalise breadcrumb on both envelopes: closed via 6 CLI parity tests (`ac_04_*`)
- ac-05 — validation against finetype: closed via this progress.md, capturing the full migration (54 rewrote, 0 parse_failed, verify clean)
- ac-06 — choice 0023 doc updates: pending (final move before close)
- ac-07 (gate) — registry-shape extensions: closed via 6 mechanism-level inline tests (`ac_07_*`)
- ac-08 (gate) — Spec.status synthesiser: closed via 2 unit tests (`ac_08_*`)
- ac-09 (gate) — criterion → description rename: closed via unit test `ac_09_*`

Test count delta:
- Reconcile inline tests: 27 → 39 (+12)
- CLI parity tests: +6 (`ac_04_*`)
- Workspace total: from session-start baseline to current (run before final close)
