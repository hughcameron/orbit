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
    ac_type: code        # code | doc | gate | config
    description: "Measurable criterion 1"
    verification: "How to verify"
  - id: ac-02
    ac_type: doc
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
  test_prefix: "remat"  # short label for this spec — disambiguates ACs across specs
  timestamp: "ISO-8601"
  ambiguity_score: null
  interview_ref: "specs/YYYY-MM-DD-<topic>/interview.md"
```

## AC Naming Convention

Every acceptance criterion gets a sequential `ac-NN` ID. These IDs are used by implementers to prefix test function names, creating a machine-checkable link from tests back to the spec.

When a project has multiple specs, AC IDs collide (`ac-01` exists in every spec). The `test_prefix` metadata field disambiguates by scoping test names to the spec:

```
Spec metadata:  test_prefix: remat
Spec AC:        ac-03: "Steps execute in declared order"
Test:           fn remat_ac03_steps_execute_in_declared_order() { ... }
```

When `test_prefix` is absent, tests use the bare `ac<NN>` prefix (backward-compatible):

```
Spec AC:  ac-03: "Steps execute in declared order"
Test:     fn ac03_steps_execute_in_declared_order() { ... }
```

**Choosing a prefix:** Use a short, unique label that identifies this spec — a slug (`remat`, `introspect`), an abbreviation, or a sequence (`s03`). Keep it short — it appears in every test name. Avoid version-like prefixes (`v03`) since `metadata.version` already carries the version and the overloading is confusing.

### AC Type Classification

Every AC must include an `ac_type` field. This tells `/orb:audit` whether to expect a test:

| Type | When to use | Test expected? |
|------|-------------|----------------|
| `code` | Functional behaviour implemented in source code | Yes |
| `doc` | Document deliverable (decision record, runbook, design doc) | No |
| `gate` | Manual or process gate (approval step, review checkpoint) | No |
| `config` | Configuration change (env vars, infra settings, CI pipeline) | No |

Default to `code` when uncertain. Most ACs are code. The type field prevents `/orb:audit` from reporting false negatives on deliverables that have no test by design.

Be specific and concrete. Extract actual requirements from the conversation, not generic placeholders.

## Evidence Validation

When extracting constraints and acceptance criteria, check whether each one is backed by evidence:

- **Evidence-backed constraint**: The interview cites data, research findings, or experimental results. Include the source and numbers in the constraint. Example: `"ATR gate threshold at 66th percentile (frontier sweep: F1=0.527 ATR-high vs 0.335 elsewhere)"`
- **Assumption without evidence**: The interview states a value or approach without citing data. Flag it with `# ASSUMPTION — needs validation` in a YAML comment. These are risk points that implementation may invalidate.
- **Research-dependent constraint**: The constraint's validity depends on conditions that haven't been verified yet (e.g., a finding from one label scheme applied to a different one). Flag it with `# CONDITIONAL — valid only if <condition>`.

This traceability prevents specs from silently inheriting stale assumptions from prior work.
