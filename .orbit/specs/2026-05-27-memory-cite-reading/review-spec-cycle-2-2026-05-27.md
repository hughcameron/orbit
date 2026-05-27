# Spec Review

**Date:** 2026-05-27
**Reviewer:** Context-separated agent (fresh session, cycle 2)
**Spec:** 2026-05-27-memory-cite-reading
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals: schema change, backwards-compat, MCP parity | 0 |
| 3 — Adversarial | not triggered | — |

## Pass 1 deterministic checks

- AC parser output: 7 ACs (ac-01..ac-07). Gates: **ac-01..ac-07 all `is_gate=1`** after cycle-1 promotion of ac-06 and ac-07.
- Gate-AC description rules (non-empty / non-placeholder / >=20 chars): **all seven gate ACs pass**.
- AC testability: every AC carries an inline `Verification:` clause naming a concrete mechanic (unit test, integration test, parity test, schema test, four-target grep). Test prefix `cite` declared on every AC.
- Scope vs goal: clean — goal claims diagnostic honesty under memory compression + structural detectability; ACs deliver detect (ac-01/02), record (ac-03), enforce (ac-04), name (ac-05), surface (ac-06), regress (ac-07).
- Content signals present (schema change, backwards-compat, MCP parity) — Pass 2 triggered.

## Cycle-1 finding closure

### [HIGH] ac-01 substrate framing — **CLOSED**
Cycle-1 finding: ac-01 mis-framed memories as SQLite-backed; demanded a DB migration test against a substrate that doesn't exist.
Fix applied: ac-01 description now reads "remember -> .orbit/memories/<key>.yaml file -> show -> match" and carries an explicit substrate note naming `layout.rs:126 memory_file` and `verbs.rs memory_remember`. Backward-compat mechanism named correctly as `#[serde(default)]` + `#[serde(skip_serializing_if = "Vec::is_empty")]`. Verification (d) now asserts the fixture file deserialises as `cites: []` via serde default and re-serialisation does not write `cites: []` back (byte-identical-on-disk preserved). Verified against `schema.rs:665-673` (Memory struct is plain serde over YAML, no SQLite) and `layout.rs` memory_file resolver.

### [MEDIUM] ac-04 loop description — **CLOSED**
Cycle-1 finding: AC ambiguous about which loop, which collection.
Fix applied: ac-04 now reads "second pass that iterates entries in `spec.memories_considered`, loads each referenced memory, and for any whose `cites` field is non-empty, asserts every cite_path appears in that entry's `cite_evidence` list. Runs AFTER the existing corpus-vs-considered pre-flight at orbit-state/crates/core/src/verbs.rs:2317-2363 ...; does not modify that existing loop." The collection-vs-collection distinction and the don't-modify-existing constraint are now both explicit. Verified at `verbs.rs:2323-2346` — existing loop iterates `memory_matches`, the new pass iterates `memories_considered` per the AC.

### [MEDIUM] ac-05 fourth grep target — **CLOSED**
Cycle-1 finding: grep mechanic could be satisfied by SKILL.md prose that names the shape without teaching excerpt-must-be-from-file discipline.
Fix applied: ac-05 verification mechanic now lists four targets (a)-(d); (d) reads "explicit direction that the excerpt MUST be drawn from the file contents at `cite_path` — not paraphrased from the memory body or synthesised." The AC text now self-cites the closure of tabletop halt-condition #2.

### [MEDIUM] ac-07 gate promotion — **CLOSED**
Cycle-1 finding: regression suite (the structural proof of backward-compat halt-condition) was gate=false.
Fix applied: `orbit spec acs` now reports `ac-07 ... 1`. Confirmed in parser output above.

### [LOW] ac-06 MCP parity gate promotion — **CLOSED**
Cycle-1 finding: MCP parity gate posture was suggested either-way; the choice should be deliberate. The author promoted to gate=true per the agent-facing-surface argument. Confirmed in parser output (`ac-06 ... 1`).

## Findings

None.

---

## Honest Assessment

Every cycle-1 finding is addressed in substance, not just in wording. The HIGH substrate-framing finding got the full rewrite it needed — ac-01 now describes the real YAML-file substrate, the real backward-compat mechanism, and a test (d) that is actually writable against the code. The two MEDIUM language-precision findings (ac-04 loop, ac-05 grep) closed cleanly with surgical edits that name the discipline they were missing. Both gate promotions (ac-06, ac-07) landed. The spec is ready for /orb:drive to open implementation. The biggest residual risk is the one the tabletop already names as kill-condition #2 — excerpt evidence being fluently faked — and the spec correctly leaves that as an in-flight escalation rather than pre-solving it. Good shape; ship.
