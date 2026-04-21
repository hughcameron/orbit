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
- Locate the associated spec (check PR description, recent `orbit/specs/*/spec.yaml`, or search)
- Gather the diff: `git diff main...HEAD`

### 2. Phase 1: Read the Diff

1. Run `git diff main...HEAD` to see all changes
2. Read the spec to understand what was intended
3. If `progress.md` exists alongside the spec, read it — this is the implementer's self-reported AC tracker from `/orb:implement`. Cross-reference it with your own findings.
4. Identify which acceptance criteria this implementation claims to satisfy
5. Run a keyword scan (see `/orb:keyword-scan`) against `orbit/decisions/` using terms from the spec's goal and constraints. If relevant decisions exist, verify the implementation respects them. Flag violations as findings.

### 3. Phase 2: Run Tests + AC Coverage Check

1. Run the project's test suite. Record pass/fail with output.
2. **AC-to-test coverage check**: Parse the spec for AC IDs (`ac-NN`), their `ac_type` field, and `metadata.test_prefix`. Only `code`-type ACs require tests. ACs typed as `doc`, `gate`, or `config` are exempt. If `ac_type` is missing, treat as `code`.

If `test_prefix` is present (e.g., `remat`), search for prefixed test names (`remat_ac01_*`). If absent, search for bare `ac<NN>` names (backward-compatible).

```
AC Coverage Report (prefix: remat):
  ac-01 (code):   ✓ remat_ac01_creates_project_structure
  ac-02 (code):   ✓ remat_ac02_manifest_has_correct_fields
  ac-03 (doc):    EXEMPT (document deliverable)
  ac-04 (code):   ✗ NO TEST FOUND
  Coverage: 2/3 testable ACs have tests (67%), 1 exempt
```

Cross-language patterns to search (with prefix `remat`; omit prefix if `test_prefix` absent):
- Rust: `fn remat_ac<NN>`
- Python: `def test_remat_ac<NN>` or `def remat_ac<NN>`
- TypeScript: `test('remat_ac<NN>` or `it('remat_ac<NN>`
- General: grep for `remat_ac<NN>` prefix in test directories

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

### Verdict line contract (machine-parseable)

The header line `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK` is a **contract**, not formatting. Downstream consumers — notably `/orb:drive` — parse the verdict from this line with a strict regex (`^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$`). Write the line exactly as shown, with one of the three tokens unquoted, case-sensitive, and no trailing prose on the same line. Deviation (lowercase, inline prose, frontmatter, sidecar files) silently breaks the contract.

### Output path (invoked inline vs forked)

- **Inline invocation** (a human running `/orb:review-pr` directly): save to the default path `orbit/specs/YYYY-MM-DD-<topic>/review-pr-<date>.md`.
- **Forked-Agent invocation** (e.g. launched by `/orb:drive`): the invoking agent's brief will supply an explicit output path — **use the brief's path verbatim**. It takes precedence over the default. Drive uses cycle-ordinal suffixes (`-v2.md`, `-v3.md`) to disambiguate REQUEST_CHANGES cycles; writing to the default path when the brief specified a cycle-specific path will cause drive to report the review as missing and trigger a retry.

## Critical Rules

- **Evidence over reasoning.** Every CRITICAL finding must include command output or file:line citations.
- The reviewer sees the diff and spec but has NO context from the implementing session.
- **Never suggest "open a follow-up card."** If you identify adjacent work or future improvements, note them in the Findings section. The implementing agent handles forwarding via memos — cards describe capabilities, not work items.
