# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-session-start-priority-synthesis
**Cycle:** 2
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | clean Pass 1 + content signal (cross-skill boundary, new conformance contract) | 1 LOW |
| 3 — Adversarial | not triggered (Pass 2 surfaced no HIGH/MEDIUM) | — |

---

## Cycle-1 finding resolution

Each cycle-1 finding mapped to its resolution in the revised spec.

| Cycle-1 finding | Severity | Resolution |
|---|---|---|
| Skill name unpinned | HIGH | AC-07 pins `/orb:prioritise` and `plugins/orb/skills/prioritise/SKILL.md`. Gate-true. |
| Adjacent-skills differentiation targets undefined neighbours | HIGH | AC-05 reworked to differentiate from the `orbit overview` verb only; `/orb:handover` reference dropped. Verb confirmed present in `orbit --help`. |
| Effort axis undefined | HIGH | AC-01 pins S (<=15min) / M (one session) / L (multi-session). |
| Ranking algorithm unspecified | MEDIUM | AC-08 pins 3-tier deterministic order (severity → memo staleness desc → spec age desc) with id-ascending tie-break and a byte-identical-output guarantee. |
| Empty-substrate behaviour undefined | MEDIUM | AC-09 covers it — planned-empty cards as runner-ups or explicit "no priorities". |
| Missing remediation.verb fallback | MEDIUM | AC-04 extended — when absent, surface `evidence` verbatim and mark item as needing manual action. |
| Top-N undefined | MEDIUM | AC-01 pins N=5 with overflow as a count, not enumeration. |
| AC-03 prose-only enforcement | LOW | AC-03 now states "via SKILL.md prose contract (matching existing read-only skills like `orbit overview`), not a hook or tool-permission gate". |
| Mid-session invocation in card but no AC | LOW | AC-02 now states "Invocation is supported at session start and mid-session; output reflects substrate state at the moment of invocation". |
| AC-06 aspirational, not measurable | LOW | AC-06 reframed to "at most 20 lines or ~500 tokens, with raw substrate references". |

All ten cycle-1 findings addressed. The notes.jsonl record on the spec matches the diff. Gate-true ACs grew from 4 to 6 (added ac-07 skill-pin, ac-08 deterministic ranking); non-gated ACs from 2 to 3 (added ac-09 empty fallback).

---

## Cycle-2 findings

### [LOW] AC-04 covers structurally-missing `remediation.verb`, not policy-suppressed verbs

**Category:** edge-case
**Pass:** 2
**Description:** AC-04 names the missing-`remediation.verb` case as "forward-compatible severity-only or info-only findings" — implying absence is the only failure mode. The current envelope (verified via `orbit --json audit conformance`) populates `remediation.verb` on every finding I see today, but the field is structurally `Option<String>`-shaped per the workflow-conformance memory. One related case isn't fully covered: a verb that exists but points at a non-runnable action (e.g. a manual operator step). AC-04 surfaces it verbatim either way, which is the right default — but the brief then claims "the author runs it without translation" which won't be true for ops-banded findings. This is a wording nit, not a behaviour gap.
**Evidence:** `orbit --json audit conformance` output shows all four current findings carry `remediation.verb` like `/orb:design 13`. METHOD.md's `ac_type` table distinguishes `ops`-banded ACs as operator actions; no equivalent banding exists on conformance findings today.
**Recommendation:** Optional. Either accept that `remediation.verb` carries a runnable command by current contract (which is true today and AC-04's verbatim surfacing is fine), or add a one-line note to AC-04 acknowledging that if a future finding family ships a non-runnable verb the brief will pass it through unchanged. Pick the first — keep AC-04 as-is. The contract today is "verb is runnable"; if that changes, a new finding is the right time to address it, not pre-emptively.

---

## Honest Assessment

The cycle-1 → cycle-2 diff is a clean specification pass. Every cycle-1 finding has a corresponding AC change with an identifiable handle (number pinned, scale pinned, neighbour pinned, fallback pinned). The two new gate-true ACs (07 skill-pin, 08 deterministic ranking) raise the bar in exactly the load-bearing places — without them the skill would ship inconsistent across invocations and undermine its own Pillar 1 claim.

Two design choices stand up to adversarial pressure on a second look:

- **AC-08's reproducibility guarantee** ("byte-identical ordering on identical substrate") is the right test surface — it makes the ranking falsifiable without mocking, and it directly serves the "Decision Brief" framing. A reader who runs `/orb:prioritise` twice expects to see the same plan.
- **AC-05 dropping `/orb:handover`** is the right call given card 0036 hasn't shipped. Differentiating against an extant verb (`orbit overview`) leaves a testable claim; the future-handover claim is correctly relegated.

The one remaining edge — non-runnable `remediation.verb` futures — is genuinely low-priority and doesn't block implementation. The spec is ready for `/orb:implement`.

**Approval is unconditional.** No revisions required.
