---
date: 2026-05-24
interviewer: claude
card: .orbit/cards/0039-workflow-conformance.yaml
mode: rally
rally: 2026-05-24-brownfield-migration-hardening-rally
---

# Design: undotted-substrate conformance finding with downstream-symptom suppression

## What good looks like

When a project has substrate at `orbit/` (no dot) instead of `.orbit/`, `orbit audit conformance` reports exactly one finding — `[high] setup/undotted_substrate orbit/` — with remediation verb `orbit setup`. The misleading downstream symptoms (METHOD.md missing, no cards visible, no memos, unpinned config) are suppressed until the layout is fixed, so the agent and `/orb:prioritise` see the real problem instead of chasing surface findings. The finding's `evidence` carries substrate volume counts so downstream consumers can convey scale ("22-card migration" vs "empty scaffold").

---

## Context

Card: *workflow-conformance* — 8 scenarios, maturity emerging, goal: agent-on-demand audit of substrate vs plugin contract with per-finding remediation verbs.

Prior specs: 2 — `2026-05-19-workflow-conformance` shipped the conformance verb with three finding families (plugin-canonical drift, card-state, memo staleness); `2026-05-20-conformance-park-signal` added park-signal awareness to the card-state family.

Gap, from `.orbit/memos/2026-05-24-brownfield-setup-friction.md` (item #1) plus the arcform session: `orbit audit conformance` currently emits MEDIUM "METHOD.md missing → run `orbit setup`" when the substrate lives under `orbit/` (no dot). The finding is technically correct but masks the real problem; running setup naïvely against this remediation would create `.orbit/` alongside `orbit/`, leaving two parallel substrate folders. The `/orb:prioritise` single-verb-remediation contract breaks here — the verb the audit names would make things worse.

Cross-drive: the sister drive on card 0017 implements the `wrapped-undotted` setup state that this finding's remediation invokes. Both drives share a detector predicate via a new Rust verb in `orbit-state`.

---

## Q&A

### Q1: Canonical predicate
**Q:** What positive signal does the detector use? Single canonical dir, any-of substrate dirs, or stricter?
**A:** Any-of: any of `orbit/{cards,choices,specs,memos}/` exists AND `.orbit/cards/` does not. The negative guard (`.orbit/cards/` absent) is the real filter; once canonical substrate exists, the finding suppresses regardless. The wider net handles partial-migration mid-states without a separate finding family. Four `stat` calls, short-circuit on first hit.

### Q2: Suppression mechanism
**Q:** Emission-time skip, post-processing filter, or separate composer?
**A:** Emission-time skip, mirroring the existing `pin_dominates` pattern at `verbs.rs:3920`. Add `layout_dominates: bool` alongside `pin_dominates`; extend the guard to `if !pin_dominates && !layout_dominates`. Same precedent, smallest diff, no wasted work on suppressed findings.

### Q3: Finding identity
**Q:** Extend the `setup` subsystem (where canonical-file and pin findings already live), add a new `layout` subsystem, or use repo-root as subject?
**A:** Subsystem `setup`, state `undotted_substrate`, subject = `orbit/` (literal folder name, repo-relative). Matches the existing pattern — pin findings sit under `setup` with `.orbit/config.yaml` as subject; canonical-file findings sit under `setup` with file path as subject. The new finding fits the same shape.

### Q4: Severity floor
**Q:** Always HIGH, or downgrade when substrate is empty?
**A:** Always HIGH, with `evidence` carrying counts of cards/choices/specs/memos under each `orbit/<subdir>/`. Severity stays simple (matches locked decision); evidence enriches the finding for any consumer wanting "22-card migration" vs "empty scaffold" granularity. Four `read_dir` calls at audit time, gated on predicate firing.

### Q5: State slug name
**Q:** `non_canonical_layout`, `undotted_substrate`, or `legacy_orbit_folder`?
**A:** `undotted_substrate`. Precise (names exactly what's wrong), greppable across code/prose/history, matches the seed memo's diagnostic language. Reads in CLI output as `[high] setup/undotted_substrate orbit/` — diagnostic is immediate.

### Q6: Suppression scope
**Q:** Suppress only canonical-files-missing (locked-decision minimum), or also card-state / memo-staleness / pin-state?
**A:** Suppress canonical-files-missing AND card-state AND memo-staleness AND pin-state. Only routine-findings (read from `.claude/skills/`, not `.orbit/`) and the aggregated `audit_drift`/`audit_topology` continue to fire. Rationale: when layout is wrong, every other finding family is structurally meaningless until `.orbit/` becomes canonical. Delivers `/orb:prioritise`'s single-verb-remediation contract — agent sees exactly one finding.

---

## Summary

### Goal

`orbit audit conformance` emits a `[high] setup/undotted_substrate orbit/` finding when any of `orbit/{cards,choices,specs,memos}/` exists AND `.orbit/cards/` does not. The finding's `evidence` carries substrate volume counts. When this finding fires, the four downstream-symptom families (canonical-files-missing, card-state, memo-staleness, pin-state) are suppressed via the existing `pin_dominates`-style emission-time skip pattern. Remediation verb is `orbit setup`. `/orb:prioritise` sees one verb; running it resolves the layout, after which a re-run of conformance shows the real state of the (now-canonical) substrate.

### Constraints

- Predicate is any-of `orbit/{cards,choices,specs,memos}/` exists AND `.orbit/cards/` does not. Four `stat` calls, short-circuit on first hit.
- Detector function `undotted_substrate_finding(layout) -> Option<ConformanceFinding>` lives in `orbit-state/crates/core/src/verbs.rs` next to `canonical_file_findings` (~line 4091).
- Severity is `high` unconditionally. `evidence` map carries `{cards_count, choices_count, specs_count, memos_count}` under `orbit/`.
- Suppression is emission-time skip (extends the existing `if !pin_dominates` guard to `if !pin_dominates && !layout_dominates`). Suppressed families: canonical-files-missing, card-state, memo-staleness, pin-state. Unsuppressed: routine-findings, aggregated `audit_drift`, aggregated `audit_topology`.
- Finding shape: `subsystem: "setup"`, `state: "undotted_substrate"`, `subject: "orbit/"`, `severity: "high"`, `remediation.verb: "orbit setup"`.
- No new subsystem name in `ConformanceFinding`. No new severity value.
- Detector predicate is shared with Drive A's setup state classifier — both call the same orbit-state helper.

### Success Criteria

- A repo with `orbit/cards/` populated and `.orbit/` absent produces exactly one finding under `orbit audit conformance`: `[high] setup/undotted_substrate orbit/`, with evidence carrying the four counts.
- The same repo's canonical-files-missing / card-state / memo-staleness / pin-state findings do NOT appear in the envelope while the `undotted_substrate` finding fires.
- A repo where `.orbit/cards/` exists (canonical substrate present) produces no `undotted_substrate` finding regardless of whether an `orbit/` directory exists for unrelated reasons.
- Once Drive A's setup runs and migrates `orbit/` → `.orbit/`, a re-invocation of `orbit audit conformance` no longer fires `undotted_substrate`, and the previously-suppressed finding families fire as appropriate against the now-canonical substrate.
- `orbit verify` is clean on the new finding schema (no field changes; only new state slug value).
- Parity test in `orbit-state/crates/cli/tests/parity.rs` covers the new finding via CLI envelope. MCP parity test in `crates/mcp/tests/parity.rs` covers the same path.
- The four suppressed finding families remain reachable in tests where only their original predicates fire (no regression).

### Decisions Surfaced

- **Any-of predicate over single-dir.** The wider net handles partial-migration shapes; the negative guard (`.orbit/cards/` absent) is the real filter. (Q1)
- **Emission-time skip mirroring `pin_dominates`.** One-line extension to existing guard, same test pattern, smallest diff. Rejected post-processing filter (wasted work, leaks state from suppressed family). (Q2)
- **Subsystem `setup`, not a new `layout` subsystem.** New subsystem axis is over-engineered for one finding; existing `setup` axis matches every finding whose remediation is `orbit setup`. (Q3)
- **HIGH unconditional with evidence counts.** Severity stays simple; evidence enriches consumers without a conditional severity branch. (Q4)
- **`undotted_substrate` slug.** Precise, greppable, matches memo language. Rejected `non_canonical_layout` (generalist, invites future-state confusion) and `legacy_orbit_folder` (historical, ages poorly). (Q5)
- **Full downstream-symptom suppression.** Suppress canonical-files-missing + card-state + memo-staleness + pin-state — every family that reads from `.orbit/` is structurally meaningless during this state. Honours `/orb:prioritise`'s single-verb-remediation contract. (Q6)

### Implementation Notes

**Codebase leads:**

- Audit verb: `orbit-state/crates/core/src/verbs.rs::audit_conformance_at` (~line 3902). Existing `pin_dominates` guard at ~line 3920 is the precedent for emission-time skip.
- Finding builders: sibling functions `canonical_file_findings` (~4087), `card_state_findings`, `memo_staleness_findings`, `routine_findings` (~3953-4271). All four except `routine_findings` will be gated on `!layout_dominates`.
- Finding schema: `ConformanceFinding` struct at `verbs.rs:1224`. State slugs are forward-compatible strings — no schema change needed to add `undotted_substrate`.
- Layout helper: `OrbitLayout::repo_root()` at `layout.rs:271` returns the parent of `.orbit/`. The predicate needs `repo_root().join("orbit").join(<subdir>)`, not anything off existing `OrbitLayout` accessors (which all anchor at `.orbit/`).
- Parity tests: `orbit-state/crates/cli/tests/parity.rs` and `orbit-state/crates/mcp/tests/parity.rs` — extend with a fixture covering the new finding.

**New surface:**

- `undotted_substrate_finding(layout) -> Option<ConformanceFinding>` in `verbs.rs`, returning `Some` when the predicate fires.
- `layout_dominates: bool` local in `audit_conformance_at`, derived from the new helper.
- Fixture directory in `orbit-state/crates/core/tests/fixtures/` simulating an `orbit/` brownfield layout with populated subdirs.

**Cross-drive coupling:**

- Drive A (card 0017) implements the setup-state classifier that this drive's detector shares. Coordination: both call the same `orbit-state` helper. If both drives ship simultaneously, the helper lands in one drive's diff (likely A — its scope establishes the state machine); this drive consumes it. If implementation runs serially (chain wired in disjointness check), the helper ships in whichever drive runs first and the second drive's diff references it.
- Drive A also ships the new `decisions-md-unmigrated` conformance finding family (locked cross-drive decision). This drive's diff does NOT include that finding builder.

**Memory dispositions:**

- `audit-conformance-cwd-dependent` — *adopted procedurally*. Fixture tests use `--root` or absolute paths.
- `private-projects-genericised-in-artefacts` — *not applicable*. arcform is meridian-online family.

**Out of scope:**

- The new finding family `decisions-md-unmigrated` (Drive A ships it; locked cross-drive decision).
- Any change to existing suppressed findings (card-state, memo-staleness, pin-state) — they're gated, not modified.
- Verb-side aggregation of evidence counts into a single rolled-up integer — keep the four-value evidence map for downstream consumer flexibility.
