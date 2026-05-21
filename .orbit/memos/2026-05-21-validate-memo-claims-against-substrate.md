# Validate memo claims against the substrate before distilling

**Date:** 2026-05-21
**Source:** Spec 2026-05-21-richer-reconcile-rules ac-05 implementation finding (assumption reversal)

## Observation

`.orbit/memos/2026-05-16-richer-reconcile-rules.md` named "missing required top-level fields — pre-orbit-state specs lack `id` (~36 specs)" as the dominant brownfield failure shape. That claim seeded the design note, the spec's goal sentence, and the initial AC set for spec 2026-05-21-richer-reconcile-rules.

When ac-05's validation ran against the actual validation-set repo (meridian-online/finetype), the dry-run revealed: **53 of 54 specs were missing `status`, not `id`**. The id-synthesis rule wasn't the load-bearing fix; the status-synthesis rule was. The composition-shape memo had inverted the diagnosis.

The implementation caught this via the working-rules escalation ("assumption reversals require escalation"), the author authorised broadening the spec inline, and two more rules shipped under the same spec — but only because the implementing agent ran the validation early. If ac-05 had been a final-stage check, the spec would have closed with a falsified premise.

## The pattern

The memo was authored from a small sample — five specs the agent looked at when first surfacing the gap. Five-spec extrapolation became a "~36 specs" claim. The actual cross-tree composition wasn't measured; it was inferred.

This is the standard distill-without-evidence shape: a real observation (some specs lack `id`) gets quantified by extrapolation (so probably ~36 of them) and the extrapolation becomes load-bearing in downstream design.

## Suggested discipline (light-touch)

For memos that claim a quantitative shape ("N specs do X", "the dominant pattern is Y") and feed a brownfield design pass, the distill phase should run a one-command sanity check against the substrate before promoting to a card:

```
# Quick composition check before promoting a brownfield memo
orbit canonicalise --reconcile --dry-run --root <validation-tree>
```

The dry-run's disposition list shows which rules fire where — a fast O(seconds) way to validate the memo's claim before it sediments into a spec. If the disposition list doesn't match the memo's diagnosis, the memo needs a revision pass, not a promotion.

This isn't a process change — it's a habit at the distill boundary. When a memo names a tree of files and a fix-shape, the agent confirms by running the verb the fix-shape would invoke.

## Why this isn't 2026-05-21-richer-reconcile-rules' scope

That spec ships rules. This memo is about *how* future brownfield memos should be validated before distill. Separate concern; possibly a small SKILL.md edit on `/orb:distill` (add a "verify quantitative memo claims" step), possibly an item for the next session-close to surface.

## Related

- `[[2026-05-16-richer-reconcile-rules]]` — the memo whose claim was inverted
- `.orbit/specs/2026-05-21-richer-reconcile-rules/progress.md` — the validation run that caught the inversion
- `[[2026-05-21-substrate-first-under-pressure]]` — adjacent posture concern (agents bypass substrate); this finding is the same shape inverted (memos claim about substrate without checking)

## Status

Memo only. Worth raising at the next `/orb:distill` SKILL.md review pass — a one-line "verify quantitative claims" step in the distill phase would catch the pattern mechanically.
