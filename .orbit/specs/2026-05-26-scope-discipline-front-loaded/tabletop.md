# Tabletop — Front-loaded scope discipline

**Date:** 2026-05-26
**Facilitator + domain expert:** Hugh
**Scribe + driver:** orb:tabletop (Claude Opus 4.7)
**Cards in scope:** .orbit/cards/0045-scope-discipline.yaml
**Methodology:** Card 0019 — 10-question methodology; choice 0017 — tabletop output is contract, not solution
**Output spec:** .orbit/specs/2026-05-26-scope-discipline-front-loaded/spec.yaml
**Design space:** open — no choice file pins the architecture; carve question (1 vs N specs) was itself the load-bearing trade-off

---

## Values

**Load-bearing value:** the default outcome of the pipeline produces work that matches the capability claim that justified it.

Four downstream consequences, not separate values:
- *Substrate-encoded* — the discipline doesn't depend on operator vigilance. New sessions inherit it without manual reminders. This is the hard constraint that makes the value reachable (PR #39 showed operator vigilance fails; vigilance can't be the mechanism).
- *Defence in depth* — the discipline sits at multiple pipeline stages so any single stage's drift is recoverable by the next.
- *Visible debt* — when a spec legitimately defers part of a card's ambition, the deferral becomes a tracked artefact instead of vapour.
- *Operator out of the catch-loop* — the user-facing outcome. Operator stops being the gate at PR-creation.

## Trade-offs

The original card framing put the per-AC scope check inside /orb:implement (scenario 2). That design was rejected mid-tabletop because mid-implement escalations create the staccato halt-and-discuss pattern tabletop was built to prevent. The reframe: front-load the scope-adequacy work to tabletop and /orb:spec; implement reads a pre-verified spec and runs.

**Four mechanisms ship together in one spec:**

1. Tabletop SKILL.md gains a step requiring the agent to state the underlying capability ambition before scope-carving begins.
2. Tabletop SKILL.md gains a step requiring per-scenario verification classification on every scenario the resulting spec will cover — either *"verifies: capability"* or *"verifies: stand-in (real thing is X), accepted because Y."*
3. /orb:spec SKILL.md requires every AC's verification clause to carry the classification verbatim. Halts and routes to AskUserQuestion if an AC arrives at write-up without one — three picks (rescope inline / re-walk tabletop on this scenario / accept-with-rationale captured in a spec note).
4. orbit audit conformance gains a card-coverage finding family — fires when a card has 2+ deferred scenarios across closed specs without follow-up specs landing. Each finding names the specific scenarios, the cumulative deferral count, and a remediation verb to file the follow-up.

**Carve trade-off — one spec, not two or three:** Operator chose single spec at session close. Two-spec carve (front-loading discipline + audit backstop separately) was defensible on grounds of differing evidence shape (doc vs code) and differing subsystem, but single-spec ships the full discipline in one PR with no half-wired intermediate state.

**Acceptable costs:**
- Tabletop output gets longer (one ambition paragraph + per-scenario classification table).
- One additional `/orb:spec` halt rule that fires rarely (paranoia check).
- Three skill/code surfaces change together; if the discipline evolves, three surfaces need updating in lockstep — mitigated by existing canonical-files conformance check.

**Expensive-but-worth-it:**
- Audit finding family threshold tuning. New finding families historically over-fire on first ship; the 2+ deferrals threshold is a guess and may need adjustment after first shipment.

## Halt conditions

**Halt 1 — Tabletop SKILL.md rebloats.**
*Trigger:* the tabletop SKILL.md edit grows the file by more than 30 lines net after this change. Context: the recent cut took it from 361 to 91 lines; rebloat would undo that work.
*Revert path:* collapse the per-scenario classification to a single-line template (`each scenario carries: verifies: capability | verifies: stand-in — <reason>`) and drop any multi-line per-scenario block from the template.

**Halt 2 — Audit fires on too many cards.**
*Trigger:* the new card-coverage finding fires on more than 5 cards on the orbit repo's substrate when run immediately after this ships.
*Revert path:* raise the deferral threshold (2+ → 3+) OR restrict the rule to cards with `maturity: emerging` or higher (skip `planned`-maturity cards) OR ship the rule disabled-by-default behind a config flag.

## Escalation triggers

**Trigger 1 — /orb:spec halt fires during write-up.**
*Condition:* an AC arrives at /orb:spec write-up without a verification classification from the tabletop sidecar.
*Surface to operator:* the AC text + the spec's tabletop sidecar reference + the missing classification field.
*Proposed action:* AUQ operator with three picks — (a) rescope the AC inline, (b) re-walk tabletop on this scenario, (c) accept the AC as-is with rationale captured in a spec note.

