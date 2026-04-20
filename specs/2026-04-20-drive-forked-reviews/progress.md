# Implementation Progress

**Spec:** specs/2026-04-20-drive-forked-reviews/spec.yaml
**Started:** 2026-04-20
**Completed:** 2026-04-20 (implementation; ac-16/ac-17/ac-18 deferred as designed)

## Hard Constraints

- [x] Reviews launched as forked Agents via the Agent tool (subagent_type: general-purpose) — drive no longer reads review-spec/SKILL.md or review-pr/SKILL.md inline at runtime — drive/SKILL.md §5.3, §7.3, Critical Rules
- [x] The verdict contract is a single canonical markdown line — `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK` — matched by a strict regex — drive/SKILL.md §5.4; review-spec + review-pr SKILL.md "Verdict line contract" subsections
- [x] File-on-disk is the only authoritative source for the verdict; drive does not consult the forked agent's chat response — drive/SKILL.md §5.3 ("Drive does not parse the chat response"), §5.4, Critical Rules
- [x] Missing/unparseable verdict → retry once with a fresh fork → escalate on second failure — drive/SKILL.md §5.4.1
- [x] Re-reviews after REQUEST_CHANGES are fully cold — drive/SKILL.md §5.3 brief prohibitions, §5.5 ("functionally identical to the first"), Critical Rules
- [x] REQUEST_CHANGES bounded at 3 cycles per stage; 4th would-be cycle → synthetic BLOCK — drive/SKILL.md §5a
- [x] Per-stage REQUEST_CHANGES counter persisted in drive.yaml across session death — drive/SKILL.md §2 initialisation, §5.5 increment, §11 resumption
- [x] No migration scaffolding — in-flight drives must be finished or parked before upgrade — drive/SKILL.md §2 refusal, §11 step 1, Critical Rules
- [x] drive.yaml + review files are the only state crossing the fork boundary — drive/SKILL.md §5.3 brief constraints
- [x] Top-level 3 NO-GO budget unchanged; synthetic BLOCK consumes iterations normally — drive/SKILL.md §5a ("consumes a top-level iteration the same way a real BLOCK does")

## Acceptance Criteria

- [x] ac-01: Both review skills declare canonical verdict line + brief-overrides-default rule — review-spec/SKILL.md and review-pr/SKILL.md
- [x] ac-02: Verdict parser uses strict regex — drive/SKILL.md §5.4 (regex literal present)
- [x] ac-03: Parser rejects fuzzy/lowercase/non-bold variants — drive/SKILL.md §5.4 (explicit counter-examples)
- [x] ac-04: Drive §5 launches Agent(subagent_type=general-purpose) with cycle-specific path — drive/SKILL.md §5.3
- [x] ac-05: Drive §7 launches Agent(subagent_type=general-purpose) with cycle-specific path — drive/SKILL.md §7.3
- [x] ac-06: Verdict read only from file, never from chat — drive/SKILL.md §5.3, §5.4
- [x] ac-07: Retry-once on missing verdict; idempotent resumption — drive/SKILL.md §5.4.1, §5.2
- [x] ac-08: Re-review briefs functionally identical to first-review briefs — drive/SKILL.md §5.3 "must NOT include" list, §5.5
- [x] ac-09: review_cycles counter increments on REQUEST_CHANGES; resets on new iteration — drive/SKILL.md §5.5, §8 step 3
- [x] ac-10: 4th would-be cycle triggers synthetic BLOCK with exact constraint string — drive/SKILL.md §5a (constraint string byte-identical to spec.yaml)
- [x] ac-11: Counter persistence across session death; resumption respects remaining budget — drive/SKILL.md §11 step 4 (synthesise BLOCK on resumption if counter==3)
- [x] ac-12: Drive §5 rewritten — no "inline" or "read SKILL.md" language — verified by grep
- [x] ac-13: Drive §7 rewritten — no "inline" or "read SKILL.md" language — verified by grep
- [x] ac-14: Critical Rules — "Both reviews run inline" removed; fork-and-file-verdict rule added
- [x] ac-15: REQUEST_CHANGES budget section with byte-identical constraint string — drive/SKILL.md §5a; grep-verified against spec.yaml
- [ ] ac-16: PR description includes migration note — **deferred to PR creation**
- [ ] ac-17: End-to-end drive produces 2 review files + correct terminal state — **deferred; exercised on next real drive after merge**
- [ ] ac-18: Post-ship verification against rally refinement spec — **deferred; not a merge gate per spec's exit_conditions split**
- [x] ac-19: `-v2`/`-v3` suffix for cycles 2–3; date captured at cycle 1 via review_cycle_dates — drive/SKILL.md §5.1, §7.1
- [x] ac-20: Absent review_cycles triggers refusal — drive/SKILL.md §2 ("Refusal on pre-change drive.yaml"), §11 step 1
- [x] ac-21: Fresh drives initialise review_cycles and review_cycle_dates — drive/SKILL.md §2 initial drive.yaml template
- [x] ac-22: Drive §11 Resumption documents review_cycles handling — drive/SKILL.md §11 steps 1, 4 with cross-refs to §2, §5a, §5.2/§7.2
- [x] ac-23: session-context.sh compatibility verified — smoke test against both new-shape and pre-change-shape drive.yaml; no edit required

## Deferred ACs (by design)

- **ac-16** (PR description migration note) — added at PR creation time; the text is fixed by the spec
- **ac-17** (end-to-end drive exercise) — executed on the first real drive after this change ships; cannot be fully simulated pre-merge without forking a real Agent, which this session should not do
- **ac-18** (rally refinement downstream) — explicitly post-ship per exit_conditions; verified when the rally refinement spec lands

## Verification Notes

- Constraint string byte-identity (ac-15): verified via `Grep "review converged on REQUEST_CHANGES after 3 iterations; findings have not been addressable within budget"` — 4 matches (spec, interview, review-spec-v2, drive/SKILL.md)
- Inline-review residue (ac-12, ac-13, ac-14): verified via grep for `run inline|inline at|follow (its|their) instructions (inline|within this session)|read (the|its) SKILL\.md` in drive/SKILL.md — sole match is the Critical Rules negation ("Drive never reads their SKILL.md files inline at runtime")
- session-context.sh compatibility (ac-23): verified by running the hook against two synthetic drive.yaml files (with and without review_cycles); both parse cleanly, emit the expected active-drive line, and exit without error
