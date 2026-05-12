# Spec Review

**Date:** 2026-05-12
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-12-reconcile-mode
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (schema migration, cross-system substrate) | 2 |
| 3 — Adversarial | not triggered | — |

Pass 1 #5 deterministic gate-description check: gate ACs are ac-01, ac-03, ac-04, ac-08. All four descriptions are non-empty, none match a placeholder token (TBD/TODO/FIXME/PLACEHOLDER/XXX/???), and all four are well above 20 characters (shortest is ac-01 at 421 chars). The gate-description check passes.

The v1 review (`review-spec-2026-05-12.md`) was REQUEST_CHANGES on five findings. The current spec materially addresses each: AC-03 now names `run_canonicalise`'s hand-rolled envelope at `cli/src/main.rs:496-516` directly (replaces the missing `audit drift --dry-run` referent); AC-02 explicitly scopes v1 to rename-only and drop, deferring value transforms; AC-04 covers inner-shape drift via new `AcceptanceCriterion::FIELDS` / `Scenario::FIELDS` / `Relation::FIELDS` constants kept in lockstep with their structs by unit test; AC-06 defines merge semantics on existing sidecars and parsed-`Value` equality; AC-08(a) names §3g (after §3d's `git mv` transaction) as the insertion point in `/orb:setup`. Substrate spot-checks confirm the referents exist as named — `run_canonicalise`'s envelope is at the cited line range, the four top-level `FIELDS` constants are at `schema.rs:52-92`, and the inner structs are declared with `deny_unknown_fields` so the new inner constants slot in without schema disruption.

The remaining findings below are minor — none block implementation. Promoting to APPROVE.

---

## Findings

### [LOW] AC-03 exit-code semantics specified for `--dry-run` only
**Category:** test-gap
**Pass:** 1
**Description:** AC-03 says "Exit code signals whether a non-dry-run would change anything (non-zero when `dispositions` is non-empty)" — but the sentence sits inside the dry-run AC and is ambiguous about the non-dry-run path. The existing `run_canonicalise` (cli/src/main.rs:493-560) exits non-zero only on parse failures; a successful rewrite returns zero. Reasonable implementers will read AC-03 two ways: (a) only `--reconcile --dry-run` follows the new exit-code rule, the non-dry-run path keeps the existing parse-failures-only semantics; (b) both paths exit non-zero whenever dispositions were applied/would-be-applied. Reading (a) preserves the existing CLI contract; reading (b) means a successful reconcile run reports non-zero, which is unusual.
**Evidence:** AC-03 sentence "Exit code signals whether a non-dry-run would change anything (non-zero when `dispositions` is non-empty)" reads as a dry-run-only statement; AC-02 (the non-dry-run AC) is silent on exit code.
**Recommendation:** No spec change required if reading (a) is intended — the implementer should default to reading (a) (dry-run only; non-dry-run keeps existing parse-failure exit semantics) and surface in the PR if reading (b) is wanted. Add one sentence to AC-02 ("Non-dry-run exit code follows the existing `run_canonicalise` contract: zero on success, non-zero on parse failure") only if the ambiguity is worth pre-empting.

### [LOW] AC-05 registry key dimension `action` reads as composite, almost certainly intended as projection
**Category:** assumption
**Pass:** 2
**Description:** AC-05 says "Registry rules are keyed by `(entity_type, structural_path, action)`". Treating `action` as part of the key permits two rules for the same `(entity_type, structural_path)` differing only by action — e.g. one rule mapping `Spec.date_opened` to `date_created` and another dropping it — which is semantically ill-defined (which fires?). The almost-certain intent is `(entity_type, structural_path) → action` — `action` is the looked-up value, not a key dimension.
**Evidence:** AC-05 sentence quoted above; design intent in interview §"Implementation Notes" describes `pub const FIELD_RULES: &[(EntityType, &str, Disposition)] = &[...]` — a 3-tuple where the third element is the disposition, used as the result of a lookup keyed by the first two.
**Recommendation:** No spec rewrite needed. The implementer should read AC-05 as `(entity_type, structural_path) → action` and uniqueness of the first two columns is enforced at registry-load time (or by the const's declaration). Flag in the PR description for the record.

### [LOW] AC-05 "bd-era legacy fields visible in `.orbit/archive/`" is harvest-input, not walk-target
**Category:** assumption
**Pass:** 1
**Description:** AC-05 names "any bd-era legacy fields visible in `.orbit/archive/` that share the same disposition pattern" as seed-registry entries. `canonicalise_all` walks `layout.specs_dir()` / `layout.cards_dir()` only — `.orbit/archive/` is never scanned (no `archive` references in `canonicalise.rs` or `layout.rs`). The spec implicitly assumes the archive is a *research source* for the implementer building the constant, not a runtime walk target. This is correct but easy to misread.
**Evidence:** `orbit-state/crates/core/src/canonicalise.rs` shows zero archive references; `layout.rs:67` and `layout.rs:96` define `specs_dir()` and `cards_dir()` against the live `.orbit/` only.
**Recommendation:** No spec change required. The implementer should read "visible in `.orbit/archive/`" as offline research input — they should `rg` the archive for unknown-field patterns when building `FIELD_RULES` and not extend the walker.

### [LOW] Partial-write failure handling unspecified
**Category:** failure-mode
**Pass:** 2
**Description:** AC-04 says the substrate never silently destroys content. The non-dry-run path needs to (a) write the canonical file, (b) write the sidecar — both atomically per file pair. If the canonical file is rewritten and the sidecar write fails (disk full, permissions), the quarantined content is lost. The spec doesn't say whether the implementer should write sidecar-then-canonical (safer — sidecar exists before any destruction), use a temp-then-rename pattern, or fail the whole entity if either write fails. The existing canonical writer (`atomic.rs`) presumably uses temp-then-rename; the reconcile path needs the same posture for the sidecar.
**Evidence:** No mention of write ordering or transactional semantics in any AC. `orbit-state/crates/core/src/atomic.rs` exists, suggesting the temp-then-rename primitive is already available.
**Recommendation:** No spec change required if the implementer reuses `atomic.rs` for both the canonical-file and the sidecar write and orders sidecar-first. Surface the ordering choice in the PR description so the reviewer can confirm.

---

## Honest Assessment

This plan is ready. The v1 review's five HIGH/MEDIUM findings were each addressed with substantive AC rewrites — not cosmetic edits. AC-03 now points at a referent that exists; AC-02 declared its v1 scope honestly; AC-04 extended the schema-side work (new inner `FIELDS` constants kept in lockstep by unit test) so inner-shape drift can't sneak past; AC-06 defined "same content" precisely; AC-08(a) named the section it edits in `/orb:setup`.

The remaining findings are all LOW and all about reading discipline rather than missing content. The implementer should default to reading (a) on AC-03's exit-code ambiguity, treat AC-05's registry shape as `(entity_type, structural_path) → action`, treat AC-05's archive reference as offline research input, and reuse `atomic.rs` for sidecar writes. None require a spec edit before implement starts.

The biggest residual risk is in execution, not specification: AC-04's new inner `FIELDS` constants must be added to `schema.rs` alongside their lockstep unit tests *before* the reconcile walker can rely on them, and AC-08's four wires (setup §3g, card 0030, choice 0023, card 0032 rewording) are easy to forget at the end of a long implement session. The implement pre-flight should surface AC-08 as a multi-part umbrella and gate close until all four parts are visibly done.
