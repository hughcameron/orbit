# Design: STYLE.md plugin-shipping

**Date:** 2026-05-20
**Interviewer:** Claude Code (Opus 4.7)
**Card:** .orbit/cards/0026-agent-prose-discipline.yaml
**Mode:** partial

---

## What good looks like

As an orbit author, when I open a session in any consumer project, I want the agent's prose discipline already loaded — directive prose, lead with the answer, no menus, the anti-patterns. Today it works in the orbit repo because STYLE.md is here and CLAUDE.md @-imports it; consumer repos still get METHOD.md's older BLUF / Decision Brief framing, which contradicts the reworked STYLE.md. Ship STYLE.md the same way METHOD.md ships — via `/orb:setup` — so the canonical prose contract is one file available everywhere orbit runs.

---

## Context

Card: *agent-prose-discipline* — 8 scenarios; goal is agent-to-author prose that is directive, plain, anti-pattern-free, warm-but-not-chatty.

Prior specs: 1 — `2026-05-08-executive-communication-wires` wired STYLE.md at the orbit-project level (created the file, @-imported from CLAUDE.md, cited from design / review-spec / review-pr SKILL.md). NOT plugin-shipped — explicitly identified as a HIGH finding in the 2026-05-19 workflow-conformance review, only METHOD.md ships via `/orb:setup` today.

Gap: STYLE.md is orbit-internal. Consumer projects fall back to METHOD.md's BLUF / Decision Brief section, which now contradicts the reworked STYLE.md (rework landed earlier this session — substance-rules retained, format prescription dropped, persona content extracted to project CLAUDE.md). This spec closes the gap by following the established plugin-canonical pattern: vendored in `orbit-state/crates/core/canonical/`, embedded via `include_str!`, written by `/orb:setup`, byte-compared by conformance audit.

## Q&A

### Q1: Pillar #1 rename
**Q:** METHOD.md pillar #1 ("Executive-level interaction") still carries the framing we just retired in STYLE.md. How should it land in this spec?
**A:** Rename in this spec — same surface, single coherent rework.

### Q2: Operator override posture
**Q:** When STYLE.md ships to consumer projects, what's the canonical posture for operators personalising prose discipline?
**A:** Persona in CLAUDE.md — STYLE.md stays canonical; project-specific stance lives in CLAUDE.md as a Persona section (the pattern just shipped in orbit's own CLAUDE.md). Conformance drift on local STYLE.md is real signal.

### Q3: Brownfield migration
**Q:** `/orb:setup` runs idempotently. For existing consumer repos (no STYLE.md yet), should next `/orb:setup` write it silently or announce the new seed?
**A:** Silent seed — match METHOD.md / topology pattern.

### Q4: Cascade scope
**Q:** Plugin SKILL.md files (design, review-spec, review-pr, setup) cite "BLUF / Decision Brief — see card 0026" — both card name and framing now stale. What's in scope for this spec?
**A:** Bundle citations + adjacent drift — fix all SKILL.md citations AND visible Decision-Brief / TL;DR-skeleton prose drift in the same pass.

---

## Summary

### Goal
Ship STYLE.md as a plugin-canonical file so the reworked prose discipline reaches every project that runs `/orb:setup`. Drop METHOD.md's duplicate BLUF / Decision Brief section. Rename METHOD.md pillar #1 to retire the "executive" framing. Update all plugin SKILL.md citations of card 0026 + BLUF / Decision Brief, including adjacent prose drift.

### Constraints
- Follow the existing plugin-canonical pattern: live at `plugins/orb/skills/setup/STYLE.md`, vendored to `orbit-state/crates/core/canonical/STYLE.md`, embedded via `include_str!`, written by `/orb:setup`, byte-compared by `orbit audit conformance`.
- Cross-compile compatibility: vendored under `orbit-state/crates/core/canonical/` so `include_str!` works under cross's docker mount (precedent: METHOD.md's 0.4.21 → 0.4.22 fix).
- Silent seed in `/orb:setup` — no special operator announcement.
- Lockstep release: STYLE.md ships only via a tagged plugin release (next bump).

