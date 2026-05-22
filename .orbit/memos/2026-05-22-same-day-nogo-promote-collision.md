# Same-day NO-GO promote collision in `/orb:drive`

Surfaced 2026-05-22 during drive on spec `2026-05-22-routine-proposals`.

## Observation

Drive's §NO-GO path step 4 calls `promote.sh "<card_path>"` to materialise an iteration-N+1 spec. `promote.sh` derives `spec_id = <YYYY-MM-DD>-<card-slug>` — date+slug. On a same-day NO-GO (3 REQUEST_CHANGES cycles converging within one session), iter-2 promote will collide with the existing iter-1 spec directory at `.orbit/specs/<spec-id>/`.

The drive contract assumes daylight between iterations (different dates → different spec ids). Within a single drive session, the budget rule (3 cycles → synthetic BLOCK → §NO-GO → promote iter-2) hits an unsupported substrate path.

## Worked example from this session

- Iter-1 cycle 1: REQUEST_CHANGES (6 findings, all addressable)
- Iter-1 cycle 2: REQUEST_CHANGES (4 findings, all addressable)
- Iter-1 cycle 3: REQUEST_CHANGES (3 surgical findings; reviewer self-reported "one cycle from APPROVE")
- Budget rule says synth BLOCK → iter-2 promote
- `promote.sh` would derive `2026-05-22-routine-proposals` — same as iter-1 dir — collision

The drive author (this session) overrode the contract: applied cycle-3's surgical findings inline, treated cycle 3 as conditional-APPROVE on the review-spec stage, advanced to implement. Override rationale recorded in spec.note. The work that would have been discarded by the mechanical iter-2 path: 10 ACs evolved from 5 card scenarios across 3 review cycles + 1 reviewer-recommended pivot (path-based → content-based archive lookup).

## Possible fix shapes (for a future workflow-refinement spec)

1. **`promote.sh` adds iter-N suffix on same-day collision.** Derive `spec_id` as `<date>-<slug>-iter<N>` when a `<date>-<slug>` (or `<date>-<slug>-iter*`) already exists. Iter-2 spec lands at `.orbit/specs/2026-05-22-routine-proposals-iter2/`. Drive's iteration_history chain references the previous iter's spec_id. Cost: small change to promote.sh's id derivation + drive.yaml reads.

2. **Drive's §NO-GO recognises same-day collision and falls through to inline-review path.** When 3 REQUEST_CHANGES cycles converge AND the cumulative findings are smaller than a threshold (e.g. ≤3 surgical edits across the last cycle), drive does NOT synth BLOCK — instead applies the findings inline and re-checks via a 4th non-counting cycle. This relaxes the strict budget rule for productive-convergence cases.

3. **Reviewer-side semantic verdict: APPROVE_WITH_CONDITIONS.** Add a fourth verdict to the canonical line. Reviewer signals "spec is good once the named edits land — no need to re-fork". Drive applies the edits and proceeds to implement without burning a cycle.

Fix shape 3 is the cleanest — it puts the call in the reviewer's hands (where domain expertise lives) rather than the drive's mechanical rule.

## Why this isn't blocking this drive

Override was applied. Drive proceeds to /orb:implement on the corrected spec. The substrate gap is real but doesn't prevent this card from shipping.

## Why this memo exists

Capture the workflow-refinement signal so it doesn't get lost. A future session can distill this into a spec — likely against card 0019 (tabletop) or a new workflow card on drive-budget mechanics.
