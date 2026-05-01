# Implementation Progress

Spec path: orbit/specs/2026-05-01-bead-native-cold-fork-reviews/spec.yaml
Spec hash: sha256:c9eb586f74852667bc1d30e1edfd34db540fdbd39174d441516831b14d5241e8
Started: 2026-05-01
Current AC: none

## Hard Constraints
- [x] Cold-fork architecture preserved: forked Agents, no shared conversation history; brief carries only bead-id, output path, verdict-line contract
- [x] Verdict-line contract unchanged: canonical `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`; drive parser not modified
- [x] Verdict-file path contract: `orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md` for both forked and inline modes
- [x] Bead acceptance convention unchanged at the AC-line level; only schema-level extension is card scenario `gate: true` propagating via promote.sh
- [x] Both reviewers use parse-acceptance.sh; no AC interpretation drift between implement and review
- [x] Hard cutover: no review_mode flag, no dual code paths, no auto-detect
- [x] No spec.yaml shim: review-spec/review-pr take a bead-id only
- [x] Cold-fork purity bounded by acceptance_criteria snapshotting; cycle-history `[x]` leak documented in MADR 0013
- [x] Decision-numbering: 0011-design-intent-not-means renamed to 0012; new MADR is 0013

## Detours

## Acceptance Criteria
- [x] ac-13 (gate-by-ordering): Resolved 0011 numbering collision (rename 0011-design-intent-not-means.md → 0012; CHANGELOG.md cross-ref updated)
- [x] ac-11: Gate semantics propagate end-to-end (card SKILL.md doc; promote.sh emits [gate]; card 0016 scenario 2 annotated; test fixture passes)
- [x] ac-01: Drive snapshot-write logic removed pipeline-wide (sections 1.1/3.1, Worked example, REQUEST_CHANGES paths, Resumption table cross-ref, intro paragraph); §1.x and §3.x renumbered consistently; all six greps return zero
- [x] ac-02: Drive Stage 1 brief example carries only bead-id, output path, verdict-line contract; uses `<bead-id>` placeholder
- [x] ac-03: Drive Stage 3 brief example carries bead-id, diff reference, output path, verdict-line contract; uses `<bead-id>` placeholder
- [x] ac-04: review-spec SKILL.md §1 rewritten to "Gather the Bead"; bead-id argument; bd show + parse-acceptance.sh; no spec.yaml or interview_ref refs
- [x] ac-05: review-spec SKILL.md §2 Pass 1 step 5 rewritten — gate detection via is_gate=1, target text is AC description; three deterministic rules preserved byte-for-byte
- [x] ac-06: review-pr SKILL.md §2/§3 rewritten for bead substrate; progress.md / ac_type / test_prefix removed; bare ac<NN> test names; reviewer-judged exemption note
- [x] ac-07: Drive Completion section drops "bead snapshots" and "snapshot path" references
- [x] ac-12: Inline-mode default output paths in both review skills updated to `orbit/reviews/<bead-id>/...`
- [x] ac-08: Test fixture for gate-AC verification rule (test-gate-ac-verification.sh) — exit 0; ac-01 PASS, ac-02 FAIL on placeholder+length
- [x] ac-09: MADR 0013 written; decision 0002 status updated to "superseded by 0013 (review-pr scope only)"
- [x] ac-10: Card 0016 specs array updated, maturity flipped to active

## Implementation summary

All 13 ACs satisfied. Tests:
- `plugins/orb/scripts/tests/test-gate-ac-verification.sh` — passes (ac-08)
- `plugins/orb/scripts/tests/test-promote-gate-propagation.sh` — passes (ac-11)
