# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-tabletop
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-system: conformance engine, three METHOD.md copies, `/orb:spec` design-note path) + cascade scope | 4 |
| 3 — Adversarial | structural ambiguity in ac-11 scope (live-surface set under-named) | 1 |

Pass 1 deterministic gate-AC check (ac-01, ac-02, ac-03, ac-11): all four descriptions non-empty, not placeholder tokens, well above the 20-char minimum (626 / 825 / 696 / 2700+ chars). PASS.

v3 findings audited against the current spec:
- v3 HIGH (ac-10 baseline artefact) — **fixed**. Three candidate specs with `interview.md` on disk are now named (verified present); the ineligibility of `2026-05-21-session-start-priority-synthesis` is explicit.
- v3 MEDIUM (formula direction) — **fixed**. ac-10 now quotes the canonical `ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)` verbatim with the `1 -` prefix; the GO rule "tabletop ambiguity ≤ baseline ambiguity" is direction-correct.
- v3 MEDIUM (CLAUDE.md line 3 narrative) — **fixed**. ac-11(2) now explicitly names CLAUDE.md line 3's `card → design → spec → implement → review pipeline` phrase.
- v3 LOW (ac-12 ordering) — **fixed**. ac-12 now pins `tabletop-replaces-design` as `proposed` during implement; promoted to `accepted` post ac-10 GO + ac-11 cascade; removal path on NO-GO option (c) is named.
- v3 LOW (cascade atomicity) — **fixed**. ac-11 now closes with single-commit-or-single-PR rule + `git restore .` rollback on partial failure.

All five v3 findings have landed. The four findings below are fresh — they live in surfaces v1–v3 didn't probe.

## Findings

### [HIGH] ac-11 scope misses `/orb:spec` SKILL.md's design-note dependency

**Category:** missing-requirement
**Pass:** 2
**Description:** `/orb:spec` SKILL.md is a load-bearing live surface that the cascade leaves stale. Three sites depend on `/orb:design` machinery:
- Line 20 — describes inputs: *"The input artefact may be a full **interview** (`interview.md`) from an open or partial design space, or a short **design note** (`design-note.md`) from a closed design space. ... see `/orb:design` §3–§4"*.
- Line 23 — *"Check conversation history for a recent `/orb:design` or `/orb:discovery` session"*.
- Line 38 — *"Ambiguity must be ≤ 0.2. If higher, suggest returning to `/orb:design` or `/orb:discovery`"*.
- Line 129 — *"Agents downstream (`/orb:design`, `/orb:implement`) rely on this array..."*.

ac-11(2)'s canonical detection (`rg -l "/orb:design" plugins/orb/skills/*/SKILL.md CLAUDE.md`) **does** surface `spec/SKILL.md` (it contains `/orb:design`), so the file would be edited — but the AC text only commits to "becomes `/orb:tabletop` or is rewritten for the new shape". The `design-note.md` filename mentioned on line 20 (and line 90 — *"or design-note.md for closed-space inputs"*) is not a `/orb:design` reference, so the grep misses it. After cascade, `/orb:spec` would still describe `design-note.md` as an input artefact — but ac-09 declares the closed-mode tabletop output is `tabletop-note.md`. The two skills will name different filenames for the same closed-space handoff.
**Evidence:** `rg -n "design-note" plugins/orb/skills/spec/SKILL.md` returns lines 20, 90. ac-09 spec text: *"a one-screen `tabletop-note.md` (mirroring `/orb:design`'s design-note shape)"*. Conformance engine `verbs.rs` lines 890, 1993, 2034 also scan `design-note.md` as a sidecar input.
**Recommendation:** Add an explicit clause to ac-11. Either (a) rename `tabletop-note.md` to `design-note.md` in ac-09 and keep the filename across both skills — minimal churn, but the prose name "design-note" reads stale; or (b) add a step to ac-11 cascade: *"`/orb:spec` SKILL.md updated — `design-note.md` references rewritten as `tabletop-note.md`; the `verbs.rs` sidecar scan-set in `spec.close`'s topology-warning path (lines 890, 1993, 2034) updated to match"*. Pick (b) for filename consistency with ac-09. Without one of these, the cascade leaves a live-surface filename mismatch that `/orb:spec` and `/orb:tabletop` will disagree on at next handoff.

