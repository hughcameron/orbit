# Design Note: Topology substrate-folder migration

**Date:** 2026-05-18
**Card:** .orbit/cards/0040-documentation-topology.yaml
**Mode:** closed
**Choice:** .orbit/choices/0025-topology-substrate-folder.yaml — Topology lives in .orbit/topology/ substrate folder

---

## What good looks like

When I open a fresh repo and run `/orb:setup`, topology is wired as a substrate folder under `.orbit/topology/` rather than a markdown doc under `docs/`. When an agent writes a topology entry for a subsystem, it edits one yaml file that file alone — not a section of a thousand-line doc. When a subsystem becomes stale, I or an agent prunes it with `rm`. When the topology audit verb checks for drift, it reads structured yaml against a declared schema — missing fields and dangling pointers surface as concrete findings, not heuristic hits on markdown shape. The session-prime envelope and spec-close warnings continue to fire against the new parser; the substrate-engagement machinery shipped today survives the file-shape rewire. The topology surface looks and behaves like every other `.orbit/` substrate (cards, choices, specs) — one file per entity, parsed and verified by the same machinery.

## Pinned approach

- **Layout pinned by choice 0025.** Per-subsystem yaml files at `.orbit/topology/<subsystem>.yaml`; same structural shape as `.orbit/cards/<id>.yaml`, `.orbit/choices/<id>.yaml`, `.orbit/specs/<id>/spec.yaml`. Parsed and verified by existing orbit-state machinery.
- **Substrate-engagement machinery preserved.** Session-prime `topology_drift` envelope (per spec 2026-05-18-topology-substrate-wires ac-02), spec-close `topology_warnings` envelope (ac-03), `--no-nudge` memory-remember flag (ac-04), and /orb:setup §6d scaffolding (ac-01) all stay — only the parser they call into changes from markdown-heuristic to per-file-yaml-structural.
- **Prior specs are supersession context, not predecessors.** `.orbit/specs/2026-05-18-documentation-topology/spec.yaml` and `.orbit/specs/2026-05-18-topology-substrate-wires/spec.yaml` shipped the markdown-doc shape and the envelope wiring respectively. This migration reverses the former and preserves the latter.

## Proposed ACs

Recommended AC set for the spec (`/orb:spec` finalises). All `code` unless noted; the migration is mostly substrate-Rust + skill-prose + bash:

1. **Schema + layout** (`code`). `TopologyEntry` serde struct + `FIELDS` const in `orbit-state/crates/core/src/schema.rs`; `Layout::topology_dir()` + scanner in `layout.rs`; schema tests; `verify_all` extension covering the new directory.
2. **Parser swap** (`code`). `audit_topology` in `orbit-state/crates/core/src/verbs.rs` switches from markdown header/list scanning to per-file yaml parse; drift detection surfaces structural findings (`missing_field`, `unknown_field`, `dangling_pointer`) alongside or replacing the current `stale_pointer` / `missing_entry` / `shape_drift` taxonomy; cli + mcp parity tests.
3. **Skill rewrite** (`doc`). `plugins/orb/skills/topology/SKILL.md` write / read / audit modes operate on per-file yamls. Write-mode edits one file; read-mode loads one file by subsystem key; audit-mode invokes the substrate verb.
4. **Wiring rewire** (`code`). `setup-topology.sh` creates `.orbit/topology/` (empty directory or seed file) instead of `docs/topology.md`; `docs.topology` config key drops (convention beats config) or repoints — decided in spec; session-prime and spec-close envelopes verified against the new parser via parity tests.
5. **Migration verb** (`code`). `orbit topology migrate-layout` (idempotent one-shot): reads `docs/topology.md` if present, parses entries with the legacy parser, writes per-subsystem yamls at `.orbit/topology/<subsystem>.yaml`, deletes the source markdown. Safe to re-run; reports per-entry success/skip.

## Deferred items

- **Card 0040 framing edits.** The card's feature name `documentation-topology` is a mild misnomer once the migration ships ("substrate-topology" is closer). Out of scope for this spec; touch the card on next pass.
- **Pre-existing consumer-repo `docs/topology.md` migration.** This repo has one (shipped today); other consumer repos may not. The migration verb handles the present-and-absent cases idempotently, but downstream-consumer roll-out (release notes, upgrade docs) is out of scope.
- **Schema field set.** What exactly is a topology entry — `canonical_code`, `decision_record`, `operational_doc`, `test_surface` per card 0040's framing, plus what else? `/orb:spec` pins the field set; this design note carries the shape but not the field list.
- **Card 0040's `notes:` cleanup.** The two open questions this choice answers (file location, file format) should drop from card 0040's notes on next touch. Bundled with the framing edit above.

## Implementation notes

- **Codebase surfaces touched.** `orbit-state/crates/core/src/schema.rs` (TopologyEntry struct + FIELDS); `layout.rs` (topology_dir method, scanner); `verbs.rs` (audit_topology parser); `verify.rs` (verify_all branch for topology dir); `plugins/orb/skills/topology/SKILL.md` (skill prose); `plugins/orb/scripts/setup-topology.sh` (scaffolding) + `tests/test-setup-topology.sh`; parity tests in `cli/tests/parity.rs` and `mcp/tests/parity.rs`.
- **Existing markdown parser is the migration source.** The current parser in `audit_topology` knows how to read `docs/topology.md`; the migration verb (AC 5) reuses it as the import path before retiring it. Don't delete the markdown parser code until the migration verb is shipped and tested.
- **Substrate-engagement envelope tests are the parity contract.** The `topology_drift` field on `SessionPrimeResult` and the `topology_warnings` field on `SpecCloseResult` must continue to populate with the new parser. Lift the existing tests from spec `2026-05-18-topology-substrate-wires` (ac-02, ac-03); they're the regression net.
- **`docs.topology` config key decision is a one-line spec call.** Two options: drop the key entirely (convention beats config — `.orbit/topology/` is canonical, no pointer needed), or repoint it to `.orbit/topology/` as a directory pointer. Recommendation: drop. Cards/choices/specs/memories have no pointer key; topology shouldn't either. Save the decision in the spec.
- **`/orb:topology` skill modes map cleanly.** Write-mode → `Write` tool against `.orbit/topology/<slug>.yaml`. Read-mode → `Read` tool by subsystem key. Audit-mode → `orbit audit topology` (existing verb, new parser). The three-mode shape survives the rewire.
- **AC ordering.** Land the schema (AC 1) and parser swap (AC 2) before the skill rewrite (AC 3) — the skill calls into both. Wiring (AC 4) follows. Migration verb (AC 5) can land in parallel with AC 3 since it depends only on AC 1.
