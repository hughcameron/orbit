# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` (v1.1)
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | content signals (cross-skill boundaries, shared config, path discipline, deployment ordering) plus MEDIUM findings in Pass 1 | 5 |
| 3 — Adversarial | not triggered — Pass 2 findings are addressable without plan rework | — |

## Findings

### [HIGH] Spec presupposes a dependency that is not merged
**Category:** missing-requirement
**Pass:** 1
**Description:** The spec claims the drive-forked-reviews work has shipped (`metadata.review_changes` F-01: "Resolved by drive-forked-reviews PR #6 shipping"; ac-07 requires drive-full's review-spec and review-pr to "spawn their own nested forked Agents"). On `main`, however, PR #6 is still open and the current drive SKILL.md (`plugins/orb/skills/drive/SKILL.md` lines 108–110, 146–148, and critical rule line 327) explicitly runs reviews inline ("Both reviews run inline. Do not invoke `/orb:review-spec` or `/orb:review-pr` as skill calls"). ac-06, ac-07, ac-14 all presuppose forked-review semantics in drive-full.
**Evidence:**
- `git branch -a` shows `drive-forked-reviews` exists but has not merged to main.
- `gh pr list` shows PR #6 is OPEN.
- Current `plugins/orb/skills/drive/SKILL.md` on main has no "forked" or "Agent tool" mechanic for review-spec/review-pr.
- Spec exit conditions do not mention a merge gate for PR #6.
**Recommendation:** Add an explicit exit condition / prerequisite: "PR #6 (drive-forked-reviews) must be merged to main before this spec's ac-06, ac-07, ac-14 can be implemented or verified." Alternatively, reword F-01's resolution to state "depends on PR #6 merging; will be truthful once merged." Honesty principle (weight 0.35) requires this.

### [HIGH] Decision numbering will conflict with drive-forked-reviews on merge
**Category:** constraint-conflict
**Pass:** 1
**Description:** `metadata.decisions_pending` lists `orbit/decisions/0004-rally-subagent-path-discipline.md`, `orbit/decisions/0005-rally-parallel-drive-full.md`, `orbit/decisions/0006-rally-thin-card-guard.md`. The drive-forked-reviews branch *already* contains `orbit/decisions/0004-drive-verdict-contract.md`, `0005-drive-review-artefact-contract.md`, `0006-drive-cold-re-review.md`, and `0007-drive-rerequest-budget.md`. When drive-forked-reviews merges (a prerequisite per the previous finding), those numbers are taken. The rally-subagent decisions cannot use 0004–0006.
**Evidence:** `git ls-tree origin/drive-forked-reviews orbit/decisions/` shows 0004–0007 are occupied on that branch. ac-07 itself references `orbit/decisions/0004 through 0007` in a separate claim (that pointer targets the drive-forked-reviews decisions, not the ones listed in `decisions_pending` — those are conflicting uses of the same numbers within the same spec document).
**Recommendation:** Renumber rally-subagent decisions to 0008+ (or whatever is first-unused after drive-forked-reviews merges). Update `metadata.decisions_pending` accordingly. Also clarify in ac-07's description that "decisions 0004–0007" refers to the drive-forked-reviews decisions, not local ones.

### [MEDIUM] `git status --porcelain orbit/specs/` is narrower than the write discipline it enforces
**Category:** failure-mode
**Pass:** 2
**Description:** Constraint #9 and ac-04 mandate a three-primitive check where the independent verification is `git status --porcelain orbit/specs/`. The brief (constraint #4, ac-08 pattern) forbids writes outside the spec_dir, but a design sub-agent that writes to `plugins/...`, `orbit/cards/...`, or repo root would not be detected by a `orbit/specs/`-scoped status scan. The brief says "do not write anywhere else" but the verification primitive only inspects `orbit/specs/`. This is a leak in the trust+post-verify model the spec is trying to make honest.
**Evidence:** ac-04 verification text: "runs `git status --porcelain orbit/specs/` in the main checkout and rejects any entry outside <spec_dir>". Nothing about other top-level paths. The interview Q1 answer also scopes the check to `orbit/specs/`.
**Recommendation:** Either (a) widen the scan to `git status --porcelain` (unscoped) minus a small allowlist of expected lead-owned paths (rally.yaml), or (b) explicitly document the scope as a known limitation and accept that writes outside `orbit/specs/` are trust-only with no independent verification. Option (a) is closer to the honesty principle's intent; option (b) at least documents the gap.

### [MEDIUM] ac-17 adjacency scan is mechanically underspecified
**Category:** test-gap
**Pass:** 2
**Description:** ac-17 says every occurrence of "enforced | blocked | prevented | guaranteed" in sections §2, §4a, §7, §11 "must be followed within 80 characters by either a primitive citation or the phrase 'trust + post-verify'." But "primitive citation" has no mechanical definition — is `Agent tool`, `git worktree add`, `git status --porcelain`, `AskUserQuestion` each a citation? Is a backticked string required? What about the negation case — e.g. "writes are not blocked by the allow-list" (honest negation) fails the scan. Implementers have no unambiguous way to make the scan pass without turning SKILL.md into a mechanical puzzle. The 80-char window also risks false positives on unrelated adjacency.
**Evidence:** ac-17 description and verification; no regex or citation-set is named. The spec says "backed by a named Claude Code primitive cited inline (e.g. 'Agent tool with run_in_background', 'git worktree add', 'git status --porcelain')" — but "e.g." is not exhaustive.
**Recommendation:** Before implementation, fix the citation vocabulary: (1) name the exact set of accepted primitive tokens as a regex or literal list; (2) define how negations are handled (e.g., "not blocked" / "cannot be prevented" — explicit whitelist or scan bypass); (3) consider whether 80 chars is right — try it on the *current* SKILL.md first and tune. This belongs in the spec, not in the implementation. As written, ac-17 is ambiguous enough that two implementers would write two incompatible scanners.

### [MEDIUM] Serial-rally drive autonomy and mechanics undefined
**Category:** missing-requirement
**Pass:** 2
**Description:** Constraint #6 specifies "Drive-level autonomy inside a parallel sub-agent is always full." It is silent on serial cards. ac-16 says "For serial cards, drive-full runs in the main checkout against the same branch" — this implies serial cards also run drive-full, but constraint #6 does not say so, and the v2.0 SKILL.md §7 Serial Implementation says "Invoke drive's stage logic inline for this card" without naming an autonomy. Is serial drive-full, drive-guided, or rally's declared autonomy? The spec's refusal in ac-01 mentions "The author can still run thin cards via individual /orb:drive guided/supervised" — implying rally's own serial path is NOT drive-full for thin cards. But ac-16 reads as unconditionally drive-full for serial. Which is it?
**Evidence:** ac-16 description text says "drive-full runs in the main checkout" for serial; constraint #6 limits its autonomy claim to "parallel sub-agent". ac-01 context note mentions guided/supervised for post-park recovery. v2.0's §7 serial text says "drive's stage logic inline". The spec itself ships as a refinement of v2.0 without clarifying this.
**Recommendation:** Add a one-line constraint: "Serial implementation also runs drive-full against the rally branch in the main checkout. Rally-level autonomy does not change drive's internal autonomy." Alternatively, if the design intent was to respect rally's autonomy for serial, state that. Either way, make the intent explicit before implementation.

### [MEDIUM] ac-09 completion-event mechanism is not named as a primitive
**Category:** missing-requirement
**Pass:** 2
**Description:** Constraint #5 and ac-09 describe parallel sub-agents running "fire-and-forget in background (run_in_background: true); the lead reacts to completion events" with "exactly N rally.yaml writes, in the order completions are observed." But Claude Code's primitives for observing background-Agent completion are not named. The spec forbids sentinel files (constraint #5). The honesty-principle (ac-17) should require naming the primitive the lead uses to observe completion — otherwise "reacts to completion events" is a mechanism the spec has not proven exists. The deferred tool list includes `Monitor` and returning from Agent subprocess completion, but the spec picks none.
**Evidence:** Constraint #5 text; ac-09 description and verification. No mention of Monitor, Agent-return semantics, polling cadence, or the sequencing primitive. ac-17 is silent on this section's enforcement claim.
**Recommendation:** Name the completion-event primitive: either (a) "lead awaits Agent subprocess completion via Agent tool return semantics" if background Agents surface a completion signal the lead can block on, or (b) "lead polls via Monitor every N seconds" if that's the mechanism, or (c) rewrite the constraint to trust+post-verify ("lead detects completion when the sub-agent's final message arrives in the run queue, which the harness surfaces asynchronously"). Pick one and cite it.

### [LOW] ac-13 worktree-path mapping for serial is implicit
**Category:** test-gap
**Pass:** 2
**Description:** ac-13 tells the lead to read `each implementing card's worktree/specs/<spec_dir>/drive.yaml`. For serial cards, worktree sentinel is the literal `"main"` (ac-11). Mechanically, the lead must map `"main"` → the main checkout's filesystem path, i.e. `orbit/specs/<spec_dir>/drive.yaml` rather than `main/specs/...`. This mapping is obvious but not stated; an implementer could write `${worktree}/specs/${spec_dir}/drive.yaml` and produce `main/specs/...`, which fails.
**Evidence:** ac-11 sentinel semantics ("'main' for serial cards once launched"); ac-13 path pattern.
**Recommendation:** Add one sentence to ac-11 or ac-13: "When worktree == 'main', the lead resolves the drive.yaml path as `<main checkout root>/specs/<spec_dir>/drive.yaml` (no worktree prefix)."

### [LOW] Lead branch-switching during serial rally is unaddressed
**Category:** failure-mode
**Pass:** 2
**Description:** Serial rally stacks PRs on previous cards' branches. ac-16 says the lead checks out the rally branch in the main checkout for serial cards. With multiple serial cards, the lead must checkout card A's branch, implement, then checkout card B's branch (on top of card A), etc. During this, the lead's own session state — open editor buffers are irrelevant here, but `orbit/specs/rally.yaml` lives in the main checkout and is not in `rally/<slug>` branches. If the lead switches to `rally/card-a` before any commits on that branch, rally.yaml is still visible (shared across branches until a write on that branch), but once the sub-agent commits spec.yaml on the rally branch and the lead writes rally.yaml, ordering of writes vs checkouts becomes a coordination problem. The spec does not address this.
**Evidence:** ac-16 for serial path; v2.0 §7 stacked-PRs model; no constraint governing rally.yaml's branch-home.
**Recommendation:** Add a constraint or section: rally.yaml lives on `main` (or a dedicated `rally-coordination` branch) and lead operations that write it happen while checked out to that branch, not the rally/<slug> branches. Alternatively, document that rally.yaml lives at `orbit/specs/rally.yaml` in every rally branch via merge-forward — less clean. Either way, name it explicitly.

### [LOW] `ambiguity_score: 0.06` is not calibrated to the findings
**Category:** test-gap
**Pass:** 1
**Description:** The spec declares ambiguity 0.06 — very low. But the findings above expose multiple undefined mechanisms (completion-event primitive, adjacency regex, serial autonomy, path-prefix mapping). 0.06 seems aspirational rather than measured.
**Evidence:** Cumulative Pass 1+2 findings.
**Recommendation:** Bump to ~0.15–0.20 or remove the claim until the above gaps are resolved. Not blocking, but worth honesty-principle attention.

---

## Honest Assessment

This is a thoughtful refinement — the v1.0 review findings are taken seriously, the honesty principle is real (making enforcement claims auditable via ac-15/ac-17 is the right instinct), and the continuation interview's two decisions (F-02 three-primitive verification, F-11 commit-before-worktree) are well-reasoned and cite the right primitives.

The biggest risk is sequencing: the spec is written as if drive-forked-reviews has shipped, but PR #6 is open on its own branch. If rally-subagent-model is implemented and merged before drive-forked-reviews, ac-06/07/14 become aspirational and the spec's honesty principle (its headline value, weight 0.35) is immediately violated. Make the merge-order explicit: drive-forked-reviews first, renumber decisions to 0008+, then rally-subagent-model. A dependency note in exit_conditions would close the HIGH findings.

The three MEDIUM findings (scan scope, adjacency ambiguity, serial autonomy) are each small scope-clarifications the author can close in a 15-minute edit. Without them, implementation will hit decision points with no spec guidance, and reasonable engineers will pick differently.

Nothing here calls for re-design or re-discovery — Pass 3 is not triggered. A focused round of spec edits addressing the HIGH sequencing and decision-numbering findings plus the four MEDIUM clarifications should produce an APPROVE on re-review.

---

**Verdict:** REQUEST_CHANGES
