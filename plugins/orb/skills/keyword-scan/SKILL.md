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

Prefer ripgrep (`rg`) for speed. Fall back to `grep` when `rg` is unavailable — both are standard on development machines, but `rg` is not guaranteed on minimal or CI environments.

```bash
# ripgrep (preferred)
rg -l "term1|term2|term3" <target>/

# grep fallback — note escaped pipes
grep -rl "term1\|term2\|term3" <target>/
```

**Detecting availability:**

```bash
if command -v rg &>/dev/null; then
  rg -l "pattern" <target>/
else
  grep -rl "pattern" <target>/
fi
```

Both commands return file paths only (`-l`), not content — keeping token usage minimal.

## Search Targets by Skill

| Skill | Target | What you're looking for |
|-------|--------|------------------------|
| `/orb:design` | `orbit/specs/` | Orphaned specs not in the card's `specs` array |
| `/orb:distill` | `orbit/cards/` | Existing cards that overlap with candidates being drafted |
| `/orb:card` | `orbit/cards/`, `orbit/specs/` | Overlap with existing capabilities before creating a new card |
| `/orb:discovery` | `orbit/specs/`, `orbit/decisions/` | Prior art — specs or decisions that already explored this topic |
| `/orb:implement` | Project source | Existing code, patterns, or tests related to the spec's ACs |
| `/orb:review-pr` | `orbit/decisions/` | Decisions the implementation should have respected |

## Interpreting Results

- **Log the keywords used** so the author can see what was searched
- **Surface hits, don't act on them silently** — the author confirms relevance
- **If no hits, move on silently** — absence is not an error
- **Hits are candidates, not conclusions** — a file mentioning "trailing stop" might be unrelated. The skill should present what was found and let the author decide
