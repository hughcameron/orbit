# Design: Rally Sub-Agent Model — Review-Closure Continuation

**Date:** 2026-04-20
**Interviewer:** Nightingale
**Card:** orbit/cards/0006-rally.yaml
**Mode:** design (continuation)
**Triggered by:** spec review of `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` — REQUEST_CHANGES with 11 findings (3 HIGH, 5 MEDIUM, 3 LOW)
**Prior record:** `orbit/specs/2026-04-19-rally-subagent-model/interview.md` (2026-04-19 discovery)

---

## Context

The 2026-04-19 discovery session produced the sub-agent model spec (v1.0) with five decisions closing F-03 and F-05 from the v2.0 PR review. That spec was then reviewed; the reviewer returned REQUEST_CHANGES with 11 findings. This session closes the findings that require a design-level call — F-02 (write-verification mechanism) and F-11 (interview.md provisioning to sub-agent worktrees) — so the spec can be amended without further discovery.

**State of the 11 findings at session open:**

| ID | Severity | Status entering this session |
|----|----------|------------------------------|
| F-01 | HIGH | **Resolved by drive-forked-reviews PR #6** — drive now forks nested reviews at the architectural root, matching constraint #2's claim. |
| F-02 | HIGH | Open — needs primitive choice for lead's post-return write verification. |
| F-03 | HIGH | **Already answered in the 2026-04-19 discovery Q3** — refuse at proposal for thin cards. The spec's ac-01 text drifted; amendment restores the discovery's answer. |
| F-04 | MED | **Already answered in the 2026-04-19 discovery Open Question #3** — single-strike NO-GO absorbs all drive-full escalation triggers. Tightening only. |
| F-05 | MED | Tightening — ac-15 coherence gate needs a mechanical verifier. |
| F-06 | MED | Tightening — ac-12 warning surfacing channel. |
| F-07 | MED | Tightening — `cards[].worktree` sentinel vs null for serial. |
| F-08 | MED | Tightening — honesty principle weight vs measurability. |
| F-09 | LOW | Tightening — v2.0 metadata drift on tools allow-list phrase. |
| F-10 | LOW | Tightening — ac-04 re-brief vs single-strike interaction. |
| F-11 | LOW | Open — genuinely new; how interview.md reaches the sub-agent's worktree. |

Only F-02 and F-11 needed this session. The other nine resolutions are applied directly in the amendment.

---

## Q&A

### Q1: Write-verification primitive for design sub-agents (F-02)

**Q:** The reviewer flagged ac-04's verification as undefined — "lead checks for any files written outside that spec_dir" names no primitive. Three options were considered in the discovery's Open Question #1: git status scan, pre/post filetree snapshot, sub-agent self-report. The reviewer added that worktrees complicate `git status` because sub-agent writes in a worktree aren't visible to main-checkout `git status`. Narrowing question: pick the primitive.

**A:** **Self-report + artefact assertion + `git status --porcelain` scan in the main checkout. All three.**

Context that narrows the choice: *design* sub-agents write to the main checkout (design runs before any worktree splits — worktrees appear only at parallel implementation launch, per the 2026-04-19 Q5 fire-and-forget answer). `git status` in the main checkout therefore *does* see design sub-agent writes. The worktree-invisibility problem only applies to parallel-implementation sub-agents, which don't need this verification (they run drive-full in bounded worktrees where path discipline is physically enforced by the separate working tree).

Belt-and-suspenders rationale:

- **Self-report alone** = trust without a check. A sub-agent that lies or silently writes extra files passes.
- **Git-status alone** = catches violations but doesn't impose a contract on the sub-agent. The sub-agent isn't told what's expected.
- **Artefact assertion alone** = proves the intended file landed but says nothing about unintended ones.

Together: self-report is the **contract** the brief imposes, artefact assertion is the **completeness check**, `git status --porcelain orbit/specs/` is the **independent verification**. Matches the spec's honesty principle — `git status` is a real Claude Code primitive, self-report is a brief-level contract, artefact existence is a filesystem check. All three are things we can honestly say we're using, and none invents a primitive.

Implementation sketch:

```
brief to sub-agent:
  "Return a JSON object {files: [list of paths you wrote]}."

lead on return:
  1. parse returned files list
  2. assert expected artefact exists at <spec_dir>/{decisions.md|interview.md}
  3. assert every returned path is under <spec_dir>
  4. git status --porcelain orbit/specs/ → reject any entry outside <spec_dir>
```

**Connects to F-10:** violation handling remains per the discovery — first violation re-briefs with an explicit warning, second violation parks the card with `parked_constraint: "sub-agent violated path discipline"`. Re-brief is a pre-qualification retry, not a drive iteration, and does not consume single-strike budget.

---

### Q2: Interview.md provisioning to parallel sub-agent worktrees (F-11)

**Q:** ac-06 tells parallel sub-agents to run `/orb:drive <card> full` using "the already-produced interview.md at `<spec_dir>/interview.md`." Drive-full then re-enters design (drive §3), which writes interview.md. Drive §11 resumption's file-presence check is what lets drive-full skip design when interview.md already exists. But how does interview.md reach the sub-agent's worktree? Three mechanical options: commit-to-branch-before-worktree, copy-after-create, or path-override via drive contract change.

**A:** **Commit `interview.md` to the card's rally branch before creating the worktree.**

