---
name: review-spec
description: Progressive spec review — depth scales with findings, not upfront classification
context: fork
agent: general-purpose
---

# /orb:review-spec

Stress-test a specification before implementation begins. Every spec gets reviewed. The review's depth scales with what it finds — straightforward specs get a quick structural pass; complex or risky specs automatically deepen.

This skill runs in a **forked context** — a fresh agent session with zero shared conversation history.

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

### 2. Progressive Review

The review runs in passes. Every spec gets Pass 1. Subsequent passes are triggered by findings or content signals — not by upfront classification.

#### Pass 1 — Structural Scan (always runs)

Quick check of spec integrity:

1. **AC testability**: Is each AC specific enough to write a test for? Flag vague criteria ("works correctly", "handles errors gracefully").
2. **Constraint conflicts**: Do any constraints contradict each other or make ACs unreachable?
3. **Scope vs goal**: Does the scope match the goal? Over-specified (ACs beyond what the goal needs)? Under-specified (goal claims more than ACs deliver)?
4. **Obvious gaps**: Error handling mentioned? Rollback plan? Monitoring? Edge cases?
5. **Content signal scan**: Check whether the spec touches any deepening triggers:
   - Training data, ground truth, model inputs, eval datasets
   - Deployment, infrastructure, cron, production services
   - Cross-system boundaries, shared config, other agents' domains
   - Security, auth, permissions, key management
   - Data migrations, schema changes, backwards compatibility

**After Pass 1:**

- If **zero findings AND no content signals** → APPROVE. Record the pass and stop. A clean structural scan on a well-scoped spec is a valid review.
- If **any finding ≥ MEDIUM severity OR content signals present** → proceed to Pass 2.

#### Pass 2 — Assumption & Failure Analysis (triggered)

Deeper scrutiny, triggered by Pass 1 findings or content signals:

1. **Assumption audit**: List every assumption the spec makes. For each, ask: what happens when this assumption is wrong? Flag assumptions not validated by acceptance criteria.

2. **Failure mode analysis**: For each AC, identify how it could pass in testing but fail in production:
   - Environment differences (dev vs prod, interactive vs cron)
   - Path assumptions (relative vs absolute)
   - Timing assumptions (race conditions, timeouts)
   - Permission assumptions

3. **Test adequacy**: For each AC's verification method — does it actually prove the criterion is met, or only under specific conditions?

**After Pass 2:**

- If **no structural concerns** → deliver verdict based on combined Pass 1 + Pass 2 findings. Most specs stop here.
- If **structural concerns found** (contradicted assumptions, cascading failure modes, untestable ACs, downstream impact unclear) → proceed to Pass 3.

#### Pass 3 — Adversarial Review (triggered)

Full adversarial mode. Only reached when Pass 2 reveals structural problems:

1. **Simultaneous failure**: What happens when multiple assumptions are wrong at the same time?
2. **Cascade analysis**: If AC-01 fails, what happens to AC-02..N? Are there hidden dependencies between criteria?
3. **Rollback feasibility**: Can the changes be undone? What state is left behind on failure?
4. **Impact radius**: What breaks outside the spec's declared scope? What systems downstream consume this spec's outputs?

### 3. Output

Produce a structured review:

```markdown
# Spec Review

**Date:** <today>
**Reviewer:** Context-separated agent (fresh session)
**Spec:** <path to spec.yaml>
**Verdict:** APPROVE / REQUEST_CHANGES / BLOCK

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | <N> |
| 2 — Assumption & failure | <reason or "not triggered"> | <N or "—"> |
| 3 — Adversarial | <reason or "not triggered"> | <N or "—"> |

## Findings

### [SEVERITY] <title>
**Category:** assumption | failure-mode | test-gap | missing-requirement | constraint-conflict | content-signal
**Pass:** <1 | 2 | 3>
**Description:** What the problem is
**Evidence:** Why you believe this (cite spec lines, interview answers)
**Recommendation:** What to change

---

## Honest Assessment

<one paragraph — is this plan ready? what's the biggest risk?>
```

### Verdict line contract (machine-parseable)

The header line `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK` is a **contract**, not formatting. Downstream consumers — notably `/orb:drive` — parse the verdict from this line with a strict regex (`^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$`). Write the line exactly as shown, with one of the three tokens unquoted, case-sensitive, and no trailing prose on the same line. Deviation (lowercase, inline prose, frontmatter, sidecar files) silently breaks the contract.

### Output path (invoked inline vs forked)

- **Inline invocation** (a human running `/orb:review-spec` directly): save to the default path `specs/YYYY-MM-DD-<topic>/review-spec-<date>.md`.
- **Forked-Agent invocation** (e.g. launched by `/orb:drive`): the invoking agent's brief will supply an explicit output path — **use the brief's path verbatim**. It takes precedence over the default. Drive uses cycle-ordinal suffixes (`-v2.md`, `-v3.md`) to disambiguate REQUEST_CHANGES cycles; writing to the default path when the brief specified a cycle-specific path will cause drive to report the review as missing and trigger a retry.

## Verdicts

- **APPROVE**: "I couldn't find problems" (not "this is good")
- **REQUEST_CHANGES**: Specific changes needed before implementation
- **BLOCK**: Plan needs rework — return to `/orb:design` or `/orb:discovery`
