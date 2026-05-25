---
name: review-pr
description: Context-separated PR review — runs tests, checks AC coverage, verifies implementation
context: fork
agent: general-purpose
---

# /orb:review-pr

Verify an implementation before merge. This skill runs in a **forked context** — a fresh agent session with execution permissions that reads the diff cold.

Agent prose follows the discipline in `.orbit/STYLE.md` (see also card 0026 — `.orbit/cards/0026-agent-prose-discipline.yaml`).

@.orbit/STYLE.md

## Usage

```
/orb:review-pr <spec-id> [branch_or_pr]
```

The skill takes an orbit-state spec id — the spec's `acceptance_criteria` are the implementation contract. The branch/PR argument is optional; if omitted the current branch is used.

## Instructions

### 1. Identify What to Review

- If a spec-id is supplied via $ARGUMENTS: use it.
- If not: call `orbit --json spec resolve --skill review-pr` and apply the
  three-step recovery from spec
  `2026-05-19-skills-infer-or-prompt-before-halt`:
  - **`outcome=resolved`** → use `data.result.id`; surface
    `data.result.source` (`bound_card` / `single_open`) in the
    response preamble so the reviewer sees which spec was picked.
  - **`outcome=prompt`** → present `data.result.candidates[]` as a
    single AskUserQuestion (one round trip). Each candidate carries
    `id` + `goal_first_line` — use both in the choice label.
  - **Verb exits non-zero with `spec.resolve: unavailable: ...`** →
    surface the message verbatim. The verb owns the two canonical
    halt templates (terminal and recoverable); do not paraphrase.
- If a branch name or PR number is provided alongside the spec-id: use it.
- If not: use the current branch or most recent PR.
- Gather the diff: `git diff main...HEAD`.

### 2. Phase 1: Read the Diff

1. Run `git diff main...HEAD` to see all changes.
2. Read the spec via `orbit --json spec show <spec-id>` to understand what was intended — the `goal` field carries the goal and the `acceptance_criteria` array enumerates the contract.
3. Run `orbit spec acs <spec-id>` to enumerate the AC list with current check status. The spec's `acceptance_criteria` field replaces the earlier `progress.md` tracker — `[x]` marks are the implementer's self-reported AC completions, set by `/orb:implement` via `orbit spec check <spec-id> <ac-id>`.
4. Identify which acceptance criteria this implementation claims to satisfy from the parsed `[x]` rows.
5. Run a keyword scan (see `/orb:keyword-scan`) against `.orbit/choices/` using terms from the spec's `goal` and any prose in the linked card files (`orbit card show <id>`). If relevant decisions exist, verify the implementation respects them. Flag violations as findings.

### 3. Phase 2: Run Tests + AC Coverage Check

**Orchestrate `/orb:code-investigate` (narrow mode) at Phase 2 entry, BEFORE the AC-coverage / call-site / related-doc checklist begins.** Per choice 0029 (pipeline-orchestrates-investigation), review-pr is a pipeline-stage moment where investigation must fire structurally, not as advice. The orchestrated invocation surfaces precise answers — *what calls X*, *where's the test for ac-04*, *is feature Y documented in METHOD.md* — to feed Phase 2's checks rather than approximating mid-review.

**Scope is the PR's changed paths.** Resolve in this order:

1. Preferred: `gh pr view --json files,baseRefName` when a PR exists. The `files[].path` array is the scope; `baseRefName` carries the base branch.
2. Fallback (no PR yet, or `gh` unavailable): `git diff --name-only $(git merge-base $BASE HEAD) HEAD` where $BASE comes from gh metadata or, absent gh, from the branch's tracking ref via `git rev-parse --abbrev-ref $BRANCH@{upstream}`.
3. Base-resolution failure: if both paths fail (no PR, no upstream tracking, no gh access), invoke the bypass path with reason `"review-pr base resolution failed"` rather than silently skipping.

**Write scope to spec note BEFORE the Skill call** (args-drop guard per memory `slash-command-args-vs-skill-tool-args` — Skill tool args can drop on forked invocations):

```bash
orbit spec note <spec-id> "investigation scope [review-pr]: <changed-paths>"
```

Then invoke `/orb:code-investigate` (narrow mode) via the Skill tool with that scope. **Quote a 5-10 line summary of the return inline** into your working context before walking the AC-coverage checklist — the marker write alone won't change your behaviour mid-review; re-quoting the prose is what makes the investigation load-bearing.

