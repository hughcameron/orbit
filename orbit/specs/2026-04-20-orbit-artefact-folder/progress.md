# Implementation Progress

**Spec:** orbit/specs/2026-04-20-orbit-artefact-folder/spec.yaml (v1.1)
**Started:** 2026-04-20
**Branch:** orbit-artefact-folder

## Hard Constraints

- [x] Single all-or-nothing migration prompt — setup SKILL.md §3c
- [x] Migration lives inside /orb:setup (no separate skill) — §3
- [x] Rewrite scope is total (incl. quoted evidence in shipped review/progress files) — bulk sed across 70 tracked files
- [x] `orbit/` is hardcoded, not configurable — SKILL.md §6 "Why `orbit/`?"
- [x] No top-level `designs/` folder — none created
- [x] No bundling of specs-array-prose-enforcement memo — memo retained untouched in orbit/cards/memos/
- [x] Setup remains idempotent — SKILL.md §5 + "Idempotency" section
- [x] session-context.sh gate switches to `-d "orbit"` — done at top of script
- [x] Migration uses `git mv`, history preserved — `git mv cards orbit/cards` etc.
- [x] `cards/memos/` moves to `orbit/cards/memos/`; hook scan path updates — session-context.sh L34 now reads `orbit/cards/memos`

## Acceptance Criteria

- [x] ac-01: Greenfield creates orbit/ with cards/, specs/, decisions/ subdirs — SKILL.md §2
- [x] ac-02: Brownfield detects bare dirs and presents single all-or-nothing prompt — SKILL.md §3a-3c
- [x] ac-03: Confirm → git mv all detected dirs under orbit/ in one transaction — SKILL.md §3d
- [x] ac-04: Decline → abort with no filesystem changes — SKILL.md §3e
- [x] ac-05: Idempotent + brownfield-then-idempotent sequence both no-op — SKILL.md §5 + Idempotency section
- [x] ac-06: CLAUDE.md snippet names orbit/ artefact locations — SKILL.md §6 snippet
- [x] ac-07: session-context.sh gate = `-d "orbit"` + legacy-layout nudge — session-context.sh L21-31
- [x] ac-08: session-context.sh scans orbit/cards/memos, orbit/specs, orbit/cards/*.yaml — bulk sed applied; verified in L34, L62, L173, L177
- [x] ac-09: rally-coherence-scan.sh references orbit/ paths — L3 now cites `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml`
- [x] ac-10: All plugins/orb/skills/*/SKILL.md files rewritten to orbit/ paths — bulk sed pass (setup SKILL rewritten in full)
- [x] ac-11: progress.md.template uses orbit/ paths — bulk sed pass
- [x] ac-12: This repo migrated — no bare dirs at root; all four under orbit/; git history preserved via git mv
- [x] ac-13: `git ls-files | xargs rg` for bare paths returns allow-listed hits only — verified: CHANGELOG entry, setup SKILL (brownfield detector), spec/interview/review-spec (describe legacy state), memo (retained for history)
- [x] ac-14: cards/memos/ → orbit/cards/memos/; hook still surfaces memo — session-context.sh scans orbit/cards/memos
- [x] ac-15: README.md reflects new layout — verified: all prose and code blocks use orbit/ prefix
- [x] ac-16: CHANGELOG.md documents refactor + trade-offs — Unreleased entry added
- [x] ac-17: .claude-plugin/plugin.json unchanged — manifest untouched
- [x] ac-18: No designs/ folder introduced at any point — none created
- [ ] ac-19: (deferred) Downstream migration verified post-ship — see Deferred Verification
- [x] ac-20: Dirty-tree handling explicit in setup SKILL.md — §3c includes the dirty-tree note
- [x] ac-21: Mixed-state refusal (orbit/ AND bare dir coexist) — SKILL.md §4
- [x] ac-22: Untracked-files residue report after migration — SKILL.md §3b + §3f
- [x] ac-23: Explicit rewrite of cards[].specs arrays, decisions/*.md, cross-refs — card 0008 specs array fixed; bulk sed covered decisions and cross-refs
- [x] ac-24: Single migration commit for git log --follow continuity — planned Commit 2 atomic

## Deferred Verification

- **ac-19** (downstream migration) — verified on the first real downstream /orb:setup invocation post-ship. Deferred by design: cannot test against a hypothetical future downstream repo without a dedicated fixture. Tracked here; will be closed with evidence when a real downstream repo upgrades.

## Implementation Notes

Plan executed:
1. Commit 1 (a773d39): design artefacts (card, interview, spec v1.1, review-spec, progress.md) — landed
2. Commit 2 (this commit): atomic migration per ac-24
   - `mkdir orbit/`
   - bulk sed rewrites (70 tracked files) leaving 9 allow-listed files with bare-path references
   - `git mv cards orbit/cards && git mv specs orbit/specs && git mv decisions orbit/decisions && git mv discovery orbit/discovery`
   - setup SKILL.md fully rewritten: 4-state machine (greenfield / brownfield / idempotent / mixed)
   - session-context.sh fully rewritten: orbit/ gate + legacy-layout nudge
   - CHANGELOG.md Unreleased entry
   - CLAUDE.md and README.md verified (bare → orbit/ prefixes)
   - card 0008 specs array fixed to orbit/specs/...

The atomic commit is required by ac-24 so that `git log --follow` stays unbroken across every renamed artefact.
