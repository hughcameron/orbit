---
name: audit
description: Audit AC-to-test traceability — find untested ACs, orphaned test prefixes, and coverage gaps
---

# /orb:audit

Audit the linkage between spec acceptance criteria and test files. Produces an actionable coverage report that distinguishes genuine gaps from ACs that are non-testable by design.

## Usage

```
/orb:audit [spec_path]
```

- If a spec path is provided: audit that single spec
- If no argument: audit all specs under `specs/`

## Why This Exists

Orbit's AC-to-test naming convention (`ac-03` in the spec maps to `test_ac03_*` in tests) makes coverage a grep instead of a re-read. But the convention only works if someone actually checks it. This skill is that check.

It also handles the reality that not every AC is testable code. Document deliverables, gate decisions, and configuration changes don't have test functions — and shouldn't be flagged as gaps. The `ac_type` field on each AC tells the audit what to expect.

## Instructions

### 1. Locate Specs

If `$ARGUMENTS` provides a path, use it. Otherwise, find all `specs/*/spec.yaml` files in the project.

If no specs exist, stop: *"No specs found. Run `/orb:spec` to create one."*

### 2. Parse Acceptance Criteria

For each spec, extract every AC entry. Each AC should have:

- `id`: the `ac-NN` identifier
- `description`: what it requires
- `ac_type`: one of `code`, `doc`, `gate`, `config` (defaults to `code` if missing)

Also extract `metadata.test_prefix` if present. This prefix scopes test names to the spec, preventing AC ID collisions across specs in multi-spec projects.

**AC type meanings:**

| Type | Meaning | Test expected? |
|------|---------|----------------|
| `code` | Functional behaviour implemented in source | Yes |
| `doc` | Document deliverable (decision record, runbook, etc.) | No |
| `gate` | Manual or process gate (approval, review checkpoint) | No |
| `config` | Configuration change (env vars, infra settings, CI) | No |

### 3. Search for Matching Tests

For each `code`-type AC, search test directories for functions matching the AC ID.

**If the spec has a `test_prefix`** (e.g., `remat`): search for the prefixed form first. The AC `ac-01` with prefix `remat` maps to `remat_ac01` in test names.

Cross-language patterns with prefix:

- Python: `def test_remat_ac<NN>` or `def remat_ac<NN>`
- Rust: `fn remat_ac<NN>`
- TypeScript/JavaScript: `test('remat_ac<NN>` or `it('remat_ac<NN>`
- Go: `func TestRematAc<NN>` or `func Test_remat_ac<NN>`
- General fallback: grep for `remat_ac<NN>` prefix in test directories

**If the spec has no `test_prefix`** (backward-compatible): search for bare `ac<NN>` as before.

Handle both naming formats:

- Zero-padded: `ac-01` maps to `ac01` prefix in tests
- Unpadded: `ac-1` also maps to `ac01` prefix (normalise to two digits)

Cross-language patterns without prefix:

- Python: `def test_ac<NN>` or `def ac<NN>`
- Rust: `fn ac<NN>`
- TypeScript/JavaScript: `test('ac<NN>` or `it('ac<NN>` or `describe('ac<NN>`
- Go: `func TestAc<NN>` or `func Test_ac<NN>`
- General fallback: grep for `ac<NN>` prefix in any file under `tests/` or `test/` or `__tests__/`

Also search for the prefix + `acNN` in test **file names** (e.g. `test_remat_ac03_ordering.py` or `test_ac03_ordering.py`).

### 4. Check for Orphaned Test Prefixes

Search test files for any `ac<NN>` prefixed functions that don't correspond to an AC in any current spec. These are either:

- Tests for ACs that were removed or superseded
- Typos in the AC number

Report these separately as orphans.

### 5. Check for Stale Specs

If a spec directory contains `progress.md` where all items are checked, and the spec is older than the most recent spec, note it as **completed**. Its ACs still count but are informational, not actionable.

### 6. Produce the Report

Output the report in this format:

```
## AC Traceability Audit

**Date:** <today>
**Scope:** <spec path or "all specs">

### <spec-directory-name> (spec.yaml)

| AC | Type | Status | Test(s) |
|----|------|--------|---------|
| ac-01 | code | COVERED | test_ac01_creates_structure, test_ac01_validates_input |
| ac-02 | code | MISSING | — |
| ac-03 | doc | EXEMPT | — (document deliverable) |
| ac-04 | code | COVERED | test_ac04_rejects_invalid |

**Coverage:** 2/3 testable ACs covered (67%)
**Exempt:** 1 non-code AC (doc)

### Orphaned Test Prefixes

| Test Function | Expected AC | Found in Spec? |
|---------------|-------------|----------------|
| test_ac99_legacy_check | ac-99 | No — no matching AC in any spec |

### Summary

| Spec | Testable | Covered | Exempt | Coverage |
|------|----------|---------|--------|----------|
| 2026-04-05-signal-frontier | 14 | 11 | 5 | 79% |
| 2026-04-06-regime-gate | 12 | 0 | 7 | 0% |
| **Total** | **26** | **11** | **12** | **42%** |

### Actionable Items

1. **ac-02** (2026-04-05-signal-frontier): Missing test — "Steps execute in declared order"
2. **ac-99** (orphan): Test exists but no AC — consider removing or linking to a spec
```

### 7. Recommendations

After the report, suggest concrete next steps:

- If ACs are missing `ac_type`: *"N ACs have no ac_type field. Run `/orb:spec` to update, or add `ac_type: code|doc|gate|config` manually."*
- If coverage is below 80% for code ACs: *"Coverage is below 80%. Consider adding tests for the MISSING items above."*
- If orphaned tests exist: *"N orphaned test prefixes found. These may reference superseded ACs — verify and clean up."*
- If naming format is inconsistent (mix of `ac-N` and `ac-NN`): *"Found inconsistent AC naming: some specs use ac-N, others use ac-NN. Standardise on ac-NN (zero-padded)."*
- If multiple specs exist but any lack `test_prefix`: *"N specs have no test_prefix in metadata. AC IDs collide across specs — add `test_prefix` to disambiguate test names (see decision 0002)."*

## Integration with Other Skills

- **`/orb:review-pr`** runs a subset of this audit (AC coverage check) during PR review
- **`/orb:implement`** tracks AC progress in `progress.md` — this audit cross-references against actual tests
- **`/orb:spec`** and `/orb:spec-architect` generate the AC IDs and `ac_type` fields that this audit reads

---

**Tip:** Run `/orb:audit` before `/orb:review-pr` to catch gaps early, or after implementation to verify coverage before opening a PR.
