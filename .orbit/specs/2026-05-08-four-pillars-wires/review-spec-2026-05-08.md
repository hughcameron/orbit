# Spec Review

**Date:** 2026-05-08
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-08-four-pillars-wires
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content-signal scan + ac-01 ambiguity | 2 |
| 3 — Adversarial | not triggered | — |

## Findings

### [HIGH] AC-01 ignores the existing "What orbit optimises for" README section
**Category:** missing-requirement
**Pass:** 1
**Description:** README.md already contains an H2 section that delivers what AC-01 asks for. Lines 13–22 of `/home/hugh/github/hughcameron/orbit/README.md` carry an H2 "What orbit optimises for" sitting between "Why a workflow at all?" (line 7) and "You decide, orbit executes" (line 24, the lead-in to install/usage downstream). The section is public-voice prose with one short paragraph per pillar — executive-level interaction, agent self-learning, agent state-persistence, long-running R&D — exactly the four AC-01 names. The AC's prescription ("README.md … *gains* a top-level 'Four pillars' H2 section … sits after the existing 'Why a workflow at all?' intro and before the install/usage sections") describes content that is already there under a different heading. The implementer is left to guess: rename the existing section to "Four pillars"? Add a second parallel section also covering pillars? Replace the existing prose with new prose? Each interpretation produces a different artefact, and reviewers can only verify against the AC after the fact.
**Evidence:**
- `README.md:13` — `## What orbit optimises for`
- `README.md:15–22` — four bullets naming each pillar in public voice (matching the AC's "executive-level interaction, agent self-learning, agent state-persistence, long-running R&D")
- `README.md:24` — `## You decide, orbit executes` follows immediately
- Spec interview, "README placement" implementation note: "The 'Four pillars' section sits after the existing 'Why a workflow at all?' intro (around line 10) and before install/usage. Public-voice — one short paragraph per pillar, no orbit-internal jargon" — which is what already exists, but neither the interview nor the spec acknowledges this.
**Recommendation:** Rewrite AC-01 to be unambiguous about the existing section. Two clean options:

  - **Option A (rename-in-place):** "README.md's existing public-pillars section (currently H2 'What orbit optimises for', lines 13–22) is renamed to 'Four pillars' to use the canonical name from CLAUDE.md and card 0028. Existing prose is preserved or lightly edited; no content is duplicated."
  - **Option B (already-delivered):** Drop AC-01 entirely if the existing section is judged to satisfy the gate scenario. Add a one-line note in the spec goal that README naming is already live (mirrors how CLAUDE.md naming is already-live in the spec context).

  Pick one. Either is defensible; leaving the AC as written produces three different reasonable implementations.

### [MEDIUM] AC-04 mixes two scenario amendments under one verification
**Category:** test-gap
**Pass:** 1
**Description:** AC-04 covers card 0028's non-gate scenarios 4 ("Distill asks the pillar question") and 5 ("Design and review surface the parent's pillar") in a single criterion. The two scenarios are on different surfaces (distill vs design/review-spec) and a partial implementation — say, scenario 4 amended cleanly but scenario 5 left untouched — would still let AC-04 read as "amended for consistency" depending on how the reviewer parses "scenarios 4 and 5". The implementing PR review has no per-scenario test to fall back on.
**Evidence:**
- AC-04 text: "Card 0028 non-gate scenarios 4 … and 5 … are amended for consistency with the realised wiring."
- The precedent spec (`2026-05-08-executive-communication-wires`) split per-skill citations across ac-03/04/05 — one AC per skill — rather than bundling them. Same pattern would apply here.
**Recommendation:** Split AC-04 into ac-04a (scenario 4) and ac-04b (scenario 5), each naming its own verification text. Or keep one AC but make the verification explicit: "scenario 4's `then` clause no longer mentions 'surfaces which pillar', and scenario 5's `then` clause no longer mentions 'parent card's pillar(s) are surfaced'; both reframe to relations-graph awareness." That gives the PR reviewer a concrete grep target per scenario.

### [MEDIUM] No AC verifies the relations-graph wire actually exists
**Category:** test-gap
**Pass:** 2
**Description:** AC-03 reframes scenario 8's wires list to name "the relations graph that pillar-defining cards form (each pillar has cards that operationalise it; the graph is the wire)". This is the load-bearing replacement for the dropped schema-field wire — without it, scenario 8 names wires that don't exist. Card 0028's `relations:` block (verified at review time) currently lists six `feeds` edges to cards 0026/0023/0022/0009/0020/0006, each tagged with the pillar it operationalises. That graph *is* the wire, and it currently exists. But no AC in this spec verifies the wire is intact at completion — a future edit that, say, removes the pillar-tagging from the `reason` text on those relations would silently break scenario 8 without flunking any AC.
**Evidence:**
- Card 0028 `relations:` lines tag each `feeds` edge with the pillar it serves (e.g. "defines pillar 1 (executive interaction) — 0026 operationalises it via BLUF / Decision Brief discipline").
- AC-03 names this graph as the realised wire but no AC checks the relations block stays well-formed or covers all four pillars.
- Compare ac-09 in the executive-communication-wires precedent — a runtime spot-check that the wire is loaded.
**Recommendation:** Add an AC: "Card 0028's `relations:` block carries `feeds` edges to at least one pillar-defining card per pillar (executive interaction, self-learning, state-persistence, long-running R&D), with the `reason` text naming the pillar served." Cheap to verify, anchors scenario 8's wires claim against the actual graph.

### [LOW] AC-05's "or annotated as historical" leaves implementer choice with no decision rule
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-05 lets the implementer either remove the stale memo reference or annotate it as historical, "mirroring the executive-communication-wires precedent". The precedent (ac-07 in the prior spec) used the same disjunction. Looking at how the prior spec resolved it would close this — but the spec doesn't cite which path was taken, so the implementer has to either re-investigate the precedent's resolution or make a fresh judgement call.
**Evidence:**
- AC-05 text: "removed from the card's `references:` list, or annotated as historical".
- Card 0028's current `references:` block has one entry pointing at the deleted memo. Removal vs annotation produces different artefacts.
**Recommendation:** Pick one. The spec already commits to "remove or annotate"; collapse to one. If the precedent removed, this should remove too (consistency); if it annotated, this should annotate. Resolves at zero cost in design.

---

## Honest Assessment

The spec is correctly scoped — documentation + card amendment + cleanup, no Rust schema work, no SKILL.md citations, no audit changes. The constraint discipline in the interview is unusually clear: the design session pruned the original "all four wires" plan down to what the realised shape actually is, and the spec inherits that pruning cleanly. Six tight ACs against four well-bounded changes.

The biggest risk is AC-01. The README already has a pillar section — under a different heading, but covering the same four pillars in the same public voice in the same position the AC prescribes. Without disambiguation, three different implementers will produce three different artefacts (rename, replace, or duplicate), and only one of them is what the gate scenario actually wants. This is a one-line fix in design, an unbounded debate in implement and review-pr.

The other findings are smaller — AC-04 should split, the relations-graph wire wants its own verification AC, AC-05 should pick a side — but they don't structurally threaten the plan. The spec is close to ready; it needs one decisive edit on AC-01 and ideally three consistency tightenings on the rest.