### [HIGH] ac-11 conformance-engine sweep is narrower than the actual `design` footprint in verbs.rs

**Category:** missing-requirement
**Pass:** 3
**Description:** ac-11(4) names four targets in `orbit-state/crates/core/src/verbs.rs`: the `remediation.verb` format string, docstrings, body strings ("use orbit memory match before /orb:design"), and test assertions. A fresh grep returns 38 hits across 5 distinct categories, three of which ac-11(4) doesn't pin:
1. The **state slug** `"ready_for_design"` — appears as a string literal at line 3833 (assignment), line 1138 (docstring enum), line 9957 (test assertion), line 10026 (test comment). The conformance schema's `finding.state` field carries `"ready_for_design"` to downstream consumers (`/orb:prioritise` reads it — see `plugins/orb/skills/prioritise/SKILL.md:77`). If ac-11(4) only updates the `remediation.verb` from `/orb:design` to `/orb:tabletop` but leaves the state slug as `"ready_for_design"`, callers reading state=ready_for_design will mismatch the verb=/orb:tabletop, and downstream prose like "ready for design" reads stale.
2. The **topology pointer** at line 4193: `operational_doc: vec!["plugins/orb/skills/design/SKILL.md".into()]`. ac-11(1) deletes that file. The topology pointer would dangle.
3. **Test fixtures and helpers** (lines 8363–8800, 9598–9699): `record_invocation(&layout, "design", ...)`, `skill_id: "design".into()`, `fn install_spec_for_warnings(... design_note: Option<&str>)`, `fn spec_close_topology_warnings_match_in_design_note_only()`. These are not all narrow-scope renames — the `record_invocation` calls reference the deleted skill folder by id and won't fail compilation if left, but the test names and parameters carry stale terminology. The `spec_close_topology_warnings_match_in_design_note_only` function name in particular hard-codes the design-note filename in its identity.

ac-11(4)'s closing `cargo test -p orbit-state` runs green check is necessary but not sufficient — green tests with `skill_id: "design"` and `state: "ready_for_design"` are still green. The drift surfaces only when a downstream agent reads `state="ready_for_design"` after cascade and renders the wrong verb in its prose.
**Evidence:** `rg "design" orbit-state/crates/core/src/verbs.rs | wc -l` returns 38. `rg "ready_for_design" .../verbs.rs` returns 5 hits in three distinct contexts (docstring enum, assignment, test). Topology pointer at line 4193 points at the file ac-11(1) deletes.
**Recommendation:** Expand ac-11(4) to enumerate five conformance-engine targets, not four:
1. `remediation.verb` format string (currently named).
2. State slug `"ready_for_design"` → `"ready_for_tabletop"` (or pick another slug — name the choice). Update all five sites (docstring enum at line 1138, assignment at line 3833, test assertion at line 9957, test comment at line 10026, the parked-card-skip comment at line 3803). Cross-update `/orb:prioritise` SKILL.md line 77 — it ties state-based remediation to the slug.
3. Topology pointer at line 4193 — `plugins/orb/skills/design/SKILL.md` → `plugins/orb/skills/tabletop/SKILL.md`.
4. Body strings + docstrings (currently named).
5. Test fixtures and function names — `install_spec_for_warnings(..., design_note: Option<&str>)` parameter rename, `spec_close_topology_warnings_match_in_design_note_only` function rename, and `skill_id: "design"` literal updates throughout the test module. Note: the `record_invocation(&layout, "design", ...)` calls test invocation tracking against the `design` skill-id — those literals reference the historical record-keeping. The implementing agent picks: rename to "tabletop" (cleaner) or leave (historical accuracy). The AC should name the pick, not let it drift.

Without this expansion, ac-11(4) closes green on a partial sweep and the conformance state-slug stays stale.

### [MEDIUM] METHOD.md narrative prose drifts past the pipeline-diagram update

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-11(3) updates the METHOD.md pipeline diagram (`memo → distill → card → design → spec → ...` → `... → tabletop → spec → ...`) across three byte-identical copies. The diagram is one line. METHOD.md carries six additional `design` references in narrative prose that the diagram-update step doesn't touch:
- Line 14 — *"**drive** — one agent runs design → spec → implement → review-pr autonomously"*.
- Line 62 — *"Is X a discrete piece of work with acceptance criteria? → **spec** via `/design` + `/spec`"*.
- Line 82 — *"Every card, skill, and design choice"* (incidental use — probably out of scope).

