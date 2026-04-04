---
name: review-pr
description: Context-separated PR review — runs tests, checks AC coverage, verifies implementation
context: fork
agent: general-purpose
---

# /orb:review-pr

Verify an implementation before merge. This skill runs in a **forked context** — a fresh agent session with execution permissions that reads the diff cold.

## Usage

```
/orb:review-pr [branch_or_pr]
```

## Instructions

### 1. Identify What to Review

- If a branch name or PR number is provided via $ARGUMENTS: use it
- If not: check the current branch or most recent PR
- Locate the associated spec (check PR description, recent `specs/*/spec.yaml`, or search)
- Gather the diff: `git diff main...HEAD`

### 2. Phase 1: Read the Diff

1. Run `git diff main...HEAD` to see all changes
2. Read the spec to understand what was intended
3. If `progress.md` exists alongside the spec, read it — this is the implementer's self-reported AC tracker from `/orb:implement`. Cross-reference it with your own findings.
4. Identify which acceptance criteria this implementation claims to satisfy

### 3. Phase 2: Run Tests + AC Coverage Check

1. Run the project's test suite. Record pass/fail with output.
2. **AC-to-test coverage check**: Parse the spec for AC IDs (`ac-NN`), then search test files for functions prefixed with `ac<NN>`:

```
AC Coverage Report:
  ac-01: ✓ ac01_creates_project_structure
  ac-02: ✓ ac02_manifest_has_correct_fields
  ac-03: ✗ NO TEST FOUND
  Coverage: 2/3 ACs have tests (67%)
```

Cross-language patterns to search:
- Rust: `fn ac<NN>`
- Python: `def test_ac<NN>` or `def ac<NN>`
- TypeScript: `test('ac<NN>` or `it('ac<NN>`
- General: grep for `ac<NN>` prefix in test directories

### 4. Phase 3: Environment Simulation

For changes that touch deployment, infrastructure, scripts, or cron:
1. Identify the deployment context
2. Simulate it (run from $HOME, minimal PATH, etc.)
3. Record what you ran and what happened

### 5. Phase 4: Edge Case Probing

1. First run? (No prior state, empty databases, missing dirs)
2. Failure? (Network down, service unavailable)
3. Repeat? (Idempotency — running twice shouldn't break things)
4. Boundary conditions? (Empty input, max input, unicode)

### 6. Output

```markdown
# Pre-Merge Review

**Date:** <today>
**Reviewer:** Context-separated agent (fresh session)
**Branch:** <branch>
**Spec:** <path to spec.yaml>
**Verdict:** APPROVE / REQUEST_CHANGES / BLOCK

---

## Test Results

| Check | Result | Details |
|-------|--------|---------|
| Test suite | PASS/FAIL | N/M tests |
| AC coverage | X/Y | See report below |

## AC Coverage Report

| AC | Status | Test(s) |
|----|--------|---------|
| ac-01 | ✓ | ac01_description |
| ac-02 | ✗ | NO TEST FOUND |

## Findings

### [SEVERITY] <title>
**Category:** bug | test-gap | environment-mismatch | edge-case | security | performance
**Description:** What the problem is
**Evidence:** Command output or file:line reference
**Recommendation:** Specific fix

---

## Honest Assessment

<one paragraph>
```

Save to: `specs/YYYY-MM-DD-<topic>/review-pr-<date>.md`

## Critical Rules

- **Evidence over reasoning.** Every CRITICAL finding must include command output or file:line citations.
- The reviewer sees the diff and spec but has NO context from the implementing session.
