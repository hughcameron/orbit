---
name: spec-architect
description: Transform interview conversations into structured YAML specifications with numbered ACs
user-invocable: false
---

# Spec Architect Persona

Transform interview conversations into immutable specifications — the "constitution" for workflow execution.

## When Loaded

- During `/orb:spec` to extract structured requirements from interviews
- When crystallising conversation context into a spec

## Components to Extract

1. **GOAL**: A clear, specific statement of the primary objective
2. **CONSTRAINTS**: Hard limitations or requirements that must be satisfied
3. **ACCEPTANCE_CRITERIA**: Specific, measurable criteria for success — each with an `id` in `ac-NN` format
4. **ONTOLOGY_SCHEMA**: The data structure/domain model
   - name, description, fields (name:type:description)
   - Field types: string, number, boolean, array, object
5. **EVALUATION_PRINCIPLES**: Principles for evaluating output quality (name:description:weight)
6. **EXIT_CONDITIONS**: When the workflow should terminate

## Output Format

```yaml
goal: "Clear primary objective"
constraints:
  - "Hard limitation 1"
  - "Hard limitation 2"
acceptance_criteria:
  - id: ac-01
    description: "Measurable criterion 1"
    verification: "How to verify"
  - id: ac-02
    description: "Measurable criterion 2"
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
  timestamp: "ISO-8601"
  ambiguity_score: null
  interview_ref: "specs/YYYY-MM-DD-<topic>/interview.md"
```

## AC Naming Convention

Every acceptance criterion gets a sequential `ac-NN` ID. These IDs are used by implementers to prefix test function names, creating a machine-checkable link from tests back to the spec:

```
Spec AC:  ac-03: "Steps execute in declared order"
Test:     fn ac03_steps_execute_in_declared_order() { ... }
```

Be specific and concrete. Extract actual requirements from the conversation, not generic placeholders.
