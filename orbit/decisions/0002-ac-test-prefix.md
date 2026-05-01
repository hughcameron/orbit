---
status: superseded by 0013 (review-pr scope only — `test_prefix` remains live in spec/spec-architect/audit/implement)
date-created: 2026-04-17
date-modified: 2026-05-01
---
# 0002. Spec-Scoped Test Prefix for AC Namespace Disambiguation

## Context and Problem Statement

Orbit's AC naming convention assigns sequential `ac-NN` IDs per spec. When a project accumulates multiple specs, the same IDs appear in different specs (both have `ac-01`, `ac-02`, etc.). Tests using `test_ac01_*` are only disambiguated by which file they live in — grep-based audit and review-pr coverage checks can't reliably attribute tests to the correct spec.

Trigger incident: An audit of a multi-spec Rust project found four specs all using `ac-01` through `ac-NN` independently. Tests in multiple modules both used `test_ac01_*` prefixes. An ad-hoc version prefix in other modules was a partial workaround that didn't fully solve the collision.

## Considered Options

- **Option A: Spec `test_prefix` field** — Add an explicit `test_prefix` to spec metadata. Implementers use it in test names: `test_remat_ac01_*`. Audit and review-pr read the prefix when searching.
- **Option B: Auto-derive prefix from spec slug** — The spec directory name (`2026-04-16-sql-introspection`) auto-generates a prefix like `introspect_ac01`.
- **Option C: Globally unique AC IDs** — ACs get project-wide sequential IDs. First spec: ac-01 to ac-14. Second spec starts at ac-15.

## Decision Outcome

Chosen option: "Option A — Spec `test_prefix` field", because it is explicit, already field-tested in practice, backward-compatible with specs that lack the field, and the simplest change to orbit's skills. The prefix is a freeform spec identifier (e.g., a slug like `remat` or `introspect`), not a version number — `metadata.version` already carries the version.

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
- Good, because already proven in practice (ad-hoc prefixes were used to disambiguate in a real project)
- Bad, because spec authors must choose a meaningful prefix at creation time
- Mitigation: `/orb:spec` derives a default from the spec directory slug; authors can override
