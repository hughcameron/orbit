# Spec Review

**Date:** 2026-05-08
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-08-four-pillars-wires
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | not triggered | — |
| 3 — Adversarial | not triggered | — |

## Cycle context

This is review cycle 2. Cycle 1's verdict was REQUEST_CHANGES with four findings:

1. **HIGH** — AC-01 ambiguous about the existing "What orbit optimises for" section.
2. **MEDIUM** — AC-04 bundled scenarios 4 and 5 without per-scenario verification text.
3. **MEDIUM** — No AC verified the relations-graph wire actually exists at completion.
4. **LOW** — AC-05's "remove or annotate" disjunction left the implementer to choose.

All four are resolved in the amended spec:

- AC-01 now reads "renamed to 'Four pillars'" with explicit cite of lines 13-22 and the "no content is duplicated and no parallel section is added" guard. Option A from cycle 1's recommendation.
- AC-04 now names explicit per-scenario grep targets — scenario 4's `then` no longer mandating the pillar question, scenario 5's `then` no longer mandating parent-pillar surfacing — both reframed to silent agent-side awareness via the relations graph.
- AC-07 added — names the `feeds` edges per pillar (executive interaction, self-learning, state-persistence, R&D) with `reason`-text pillar tagging as the verification surface. Anchors AC-03's wires claim against the actual graph.
- AC-05 collapsed to "annotated as historical … rather than removed", matching card 0026's precedent verbatim.

## Findings

### [LOW] AC-04 grep targets paraphrase rather than quote scenario text
**Category:** test-gap
**Pass:** 1
**Description:** AC-04 says scenario 4's `then` clause "no longer mandates surfacing the pillar question to the author at distill time". Card 0028's scenario 4 actually reads *"it surfaces which pillar the candidate card claims to move, before the card is presented for approval"* — no literal "pillar question" string. Same for scenario 5: AC-04's "surfacing the parent card's pillar" paraphrases scenario 5's "the parent card's pillar(s) are surfaced so the spec inherits the why". The intent is unambiguous, but a strict grep-against-quoted-text reviewer could stutter at PR time.
**Evidence:**
- Card 0028 line 26 (scenario 4 `then`): "it surfaces which pillar the candidate card claims to move, before the card is presented for approval — the pillar test runs at the front gate, not retroactively"
- Card 0028 lines 30-31 (scenario 5 `then`): "the parent card's pillar(s) are surfaced so the spec inherits the why, and reviewers can flag drift"
- AC-04 paraphrases both rather than quoting the existing text the implementer must amend.
**Recommendation:** No action required at design time — the implementer can reasonably match semantic intent to the existing scenario text, and the amendment direction (silent agent-side awareness via the relations graph) is clearly named. Worth noting at implement time so the AC verification is satisfied by inspecting the amended scenario `then` rather than by literal-string grep.

---

## Honest Assessment

The spec is ready. Cycle 1's findings have been addressed cleanly: AC-01 is decisive about the existing section (rename, not duplicate), AC-04 carries explicit per-scenario amendment targets, the new AC-07 anchors the relations-graph wire against verifiable graph structure, and AC-05 picks one path. The seven-AC shape mirrors the executive-communication-wires precedent at the right scale for this scope.

The single LOW finding is cosmetic — AC-04 paraphrases rather than quotes the scenario text it's amending. The implementer will not find this load-bearing; flagging it for surface awareness only.

The honest test: would a fresh pair of eyes reading this spec produce the same artefact as another fresh pair? Yes. Each AC names its target file, target field or scenario id, and the verification surface. The constraint discipline — no Rust schema, no SKILL.md citations, no audit-mode expansion — is documented in the spec goal and reinforced by the AC text.

I couldn't find structural problems. Approve.
