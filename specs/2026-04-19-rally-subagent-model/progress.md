# Implementation Progress — rally subagent-model v1.3

**Spec:** specs/2026-04-19-rally-subagent-model/spec.yaml (v1.3)
**Started:** 2026-04-20
**Completed (first PR):** 2026-04-20
**Scope:** First PR covers the 13 PR-#6-independent ACs. ac-06, ac-07, ac-14 are deferred to a follow-up after drive-forked-reviews PR #6 merges to main (per the first exit condition).

## Hard Constraints

- [x] #1 Path discipline is trust + post-verify — SKILL.md §4a + honesty callout + rally-coherence-scan.sh asserts no 'tools allow-list' phrase
- [--] #2 Parallel implementation sub-agents run drive-full with nested forked reviews — DEFERRED (PR #6). SKILL.md §7c has a dependency-note callout plus an aspirational brief template gated on PR #6 merge
- [x] #3 Rally refuses at proposal any card with <3 scenarios — SKILL.md §2a
- [x] #4 rally.yaml written only by lead — SKILL.md §3 + §7c brief forbids it + critical rule
- [x] #5 Parallel sub-agents via Agent(run_in_background: true); lead awaits Agent-tool background-completion notification — SKILL.md §7c + constraint citation inline
- [x] #6 Rally-level autonomy independent of drive autonomy (always full inside rally) — SKILL.md Autonomy Levels
- [x] #7 cards[].worktree sentinels — SKILL.md §3 comments + §11 resolver + §12 validation; ontology extended in specs/2026-04-19-rally/spec.yaml
- [--] #8 Drive-full escalation absorbed by single-strike NO-GO — DEFERRED (PR #6). SKILL.md §9 names the five triggers with a forward-reference callout
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
- [--] ac-06 (code): parallel sub-agents launched with Agent(run_in_background: true) — DEFERRED (PR #6). SKILL.md §7c has the brief template with dependency-note callout
- [--] ac-07 (doc): SKILL.md §7 cites drive-forked-reviews decisions by title — DEFERRED (PR #6). SKILL.md Integration section and §7c note reference the spec by path
- [x] ac-08 (code): parallel sub-agent brief forbids rally.yaml — SKILL.md §7c brief template
- [x] ac-09 (code): Agent-return completion surfacing (no polling, no sentinels); N completions → N rally.yaml commits on main — SKILL.md §7c
- [x] ac-10 (doc): Autonomy Levels section distinguishes rally-level vs drive-level autonomy — SKILL.md Autonomy Levels
- [x] ac-11 (code): ontology cards[].worktree with sentinel semantics + path resolution rule — specs/2026-04-19-rally/spec.yaml ontology_schema; SKILL.md §3, §11, §12
- [x] ac-12 (code): rally.yaml validation recognises worktree field + missing-directory warning flow — SKILL.md §12
- [x] ac-13 (code): resumption reads each card's drive.yaml via ac-11 resolver — SKILL.md §11
- [--] ac-14 (code): drive-full escalation absorbed as single-strike park with bracketed label — DEFERRED (PR #6). SKILL.md §9 describes the mechanism; v2.0 ontology parked_constraint field text updated to describe the prefix
- [x] ac-15 (code): plugins/orb/scripts/rally-coherence-scan.sh — 6 mutation tests verified (baseline pass, forbidden-present fail, required-absent fail, verb-no-citation fail, verb-with-negation pass, verb-with-primitive pass, verb-with-trust-marker pass)
- [x] ac-16 (code): commit-before-delegation discipline — SKILL.md §7a (both serial and parallel); critical rule updated
- [x] ac-17 (code): extended coherence scan with citation vocabulary — same script; window=80 tuned against current SKILL.md (passes cleanly)

## Decision Records Created (this PR)

- decisions/0008-rally-subagent-path-discipline.md — trust + post-verify with three primitives
- decisions/0010-rally-thin-card-guard.md — refuse at proposal, unconditional

## Decision Records Deferred

- decisions/0009-rally-parallel-drive-full.md — held for PR-#6-gated follow-up; depends on drive-forked-reviews decisions 0004–0007 existing on main

## Verification Evidence

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

(Note: SKILL.md §2/§4a/§7/§11 contain no enforcement verbs — the rewrite
favoured "verifies", "checks", "rejects", "refuses". The adjacency scan
therefore has nothing to check and trivially passes. If a future edit
introduces enforcement verbs in those sections, the scan will exercise the
adjacency rule against the concrete citation vocabulary.)

Mutation matrix (run against temp copy):
  1. Baseline                                   → PASS (exit 0)
  2. Add forbidden phrase                       → FAIL (exit 1, offending line named)
  3. Remove required phrase                     → FAIL (exit 1, phrase named)
  4. Verb, no citation/trust/negation           → FAIL (exit 1, offending line named)
  5. Verb + negation whitelist token            → PASS (exit 0)
  6. Verb + primitive citation                  → PASS (exit 0)
  7. Verb + trust marker                        → PASS (exit 0)
```

YAML parse check:
```
$ python3 -c "import yaml; yaml.safe_load(open('specs/2026-04-19-rally/spec.yaml'))"
v2.0 YAML OK
$ python3 -c "import yaml; yaml.safe_load(open('specs/2026-04-19-rally-subagent-model/spec.yaml'))"
subagent v1.3 YAML OK
```

## Deferred Follow-Up PR (after PR #6 merges)

Scope:
- ac-06: Flesh out SKILL.md §7c parallel-launch mechanics now that drive-forked-reviews ships. Remove the dependency-note callout.
- ac-07: Replace the forward-reference "when PR #6 merges" language with concrete citations to decisions 0004-drive-verdict-contract through 0007-drive-rerequest-budget by title.
- ac-14: Implement the drive-full escalation-to-parked_constraint label mapping (bracketed prefix from drive's escalation reason label).
- Create decisions/0009-rally-parallel-drive-full.md (references 0004–0007 once they exist on main).

## Files Changed This PR

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
