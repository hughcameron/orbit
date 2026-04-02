---
name: review-spec
description: Context-separated spec review — spawns a fresh agent to stress-test a plan before implementation
context: fork
agent: general-purpose
---

# /orb:review-spec

Stress-test a specification before implementation begins. This skill runs in a **forked context** — a fresh agent session with zero shared conversation history.

## Usage

```
/orb:review-spec [spec_file]
```

## Why Context Separation Matters

A reviewer who watched you build something has confirmation bias. A fresh agent reads the spec cold. Context-separated review catches problems that same-session review misses.

## Instructions

### 1. Gather the Spec

- If a spec file path is provided via $ARGUMENTS: read it
- If not: look for the most recent `specs/*/spec.yaml` file
- Also read the associated interview file (from `metadata.interview_ref`)
- If neither exists, report that no spec was found

### 2. Review Process

1. **Assumption audit**: List every assumption the spec makes. For each, ask: what happens when this assumption is wrong? Flag assumptions not validated by acceptance criteria.

2. **Failure mode analysis**: For each AC, identify how it could pass in testing but fail in production:
   - Environment differences (dev vs prod, interactive vs cron)
   - Path assumptions (relative vs absolute)
   - Timing assumptions (race conditions, timeouts)
   - Permission assumptions

3. **Test adequacy**: For each AC's verification method — does it actually prove the criterion is met, or only under specific conditions?

4. **Gap analysis**: What's NOT in the spec? Missing error handling, rollback plan, monitoring, edge cases.

5. **Constraint check**: Are constraints realistic? Do any contradict each other?

### 3. Output

Produce a structured review:

```markdown
# Spec Review

**Date:** <today>
**Reviewer:** Context-separated agent (fresh session)
**Spec:** <path to spec.yaml>
**Verdict:** APPROVE / REQUEST_CHANGES / BLOCK

---

## Findings

### [SEVERITY] <title>
**Category:** assumption | failure-mode | test-gap | missing-requirement | constraint-conflict
**Description:** What the problem is
**Evidence:** Why you believe this (cite spec lines, interview answers)
**Recommendation:** What to change

---

## Honest Assessment

<one paragraph — is this plan ready? what's the biggest risk?>
```

Save the review to: `specs/YYYY-MM-DD-<topic>/review-spec-<date>.md`

## Verdicts

- **APPROVE**: "I couldn't find problems" (not "this is good")
- **REQUEST_CHANGES**: Specific changes needed before implementation
- **BLOCK**: Plan needs rework — return to `/orb:interview`
