---
name: spec
description: Generate a structured YAML specification with numbered ACs from interview results
---

# /orb:spec

Generate a validated specification from interview results or conversation context.

## Usage

```
/orb:spec [interview_file]
```

## Instructions

### 1. Gather Interview Context

- If an interview file path is provided: Read it
- If no file path: Check conversation history for a recent `/orb:design` or `/orb:discovery` session
- If neither: Ask the author what to crystallise

### 2. Assess Ambiguity

Score clarity before generating:

- **Goal Clarity** (40% weight): Is the goal specific?
- **Constraint Clarity** (30% weight): Are constraints specified?
- **Success Criteria Clarity** (30% weight): Are criteria measurable?

**Formula:** `ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)`

**Threshold:** Ambiguity must be ≤ 0.2. If higher, suggest returning to `/orb:design` or `/orb:discovery`.

```
Ambiguity Assessment:
  Goal Clarity:             X% (weight: 40%)
  Constraint Clarity:       X% (weight: 30%)
  Success Criteria Clarity: X% (weight: 30%)
  Overall Ambiguity:        X.XX (threshold: ≤ 0.2)
  Ready for Spec:           Yes/No
```

### 3. Generate the Spec

Adopt the spec-architect role (see `/orb:spec-architect` for extraction guidelines).

Every acceptance criterion gets a sequential `ac-NN` ID. These IDs are used by implementers to prefix test function names. The `test_prefix` in metadata disambiguates ACs when a project has multiple specs:

```yaml
goal: "Clear primary objective"

constraints:
  - "Hard limitation 1"

acceptance_criteria:
  - id: ac-01
    ac_type: code        # code | doc | gate | config
    description: "Measurable criterion"
    verification: "How to verify"

implementation_notes:          # means-level leads from the design session — not constraints
  - "Starting context for the implementing agent"

ontology_schema:
  name: "DomainModel"
  description: "What this models"
  fields:
    - name: "field_name"
      type: "string"
      description: "What this field represents"

evaluation_principles:
  - principle: "Quality dimension"
    weight: 0.3

exit_conditions:
  - "When to stop iterating"

metadata:
  version: "1.0"
  test_prefix: "remat"  # short label for this spec — disambiguates ACs across specs
  timestamp: "YYYY-MM-DDTHH:MM:SSZ"
  ambiguity_score: 0.15
  interview_ref: ".orbit/specs/YYYY-MM-DD-<topic>/interview.md"
```

### 4. Save the Spec

Save to: `.orbit/specs/YYYY-MM-DD-<topic-slug>/spec.yaml`

If the interview file exists in a spec directory, save alongside it.

### 5. Update the Card's Specs Array

The spec references a card (from the interview's `Card:` line). After saving `spec.yaml`, append its path to the card's `specs` array so the work trail stays complete.

1. Parse the card path from the interview record (the `**Card:**` line) or from conversation context
2. If no card is identified, skip this step — not all specs originate from a card
3. Read the card YAML
4. Append the new spec path (e.g. `.orbit/specs/2026-04-12-topic/spec.yaml`) to the `specs` array
5. If the `specs` array doesn't exist yet, create it
6. Write the updated card back to disk

**This is non-negotiable.** Every spec that addresses a card must appear in the card's `specs` array. Agents downstream (`/orb:design`, `/orb:implement`) rely on this array to understand cumulative progress. An incomplete array causes agents to lose the thread and repeat or contradict prior work.

---

**Next step:** Run `/orb:review-spec` to stress-test the plan, then `/orb:implement`.
