---
name: evaluate
description: Three-stage evaluation — mechanical checks, semantic AC verification, consensus deliberation
disable-model-invocation: true
---

# /orb:evaluate

Evaluate an artifact against a spec using the three-stage verification pipeline.

## Usage

```
/orb:evaluate <spec_file> [artifact_path]
```

## Instructions

### 1. Gather Inputs

- **Spec**: Read the spec YAML file (or find one in conversation context)
- **Artifact**: The code/output to evaluate — file path, directory, or recent output
- If either is missing, ask the user

### 2. Stage 1: Mechanical Verification ($0 cost)

Run automated checks using Bash:

- **LINT**: Run the project's linter (detect from config: eslint, ruff, clippy, etc.)
- **BUILD**: Run the build command if applicable
- **TEST**: Run the test suite
- **STATIC**: Run type checker if available (mypy, tsc, etc.)

For each check, record PASS/FAIL with output.

**Gate:** If any check fails, stop here. Do not proceed to Stage 2.

### 3. Stage 2: Semantic Evaluation

For each acceptance criterion in the spec:
1. **Evidence**: Does the artifact provide concrete evidence?
2. **Completeness**: Is the criterion fully satisfied?
3. **Quality**: Is the implementation sound?

**Scoring:**
- AC Compliance: % of criteria met (threshold: 100%)
- Overall Score: weighted by evaluation principles (threshold: 0.8)

**Gate:** AC compliance must be 100%. If any AC is unmet, stop and report.

### 4. Stage 3: Consensus (Triggered, Optional)

Only runs when:
- User explicitly requests it
- Stage 2 score is borderline (0.7-0.8)
- High uncertainty in evaluation

**Process — delegate to three personas:**
1. **Advocate** (`/orb:advocate`): Makes the case FOR
2. **Contrarian** (`/orb:contrarian`): Challenges using ontological analysis
3. **Judge** (`/orb:judge`): Weighs both positions, renders verdict

**Threshold:** Majority approval (≥ 2 of 3).

### 5. Save Evaluation Report

Save to: `specs/YYYY-MM-DD-<topic>/evaluation-<date>.md`

```markdown
# Evaluation Report

**Date:** <today>
**Spec:** <path to spec.yaml>
**Artifact:** <path or description>

## Stage 1: Mechanical Verification
[results per check]
**Result**: PASSED / FAILED

## Stage 2: Semantic Evaluation
[AC-by-AC analysis]
**AC Compliance**: X%
**Overall Score**: X.XX
**Result**: PASSED / FAILED

## Stage 3: Consensus (if run)
[deliberation summary]
**Approval**: X%
**Result**: APPROVED / REJECTED

## Final Decision: APPROVED / REJECTED / CONDITIONAL
```

### 6. Next Steps

- **APPROVED**: Ship it.
- **CONDITIONAL**: List changes needed. Fix and re-evaluate.
- **REJECTED**: Run `/orb:evolve` to improve the spec, or fix the artifact.

---

**Next step:** If rejected or conditional, use `/orb:evolve` to iterate the spec.
