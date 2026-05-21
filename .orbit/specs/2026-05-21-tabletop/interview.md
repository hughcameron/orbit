---
date: 2026-05-21
interviewer: Claude Opus 4.7
card: .orbit/cards/0019-tabletop.yaml
mode: partial
---

# Design: Tabletop — front-loaded thinking before specs are written

## What good looks like

When I'm about to start substantive work — one card or a cluster — I want a structured session that pulls all the alignment work upstream of the spec. We walk values, trade-offs, lateral approaches, success criteria, halt conditions, and escalation triggers before any spec or implementation is drafted. The output is a contract, not a solution: it captures what to optimise for and what would stop us, but explicitly does not pick the approach. Cross-cutting work gets cross-cutting alignment in one session rather than running the same conversation N times. Substantive drives start with the wrong-track risk already wrung out — predictable forks have pre-committed answers instead of mid-flight interruptions.

---

## Context

Card: *Tabletop — front-loaded thinking before specs are written* — 11 scenarios, three relations (depends-on 0010 objective-functions, feeds 0009 mission-resilience, feeds 0006 rally), two embedded notes (0026 overlap, AUQ-hybrid pattern).

Prior specs: **none live**. One archived methodology dogfood at `.orbit/archive/specs/2026-05-07-orbit-state-v0.1/tabletop.md` (241 lines) ran the 10-question methodology by hand during the orbit-state v0.1 cluster pass; it produced the values + trade-offs + halt-conditions + escalation-triggers + kill-conditions + hot-wash shape that this skill formalises.

Pinned by **choice 0017** (`tabletop-output-is-contract.yaml`, status accepted): tabletop's output is values + trade-offs + goal narrowing + scenario walk + halt conditions + escalation triggers + acceptance criteria — *never* approach selection, pseudo-code, or step-by-step plans. This is the load-bearing rule that prevents v1 `/orb:design`'s failure mode (drift into implementation).

Memory matching ran below the 0.3 threshold (top score 0.25); no formal `memories_considered` reconciliation gate. Two matches are informational and pinned in card notes already:
- `tabletop-auq-hybrid-pattern` — prose opens forks, AUQ closes them.
- `agent-estimate-inflation-guard` — recut agent estimates at Claude-pace (÷~3) before treating as planning input.

Gap: the methodology is proven; the skill (`/orb:tabletop`) is not on disk. Spec.cards is already a `Vec<String>` (plural) so the cards-array claim is substrate-ready. Spec is `deny_unknown_fields` and carries none of values/trade-offs/halt/escalation/kill/hot-wash — that decides where the contract lands.

## Q&A

### Q1: Relationship to `/orb:design`

**Q:** What is `/orb:tabletop`'s relationship to `/orb:design`? Three live options surfaced: replace, multi-card variant, or heavyweight tier.

**A (verbatim):** *"I've typically run design ahead of spec definition. It's useful, but gets into implementation too quickly. I'd be open to tabletop replacing design as long as spec ambiguity is still minimised. Tabletop typically has a 'one to many' approach to specs whereas design is still one to one. Honestly — as an author — I'd like to spend more time at a one-to-many level and encourage more fan out as long as quality and visibility remains high."*

**Interpretation:** Tabletop replaces `/orb:design` conditionally. Two constraints attach:
1. **Ambiguity floor** — tabletop must produce specs at least as unambiguous as `/orb:design`'s current target (≤0.2, per `/orb:design` §7).
2. **One-to-many fan-out is the canonical shape** — tabletop produces one or more specs from one session; `/orb:design` was one-to-one. The author wants more time at the one-to-many level, with visibility maintained across the output specs.

### Q2: Contract home

**Q:** Where does the tabletop contract (values, trade-offs, halt conditions, escalation triggers, kill conditions, hot-wash) land on disk? Spec schema is `deny_unknown_fields`.

**A:** Sidecar `tabletop.md` next to `spec.yaml`. Matches the 2026-05-07 dogfood pattern; no substrate schema change; markdown shape is human-first.

### Q3: Goal-string input mode

**Q:** How does `/orb:tabletop` handle goal-string input with no card list?

