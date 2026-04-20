# Specs-array prose enforcement gap

**Date:** 2026-04-20
**Source:** `specs/2026-04-12-specs-array-integrity/review-pr-2026-04-20.md` (post-hoc cold review)

## Observation

The `specs-array-integrity` spec's ac-01 — "the `/orb:spec` skill appends the new spec path to the card's specs array" — is enforced only by prose in `plugins/orb/skills/spec/SKILL.md` step 5. On 2026-04-20 the post-hoc review proved this enforcement already failed in production: card `0007-drive-forked-reviews.yaml` shipped with `specs: []` despite a matching spec existing with a correct `**Card:**` anchor in its interview. One orphan in the first 8 days of use.

Manual repair committed alongside this memo (card 0007 now lists `specs/2026-04-20-drive-forked-reviews/spec.yaml`).

## Why it failed

- `/orb:spec` step 5 is prose the agent is asked to follow. No deterministic check.
- The original spec's constraints prohibited write-time hooks — specifically so orbit would not rely on Claude Code hook primitives that may not exist everywhere.
- Without a hook, no safety net exists between "spec written" and "card updated". The agent can simply skip the step silently.
- The review also flagged a secondary leak: step 5.2 ("if no card is identified, skip") reads as a permissive loophole even when an interview is present but its `Card:` line uses a non-canonical format.

## Candidate responses (for future design)

- **Harden `/orb:spec` step 5 into a gate.** Make the skill refuse to consider itself complete until an Edit of the card file has landed, when a Card: line is present. The agent self-checks via Read-after-Edit.
- **Add a reconciliation pass to `/orb:review-pr`.** Before a review-pr finishes, it keyword-scans the card's topic against `specs/` and flags orphans as a finding. This catches misses at the last gate before merge rather than post-hoc.
- **Accept a local hook.** Re-visit the write-time-hook prohibition: a repo-local git pre-commit or post-write check that a newly-added `specs/*/spec.yaml` is referenced by exactly one `cards/*.yaml` is deterministic and cheap.
- **Tighten the "no Card:" skip.** Require an explicit `orphan: true` or `no_card: true` marker in the interview for step 5 to skip, rather than treating a missing `Card:` line as implicit permission.

## Ac-08 literal-reading miss (separate thread)

The same review noted that ac-08 (the grep-fallback note lives in both `/orb:spec` and `/orb:design` SKILL.md files) is not satisfied literally: neither file mentions grep, ripgrep, or fallback; the references live only in the shared `/orb:keyword-scan` skill. The implementer's progress.md argued delegation satisfies ac-08; the cold reviewer disagrees. Either rewrite the AC (future spec) or add a one-liner to both files pointing at `/orb:keyword-scan`.

## Status

Not scoped to a card or spec yet. Held here as a memo pending a design session — likely to become a new card ("Specs-array integrity: deterministic enforcement") or be folded into an existing one.
