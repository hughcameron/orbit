---
date: 2026-05-24
interviewer: claude
card: .orbit/cards/0017-setup-is-orbit-state-aware.yaml
mode: rally
rally: 2026-05-24-brownfield-migration-hardening-rally
---

# Design: setup is orbit-state-aware — undotted-substrate state, topology seed scoping, decisions/ move-and-warn

## What good looks like

A brownfield repo with `orbit/` (no dot) wrapping cards/choices/specs/memos lands cleanly on `.orbit/` in one `/orb:setup` run — substrate becomes tool-visible, history preserved via `git mv`, no parallel folders left behind. Setup running in a non-plugin project does not seed substrate-typed topology entries that misrepresent the project's architecture. A repo carrying MADR markdown under `decisions/` migrates the folder into `.orbit/decisions/` and surfaces a durable signal that content needs hand-conversion — the operator sees the limitation immediately and the conformance audit catches it next session.

---

## Context

Card: *setup-is-orbit-state-aware* — 9 scenarios, maturity planned, goal: `/orb:setup` is the canonical orbit-state-init gate for greenfield and brownfield projects in one atomic pass.

Prior specs: 1 — `2026-05-09-orbit-method-md` shipped the METHOD.md canonical pipeline plus the byte-compare-and-prompt mechanism.

