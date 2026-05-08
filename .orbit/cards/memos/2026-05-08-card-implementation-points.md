# Cards describe intent; the framework needs explicit wiring points to enforce them

**Date:** 2026-05-08
**Source:** Hugh, asking "how is card 0026-executive-communication actually implemented in the framework?" — and the honest answer being: it isn't, structurally. Repo-wide search for BLUF / TL;DR / decision brief / executive communicat returns no hits outside the card itself. The discipline runs only when an agent happens to read the card.

## The pattern

Cards in `.orbit/cards/` describe **capabilities the framework should produce**. They are read at design time and review time. They are *not* automatically loaded into agent context, *not* enforced by the substrate, and *not* checked at output time. So a card that says "every response leads with TL;DR" is a description of intended behaviour, not a behaviour the framework guarantees.

Today's audit (samples from 2026-05-08, before this memo lands):

| Card | Maturity | Specs | Wires found in framework |
|------|----------|-------|--------------------------|
| 0026 executive-communication | planned | 0 | none — no CLAUDE.md preamble, no skill prompt citation, no audit column |
| 0023 memory-loop | planned | 0 | partial — `orbit memory remember` exists; auto-injection at prime exists in PRIME.md |
| 0009 mission-resilience | emerging | 1 | partial — spec contract + notes.jsonl land in code; halt conditions land via tabletop card |
| 0019 tabletop | emerging | several | partial — pre-flight ACs in spec template |

The pattern: cards range from *fully aspirational* (0026) to *partially wired* (0009, 0019). No card is yet *fully wired with audit*. Nothing in the framework currently distinguishes "this card describes intent" from "this card describes intent AND the framework enforces it AND the wires can be checked."

## Why naming this matters

Without an explicit wiring discipline, every new card defaults to aspirational. You file a card; you write a memo. The card describes the desired behaviour. The behaviour does not happen, because nothing in the framework's context-loading, skill prompts, hooks, or audits encodes the rule.

Concrete consequences observed:

- Card 0026 specifies BLUF / Decision Brief / seven anti-patterns / variants by response type. No agent in this repo or its dependants is told to apply any of it. Application is happenstance.
- Card 0028 (four-pillars, just landed) specifies a `pillar` field on every card. The field does not yet exist in the substrate schema. Cards continue to be written without it.
- Cards 0030 and 0031 (just landed) describe schema-doc and design-session disciplines that depend on framework wires that do not yet exist.

The honest path: every new card carries an explicit "Wired into the framework" gate scenario naming the enforcement points it depends on. Cards without named wires don't pass review. Cards with named wires get audited.

## What good looks like

A capability — likely one card — that:

1. **Defines "wires" as a card-level concern.** A wire is a concrete enforcement point: a CLAUDE.md preamble, a skill prompt citation, a substrate schema field, a SessionStart hook, an `/orb:audit` column, a test. The list is small and finite.
2. **Adds a `wires:` field (or its equivalent) to the card schema.** Each entry names the wire type, the file or path it lives in, and a short note. Cards with `wires: []` are flagged as aspirational.
3. **Makes "Wired into the framework" a standard gate scenario.** Every card carries one. The scenario names which wires the card's contract depends on. Without at least one named wire, the card cannot pass review.
4. **`/orb:audit` checks wires are live.** A new column: for each card, are the named wires actually present at the named paths? Cards with named-but-missing wires get flagged; cards with no named wires get flagged separately.

## Anti-patterns to head off

- **Wires as box-ticking.** A card claims "CLAUDE.md preamble" wire but the preamble doesn't actually encode the contract. The audit must check the wire's content, not just its existence — at minimum, that the wire references the card by ID.
- **Wires as substitute for specs.** A card with named wires still needs specs to deliver the wires. Naming the wire is not implementing it; it is the contract for the implementing spec.
- **Hidden dependency on agent reading the card.** A wire that depends on "the agent reads card 0026 before responding" is not a wire — it's a hope. Real wires are loaded into context regardless of what the agent reads (CLAUDE.md preamble) or run by the substrate (hooks, audits, schema enforcement).
- **Conflating wires with cross-card relations.** `relations:` connects cards in the dependency graph. Wires connect a card to enforcement points in code / docs / hooks. Different concerns; don't merge.

## Coupling to the four pillars

Primary: **agent state-persistence (3)** — wires turn cards from documents into substrate-level guarantees. The framework keeps agents on track because the rules live in the substrate, not in the agent's optional reading list. Secondary: **agent self-learning (2)** — wires that fail audit produce signal for the next memory: "this contract was claimed but not delivered."

## Why this couples to the schema-glossary card (0030)

The wires field belongs in the canonical schema. If 0030 lands first, it can include `wires:` in the card schema documentation. If this memo's card lands first, 0030's "schema additions land with documentation" scenario covers the case.

Most likely outcome: this card sits next to 0030 and they ship together — schema-glossary defines the wires field; this card defines the wiring discipline.

## Status

Memo only. Distill candidate. Likely a single card with two specs underneath:

1. Add `wires:` to the card schema (couples to 0030); add the standard "Wired into the framework" gate scenario as a template for new cards.
2. Add a wires-audit column to `/orb:audit` that checks named wires exist at named paths and contain a back-reference to the card ID.

## Related

- 0026-executive-communication — example of a fully-aspirational card; this memo is the discipline that would catch it
- 0028-four-pillars — its "wired into" scenario was added in the same change as this memo
- 0030-canonical-schema-and-glossary — the schema layer this card lands in
- 0031-design-session-user-language — its "wired into" scenario was added in the same change
- `.orbit/cards/memos/2026-05-08-card-schema-glossary.md` (consumed) — sibling memo proposing the schema doc