```
lead (main checkout):
  git checkout -b rally/<slug>
  git add orbit/specs/<slug>/interview.md
  git commit -m "rally/<slug>: approved design"
  git worktree add ../<repo>-rally-<slug> rally/<slug>

sub-agent (worktree):
  reads orbit/specs/<slug>/interview.md at the expected path
  drive-full §11 resumption fires → skips design, starts at spec stage
```

Rationale against the alternatives:

- **vs copy-after-create:** leaves interview.md as an uncommitted working-tree file in the sub-agent's worktree. Drive §11 resumption works mechanically (file-exists check), but when the sub-agent commits implementation, the working tree has an untracked design artefact — either swept into the implementation commit (messy) or sitting unstaged (inconsistent). Git history on the branch doesn't show the approved design as the starting point.

- **vs path-override (drive contract change):** breaks drive's fork-boundary cleanliness. Drive §11 resumption would need to accept an external path, contradicting decision 0005's "artefact-path contracts are durable across session boundaries; session state is not." Drive-forked-reviews PR #6 specifically established the fork boundary as self-contained; a rally sub-agent reaching back into the main checkout reintroduces the cross-boundary state problem that decision 0005 eliminated.

- **For commit-before-worktree (recommended):** the rally branch naturally tells the card's story — "approved design → spec → implementation → tests" — in commit order. Mirrors how drive accumulates artefacts within a single-card flow. The sub-agent's worktree is a clean working copy of that branch; its commits append cleanly. No contract changes to drive.

**Edge case — serial rallies:** for cards that run serial (no worktree split), the lead commits interview.md to the rally branch but keeps working in the main checkout. Drive-full runs in the main checkout against the rally branch. Same §11 resumption path fires. The commit-before-worktree rule generalises to "commit-before-delegation" — before any sub-agent or driver is handed the card.

---

## Summary

### Goal (unchanged from 2026-04-19 discovery)

Rally's sub-agent orchestration must honestly describe what Claude Code actually provides: no invented path-enforcement primitive, no interactive gates inside non-interactive contexts, no state ownership that races.

### Constraints added by this session

- Design sub-agent path discipline is enforced by three cheap primitives on return: sub-agent self-report of files written (contract), artefact-existence assertion at the expected path (completeness), and `git status --porcelain orbit/specs/` in the main checkout (independent verification).
- Before launching any parallel implementation sub-agent, the lead commits the approved `interview.md` to the card's rally branch and creates the worktree from that branch. Drive-full's §11 resumption detects `interview.md` and starts at the spec stage. No drive contract change.

### Decisions Surfaced

- **F-02 — Design sub-agent write verification is self-report + artefact assertion + `git status` scan.** Chose three-primitive belt-and-suspenders over single-primitive trust or single-primitive check. Matches honesty principle; all three primitives are real. → recorded inline in the amended spec; no standalone MADR (continuation of discovery decisions, not a new architectural axis).
- **F-11 — Interview.md reaches sub-agent worktrees via commit-to-rally-branch-before-worktree.** Chose this over copy-after-create or path-override. Keeps drive's fork boundary self-contained (decision 0005), preserves clean git history on the rally branch, no contract change to drive. → recorded inline in the amended spec.

### Applied-directly resolutions (no discovery needed)

- **F-03** — restore 2026-04-19 discovery Q3 answer: "refuse at proposal" for thin cards. Drop ac-02's serial carve-out that drifted from the discovery. Thin cards warn-but-allow in serial rallies per Open Question #2.
- **F-04** — single-strike NO-GO absorbs all four drive-full escalation triggers (budget, recurring, contradicted, diminishing) plus the synthetic BLOCK from drive-forked-reviews decision 0007 (review converged on REQUEST_CHANGES after 3 iterations). `parked_constraint` includes drive's escalation reason label verbatim.
- **F-05** — ac-15 coherence gate gains a mechanical keyword-scan verifier: assert forbidden phrases absent ("tools allow-list", "drive's stage logic inline"), assert required phrases present ("trust + post-verify", "recursive context separation").
- **F-06** — `worktree_missing` warning surfaces once at rally resumption or the next rally.yaml-writing transition, suppressed on subsequent reads if still missing.
- **F-07** — `cards[].worktree` uses an explicit sentinel: `"main"` for serial cards (once launched), absolute path for parallel cards (once launched), `null` only for pre-launch.
- **F-08** — honesty principle gains a measurable AC: every mechanism SKILL.md claims to enforce is either backed by a named Claude Code primitive cited inline, or explicitly marked as convention (trust + post-verify).
- **F-09** — v2.0's `metadata.review_changes` line on the tools allow-list is rewritten to reflect trust + post-verify (amendment exit condition).
- **F-10** — ac-04 explicitly names re-brief as a pre-qualification retry, not a drive iteration; does not consume single-strike budget; one retry then park.

### Open Questions

None. All 11 review findings have a decided resolution.

---

**Next step:** `/orb:spec` amends `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` to v1.1 with:
1. Two new constraints capturing F-02 and F-11 decisions.
2. ac-01/ac-02 rewrites per F-03 restoration.
3. ac-04 rewrite per F-02 + F-10.
4. ac-11 rewrite per F-07 sentinel.
5. ac-12 rewrite per F-06 surfacing channel.
6. ac-14 rewrite per F-04 escalation taxonomy.
7. ac-15 rewrite per F-05 keyword-scan verifier.
8. New ac for interview.md provisioning (F-11).
9. New ac for honesty-principle auditability (F-08).
10. `metadata.review_changes` documents all 11 findings and their resolutions; bump to v1.1.
11. Exit condition added for v2.0 metadata.review_changes amendment (F-09).
