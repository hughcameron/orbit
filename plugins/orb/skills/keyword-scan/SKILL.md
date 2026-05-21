---
name: keyword-scan
description: Shared technique — keyword search across orbit artifacts using rg/grep
user-invocable: false
---

# Keyword Scan Technique

A lightweight search technique for finding related work across orbit artifacts. Skills reference this technique rather than inlining the same pattern.

## The Pattern

1. **Extract keywords** from an orbit artifact (card, spec, interview, memo)
2. **Build an alternation pattern** from 5-8 distinctive domain terms
3. **Run a single search** against a target directory
4. **Use the results** to inform the skill's next step

## Keyword Extraction

Pull distinctive domain terms — skip generic words (the, when, should, given, then, a, is, are, that, this, for, with, can, will, from, into, about).

| Artifact | Extract from |
|----------|-------------|
| Card | `feature`, scenario `name` fields, `goal` |
| Spec | `goal`, `constraints`, AC `description` fields |
| Interview | Summary section — goal, constraints, decisions |
| Memo | Full text (memos are short by nature) |

**Term formatting:**
- Dot-separate compound terms for flexible matching: `trailing.stop`, `win.rate`, `session.hook`
- Keep terms lowercase
- 5-8 terms is the sweet spot — fewer misses context, more adds noise

## Search Command

Use `rg` or `grep -rlE` — both return file-list output. Some environments hook-route `rg` invocations through a token-frugal grep proxy (e.g. `rtk grep`), so write queries that work in either: POSIX ERE alternation, no PCRE2.

```bash
# alternation pattern that works in both rg and grep -rE
rg -l "term1|term2|term3" <target>/
grep -rlE "term1|term2|term3" <target>/
```

When you specifically need ripgrep features (PCRE2, `--json`, regex extensions) and the shell may wrap `rg`, invoke the binary by absolute path (e.g. `/home/linuxbrew/.linuxbrew/bin/rg`).

`-l` returns file paths only, not content — keeping token usage minimal.

## Search Targets by Skill

| Skill | Target | What you're looking for |
|-------|--------|------------------------|
| `/orb:design` | `.orbit/specs/` | Orphaned specs not in the card's `specs` array |
| `/orb:distill` | `.orbit/cards/` | Existing cards that overlap with candidates being drafted |
| `/orb:card` | `.orbit/cards/`, `.orbit/specs/` | Overlap with existing capabilities before creating a new card |
| `/orb:discovery` | `.orbit/specs/`, `.orbit/choices/` | Prior art — specs or decisions that already explored this topic |
| `/orb:implement` | Project source | Existing code, patterns, or tests related to the spec's ACs |
| `/orb:review-pr` | `.orbit/choices/` | Decisions the implementation should have respected |

## Interpreting Results

- **Log the keywords used** so the author can see what was searched
- **Surface hits, don't act on them silently** — the author confirms relevance
- **If no hits, move on silently** — absence is not an error
- **Hits are candidates, not conclusions** — a file mentioning "trailing stop" might be unrelated. The skill should present what was found and let the author decide
