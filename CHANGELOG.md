# Changelog

All notable changes to orbit are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.2.18] - 2026-04-17

### Added
- `test_prefix` metadata field for specs — disambiguates AC-to-test mapping across multi-spec projects. Skills `spec`, `spec-architect`, `audit`, `review-pr`, and `implement` all consume the prefix.
- `decisions/0002-ac-test-prefix.md` — documents the choice of explicit spec-scoped prefixes over globally unique IDs or auto-derived slugs.

### Changed
- AC naming guidance now recommends slug-style prefixes (`remat`, `introspect`) over version-like prefixes (`v03`), since `metadata.version` already carries the version.
- `/orb:audit` warns when multiple specs exist but any lack `test_prefix`.

## [0.2.17] - 2026-04-16

### Changed
- `/orb:release` — moved from user-level skill (`~/.claude/skills/release/`) into the orbit plugin. Invoked as `/orb:release` instead of `/release`, freeing the `/release` namespace for project-specific release skills.

## [0.2.16] - 2026-04-16

### Changed
- `/orb:drive` pipeline expanded to 5 stages: Design → Spec → **Review-Spec** → Implement → Review-PR. Every spec now gets reviewed as part of the drive.
- `/orb:drive` guided mode removes intermediate go/no-go gates. Reviews ARE the quality gates. The only interactive pause is a rich final summary (spec review verdict, AC coverage, honest assessment) before PR creation. "Let me read the reviews first" is an explicit option.
- `/orb:drive` supervised mode gates now include richer context (AC counts, finding summaries) instead of bare "greenlight?" prompts.
- `/orb:review-spec` replaced with progressive 3-pass model (decision 0001). Pass 1 (structural scan) always runs. Pass 2 (assumption & failure analysis) triggered by findings or content signals. Pass 3 (adversarial review) triggered by structural concerns. Depth scales with findings, not upfront classification.
- Removed risk tier classification (HIGH/STANDARD/SKIP) from `/orb:spec`. Every spec gets reviewed — the progressive model makes tier gating unnecessary.

### Added
- `decisions/0001-progressive-spec-review.md` — first orbit decision record. Documents why tier-based review gating was replaced with progressive review.

## [0.2.15] - 2026-04-15

### Added
- `/orb:drive` Disposition section — defines the agent's working stance: find the way through, treat negatives as constraints on the next iteration, push past the first plateau. Ported from prior agent research disposition.
- Semantic escalation triggers (recurring failure mode, contradicted hypothesis, diminishing signal) alongside the mechanical 3-iteration budget. An honest agent may escalate before the budget is spent.
- Escalation summaries now include "What would have to be true" — what assumptions need revisiting for a future attempt to succeed.

## [0.2.14] - 2026-04-15

### Added
- `/orb:drive` — agent-driven card delivery. Takes a card path and autonomy level (full/guided/supervised), then drives the full orbit pipeline (design → spec → implement → review-pr) as a single inline session. Tracks state in `drive.yaml` for session resumption, with a 3-iteration budget before escalation. Thin cards (< 3 scenarios) are refused for full autonomy.
- `session-context.sh` now detects `drive.yaml` and surfaces active drive state (card, autonomy level, iteration, status, next action) at session start. Escalated drives show a distinct message.

## [0.2.13] - 2026-04-13

### Added
- `/orb:card` "What Gets Closed" section — specs are the closure unit; cards are never closed. A NO-GO result updates the card's `goal` and `maturity`, not its existence.
- `/orb:implement` step 7 "When a Spec Produces a NO-GO" — guidance to record evidence in `progress.md`, mark ACs with the result, update the card's goal, and loop back to `/orb:design` with the new evidence.

## [0.2.12] - 2026-04-12

### Added
- `/orb:keyword-scan` — shared technique for keyword-based search across orbit artifacts. Extracts 5–8 distinctive domain terms from a card, spec, or interview; builds a ripgrep alternation pattern; falls back to `grep -rl` in environments without `rg`. Referenced by all workflow skills rather than inlining the pattern.
- `/orb:spec` now appends the new spec path to the card's `specs` array after saving (write-time enforcement). Agents downstream that read the array get a complete work trail without manual upkeep.
- `/orb:design` reconciles the card's `specs` array against a keyword scan of `specs/` before the session starts — surfaces orphaned specs the author can confirm to link.
- `/orb:distill` checks `cards/` for existing capability overlap before drafting new cards.
- `/orb:card` checks `cards/` and `specs/` for overlap before finalising a new card.
- `/orb:discovery` searches `specs/` and `decisions/` for prior art before the interview begins.
- `/orb:implement` searches the project source for existing code and patterns related to the spec's ACs.
- `/orb:review-pr` searches `decisions/` for architectural choices the implementation should respect.

### Changed
- README workflow diagram shows the multi-spec loop: dashed edge from Ship back to Design when the card goal is not yet met.
- End-to-end walkthrough describes iterative goal pursuit across multiple specs.

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
