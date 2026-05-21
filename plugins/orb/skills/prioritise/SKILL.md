---
name: prioritise
description: Session-start priority synthesis — re-derive a ranked Decision Brief live from workflow conformance, the session-prime envelope, and recent memories. Read-only — the author picks, the author runs.
user-invocable: true
created_by: claude
created_at: 2026-05-21
---

# /orb:prioritise

Re-derive a ranked priority brief live from substrate signals. Returns the top 5 priorities — each with what, why, effort, and next-action verb. The author picks; the author runs. The skill never auto-executes.

## Usage

```
/orb:prioritise
```

No arguments. Invocable at session start or any time mid-session — the output reflects substrate state at the moment of invocation, not a frozen handover.

## Why this exists

Session handovers freeze priorities at the moment they're written. By the next session, memos accumulate, cards age, conformance findings change. This skill re-derives the same synthesis live from the same substrate inputs the close-time handover used. The data is already there; the agent's job is the compression.

This is the author-level interaction layer (pillar 1) over the conformance verb and session-prime envelope.

## Substrate inputs

Read these three sources live — do not extrapolate from git log or filesystem scans.

1. **`orbit audit conformance --json`** — findings carry `severity` (HIGH/MEDIUM/LOW), `subsystem`, `subject`, `evidence`, and `remediation.verb`.
2. **`orbit session prime`** — open specs, recent memories, prior handover distillate.
3. **Recent memories** (from the session-prime envelope or `orbit memory list --recent`) — keep only those signalling deferred decisions, calendar deadlines, or unresolved questions.

## Ranking algorithm

Priorities are ranked deterministically — two consecutive invocations on identical substrate must produce byte-identical ordering:

1. Conformance severity: `HIGH` > `MEDIUM` > `LOW`
2. Within severity: memo staleness in days, descending (older = higher)
3. Within staleness: open-spec age, descending (older = higher)
4. Ties broken by id ascending

## Next-action verb

For each priority drawn from a conformance finding, surface its `remediation.verb` verbatim — no translation, no paraphrase. The author runs the verb directly.

When a finding lacks `remediation.verb` (forward-compatible severity-only or info-only findings), surface the finding's `evidence` field verbatim and tag the item `manual action`.

For priorities drawn from non-conformance sources (memos, open specs), the next-action is the verb the author would type — e.g. `/orb:distill .orbit/memos/<file>.md`, `/orb:drive <spec-id>`, `/orb:tabletop <card-id>`.

## Output shape

Top 5 only. Overflow surfaces as a count line, never enumerated:

> `+ 3 more deferred (run \`orbit audit conformance --json\` for the full list)`

Each priority is at most 4 lines:

```
N. <one-line what>
   why: <one-line stake/blocker>
   effort: <S | M | L>
   next: <verb verbatim>
```

- **`S`** — ≤15 min
- **`M`** — one session
- **`L`** — multi-session

Total brief: **≤20 lines or ~500 tokens**. Cite memory keys, conformance subjects, and file paths the author can open on demand — do NOT inline substrate content.

## Empty-substrate fallback

When `orbit audit conformance --json` returns zero findings and recent memos / open specs are exhausted:

1. Surface planned-empty cards — those with `maturity: planned` AND `specs: []`. Each becomes a priority with next-action `/orb:tabletop <card-id>`. Use `orbit overview` to list orphans / planned-empty cards.
2. If no runner-ups exist either, output exactly: `no priorities — substrate is clean and the backlog is exhausted`.

Never return an empty brief without an explanation.

## Algorithm

1. Run `orbit audit conformance --json`. Collect findings.
2. Run `orbit session prime`. Collect open specs and recent memories.
3. Filter memories for actionable signals (deferred deadlines, blocked decisions, unresolved questions). Drop status-recap and session-close summaries.
4. Apply the ranking algorithm above.
5. Compose the brief — top 5 entries in the bounded shape.
6. If no priorities surfaced, apply the empty-substrate fallback.
7. Return. Do **not** execute any next-action verb.

## What this is NOT

- **Not `orbit overview`.** That verb produces a status snapshot (cards-by-maturity, orphans, most-connected card). This skill produces a ranked ACTION plan keyed off the conformance findings + session envelope.
- **Not auto-executing.** Enforcement is via this SKILL.md's prose contract — there is no hook or tool-permission gate. The skill matches existing read-only surfaces like `orbit overview`. If the author wants the priority run, they type the next-action verb themselves.
- **Not session-start frozen.** Invoke any time. The output is live re-derivation.

## Discipline

- **Read-only.** No `orbit task open`, no `orbit memory remember`, no `orbit spec note`. The skill never writes.
- **Compress, don't dump.** The author can run `orbit audit conformance --json` themselves if they want raw findings — this skill spares them the parse.
- **Cite, don't paraphrase.** Name memory keys, conformance subjects (`<subsystem>/<subject>`), spec ids, and card slugs verbatim. The author drills down by opening the cited substrate, not by re-reading prose.
- **No fanfare.** No header like "Decision Brief" or "Executive Summary". The numbered list IS the brief.
