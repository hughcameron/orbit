# Implementation Progress

**Spec:** specs/2026-04-20-orbit-artefact-folder/spec.yaml (v1.1)
**Started:** 2026-04-20
**Branch:** orbit-artefact-folder

## Hard Constraints

- [ ] Single all-or-nothing migration prompt
- [ ] Migration lives inside /orb:setup (no separate skill)
- [ ] Rewrite scope is total (incl. quoted evidence in shipped review/progress files)
- [ ] `orbit/` is hardcoded, not configurable
- [ ] No top-level `designs/` folder
- [ ] No bundling of specs-array-prose-enforcement memo
- [ ] Setup remains idempotent
- [ ] session-context.sh gate switches to `-d "orbit"`
- [ ] Migration uses `git mv`, history preserved
- [ ] `cards/memos/` moves to `orbit/cards/memos/`; hook scan path updates

## Acceptance Criteria

- [ ] ac-01: Greenfield creates orbit/ with cards/, specs/, decisions/ subdirs
- [ ] ac-02: Brownfield detects bare dirs and presents single all-or-nothing prompt
- [ ] ac-03: Confirm → git mv all detected dirs under orbit/ in one transaction
- [ ] ac-04: Decline → abort with no filesystem changes
- [ ] ac-05: Idempotent + brownfield-then-idempotent sequence both no-op
- [ ] ac-06: CLAUDE.md snippet names orbit/ artefact locations
- [ ] ac-07: session-context.sh gate = `-d "orbit"` + legacy-layout nudge
- [ ] ac-08: session-context.sh scans orbit/cards/memos, orbit/specs, orbit/cards/*.yaml
- [ ] ac-09: rally-coherence-scan.sh references orbit/ paths (or path-agnostic)
- [ ] ac-10: All 18 plugins/orb/skills/*/SKILL.md files rewritten to orbit/ paths
- [ ] ac-11: progress.md.template uses orbit/ paths
- [ ] ac-12: This repo migrated — no bare dirs at root; all four under orbit/; git history preserved
- [ ] ac-13: `git ls-files | xargs rg` for bare paths returns allow-listed hits only
- [ ] ac-14: cards/memos/ → orbit/cards/memos/; hook still surfaces memo
- [ ] ac-15: README.md reflects new layout
- [ ] ac-16: CHANGELOG.md documents refactor + trade-offs
- [ ] ac-17: .claude-plugin/plugin.json unchanged
- [ ] ac-18: No designs/ folder introduced at any point
- [ ] ac-19: (deferred) Downstream migration verified post-ship
- [ ] ac-20: Dirty-tree handling explicit in setup SKILL.md
- [ ] ac-21: Mixed-state refusal (orbit/ AND bare dir coexist)
- [ ] ac-22: Untracked-files residue report after migration
- [ ] ac-23: Explicit rewrite of cards[].specs arrays, decisions/*.md, cross-refs
- [ ] ac-24: Single migration commit for git log --follow continuity

## Deferred Verification

- **ac-19** (downstream migration) — verified on the first real downstream /orb:setup invocation post-ship. Deferred by design: cannot test against a hypothetical future downstream repo without a dedicated fixture. Tracked here; will be closed with evidence when a real downstream repo upgrades.

## Implementation Notes

Plan:
1. Commit 1 (design artefacts, this branch): card, interview, spec, review-spec, progress.md — groundwork
2. Commit 2 (the migration, ac-24 atomic): git mv × 4, all path rewrites, setup SKILL new behaviour, hook updates, CHANGELOG/README

The atomic commit is required by ac-24 so that `git log --follow` stays unbroken across every renamed artefact. Mixing moves + rewrites + setup logic into one commit is unusual but deliberate.
