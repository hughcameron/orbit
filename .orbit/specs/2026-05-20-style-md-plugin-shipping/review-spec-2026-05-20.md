# Spec Review

**Date:** 2026-05-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-20-style-md-plugin-shipping
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (cross-system Rust/plugin boundary, deployment via release, backwards compatibility on brownfield seeds) | 2 |
| 3 — Adversarial | not triggered | — |

Pass 1 deterministic gate-AC description check: all seven gate ACs (ac-01, ac-02, ac-03, ac-04, ac-07, ac-08, ac-09) pass non-empty, non-placeholder, ≥20-char rules. Goal-vs-scope alignment is tight; ACs map 1:1 onto the four interview decisions (plugin-canonical mechanism, METHOD.md prose drop, pillar #1 rename, SKILL.md cascade) plus the audit/sync-check/release/post-ship support tail.

## Findings

### [HIGH] CLAUDE.md @-import wiring for STYLE.md is unspecified

**Category:** missing-requirement
**Pass:** 2
**Description:** The spec ships STYLE.md as a seed under `.orbit/STYLE.md` (ac-04) but does not require `/orb:setup` to ensure CLAUDE.md @-imports it. The setup SKILL.md today has §6c (`Ensure CLAUDE.md @-import` for METHOD.md). Without a parallel §6c-style step for STYLE.md, consumer projects will receive the seed file but Claude Code sessions won't load its contents — STYLE.md will sit on disk and not influence prose. The interview's "what good looks like" explicitly frames success as "the agent's prose discipline already loaded" in consumer sessions; the load mechanism is the @-import, and the spec is silent on wiring it.
**Evidence:**
- `plugins/orb/skills/setup/SKILL.md:198` — current METHOD.md @-import step explicitly references `@.orbit/STYLE.md` as a shape to match, meaning STYLE.md @-import is already understood as required in *this* repo but the spec does not propagate that requirement to `/orb:setup` for consumer projects.
- ac-04 covers seed-write only ("writes STYLE.md to .orbit/STYLE.md… write on greenfield, preserve operator-edited content on brownfield").
- Goal sentence: "the reworked prose discipline reaches every consumer project" — reach requires load, load requires @-import.
**Recommendation:** Add an AC (or extend ac-04's verification) requiring `/orb:setup` to append `@.orbit/STYLE.md` to CLAUDE.md when absent, mirroring the §6c METHOD.md flow. Verification: greenfield fixture without `@.orbit/STYLE.md` in CLAUDE.md → setup appends it; idempotent when already present.

### [MEDIUM] ac-04 conflates "silent seed" with METHOD.md's actual brownfield UX

**Category:** assumption
**Pass:** 2
**Description:** ac-04 says STYLE.md is written as a "silent seed (matching METHOD.md seed semantics — write on greenfield, preserve operator-edited content on brownfield, no special operator announcement)." But `plugins/orb/skills/setup/SKILL.md:188-196` shows METHOD.md's actual brownfield behaviour is *not* silent: when `.orbit/METHOD.md` already exists and differs from canonical, setup emits an interactive prompt (`Overwrite with canonical? (y/N)`). The implementing agent will either (a) implement STYLE.md as truly silent and diverge from the METHOD.md pattern they were told to mirror, or (b) inherit the interactive prompt and silently violate ac-04's "silent seed" wording. Either way the verification ("preserve operator-edited content on brownfield") is ambiguous because METHOD.md's path doesn't preserve — it prompts.
**Evidence:**
- `plugins/orb/skills/setup/SKILL.md:188-196` — METHOD.md brownfield is an interactive byte-compare-and-prompt flow, not a silent preservation.
- Interview Q3 answer: "Silent seed — match METHOD.md / topology pattern." The interview encoded an assumption about METHOD.md behaviour that the actual flow contradicts.
**Recommendation:** Resolve the ambiguity in ac-04 before implementation. Either: (a) state explicitly that STYLE.md follows METHOD.md's interactive byte-compare-and-prompt flow (drop "silent" wording), or (b) state STYLE.md diverges from METHOD.md and is genuinely silent (no prompt; operator content always preserved on drift) and accept the small inconsistency. The "topology pattern" mentioned in the interview is the third option — topology's wire-or-decline prompt fires only when absent/empty. Pick one and update ac-04's verification to match.

### [LOW] ac-04 verification names "CLI + MCP parity test on the setup verb" but `/orb:setup` is skill-driven

**Category:** test-gap
**Pass:** 2
**Description:** ac-04 verification requires "CLI + MCP parity test on the setup verb covers the STYLE.md seed path." The setup operation is partly skill-prose-driven (§6a legacy detection, §6c @-import wiring) and partly Rust-verb-driven (`orbit topology setup` per §6d). The METHOD.md seed copy at §6b is skill prose, not a verb. If "the setup verb" in ac-04 means a Rust verb that doesn't yet exist for METHOD.md seeding, the AC requires inventing one for STYLE.md; if it means the existing `orbit topology setup` verb, that verb covers topology only. The implementing agent needs guidance on which path to take.
**Evidence:** `plugins/orb/skills/setup/SKILL.md:186` — METHOD.md copy step is skill-prose with no Rust-verb wrapper; the only `orbit setup` verb visible is `orbit topology setup` for topology scaffolding only.
**Recommendation:** Clarify whether the STYLE.md seed copy lands as (a) skill prose mirroring §6b, (b) a new Rust verb (e.g. `orbit setup canonical-seed`) that both METHOD.md and STYLE.md flow through, or (c) extends `orbit topology setup` to cover canonical files generally. Option (b) implies a refactor beyond the current spec scope and should be called out explicitly if chosen.

---

## Honest Assessment

The spec is well-grounded: it follows a recently shipped precedent (METHOD.md 0.4.21 → 0.4.22 vendoring fix), has crisp acceptance criteria, and the interview surfaces all four decision points explicitly. The mechanism — plugin source → vendored canonical → include_str! → setup seed → conformance byte-compare — is proven, and the four memories considered are correctly adopted.

The biggest risk is the @-import wiring gap (HIGH finding). Ship the file without wiring the @-import and the goal sentence fails silently: STYLE.md reaches the disk but not the session context. The MEDIUM finding (silent vs interactive brownfield UX) is recoverable in implementation but worth pinning down so the implementer doesn't have to choose under time pressure. The LOW finding flags a verification-method ambiguity that may resolve to "skill prose only" without further work.

Pass 3 (adversarial) not triggered — the structural concerns are localised to under-specified ACs, not cascading failure modes or rollback-unsafe state. Once the three findings are addressed, this is a clean implement-ready spec.