**A:** Accept zero-card mode. Agent infers which cards the goal touches and presents the inferred cluster; author approves or modifies before alignment work begins.

### Q4: Trivial-skip posture

**Q:** How strict is the trivial-skip guard (scenario 10)?

**A:** Advisory. Skill warns when work looks trivial but does not refuse; operator decides whether to continue or fall back to `/orb:spec` directly.

---

## Summary

### Goal

`/orb:tabletop` ships as the canonical pre-spec session for substantive R&D: goal-scoped, one-to-many in spec output, multi-card in scope, producing aligned contract sidecars while the spec.yaml carries only the structured AC contract. `/orb:design` retires when tabletop reaches parity-or-better on spec ambiguity.

### Constraints

- **Ambiguity floor** — tabletop-produced specs must score at least as well as `/orb:design`-produced specs on the ambiguity rubric (goal/constraint/success-criteria clarity ≤0.2). The replacement is conditional on this holding.
- **Visibility on fan-out** — when one tabletop produces N specs, the author retains insight into all of them. The skill's UX is responsible for surfacing the spec set in a digestible form (numbered list with one-line summaries; not N silent file writes).
- **Choice 0017 binds** — tabletop output is contract, never solution. Approach-selection forks are named in the scenario walk and explicitly deferred to the spec or to a choice file.
- **AUQ-hybrid** — prose opens forks (Q1–Q5, Q8 — values, trade-offs, failure modes, laterals, adjacent code); AUQ closes them (Q6 success criteria, Q7 escalations, Q9 budget recut, Q10 kill conditions). Per card 0019 note + 2026-05-07 dogfood.
- **Sidecar location** — `.orbit/specs/<spec-id>/tabletop.md` per spec; when one session produces N specs, one sidecar per spec (each contract is per-spec). A shared session-preamble at the top of each can reference the cluster.

### Success Criteria

