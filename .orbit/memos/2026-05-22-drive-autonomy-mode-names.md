# Drive autonomy mode names — `guided` vs `supervised`

Surfaced mid-tabletop on card 14 (default-merge-after-review). Filed as a follow-up to card 0005 (`drive`), not in scope for the 14 spec.

## Observation

The three autonomy modes on `/orb:drive` differ on **gate frequency across the pipeline**, but the names suggest "how watched" rather than "how often":

| Mode | Author gates | Where |
|------|-----|------|
| `full` | zero | nowhere (today pauses only for PR merge) |
| `guided` (default) | one | post-review-pr APPROVE only |
| `supervised` | one per stage | promote, design, spec, implement, review-spec APPROVE, review-pr APPROVE |

`guided` and `supervised` both connote "watched" — the real axis is *count of author gates*, not *intensity of supervision*. On first read it's easy to think they're synonyms.

## Why not collapse them

They serve genuinely different use cases:
- One-gate (`guided`) — operator trusts the reviews and wants one confirmation before PR creation.
- Per-stage (`supervised`) — operator wants to inspect at every stage handoff (promote, design, spec output, implement, etc.).

Collapsing would lose the per-stage mode.

## Candidate fixes

- Rename to surface the axis: e.g. `none` / `final-gate` / `per-stage`, or `auto` / `gated` / `step-by-step`.
- Or keep names, add a one-line explainer at every site where the modes are introduced (CLI help, drive SKILL.md table, prompts).
- Or document the axis explicitly in the drive SKILL.md table header (e.g. column "Author gates per drive").

## Why not pursued in card 14's tabletop

Card 14's contract treats `guided` and `supervised` identically (both retain author-confirms-merge), so the naming question doesn't block the merge-after-review work. Widening scope here would slow card 14 without serving its goal.

## Next step

Distill into card 0005 (`drive`) as a refinement, or open a fresh card if the rename touches enough surface to warrant one. Probably card 0005 refinement — same capability, sharpening the mode taxonomy.