**Trigger 2 — Tabletop rebloat halt fires during implementation.**
*Condition:* the SKILL.md edit draft exceeds 30 lines net while still in the implementing agent's hand.
*Surface to operator:* the draft diff + line count + the halt-1 revert-path text.
*Proposed action:* AUQ operator with two picks — (a) apply the halt-1 collapse (single-line template, no per-scenario block), (b) drop scenario 2 (per-scenario classification) entirely and ship only the ambition paragraph.

**Trigger 3 — Audit over-fires during smoke-test.**
*Condition:* a post-implementation smoke run of `orbit audit conformance --json` against the orbit repo's own substrate emits more than 5 card-coverage findings.
*Surface to operator:* the finding count + the cards flagged + which threshold setting was used.
*Proposed action:* AUQ operator with three picks — (a) raise threshold to 3+ deferrals, (b) restrict to emerging+ cards, (c) ship behind a config flag disabled by default.

## Kill conditions

**K1 — Substrate-encoded discipline (tabletop locus).**
*Claim being killed:* the tabletop posture can be encoded in canonical SKILL.md prose without bloating the file past viability.
*Trigger:* the tabletop SKILL.md edit cannot fit the ambition paragraph + per-scenario classification in <30 lines net even with the halt-1 collapse applied.
*Pivot path:* ship /orb:spec gate + audit backstop only; leave tabletop posture as an operator-vigilance step (operator names the ambition manually each session). Accept that one of three loci falls back to vigilance.

**K2 — Audit-as-backstop.**
*Claim being killed:* the card-coverage finding family can be tuned to fire usefully without becoming noise.
*Trigger:* after threshold tuning attempts (3+ deferrals, emerging+ restriction, config-flag opt-in), the finding either fires on 0 cards (no signal) or >5 cards (noise) on the orbit repo's clean substrate.
*Pivot path:* ship the upstream discipline (tabletop + spec edits) only; drop the audit finding family from this spec. Defer the post-hoc backstop to a future spec once empirical data on deferral patterns accumulates across 5+ shipped specs under the new discipline.

**K3 — Front-loading reduces implementing-agent interruption.**
*Claim being killed:* the upstream cost of tabletop + spec classification is less than the downstream cost of per-AC mid-implement escalations would have been.
*Trigger:* the next 3 specs through tabletop after this ships show tabletop session time growing by >30 minutes per session AND implementing-agent halt frequency hasn't decreased.
*Pivot path:* revert tabletop classification step; restore the card's original scenario 2 (per-AC scope check during implement) but with a tightened phrasing that batches questions rather than firing per-AC.
*Imagined — no run-log evidence. Carry as a post-ship observation AC if desired.*

## Hot-wash

**Recurred:**
- The Q3 reframe pattern from the 2026-05-07 dogfood — a cost surfaced at Q3 triggered redesign rather than acceptance, just like that session. Worth promoting to a meta-pattern: Q3 isn't just naming costs, it's checking whether the costs themselves indicate mis-staged discipline.
- Defence-in-depth as the answer to "where does the discipline live" — same shape as choice 0017's contract-not-solution rule, applied at the meta-level of pipeline staging.

**Surprised:**
- The cost framing *"implementing agents stop more often"* triggered immediate redesign rather than acceptance. Author's instinct (*"one coherent discussion → solid parallel work → outcome"*) was sharp enough to reject mid-flight halts as a design pattern even when the cost was named honestly.
- The reframe arrived in one round-trip — no need to walk three alternative architectures before the front-loading shape was obvious.

**Friction:**
- Prose drifted into term-of-art density early (*"substrate-encoded," "load-bearing," "co-load-bearing," "downstream consequence"*). Author called it out at Q2 close. Memory saved: `tabletop-prose-too-jargony`. Stayed plain from Q3 onward.
- The /orb:prioritise output at session start was procedurally correct but read as a sort-key gloss rather than selection logic. Memo saved: `.orbit/memos/2026-05-26-prioritise-output-too-procedural.md`.

**Meta-patterns for future tabletops:**
- When a Q3 cost framing produces a *"this creates the staccato halt pattern"* reaction from the operator, the discipline is mis-staged. Front-load by default; the upstream cost is almost always less than the downstream interruption cost.
- Tabletop prose stays plain. Term-of-art shorthand creeps in fast when walking the methodology vocabulary itself — name each idea once in plain English and reuse the plain phrase. Reserve methodology terms for when they're genuinely the right word.
- When a card scenario gets rewritten mid-tabletop (as scenario 2 of card 0045 did here), update the card before /orb:spec runs — the spec's ACs reference card scenarios, and a stale scenario produces a stale AC.

---

**Next step:** `/orb:spec` against this folder to crystallise the AC contract from the values, trade-offs, halt conditions, escalation triggers, and kill conditions above.
