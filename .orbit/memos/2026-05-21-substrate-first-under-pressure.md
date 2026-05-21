# Substrate-first posture fails under pressure

**Date:** 2026-05-21
**Source:** Finetype session triage — downstream agent diagnosed legacy-spec parse failure without ever reading `.orbit/memos/` or running `orbit audit conformance --json`

## What surfaced

A session in `meridian-online/finetype` hit the legacy-spec brick-wall: 53 specs + `_template/spec.yaml` carry `constraints:` (rejected by orbit 0.4.26's strict parser), plus missing `id:` per AC and string-shaped ACs. Session prime trips on parse failure.

The agent in that session reasoned from scratch:

- Read `canonicalise --help`, ran `--reconcile --dry-run`, observed "0 would rewrite, 54 parse-failed"
- Diagnosed the deeper schema issues from the error messages
- Concluded: "needs richer reconcile rules; not in scope right now; want me to file as memory + follow-up?"

What it never did:

- Run `orbit audit conformance --json` — the canonical structured-findings surface shipped per spec 2026-05-19-workflow-conformance
- Grep `.orbit/memos/` — the two memos at 2026-05-16 and 2026-05-20 *already* name this gap, with the exact validation set the agent re-derived
- Read METHOD.md's substrate rules — which list conformance as on-demand

The substrate had the answer pre-written. The agent re-derived it in ~90 seconds.

## Three threads

One observation, three independent gaps:

1. **Posture** — METHOD.md already says "Before reasoning about how a subsystem works, grep the code tree and `docs/` for it." Agents under pressure default to first-principles reasoning anyway. The architectural-investigation line is necessary but not sufficient.

2. **Coverage** — `orbit audit conformance --json` doesn't currently emit a `parse_failed_spec` finding family. If the finetype agent *had* run it, the verb would have returned silence on the actual problem. The discovery chain breaks here even with the right posture.

3. **Discoverability** — METHOD.md mentions conformance as on-demand but doesn't name *trigger conditions* — when to reach for it, what symptom routes to it, what chain follows from it. The agent has no breadcrumb to follow.

## Why prose-only fixes won't carry

METHOD.md already carries "Substrate beats extrapolation" *and* the "Architectural-investigation posture" line. Both were in the finetype agent's context when it bypassed them. Adding a fourth runbook entry in the same channel is more rules for the same bypass — visible on every load, ignored under pressure.

For reference, the prose form would look like this:

```
**When session prime trips or substrate-shape errors surface:**
1. `orbit audit conformance --json` — read the structured findings envelope
2. Each finding carries a `remediation.verb` — run it, don't translate it
3. Common chain: parse-failed spec → `orbit canonicalise --reconcile --dry-run` → `--reconcile`
4. If reconcile bails (post-clean parse still fails), the gap is in canonicalise's registry — file a memo, don't hand-edit specs
```

It's correct. It's also low-leverage on its own — the same channel that already failed.

## Where the leverage is

Mechanical, not prose. Two surfaces where the breadcrumb can meet the agent where it already is:

- **Failure surfaces emit the breadcrumb at the point of pain.** When `orbit session prime` or `orbit canonicalise --reconcile` hits parse failure, the error itself prints "N specs failed to parse — run `orbit audit conformance --json`". ~5 lines per surface. Agents can't skip an error message the way they skip a doc.
- **Conformance gains a `parse_failed_spec` finding family.** Without this, even an agent that follows the breadcrumb gets silence. Coverage extension under card 0039.

These two siblings + the richer-reconcile schema migration are the actual bundle. Each piece is necessary; none stands alone.

## Recommendation

Distill `[[2026-05-16-richer-reconcile-rules]]` into a card. In that card's design pass, scope three siblings together:

1. The schema migration the memo names (filename-derived `id`, scalar-AC wrapping, layout discrimination)
2. The `parse_failed_spec` conformance finding family with remediation verb `orbit canonicalise --reconcile`
3. The failure-surface breadcrumbs on `orbit session prime` and `orbit canonicalise --reconcile`

Ship together. The METHOD.md runbook block becomes an optional postscript inside the same spec — included if it earns its place, not as a standalone edit.

Thread 1 (posture) stays observational. If a second downstream session bypasses the substrate after the breadcrumbs ship, the pattern earns its own card.

## Related

- `[[2026-05-16-richer-reconcile-rules]]` — what reconcile needs to learn to handle
- `[[2026-05-20-reconcile-from-idempotent-setup]]` — when reconcile should be offered
- `[[2026-05-19-conformance-park-signal-gap]]` — already-resolved sibling, shipped 2026-05-20
- Card 0039-workflow-conformance — defines the conformance surface this memo proposes extending
- METHOD.md "Architectural-investigation posture" line — the rule the agent ignored

## Status

Memo only. Proceeds into `/orb:distill` on `[[2026-05-16-richer-reconcile-rules]]`, where the three siblings get scoped together at design time.