Of those, the line-14 drive description and the line-62 `/design` reference are agent-facing prose that goes stale post-cascade. ac-11(3) commits to "pipeline diagram updates" but not to "all narrative references in METHOD.md". The three-copy byte-identical gate (conformance) is preserved either way — but stale prose ships to every project that consumes setup-skill-seeded METHOD.md.
**Evidence:** `rg -n "design" .orbit/METHOD.md` returns 6 hits; ac-11(3) commits to "pipeline diagram updates" only.
**Recommendation:** Broaden ac-11(3) from "pipeline diagram updates" to "METHOD.md pipeline diagram AND narrative `/design` references update". Detection: `rg "/orb:design\|/design\|(?<![a-z])design(?![a-z])" .orbit/METHOD.md` and edit each hit in turn (the third regex catches bare `design` in narrative — the line-82 incidental "design choice" use stays because of context). The byte-identical-across-three-copies gate is preserved automatically because the same edit lands in all three.

### [LOW] ac-09 closed-mode tabletop classification ambiguity at goal-string entry

**Category:** failure-mode
**Pass:** 2
**Description:** ac-04 names the goal-string entry path: agent infers cards from goal string, presents cluster, fires AUQ. ac-09 names the closed-mode path: an associated choice file pins the approach + prior specs build on the pattern → `tabletop-note.md` not `tabletop.md`. The interaction between the two is unstated. A goal string like *"add another reconcile rule"* could legitimately map to a closed design space (choice 0023 reconcile-as-canonicalise-mode pins the approach, prior specs build on it) — but the inferred-cards-via-AUQ flow in ac-04 walks the operator through cluster confirmation, not through design-space classification. If the operator confirms a cluster that turns out to be closed-mode, when does ac-09's classification fire? Before AUQ? After? The current spec is silent.

This is recoverable at SKILL.md authoring time — the implementing agent will pick an ordering — but spec-level ambiguity here means two different SKILL.md drafts could both close the AC validly with incompatible flows.
**Evidence:** ac-04 names AUQ confirmation as "the load-bearing safety valve" for goal-string input. ac-09 names classification triggers (choice + prior specs) but no entry point. No AC names which fires first.
**Recommendation:** Add one sentence to ac-09: *"For goal-string entry (ac-04), design-space classification (open / closed / partial) runs **after** the AUQ cluster confirmation — the agent first locks the card scope, then assesses whether any associated choice pins the approach within that scope."* This pins the ordering and removes the ambiguity at SKILL.md authoring time.

---

## Honest Assessment

The methodology core (ac-01..ac-09) is sound and the four gate ACs pass the deterministic Pass-1 check. The v3 cycle's findings have all landed correctly — the ac-10 baseline names eligible specs, the formula direction is consistent, CLAUDE.md line 3 is in scope, ac-12 ordering is pinned, and ac-11 has atomicity guidance.

What this pass surfaces is a different structural concern: **ac-11's cascade scope is under-named relative to the actual live-surface footprint of `/orb:design`**. Three findings (two HIGH, one MEDIUM) all share the same shape — the cascade names some live surfaces precisely (skill folder, METHOD.md diagram, conformance verb format string) but misses adjacent surfaces in the same files (the `design-note.md` filename in `/orb:spec`, the `ready_for_design` state slug in verbs.rs, the narrative prose in METHOD.md beyond the diagram line).

The risk is concentrated: ac-11 is one-shot (single commit / single PR per the atomicity rule), the `cargo test -p orbit-state` gate runs green on partial sweeps because the missed surfaces are string literals and slug values that don't fail compilation, and the drift only surfaces when a downstream agent — `/orb:prioritise`, `/orb:spec`, the topology checker — reads a stale slug or filename and renders inconsistent prose. A wrong-but-green cascade is still wrong.

The fix is bounded: three AC expansions (ac-11(2), ac-11(3), ac-11(4)) each name additional sweep targets; one one-sentence clarification to ac-09 fixes the goal-string × closed-mode ordering. All sentence-sized edits. The methodology halves of the spec are ready; the retirement-cascade half needs one more pass to harden its scope to match the live-surface set.
