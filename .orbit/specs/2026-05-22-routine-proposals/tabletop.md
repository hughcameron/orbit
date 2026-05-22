# Tabletop — Routine proposals from recurring skill chains

**Date:** 2026-05-22
**Facilitator + domain expert:** Hugh
**Scribe + driver:** Claude Opus 4.7
**Cards in scope:** .orbit/cards/0013-routine-proposals.yaml
**Methodology:** Card 0019 — 10-question methodology; choice 0017 — output is contract, not solution
**Output spec:** .orbit/specs/2026-05-22-routine-proposals/spec.yaml

---

## Values

**Load-bearing value:** the author's time concentrates on higher-order thinking — judgment, synthesis, vision — as named routines absorb the lower-order "how".

Downstream consequences:
- **Substrate trustworthiness** is the precondition for the higher-order gain. Untrusted routines force re-evaluation = lower-order work.
- **Agent autonomy at safe scale** is the cost paid to maintain trust. Cost must stay small enough that net higher-order time goes up.
- **Discoverability** is what realises the higher-order gain. Undiscovered routine = author re-derives from scratch.

Pillar 2 (agent self-learning) is the mechanism; pillar 1 (author-level interaction) is the load-bearing why.

## Trade-offs

**Simplest cut (the v1 mechanism):** agent reads SkillInvocation JSONL across sessions; when a skill_id sequence repeats ≥2 times with consistent ordering, agent writes `.claude/skills/<name>/SKILL.md` directly carrying front-matter per card 0022 (`created_by: agent`, `created_at`, `pinned: false`, `last_verified: <timestamp>`); commits with a clear message. Author sees on commit, edits inline, or archives via curator. No separate proposal artefact.

| Trade-off | Cost | Classification |
|---|---|---|
| Trust budget — agent can ship a bad routine; author edits or archives | Recoverable but spends author attention | expensive-but-worth-it |
| False-positive skills (coincidence not pattern) | Author archives some | acceptable at ≥2 threshold; revisit if archival rate >30% over 10-routine window |
| Substrate weight — `.claude/skills/` grows | Discoverability decays past N skills | expensive-but-worth-it (card 0022 curator handles archival) |
| Routine drift — skill ships, substrate evolves, skill stales | Author follows stale routine | halt-trigger (see below) |
| Discoverability — author has to notice the new skill | Without surfacing, the skill is invisible | acceptable (commit message is v1 surfacing; session-prime upgrade is v2) |
| Auto-generated skill names | Agent picks a name | acceptable — author renames |
| Detection complexity — read invocation streams across sessions | Implementation cost | acceptable — substrate already exists |

## Halt conditions

1. **Front-matter divergence from card 0022 convention.**
   - Trigger: any agent-written SKILL.md fails front-matter schema validation on write.
   - Revert: write rejected; agent re-templates from canonical front-matter or NO-GOs.
2. **Routine drift undetected.**
   - Trigger: any routine with `last_verified` >30 days that conformance audit failed to flag.
   - Revert: archive all routines >30 days unverified pending re-verification; halt for freshness-mechanism re-design.

## Escalation triggers

[Inferred at wrap-up — author confirms during /orb:spec]

