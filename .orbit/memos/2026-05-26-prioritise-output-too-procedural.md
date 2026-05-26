# prioritise output too procedural

Observed 2026-05-26 dogfooding `/orb:prioritise` at session start. Output picked the oldest stale memo via the deterministic algorithm, with this why-now sentence:

> "Oldest of two stale memos (10 days, past the 7-day threshold), promoted ahead of the two ready_for_tabletop cards by the staleness tiebreaker..."

Author response: too procedural — describes how the ranking algorithm sorted things, doesn't say why this is the best move to take.

## The drift

SKILL.md already prescribes "Why-now over why-this" and gives examples like *"this memo's 4-line fix unblocks the follow-up cluster"* or *"only open spec, deferred twice"*. Both of those name a **consequence of acting now** — what the move unlocks, closes, or unblocks.

What the live output produced instead was a **restatement of the sort key** — "older than the other stale memo, which is older than the two cards because cards don't have a staleness field." That's the algorithm explaining itself, not selection logic justifying the call.

## Why it slips

The ranking algorithm is mechanical (severity → memo-age → spec-age → id). It's easy for the why-now sentence to inherit the algorithm's vocabulary and become a sort-key gloss. The skill needs to push the agent past "what put this on top" into "what changes when this lands."

A sharper bar: the why-now sentence should still read sensibly if the ranking algorithm changed tomorrow. If it reads as "X is older than Y," it's a sort-key gloss and fails. If it reads as "distilling this unblocks the follow-up card on schema-tightening," it's selection logic and passes.

## Fix candidate

Tighten the SKILL.md why-now prose-contract:
- Add an anti-example: *"oldest of two stale memos ranked above the cards by tiebreaker"* — explicitly call this out as restating the algorithm.
- Reframe the contract as: the sentence must name **what the move unlocks, completes, or unblocks** — never **why the ranking put it first**.
- Optional: require the why-now sentence to mention a downstream artefact (card, spec, follow-up memo, drive) so it can't survive without naming consequence.

Lower-noise: just the anti-example + reframing. The examples in the SKILL.md are already good; the contract needs to forbid the failure mode explicitly.

## Also worth noting

The prioritise output **didn't address the author's actual queued work** (CLI verb-surface cull + /orb:tabletop on card 0045). It correctly applied the algorithm to substrate state, but session-handover direction in the opening notes overrode it. That's not a bug — the skill is read-only and substrate-only by contract — but it means the author has to mentally diff "prioritise picked X" against "I named Y at session open." A future sharpening might let prioritise notice when the opening notes name a specific move and surface that alongside the deterministic pick.
