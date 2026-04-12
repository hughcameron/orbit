# Design: Specs Array Integrity

**Date:** 2026-04-12
**Interviewer:** Agent
**Card:** cards/0004-specs-array-integrity.yaml

---

## Context

Card: *Specs array stays complete without manual upkeep* — 6 scenarios, goal: zero orphaned specs
Prior specs: 0 — this is the first spec for this card
Gap: The entire capability is unbuilt. Currently no skill updates the card's specs array, and no skill checks for orphans.

## Q&A

### Q1: Where should write-time enforcement live?
**Q:** When a spec is created, something needs to append its path to the card's specs array. Should this live in `/orb:spec`, in a post-save hook, or elsewhere?
**A:** In `/orb:spec` — the interview already names the card (`Card: cards/NNNN-slug.yaml`), so the spec skill has the information it needs. Keep it in one place rather than adding a hook.

### Q2: What about the read-time reconciliation approach?
**Q:** The write-time update prevents future gaps, but existing drift needs a catch-up mechanism. Should `/orb:design` scan for orphaned specs?
**A:** Yes. Design already loads the card and reads the specs array. Adding a keyword scan before the session starts is the natural place. It should surface orphans as a prompt, not auto-link them.

### Q3: How should keyword matching work?
**Q:** The reconciliation needs to find specs related to a card without reading every spec fully. Card slugs alone are fragile — specs may discuss the topic without referencing the card by name. What matching strategy?
**A:** Extract keywords from the card (feature name, scenario names, goal text) and build a ripgrep alternation pattern. One `rg -l "term1|term2|term3" specs/` call searches across all spec artifacts (spec.yaml, interview.md, progress.md). Compare hits against the card's specs array. This catches semantically related work, not just formally linked work. Fall back to `grep -rl` when rg is unavailable.

### Q4: How many keywords, and how to select them?
**Q:** The card has a feature name, multiple scenario names, and a goal. Extracting every word would be noisy. What's the right granularity?
**A:** 5-8 terms. Pull from the feature name (split on spaces, take distinctive terms), scenario `name` fields (take the nouns/verbs that are domain-specific), and the goal. Skip generic words like "the", "when", "should". Dot-separate for word-boundary flexibility in rg patterns.

### Q5: Should the session hook also flag orphans?
**Q:** The SessionStart hook already surfaces outstanding memos and in-flight specs. Should it also check for orphaned specs?
**A:** No — keep the hook lightweight. It runs every session. The keyword scan is heavier and only matters when starting a design session. Keep it in `/orb:design`.

---

## Summary

### Goal
Zero orphaned specs — every spec that addresses a card is in that card's specs array, maintained automatically.

### Constraints
- Write-time update must be in `/orb:spec`, not a separate hook
- Keyword scan must be a single rg/grep command, not full spec reads
- Orphans are surfaced for author confirmation, never auto-linked
- Must work in environments without ripgrep (grep -rl fallback)

### Success Criteria
- `/orb:spec` appends to the card's specs array on every spec creation
- `/orb:design` reconciles the specs array against keyword-matched specs before starting
- The keyword scan uses 5-8 terms from the card, built as an alternation pattern
- Environments without rg fall back to grep with equivalent results

### Decisions Surfaced
- Write-time enforcement lives in `/orb:spec`, not a hook: simpler, the information is already available
- Read-time reconciliation lives in `/orb:design`, not the session hook: too heavy for every session
- Keyword matching over slug matching: catches semantically related work, not just formally linked specs

### Open Questions
- None