1. **Chain-shape ambiguity.** Agent detects recurring pattern but can't cleanly decide between chain and DAG (e.g. ordering varies on one branch).
   - Surface: invocation evidence + both candidate shapes.
   - Action: AUQ author — (a) chain, (b) DAG, (c) defer (don't propose yet).
2. **Schema validation fails on write.** Front-matter doesn't validate against card 0022 convention.
   - Surface: rejected SKILL.md content + validation error.
   - Action: agent re-templates from canonical and retries; if retry also fails, halt for re-design.
3. **Batch verification failures.** Audit flags >5 routines simultaneously failing verification (e.g. after a skill rename).
   - Surface: list of affected routines + the underlying skill change.
   - Action: AUQ author — (a) bulk-edit references, (b) archive batch, (c) one-at-a-time review.

## Kill conditions

[Inferred at wrap-up — author confirms during /orb:spec]

1. **K1: Higher-order time gain (Q2 value).** If author cognitive load goes UP (not down) after v1 — because routine quality is too low and curation cost exceeds the savings — pivot: archive all v1 routines, ship verification mechanism only, keep authoring manual.
2. **K2: Mechanical front-matter validation (Q4 halt-1).** If schema validation can't reliably catch front-matter drift in practice, pivot: stricter write-path lock (agent generates from template, no free-form fields).
3. **K3: `last_verified` freshness signal (Q4 halt-2).** If the audit-conformance verification mechanism doesn't catch the drift modes it was designed for, pivot: revert to manual-only verification (author re-confirms periodically; no automatic flag).

## Implementation Notes

Means-level leads, routed here per the intent/means filter (choice 0012):

- **What `verified` means.** Audit conformance has confirmed every `/orb:<verb>` reference in the routine's SKILL.md body still resolves to a live skill (existence + non-retirement). Mechanical, binary, cheap (`grep` + cross-check). `last_verified` = timestamp of most recent passing audit. Not execution success (fuzzy for chains), not author touch-up (sparse), not component SKILL.md hash (noisy).
- **External plugin scope.** v1 limited to orbit-skill chains only; external-plugin chains (`/<plugin>:verb`) are a v2 extension.
- **Recurrence detection.** Allow ≤1 skipped step in chains of ≥3 for consistency-counting (catches minor variations). Pick the longest consistent chain when ambiguous (avoids re-proposing sub-chains).
- **Surfacing in v1.** Commit message (`feat(skills): draft .claude/skills/<name> from recurring chain (N occurrences)`) is the v1 surfacing mechanism. Session-prime "agent-authored routines pending review" upgrade is v2.
- **AC types per card 0035.** Detection + write path + audit integration = `code`. Front-matter convention compliance = `config`. Routine drift soak (post-ship observation that mechanism catches real drift) = `observation` (defers).
- **Adjacent code layers (Q8).** orbit-state core (SkillInvocation reader extension or new chain-aggregator verb); conformance audit (new finding family for routine drift / front-matter divergence); `.claude/skills/` directory (new write target, project-local). File-level decisions deferred to implementing agent.

## Hot-wash

**Recurred:**
- "Is this implementation?" The author enforced the means/ends filter (choice 0012) twice — first implicitly on verification mechanics, then explicitly on external-plugin scope. The filter is load-bearing for tabletop sessions themselves, not just for the cards they shape.
- "Routine" vs "orchestration" — vocabulary refinement landed mid-Q1; user-language matters even for substrate-internal naming.

**Surprised:**
- The card reframe arc happened *during* the tabletop walk: playbook-fast-path → success-routing-for-individual-skills → skill-orchestration-proposals → routine-proposals. Three pivots before Q2 closed. Tabletop turned out to be more substantive than the v1 framing suggested, and the card was renamed twice with `git mv` mid-session.
- Card 0013 nearly merged into card 0022 twice during the walk; the chain-vs-procedure distinction needed sharp articulation each time. The accepted distinction: card 0022 = single-skill authoring via spec-author path (heavyweight, evidence is the recurrence); card 0013 = chain-skill authoring directly to `.claude/skills/` (lightweight, the chain IS the evidence and IS the spec).

**Friction:**
- Drifted into implementation on Q3 (`last_verified` definition) and Q4 (verification semantics). Author called the boundary; agent had to wrap.
- Q5–Q10 collapsed into inference-from-session rather than the full walk because the author's "wrap this up" landed before Q5 opened. Escalation triggers and kill conditions are author-confirms-at-spec-stage markers, not walked positions.

**Meta-patterns for future tabletops:**
- For pillar-2 self-learning cards, "what does X mean mechanically" questions are often the implementation-question filter firing — route to Implementation Notes immediately, don't open a fork.
- When the card's framing is unstable, walk Q1 in iterations — the card may not be the right card until Q2 closes.
- Cards near other cards in the same neighbourhood (0013 vs 0022, here) need explicit articulation of the distinction in the card body itself; otherwise tabletop keeps re-discovering the overlap.

---

**Next step:** `/orb:spec 2026-05-22-routine-proposals` to crystallise the AC contract. Author should refine the Escalation triggers and Kill conditions sections during /orb:spec — they were inferred during wrap-up, not walked properly.
