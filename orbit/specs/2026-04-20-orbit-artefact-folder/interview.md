# Design: Consolidated orbit artefact folder

**Date:** 2026-04-20
**Interviewer:** Nightingale
**Card:** cards/0008-consolidated-orbit-artefact-folder.yaml

---

## Context

**Card:** *Consolidated orbit artefact folder* — 6 scenarios, goal: "Orbit-workflow artefacts in any project live under a single orbit/ folder; no cards/, specs/, decisions/, or discovery/ directory appears at the repo root".

**Prior specs:** none (`specs: []`). First spec against this card.

**Gap:** The card is clear on *what* — four top-level artefact directories move under `orbit/`. The open questions were *how*: setup-skill behaviour on brownfield, rewrite scope for live vs evidence files, downstream migration story, and whether to bundle an adjacent hardening memo.

**Blast radius from repo audit:**

- 93 occurrences of bare `cards/|specs/|decisions/|discovery/` paths across 18 files
- Heaviest: `plugins/orb/skills/rally/SKILL.md` (26 hits), then `design` (8), `distill` (7), `setup` (7), `discovery` (6), `session-context.sh` (7)
- `session-context.sh` gates on `-d "specs" || -d "cards"` as its "project uses orbit" detector — this gate moves to `-d "orbit"`
- `/orb:setup` currently creates `cards/`, `specs/`, `decisions/` (not `discovery/`) at repo root and writes a CLAUDE.md snippet naming those three

**Card-level note (pre-decided):** No top-level `designs/` folder. Per rally SKILL.md §4a/§6a and `/orb:design`, design artefacts live inside `<spec_dir>/`. This spec does not introduce `designs/`.

## Q&A

### Q1: Brownfield setup behaviour
**Q:** How should `/orb:setup` behave when invoked on a repo that already has bare `cards/`, `specs/`, `decisions/`, or `discovery/` at the root (brownfield with pre-refactor orb state)?
**A:** Detect + interactive migration. Setup detects the bare dirs and asks the author to confirm before moving them under `orbit/`.

### Q2: Rewrite scope
**Q:** How far should the rewrite of bare-path references reach?
**A:** Everything in the repo. Sed across every tracked file. No exclusions for historical artefacts.

### Q3: Downstream migration story
**Q:** What's the migration story for projects downstream that already use orb (not this repo)?
**A:** `/orb:setup` auto-migrates. The brownfield branch of setup (Q1) IS the downstream story — one mechanism, one test surface. No separate `/orb:migrate-layout` skill.

### Q4: Bundle specs-array-gate hardening
**Q:** Should this spec bundle the specs-array-prose-enforcement gate hardening (memo 2026-04-20)?
**A:** No — keep scope clean. The memo stays on the shelf for its own card.

### Q5: Interactive migration granularity
**Q:** For the interactive migration prompt, what granularity?
**A:** Single all-or-nothing prompt. One confirmation: 'Detected N bare orbit dirs. Move them under orbit/? (y/n)'. If yes, all four move in one `git mv` transaction. If no, setup aborts. No per-directory partial states. Dirty-tree handling deferred to implementation — the user did not require a dirty-tree refusal.

### Q6: Evidence-files clarification
**Q:** Does "rewrite everything" (Q2) include the quoted evidence blocks in shipped `review-pr-*.md` and `progress.md` files, given that rewriting them silently falsifies the historical evidence the review model relies on?
**A:** Yes, rewrite everything including quoted evidence. The migration commit itself becomes the audit record that these paths used to be at the repo root. Clean end-state prioritised over historical quote fidelity.

---

## Summary

### Goal

All orbit workflow artefacts (cards, specs, decisions, discovery) live under a single top-level `orbit/` folder. No artefact directory appears at the repo root. `/orb:setup` creates `orbit/` on greenfield and migrates bare directories interactively on brownfield. A grep for bare paths across any live tracked file returns nothing after the refactor.

### Constraints

