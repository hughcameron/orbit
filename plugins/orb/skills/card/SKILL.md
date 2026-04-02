---
name: card
description: Write a feature card — capture who needs what, why, and expected behaviours as scenarios
---

# /orb:card

Interactively write a feature card that captures a user need with expected behaviours.

## Usage

```
/orb:card [topic]
```

## What a Card Is

A card captures a **feature**: who needs it, why it matters, and what they'd expect to see. It follows a Gherkin-inspired structure — a feature description with scenarios — expressed in YAML.

Cards are NOT specs. They don't prescribe solutions. "The step name is visible immediately" doesn't say "flush stdout" — it says what the user observes. The *how* comes during the interview.

## Instructions

### 1. Determine the Next Card Number

Read the `cards/` directory. Find the highest existing `NNNN-*.yaml` number and increment by 1. If no cards exist, start at `0001`.

### 2. Interview for Card Content

Use **AskUserQuestion** to gather:

1. **Feature name**: What is this feature called? (short, descriptive)
2. **Role**: Who has this need? (as_a)
3. **Desire**: What do they need? Outcome, not solution. (i_want)
4. **Benefit**: Why does it matter? (so_that)
5. **Scenarios**: What would the user expect to see? Gather 2-5 scenarios, each with:
   - **name**: Short scenario label
   - **given**: Precondition
   - **when**: Action or event
   - **then**: Observable outcome (in user language, not engineering language)
6. **Priority**: now / next / later (optional)
7. **References**: Are there existing tools, libraries, or approaches that inspire this feature? (optional) — these are not solutions, they're prior art that provides context. Examples: "uv: fast, minimal output", "cargo: step-by-step compile progress".

### 3. Write the Card

Save as `cards/NNNN-<slug>.yaml`:

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

  - name: "<scenario name>"
    given: "<precondition>"
    when: "<action or event>"
    then: "<observable outcome>"

priority: "now"

references:                          # optional — prior art and inspiration
  - "<tool/approach>: <what's relevant about it>"
```

### 4. Quality Check

Verify the card against INVEST criteria:
- **Independent**: Can be delivered without other cards
- **Negotiable**: Scenarios describe outcomes, not solutions
- **Valuable**: Clear benefit to the user
- **Estimable**: Enough detail to estimate effort
- **Small**: 2-5 scenarios (if more, suggest splitting)
- **Testable**: Each scenario has an observable outcome

### Card Lifecycle

Cards don't have a status field. Instead:

| State | How you know |
|-------|-------------|
| Open | The file exists in cards/ |
| In progress | A spec references this card |
| Delivered | The `releases` field lists specs that addressed it |
| Closed | Moved to cards/done/ or deleted (git history preserves) |

---

**Next step:** Start an interview with `/orb:interview` to refine this card into a specification.
