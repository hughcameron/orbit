# Spec Review

**Date:** 2026-04-19
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-19-rally/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

```
| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 7 |
| 2 — Assumption & failure | MEDIUM+ findings and content signals (cross-system boundaries, sub-agent infrastructure, session resumption) | 5 |
| 3 — Adversarial | structural concerns from Pass 2 (PreToolUse hook feasibility, cascade risk from rally.yaml as single source of truth) | 3 |
```

## Findings

### [HIGH] PreToolUse hook enforcement is specified as a constraint but has no implementation path or fallback
**Category:** missing-requirement
**Pass:** 1
**Description:** The spec declares as a constraint (line 6) that "Sub-agents write directly to assigned .orbit/specs/ paths, constrained by PreToolUse hooks" and AC-06 requires verifying "a PreToolUse hook or tools allow-list restricts writes to the card's spec directory." However, orbit currently has zero PreToolUse hooks (only a SessionStart hook exists in `plugins/orb/scripts/session-context.sh`). The interview's Open Questions section explicitly flags this as unresolved: "does orbit need to ship a hook script, or can the sub-agent brief include path constraints that Claude Code enforces natively via the tools: allow-list?" The spec treats a hook as a given rather than something that must be built and tested as part of this deliverable.
**Evidence:** Glob search for `plugins/orb/**/*.sh` returns only `session-context.sh`. Grep for `PreToolUse` across the repo shows zero existing hook implementations. The interview's Open Questions list this as unresolved.
**Recommendation:** Either (a) add an AC that specifies implementing the PreToolUse hook as a deliverable with its own verification, or (b) add a constraint explicitly choosing the `tools:` allow-list approach instead. The current state leaves AC-06 untestable without first resolving this dependency.

---

### [HIGH] AC-05 parallel design fan-out assumes sub-agents can run the design interview without the interactive design skill
**Category:** assumption
**Pass:** 2
**Description:** AC-05 says design sub-agents are "briefed with... the design skill's interview conventions." The design skill (`/orb:design`) is fundamentally interactive: it uses `AskUserQuestion` for each of 4-6 questions (step 4), loads evidence bases, performs keyword scans for orphaned specs, and presents a progress summary. A background sub-agent cannot use `AskUserQuestion` to interview the author. The interview's Open Questions explicitly flag this: "How does the lead agent brief sub-agents with design skill conventions without running /orb:design as a sub-skill?" The spec does not resolve this. If design sub-agents self-answer (full autonomy), drive's full-mode rules require >=3 card scenarios -- but the spec does not reference this prerequisite.
**Evidence:** The design SKILL.md step 4 says "Present the question using AskUserQuestion." Drive's full-mode rules (drive SKILL.md section 3) require >=3 scenarios for self-answering. The interview's Open Questions list this as unresolved. AC-05 treats it as a solved problem.
**Recommendation:** Add a constraint or AC that specifies how design sub-agents produce interview.md without interactive Q&A. Options: (a) rally only works in full autonomy for the design stage and requires >=3 scenarios per card, or (b) designs are sequential/interactive, not parallel (which changes the throughput model), or (c) sub-agents receive enough card context to self-answer and are explicitly briefed with drive's full-mode self-answer protocol. The spec must pick one.

---

### [HIGH] Open Questions from interview not resolved in spec
**Category:** missing-requirement
**Pass:** 1
**Description:** The interview surfaces three Open Questions that represent genuine design gaps, not minor details. The spec proceeds as if these are resolved, but they are not: (1) PreToolUse hook implementation path, (2) session-context.sh detection granularity, (3) how sub-agents get design skill conventions without running the skill. ACs 05, 06, and 16 directly depend on these unresolved questions.
**Evidence:** Interview section "Open Questions" lists three items. Spec ACs reference the outcomes of these decisions without recording which option was chosen.
**Recommendation:** Resolve each Open Question explicitly in the spec's constraints section, or add decision records for each and reference them from the spec. Unresolved design questions in the interview should not silently become assumed-resolved ACs.

