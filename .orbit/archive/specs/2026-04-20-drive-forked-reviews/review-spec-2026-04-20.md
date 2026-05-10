# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** /home/hugh/github/hughcameron/.orbit/specs/2026-04-20-drive-forked-reviews/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (cross-system boundary, schema change, migration language) + Pass 1 findings | 4 |
| 3 — Adversarial | not triggered | — |

## Findings

### [MEDIUM] Resumption behaviour undefined when drive.yaml predates `review_cycles`

**Category:** failure-mode
**Pass:** 2
**Description:** AC-11 specifies that `review_cycles` is read on resumption, but the spec does not define what drive must do when resuming a `drive.yaml` created *before* this change — the field will be absent. Constraint #7 explicitly forbids any migration scaffolding (no `review_mode` field, no dual code paths). AC-16's migration note says "Drives initialised before this change run inline reviews; drives initialised after this change run forked reviews. Do not mix within a single drive" — but there is no programmatic guard. A user who ignores the note (or whose session-context.sh warning is missed) will resume an in-flight drive under the new SKILL.md with a stale drive.yaml; ambiguous behaviour results.
**Evidence:** constraint #7 (spec lines 11, 12); AC-11 (spec lines 70–73); AC-16 (spec lines 98–100); drive/SKILL.md §11 Resumption (lines 263–288) relies on drive.yaml + file presence and has no version field.
**Recommendation:** Add one of:
  (a) an AC requiring drive to detect absent `review_cycles` on resumption and treat it as "this drive.yaml predates forked reviews — refuse to resume with a clear message pointing to the migration note", OR
  (b) an AC specifying that absent `review_cycles` is initialised to `{review_spec: 0, review_pr: 0}` and the drive proceeds (accepting that an in-flight drive may silently switch review modes mid-stage, which contradicts AC-16's "do not mix" guidance).
Either is fine; picking one removes the ambiguity.

### [MEDIUM] Review file naming collision on same-day REQUEST_CHANGES cycles

**Category:** test-gap
**Pass:** 2
**Description:** The skill templates specify `review-spec-<date>.md` and `review-pr-<date>.md` (date only, no time or cycle counter). With REQUEST_CHANGES bounded at 3 cycles per stage, a single day can easily produce 3 review-spec files at the same path. AC-17 verifies the APPROVE path ("exactly two review files") but no AC addresses what drive does when cycle 2 writes a review file whose path already exists from cycle 1 — overwrite, suffix, or error. This also affects AC-02's "first line matching the regex" parsing if a prior cycle's file is read by accident.
**Evidence:** AC-04 (line 35) and AC-05 (line 40) both reference `<spec_dir>/review-spec-<date>.md` with no cycle discriminator; review-spec/SKILL.md line 123 and review-pr/SKILL.md line 113 use the same `<date>` convention; AC-09 (lines 62–63) tracks cycle counts but does not specify file path behaviour per cycle.
**Recommendation:** Add an AC specifying the filename contract for multi-cycle reviews. Two options:
  (a) Drive overwrites the review file each cycle and retains the latest only (history lives in drive.yaml's review_cycles counter + git); simplest.
  (b) Drive appends a cycle suffix, e.g. `review-spec-<date>-cycle<N>.md`; preserves history on disk.
Option (a) matches the "minimal migration surface" value; option (b) is friendlier to post-hoc investigation.

### [LOW] Distinction between fork-retry counter and REQUEST_CHANGES counter not tested

**Category:** test-gap
**Pass:** 2
**Description:** AC-07 (fork retry on no-verdict) and AC-10 (REQUEST_CHANGES cycle budget) describe two independent counters. A failure mode — forked agent returns REQUEST_CHANGES on cycle 1, then on cycle 2 writes an unparseable file — is not explicitly covered. Does the malformed cycle-2 verdict trigger the AC-07 retry (one more fork) before incrementing `review_cycles.review_spec`, or does it count as a REQUEST_CHANGES cycle? The spec implies the former (AC-02 zero-matches → "no verdict" → AC-07 retry), but no AC exercises the interaction.
**Evidence:** AC-07 verification (lines 50–51) simulates only no-file failures; AC-10 verification (lines 67–68) simulates only clean REQUEST_CHANGES returns; no combined case.
**Recommendation:** Add a verification sub-case to AC-07 or AC-10: "A fork that writes an unparseable file triggers the AC-07 retry path, not the AC-10 cycle increment. Only parseable REQUEST_CHANGES verdicts count against the cycle budget." One sentence in either AC's description would close this.

### [LOW] AC-03 conflates verdict-token case with full-line case sensitivity

**Category:** test-gap
**Pass:** 1
**Description:** AC-02 defines the regex `^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$` and AC-02's prose says "case-sensitive on the verdict token." AC-03's test cases include `**verdict:** APPROVE` (lowercase "verdict") — this tests case-sensitivity on the *prefix*, not the *token*. The spec is likely intending full-line case-sensitivity per the regex, but the prose emphasises only the token. Minor prose clarity issue.
**Evidence:** AC-02 (line 24) says "case-sensitive on the verdict token"; AC-03's `**verdict:** APPROVE` case (line 30) tests prefix case, not token case.
**Recommendation:** Change AC-02 prose to "The match is case-sensitive on the full canonical line (both the `**Verdict:**` prefix and the verdict token)." No behaviour change — just aligns prose with the regex and AC-03's intent.

### [LOW] "Code" ACs describe skill-prose instructions, not compiled code

**Category:** assumption
**Pass:** 2
**Description:** ACs tagged `ac_type: code` (ac-02, ac-03, ac-04, ac-05, ac-06, ac-07, ac-08, ac-09, ac-10, ac-11, ac-17) describe runtime behaviour that drive must exhibit. But drive is a skill — text instructions interpreted by an agent at runtime, not compiled code. The "verdict parser" of AC-02 is an instruction in drive/SKILL.md telling the driving agent what regex to run against the file. "Unit-style test" verifications (AC-02, AC-03) will in practice be manual or scripted runs of drive. This isn't a defect but affects `test_prefix: dfr` interpretation in /orb:audit — tests will likely be integration-style, not unit-style.
**Evidence:** plugins/orb/skills/drive/SKILL.md is a markdown instruction file, not code; spec's ac_type values of `code` (spec lines 23, 28, 34, 39, 44, 48, 54, 60, 65, 70, 103) imply compiled/interpreted code; verification methods mention "unit-style test" (AC-02).
**Recommendation:** Either (a) add an exit_condition / constraint noting "AC verifications are integration tests against a driving agent, not unit tests of a parser library", or (b) reclassify pure-prose ACs (e.g., ac-04, ac-05 brief inspection) to `ac_type: config` or a new type. Not blocking — just surfaces an ambiguity for implementers writing `dfr_ac02_*` tests.

### [LOW] Top-level review fan-out not explicitly bounded

**Category:** missing-requirement
**Pass:** 2
**Description:** Each top-level iteration can burn up to 3 review-spec cycles + 3 review-pr cycles = 6 forked reviews. At the top-level budget of 3 iterations, the worst case is 18 forked review invocations per card drive. This is disclosed implicitly via constraint #9 ("top-level iteration budget (3 NO-GO events) is unchanged") but not surfaced as an explicit cost consideration. Not blocking — the synthetic BLOCK in AC-10 caps it — but worth noting the agent-invocation cost.
**Evidence:** constraint #9 (line 13); AC-10 (lines 66–68) 3-cycle cap per stage; no AC discusses aggregate fork count.
**Recommendation:** Optional: add a note to the disposition / rationale section that each top-level iteration costs up to 6 forked reviews, so agent-invocation economics inform whether 3 × 3 is the right shape. No AC change needed.

---

## Honest Assessment

This spec is structurally strong: 18 ACs map 1:1 onto the card's 10 scenarios plus explicit coverage of the verdict contract, fork invocation, re-review coldness, and budget. The goal and constraints are internally consistent, and the downstream-impacts metadata is unusually honest about what this spec *frees* the rally refinement spec from. The biggest risk is the resumption gap: constraint #7 forbids migration scaffolding, AC-16 documents an operator-expected migration procedure, and AC-11 requires counter persistence — but no AC defines the behaviour when a pre-change drive.yaml (no `review_cycles` field) is resumed under post-change SKILL.md. If an operator ignores the migration note, drive silently switches modes mid-stage, which undermines the "do not mix" discipline AC-16 depends on. The secondary risk is review-file naming on same-day REQUEST_CHANGES cycles — current naming (`review-spec-<date>.md`) is deterministic to the day, and the spec never defines what happens on cycle 2 writing to the same path as cycle 1. Both findings are tractable with one AC each; hence REQUEST_CHANGES rather than BLOCK. The remaining findings are LOW-severity prose/test-gap issues worth tightening but not blocking implementation.
