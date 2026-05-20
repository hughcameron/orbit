# Spec Review

**Date:** 2026-05-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-20-style-md-plugin-shipping
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-system Rust/plugin boundary, release-flow deployment, brownfield migration) | 0 |
| 3 — Adversarial | not triggered | — |

Pass 1 deterministic gate-AC description check: all eight gate ACs (ac-01, ac-02, ac-03, ac-04, ac-07, ac-08, ac-09, ac-12) pass non-empty, non-placeholder, ≥20-char rules. Goal-vs-scope alignment is tight; the twelve ACs map onto the four interview decisions (plugin-canonical mechanism, METHOD.md prose drop, pillar #1 rename, SKILL.md cascade) plus the audit/sync-check/release/@-import-wiring/post-ship tail.

Pass 2 ran because the spec touches multiple content signals: a cross-system boundary (plugin source ↔ Rust-vendored canonical ↔ include_str! const), release-flow deployment, and brownfield migration on consumer projects. No new structural concerns surfaced.

## Findings

None. The three findings from v1 (HIGH @-import wiring gap, MEDIUM silent-vs-interactive brownfield ambiguity, LOW CLI + MCP parity verification on a non-existent verb) are all addressed in v2:

- **v1 HIGH @-import wiring** — closed by new ac-12, which requires `/orb:setup` to ensure CLAUDE.md @-imports `@.orbit/STYLE.md` with the same shape as the existing METHOD.md @-import step (§6c). Verification fixtures cover greenfield, idempotent re-run, and CLAUDE.md-absent cases.
- **v1 MEDIUM brownfield UX ambiguity** — closed by ac-04's revised wording, which now explicitly says "prompt the operator before overwrite when present and divergent" and "no additional STYLE.md is a behavioural surface announcement is added; STYLE.md seeds with the same operator UX as METHOD.md." This matches the actual METHOD.md §6b interactive byte-compare-and-prompt flow rather than the v1 wording's "silent seed" claim. The "silent" framing is dropped.
- **v1 LOW CLI + MCP parity on the setup verb** — closed by ac-04's revised verification, which now requires "fixture test on the /orb:setup skill prose" with four sub-cases (greenfield, brownfield byte-equal, brownfield drift, grep for the new entry alongside METHOD.md in §6b). No CLI + MCP parity claim remains in ac-04; the AC explicitly says "no new Rust verb is introduced." The remaining CLI + MCP parity test in ac-05 attaches to the conformance verb, which is a genuine Rust verb — that placement is correct.

---

## Honest Assessment

This is now an implement-ready spec. The mechanism is proven (METHOD.md 0.4.21 → 0.4.22 vendoring precedent is cited and re-used line-for-line), the cascade is bounded (twelve ACs, each with grep-checkable or diff-checkable verification), and the v1 review's three concerns are all closed without scope creep.

The new ac-12 closes the load-bearing risk — without it, STYLE.md would have shipped to disk in consumer projects but never reached agent session context, silently failing the spec's stated goal. ac-12's verification triplet (greenfield append, idempotent re-run, CLAUDE.md-absent create) is the right shape and matches the §6c METHOD.md flow it parallels.

The biggest residual risk is at implementation time, not in the spec: keeping the eight `gate: true` ACs honestly checked rather than waved through (especially ac-09's grep-the-cascade and ac-08's three-copy pillar rename). That's a `/orb:implement` discipline question, not a spec-quality question. Pass 3 (adversarial) not triggered — no cascading failure modes, no rollback-unsafe state. The conformance audit already covers post-ship drift detection (ac-05), and the sync-check unit test (ac-06) prevents the plugin-vendored divergence that was the original METHOD.md cross-compile failure mode.