---

### [MEDIUM] AC-06 verification method is observational, not functional
**Category:** test-gap
**Pass:** 1
**Description:** AC-06's verification says "Inspect the sub-agent launch. Verify a PreToolUse hook or tools allow-list restricts writes to the card's spec directory." This is an inspection of the mechanism, not a test that the mechanism works. A functional test would attempt a write outside the allowed directory and verify it is blocked.
**Evidence:** The verification method says "inspect" and "verify... restricts writes" but does not describe triggering the restriction and observing the block.
**Recommendation:** Change verification to: "Launch a design sub-agent. Attempt a write outside its assigned spec directory. Verify the write is blocked and the sub-agent reports the restriction."

---

### [MEDIUM] rally.yaml as singleton at .orbit/specs/rally.yaml prevents concurrent rallies and creates implicit constraint
**Category:** assumption
**Pass:** 2
**Description:** The spec places rally.yaml at a fixed path (`.orbit/specs/rally.yaml`). This means only one rally can exist at a time. This is not stated as a constraint -- it is an implicit consequence of the path choice. If a second rally is attempted while one is active, behavior is undefined. Additionally, when a rally completes, the rally.yaml presumably remains. AC-15 (session resumption) reads rally.yaml and resumes -- what happens when the rally is already complete? The ontology schema includes a `complete` phase, but AC-15 does not describe this case.
**Evidence:** AC-04 says "rally.yaml is created in .orbit/specs/ after approval." AC-15 says "when /orb:rally is invoked and .orbit/specs/rally.yaml exists, the skill reads it and resumes." No AC addresses the case where rally.yaml exists but the rally is already complete.
**Recommendation:** (a) Add an explicit constraint: "one rally at a time." (b) Add handling for the completed-rally case in AC-15: detect `phase: complete` and offer to start a new rally (archiving or removing the old rally.yaml). (c) Consider whether rally.yaml should be archived after completion (e.g. moved into the spec directory of the first card, or suffixed with the rally date).

---

### [MEDIUM] Card scenario "Disjointness check rejects unsafe rallies" contradicts spec's two-stage disjointness model
**Category:** constraint-conflict
**Pass:** 1
**Description:** Card scenario 9 says the pre-flight check "rejects the rally and explains which cards conflict." The spec's AC-03 (lightweight pre-flight) merely "flags obvious conflicts" -- it does not reject. The spec's constraint says "Disjointness check runs twice: lightweight heuristic at invocation, definitive symbol/file scan at consolidated design review." The pre-flight is a flag, not a gate. The card scenario implies rejection at pre-flight; the spec implies the author proceeds despite flags and the definitive check happens post-design. The spec is more nuanced and likely correct, but it contradicts the card.
**Evidence:** Card scenario 9: "the agent rejects the rally." AC-03: "flag obvious conflicts." The interview Q2 answer says "flags obvious conflicts early" (not rejects).
**Recommendation:** Update the card scenario to match the spec's two-stage model (pre-flight flags, post-design gates), or add an AC for the case where pre-flight overlap is so severe that rejection is warranted before design begins.

---

### [MEDIUM] No AC addresses cleanup or lifecycle of rally.yaml after rally completion
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-18 describes a completion summary, but no AC addresses what happens to rally.yaml after the rally ends. It persists at `.orbit/specs/rally.yaml`, which means the next session start (AC-16) will surface a stale rally, and the next `/orb:rally` invocation (AC-15) will attempt to resume a completed rally. The exit conditions say "All cards reach complete or parked" and "completion summary has been presented" but do not specify rally.yaml cleanup.
**Evidence:** Exit conditions list three items, none involving rally.yaml archival. AC-16 will fire on any existing rally.yaml. AC-15 will attempt resumption on any existing rally.yaml.
**Recommendation:** Add an AC or exit condition that specifies what happens to rally.yaml after completion. Options: archive it, delete it, or set phase to `complete` and ensure AC-15 and AC-16 handle the completed state gracefully.

