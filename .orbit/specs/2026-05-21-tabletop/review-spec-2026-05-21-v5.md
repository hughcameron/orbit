# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-tabletop
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-cutting refactor of agent-facing live surface, conformance engine state-slug rename, downstream skill consumers, three byte-identical METHOD.md copies, README, README mermaid) | 2 (LOW) |
| 3 — Adversarial | not triggered | — |

## Pass 1 — structural scan

- AC testability: every AC is a doc/observation gate on a tangible artefact path or grep result. ac-10's empirical probe pins a concrete formula, two scored populations, named candidate baselines, and a deterministic GO/NO-GO comparator. ac-11's cascade enumerates the seven step categories with grep/rg detection commands and rebuild-from-scratch atomicity.
- Constraint conflicts: none. ac-10 explicitly gates ac-11 (cascade only after GO); ac-12 explicitly sequences the two choice files against ac-10's outcome (proposed→accepted on GO; removed on NO-GO option c). ac-09 explicitly sequences after ac-04 in the runtime flow. ac-02 explicitly carves out closed-mode (ac-09) from the sidecar requirement and pins mutual exclusivity.
- Scope vs goal: the goal scope (`ship /orb:tabletop … and retire /orb:design once an ambiguity-floor probe shows parity-or-better`) maps cleanly onto the two AC clusters: ac-01..ac-09 = ship; ac-10 = probe; ac-11..ac-12 = retire + record. No over- or under-specification.
- Obvious gaps: rollback on cascade failure is named (`git restore .` re-attempt atomicity in ac-11). ac-10's NO-GO branch is named with three pre-committed picks (re-design / accept parallel / revert). Choice-file revert on NO-GO option c is named in ac-12. Memo on NO-GO is named in ac-10.
- Gate-AC description check (deterministic, no LLM judgement): four gate ACs — ac-01, ac-02, ac-03, ac-11 — all pass non-empty + not-placeholder + ≥20-char rules. Lengths run from 446 chars (ac-01) to ~5400 chars (ac-11). No flag.
- Content signal scan: cross-system boundaries present (conformance engine in `verbs.rs`, downstream `prioritise/SKILL.md` consumer of `ready_for_design` slug, three real METHOD.md copies with a byte-identical gate, README mermaid, `/orb:spec` SKILL.md design-note→tabletop-note rename). Probe deals with an empirical gating mechanism. Triggers Pass 2.

I verified the load-bearing line-number citations in ac-11 against the live tree:
- `verbs.rs:3836` carries `verb: format!("/orb:design {numeric_id}")` — confirmed.
- `verbs.rs:3833` carries `state: "ready_for_design".into()` — confirmed.
- `verbs.rs:1138` docstring enum includes `"ready_for_design"` — confirmed.
- `verbs.rs:4193` carries `operational_doc: vec!["plugins/orb/skills/design/SKILL.md".into()]` — confirmed.
- Three METHOD.md copies (`orbit-state/crates/core/canonical/METHOD.md`, `plugins/orb/skills/setup/METHOD.md`, `.orbit/METHOD.md`) all exist and are byte-identical — confirmed.
- `.orbit/METHOD.md:10` pipeline diagram, `:14` drive description, `:62` `/design + /spec` references — confirmed at those exact lines.
- `plugins/orb/skills/spec/SKILL.md:20` and `:90` carry `design-note.md` references — confirmed.
- `plugins/orb/skills/prioritise/SKILL.md` references `ready_for_design` slug — confirmed.

Line numbers in a spec text are usually stale by implement time. They are unusually load-bearing here because ac-11 declares them as the locator scaffold. The implement-time `rg`-based detection commands (`rg -l "/orb:design" plugins/orb/skills/*/SKILL.md CLAUDE.md`, `rg "(/orb:)?design|/design" .orbit/METHOD.md`, `rg "(/orb:)?design|design-note|ready_for_design" …`) are the authoritative locators; line numbers are documentation, not contract. The spec gets this right by structuring the AC around the detection commands and treating line numbers as hints. The substrate is well-cited, not brittle.

## Pass 2 — assumption & failure analysis

Triggered by content signals. The cascade touches the conformance engine, three METHOD.md copies bound by a byte-identical gate, and a downstream `prioritise/SKILL.md` consumer of the renamed state slug. Two LOW findings — neither blocks.

## Findings

