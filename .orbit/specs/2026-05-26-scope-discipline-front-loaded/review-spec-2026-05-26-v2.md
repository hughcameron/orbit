# Spec Review

**Date:** 2026-05-26
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-26-scope-discipline-front-loaded
**Verdict:** REQUEST_CHANGES

---

The spec is one revision away from approval. The v1 review's two blockers and four observations have all been addressed in this cycle — the deferred-scenario prefix convention is named, `<card-id>:<scenario-name>` anchoring is explicit, the evidence YAML shape is structured, the negative-scope is tightened, AC-07 baseline is captured, AC-08's vacuous-pass framing is acknowledged, and the truncated memory rationale is fully written. A new substrate-level blocker is now the only thing standing in the way: AC-05's audit-window gate relies on a spec close-timestamp field that the Spec schema does not carry. Two narrower clarifications follow it.

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 structural; 2 content signals (cross-system surface + new convention/schema-adjacent shape) |
| 2 — Assumption & failure | content signals + cross-substrate dependencies | 1 HIGH, 2 MEDIUM, 1 LOW |
| 3 — Adversarial | not triggered | — |

## Findings

### [HIGH] AC-05 names a "spec's close timestamp" that the substrate does not carry

**Category:** missing-requirement
**Pass:** 2
**Description:** AC-05 specifies *"A timestamp gate compares each spec's close-date (or the audit-rule's introduction date constant) against the spec's close timestamp; pre-ship specs are excluded by audit-window contract."* The `Spec` struct in `orbit-state/crates/core/src/schema.rs:262-289` carries `status: SpecStatus::{Open, Closed}` as a binary enum — there is no `closed_at`, `close_date`, or equivalent timestamp field on Spec or anywhere in the orbit-state core schema (grep `closed_at\|close_at\|close_date\|closed_on` across `orbit-state/crates/` returns zero matches). The implementing agent has no canonical timestamp to compare against the introduction-date constant.

**Evidence:**
- `orbit-state/crates/core/src/schema.rs:262-289` — Spec field list; only `status: SpecStatus` for close state.
- `orbit-state/crates/core/src/schema.rs:291-296` — `SpecStatus` is `Open | Closed`, no timestamp.
- AC-05 description: *"a timestamp gate compares… against the spec's close timestamp"*.
- AC-05 verification: test `scope_audit_window_excludes_pre_ship_specs` "seeds two specs (one closed BEFORE the audit-rule introduction date constant, one AFTER)" — the test fixture itself needs a substrate field to set.

**Recommendation:** Pick one before implement begins:
- (a) Add an AC that introduces `Spec.closed_at: Option<DateTime>` (set on `spec.close`) and update AC-05 to read from it. This is the cleanest mechanism but expands spec scope by one schema field + migration consideration for existing closed specs.
- (b) Reuse the spec file's git-blame-of-status-line-change as the close timestamp, and name that explicitly in AC-05's verification. Avoids schema change but couples the audit to git history shape.
- (c) Reuse the spec file's filesystem mtime when status transitions to `Closed` (recorded into a sidecar manifest at close time). New manifest = new convention.
- (d) Drop AC-05's audit-window gate and instead use a hardcoded spec-id allow-list of pre-ship specs to exclude. Crudest but smallest change; auditable from the source constant alone.

Pick (a) is the right shape long-term and matches the spec's discipline (substrate-encoded, not vigilance-encoded). Picks (b)-(d) are smaller patches if the spec wants to stay one-revision-from-approval; the choice belongs in this spec, not deferred — this is the same recursive-irony point the v1 review made about AC-04's note shape.

### [MEDIUM] AC-04 lacks a follow-up-spec-open suppression test

**Category:** test-gap
**Pass:** 2
**Description:** AC-04's firing rule has three conjunctive conditions: (a) closed specs exist, (b) cumulative deferrals ≥ 2, (c) no follow-up spec is currently open against the card. The verification test `scope_emits_card_coverage_finding_on_two_deferrals` exercises (a) + (b) and the positive case. Nothing tests (c) — that the finding is suppressed when an open spec exists against the same card. A bug in the (c) branch ships untested.

**Evidence:** AC-04 description naming the three-condition rule; AC-04 verification naming only the positive test fixture. AC-07's `cargo test --workspace -- scope_` count assertion (≥2) is satisfied by AC-04 + AC-05 tests, so adding a third test for (c) does not break AC-07.

**Recommendation:** Add a sentence to AC-04's verification or a sibling AC: *"A second test `scope_card_coverage_suppressed_by_open_followup_spec` constructs the same two-deferral fixture plus an open spec referencing the card, runs the audit, and asserts no card-coverage finding fires."* This bumps AC-07's expected delta from ≥2 to ≥3 if AC-07 is amended.

### [MEDIUM] AC-05's introduction-date boundary is unspecified (inclusive vs exclusive)

**Category:** test-gap
**Pass:** 2
**Description:** AC-05 says pre-ship specs are excluded. The introduction-date constant choice and the comparison semantics at the boundary (specs closed *on* the introduction date — included or excluded?) are left to the implementing agent. On a merge day the constant might be set to the merge day itself, and any spec closed in the hours-window before or after the merge will land on or near the boundary. A test fixture that closes a spec at exactly the introduction date will pass or fail depending on `<` vs `≤`, and the spec doesn't pin which.

**Evidence:** AC-05 description names "compares" without operator; the test name `scope_audit_window_excludes_pre_ship_specs` does not specify boundary handling.

**Recommendation:** Add one sentence to AC-05 naming the operator: *"Specs closed strictly before the introduction date are excluded; specs closed on or after are included."* (Or the opposite — pick one, write it.) Update the test name or a sibling test to cover the boundary explicitly.

### [LOW] AC-04 does not address malformed deferred-scenario notes

**Category:** failure-mode
**Pass:** 2
**Description:** The deferred-scenario prefix convention (`deferred-scenario: <card-id>:<scenario-name> -- <rationale>`) is now well-defined. AC-04's audit walks each closed spec's notes, matches the prefix, parses the tuple. What happens when a note starts with the prefix but the body is malformed (missing `--`, scenario-name not in card.scenarios[].name, card-id doesn't match any card)? Three reasonable behaviours: ignore (silent), warn (emit a separate finding), error (audit fails). AC-04 picks none.

**Evidence:** AC-04 description names the parse contract; the verification test constructs only well-formed notes.

**Recommendation:** Add a sentence to AC-04: *"Malformed notes (missing tuple separator, unknown card-id, scenario-name not found on card.scenarios[].name) are silently skipped — the audit emits no finding for them and continues parsing remaining notes."* (Silent-skip is the right default for an audit; it matches how `audit.conformance`'s existing finding families handle unparseable input. Surface the choice in prose.)

---

## Honest Assessment

The spec is in good shape — the v1 review's blockers and observations are cleanly resolved, the dog-fooded classification convention is consistently applied, and the halt/escalation/kill conditions in the tabletop sidecar give the implementing agent revert paths for the two most likely failure modes (rebloat, audit overfire). The HIGH finding is the one substantive risk: AC-05 names a substrate field that doesn't exist, and the implementing agent will hit it the moment they try to write the audit-window test. The other three findings are tightening — small clarifications that prevent the implementing agent from making picks that should belong to the spec author. None require a re-tabletop; all are AC-text edits the spec author can make in one pass before re-review.