---

### [MEDIUM] rally.yaml does not track per-card stage within implementation
**Category:** missing-requirement
**Pass:** 2
**Description:** The ontology schema defines per-card `status` values including `speccing`, `implementing`, and `reviewing`, but there is no field tracking which sub-stage of drive a card is at (design vs spec vs review-spec vs implement vs review-pr). Rally delegates to drive, and drive tracks its own state in `drive.yaml`. But if rally needs to resume (AC-15), it needs to know not just that a card is "implementing" but where within the drive pipeline it stopped. The spec says rally.yaml "is the single source of orchestration state" but the per-card granularity may be insufficient for resumption.
**Evidence:** Ontology schema `cards[].status` has `proposed | designing | designed | speccing | implementing | reviewing | complete | parked`. Drive's internal stages are `design | spec | review-spec | implement | review`. Rally would need to read drive.yaml for each card to determine exact position.
**Recommendation:** Clarify the relationship between rally.yaml's per-card status and drive.yaml's per-card state. Either (a) rally.yaml is the coordination layer and drive.yaml per card is the execution layer (document this explicitly), or (b) rally.yaml tracks drive sub-stages per card (add the fields). Option (a) is simpler and consistent with the decision to keep rally and drive separate.

---

### [MEDIUM] Stacked PRs (AC-17) assume linear branch history but NO-GO parking creates gaps
**Category:** failure-mode
**Pass:** 2
**Description:** AC-17 says "each card's PR targets the previous card's branch." AC-13 says a NO-GO parks a card and the rally continues. If card 2 of 3 is parked, card 3's PR should target card 1's branch, not card 2's (which was never implemented). The spec does not describe this gap-handling. The verification for AC-17 tests the happy path (3 cards, 3 PRs) but not the parked-card case.
**Evidence:** AC-13 verification: "Trigger a NO-GO on one card in a 3-card rally. Verify remaining cards continue." AC-17 verification: "Complete a 3-card serial rally. Verify 3 PRs exist where PR 2 targets PR 1's branch and PR 3 targets PR 2's branch." No verification covers the intersection of these two behaviors.
**Recommendation:** Add a verification case for AC-17 (or a new AC) that tests: "In a 3-card serial rally where card 2 is parked, verify card 3's PR targets card 1's branch, not card 2's."

---

### [MEDIUM] Parallel implementation path is under-specified
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-11 says "implementation proceeds in parallel" when no shared symbols are found, and AC-12 says cards are implemented "concurrently (parallel)." But the spec provides no detail on parallel implementation mechanics: Are parallel implementations also sub-agents? Do they get the same PreToolUse path constraints? How does rally.yaml track N concurrent implementations? How are N concurrent PRs created and reviewed? AC-17 only describes stacked PRs for serial implementation. The card scenario 8 mentions "parallel rallies use batched diff review" but no AC covers batched diff review.
**Evidence:** AC-11 says "implementation proceeds in parallel." AC-12 mentions "concurrently (parallel)." AC-17 only covers stacked PRs (serial). Card scenario 8 mentions "batched diff review" for parallel rallies. No AC covers parallel implementation orchestration or batched diff review.
**Recommendation:** Either (a) add ACs covering parallel implementation (sub-agent launch, path constraints, rally.yaml tracking, batched diff review), or (b) constrain v1 to serial implementation only and defer parallel implementation. Given the spec already has 18 ACs and parallel implementation adds significant complexity, option (b) may be prudent for v1.

---

