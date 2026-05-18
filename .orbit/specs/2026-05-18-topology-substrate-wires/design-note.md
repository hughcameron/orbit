# Design Note: Topology substrate wires

**Date:** 2026-05-18
**Card:** .orbit/cards/0040-documentation-topology.yaml
**Mode:** closed
**Pinned by:** `.orbit/specs/2026-05-18-documentation-topology/` — parent spec's `interview.md` carries the design intent contract; ACs 07/08/09/12/13 left unchecked at parent close (partial drive, 8/13 ACs shipped per Hugh's "continue through ac-06 only" decision). Follow-on scope memo: `.orbit/memos/2026-05-18-topology-follow-on.md`.

---

## What good looks like

In a topology-configured repo the substrate maintains itself around the agent. `/orb:setup` scaffolds the config + stub on the first project boot, so the capability is alive from day one without explicit wiring. `orbit session prime` surfaces topology drift in its envelope at session start — no need to remember to run an audit; the noise floor reaches the agent. When the agent closes a spec that touched a documented subsystem, `spec.close` flags drift in the closing envelope as a non-blocking nudge — pressure without ceremony. When the agent runs `orbit memory remember --label topology`, the verb's response nudges toward `/orb:topology` so the index gets updated at the learning moment, not at the next gate. Four weeks in, an observation audit reads usage data + drift counts and tells the operator whether the loop is working or needs tightening.

## Pinned approach

- **Substrate already shipped by parent spec.** `/orb:topology` skill (write / read / audit) at `plugins/orb/skills/topology/SKILL.md`; `Config + DocsConfig` schemas with `FIELDS` consts in `orbit-state/crates/core/src/schema.rs`; `layout.config_file()` and `verify_all` Config branch; METHOD.md posture line *"Substrate beats extrapolation"* + `--label topology` convention; `orbit audit topology` verb on CLI + MCP with topology doc parser + three drift categories (`stale_pointer`, `missing_entry`, `shape_drift`); symmetric exit-0 contract with `orbit audit drift`. `TopologyDriftEntry` type already lives in verbs.rs and is shared across the new envelope extensions. 306 / 306 tests pass at parent close.
- **Five follow-on ACs are pure envelope / UX surface.** ac-07 (`/orb:setup` integration — greenfield scaffold + brownfield-accept), ac-08 (session prime envelope extension), ac-09 (spec.close `topology_warnings` + word-boundary heuristic), ac-12 (memory.remember `--label topology` nudge), ac-13 (4-week observation audit). All four work-ACs consume the parent spec's substrate; none introduces new schema or new verbs.
- **Cycle-2 review LOWs carried forward.** (1) `regex::escape` on subsystem name before `\b<subsystem>\b` interpolation in ac-09 — careful implementer gets this by default but flag it explicitly. (2) Reuse the existing `TopologyDriftEntry` shape across ac-08 / ac-09 — DRY against the type already defined in verbs.rs, do not redeclare. (3) Cycle-2 reviewer accepted ac-02's bundling (struct + FIELDS const + drift test in one AC) for the parent spec but flagged it; carry the same lens to ac-07 (setup skill prose + scaffold script in one AC) — keep bundled unless implementing agent prefers split.

## Deferred items

- **`topology.suppress` array on Config** — parent spec's ac-09 description names a per-subsystem suppression follow-on. Out of scope here; needs a separate spec when noise actually warrants it.
- **Subsystem-detection heuristic refinement** — parent spec's ac-06 uses "top-level directory under `src/` or `crates/`" as the missing-entry heuristic. May need broadening (`lib/`, `app/`) once consumer-repo adoption surfaces patterns.
- **Cluster synthesis card** — card 0040 sits in the agent-side substrate-engagement cluster with 0025 / 0037 / 0038 + the autonomy-too-ready-to-halt memo. No synthesis trigger yet; this follow-on adds one more concrete instance, not a synthesis trigger.

## Implementation notes

Per-AC `ac_type` declarations (per spec 2026-05-16-ac-taxonomy):

- **ac-07 — `/orb:setup` integration.** `ac_type: code`. Surfaces: `plugins/orb/skills/setup/SKILL.md` (skill prose, byte-compare-and-prompt voice per existing §6b convention) + setup shell script(s) (scaffold logic for `.orbit/config.yaml` with `docs.topology: docs/topology.md` + stub `docs/topology.md` with heading + one-paragraph explainer + empty entry list). Brownfield-accept rule explicit: if `docs.topology` target path does not exist on disk, ALSO create the stub (suppresses first-prime drift noise); if it exists, wire the pointer, do NOT overwrite.
- **ac-08 — session prime envelope extension.** `ac_type: code`. Surfaces: `SessionPrimeResult` in `orbit-state/crates/core/src/verbs.rs` (add `topology_drift` field — type already exists from `audit topology`); `session_prime` function (call `audit_topology` internally when configured); parity test on CLI + MCP for all three states (configured-clean / configured-drift / not-configured). Skip-on-default contract: when `.orbit/config.yaml` is absent or `docs.topology` not set, the key is omitted entirely from the envelope (not an empty array — confirmed in parent spec's ac-08 verification).
- **ac-09 — spec.close `topology_warnings`.** `ac_type: code`. Surfaces: `SpecCloseResult` in `verbs.rs` (add `topology_warnings` field); `spec_close` function (load topology entries via existing `audit_topology` code path, apply word-boundary heuristic against concatenated `spec.yaml` + `interview.md` text). Heuristic: case-insensitive `\b<regex::escape(subsystem)>\b` for subsystems with names ≥ 5 chars; short-name filter excludes false-positives on common tokens (memo, spec, ac). Non-blocking — closure proceeds exit 0, warnings appear in the ok envelope's `topology_warnings` array. Parity test on CLI + MCP.
- **ac-12 — memory.remember `--label topology` nudge.** `ac_type: code`. Surfaces: `MemoryRememberResult` in `verbs.rs` (add `nudge: Option<String>` advisory field — envelope-side is cleaner for MCP than stderr emission); `memory_remember` function (post-store hook checks labels for `"topology"`, populates nudge field); CLI renders the nudge in human mode; add `--no-nudge` flag (final name decided during implementation). Parity test on CLI + MCP.
- **ac-13 — 4-week observation audit.** `ac_type: observation`. Anchor: 2026-06-15 (parent spec ship-date 2026-05-18 + 4 weeks). Audit reads (a) topology doc entry count + update frequency by subsystem; (b) memories labelled `topology` over the window; (c) `topology_warnings` counts from `spec.close`; (d) `topology_drift` counts from `session prime`. Output: memo at `.orbit/memos/2026-06-15-topology-4-week-audit.md` with data tables + recommendation. `spec.close` defers this AC per ac-taxonomy observation band (does not block close).

Implementation sequencing: ac-08 → ac-09 → ac-12 can land in any order (each is a `verbs.rs` envelope add with parity tests). ac-07 is independent of those — it edits the setup skill + scaffold script. ac-13 is a 2026-06-15 observation; do not implement now.
