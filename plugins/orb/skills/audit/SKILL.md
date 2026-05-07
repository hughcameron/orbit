---
name: audit
description: Audit AC-to-test traceability — find untested ACs, orphaned test prefixes, and coverage gaps
---

# /orb:audit

Audit the linkage between spec acceptance criteria and test files. Produces an actionable coverage report that distinguishes genuine gaps from ACs that are non-testable by design.

## Usage

```
/orb:audit [spec-id]
```

- If a spec id is provided: audit that single spec.
- If no argument: audit every spec listed by `orbit --json spec list`.

## Why This Exists

Orbit's AC-to-test naming convention (`ac-03` in the spec maps to `test_ac03_*` in tests) makes coverage a grep instead of a re-read. But the convention only works if someone actually checks it. This skill is that check.

It also handles the reality that not every AC is testable code. Document deliverables, gate decisions, and configuration changes don't have test functions — and shouldn't be flagged as gaps. The judgement of which ACs are non-code is made from each AC's `description` (and the `gate` flag where present), captured per-spec in the report below.

## Instructions

### 1. Locate Specs

If `$ARGUMENTS` provides a spec id, use it. Otherwise:

```bash
orbit --json spec list
```

…and audit every spec returned. The wire envelope's `data.result.specs[].id` field is the canonical id. If no specs exist, stop: *"No specs found. Run `/orb:spec` to create one."*

### 2. Parse Acceptance Criteria

For each spec, fetch its acceptance_criteria via:

```bash
plugins/orb/scripts/orbit-acceptance.sh acs <spec-id>
```

The parser emits one tab-separated tuple per AC: `<ac-id>\t<status>\t<description>\t<is_gate>`. The `<is_gate>` flag mirrors the spec's `acceptance_criteria[].gate` boolean.

orbit-state v0.1 specs do not carry an explicit `ac_type` field — all ACs are stored uniformly with `id`, `description`, `gate`, and `checked`. Decide non-code status from the AC's description text and gate marker (see classification below).

**AC classification (from description + gate flag):**

| Class | Detection signal | Test expected? |
|------|---------|----------------|
| `code` | description names runtime behaviour or a code change | Yes |
| `doc` | description names a documentation deliverable (decision record, runbook, README clause) | No |
| `gate` | `gate=1` AND description names a process/manual checkpoint without functional code (e.g. "tag pushed", "review approved") | No — but flag if description implies code |
| `config` | description names env vars, infra settings, CI workflow, gitignore | No |

A `gate=1` flag does **not** automatically mean non-code — many gates are gateways on testable behaviour. Use the description text as the source of truth.

### 3. Search for Matching Tests

For each `code`-class AC, search test directories for functions matching the AC ID.

If the project uses a per-spec test prefix (e.g. `remat`): search for the prefixed form first. The AC `ac-01` with prefix `remat` maps to `remat_ac01` in test names. Test prefixes are project conventions; orbit-state v0.1 specs do not store them in the spec record itself, so derive the prefix from the project's existing test naming (a quick `rg "fn ac0\\d|def test_ac0\\d|fn [a-z]+_ac0\\d" tests/` pass surfaces the convention).

Cross-language patterns (with optional `<prefix>_` prefix):

- Python: `def test_<prefix>_ac<NN>` or `def <prefix>_ac<NN>`
- Rust: `fn <prefix>_ac<NN>`
- TypeScript/JavaScript: `test('<prefix>_ac<NN>` or `it('<prefix>_ac<NN>`
- Go: `func Test<Prefix>Ac<NN>` or `func Test_<prefix>_ac<NN>`
- General fallback: grep for `<prefix>_ac<NN>` (or bare `ac<NN>`) prefix in any file under `tests/`, `test/`, or `__tests__/`

Handle both naming formats:

- Zero-padded: `ac-01` maps to `ac01` prefix in tests
- Unpadded: `ac-1` also maps to `ac01` prefix (normalise to two digits)

Also search for the prefix + `acNN` in test **file names** (e.g. `test_remat_ac03_ordering.py` or `test_ac03_ordering.py`).

### 4. Check for Orphaned Test Prefixes

Search test files for any `ac<NN>` prefixed functions that don't correspond to an AC in any current spec. These are either:

- Tests for ACs that were removed or superseded
- Typos in the AC number

Report these separately as orphans.

### 5. Check for Stale Specs

If a spec's status is `closed` (per `orbit spec list`), note it as **completed**. Its ACs still count but are informational, not actionable.

### 6. Produce the Report

Output the report in this format:

```
## AC Traceability Audit

**Date:** <today>
**Scope:** <spec-id or "all specs">

### <spec-id> (status: <open|closed>)

| AC | Class | Status | Test(s) |
|----|-------|--------|---------|
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

- If coverage is below 80% for code ACs: *"Coverage is below 80%. Consider adding tests for the MISSING items above."*
- If orphaned tests exist: *"N orphaned test prefixes found. These may reference superseded ACs — verify and clean up."*
- If naming format is inconsistent (mix of `ac-N` and `ac-NN`): *"Found inconsistent AC naming: some specs use ac-N, others use ac-NN. Standardise on ac-NN (zero-padded)."*
- If multiple specs share AC ids without test prefixes: *"AC ids collide across specs and your tests use bare `ac<NN>` names. Consider per-spec test prefixes (see decision 0002) to disambiguate."*

## Integration with Other Skills

- **`/orb:review-pr`** runs a subset of this audit (AC coverage check) during PR review.
- **`/orb:implement`** flips spec ACs to `checked: true` via `orbit-acceptance.sh check` (which calls `orbit spec update --ac-check`); this audit cross-references those `[x]` flags against actual tests.
- **`/orb:spec`** and `/orb:spec-architect` generate the AC ids and descriptions that this audit reads.

---

**Tip:** Run `/orb:audit` before `/orb:review-pr` to catch gaps early, or after implementation to verify coverage before opening a PR.
