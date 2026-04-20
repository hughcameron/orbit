# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` (v1.2)
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (honesty AC, cross-branch sequencing, ontology migration) | 1 |
| 3 — Adversarial | no structural concerns in Pass 2 reaching MED+ | — |

## Findings

### [LOW] Unscoped `git status --porcelain` can produce false positives from lead-side activity

**Category:** Verification mechanism / test adequacy
**Pass:** 2
**Description:** Constraint #9 and ac-04 (lines 12, 34–37) specify that the lead runs an unscoped `git status --porcelain` in the main checkout and rejects any entry that is neither under the sub-agent's `<spec_dir>` nor on the fixed allowlist (`orbit/specs/rally.yaml`). This correctly widens the scan scope to catch design sub-agent leaks anywhere in the tree (the F-10/R3 resolution). However, the spec does not acknowledge that the lead itself may cause main-checkout mutations between sub-agent launch and return — e.g. rally.yaml writes under constraint #12, edits the lead performs while awaiting a background completion event, or pre-existing uncommitted state present at session start. An unscoped diff at return time will include those entries and attribute them (falsely) to the sub-agent. The allowlist covers only `orbit/specs/rally.yaml`; it does not cover "entries that already existed before the sub-agent was launched" or "entries the lead wrote while the sub-agent was running".

**Evidence:**
- Constraint #9 (line 12): "unscoped `git status --porcelain` in the main checkout minus a fixed lead-owned allowlist (`orbit/specs/rally.yaml` plus any rally-branch commit markers)".
- ac-04 verification (line 37) simulates only sub-agent-caused leaks; it does not simulate a pre-existing unrelated entry (e.g. a CLAUDE.md edit) present before launch.
- Constraint #12 (line 15) has the lead writing rally.yaml on main between card delegations, which means main-checkout state is intentionally mutable between sub-agent launches.

**Recommendation:** Add a pre-launch snapshot primitive: the lead captures `git status --porcelain` output immediately before launching each design sub-agent and diffs pre-vs-post on return, rejecting only entries new in the post set. Alternatively, narrow the scan to `--porcelain -- orbit/specs/ <lead-owned excludes>` with an explicit note that leaks outside `orbit/specs/` are out of scope for this mechanism (honest-scoped). Either change is a small amendment to constraint #9 and ac-04; the current formulation risks spurious rejections in normal lead-active sessions.

### [LOW] ac-04 conflates re-brief with "drive iteration" for an agent that is not running drive

**Category:** Minor wording / coherence
**Pass:** 1
**Description:** ac-04 (line 36) states: "A first violation triggers a pre-qualification re-brief with an explicit path warning... this re-brief is NOT a drive iteration and does NOT consume single-strike budget." But a design sub-agent is not running `/orb:drive` — per constraint #4 and the 2026-04-19 interview Q1, design sub-agents are briefed with card/interview.md/spec_dir and run an interview-or-design task. "Drive iteration" is the wrong referent here; the intended meaning is presumably "rally-level single-strike budget" (ac-14's NO-GO count).

**Evidence:**
- ac-04 line 36 literal text: "re-brief is NOT a drive iteration".
- ac-14 (lines 89–92) establishes the single-strike policy for drive-full escalation, not for design sub-agent path violations. The relationship between ac-04's re-brief and ac-14's strike is therefore stipulative; the current wording borrows drive's vocabulary for a non-drive flow.

**Recommendation:** Rephrase ac-04 to "this re-brief is a pre-qualification retry, not a rally-level strike, and does not count against any drive-full escalation budget." Purely editorial; does not affect mechanism.

---

## Honest Assessment

The spec is unusually thorough and self-aware. The honesty principle is genuinely operationalised: every mechanism it claims to enforce is cross-referenced against a named Claude Code primitive (Agent tool / run_in_background, git worktree add, git status --porcelain, git commit / git checkout, file-on-disk review-artefact contract) or explicitly marked trust + post-verify — and ac-17 makes that property mechanically auditable with a concrete vocabulary (lines 119–129) and a pragmatic "tune the window against the current SKILL.md" instruction (lines 132–135). The PR #6 dependency is correctly named as a merge-order gate in the first exit condition (line 167) and cited in ac-06, ac-07, and ac-14 rather than assumed-shipped. The explicit sentinel treatment of `cards[].worktree` ("main" vs absolute vs null pre-launch) with validation rules and a path-resolution rule for the lead resolver (ac-11 lines 73–75, ac-13 lines 83–86) closes what would otherwise be the largest failure surface on resumption. The rally.yaml-on-main discipline (constraint #12) is a sharp, testable invariant that maps cleanly to `git reflog` observation. The two LOW findings above are editorial and do not affect the spec's implementability. There is no HIGH or MED structural concern; the v1.2 review-changes block (lines 191–246) demonstrates the spec correctly absorbed all nine prior findings.

---

**Verdict:** APPROVE