**Bypass shape.** If the diff is trivial (single-line README fix, a typo, a comment-only change) or base resolution fails (see (3) above), call AskUserQuestion with:
- (a) Run `/orb:code-investigate` now (proceed with the orchestrated invocation)
- (b) Skip with logged reason

If (b), log via `orbit spec note <spec-id> "investigation bypass [review-pr]: <reason>"` and proceed to the checklist.

1. Run the project's test suite. Record pass/fail with output.
2. **AC-to-test coverage check**: For every AC parsed in Phase 1, search the project's test sources for a test bearing the bare AC identifier (`ac<NN>` or `ac-NN`).

#### Type-keyed evidence expectations

Per spec 2026-05-16-ac-taxonomy ac-08, each AC carries an `ac_type` declaring what kind of evidence closes it. The reviewer selects the right evidence shape per AC from this table — applying it BEFORE the AC-walk so missing-test findings only surface where a test was actually expected:

| `ac_type`     | Expected evidence                                                                                                                                            |
|---------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `code`        | Passing test bearing the AC id + commit reference. AC-to-test coverage rules below apply directly.                                                           |
| `config`      | Grep + file diff on the named config or manifest. If the config touches an external system, an external-system check (e.g. external query, dashboard read). |
| `doc`         | Grep + content check on the named written artefact (CLAUDE.md edit, card text, memo, MADR).                                                                  |
| `ops`         | Operator log line + signoff quote (PR comment, chat record, or substrate memory entry). The verification field names the operator artefact location.        |
| `observation` | Dated metric window + metric reference (dashboard URL, log query, empirical artefact captured in a follow-up commit message or substrate memory).            |

**ACs with `ac_type: ops` or `ac_type: observation` MUST NOT be flagged as missing test evidence.** These never had unit tests by design — the review walk routes them to the appropriate evidence shape from the table above instead. (Live-trigger pattern: spec 2026-05-16-memos-own-folder ac-12 was a memo-based smoke that the previous review treated as test-shaped; the typed-AC field prevents that confusion.)

#### AC-to-test coverage (`ac_type: code`)

```
AC Coverage Report:
  ac-01:   ✓ ac01_creates_project_structure
  ac-02:   ✓ ac02_manifest_has_correct_fields
  ac-03:   ✗ NO TEST FOUND
  ac-04:   ✓ ac04_handles_edge_case
  Coverage: 3/4 ACs have tests (75%)
```

Cross-language patterns to search:
- Rust: `fn ac<NN>` or `fn test_ac<NN>`
- Python: `def test_ac<NN>` or `def ac<NN>`
- TypeScript: `test('ac<NN>` or `it('ac<NN>`
- Bash/general: grep for `ac<NN>` or `ac-<NN>` in test directories

In the honest-assessment paragraph, contextualise which uncovered code-type ACs are still gate-style (judged from each AC's description text — e.g. a sequencing gate that the implementation respects through other artefacts) versus genuine test gaps. The orbit-state spec carries `ac_type`, description text, and a `gate` flag per AC; the reviewer reads all three and judges whether a missing test is a real gap or an exempt non-test AC.

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
**Spec:** <spec-id>
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

- **Inline invocation** (a human running `/orb:review-pr <spec-id>` directly): save to the default sidecar path `.orbit/specs/<spec-id>/review-pr-<date>.md`. For re-reviews on the same date, append `-v2`, `-v3` cycle suffixes (`<spec-id>/review-pr-<date>-v2.md`).
- **Forked-Agent invocation** (e.g. launched by `/orb:drive`): the invoking agent's brief will supply an explicit output path — **use the brief's path verbatim**. It takes precedence over the default. Drive uses cycle-ordinal suffixes (`-v2.md`, `-v3.md`) to disambiguate REQUEST_CHANGES cycles; writing to the default path when the brief specified a cycle-specific path will cause drive to report the review as missing and trigger a retry.

## Critical Rules

- **Evidence over reasoning.** Every CRITICAL finding must include command output or file:line citations.
- The reviewer sees the diff and spec but has NO context from the implementing session.
- **Never suggest "open a follow-up card."** If you identify adjacent work or future improvements, note them in the Findings section. The implementing agent handles forwarding via memos — cards describe capabilities, not work items.
