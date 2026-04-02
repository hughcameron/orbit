---
name: evolve
description: Iterate a spec based on evaluation results — wonder, reflect, mutate, converge
---

# /orb:evolve

Evolutionary loop that iteratively refines specifications based on evaluation results. Each generation improves the spec until the ontology converges.

## Usage

```
/orb:evolve <spec_file> [evaluation_file]
```

## Concept

```
Gen 1: Interview → Spec(O1) → Implement → Evaluate
Gen 2: Wonder → Reflect → Spec(O2) → Implement → Evaluate
Gen 3: Wonder → Reflect → Spec(O3) → Implement → Evaluate
...until ontology converges or max generations reached
```

## Instructions

### 1. Gather Context

- **Spec**: Read the current spec YAML
- **Evaluation**: Read the most recent evaluation report (or from conversation context)
- **Generation**: Determine which generation we're on (check for `spec-gen-N.yaml` files)

### 2. Wonder Phase — "What do we still not know?"

Examine evaluation results to identify:
- **Ontological gaps**: Concepts missing from the schema
- **Hidden assumptions**: What the evaluation revealed
- **AC failures**: Which criteria failed and why
- **Drift indicators**: Where implementation diverged from intent

```
## Wonder (Gen N)

### Gaps Identified
- [Gap 1: description and evidence from evaluation]

### Assumptions Exposed
- [Assumption that proved wrong]

### Questions for Reflection
- [Question about the ontology]
```

### 3. Reflect Phase — "How should the ontology evolve?"

Propose specific mutations:
- **Add fields**: New concepts discovered during implementation
- **Remove fields**: Concepts that proved unnecessary
- **Modify constraints**: Too tight or too loose
- **Refine AC**: Criteria that need to be more specific
- **Adjust weights**: Evaluation principle rebalancing

```
## Reflect (Gen N)

### Proposed Mutations
1. ADD field: `status_transitions` (array) — evaluation showed this is core
2. REMOVE field: `priority_score` — unused
3. MODIFY constraint: "No database" → "SQLite allowed"
4. REFINE AC #3: More specific persistence requirement
```

### 4. Generate New Spec

Apply approved mutations:
- Increment version: `1.0` → `2.0`
- Update timestamp
- Record generation in metadata
- Save as `specs/YYYY-MM-DD-<topic>/spec-gen-<N>.yaml`
- Keep previous spec intact for history

### 5. Convergence Check

Compare new spec's ontology against previous generation:
- Extract field names from both
- Calculate Jaccard similarity: `|intersection| / |union|`
- **Converged**: similarity ≥ 0.95 — evolution complete
- **Stagnated**: Same ontology 3+ generations — try `/orb:contrarian` or `/orb:hacker`
- **Exhausted**: 30+ generations — stop, present best result
- **Continue**: similarity < 0.95 — more evolution needed

### 6. Next Steps

- **CONTINUE**: Implement new spec → `/orb:evaluate` → `/orb:evolve` again
- **CONVERGED**: Ontology is stable. Final implementation.
- **STAGNATED**: `/orb:contrarian` or `/orb:hacker` to break through
- **EXHAUSTED**: Manual review of best generation

---

**Next step:** Implement the evolved spec, then `/orb:evaluate` to verify.
