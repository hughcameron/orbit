# Implementation Progress

**Spec:** .orbit/specs/2026-04-12-specs-array-integrity/spec.yaml
**Started:** 2026-04-12

## Hard Constraints
- [x] Write-time update lives in /orb:spec, not a separate hook
- [x] Keyword scan uses a single rg (or grep fallback) command, not full spec reads
- [x] Orphaned specs surfaced for author confirmation — never auto-linked
- [x] Works in environments without ripgrep — grep -rl as fallback
- [x] Keyword extraction produces 5-8 distinctive terms

## Acceptance Criteria
- [x] ac-01: /orb:spec appends new spec path to card's specs array — added as step 5
- [x] ac-02: /orb:spec identifies card from interview's Card reference line — step 5.1
- [x] ac-03: /orb:design extracts 5-8 keywords from card — references keyword-scan technique
- [x] ac-04: /orb:design runs single rg -l with alternation pattern — references keyword-scan technique
- [x] ac-05: Falls back to grep -rl when rg unavailable — documented in keyword-scan technique
- [x] ac-06: /orb:design compares hits against specs array, surfaces orphans — reconciliation steps 2-4
- [x] ac-07: Author-confirmed orphans appended to card's specs array — reconciliation step 4
- [x] ac-08: Fallback note in both skills documents grep as alternative — keyword-scan is the shared reference

## Scope Expansion

Extracted the keyword scan into a shared technique (`/orb:keyword-scan`) and wired it into 6 skills:

| Skill | Search target | Purpose |
|-------|--------------|---------|
| `/orb:design` | `.orbit/specs/` | Orphaned specs not in card's specs array |
| `/orb:distill` | `.orbit/cards/` | Overlap with existing capabilities |
| `/orb:card` | `.orbit/cards/`, `.orbit/specs/` | Overlap check before creating |
| `/orb:discovery` | `.orbit/specs/`, `.orbit/choices/` | Prior art on the topic |
| `/orb:implement` | Project source | Existing code/patterns to build on |
| `/orb:review-pr` | `.orbit/choices/` | Decisions the PR should respect |
