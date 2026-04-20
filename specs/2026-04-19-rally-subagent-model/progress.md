# Implementation Progress — rally subagent-model v1.3

**Spec:** specs/2026-04-19-rally-subagent-model/spec.yaml (v1.3)
**Started:** 2026-04-20
**Completed (first PR):** 2026-04-20
**Completed (follow-up PR):** 2026-04-20
**Scope:** First PR landed the 13 PR-#6-independent ACs. Follow-up PR (this update) lands ac-06, ac-07, ac-14 and decision 0009 now that drive-forked-reviews (PR #6) has merged to main — satisfying the first exit condition.

## Hard Constraints

- [x] #1 Path discipline is trust + post-verify — SKILL.md §4a + honesty callout + rally-coherence-scan.sh asserts no 'tools allow-list' phrase
- [x] #2 Parallel implementation sub-agents run drive-full with nested forked reviews — SKILL.md §7c (recursive context separation callout + decision citations); follow-up PR
- [x] #3 Rally refuses at proposal any card with <3 scenarios — SKILL.md §2a
- [x] #4 rally.yaml written only by lead — SKILL.md §3 + §7c brief forbids it + critical rule
- [x] #5 Parallel sub-agents via Agent(run_in_background: true); lead awaits Agent-tool background-completion notification — SKILL.md §7c + constraint citation inline
- [x] #6 Rally-level autonomy independent of drive autonomy (always full inside rally) — SKILL.md Autonomy Levels
- [x] #7 cards[].worktree sentinels — SKILL.md §3 comments + §11 resolver + §12 validation; ontology extended in specs/2026-04-19-rally/spec.yaml
- [x] #8 Drive-full escalation absorbed by single-strike NO-GO — SKILL.md §9 label-mapping table; follow-up PR
- [x] #9 Three-primitive verification with pre/post snapshot diff — SKILL.md §4b
- [x] #10 Commit interview.md to rally branch before delegation — SKILL.md §7a
- [x] #11 Serial implementation runs drive-full in main checkout against rally branch — SKILL.md §7b, autonomy callout
- [x] #12 rally.yaml lives only on main; lead checks out to main before every rally.yaml write — SKILL.md §3, §7b, §7c, §11; critical rule
- [x] #13 Refinement of v2.0, not replacement — v2.0 ac-06 amended in place, ontology extended inline

## Acceptance Criteria

- [x] ac-01 (code): thin-card guard at proposal — SKILL.md §2a; critical rule updated
- [x] ac-03 (doc): SKILL.md §4a names trust + post-verify; forbidden 'tools allow-list' phrase absent (coherence-scan verified)
- [x] ac-04 (code): three-primitive verification with pre/post snapshot-diff — SKILL.md §4b; re-brief worded as "pre-qualification retry, not a rally-level strike"
- [x] ac-05 (doc): v2.0 ac-06 amended to trust + post-verify + snapshot-diff language; metadata.review_changes HIGH line rewritten (F-09 exit condition)
- [x] ac-06 (code): parallel sub-agents launched with Agent(run_in_background: true, subagent_type: general-purpose); all calls in a single message — SKILL.md §7c brief template + launch-mechanics paragraph (follow-up PR)
- [x] ac-07 (doc): SKILL.md §7c and Integration section cite decisions 0004-drive-verdict-contract, 0005-drive-review-artefact-contract, 0006-drive-cold-re-review, 0007-drive-rerequest-budget by title (follow-up PR)
- [x] ac-08 (code): parallel sub-agent brief forbids rally.yaml — SKILL.md §7c brief template
- [x] ac-09 (code): Agent-return completion surfacing (no polling, no sentinels); N completions → N rally.yaml commits on main — SKILL.md §7c
- [x] ac-10 (doc): Autonomy Levels section distinguishes rally-level vs drive-level autonomy — SKILL.md Autonomy Levels
- [x] ac-11 (code): ontology cards[].worktree with sentinel semantics + path resolution rule — specs/2026-04-19-rally/spec.yaml ontology_schema; SKILL.md §3, §11, §12
- [x] ac-12 (code): rally.yaml validation recognises worktree field + missing-directory warning flow — SKILL.md §12
- [x] ac-13 (code): resumption reads each card's drive.yaml via ac-11 resolver — SKILL.md §11
- [x] ac-14 (code): drive-full escalation absorbed as single-strike park with bracketed label — SKILL.md §9 label-mapping table (5 triggers → 5 tokens → 5 bracketed prefixes; unknown labels fall through to `[unknown]`) (follow-up PR)
- [x] ac-15 (code): plugins/orb/scripts/rally-coherence-scan.sh — 7 mutation tests verified (baseline pass, forbidden-present fail, required-absent fail, verb-no-citation fail, verb-with-negation pass, verb-with-primitive pass, verb-with-trust-marker pass)
- [x] ac-16 (code): commit-before-delegation discipline — SKILL.md §7a (both serial and parallel); critical rule updated
- [x] ac-17 (code): extended coherence scan with citation vocabulary — same script; window=80 tuned against current SKILL.md (passes cleanly)

## Decision Records Created

- decisions/0008-rally-subagent-path-discipline.md — trust + post-verify with three primitives (first PR)
- decisions/0010-rally-thin-card-guard.md — refuse at proposal, unconditional (first PR)
- decisions/0009-rally-parallel-drive-full.md — delegate to drive-full; nested forked Agents are transitive (follow-up PR; cites 0004–0007)

## Verification Evidence (follow-up PR)

```
$ plugins/orb/scripts/rally-coherence-scan.sh
== Keyword scan ==
ok   forbidden-absent : 'tools allow-list'
ok   forbidden-absent : 'drive\'s stage logic inline'
ok   forbidden-absent : 'writes are blocked'
ok   required-present: 'trust + post-verify'
ok   required-present: 'recursive context separation'
ok   required-present: 'drive-full'
ok   required-present: 'nested forked Agents'
== Adjacency scan (window=80) ==

rally-coherence-scan: PASS
```

§7c now concretely describes the launch pattern (Agent tool + run_in_background + single-message dispatch), cites decisions 0004–0007 by title, and replaces the dependency-note callout with a Recursive context separation callout. §9 replaces the "live when PR #6 merges" language with a fixed 5-row label-mapping table. The Integration section replaces its forward-reference sentence with a concrete decision-citation pointer.

## Files Changed This Follow-Up PR

- plugins/orb/skills/rally/SKILL.md (§7c rewritten, §9 label table, Integration citation update)
- decisions/0009-rally-parallel-drive-full.md (new)
- specs/2026-04-19-rally-subagent-model/progress.md (this file)

## Files Changed — First PR (recorded for completeness)

- plugins/orb/skills/rally/SKILL.md (extensive refinement)
- plugins/orb/scripts/rally-coherence-scan.sh (new)
- specs/2026-04-19-rally/spec.yaml (ac-06 amended, ontology extended with cards[].worktree, review_changes line rewritten)
- specs/2026-04-19-rally-subagent-model/spec.yaml (v1.0 → v1.3 across the session — from full review cycle)
- specs/2026-04-19-rally-subagent-model/interview-2026-04-20.md (design continuation)
- specs/2026-04-19-rally-subagent-model/review-spec-2026-04-20.md (v1.1 review)
- specs/2026-04-19-rally-subagent-model/review-spec-2026-04-20-c2.md (v1.2 re-review, APPROVE)
- specs/2026-04-19-rally-subagent-model/progress.md (this file)
- decisions/0008-rally-subagent-path-discipline.md (new)
- decisions/0010-rally-thin-card-guard.md (new)