### [LOW] Past-tense observation language inside ac-09 design-space pre-flight assumes /orb:design SKILL.md is still on disk at implement time
**Category:** assumption
**Pass:** 2
**Description:** ac-09 says SKILL.md "ports `/orb:design`'s §3 design-space pre-flight (open / closed / partial classification) and §4 closed-mode design-note path". The implementing agent reads `plugins/orb/skills/design/SKILL.md` §3–§4 as the source-of-port at implement time. ac-11 step (1) removes the whole `plugins/orb/skills/design/` directory. Ordering: ac-01..ac-09 land first (the implement phase that writes tabletop's SKILL.md); ac-10 runs the probe; ac-11 cascade fires only on GO. So when ac-09's port happens, `/orb:design/SKILL.md` is still on disk and the port can read from it. The ordering works. The risk is implicit: a future re-implementation or partial-roll-forward (e.g. someone re-runs ac-09 after ac-11 has landed for documentation reasons) would have nothing to port from. Low blast radius because that path requires deliberate out-of-order replay, not a normal failure mode.
**Evidence:** ac-09's prose: "SKILL.md ports `/orb:design`'s §3 design-space pre-flight …". ac-11 step (1): "`plugins/orb/skills/design/SKILL.md` removed (whole directory)."
**Recommendation:** Optional, not blocking. At implement-time, the agent should snapshot the §3 / §4 prose into tabletop's SKILL.md verbatim (or as paraphrase) at ac-09's landing time, so the resulting tabletop SKILL.md is self-contained and survives the ac-11 deletion. The spec already implies this by saying "ports … into SKILL.md" — calling it out in the implement notes is enough.

### [LOW] Conformance test sweep at ac-11 step 4(e) leaves disposition open and may need a follow-up review pass
**Category:** test-gap
**Pass:** 2
**Description:** ac-11 step (4e) says the `record_invocation(&layout, "design", ...)` test fixtures "may be left as historical accuracy OR renamed to `tabletop`" with the implementing agent picking. The downstream test fixtures touch eight literal `"design".into()` sites (verbs.rs:8369, 8391, 8423, 8445, 8471, 8489, 8800, plus 8363 and 8386/8440 in `record_invocation` calls). If the agent picks "leave as historical accuracy", future readers see a mismatch between a renamed `ready_for_tabletop` state slug and historical `skill_id: "design"` test invocations. If the agent picks "rename to tabletop", historical accuracy of the invocation log is lost. Either pick is defensible, but the consequence either way is small surface drift that a reviewer will surface.
**Evidence:** ac-11 step (4e) prose. Eight live `"design".into()` matches at the cited line ranges in `orbit-state/crates/core/src/verbs.rs`.
**Recommendation:** Optional, not blocking. The spec already pins "implementing agent picks and names the disposition in the cascade commit". That's correct — a commit-time note resolves the ambiguity. If the author wants to pre-commit, the cleaner pick is "rename to tabletop" because the field is `skill_id` (not `historical_skill_invocation_id`); the column names what skill ran, and post-cascade the skill is named tabletop.

---

## Honest Assessment

Ready. This is the strongest spec I've seen in the project on cascade-rename-style work — line-numbered locators, grep-based detection commands, three-copy byte-identical METHOD.md gate explicitly acknowledged, downstream consumer in `prioritise/SKILL.md` named, atomicity rule for the cascade pre-committed, three pre-committed picks on probe NO-GO, and the choice-file pair sequenced against the probe outcome. The empirical gate at ac-10 (parity-or-better against a baseline `/orb:design` spec via the canonical ambiguity formula, scored in a fresh context-separated agent) turns "is tabletop better than design" from an opinion into a probe artefact. That's the load-bearing risk-management move.

Biggest risk is not in the spec — it is downstream of ac-10 NO-GO. Option (b), "accept parallel `/orb:tabletop` + `/orb:design` operation indefinitely", is the trap. Two skills covering the same workflow slot drift in prose, fragment author muscle memory, and double the agent's surface to keep current. If ac-10 returns NO-GO, the cleanly-preferred fallback is (a) re-design and re-probe; (b) should be reached only when re-design has been tried and the gap persists. The spec lists (a) as the default which is the right call. Worth carrying that bias into the NO-GO decision moment.

Two LOW findings flagged for the implement phase. Neither blocks. Approve.
