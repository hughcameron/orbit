Spec path: .orbit/specs/2026-04-20-implement-session-visibility/spec.yaml
Spec hash: sha256:58686bd0a4e5a9f41e0a04f52e7c4350c19d3d2805546bd498e49f48c31168f7
Started: 2026-04-20
Current AC: none

# Implementation Progress

## Hard Constraints
- [x] Shipped pre-flight behaviour byte-identical — ac-07(d) sha256/diff empty vs post-0009 baseline.
- [x] progress.md schema owned by 0009 — this card consumes it; no new fields introduced.
- [x] progress.md is the single source of truth for task emission — §4d reads via parse-progress.sh.
- [x] Task emission is FLAT — §4d sets no `addBlockedBy` / `addBlocks`; verbatim statement in skill text.
- [x] TaskUpdate same-turn rule in SKILL.md §5 — stated verbatim with "same tool-call turn" and "protocol violation" language.
- [x] metadata.spec_path scoping required — §4d + §5 reconcile both state the rule; hook surface passes spec_path.
- [x] Monitor heuristic (>60s / full-suite) documented in SKILL.md §5.
- [x] Canonical failure-marker filter `grep --line-buffered -E 'FAIL|ERROR|AssertionError|Traceback'` documented in SKILL.md §5.
- [x] First-failure interactive/non-interactive split — §5 defines both paths with canonical option strings + non-interactive marker.
- [x] Resume hook uses parse-progress.sh — next-AC + reconcile-surface blocks both invoke the helper; ## Detours ignored (fixture tested).
- [x] Reconcile match algorithm deterministic; cancel-then-recreate via TaskUpdate + TaskCreate — §5 algorithm spells out the four steps.
- [x] Canonical RESUME_REBUILD_WARNING declared as single-source constant in SKILL.md §5.
- [x] TaskCreate sequencing pinned — §4d explicit "after §4, before first AC pre-AC check"; resume reconcile runs after 0009 pre-AC sequence.
- [x] All changes in existing files + plugins/orb/scripts/parse-progress.sh (new helper); no new skill added.

## Detours

## Acceptance Criteria
- [x] ac-01: §4d emits TaskCreate loop deriving constraints + ACs from parse-progress.sh; metadata.spec_path set; flat; fixture simulation produced 7 tasks with correct subjects and no (gate) suffix.
- [x] ac-02: SKILL.md §5 "TaskUpdate rule — same tool-call turn" section declares the rule; "same tool-call turn" and "protocol violation" both present.
- [x] ac-03: SKILL.md §5 "Resume reconcile" section defines the four-step algorithm (filter → expect → compare → cancel-then-recreate); session-context.sh emits reconcile-pending surface when unchecked work exists.
- [x] ac-04: SKILL.md §5 reconcile step 1 requires filter by metadata.spec_path; rule states "tasks without the tag ... are untouched, never read, never mutated".
- [x] ac-05: SKILL.md §5 "Monitor-for-tests heuristic" present with literal '60 seconds', 'grep --line-buffered', 'FAIL|ERROR|AssertionError|Traceback'; no new spec field (grep of spec-architect/SKILL.md and spec/SKILL.md returned no matches).
- [x] ac-06: SKILL.md §5 "First-failure checkpoint" section defines interactive (AskUserQuestion with two canonical option strings) and non-interactive (stderr marker + exit 2) paths; FIRST_FAILURE_NONINTERACTIVE_MARKER constant declared.
- [x] ac-07: §4d added + four §5 rules added; §1–§4c byte-identical — sha256(extracted sections) matches pre-change baseline; diff empty.
- [x] ac-08: plugins/orb/scripts/parse-progress.sh authored with subcommands (acs, constraints, spec-path, next-unchecked-ac, post-gate-ac, has-unchecked); session-context.sh next-AC and reconcile blocks refactored to invoke the helper; structural ac-08(e) assertion passes (zero awk|sed hits in the refactored blocks); fixture test confirms Detours content ignored, gate flag correct, post-gate AC surfacing works.

## Notes

- Spec hash recorded at pre-flight: 58686bd0a4e5a9f41e0a04f52e7c4350c19d3d2805546bd498e49f48c31168f7.
- Verification runner: the project has no test harness; verification was performed via shell fixtures under /tmp/isv-hook-test and /tmp/isv-fixtures, plus grep-assertions against the shipped files. The spec's verification language (mock assertions, TaskList fixtures) is specified for a future test runner; fixture-level verification substitutes until that harness is added.
- ac-08(e) structural assertion: `awk '/# ac-08 —/,/^fi$/' plugins/orb/scripts/session-context.sh | grep -nE '\bawk\b|\bsed\b'` returns zero hits after the refactor — both the next-AC-surfacing block and the resume reconcile block delegate to parse-progress.sh.
- ac-07(d) byte-identity: `diff` of §1–§4c extracted from pre-change baseline (rally/mission-resilience tip) vs current HEAD is empty; sha256 hashes match (13648ec3cab4b0cd47b9ee30ac3f698d8a47ee72dc043d7622bc3ee32d8c74f3).
- Mission-resilience regression: drift scenario still emits "spec modified since implementation started, re-review recommended"; missing-hash silently skips drift check; gate blocking surfacing still names the post-gate AC.
- Claude Code's Task tools (TaskCreate/TaskUpdate/TaskList) are agent-turn primitives, not bash-callable. The SessionStart hook's role is limited to surfacing the reconcile-pending prompt; the reconcile algorithm runs agent-side per SKILL.md §5 on next turn.
