# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-18-topology-substrate-migration
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (schema rewire, cross-system parity, deprecation surface, idempotent setup verb) | 2 |
| 3 — Adversarial | not triggered (no cascading dependency or rollback shape — every finding is a tighten-the-contract item, both LOW) | — |

## Findings

### [LOW] ac-02 names only one of the three legacy `audit_topology` tests slated for removal/rewrite
**Category:** test-gap
**Pass:** 2
**Description:** ac-02's verification line calls out the legacy test at `verbs.rs:7237` (`session_prime_topology_drift_none_when_docs_topology_unset` — the session-prime envelope test that asserts `docs.topology` unset → envelope absent). It does not name the two **verb-level** tests on the same predicate that will also break once the parser swap lands: `audit_topology_not_configured_when_docs_topology_unset` at `verbs.rs:7015` (asserts `configured == false` when the legacy config key is unset — predicate inverted by ac-02's new "configured iff `.orbit/topology/` exists AND ≥ 1 entry" rule) and `audit_topology_stale_pointer_when_topology_doc_missing` at `verbs.rs:7024` (asserts `drift_kind == "stale_pointer"` when the markdown file is missing — the drift-code vocabulary ac-02 explicitly routes to the implementer's call). Both fall in the same "predicate that no longer exists" bucket as the named 7237 test.
**Evidence:**
- `orbit-state/crates/core/src/verbs.rs:7015` — `fn audit_topology_not_configured_when_docs_topology_unset()` — writes `"{}\n"` to `config_file` and asserts `!result.configured`. Under ac-02 the empty-config path no longer drives `configured`; the empty-topology-dir path does.
- `orbit-state/crates/core/src/verbs.rs:7024` — `fn audit_topology_stale_pointer_when_topology_doc_missing()` — writes `docs:\n  topology: docs/topology.md\n` and asserts `drift_kind == "stale_pointer"`. Under ac-02 the wiring source is `.orbit/topology/*.yaml`, not the legacy config pointer.
- ac-02 verification text: "Assert the verbs.rs:7237 legacy test (config-present-but-docs.topology-absent) is removed or rewritten…" — singular, names only the session-prime test.
**Recommendation:** Tighten ac-02's verification surface to: "Assert the three legacy `audit_topology` tests on the dropped predicate (`verbs.rs:7015 audit_topology_not_configured_when_docs_topology_unset`, `verbs.rs:7024 audit_topology_stale_pointer_when_topology_doc_missing`, `verbs.rs:7237 session_prime_topology_drift_none_when_docs_topology_unset`) are removed or rewritten to cover the new empty-directory path." One-line addition; closes the legacy-test cleanup checklist.

### [LOW] ac-04's "deprecated and unused" framing leaves runtime signalling unstated
**Category:** test-gap
**Pass:** 2
**Description:** ac-04 lands Option A from cycle-1 — `DocsConfig::topology` is retained as a parse-only deprecated field — and pins write-time behaviour ("preserved on canonical write, follow-on spec deletes the field"). The cycle-2 LOW finding on round-trip behaviour is now closed. What remains undecided is whether **any runtime signal** fires when a brownfield config carrying `docs.topology` is loaded — a session-prime info-line, a one-shot deprecation warning, or nothing. Cycle-1 Option A originally suggested "a deprecation warning is emitted once"; ac-04 retains a source-level doc comment but does not say whether the deprecation surfaces at runtime. Two defensible answers, observably different: (a) silent — the doc comment is the only signal, brownfield operators discover the deprecation by reading release notes; (b) audible — `session-prime` (or `verify`) emits a one-time `deprecated_config_key` notice the first time it sees the field. Either is fine; the spec doesn't pin which.
**Evidence:**
- ac-04 description: "no code path reads it; serde continues to accept it so brownfield consumer repos … do not hard-fail Config::from_str on session-prime." — names *parse* behaviour, leaves *notification* behaviour blank.
- ac-04 verification: round-trip on `from_str` + canonical writer; grep for the deprecation doc comment. No verification line on runtime warning.
- `orbit-state/crates/core/src/schema.rs:469` — existing doc comment on `DocsConfig`: "Currently surfaces the `docs.topology` pointer used by the …" — implementation cue, no operator-facing emission today.
**Recommendation:** Add one sentence to ac-04: "No runtime signal fires when the deprecated field is parsed — discovery is via source-level doc comment, `.orbit/config.yaml` schema docs, and the follow-on deletion spec's release notes." (Silent path — matches the design intent of "brownfield does not hard-fail" without polluting session-prime output.) Alternatively, if a one-time warning is wanted, name where it surfaces (session-prime envelope key, `orbit verify` output, etc.). Pin one to give the implementer a closed contract.

---

## Honest Assessment

The cycle-3 spec materially closes both MEDIUMs and both LOWs from cycle-2. ac-01's helper-name error is replaced with the canonical two-step pattern (`resolve_numeric_slug(VERB, &layout.choices_dir(), id)` then `layout.choice_file(&resolved)`), verified against `layout.rs:114-119`. ac-02 correctly names `parse_topology_doc` (at `verbs.rs:2910`) and `load_topology_subsystem_names` (at `verbs.rs:2985`, called from `verbs.rs:1652`), with `load_topology_entries` (a name that never existed) removed. ac-04 pins write-time round-trip to Option A explicitly. ac-05 carries the two-stage idempotency verification line. The substrate-engagement parity contract from the predecessor spec remains the regression net.

The two LOW findings here are precision items, not design defects. Finding 1 is a one-line addition that broadens the legacy-test cleanup surface from one test to three — the implementer will hit the additional failures during `cargo test` anyway, but listing them in the AC closes the checklist before implementation. Finding 2 names a runtime-signalling gap that the spec doesn't pin; both paths (silent, audible-once) are defensible — the cost of leaving it unpinned is that close-time evidence may reflect whichever the implementer picked rather than a deliberate design choice.

Neither blocks implementation. The spec is structurally sound, the goal-to-AC mapping is clean, the supersession story stands, and the verification lines now point at functions and lines that exist in the codebase. Recommend APPROVE; the two LOWs can be addressed as small spec edits before drive enters implement, or accepted as implementer-discretion items.
