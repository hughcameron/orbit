# Lean pass as a recurring discipline

**Date:** 2026-05-01
**Source:** session-close audit at end of orbit 0.4.0 release

## Observation

At the close of the 0.4.0 release I had 7 pinned bd memories. Three of them (`drive-skill-beads-rewrite-orbit-6da-2-drive`, `implement-skill-beads-rewrite-shipped-orbit-6da-1`, `rally-skill-beads-collapse-orbit-6da-3-rally`) were "shipped" markers — useful as in-flight signals while the bead-native rewrite was underway, but reference material once the work landed. A fourth (`0-4-0-rollout-plan`) was a planning artefact whose must-includes had all shipped. A fifth (`card-0016-...-shipped`) was subsumed by the natural consolidation of the four orbit-6da.x rewrites.

I compressed all five into one `0-4-0-shipped` memory carrying the same information at lower volume, and pinned a separate `next-session-pilot-rollout` for the actually-actionable signal. Memory count: 7 → 3.

The consolidation was obvious in hindsight but I would not have done it without the explicit close-out audit. The pattern generalises: artefacts accrete; signal-to-volume falls; periodic compression restores it.

## Where this pattern shows up

Beyond bd memories, the same accretion happens in:

- **Decision register.** Decisions sit in `accepted` status long after they've been superseded in practice. Card 0016 caught one (decision 0002 only partially superseded for review-pr scope). A periodic audit would catch others.
- **Card maturity.** Cards advance through `planned → emerging → established` but rarely backslide or get refined when the goal narrows. Some cards sit at `established` with goals that no longer reflect current capability.
- **Old branches.** Five unmerged local branches surfaced in the 0.4.0 close-out (`cards-0001-0003-specs-and-implement`, `cards-0001-memos`, `cards-0002-distill`, `feat/ac-test-prefix`, `refactor/design-intent-boundary`). I don't know if they're parked or abandoned. Without periodic review, the branch list grows unbounded.
- **Docstrings and inline comments.** Inline comments calibrated for an old code shape often outlast the refactor. The cycle-1 review of card 0016 caught several stale `§1.x` cross-references in `drive/SKILL.md` only because a fresh-context reviewer looked.
- **Dead code.** Conditionals that handle removed features, fallback paths for removed substrates, parameters never set by callers. The 0.3.3 → 0.4.0 cutover removed the snapshot bridge but the *concept* of "old vs new substrate" leaves behind dead branches in any project that imports orbit pipeline conventions.
- **Old beads.** Closed beads accumulate. `bd ready` filters them out, but `bd list` doesn't. Status-based pruning could keep the active set scannable.
- **Memos.** Memos accumulate in `orbit/cards/memos/`. The 0.4.0 close-out caught one memo (`2026-04-20-specs-array-prose-enforcement-gap.md`) that was already dissolved by 0.4.0 but never deleted. `/orb:distill` runs ad-hoc; nothing forces a memo lifecycle review.
- **CLAUDE.md files.** Project CLAUDE.md files accrete instructions across sessions and rarely shed them. The FineType repo crossed the threshold — Claude Code now warns at session start: `⚠ Large CLAUDE.md will impact performance (41.5k chars > 40.0k) · /memory to edit`. The harness flags it, but only as a soft nudge at startup. Without a deliberate lean pass, CLAUDE.md drifts past the performance threshold and stays there — every session pays the tax until someone audits.

## What "lean" might mean

Lean is the inverse operation of distill. Distill *extracts* signal from raw input (memos → cards). Lean *removes* artefacts that no longer carry signal (stale memories → forgotten; superseded decisions → status updated; dead code → removed; closed-and-summarised beads → archived).

Distill creates capabilities. Lean preserves capacity to reason about the current state.

## Open questions

- **Cadence.** Per release? Per quarter? On a "memory count > N" trigger? On a "feels heavy" trigger? Some signals are calendar-shaped (quarterly review), others are state-shaped (memory > 10).
- **Surface.** A `/orb:lean` skill that runs the audit and proposes deletions/compressions interactively? A scheduled background agent? A discipline embedded in `/review-session` or `/orb:release` (lean as part of every release)? Or just a habit captured in CLAUDE.md?
- **Scope per pass.** All-of-the-above per session, or one domain per pass (memories one week, decisions next, branches the week after)?
- **Reversibility.** `bd forget` is destructive; `git branch -D` is destructive; deleting a memo is destructive. Lean needs a "propose then act" shape rather than "act and surface" — the cost of a wrong delete is higher than the cost of a wrong distill.
- **Co-evolution with /orb:distill.** Distill already runs over memos and produces cards. Could lean run *first* (drop dissolved memos) and then distill (extract from what remains)? That would keep distill's input surface honest.

## Status

Held as a memo for `/orb:distill` to consider. Likely candidate to become its own card (`/orb:lean` as a workflow capability, dual to `/orb:distill`).

If the card hypothesis holds, the next obvious driving question is: what's the smallest first version that earns its keep? Probably *just* memory compression (the trigger that surfaced this memo) — narrower scope ships faster and proves the shape.
