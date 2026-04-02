---
name: spec
description: Generate a structured YAML specification with numbered ACs from interview results
disable-model-invocation: true
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
- If no file path: Check conversation history for a recent `/orb:interview` session
- If neither: Ask the user what to crystallise

### 2. Assess Ambiguity

Score clarity before generating:

- **Goal Clarity** (40% weight): Is the goal specific?
- **Constraint Clarity** (30% weight): Are constraints specified?
- **Success Criteria Clarity** (30% weight): Are criteria measurable?

**Formula:** `ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)`

**Threshold:** Ambiguity must be ≤ 0.2. If higher, suggest returning to `/orb:interview`.

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

Every acceptance criterion gets a sequential `ac-NN` ID. These IDs are used by implementers to prefix test function names:

```yaml
goal: "Clear primary objective"

constraints:
  - "Hard limitation 1"

acceptance_criteria:
  - id: ac-01
    description: "Measurable criterion"
    verification: "How to verify"

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
  timestamp: "YYYY-MM-DDTHH:MM:SSZ"
  ambiguity_score: 0.15
  interview_ref: "specs/YYYY-MM-DD-<topic>/interview.md"
```

### 4. Save the Spec

Save to: `specs/YYYY-MM-DD-<topic-slug>/spec.yaml`

If the interview file exists in a spec directory, save alongside it.

### 5. Assess Risk Tier

Based on the spec content, assess the risk tier:

- **HIGH**: Deployment, infrastructure, cron, production-touching. Both spec review + PR review.
- **STANDARD**: Feature work. PR review only.
- **SKIP**: Docs, config-only. No review.

State the tier and reasoning.

---

**Next step:** For HIGH-tier work, run `/orb:review-spec` to stress-test the plan. Otherwise, proceed to implementation.
