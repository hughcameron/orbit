# Changelog

All notable changes to orbit are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.2.11] - 2026-04-11

### Changed
- `/orb:design` now reads the card's `specs` array as cumulative progress. Presents what each prior spec contributed and anchors the session on the gap between current state and goal.
- Design sessions no longer assume linear spec progression — specs may enhance a capability from different angles (infrastructure, data quality, tooling, adjacent work). The session surfaces which path the author intends.
- Interview record template includes goal, prior spec summary, and gap context.

### Added
- `CHANGELOG.md` backfilled from v0.2.0 through v0.2.10 (added in this release cycle).

## [0.2.10] - 2026-04-09

### Added
- `goal` field on cards — specific, measurable target at the current maturity. `so_that` is timeless (why); `goal` is current (what success looks like now). Goals evolve as the capability matures; git history tracks the progression.
- Sprint goal structure in CLAUDE.md — `/orb:setup` scaffolds a `Current Sprint` section listing the objective and card goals.
- README documents goals and sprint concepts.

## [0.2.9] - 2026-04-09

### Changed
- Replaced `priority` field (now/next/later) with `maturity` (planned/emerging/established) on cards. Cards describe capability state, not work priority.

### Added
- `specs` array on cards — lists the specs that have addressed each capability, giving a clear trail of work done.

## [0.2.8] - 2026-04-09

### Changed
- Distill now uses a staged **Draft → Review → Write** flow instead of per-card approve/edit/reject.
- All cards are drafted first and presented as a numbered batch.
- The agent surfaces overlaps, gaps, and low-confidence cards during review.
- Batch feedback (merge, split, drop, rename) replaces individual card gates.
- Nothing is written to disk until the author explicitly says "write."

## [0.2.7] - 2026-04-08

### Changed
- Cards are living documents — the lifecycle table (Open/In progress/Delivered/Closed) replaced with "Cards Are Living Documents" section.
- `cards/done/` directory removed from prescribed structure.
- Distill now accepts files, directories, or natural-language scope descriptions (not just a single file path).
- First-principles lens is always applied: "what does this product do?" not "what's planned next?"

### Added
- `CLAUDE.md` for the orbit repo — establishes that sessions here are about workflow refinement.

## [0.2.6] - 2026-04-08

### Changed
- Renamed `/orb:init` to `/orb:setup` to avoid collision with built-in `/init` command.

## [0.2.5] - 2026-04-08

### Changed
- Use "the author" for the human driving the workflow; reserve "the user" for end-users of the software being built.

## [0.2.4] - 2026-04-07

### Added
- `/orb:audit` skill — audit AC-to-test traceability across specs, finding untested code ACs, orphaned test prefixes, and coverage gaps.
- `ac_type` classification (code/doc/gate/config) for acceptance criteria.

## [0.2.3] - 2026-04-06

### Added
- `/orb:memo` skill — quickly jot rough ideas as freeform markdown in `cards/memos/`.
- README rewritten for orbit 0.3 — end-to-end walkthrough, "four ways in" section.

### Changed
- Evidence hierarchy added to interviewer, design, implement, and spec-architect skills.
- Implement skill proceeds after checklist without waiting for confirmation.

## [0.2.2] - 2026-04-05

### Fixed
- Implement skill: clarify that step 4 proceeds to write code after presenting the checklist.

## [0.2.1] - 2026-04-04

### Added
- Specs for cards 0001–0003.
- Implemented 0001-memos: freeform idea capture in `cards/memos/`.
- Implemented 0002-distill: extract cards from unstructured input.
- Implemented 0003-implement: pre-flight spec check with AC checklist.

### Added
- Cards for memos (0001), distill (0002), and implement (0003) features.

## [0.2.0] - 2026-04-03

### Added
- All 18 orbit workflow skills, README, and LICENSE.
- References field on cards; card-aware interview mode.
- Split interview into separate design and discovery skills.
- SessionStart hook for workflow context injection.

### Removed
- Evaluate and evolve skills (superseded by audit and design).
- `disable-model-invocation` from workflow skills.