- `/orb:tabletop` exists at `plugins/orb/skills/tabletop/SKILL.md` with the 10-question methodology codified.
- Tabletop produces specs whose `cards:` array carries one or more entries (multi-card fan-out exercised).
- Tabletop produces a `tabletop.md` sidecar carrying values, trade-offs, halt conditions, escalation triggers, kill conditions, and hot-wash for each spec it writes.
- `/orb:design` retires: SKILL.md removed, README + METHOD.md + skill cascade updated to point at tabletop.
- An ambiguity-floor probe (dry-run a tabletop session, compare resulting spec's clarity against a recent `/orb:design`-produced spec) demonstrates parity-or-better before the design/ skill is deleted.

### Decisions Surfaced

- **Tabletop replaces `/orb:design`** — conditional on ambiguity floor. Choice file candidate (e.g. `0026-tabletop-replaces-design` or similar id pending). Alternatives considered: heavyweight tier (design stays as lightweight pre-flight), multi-card variant (design stays for single-card). Author chose replacement, citing the one-to-many fan-out preference.
- **Contract sidecar** — `tabletop.md` next to `spec.yaml`. Choice file candidate. Alternatives considered: extend Spec schema with tabletop-contract fields (rejected — widens substrate API), fold into ACs (rejected — loses contract distinctness).
- **Zero-card goal-string mode accepted** — agent infers cards from goal string, author approves cluster pre-alignment.
- **Trivial-skip stays advisory** — no hard guard.

### Implementation Notes

- **Schema reality**: `Spec.cards: Vec<String>` already exists in `orbit-state/crates/core/src/schema.rs:238`. No schema change required. `deny_unknown_fields` on Spec means any tabletop-contract content lives outside `spec.yaml` (sidecar resolves this).
- **The 10 canonical questions** are pinned by card 0019 scenario 2 and the 2026-05-07 dogfood: goal, values, trade-offs (simplest way), failure modes (what could go wrong), laterals, success criteria, escalations, adjacent code, budget, kill conditions. Plus a closing hot-wash debrief.
- **AUQ-hybrid pinning** — per the 2026-05-07 hot-wash and card 0019 notes: prose for opening / reframable questions (Q1 goal, Q2 values, Q3 trade-offs, Q4 failure modes, Q5 laterals, Q8 adjacent code); AUQ for closing picks (Q6 success criteria confirmation, Q7 escalation confirm, Q9 budget option pick, Q10 kill-condition confirm). Trivial-skip nudge is prose.
- **Halt conditions** (scenario 8) must name a measurable trigger and a revert path. "Things go wrong" / "halt" alone are rejected by the skill's prose. The dogfood's K1–K6 / E1–E7 / H1–H5b shape is the canonical template.
- **Escalation triggers** (scenario 9) must name condition + state snapshot + proposed action. "Ask Hugh if confused" / "I'm stuck" are non-actionable and rejected.
- **Real scenarios preferred over imagined** (scenario 7) — the skill should walk past run-logs (each card's `specs[]` array; `.orbit/archive/specs/`) before inventing scenarios; imagined ones are explicitly flagged in the sidecar.
- **One-to-many output shape** — one tabletop session produces N specs (N≥1). Each spec gets its own folder `.orbit/specs/<date>-<slug>/` with `spec.yaml` + `tabletop.md`. A session-level preamble may reference the cluster from each sidecar's header, but each contract is per-spec.
- **`/orb:design` retirement cascade** — files/sections to update:
  - Delete `plugins/orb/skills/design/SKILL.md`.
  - Cascade SKILL.md prose across: `distill`, `spec`, `spec-architect`, `interviewer`, `drive`, `rally`, `implement`, `review-spec`, `review-pr`, `setup`, `release`, `discovery`. Any prose referring to `/orb:design` updates to `/orb:tabletop`.
  - README workflow flowchart and skills table.
  - METHOD.md pipeline diagram (`memo → distill → card → design → spec → ...` becomes `... → card → tabletop → spec → ...`).
  - Vendored copies under `plugins/orb/skills/setup/canonical/` if any reference design.
  - Conformance audit: check for `/orb:design` plugin-canonical-file drift findings post-rename.
- **Ambiguity-floor probe** — before deleting `/orb:design`, run a single tabletop session against a card that recently had a `/orb:design` pass (e.g. 0043-session-start-priority-synthesis) and compare resulting spec's clarity scores. Probe is a success criterion AC.
- **Card 0019 maturity bump** — manual planned→emerging on first spec close (the recurring N=8+ pattern: spec close doesn't auto-bump). Hugh's call at session close.
- **Card relation upkeep** — when tabletop replaces design, card 0019's `relations:` and `references:` may want a note on the supersession; card 0031 (`design-session-user-language`) — closed-mode design-note path — needs prose on whether the shipped capability migrates into tabletop or stays as a tabletop-light path. Recommend the latter: tabletop's closed-mode path produces a tabletop-note (same shape as design-note) when an associated choice pins the approach.
- **Closed-mode tabletop** — `/orb:design`'s §3 design-space pre-flight (open/closed/partial) and §4 closed-mode design-note are load-bearing; port them into `/orb:tabletop` SKILL.md verbatim. Tabletop in closed mode produces a tabletop-note plus a one-or-more-spec set with sidecar contracts; tabletop in open/partial mode runs the 10-question methodology.
- **Schema additions deferred** — no new fields on Spec or Card. If conformance later wants machine-readable halt/escalation/kill enforcement, that's a substrate spec on its own.

### Open Questions

- **Ambiguity-floor measurement** — the rubric used inside `/orb:design` §7 is qualitative (goal/constraint/success-criteria clarity, ≤0.2 numeric anchor). The replacement constraint asks tabletop to meet this floor "or better". Open: do we want a more precise measurement (e.g. a structured rubric the skill emits at session close), or is the existing qualitative rubric enough? Recommend the existing rubric — over-engineering the measurement is itself a v1 failure mode. Resolve at spec-architect time.
- **Choice file ids** — two choice files are surfaced (tabletop-replaces-design, contract-sidecar). Next free id under `.orbit/choices/` is pending lookup. Resolve at spec-architect / implement time.
- **Visibility surface for fan-out** — when one tabletop produces N specs, what's the author-facing summary shape? Card 0019 doesn't prescribe. Recommend: prose summary at session close — "Tabletop produced 3 specs: A (cards X,Y), B (cards Z), C (cards Y,W) — alignment captured at .orbit/specs/.../tabletop.md each". Resolve at implement time.
