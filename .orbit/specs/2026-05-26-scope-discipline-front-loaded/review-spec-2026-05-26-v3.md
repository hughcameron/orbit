# Spec Review

**Date:** 2026-05-26
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-26-scope-discipline-front-loaded
**Verdict:** APPROVE

---

Every blocker and observation from the v1 and v2 reviews has landed. AC-10 introduces `Spec.closed_at: Option<DateTime<Utc>>` with serde-default `None` (closing v2's HIGH); AC-04 now defines the canonical `deferred-scenario:` prefix, the parsed tuple shape, the `ConformanceFinding.evidence` shape, malformed-note silent-skip semantics, and an open-followup suppression test; AC-05 pins strict-before exclusion with an inclusive boundary test and a None-as-pre-window rule; AC-07 captures the test-count baseline in a `baseline_test_count: NNNN at SHA xxxxxxx` spec note before first commit. The dog-fooded `verifies:` classification appears on every AC. Substrate spot-checks confirm the spec's claims are buildable against the existing core: `Spec.notes: Vec<String>` already supports the prefix convention (schema.rs:545), the `"cards"` subsystem token already exists in audit conformance (verbs.rs:12103, :12208), and tabletop/SKILL.md is exactly the 94-line baseline AC-09 names.

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 structural; content signals present (schema change, cross-system audit surface) |
| 2 — Assumption & failure | content signals | 0 |
| 3 — Adversarial | not triggered | — |

## Findings

None blocking. Two notes for the implementing agent — neither is a verdict-changer:

### [INFO] AC-10's `closed_at` write must land before AC-05's audit-window test fixtures can pass

**Category:** ordering
**Pass:** 2
**Description:** AC-05's tests seed specs with `closed_at` set to specific values (pre-window, post-window, exactly the introduction date, `None`). The fixture writes depend on the schema field AC-10 introduces. Implement AC-10 before AC-05 — the spec doesn't ordain the order but the data dependency is one-way. Not a finding against the spec; flagged so the drive cycle picks AC-10 first.

**Evidence:** AC-05 verification names tests that set `closed_at` on fixtures; AC-10 verification introduces the schema field.

**Recommendation:** No spec change. Drive picks AC-10 before AC-05 by data dependency.

### [INFO] AC-04 test asserts `cumulative_count: 2` literal; AC-05 boundary test reuses the two-deferral fixture shape

**Category:** test-shape consistency
**Pass:** 2
**Description:** AC-04's `scope_emits_card_coverage_finding_on_two_deferrals` and AC-05's two tests all construct two-deferral fixtures. The shared shape is good (one canonical fixture-builder serves all three tests) but the spec doesn't name it as a shared helper — implementing agent may build three copies. Cosmetic.

**Evidence:** AC-04 verification names two scenarios `scope-test-a` and `scope-test-b` in the fixture; AC-05 verification names "the same two-deferral fixture" without specifying scenario names. Either repeat names or factor.

**Recommendation:** No spec change. Implementing agent factors the fixture-builder at their discretion.

---

## Honest Assessment

The spec is ready. It dog-foods its own discipline — every AC carries the `verifies:` classification AC-02 and AC-03 install for future specs; halts and escalations live in the tabletop sidecar per the canonical seven-field Spec schema; the audit-window gate (AC-05) and `closed_at` field (AC-10) form the substrate dependency-chain that prevents retroactive finding-spam on historical specs. The recursive irony the v1 review warned about — under-specified scope-discipline spec — does not apply: AC-04 names the prefix, the parsed tuple, the evidence YAML, malformed-note handling, and the suppression test; AC-05 names the boundary semantics with explicit `<` vs `≤` direction. The biggest residual risk is operational, not specificational: AC-08's first-run vacuous pass means real threshold validation arrives only as new specs close under the discipline — and the spec acknowledges this in AC-08's prose. Halt-2 stands ready if the threshold proves wrong in practice.

Proceed to `/orb:implement`. Start with AC-10 (schema field) so AC-05's fixtures have somewhere to write.
