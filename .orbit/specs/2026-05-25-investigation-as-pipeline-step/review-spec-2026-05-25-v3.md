# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-investigation-as-pipeline-step
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (hook config edit, cross-skill pipeline integration, observation-band audit, args-drop guard) | 1 |
| 3 — Adversarial | not triggered | — |

Spec resolved via `orbit spec resolve --skill review-spec` outcome=prompt; one of two candidates picked from the prompt list as the cycle-3 review target (the skill brief named this id directly).

## Findings

### [MEDIUM] ac-02's structural placement of the investigation step is ambiguous (pre-flight one-shot vs per-AC)
**Category:** test-gap
**Pass:** 2
**Description:** ac-02 says the implement skill gains "a structural pre-flight step that orchestrates /orb:code-investigate (narrow mode) **BEFORE AC-traversal begins**." That phrasing reads as one-shot pre-flight: a single investigation upfront covering whatever scope the agent picks for the whole spec.

But the same AC's scope-derivation prose includes "**the current AC's description**" as a scope source — wording that only makes sense if the call is per-AC (entering each AC, read its description, investigate that AC's area). Pre-flight reads ALL ACs at once, not "the current AC".

The two readings have materially different test surfaces and runtime behaviour:
- **One-shot pre-flight (N ACs → 1 investigation):** scope is a union summary across all AC areas, broader by necessity, fired once before AC-traversal. Marker file gets one entry. notes.jsonl gets one "investigation scope: …" line.
- **Per-AC (N ACs → N investigations):** scope is narrower per call, fired N times. Marker gets N entries. notes.jsonl gets N scope lines.

ac-02's smoke-marker verification asks "notes.jsonl carries the 'investigation scope: <paths>' line" (singular) and "`.orbit/.code-investigate-recent` carries marker entries matching those paths" — both readings could pass this verification. The args-drop cross-check (compare notes line vs marker entries) becomes harder under per-AC because pairing requires ordering.

ac-01's per-stage scope-derivation principle has the same ambiguity ("implement pre-flight: narrow mode; agent picks scope from the spec's tabletop.md Adjacent-code section (Q8 output, when present), the spec's cards' references[], and the current AC's description text"). The phrase "implement pre-flight" suggests one-shot but "the current AC's description text" suggests per-AC.

**Evidence:**
- ac-02 description: "BEFORE AC-traversal begins" (one-shot reading) vs "the current AC's description" (per-AC reading) — both in the same paragraph.
- ac-01 per-stage principle: "implement pre-flight" + "the current AC's description text" — same internal tension.
- The implement skill at `plugins/orb/skills/implement/SKILL.md:149` currently has a per-AC advisory ("`/orb:code-investigate` (broad mode) on the module the next AC touches before proposing edits"). ac-02 says this line is "replaced — no duplication." If the replacement is one-shot pre-flight, the per-AC investigation discipline (one investigation per module the next AC touches) is lost without a replacement at AC-entry. If the replacement is per-AC, "pre-flight" is the wrong word.

**Recommendation:** Pick one shape and say it in one sentence. Suggested resolution per the spec's goal ("agents edit from evidence rather than working memory… narrow when there's a specific question"):

- **Pick per-AC** if the goal is fine-grained scope per AC. Rewrite ac-02 opening as "plugins/orb/skills/implement/SKILL.md gains a structural step orchestrating /orb:code-investigate (narrow mode) at the entry of each AC, BEFORE edits land for that AC". Drop "pre-flight". The args-drop cross-check then compares N scope-lines against N marker entries pairwise.
- **Pick one-shot pre-flight** if the goal is one upfront sweep. Drop "the current AC's description" from scope sources; keep tabletop.md Adjacent-code + cards' references[] only. The investigation runs once at implement-entry; per-AC investigation reverts to ad-hoc invoke-when-needed. Note that line-149's per-AC discipline is then lost.

Mirror the pick into ac-01's per-stage principle (the "implement pre-flight" bullet) so the choice file and the SKILL.md edit speak with one voice.

---

## Honest Assessment

The cycle-3 reframe cleared the v2 findings cleanly: dangling substrate refs (progress.md / session-card) replaced with orbit-spec-note and orbit-memory-remember; ac-07's incoherent path-filter clause removed; ac-03's circular Q1-Q5 derivation dropped; ac-08 gains the retrieval-method note; args-drop guard added to ac-01 and cross-check verifications added to ac-02..ac-05. Every v2 finding has a corresponding text edit in the current spec.

What blocks APPROVE is one residual ambiguity that survived the v2 → v3 edit: ac-02 (and ac-01's mirror) doesn't disambiguate whether the implement-skill investigation step is one-shot pre-flight or per-AC. The phrasing carries both readings, and the verification clause doesn't force the choice. Both shapes are defensible; the spec just has to pick one. This is a 2-3 line edit, not rework.

After that pick lands, the spec is ready for implement. The 610/0 evidence is load-bearing and consistent across cycles; the per-stage scope-derivation principle is concrete; the bypass shape is named with real substrate; the measurement is per-repo and same-population pre/post. The phased-rollout escape hatch (minimum shippable = ac-01 + ac-02 + ac-07) is in place if budget pressure surfaces in implement.
