# The four pillars — orbit's load-bearing user outcomes

**Date:** 2026-05-08
**Source:** Hugh, in conversation, restating a framework he has been holding implicitly across projects

## The pillars (verbatim from Hugh)

> I want to streamline workflow to optimise towards four pillars:
>
> - **Executive level interaction** — while I have a clear vision of what I want, I'm managing multiple things. I don't have time to digest each artefact. I need concise, actionable information.
> - **Agent self-learning** — Hermes regularly saved its own memory and grew its skillset. This is vital for improvement.
> - **Agent state-persistence** — I think beads is good *for agents* — I won't read most beads — but if it keeps agents on track then great.
> - **Long-running R&D** — Make sure you do a full session's work before coming back to check in. This gives me time to dive deep when we check in. Start/stop is killing our progress.

These are the user outcomes orbit exists to deliver. Everything else is means.

## Why this needs to be a card (not just text)

The pillars aren't currently named anywhere in this repo. They're scattered across cards by accident:

| Pillar | Currently served by |
|--------|---------------------|
| Executive interaction | 0026-executive-communication |
| Agent self-learning | 0023-memory-loop, 0022-skill-curator |
| Agent state-persistence | 0009-mission-resilience, 0020-orbit-state |
| Long-running R&D | 0006-rally, 0011-cron-driven-execution |

Coverage is partial-by-coincidence. The risk is twofold:

1. **New cards drift.** Without an explicit why-test, every card argues its own value in isolation. Cards that don't move a pillar still get filed because nothing forces the question.
2. **Existing cards get built without examining whether they actually deliver the pillar they claim.** finetype's cron-driven autonomy contract was the canonical case — built ostensibly for "long-running R&D" but actually delivered "agent runs in a loop with no progress." The pillar wasn't operationalised; the *idea* of the pillar was used to justify the work.

Naming the pillars makes the failure mode visible: "this card claims to serve long-running R&D, but does it actually let Hugh do a full session of work before checking in?" is a question only available once the pillar exists as an artefact.

## What good looks like

A card — `0028-four-pillars.yaml` (or similar) — that:

1. **Names the pillars.** Verbatim from Hugh's framing, with the *user outcome* rather than the *mechanism* leading each one.
2. **Declares them as the why-test.** Every card must cite at least one pillar in a `pillars:` field, with a one-line claim about how the card moves it. Cards that can't make that claim get questioned at distill time.
3. **Audits existing cards.** A one-time pass adds `pillars:` to every existing card. Cards that don't move a pillar after honest examination get re-scoped, merged, or retired.
4. **Lives at the top of the orbit hierarchy.** Pillars are Tier 0 — above cards. The README and CLAUDE.md mention them. Any new contributor reads them before they read about cards or specs.

## Where this couples

- **`/orb:distill`** asks "which pillar?" before extracting a card from a memo.
- **`/orb:design`** surfaces the parent card's pillar(s) so the spec inherits the why.
- **`/orb:review-spec`** can flag specs that drift from the cited pillar.
- **`/orb:audit`** has a new column: which cards cite which pillars; which pillars are under-served.

## Anti-patterns to head off

- **Pillar theatre** — every card claims all four pillars. The point of the test is exclusion, not inclusion. Cards that claim more than two pillars should be split.
- **Pillar drift** — cards that originally served one pillar slowly drift to claim another to justify scope creep. The `pillars:` field should be append-only with timestamps when re-scoped.
- **Inventing a fifth pillar.** If something doesn't fit one of the four, the question is whether it's truly orbit's job — not whether to add a pillar. Pillars are the user's outcomes; they don't grow by accretion.

## Related

- `0026-executive-communication` — operationalises pillar 1
- `0023-memory-loop` — operationalises pillar 2
- `0009-mission-resilience`, `0020-orbit-state` — operationalise pillar 3
- `0006-rally`, `0011-cron-driven-execution` — claim to operationalise pillar 4 (but cron's claim is suspect — see `2026-05-08-fan-out-first-class.md` memo)

## Status

Memo only. To be distilled into a Tier-0 card. The pillars themselves are non-negotiable; what's negotiable is *how orbit operationalises them*.