### [LOW] AC-01 verification does not define "relevant" or "unrelated"
**Category:** test-gap
**Pass:** 1
**Description:** AC-01's verification says "Verify the proposal lists relevant cards with rationale and omits unrelated cards." In a test with 5+ cards, what counts as "relevant" to the goal string is subjective. This is an LLM judgment call, and per the engineering principles ("LLMs for parsing, programmatic checks for validation"), LLM relevance scoring should not be the sole gate.
**Evidence:** Engineering principle 3: "LLMs are unreliable for confirmation and validation tasks." AC-01 verification relies entirely on LLM judgment of relevance.
**Recommendation:** Acknowledge this is necessarily LLM-driven (card selection is a parsing/reasoning task, not validation) but add a deterministic component: verify the proposal includes rationale per card and verify the author gate (AC-02) provides the human check on relevance.

---

### [LOW] The ontology schema lacks a `completed` timestamp field
**Category:** missing-requirement
**Pass:** 1
**Description:** The schema has `started` but no `completed` timestamp. AC-18 (completion summary) would benefit from a duration calculation.
**Evidence:** Ontology schema lists `started` as the only timestamp. No `completed` field exists.
**Recommendation:** Add a `completed` timestamp field to the ontology schema.

---

### [LOW] Session-context detection (AC-16) may conflict with drive detection
**Category:** failure-mode
**Pass:** 3
**Description:** session-context.sh currently detects `drive.yaml` and surfaces drive state. AC-16 adds rally.yaml detection. During a rally, both rally.yaml and per-card drive.yaml files will exist. The session hook needs to present rally state as the primary context, with per-card drive state subordinated, not two independent status lines.
**Evidence:** Current session-context.sh (lines 36-68) already outputs drive state. AC-16 adds rally state output. Both will fire simultaneously during a rally.
**Recommendation:** Specify in AC-16 that when rally.yaml is active, the session hook presents rally state as primary and suppresses or subordinates individual drive state displays.

---

### [LOW] Cascade: if rally.yaml becomes corrupted or is manually edited incorrectly, all resumption and session-context features fail
**Category:** failure-mode
**Pass:** 3
**Description:** rally.yaml is the single source of orchestration state (constraint 4). ACs 14, 15, and 16 all depend on it being well-formed. If rally.yaml is manually edited (which is natural since it is a YAML file in the working tree), or if a write is interrupted mid-phase-transition, all rally coordination fails. There is no validation step on rally.yaml read.
**Evidence:** Constraint: "rally.yaml in .orbit/specs/rally.yaml is the single source of orchestration state." No AC specifies validation or error handling when rally.yaml is malformed.
**Recommendation:** Add a constraint or AC requiring rally.yaml validation on read, with a clear error message if the file is malformed (missing required fields, invalid phase value, etc.).

---

### [LOW] Rollback plan is absent
**Category:** missing-requirement
**Pass:** 3
**Description:** If rally is shipped and proves problematic, there is no documented rollback path. Rally creates rally.yaml, modifies card files, produces stacked PRs, and updates session-context.sh. A rollback would need to address all of these artefacts. Since rally is a new skill (not modifying drive), rollback is straightforward in principle (remove the skill), but the artefacts it creates (rally.yaml, stacked branches) would need cleanup guidance.
**Evidence:** No exit condition or constraint addresses rollback.
**Recommendation:** Add a brief note in constraints or metadata about rollback: "Rally is additive. Rollback: remove the skill file. Clean up any active rally.yaml and stacked branches manually."

---

## Honest Assessment

This spec is thorough and well-grounded in the discovery research. The 18 ACs cover the core rally lifecycle comprehensively, and the ontology schema is well-designed. The biggest risk is the spec treating three genuinely unresolved design questions (from the interview's Open Questions) as decided: the PreToolUse hook implementation path, how sub-agents produce design interviews without interactive Q&A, and session-context.sh integration. These are not edge cases -- they sit on the critical path of AC-05 (parallel design fan-out) and AC-06 (path enforcement), which are the central value proposition of rally. The parallel implementation path (AC-11/12 vs AC-17) also has a gap between serial and parallel modes that needs either more ACs or a scope reduction to serial-only for v1. None of these issues are blocking in the sense that the design is wrong -- the design is sound. They are blocking in the sense that the spec has holes where implementation will require decisions that should have been made at spec time.
