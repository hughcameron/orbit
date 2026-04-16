---
status: accepted
date-created: 2026-04-17
date-modified: 2026-04-17
---
# 0002. Spec-Scoped Test Prefix for AC Namespace Disambiguation

## Context and Problem Statement

Orbit's AC naming convention assigns sequential `ac-NN` IDs per spec. When a project accumulates multiple specs, the same IDs appear in different specs (both have `ac-01`, `ac-02`, etc.). Tests using `test_ac01_*` are only disambiguated by which file they live in — grep-based audit and review-pr coverage checks can't reliably attribute tests to the correct spec.

Trigger incident: Nightingale audit of Meridian's DuckDB extension found four specs all using `ac-01` through `ac-NN` independently. Tests in `introspect.rs` and `state.rs` both used `test_ac01_*` prefixes. The `v03_ac` prefix in `runner.rs` and `asset.rs` was an ad-hoc workaround that partially solved the collision.

## Considered Options

- **Option A: Spec `test_prefix` field** — Add an explicit `test_prefix` to spec metadata. Implementers use it in test names: `test_v03_ac01_*`. Audit and review-pr read the prefix when searching.
- **Option B: Auto-derive prefix from spec slug** — The spec directory name (`2026-04-16-sql-introspection`) auto-generates a prefix like `introspect_ac01`.
- **Option C: Globally unique AC IDs** — ACs get project-wide sequential IDs. First spec: ac-01 to ac-14. Second spec starts at ac-15.

## Decision Outcome

Chosen option: "Option A — Spec `test_prefix` field", because it is explicit, already field-tested in Meridian (`v03_ac`), backward-compatible with specs that lack the field, and the simplest change to orbit's skills.

### What Changed

- `spec-architect/SKILL.md`: Added `test_prefix` to metadata template and AC naming section
- `spec/SKILL.md`: Added `test_prefix` to the YAML format example
- `audit/SKILL.md`: Updated test search to use `test_prefix` when present
- `review-pr/SKILL.md`: Updated AC coverage check to use `test_prefix` when present
- `implement/SKILL.md`: Reference the prefix in progress checklist
- `README.md`: Updated AC naming documentation

### Consequences

- Good, because AC-to-test mapping becomes unambiguous across multi-spec projects
- Good, because backward-compatible — specs without `test_prefix` work exactly as before
- Good, because the prefix is explicit and auditable in the spec YAML
- Good, because already proven in practice (Meridian `v03_ac` convention)
- Bad, because spec authors must choose a meaningful prefix at creation time
- Mitigation: `/orb:spec` derives a default from the spec version or slug; authors can override
