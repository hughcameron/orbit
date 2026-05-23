# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-22-default-merge-after-review
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (deployment, gh CLI, GitHub auth, branch protection) | 2 |
| 3 — Adversarial | not triggered (Pass 2 surfaced no cascading or untestable ACs) | — |

## Findings

### [LOW] AC-05 calls the `<reason>` token list "e.g." while AC-06 treats it as a closed canonical set
**Category:** assumption
**Pass:** 2
**Description:** AC-05 introduces the `<reason>` token list with "e.g. `auto-merge-disabled`, `branch-protection`, `draft-pr`, `auth-failure`, `network-error`" — the "e.g." reads as illustrative, not exhaustive. AC-06 then says the close-comment payload uses "the same canonical token written by ac-05's spec note, so audit-trail entries do not diverge between the close-comment and the spec note" — which only holds if the token set is closed. If the implementing agent reads ac-05 literally and invents a sixth token to cover a real gh exit case (e.g. `permission-denied`, `merge-conflict`), the audit-trail-non-divergence claim in ac-06 still holds (it's the same token in both places), but the "canonical" framing becomes aspirational rather than enforced. Minor — the underlying guarantee (same token both places) is intact either way.
**Evidence:** ac-05: "where `<reason>` is a short canonical token (e.g. `auto-merge-disabled`, `branch-protection`, `draft-pr`, `auth-failure`, `network-error`)". ac-06: "the same canonical token written by ac-05's spec note".
**Recommendation:** Tighten ac-05 to either (a) drop "e.g." and declare the five tokens an exhaustive set ("`<reason>` is one of: `auto-merge-disabled`, `branch-protection`, `draft-pr`, `auth-failure`, `network-error`"), or (b) keep "e.g." and rephrase ac-06 to say "the same `<reason>` token written by ac-05's spec note (canonical-set or implementer-extended)". Option (a) is cleaner and matches the spec's other places where canonicality is asserted.

### [LOW] No AC specifies how `gh pr merge` exit codes / stderr patterns map to the `<reason>` tokens
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-05 names five canonical `<reason>` tokens but doesn't say how the drive picks which token to write for a given non-zero exit. The implementing agent must invent a mapping: does `gh pr merge` return distinct exit codes for "auto-merge disabled" vs "branch protection refuses"? Does stderr contain a stable substring? In practice, `gh pr merge`'s exit codes are mostly `1` with the discrimination in stderr text, and stderr text is not contract-stable across `gh` versions. The implementer will have to either (a) inspect `gh pr merge` source for current stderr patterns, (b) use `gh api` pre-check (e.g. `gh repo view --json mergeCommitAllowed,squashMergeAllowed,autoMergeAllowed`) to pre-classify some failures, or (c) accept that some exits will land in a generic catch-all token (e.g. `unknown`) not in the canonical list.

This isn't blocking — the graceful-degradation contract holds regardless of which token gets written (PR url is in the close-comment, manual action is signalled). But the spec's "canonical token" framing implies a mapping discipline the spec doesn't enforce.
**Evidence:** ac-05 enumerates tokens; no AC enumerates the gh→token decision rule. `gh pr merge` documentation: non-zero exit on any failure, stderr varies by cause.
**Recommendation:** Add a one-line clause to ac-05: "the drive picks the `<reason>` token by stderr-substring match against a documented table in the §Completion prose; unmatched failures use `<reason> = unknown` (which is therefore a sixth canonical token)." OR add a six-token set including `unknown` and require the SKILL.md prose to define the matching table. This is a friction-reduction step for the implementing agent rather than a contract gap.

---

## Honest Assessment

The spec is ready for implement. All seven findings from cycle 1's REQUEST_CHANGES verdict are addressed substantively — `ac_type: doc` propagated to ac-01..ac-08; ac-07's stage-discriminator gap closed by dropping the `drive.yaml.stage` clause and relying solely on `gh pr view`; ac-05's missing referent replaced with an explicit five-step branch and named `<reason>` tokens; ac-06 corrected to enumerate only `queued` and `deferred-<reason>`; ac-04's push-universality clarified; ac-05/ac-06 token-divergence sealed by cross-reference; tabletop Q4 #4's draft-check decision made explicit ("drafts fall through to graceful-degradation").

The remaining two LOW findings (`<reason>` set canonicality framing; gh-exit-to-token mapping discipline) are friction-reducers for the implementing agent, not contract gaps. The graceful-degradation guarantee (commit landed, PR open, spec closed, manual action surfaced) holds at full strength regardless of which token gets written.

Pass 2's content-signal scan caught deployment / GitHub auth / branch protection — the legitimate adversarial surface — and the spec already routes each through ac-05's failure branch and ac-09's observation window. K1 (forked-review-trust) and K2 (auto-merge-edge-cases) kill conditions provide the second-order safety floor.

Recommended next move — proceed to implement. Optionally tighten ac-05's token list to closed-set framing in a follow-up commit if the implementing agent finds the friction warrants it.
