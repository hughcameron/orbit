---
name: prioritise
description: Session-priming synthesis — name the next move live from workflow conformance, the session-prime envelope, and recent memories. One verb, one sentence on why now, a deferred count. Read-only — the author runs the verb.
user-invocable: true
created_by: claude
created_at: 2026-05-21
---

# /orb:prioritise

Name the next move live from substrate signals. Returns one imperative verb, one sentence on why-now, and a count of what's deferred. The author runs the verb. The skill never auto-executes.

## Usage

```
/orb:prioritise
```

No arguments. Invocable at session start or any time mid-session — the output reflects substrate state at the moment of invocation, not a frozen handover.

## Why this exists

Session handovers freeze priorities at the moment they're written. By the next session, memos accumulate, cards age, conformance findings change. The author wants one thing: *what should I do first?* This skill compresses substrate to a single recommended move, surfaced with the pattern that makes it the move (not just one of five).

Pillar 1 (author-level interaction). Distinct from `orbit session prime`, which produces the raw envelope (handover, open specs, memories) for the agent to bootstrap context. `prioritise` consumes that envelope and names the move.

## Substrate inputs

Read these three sources live — do not extrapolate from git log or filesystem scans.

1. **`orbit audit conformance --json`** — findings carry `severity` (HIGH/MEDIUM/LOW), `subsystem`, `subject`, `evidence`, and `remediation.verb`.
2. **`orbit session prime`** — open specs, recent memories, prior handover distillate.
3. **Recent memories** (from the session-prime envelope or `orbit memory list --recent`) — keep only those signalling deferred decisions, calendar deadlines, or unresolved questions.

## Ranking algorithm

Pick *one* deterministically — two consecutive invocations on identical substrate must produce the same move:

1. Conformance severity: `HIGH` > `MEDIUM` > `LOW`
2. Within severity: memo staleness in days, descending (older = higher)
3. Within staleness: open-spec age, descending (older = higher)
4. Ties broken by id ascending

The first-ranked item is the move. Everything else becomes the deferred count.

## Next-action verb

For a conformance finding, surface its `remediation.verb` verbatim — no translation, no paraphrase. The author runs the verb directly.

When a finding lacks `remediation.verb` (forward-compatible severity-only or info-only findings), surface the finding's `evidence` field verbatim and tag the move `manual action`.

For a move drawn from a non-conformance source (memo, open spec, planned-empty card), the next-action is the verb the author would type — e.g. `/orb:distill .orbit/memos/<file>.md`, `/orb:drive <spec-id>`, `/orb:tabletop <card-id>`.

## Output shape

Three lines, blank lines between:

```
<verb verbatim>

<one sentence on why-now — name the pattern that makes this the move, not just the item's own stake>

+ N other items deferred — ask if you want them.
```

The deferred tail is omitted when `N == 0`.

Total: ≤5 lines. The why-now is not a restatement of the item's purpose — it's the pattern that promotes this to *now* (e.g. "older of two MEDIUM conformance findings sat across three sessions", "only open spec, deferred twice", "this memo's 4-line fix unblocks the follow-up cluster").

## On request: the full list

If the author asks "what are the others?", "show me the full list", or anything matching that intent, render up to top 5 on the follow-up turn:

```
N. <one-line what>
   why: <one-line stake>
   effort: <S | M | L>
   next: <verb verbatim>
```

Effort bands: `S` ≤15 min, `M` one session, `L` multi-session.

Never enumerate beyond 5 in one response. Cite memory keys, conformance subjects, and file paths — do not inline substrate content.

## Empty-substrate fallback

When `orbit audit conformance --json` returns zero findings and recent memos / open specs are exhausted:

1. Pick the oldest planned-empty card — `maturity: planned` AND `specs: []`, lowest id. Use `orbit overview` to list. Recommend `/orb:tabletop <card-id>`.
2. If no runner-ups exist either, output exactly: `no priorities — substrate is clean and the backlog is exhausted`.

Never return an empty brief without an explanation.

## Algorithm

1. Run `orbit audit conformance --json`. Collect findings.
2. Run `orbit session prime`. Collect open specs and recent memories.
3. Filter memories for actionable signals (deferred deadlines, blocked decisions, unresolved questions). Drop status-recap and session-close summaries.
4. Apply the ranking algorithm above.
5. Compose the move — verb, why-now sentence, deferred count.
6. If nothing surfaced, apply the empty-substrate fallback.
7. Return. Do **not** execute the verb.

## What this is NOT

- **Not `orbit session prime`.** Prime is the agent-facing raw envelope (handover, open specs, memories) — it bootstraps context. `prioritise` consumes that envelope and names the move.
- **Not `orbit overview`.** Overview produces a status snapshot (cards-by-maturity, orphans, most-connected card). `prioritise` produces an action recommendation.
- **Not auto-executing.** Enforcement is via this SKILL.md's prose contract — there is no hook or tool-permission gate. If the author wants the move run, they type the verb themselves.
- **Not session-start frozen.** Invoke any time. The output is live re-derivation.
- **Not a menu.** One move. The full list is on request.

## Discipline

- **Read-only.** No `orbit task open`, no `orbit memory remember`, no `orbit spec note`. The skill never writes.
- **Pick the move.** STYLE.md is the spine: one imperative action, in plain voice. A top-5 list as the default output is a menu — it violates the contract.
- **Why-now over why-this.** The sentence justifies the *timing* of the pick, not the item's intrinsic value. "Older of three card gaps that have sat for three sessions" beats "card 0013 is planned with scenarios but no tabletop pass" — the latter is restatement, the former is selection logic.
- **Cite, don't paraphrase.** Name memory keys, conformance subjects (`<subsystem>/<subject>`), spec ids, and card slugs verbatim. The author drills down by opening the cited substrate, not by re-reading prose.
- **No fanfare.** No header. The verb IS the brief.
