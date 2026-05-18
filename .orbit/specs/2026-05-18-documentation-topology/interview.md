# Design: Documentation topology

**Date:** 2026-05-18
**Interviewer:** Claude (Opus 4.7)
**Card:** .orbit/cards/0040-documentation-topology.yaml
**Mode:** open

---

## What good looks like

When I — or any agent in my orbit-using repos — investigate how a subsystem works, I can find its canonical sources in one file-read rather than extrapolating from a single docstring or grepping five places. A subsystem-keyed index points to the authoritative code, the decision record, the operational doc, and the test surface for each subsystem; it carries no content of its own, so it stays correct when those sources update. A posture line in CLAUDE.md or METHOD.md makes reaching for the index the default — substrate beats extrapolation. Together they make architecture-level investigation as cheap as `/orb:code-investigate` already makes file-level investigation, and the index accretes as the codebase does — substrate that compounds rather than rots.

---

## Context

Card: *Documentation topology — architecture-level investigation discipline made cheap* — 7 scenarios, goal: agents investigating any architectural question find the canonical sources within one file-read; the index stays correct without duplicating content because it carries only pointers.

Prior specs: 0 — this is the first spec against card 0040.

Gap: the entire capability surface — distribution shape (orbit-shipped vs convention), skill scope (read/write/audit), update-trigger heuristic, audit strictness, doc location, and conformance integration are all undetermined.

Cluster siblings shipped: card 0025-codebase-mastery + `/orb:code-investigate` skill (file-level analogue). Cluster siblings drafted: 0037, 0038, 0039. Paired navigation surface: card 0033-see-the-tree (orbit-substrate side).

## Q&A

### Q1: Distribution shape
**Q:** Where does the topology capability live — orbit-owned substrate, consumer convention, or hybrid (template + skill)?
**A:** The capability is a skill, but the docs folder lives in the repo. The `.orbit/` directory keeps a configuration pointing at where docs are stored in the repo — so the doc itself is consumer-owned content at a consumer-chosen path, while orbit knows about it via config and can conformance-check it. The conformance check stays in play through the config pointer.

### Q2: Skill scope
**Q:** What does the skill cover — read-only, read+write, read+write+audit, or read+write+audit+gate?
**A:** Read, write, and audit, with a heuristic to update anytime something new is learned. The skill is the substrate side; the heuristic is the behavioural pair that makes the reach default.

### Q3: Update-trigger heuristic
**Q:** Where does the "update anytime something new is learned" trigger fire — PreToolUse hook, skill prose in /orb:distill/design/spec, session-prime, or on distill/memory remember?
**A:** Two surfaces: (1) on `/orb:distill` completion and on `orbit memory remember` (with an architecture-flavoured label), and (2) `orbit session prime` surfaces topology drift / freshly-touched subsystems at session boundaries. No PreToolUse hook (would be noisy); no prose nudge in design/spec (wrong moment).

### Q4: Audit strictness
**Q:** When the audit catches drift, what's the consequence — report-only, warn at gate moments, or block at gate moments?
**A:** Warn at gate moments. `spec.close`, `release`, `session prime` surface drift as a warning in their output. Pressure without ceremony — doesn't block, but the noise floor makes it hard to ignore.

---

## Summary

### Goal

Agents investigating any architectural question find the canonical sources within one file-read. The substrate-side index (a single pointer-only doc, subsystem-keyed) is matched by a behavioural posture (CLAUDE.md / METHOD.md line) and a maintenance loop (skill-driven scaffolding + audit, fired at learning moments and surfaced at session/gate boundaries).

### Constraints

- **Doc is consumer-owned content at a consumer-chosen path.** Orbit ships no canonical topology doc — it ships the skill, the config-pointer mechanism, and the conformance check that reads the pointer.
- **No PreToolUse hook for updates.** Update triggers fire at learning moments (distill, memory) and at session boundaries, not on every code edit.
- **Audit warns, never blocks.** `spec.close`, `release`, and `session prime` surface drift but proceed. Hard gates would create false-positive friction.
- **The skill must be cheap.** Modelled on `/orb:code-investigate`: invocation cost is low enough that agents actually reach for it.
- **Pointer-only doc shape.** The topology index references canonical sources by path; it never duplicates content. Updates to those sources do not require updates to the index (only structural changes do).