Gap, from `.orbit/memos/2026-05-24-brownfield-setup-friction.md` (items #2 and #4) plus the arcform brownfield session: setup's 4-state table (`plugins/orb/skills/setup/SKILL.md:24-31`) enumerates greenfield / brownfield-bare / mixed / idempotent — no state for "`orbit/` (no dot) wrapper present, `.orbit/` absent". A downstream project (arcform) had exactly this layout; the agent had to `git mv` manually. Setup also auto-seeds topology entries (`cards`/`choices`/`memories`/`specs-substrate`/`topology`) that only make conceptual sense in the orbit-plugin repo, producing 21 stale-pointer drift findings on any non-plugin project. Setup's brownfield prompt names `decisions/   → .orbit/choices/` but the migration is currently a folder rename only — MADR markdown does not parse as YAML, so the post-migration tree has invisible MADR content.

Cross-drive: the sister drive on card 0039 ships an `undotted_substrate` conformance finding that suppresses canonical-files-missing — this drive's design assumes that suppression exists.

---

## Q&A

### Q1: Name of the 5th setup state
**Q:** What slug does the new state take, given it appears in conformance findings, SKILL.md prose, and CLI output?
**A:** `wrapped-undotted`. Structural classifier, composes with existing greenfield/brownfield/mixed/idempotent vocabulary. Avoids the "legacy" overload already used for CLAUDE.md blocks and reconcile sidecars.

### Q2: `orbit/ → .orbit/` migration mechanism
**Q:** How is the rename performed — single `git mv`, per-subdir mirror of the bare-brownfield pattern, or copy-and-remove?
**A:** Single `git mv orbit .orbit`. Preserves history cleanly, makes untracked-residue handling a non-issue (whole dir moves), fails fast on `mixed-undotted` because the target already exists. The §3b residue scan in SKILL.md is skipped on this path.

### Q3: Detector location
**Q:** Where does the new state's detection logic live — SKILL.md bash, a new Rust verb in orbit-state, or both?
**A:** New detector in `orbit-state`, called from both the setup skill and the conformance audit. Same direction as choice 0020 (`shell-scripts-to-rust-verbs`). The sister drive's `undotted_substrate` finding shares the same predicate via this verb — single source of truth, no skill-vs-audit drift.

### Q4: Topology auto-seed scoping
**Q:** How does setup recognise "this is the orbit-plugin repo" so substrate-typed seed entries fire there but not elsewhere?
**A:** Explicit `plugin_repo: true` config flag in `.orbit/config.yaml`, default `false`. Setup in non-plugin projects scaffolds an empty `.orbit/topology/` with a one-line README pointing at `/orb:topology` for opt-in entries. Setup in the orbit-plugin repo (where the flag is set) seeds the 5 substrate-typed entries. Secondary validate step: refuse to write any seed whose `canonical_code` path does not exist in the working tree.

### Q5: `decisions/` handling
**Q:** Locked upstream is detect-and-warn (no auto-conversion). What does the emission shape look like?
**A:** Move-and-warn:
- During brownfield migration (both `brownfield-bare` and `wrapped-undotted` paths), `decisions/` is `git mv`-ed to `.orbit/decisions/` — folder rename only, no content conversion.
- Setup prints an inline one-paragraph warning naming the unmigrated content and the conversion task.
- SKILL.md migration-prompt example updated from `decisions/   → .orbit/choices/` to `decisions/   → .orbit/decisions/  (MADR files; manual MD→YAML conversion needed)`.
- New conformance finding family `decisions-md-unmigrated` fires on presence of `.orbit/decisions/` with `.md` content and no matching `.orbit/choices/<slug>.yaml`. Remediation pointer is a docs page or future `orbit choice import-madr` verb. **This finding family ships in this drive (cross-drive decision locked by author).**

### Q6: State-machine table shape
**Q:** How does the table represent the new state — flat enumeration, prose flowchart, or two tables?
**A:** Flat 6-row table enumerating all reachable combinations:

| State | Condition |
|-------|-----------|
| `greenfield` | none of `.orbit/`, `orbit/`, or bare dirs present |
| `idempotent` | `.orbit/` present, neither `orbit/` nor bare dirs present |
| `brownfield-bare` | bare dirs present, neither `.orbit/` nor `orbit/` |
| `wrapped-undotted` | `orbit/` present, `.orbit/` absent, no bare dirs |
| `mixed-bare` | `.orbit/` present AND bare dirs present (refuse) |
| `mixed-undotted` | `.orbit/` present AND `orbit/` present (refuse) |

Renames existing `brownfield` → `brownfield-bare` and `mixed` → `mixed-bare` for parallelism. SKILL.md description line (`creates orbit/ directory`) corrected to `creates .orbit/ directory` incidentally.

---

## Summary

### Goal

`/orb:setup` recognises and migrates the `wrapped-undotted` brownfield state — `orbit/` (no dot) substrate moves to `.orbit/` via a single `git mv`, history-preserving and atomic. Setup detection logic lives in `orbit-state` as a Rust verb shared by skill and audit. Topology auto-seeding is gated by an explicit `plugin_repo: true` config flag; non-plugin projects get an empty topology dir + README. `decisions/` migrates into `.orbit/decisions/` (folder rename only) with an inline warning and a new `decisions-md-unmigrated` conformance finding family for durability.

### Constraints

- The `wrapped-undotted` state appears in the SKILL.md table at a defined position with a single condition (`orbit/` present, `.orbit/` absent, no bare dirs).
- The migration mechanism is `git mv orbit .orbit` — single rename, not per-subdir.
- Detection lives in `orbit-state` (not bash). Called from setup skill AND `orbit audit conformance`. No duplicate predicate.
- Topology auto-seed fires only when `.orbit/config.yaml` has `plugin_repo: true`. Default false; this repo (orbit-plugin) sets it true as a one-line edit shipped in this spec.
- `decisions/` is `git mv`-ed to `.orbit/decisions/` (not `.orbit/choices/`) — directory rename honest about content shape.
- The new `decisions-md-unmigrated` conformance finding family ships in this drive (cross-drive decision; Drive B's scope stays focused on `undotted_substrate`).
- The SKILL.md description line `creates orbit/ directory` is corrected to `creates .orbit/ directory`.

### Success Criteria

- A brownfield repo with `orbit/cards/`, `orbit/choices/`, `orbit/specs/` populated and `.orbit/` absent runs `/orb:setup` once and produces a canonical `.orbit/` layout with all substrate visible to `orbit card list` and friends. `git mv` preserves history; `git log --follow` on any moved file traces back to its `orbit/` ancestor.
- The same run, performed against a project that does NOT have `plugin_repo: true` in `.orbit/config.yaml`, scaffolds `.orbit/topology/` empty (with README). No substrate-typed seed entries are written. `orbit audit topology` reports `configured: false, drift: []`.
- The same run performed against the orbit-plugin repo (where `plugin_repo: true` is set in config) seeds the 5 substrate-typed entries as today. Topology audit reports `configured: true`.
- A brownfield repo carrying `decisions/<NNNN>-<slug>.md` MADR files runs `/orb:setup` once, ending with `.orbit/decisions/` containing the same files (folder rename only) and an inline warning naming the MD→YAML conversion. `orbit audit conformance` reports a `decisions-md-unmigrated` finding for the unconverted content.
- The SKILL.md state table at `plugins/orb/skills/setup/SKILL.md:24-31` is the 6-row form above. The `wrapped-undotted` row is present; `brownfield-bare` and `mixed-bare` replace the prior names.
- The orbit-state Rust verb implementing the detector is callable from CLI and produces identical classification when called from the setup skill's bash prelude. Parity test in `crates/cli/tests/` covers the wrapped-undotted case.
- The new `plugin_repo: true` flag is set in this repo's `.orbit/config.yaml` as part of the implementation.

### Decisions Surfaced

- **Single-rename migration over per-subdir.** `git mv orbit .orbit` is the structurally honest move; the dir name is the only thing wrong. Per-subdir mirror of bare-brownfield was rejected because it strands untracked residue inside `orbit/`. (Q2)
- **Detector in orbit-state, shared between skill and audit.** Avoids the drift class the seed memo names ("setup says one thing, audit says another"). Single test surface. Matches choice 0020's direction. (Q3)
- **Explicit `plugin_repo: true` flag over filesystem heuristic.** Categorical-error prevention deserves an explicit signal — a heuristic on path presence could misfire and reintroduce the seed problem in a non-plugin repo. One-time cost (set flag on plugin repo); safety property holds forever. (Q4)
- **Move-and-warn for decisions/ with new conformance finding family.** Composes with the locked "no auto-conversion" decision without leaving the substrate in the broken half-state of the arcform session. The new finding family `decisions-md-unmigrated` ships in this drive (cross-drive locked decision). (Q5)
- **Flat 6-row state table over prose flowchart.** Exhaustiveness is scannable; the "no other state" claim becomes a sentence the reader can verify by counting rows. (Q6)

### Implementation Notes

**Codebase leads:**

- Setup skill: `plugins/orb/skills/setup/SKILL.md` — state table at §1 (lines 22-31), brownfield migration block at §3 (lines 48-98). Description line at line 3.
- Topology seed code: `orbit-state/crates/core/src/verbs.rs:4451-4532` — the substrate-typed entries that this drive gates behind `plugin_repo: true`.
- Conformance audit: `orbit-state/crates/core/src/verbs.rs::audit_conformance_at` (~line 3902) — the new `decisions-md-unmigrated` finding family fires from here.
- Setup helper script: `plugins/orb/scripts/setup-method.sh` — handles the canonical-files step; needs no change for the new state but may surface the warning text.

**New orbit-state surface:**

- New Rust function `classify_substrate_layout(layout) -> SubstrateLayoutState` returning the 6-variant enum. Lives in `orbit-state/crates/core/src/verbs.rs` next to `audit_conformance_at`. Called from both the conformance audit's `undotted_substrate_finding` (Drive B) and the setup skill's bash prelude (via a thin `orbit audit substrate-layout` CLI verb or similar).
- New `decisions-md-unmigrated` finding builder, sibling to `canonical_file_findings` (`verbs.rs:4087`).
- `OrbitConfig` schema (file at `.orbit/config.yaml`) gains an optional `plugin_repo: bool` field, default `false`.

**SKILL.md edits:**

- Line 3 description: `creates orbit/ directory` → `creates .orbit/ directory`.
- §1 state table (lines 24-31): replaced with the 6-row form above.
- §3 brownfield migration block (lines 48-98): extended to handle `wrapped-undotted` via the single-rename path; existing bare-dir path preserved.
- Migration prompt example (line 62): `decisions/   → .orbit/choices/` → `decisions/   → .orbit/decisions/  (MADR files; manual MD→YAML conversion needed)`.
- §6d topology scaffolding section: gated on `plugin_repo` flag.

**Repo-self update:**

- `.orbit/config.yaml` in this (orbit-plugin) repo gains `plugin_repo: true` as part of this drive's implementation. One-line edit, ensures setup in this repo continues to seed the substrate-typed topology entries.

**Memory dispositions:**

- `private-projects-genericised-in-artefacts` — *not applicable*. arcform is in the meridian-online family (user confirmation 2026-05-24); spec text may name it.
- `audit-conformance-cwd-dependent` — *adopted procedurally*. Tests for the new detector use absolute paths or `--root` to avoid the cwd-dependent footgun.

**Out of scope:**

- MADR markdown → YAML choice conversion. Locked out by rally proposal — the finding family is the placeholder.
- A general `orbit choice import-madr` verb. Future spec against card 0017 or 0030 (canonical-schema-and-glossary).
