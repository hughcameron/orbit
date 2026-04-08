---
name: distill
description: Extract structured feature cards from memos, interviews, or research output
---

# /orb:distill

Extract structured feature cards from unstructured input. Takes a memo, discovery interview, or research document and identifies distinct feature ideas, presenting each as a card for individual approval.

## Usage

```
/orb:distill <file_path>
```

Where `<file_path>` is a markdown file — typically `cards/memos/YYYY-MM-DD-slug.md` or `specs/<topic>/interview.md`.

## Why This Exists

Ideas arrive as freeform text. Turning them into actionable cards currently requires a full `/orb:card` interview per feature. Distill bridges the gap — it reads what you've already written and extracts cards from it, so existing work product becomes actionable without re-interviewing.

## Instructions

### 1. Read the Source

Read the file provided via `$ARGUMENTS`.

- If no argument is provided: tell the author distill requires a file path
- If the file does not exist or is unreadable: report a clear error message and stop — do not create any files
- Read the full contents of the file

### 2. Identify Distinct Features

Analyse the source material and identify distinct feature ideas. A "feature" is a user-facing need with observable outcomes — something that could be its own card.

**Rules:**
- Each feature must be **distinct** — different user need, different outcomes
- If the source contains only one feature, that's fine — produce one card
- If the source contains **no identifiable feature ideas** (e.g. a grocery list, meeting notes with no actionable features): report "No features found in `<file_path>` — nothing to distill." and stop. Do **not** hallucinate cards from non-feature content.

Present a brief summary before starting the approval loop:

```
Found N feature(s) in <file_path>:
1. <short feature name>
2. <short feature name>
...

Presenting each for your approval.
```

### 3. Determine Card Numbering

Read the `cards/` directory. If `cards/` does not exist, create it and start numbering at `0001`. Otherwise, find the highest existing `NNNN-*.yaml` number. The first approved card gets that number + 1, and subsequent approvals increment from there.

Card numbering is determined at write time (when the author approves), not at extraction time. This is a single-user workflow — concurrent numbering is a known limitation, not a bug to solve.

### 4. Present Cards One-by-One

For each candidate, draft a card in the standard YAML format and present it to the author.

**Card format:**

```yaml
feature: "<short feature name>"
as_a: "<role>"
i_want: "<desired outcome>"
so_that: "<reason/benefit>"

scenarios:
  - name: "<scenario name>"
    given: "<precondition>"
    when: "<action or event>"
    then: "<observable outcome>"
    source_lines: "<quoted passage from source>"

  - name: "<scenario name>"
    given: "<precondition>"
    when: "<action or event>"
    then: "<observable outcome>"
    source_lines: "<quoted passage from source>"

priority: "next"                    # default; user can override via edit

references:
  - "<path to source file>"
```

**Critical rules for card content:**

- **Extract, don't invent.** Every scenario MUST trace to something in the source material. The `source_lines` field quotes the originating passage. If you can't point to a passage that supports a scenario, don't include that scenario.
- **`source_lines` is mandatory** on every scenario. It must quote text that exists verbatim (or near-verbatim) in the source document. This is the mechanically verifiable link between the card and its source.
- **`references` always includes the source path.** Every card produced by distill includes the input file path in its references list.
- **Scenarios describe outcomes, not solutions.** Follow the same principle as `/orb:card` — what the user observes, not how it's built.

**Presenting to the author:**

Use **AskUserQuestion** to present each card. Show the full YAML block, then offer three options:

- **approve** — save this card as-is
- **edit** — provide modification instructions (see step 5)
- **reject** — discard this card, move to the next

### 5. Handle Edits

When the author chooses "edit":

1. The author's next response is interpreted as **free-text modification instructions** (e.g. "change the feature name to X" or "split scenario 2 into two scenarios")
2. Apply the requested changes to the card
3. Re-present the updated card with the same approve/edit/reject options
4. **Maximum 3 edit rounds per card.** After 3 edits, present the card one final time and require approve or reject — no further edits.

**Edits and `source_lines`:** If the author requests adding a new scenario during editing that has no corresponding passage in the source document, set `source_lines` to `"user-requested during edit"`. The extract-not-invent rule applies to the *initial* extraction — author-directed edits are explicitly authored, not LLM-invented.

### 6. Write Approved Cards

When the author approves a card:

1. Determine the next available card number (read `cards/` directory at write time)
2. Generate a slug from the feature name (lowercase, hyphens, no special characters)
3. Save as `cards/NNNN-<slug>.yaml`
4. Confirm: "Saved as `cards/NNNN-<slug>.yaml`"

**Do not write anything to disk for rejected cards.**

### 7. Summary

After all candidates have been presented:

```
Distill complete:
  Source: <file_path>
  Approved: N card(s) — <list of files>
  Rejected: M card(s)
```

If any cards were approved, suggest next step: `/orb:design` to refine a card into a spec.

## Integration with Other Skills

- **`/orb:card`** — distill produces the same YAML format, so distilled cards are interchangeable with interview-created cards
- **`cards/memos/`** — the primary input source; the SessionStart hook tracks which memos have been distilled by checking card references
- **`/orb:design`** — the natural next step after distilling a card
- **`/orb:discovery`** — interview.md files from discovery sessions are valid distill inputs

---

**Next step:** Run `/orb:design` on an approved card to work out the technical approach.