### Success Criteria
- STYLE.md content reaches consumer projects on next `/orb:setup` run.
- METHOD.md no longer carries the BLUF / Decision Brief prose section.
- METHOD.md pillar #1 reads with new framing.
- Plugin SKILL.md files (design, review-spec, review-pr, setup, plus any others surfaced by grep) no longer cite "BLUF / Decision Brief" framing or the old `0026-executive-communication` slug.
- `orbit audit conformance` clean post-ship (no plugin-canonical-file drift findings on STYLE.md or METHOD.md).
- Operator personalisation guidance points at the CLAUDE.md Persona section pattern.
- Sync-check unit test covers STYLE.md byte-parity (matching METHOD.md test pattern).

### Decisions Surfaced
- **Pillar #1 rename**: in-scope over deferred. Rationale: same surface as STYLE.md rework; deferring leaves a contradictory pillar name in METHOD.md alongside the new STYLE.md.
- **Override posture**: Persona-in-CLAUDE.md as canonical. Rationale: matches the pattern just shipped in orbit's own CLAUDE.md; downstream operators get the same separation of concerns (universal prose discipline vs project-specific stance). Conformance drift on local STYLE.md is real signal, not noise.
- **Migration UX**: silent seed. Rationale: STYLE.md ships like every other canonical seed; `/orb:setup`'s normal output + conformance audit cover the surface.
- **Cascade scope**: bundle. Rationale: same-surface change — leaving stale citations or BLUF-framed prose in skill SKILL.md files creates the same in-session contradiction this spec exists to remove.

### Implementation Notes
- Mechanism precedent: `orbit-state/crates/core/src/verbs.rs:3719` shows the METHOD.md `include_str!` pattern. STYLE.md follows the same shape — a new const alongside the METHOD.md one.
- Conformance scope: the audit's plugin-canonical-file finding family currently covers METHOD.md (`.orbit/METHOD.md` byte-compare). Extend to STYLE.md the same way.
- `/orb:release §1.5` pre-flight sync step currently syncs `plugins/orb/skills/setup/METHOD.md` → vendored canonical. Extend to also sync STYLE.md.
- `/orb:setup` already writes METHOD.md as a seed under `.orbit/`. STYLE.md becomes a parallel seed.
- The setup skill's own SKILL.md contains a BLUF / Decision Brief citation — same cascade.
- METHOD.md pillar #1 rename: candidate name needs to retire "executive" framing while preserving the load-bearing observation (author has clear vision but no time to digest each artefact; agents pay the compression cost). Candidates: "Author-level interaction", "Decisive interaction", "Compression contract". Implementation choice for the implementing agent.
- METHOD.md prose section disposition: dropping entirely is cleanest. Alternative: replace with one-liner *"Agent-to-author prose follows the discipline in `.orbit/STYLE.md`."* Implementation choice.
- SKILL.md cascade scope per Q4: includes the 4 named SKILL.md files (design, review-spec, review-pr, setup) plus any other plugin SKILL.md citing BLUF / Decision Brief / `0026-executive-communication`. Implementing agent runs a grep pass to inventory.
- Sync-check unit test (introduced in 2026-05-19-workflow-conformance for METHOD.md) — extend to STYLE.md.
- Prior spec `2026-05-08-executive-communication-wires` ac-06 verified `@` import semantics from plugin SKILL.md files; result recorded in that spec's notes. Check there before reinventing the citation pattern for plugin SKILL.md.

### Memories Considered
- `private-projects-genericised-in-artefacts` (saved this session) — applies to any spec / commit / PR text landing in the public orbit repo. ADOPTED: implementing agent uses generic descriptors when referring to consumer projects in spec prose, commit messages, or PR body.
- `claude-code-at-import-fork-behaviour` — confirms `@-import` semantics for CLAUDE.md. ADOPTED as constraint on verification approach: post-ship spot-checks need a fresh session, not the implementing session itself.
- `session-close-2026-05-19-workflow-conformance-shipped` — captures the failed-CI `include_str!` cross-compile pattern that vendored METHOD.md. ADOPTED as constraint: STYLE.md vendoring lives under `orbit-state/crates/core/canonical/`.

### Open Questions
- None at intent level. All four residual trade-offs resolved.

---

**Next step:** `/orb:spec 2026-05-20-style-md-plugin-shipping` to materialise the spec from this design.
