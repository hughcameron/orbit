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

[Confirmed 2026-05-22 via AUQ — approved as walked]

1. **Chain-shape ambiguity.**
   - Condition: agent detects recurrence but can't cleanly decide between chain and DAG (ordering varies on one branch, or parallelism is implicit).
   - Surface: invocation evidence + both candidate shapes drawn out.
   - Action: AUQ author — (a) chain, (b) DAG, (c) defer (don't propose yet).
2. **Front-matter schema validation fails on write.**
   - Condition: the SKILL.md the agent drafted doesn't validate against card 0022's front-matter convention.
   - Surface: rejected SKILL.md content + the specific validation error + the canonical template.
   - Action: agent re-templates from canonical and retries once; if retry also fails, halt for re-design — the write-path mechanism is broken.
3. **Batch verification failures.**
   - Condition: audit flags >5 routines simultaneously failing verification (typically after a skill rename or retirement, like `/orb:design` → `/orb:tabletop` this session).
   - Surface: list of affected routines + the underlying skill change that caused the batch failure.
   - Action: AUQ author — (a) bulk-edit references across affected routines, (b) archive the batch, (c) one-at-a-time review.
4. **Cross-card scope expansion.**
   - Condition: implementing requires modifying card 0022's front-matter convention (e.g. adding `last_verified` to 0022's scenario 2, or amending 0022's curator rules to read it).
   - Surface: proposed 0022 amendment + impact on existing 0022 scenarios + which scenarios would need rewriting.
   - Action: AUQ author — (a) extend 0022 inline as part of this spec, (b) keep `last_verified` card-0013-only (don't touch 0022 front-matter), (c) defer `last_verified` to v2.
5. **Substrate-shape question for chain detection.**
   - Condition: existing SkillInvocation JSONL (`.orbit/skills/<skill_id>.invocations.jsonl`, per `skill-self-improvement.md`) records single-skill invocations only — implementing agent discovers it can't reconstruct chains without substrate extension (invocations lack session-id, session boundaries unrecoverable, etc.).
   - Surface: the substrate gap + 2-3 candidate approaches (extend SkillInvocation with chain-context fields / new aggregator verb / compose existing reads with timestamp+session joins) with trade-offs.
   - Action: AUQ author — pick the approach; substrate decisions don't get made by the implementing agent alone.

## Kill conditions

[Confirmed 2026-05-22 via AUQ — approved as walked]

1. **K1: Higher-order time gain (C1 — Q2's load-bearing value).**
   - Claim killed: routines absorb the "how" and net author higher-order time goes UP, not down.
   - Trigger: post-ship observation that author cognitive load *increased* — curation cost on routines exceeds the savings from re-using them.
   - Pivot: archive all v1 routines; ship the verification mechanism only (independently valuable for any future agent-authored skills under card 0022); keep authoring manual.
2. **K2: Trust controls sufficiency (C2 — Q3 trust-budget classification).**
   - Claim killed: ≥2 threshold + curator + commit-surfacing are sufficient controls; archival rate stays ≤30%.
   - Trigger: archival rate >50% over a 20-routine window — agent's recurrence detection produces majority noise.
   - Pivot: raise threshold to ≥3 with stricter ordering match (no skipped-step allowance); if archival rate still >30%, add author-gate back (proposal artefact + approve before SKILL.md lands).
3. **K3: Substrate composition viability (C3 — Q3 simplest-cut).**
   - Claim killed: existing SkillInvocation JSONL + sequence detection + `.claude/skills/` write is a viable v1 mechanism without substrate work.
   - Trigger: implementing agent escalates (per Q7-#5) that chain detection requires substrate extension, AND author confirms the extension can't fit in this spec's budget.
   - Pivot: postpone this card; precursor card opens for the SkillInvocation substrate extension; routine-proposals returns once the substrate is ready.
4. **K4: Schema validation reliability (C4 — Q4 halt-1).**
   - Claim killed: front-matter schema validation reliably catches divergence on write.
   - Trigger: a routine ships with malformed front-matter that validation didn't catch, AND the curator desyncs because of it (curator skips it or acts on it incorrectly).
   - Pivot: stricter write-path lock — agent generates front-matter from a fixed template only, no free-form fields; if still leaky, halt agent-authoring and require author write.
5. **K5: Freshness signal reliability (C5 — Q4 halt-2).**
   - Claim killed: audit-driven `last_verified` reliably catches drift modes in practice.
   - Trigger: a routine becomes stale (component skill renamed/retired) and the audit doesn't flag it within 30 days — silent staleness slips through.
   - Pivot: revert to manual-only verification (author re-confirms each routine periodically; no automatic flag); accept the operator-attention cost as the price of correctness.

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
- Q5–Q6 captured implicitly (laterals via the reframe arc, success criteria deferred to /orb:spec). Q8 routed to Implementation Notes per intent/means filter. Q9 (budget) deferred to /orb:drive invocation. Q7 (escalation triggers) and Q10 (kill conditions) walked properly post-wrap-up and confirmed via AUQ — needed for full-auto drive readiness.

**Meta-patterns for future tabletops:**
- For pillar-2 self-learning cards, "what does X mean mechanically" questions are often the implementation-question filter firing — route to Implementation Notes immediately, don't open a fork.
- When the card's framing is unstable, walk Q1 in iterations — the card may not be the right card until Q2 closes.
- Cards near other cards in the same neighbourhood (0013 vs 0022, here) need explicit articulation of the distinction in the card body itself; otherwise tabletop keeps re-discovering the overlap.

---

**Next step:** `/orb:drive 2026-05-22-routine-proposals` at full autonomy with a working-day budget (recut from conservative-engineering quote per Q9 inflation-guard pattern). Escalation triggers and kill conditions are walked and confirmed — drive surfaces only on those conditions; otherwise runs /orb:spec → /orb:review-spec → /orb:implement → /orb:review-pr → close autonomously.
