# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-investigation-as-pipeline-step
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | Pass 1 HIGH finding + content signals (hooks, cross-skill pipeline, observation-band audit) | 5 |
| 3 — Adversarial | Pass 2 surfaced contradicted assumption (`adjacent_files` doesn't exist), cascading derivation gaps, and a rollback hole around ac-07 | 3 |

## Findings

### [HIGH] ac-02's primary scope source (`adjacent_files`) does not exist
**Category:** missing-requirement
**Pass:** 1
**Description:** ac-02 routes implement's pre-flight scope from "the spec's `adjacent_files` (or, if absent, the current AC's named files derived from the AC description's file:line citations)." Neither half of the fallback chain rests on an existing substrate. `adjacent_files` appears nowhere in `orbit-state/crates/`, in any other spec.yaml under `.orbit/specs/`, or anywhere in `plugins/orb/`. The AC `description` field is freeform prose with no enforced `file:line` grammar — this spec's own ACs cite paths like `plugins/orb/skills/implement/SKILL.md` without line ranges, in a paragraph the agent would need to parse heuristically.
**Evidence:**
- `grep -rn "adjacent_files" orbit-state/ plugins/` returns zero matches outside this spec's own goal/AC prose.
- The only occurrences are spec.yaml line 28 (goal narrative), line 50 (ac-02 description), line 66 (ac-02 verification).
- The Karpathy-style "named files" path from AC descriptions is not specified — no grammar, no parser, no example.
**Recommendation:** Add a precursor AC (or split a precursor spec) that either (a) extends the orbit-state spec schema with an `adjacent_files` field, populates it on this spec, and documents the field in `.orbit/conventions/`, or (b) narrows ac-02's scope source to something that already exists (e.g. `git diff --name-only` on the spec folder's progress.md, or the spec's `cards[]` → card `references[]` chain). Without one of these, ac-02 ships an orchestration step with no scope to pass.

### [MEDIUM] ac-03 and ac-05 inherit the same scope-derivation gap
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-03 derives tabletop's broad-mode scope from "cluster cards' adjacent code"; ac-05 derives researcher's broad-mode scope from "the topic argument (or the named area passed at invocation)." Cards carry a `references[]` field but no structured `adjacent_code` field. The "topic argument" for `/orb:researcher` has no defined shape in the skill's current SKILL.md. Both ACs ship orchestration prose that calls into derivation logic the substrate doesn't currently surface.
**Evidence:** `orbit card show 0025-codebase-mastery` returns a `references[]` mixing URLs, file paths, and freeform descriptions; no structured adjacency. `plugins/orb/skills/researcher/SKILL.md` line 20 is a single advisory line — invocation contract isn't documented.
**Recommendation:** Either define the derivation rules concretely (which `references[]` entries count as code? what's the parser?) inside ac-01's choice file, or weaken these ACs to "pre-flight invokes `/orb:code-investigate` with explicit agent-typed scope" and defer the auto-derivation to a follow-on spec.

### [MEDIUM] ac-04's git incantation is fragile across review contexts
**Category:** failure-mode
**Pass:** 2
**Description:** ac-04 specifies `git diff --name-only $(git merge-base origin/main HEAD) HEAD` as the scope derivation. This breaks in several real contexts review-pr enters: a freshly-cloned worktree where `origin/main` isn't fetched, a branch off a non-main base, a detached HEAD during a rebase mid-review, or any consumer repo whose default branch isn't named `main`. Failure mode is opaque — `merge-base` returns empty, the diff command exits non-zero, and the pre-flight has nothing to pass.
**Evidence:** Spec text in ac-04 description is the verbatim command; no fallback or error path is named. The skill's existing line-55 advisory says "narrow mode when probing call sites" without prescribing a derivation.
**Recommendation:** Resolve the base branch via the spec's `base_branch` field if one exists, or via `gh pr view --json baseRefName` when a PR exists, with a documented fallback when both fail. Name the error handling explicitly so review-pr doesn't fall through to silent bypass.

### [MEDIUM] AC verification proves prose exists, not that orchestration converts
**Category:** test-gap
**Pass:** 2
**Description:** ac-02..ac-05 verify by `grep <SKILL.md> for orchestrated-step prose` + "manual smoke." The load-bearing value declared at Q2 is *behaviour-change* — the 610/0 conversion is the failure mode being addressed. A grep-passes-but-orchestration-misfires path leaves all four ACs closeable green while the spec's actual goal stays unmoved until the +4w audit (ac-08) fires. There is no in-cycle assertion that the Skill tool call returns successfully or that the called skill received non-empty scope.
**Evidence:** ac-02 verification: "Grep ... for the structural step prose ... Manual smoke." ac-03/04/05 follow the same shape. ac-08 carries the conversion measurement but as a deferred observation band — not a gate on shipping ac-02..ac-05.
**Recommendation:** Add a verification clause to ac-02..ac-05 that checks orchestration *fires* (e.g. an integration test invoking the pre-flight against a fixture spec and asserting the Skill call lands), or add a near-term (1-week) success-criterion AC distinct from the +4w audit so a misfire surfaces before the audit deadline.

### [MEDIUM] ac-07 hook retirement leaves no backstop for non-pipeline edits
**Category:** failure-mode
**Pass:** 2
**Description:** ac-07's recommended lead retires the PreToolUse Edit|Write hook. The 610/0 evidence justifies retirement *for paths the pipeline orchestrates*. But not all Edits route through pipeline skills — ad-hoc fixes, memo authoring outside `/orb:implement`, hook bypass via manual editor, and direct CLI edits all lose the only nag they had. The spec asserts "orchestration is upstream of edits" without naming the share of edits that actually enter through a pipeline-skill door.
**Evidence:** `plugins/orb/hooks/hooks.json` shows the matcher is `Edit|Write` unconditionally — every Edit/Write fires it. The 1725-edits population in the audit isn't decomposed by "entered via a pipeline skill" vs "ad-hoc"; the recommendation to retire treats all 1725 as equivalent.
**Recommendation:** Before retiring, measure (or estimate) what fraction of Edit/Write tool uses originate inside a pipeline-skill invocation. If a non-trivial fraction is ad-hoc, prefer ac-07 path (2) — retain the hook with sharpened text — at least until ac-08's audit settles the question.

### [MEDIUM] ac-08's success measurement compares mixed-population data
**Category:** test-gap
**Pass:** 2
**Description:** ac-08 target (1) reads: "sustained lift above the 47.1% pre-ship baseline measured on brightfield." The audit aggregates across 5 repos but the baseline is from one. A lift in orbit's own ratio could be masked by a drop in another repo, or vice versa — the comparator is apples-to-oranges. The kill condition K1 ("if after ship + 2w the investigation-before-edit ratio hasn't moved measurably") inherits this ambiguity and the spec doesn't say *which* repo's ratio carries the kill judgement.
**Evidence:** ac-08 description names the brightfield baseline; tabletop Q6 (success criteria #1) says "across orbit + at least 2 consumer repos." No per-repo target is given.
**Recommendation:** Either name per-repo baselines (measure each of the 5 repos pre-ship at the same time and store them, then compare same-repo post-ship), or restrict the kill judgement to one repo (orbit itself or brightfield) with the others reported as supporting evidence.

### [LOW] ac-06's "one-line acknowledgement" has no enforcement
**Category:** test-gap
**Pass:** 3
**Description:** ac-06 prevents accidental double-fire by adding a sentence to drive/rally SKILL.md. Prose drift is not blocked — a future edit to drive or rally could add an investigation step at the orchestrator layer without anything alerting. The contract sits in plain English in two files.
**Evidence:** No machine-readable assertion is named (no lint, no test, no audit-conformance finding).
**Recommendation:** Optional — add a workflow-conformance finding-family that checks for the "fires at routed stage" acknowledgement string in drive/rally SKILL.md and warns if absent, or accept this as documentation-only and note it explicitly in ac-06.

### [LOW] Cascade rollback path for kill condition K1 isn't operable
**Category:** assumption
**Pass:** 3
**Description:** Tabletop Q10's K1 pivot says "try a different mechanism from Q5 laterals A/B/C; or conclude the integration shape itself is wrong." But ac-07 has already retired the hook by ship-time (recommended path). If K1 fires at +2w, the pivot to lateral B (marker-gate) needs the hook restored or replaced with a new pre-edit assertion surface. The spec doesn't carry an AC or a kill-condition action for "restore the backstop if retired and K1 fires."
**Evidence:** ac-07's recommended path is retirement; kill-condition pivots in tabletop.md don't reference hook restoration.
**Recommendation:** Either choose ac-07 path (2) — retain as backstop — so the pivot surface stays available, or add a half-sentence to the spec's halt/kill section naming hook restoration as part of the K1 pivot. This costs nothing now and saves a scramble at +2w.

### [LOW] Skill-tool return shape isn't named
**Category:** assumption
**Pass:** 3
**Description:** ac-02 says "The orchestration is a Skill tool call from implement's flow, not standalone prose." The Skill tool executes the called skill inline, but the return contract — what the calling skill receives back, whether the investigation's findings inline into context or only the marker is written — isn't named. The existing `/orb:code-investigate` SKILL.md writes a marker via `code-investigate-mark.sh` and emits prose; how implement's flow consumes that prose to gate AC-traversal is implicit.
**Evidence:** `plugins/orb/skills/code-investigate/SKILL.md` lines 22-33 describe the invocation surface and marker-writing; no calling-skill consumption contract.
**Recommendation:** Spell out in ac-01's choice file (or a tightening to ac-02) what the caller does with the returned content: inline into context for downstream ACs, gate on marker presence, or read the marker explicitly. This is the difference between "fires and forgets" and "informs the edit."

---

## Honest Assessment

The plan is structurally sound at the Q2 level — pipeline orchestration is the right answer to 610/0, and the tabletop work is unusually disciplined. What's not ready is the substrate underneath ac-02..ac-05: the spec assumes `adjacent_files` exists on specs, that cards expose structured adjacency, and that AC descriptions carry a parseable file-citation grammar — none of which is true today. The biggest risk is shipping six SKILL.md edits that orchestrate `/orb:code-investigate` with empty or wrong scope, all four ACs close green on grep-verification, the +4w audit reports unchanged baseline, K1 fires, and the hook has already been retired so the pivot surface is gone. The fix is small: add a precursor AC (or a precursor spec) that lands the scope-derivation substrate first, sharpen ac-04's git command, retain ac-07's hook as backstop until the audit settles, and add a per-repo baseline to ac-08. None of these change the load-bearing pick — they make sure the pick can be measured fairly.