- **Single all-or-nothing migration.** Brownfield setup prompts once; all four directories move in one `git mv` transaction or none do. No per-directory partial state.
- **Migration in `/orb:setup`, not a separate skill.** The brownfield branch of setup is the downstream migration story. One mechanism, one test surface.
- **Full rewrite, no exclusions.** Every tracked file that references bare `cards/|specs/|decisions/|discovery/` paths — skills, scripts, hooks, shipped review artefacts, progress files, decisions, cards, memos, CLAUDE.md, README, CHANGELOG — gets rewritten. The migration commit is the audit record.
- **`orbit/` is hardcoded.** Not configurable. One folder name, one convention.
- **Scope does not bundle the specs-array-prose-enforcement memo.** That thread stays on the shelf.
- **Idempotency preserved.** Running `/orb:setup` on an already-migrated repo is a no-op (same contract as today).
- **Hook gate updated.** `session-context.sh` switches from `-d "specs" || -d "cards"` to `-d "orbit"`.
- **No top-level `designs/` folder.** Design artefacts stay inside `<spec_dir>/` per rally §4a/§6a.

### Success Criteria

- `grep -rE '(^|[^/])(cards|specs|decisions|discovery)/' --include='*.md' --include='*.sh' --include='*.yaml' --include='*.json' .` returns zero hits against tracked live files after the refactor (modulo intentional string literals in tests or this interview record)
- This repo itself is migrated: `cards/`, `specs/`, `decisions/`, `discovery/` no longer appear at the repo root; the same trees exist under `orbit/`
- `/orb:setup` on a fresh (empty) directory creates `orbit/` with `cards/`, `specs/`, `decisions/` subdirectories and writes the updated CLAUDE.md snippet
- `/orb:setup` on a brownfield repo with bare artefact dirs prompts once, migrates with `git mv`, and reports the new layout
- `/orb:setup` on an already-migrated repo does nothing (idempotency)
- SessionStart hook (`session-context.sh`) fires on repos with `orbit/` and stays silent on repos without it
- Every skill's referenced artefact paths resolve under `orbit/<subdir>/` when the skill runs

### Decisions Surfaced

1. **Brownfield setup migrates in-place, interactively.** Options considered: automatic, interactive, separate `/orb:migrate-layout` skill, ignore existing. Chose interactive migration inside setup — preserves author control over the move while keeping the migration story inside one skill. (→ decisions/NNNN-orbit-setup-brownfield-migration.md)

2. **Migration is all-or-nothing with a single prompt.** Options considered: all-or-nothing, per-directory, dirty-tree refusal. Chose single prompt; no per-directory partial states; dirty-tree handling left to implementation judgement. (→ same decision record as #1, or a sibling)

3. **Rewrite scope is total.** Options considered: live files only, everything, live + dual-read compat. Chose everything — including quoted evidence in shipped review/progress files. Trade-off accepted: historical fidelity of specific quoted `ls`/`grep` output is lost; the migration commit is the record that paths used to be bare. (→ decisions/NNNN-orbit-refactor-rewrite-scope.md)

4. **No top-level `designs/` folder.** Pre-decided during card writing. Design artefacts co-locate in `<spec_dir>/` per rally. Already noted on card 0008. (→ no decision record — already settled.)

5. **`orbit/` is not configurable.** Hardcoded folder name. (Implicit in the card's goal; worth recording so a future spec doesn't relitigate.) (→ decisions/NNNN-orbit-folder-name-fixed.md)

### Open Questions (for spec or implementation)

- **Dirty-tree handling** — the user chose option 1 (single prompt) over option 3 (single prompt + dirty-tree refusal). Implementation should decide whether to warn or proceed silently when `git status --porcelain` is non-empty. `git mv` on tracked-but-modified files preserves modifications, so this is a UX judgement, not a correctness risk.
- **Plugin cache / installed copies** — this repo is the source. Downstream projects installed via the marketplace get the skill update on next `/plugin update`. Their brownfield migration fires when they next run `/orb:setup`. Worth an explicit acceptance criterion verifying the upgrade flow.
- **Memos subfolder** — `cards/memos/` moves to `orbit/cards/memos/` implicitly. Worth an explicit scenario or AC for the hook path.
- **Reviewer smell-test for the rewrite-everything choice** — `/orb:review-spec` in a forked context will likely flag the evidence-fidelity trade-off. The interview record (this file) documents that the trade-off was deliberate; the reviewer can confirm without relitigating.
