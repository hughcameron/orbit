# Tabletop Note: Port promote.sh to native `orbit spec promote` verb

**Date:** 2026-05-25
**Cards:** .orbit/cards/0005-drive.yaml, .orbit/cards/0006-rally.yaml, .orbit/cards/0020-orbit-state.yaml
**Mode:** closed
**Choice:** .orbit/choices/0020-shell-scripts-to-rust-verbs.yaml — substrate-shaped shell scripts under `plugins/orb/scripts/` migrate into orbit Rust verbs

---

## What good looks like

Drive's promote stage and rally's fan-out stop shelling out to a 197-line python3-in-bash script that re-implements card parsing, spec id derivation, and scenario→AC fanout in shell. The work lands as a single `orbit spec promote <card-path>` verb that does the read-card, derive-id, create-spec, populate-acceptance_criteria, canonicalise pipeline server-side with `Error::conflict` if the target spec already exists. The shim becomes a thin compat wrapper, survives until `rg promote\.sh` against every consumer SKILL.md returns zero, then the shim and its dedicated test delete in the same commit. Cards 0005 and 0006 gain the `relations:respects → choice 0020` edges the policy's Consequences clause has been promising since 2026-05-09.

## Pinned approach

- Choice 0020 names the direction: substrate-shaped scripts under `plugins/orb/scripts/` port to `orbit-state` Rust verbs as the default; per-script migration sequenced opportunistically, each in its own spec citing the choice.
- Direct precedent: spec `2026-05-24-port-acceptance-shim` (shipped yesterday as v0.4.34, PR #32) ported the AC-traversal shim into six native verbs on `orbit spec` with the same shape — new verb impl + CLI surface + MCP tool descriptor + CLI/MCP parity tests + same-commit SKILL.md rewrite + shim deletion + choice 0020 table update + CHANGELOG entry. Mechanical to mirror.
- Decommission discipline per choice 0020's "Test-vs-prod parity" clause: the shell shim stays in place until grep on every consumer SKILL.md returns clean; the shim and its tests then delete in one commit with the last call-site rewrite. The port-acceptance-shim drive took the wrapper-skip path (single PR with verbs ship + rewrite + delete); same shape recommended here.

## Deferred items

- `setup-method.sh` port — third opportunistic spec per choice 0020's enumeration (1 SKILL.md call site but introduces a new `orbit setup` verb family with interactive drift/legacy prompts; bundles the edge write on card 0017).
- `code-investigate-mark.sh` discrimination — a `/orb:choice` decision before any port; the script post-dates choice 0020 and sits on the substrate-vs-tooling boundary (session-scoped marker file, not canonical substrate).
- Migration of `plugins/orb/scripts/tests/test-promote-gate-propagation.sh` and any other promote-touching test files to the Rust suite — moves with the shim's deletion per choice 0020's test-vs-prod parity rule; no separate test-migration spec.
- Card-id resolution on the verb (accepting `0005` / `0005-drive` / `.orbit/cards/0005-drive.yaml` like `card.show` does) — v1 takes a path matching the shim's contract literally; add slug/short-id resolution in a follow-up if a consumer needs it.
- Auto-promotion-from-card-binding (`orbit spec promote` with no arg, reads `.orbit/.session-card`) — defer; the verb is explicit-path-only at v1, matching the shim.

## Implementation notes

- **Verb surface — single verb (no fork worth flagging).** `orbit spec promote <card-path> [--dry-run]` maps cleanly to one verb `spec.promote` with `SpecPromoteArgs { card_path: String, dry_run: bool }`. The dry-run flag belongs on the core arg struct (not just the CLI) so MCP callers can preview without writing — matches the `spec_close --force` pattern at `crates/core/src/verbs.rs:269-278`. Returns `SpecPromoteResult { spec: Spec, dry_run: bool }` — the full freshly-created spec when actually written; the planned spec (status `open`, AC list populated, `id` set) when `dry_run: true`. Same struct, different write-side-effect.
- **Implementation can reuse the existing pieces.** Card parse lands in `orbit-state/crates/core/src/canonical.rs` via the existing `parse_yaml::<Card>(&text)` helper — no new YAML machinery. Spec id derivation (`<today's-iso-date>-<card-slug-without-NNNN-prefix>`) is a 5-line function — put it inline. The create→write-ACs→canonicalise pipeline is `spec_create` (existing) + the AC list write (mirror `flip_ac_checked`'s read-mutate-write at `verbs.rs:2655+` but writing the AC list wholesale, not flipping one flag) + the canonical-writer-fixes-byte-drift hand-off (no separate `canonicalise` call needed if `serialise_yaml` is used end-to-end — verify).
- **Idempotency.** If a spec at the derived id already exists, error `Error::conflict("spec.promote", "spec '<id>' already exists; promote produces fresh specs")`. Matches the shim's behaviour today (the shim's `orbit spec create` call errors on existing id; my verb propagates that with a verb-specific error message). The dry-run path stays read-only — it can dry-run a promotion of a card whose existing spec is already on disk without erroring, since no write happens.
- **Card-path validation.** Verb takes a string path; reject paths with `..` or absolute paths outside the layout root via the existing `validate_*` helpers (mirror `spec_show`'s id-validation rejections). The shim takes an absolute or relative path and `cd`s to it; the verb should accept either relative-to-layout-root or absolute, normalising to absolute internally.
- **Stdout contract.** The shim's stdout contract is a bare spec id (one line, no trailing whitespace) — drive's `SPEC_ID=$(... promote.sh ...)` depends on it. The Rust verb returns the structured envelope; the CLI's default-mode renderer for `spec.promote` must emit the spec id alone on stdout (no labels, no formatting) so the existing call-site shape survives the migration window even before SKILL.md is rewritten. The `--json` mode emits the full envelope as usual.
- **Source-of-truth lifts:** `orbit-state/crates/core/src/verbs.rs::spec_create` (line ~2110 — the existing creator), `spec_check` / `spec_uncheck` (line ~2625 — the read-mutate-write pattern for AC list writes from yesterday's port), `error.rs` (`Error::not_found` / `Error::conflict` shapes). The shim's logic to port lives at `plugins/orb/scripts/promote.sh` lines 60-200 (the python3 heredocs that parse the card + write the acceptance_criteria).
- **Recommended `ac_type` distribution** for the spec:
  - `code` — the `spec.promote` core verb, CLI subcommand wiring + bare-spec-id stdout renderer, MCP tool descriptor, CLI/MCP parity tests (fixture: a card with mixed gate / non-gate scenarios), `--dry-run` flag round-trip test.
  - `code` — SKILL.md call-site rewrites verified by the `rg ... | wc -l == 0` check (5 SKILL.md files implicated: drive 3, rally 1, card 2 — confirm exact list during implement).
  - `doc` — shim + test deletion commit, choice 0020 table update (mark `promote.sh` row migrated), cards 0005 and 0006 relations-edge writes (`relations:respects → choice 0020`), CHANGELOG entry under the bumped version.
  - No `ops` or `observation` ACs expected — internally verifiable end-to-end like yesterday's port.
- **Bundled-with-the-drive edge writes (per choice 0020 Consequences):** cards 0005-drive.yaml and 0006-rally.yaml gain `relations:` entries `{ choice: '0020-shell-scripts-to-rust-verbs', type: respects, reason: drive/rally's promote stage consumes the Rust verb that this choice authorised the migration to }`. Check the existing `Relation` schema variants — may need a `choice:` (not `card:`) relation type added to `crates/core/src/schema.rs` if relations are currently card-only.
