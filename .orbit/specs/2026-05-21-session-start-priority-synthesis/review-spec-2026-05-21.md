# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-session-start-priority-synthesis
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 5 |
| 2 — Assumption & failure | Pass 1 findings + content signal (cross-skill boundary) | 4 |
| 3 — Adversarial | Pass 2 surfaced undefined neighbours, undefined effort scale, undefined ranking | 1 |

## Findings

### [HIGH] Skill name unpinned — spec, card, memo, and convention disagree
**Category:** missing-requirement
**Pass:** 1
**Description:** The card and memo call the skill `/prioritise`. Every other orb skill uses the `/orb:<name>` prefix (`/orb:design`, `/orb:implement`, `/orb:review-spec`, etc.). No AC pins the actual command string or the on-disk path (`plugins/orb/skills/<name>/SKILL.md`). Implementation will guess.
**Evidence:** Card 0043 scenario 1 says "the author invokes /prioritise"; spec ACs never mention the command. `ls plugins/orb/skills/` shows all existing skills follow `/orb:<name>` convention.
**Recommendation:** Add an AC pinning the command (recommended: `/orb:prioritise`, matching convention) and the skill path (`plugins/orb/skills/prioritise/SKILL.md`). Gate-true.

### [HIGH] Adjacent-skills differentiation targets undefined neighbours
**Category:** assumption
**Pass:** 2
**Description:** AC-05 differentiates the new skill from `/orb:overview` and `/orb:handover` (per card 0043's references). But `/orb:overview` is currently an `orbit` verb (`orbit overview`), not a skill — `plugins/orb/skills/` has no `overview/` directory. `/orb:handover` doesn't exist yet at all — card 0036 is `planned` with an open spec (2026-05-16-session-handover) but no shipped skill. Differentiating against a non-skill and a not-yet-existing-skill makes AC-05 unprovable today.
**Evidence:** `ls plugins/orb/skills/` shows no `overview/` or `handover/`. Memory `session-close-2026-05-19-workflow-conformance-shipped` confirms `orbit overview` is a verb shipped via `tree-views`. Card 0036 maturity is `planned`.
**Recommendation:** Either (a) reword AC-05 to differentiate from the `orbit overview` verb (not skill) and drop the `/orb:handover` reference until that skill ships; or (b) make AC-05 a forward-looking note in `notes:` rather than a numbered AC. Pick (a) — a soft "differentiated from `orbit overview` (status snapshot, not action plan)" is testable today.

### [HIGH] "Estimated effort" axis undefined — undermines Pillar 1 mechanism (AC-06)
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-01 requires each item to carry "estimated effort". No definition pins the unit (S/M/L? minutes? sessions?). Without a defined scale the agent improvises per-invocation, the brief becomes inconsistent across sessions, and the Pillar 1 contract in AC-06 (agent pays compression cost so author doesn't) breaks — the author re-parses each invocation's scale.
**Evidence:** AC-01 description, AC-06 description. No memory or card defines an effort axis.
**Recommendation:** Pin the scale in AC-01. Recommended: `S` (<=15min) / `M` (one session) / `L` (multi-session) — three buckets the agent can map consistently. Add a sentence to AC-01 or a new AC-01a.

### [MEDIUM] Ranking algorithm unspecified
**Category:** missing-requirement
**Pass:** 2
**Description:** "Ranked" appears in AC-01 and AC-05 without defining what drives the rank. Conformance findings carry `severity` (likely HIGH > MEDIUM > LOW); memos carry filename-date staleness; open specs carry no obvious rank signal. How does the agent merge these into one ordered list? Without a defined ranking the agent improvises, and two consecutive `/prioritise` invocations on the same substrate may produce different orderings.
**Evidence:** AC-01 description, AC-05 description. Memory `session-close-2026-05-19-workflow-conformance-shipped` confirms conformance has severity but doesn't pin merge logic.
**Recommendation:** Add an AC (or extend AC-01) pinning the ranking: e.g. "ordered by (1) conformance severity HIGH > MEDIUM > LOW, then (2) memo staleness in days descending, then (3) open-spec age descending; ties broken by id ascending."

### [MEDIUM] Empty-substrate behaviour undefined
**Category:** missing-requirement
**Pass:** 2
**Description:** When `orbit audit conformance` returns zero findings, no memos are stale, and open specs are quiet, what does `/orb:prioritise` produce? An empty brief, a "nothing to do" message, or does it pull lower-signal sources (planned cards, undriven specs, recent commits)? Today's session prime envelope is already clean — this is the case in the wild, not a hypothetical.
**Evidence:** Session prime output for this very session shows conformance clean, only 1 open spec with empty ACs (BM25) deliberately deferred. No AC covers the empty case.
**Recommendation:** Add an AC: "When conformance is clean and recent memories/memos are exhausted, the brief surfaces planned-empty cards (maturity:planned + specs:[]) as runner-ups, or declares 'no priorities' explicitly."

### [MEDIUM] No fallback when remediation.verb is missing on a finding
**Category:** failure-mode
**Pass:** 2
**Description:** AC-04 says "remediation.verb is surfaced verbatim". The conformance envelope's `ConformanceFinding` carries `remediation.verb` per the workflow-conformance shipping memory, but nothing guarantees every finding family populates it, and forward-compatible additions (severity-only findings, info-only findings) may legitimately omit it. Verbatim of nothing is undefined.
**Evidence:** Memory `session-close-2026-05-19-workflow-conformance-shipped` describes the envelope but doesn't claim `remediation.verb` is universally non-null.
**Recommendation:** Extend AC-04 with the missing-verb behaviour: e.g. "if a finding lacks remediation.verb, the brief surfaces `evidence` verbatim and marks the item as needing manual action." Or pin that `/orb:prioritise` filters findings without `remediation.verb` from the brief.

### [MEDIUM] "Top-N" undefined
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-01 says "top-N ranked list". N is undefined. Implementation will pick a number; future readers will second-guess. Pillar 1 (author compression) argues for small N; completeness argues for large.
**Evidence:** AC-01 description.
**Recommendation:** Pin N. Recommended: `N=5` for the headline brief, with overflow surfaced as a count ("3 more deferred") not an enumeration. Add to AC-01 or as a new AC.

### [LOW] AC-03 enforcement is prose-only
**Category:** failure-mode
**Pass:** 2
**Description:** AC-03's "read-only — the skill does not auto-execute any remediation verb" can only be enforced by SKILL.md prose and the agent's adherence. No mechanism (hook, tool-permission gate) prevents auto-exec under high autonomy. This is consistent with other read-only skills (`/orb:overview` verb has the same property) so it's acceptable, but the spec should acknowledge the enforcement surface.
**Evidence:** AC-03 description; no companion AC names a mechanism.
**Recommendation:** Either accept prose enforcement explicitly (add a note: "enforcement is via SKILL.md contract; agent autonomy is bound by the read-only declaration") or escalate to a hook. Pick the prose path — matches existing patterns. Optional change.

### [LOW] Mid-session invocation in card scenario but no AC
**Category:** test-gap
**Pass:** 2
**Description:** Card 0043 says "at session start (or any time the author wants to re-plan)" but no AC covers mid-session invocation explicitly. AC-02's "Substrate is re-derived live" implicitly supports it, but a reader checking ACs alone won't see the mid-session use case affirmed.
**Evidence:** Card 0043 `i_want` field; spec AC list.
**Recommendation:** Either add a scenario-aligned AC ("invocable mid-session — output is current at the moment of invocation, not session-start frozen") or accept that AC-02 covers it implicitly. Low priority.

### [LOW] AC-06 is aspirational, not measurable
**Category:** test-gap
**Pass:** 2
**Description:** "Pillar 1 mechanism — author pays no compression cost" is a design intent, not a testable criterion. How does one prove "the author sees a brief, not raw substrate"? An implementation passes by definition if it outputs anything brief-shaped.
**Evidence:** AC-06 description.
**Recommendation:** Either reframe AC-06 as a measurable proxy (e.g. "brief output is <=500 tokens / 20 lines, with raw substrate references behind file paths the author can open on demand") or move it to `notes:` as the why-context for the gated ACs. Pick the latter — AC-01 + AC-04 already cover the testable mechanism.

---

## Honest Assessment

Plan is mostly sound — the goal is clear, the four gated ACs name the right surfaces (one command, live substrate, read-only, verbatim verb), and rollback is trivial (additive skill). But three load-bearing pieces are unpinned that will bite at implementation: the skill name itself (the convention is `/orb:<name>`, not `/<name>`), the effort axis (a Pillar 1 skill can't have inconsistent compression), and the ranking algorithm (reproducibility matters for a "what do I do next" skill). AC-05's differentiation targets neighbours that aren't currently skills — that AC is unprovable today.

Biggest risk: shipping with an improvised effort scale and ranking algorithm. The skill will work, but it won't be the same skill twice — which directly undermines the Pillar 1 claim it's built on. Pin these in the spec before implementation, not in the SKILL.md after.