### Success Criteria

- A consumer repo can `/orb:setup` and end up with a `.orbit/config.yaml` (or equivalent) entry naming a topology doc location, a posture line in CLAUDE.md / METHOD.md, and a `/orb:topology` skill that reads/writes/audits against that path.
- `/orb:distill` completion and `orbit memory remember --label architecture` (or equivalent) fire the update prompt.
- `orbit session prime` includes a "topology drift" surface alongside the existing open-specs / recent-memories / next-step fields.
- `spec.close` and `release` emit a non-blocking warning when drift exists in subsystems the spec or release touched.
- The conformance check (card 0039) handles the topology doc via the config pointer, not by hard-coding its location.

### Decisions Surfaced

These warrant MADR choice files during or after `/orb:spec`:

1. **Topology doc is consumer-content, not orbit-canonical.** Decoupled via a `.orbit/` config pointer. Chose consumer-content over orbit-canonical because the doc's value depends on subsystem-specific prose that orbit can't author. Conformance check reads the pointer rather than knowing the path.
2. **Skill scope is read + write + audit with continuous-update heuristic.** Chose this over read-only (insufficient — discipline rots without authoring support) and gate-strength (excessive — false-positive friction at spec.close).
3. **Update triggers are learning-moments + session boundaries.** Chose distill / memory-remember / session-prime over PreToolUse hook (signal-to-noise) and design/spec prose nudges (wrong moments — those are intent-shaping, not subsystem-learning moments).
4. **Audit strictness is warn-at-gate-moments.** Chose middle path over report-only (too easy to ignore) and block-at-gate-moments (too much friction). Pressure without ceremony.

### Implementation Notes

Means-level leads for the implementing agent — not author-facing decisions.

- **Skill location:** `plugins/orb/skills/topology/SKILL.md` (proposed name `/orb:topology`; alternatives `/orb:doc-topology`, `/orb:architecture` — final name during spec).
- **Default doc location:** memo suggests `docs/topology.md`; consumer repos can override via config. `.orbit/topology.md` is an alternative (keeps the doc in the substrate folder); repo-root rejected (clutters the top of the tree).
- **Config shape:** add `docs.topology` (or similar) key to `.orbit/config.yaml` if one exists, or introduce a config file as part of this spec. Card 0039's conformance check reads the same config.
- **Memory-trigger label:** convention parallel to `--label code-investigate` (established in METHOD.md). Candidate labels: `architecture`, `topology`, `subsystem`. Final choice during spec.
- **Posture line:** lives in METHOD.md (orbit-canonical) — propagates to consumer repos via the existing /orb:setup byte-compare mechanism. Draft phrasing from the memo: *"Before reasoning about how a subsystem works, grep the code tree and `docs/` for it. Substrate beats extrapolation."*
- **Session-prime drift surface:** extend the existing envelope (item_bound / memories / next_step / open_specs) with a `topology_drift` or `subsystems_touched` field. Output shape per memory `spec-verification-against-real-envelope-shape` — verify the actual envelope before writing AC verification lines.
- **Audit verb:** likely `orbit audit topology` (parallel to `orbit audit drift`). Implementation reads the config pointer, walks the topology doc, cross-checks pointer targets exist, and reports drift to stdout in the standard envelope shape.
- **Warning emission at gate moments:** `spec.close` and `release` already structure their output envelopes — add a `topology_warnings` field that lists drift items. Non-blocking by spec; warns by surface.
- **Cluster-fit AC:** add an AC referencing the cluster — this card is the architecture-level analogue of `/orb:code-investigate`, so the spec should explicitly state the parallel and follow the same skill shape (token-frugal default, session-state surface, etc.).

### Open Questions

- **Conformance-check ordering vs card 0039.** Card 0039 is drafted (currently in queue for /orb:design). If this spec ships before 0039 is designed, it informs 0039's conformance-check surface. If 0039 is designed first, this spec consumes whatever conformance contract 0039 establishes. Default plan: this spec ships first and defines the config-pointer pattern; 0039's design references it.
- **Cluster synthesis card.** The memo flagged this as "worth watching whether a synthesis card surfaces; not opening one preemptively". Five concrete instances now (0037, 0038, 0040, /orb:code-investigate, autonomy-too-ready-to-halt memo). Not opening here.

