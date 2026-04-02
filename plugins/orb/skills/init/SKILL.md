---
name: init
description: Set up a project for the orbit workflow — creates directories, CLAUDE.md snippet, and first card
---

# /orb:init

Set up a project for the orbit specification-driven workflow.

## Usage

```
/orb:init
```

## Instructions

### 1. Check Existing State

Before creating anything, check what already exists:
- Does `cards/` already exist?
- Does `specs/` already exist?
- Does `decisions/` already exist?
- Does `CLAUDE.md` already exist? If so, does it already contain the orbit snippet?

If the project is already fully initialised, inform the user and skip to the card tutorial.

### 2. Create Directory Structure

Create the following directories at the project root (skip any that exist):

```
cards/          # Feature cards — who needs what and why
specs/          # Specifications, interviews, evaluations
decisions/      # MADR decision records
```

### 3. Append CLAUDE.md Snippet

If `CLAUDE.md` does not exist, create it. If it exists, append to it — but only if the orbit snippet is not already present (check for the marker `## Workflow (orbit)`).

**Snippet to append:**

```markdown
## Workflow (orbit)

This project uses the orbit workflow: Card → Interview → Spec → Review → Ship.

- `/orb:card` — capture a feature need with expected behaviours
- `/orb:interview` — clarify requirements via Socratic Q&A
- `/orb:spec` — crystallise interview into a structured specification
- `/orb:review-spec` — stress-test the spec before implementation
- `/orb:review-pr` — verify the PR against the spec's acceptance criteria
- `/orb:evaluate` — formal 3-stage verification against spec
- `/orb:evolve` — iterate spec based on evaluation results

Artifacts live in `cards/`, `specs/`, and `decisions/`.
```

### 4. First Card Tutorial

Walk the user through writing their first feature card using `/orb:card`. Explain:

- A card captures **who** needs something, **why** it matters, and **what they'd expect to see**
- Scenarios are written in user language, not engineering language
- Cards are the intake layer — they survive context loss and ground future interviews

Then invoke `/orb:card` to interactively write the first card.

## Idempotency

This skill is idempotent. Running it again on an already-initialised project:
- Does NOT recreate existing directories
- Does NOT duplicate the CLAUDE.md snippet
- DOES offer to write another card

---

**Next step:** Write feature cards with `/orb:card`, then start an interview with `/orb:interview`.
