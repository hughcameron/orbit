---
date: 2026-05-20
interviewer: claude
card: .orbit/cards/0039-workflow-conformance.yaml
mode: open
---

# Design: Park signal for conformance

## What good looks like

When I park a card, I'm declaring I've decided not to move it forward yet — usually because I'm waiting on a second use-case to confirm the pattern, on a related cluster to crystallise, or on a date. From that moment, the conformance audit should be quiet about that card: no `ready_for_design` finding, no nagging. The card stays alive in the substrate, the rationale stays attached, and when the hold condition matures I unpark it and it re-enters the audit's view. Parking is opinionated and visible — looking at the card I can see at a glance that it's held, why, and what would lift the hold.

---

## Context

Card: *workflow conformance* — 8 scenarios, maturity emerging, goal: agent-on-demand audit of substrate vs plugin contract with per-finding remediation verbs.

Prior specs: 1 — `2026-05-19-workflow-conformance` shipped the conformance verb with three finding families (plugin-canonical drift, card-state, memo staleness). The card-state finding fires hard on `maturity==planned && specs.is_empty()`, remediation `/orb:design <id>`.

Gap: the card-state rule has no carve-out for deliberate holds. Memo `2026-05-19-conformance-park-signal-gap` records a session where 14 cards tripped that rule — only 3 were genuinely undesigned, 4 were deliberately parked awaiting upstream conditions, ~4 had stale fields from prior closes. Operator absorbs the triage cost every pass. This spec addresses the parked-card source (shape 2 in the memo); the stale-field source (shape 1) is deferred to its own spec.

---

## Q&A

### Q1: Scope
**Q:** Park-signal alone, or include the stale-field fix the matching memory points at?
**A:** Park-signal only (shape 2). Shape 1 (spec.close not bumping maturity / populating specs[]) deferred to its own spec.

### Q2: Mechanism posture
**Q:** How should a parked card look on disk?
**A:** Explicit `park:` block on the Card schema with `reason:` and `until:` subfields. Visible, structured, opt-in — clearly separate from maturity lifecycle.

### Q3: Unpark expressiveness
**Q:** How expressive should `until:` be?
**A:** Free-form prose only. Cheap to ship; no validation; defers automation. Human is the parser for v1.

---

## Summary

### Goal

The conformance audit's card-state finding family honours an explicit park signal on the card. Cards carrying a `park:` block (with `reason:` and `until:`) are excluded from the `ready_for_design` finding. The signal is structured, opt-in, and visible at the top of the card YAML. Parking is reversible — removing the block returns the card to the audit's view.

### Constraints

- The `park:` block lives on the Card schema, not in `notes:` or `maturity`.
- `reason:` and `until:` are both free-form strings (no enum, no date parsing) in v1.
- Parking does not change `maturity:` — a parked card stays at whatever lifecycle stage it occupies.
- The audit must be quiet about parked cards in the card-state family; remaining finding families (memo staleness, plugin-canonical drift, aggregated audits) are unaffected.

### Success Criteria

- A card with a `park:` block does not produce a `ready_for_design` finding under conformance.
- A card without a `park:` block continues to fire the existing finding when `maturity==planned && specs.is_empty()`.
- The Card schema accepts the new optional fields and round-trips through the canonical writer (`orbit verify` clean).
- `Card::FIELDS` is extended so `audit drift` does not flag `park` as unknown.

### Decisions Surfaced

- **Mechanism: schema field over maturity vocabulary or notes-parsing.** The explicit `park:` block was chosen over a `parked` maturity value (would conflate intent with lifecycle) and a `PARKED:` notes prefix (fragile prose parsing). Likely a new MADR choice file (`.orbit/choices/NNNN-park-signal-shape.yaml`) recording this — implementing agent to decide whether the choice is heavyweight enough to warrant the file.
- **Expressiveness: free-form `until:`.** Structured triggers (ISO date, spec-id) deferred. Automation that auto-unparks on date/spec resolution can be retrofitted by extending the field shape later — backwards-compatible because free-form strings remain accepted.
- **Scope cut: shape 1 (stale-field) deferred.** The matching memory `spec-close-does-not-bump-card-maturity` (N=7 observed) names a sibling false-positive source. This spec does not address it; a follow-on spec automates `spec.close` to bump maturity and/or populate `specs[]`.

### Implementation Notes

**Memory dispositions (per /orb:design §2):**

- `spec-close-does-not-bump-card-maturity` — *partial-adopt*. Memory identifies a sibling false-positive cause (~4 of 14 cards in the memo's session). This spec does not adopt its mechanism (auto-bump at spec.close); deferred to a follow-on spec. Park-signal alone leaves shape-1 false positives in place until that spec ships.
- `feedback-card-vs-choice-distinction` — *adopted procedurally*. Park-signal is a spec-level extension to existing card 0039, not a new card or a new MADR choice. The mechanism choice (Q2) may warrant a choice file; the capability itself does not.

**Codebase leads:**

- Schema: `orbit-state/crates/core/src/schema.rs::Card` — add optional `park: Option<ParkSignal>` field. New `ParkSignal { reason: String, until: String }` struct. Extend `Card::FIELDS` const so audit drift does not flag the new field.
- Audit rule: `orbit-state/crates/core/src/verbs.rs::audit_conformance` — in the card-state walk, skip cards where `card.park.is_some()`.
- Canonical writer: confirm `ParkSignal` round-trips through serde (no manual edit needed if struct uses standard derives; canonical writer emits fields in declaration order).
- `/orb:card` SKILL.md: surface the `park:` field in the card-authoring prose. One paragraph naming when to park, what to put in `reason:` and `until:`, and how to unpark.
- `/orb:audit` and conformance prose: cite the carve-out so agents reading findings understand why some `planned`+`[]` cards don't appear.

**Open implementation decision: envelope visibility for parked cards.** Three plausible shapes — fully silent (no trace in the envelope), aggregated count (`aggregated.parked: [<ids>]` or `parked_count: N`), or low-severity findings (`severity: low`, `state: parked`, `remediation: null`). Recommendation: **silent** for v1 — matches the "stop nagging" intent most literally, smallest envelope change. The aggregated-count shape is a cheap one-line addition the implementing agent may choose if they judge agents reading the envelope benefit from knowing the count exists. Low-severity findings are over-engineering for v1.

**Test expectations:**

- Schema test: Card parses with and without `park:`, round-trips through canonical writer, `FIELDS` extension prevents drift false-positive.
- Unit test matrix on audit_conformance card-state walk: (a) `planned` + `[]` + no `park` → finding fires; (b) `planned` + `[]` + `park: {reason, until}` → no finding; (c) `emerging` + `park` → no finding (maturity check is upstream); (d) `planned` + non-empty specs + `park` → no finding (specs check is upstream); (e) multiple parked cards → no card-state findings for any of them.
- CLI + MCP parity test on the carve-out case.

**Cross-skill prose:**

- `/orb:card` — name the `park:` field in card-authoring prose
- `/orb:audit` (if a SKILL.md exists for it) — name the carve-out
- Conformance documentation in `/orb:setup` §6e (or wherever the verb is surfaced) — mention parked cards as a deliberate carve-out from the `ready_for_design` family

### Open Questions

None at intent level. Visibility envelope shape (above) is routed to implementation notes with a recommendation.
