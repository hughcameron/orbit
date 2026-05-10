# Design: Drive's Reviews Fork

**Date:** 2026-04-20
**Interviewer:** Nightingale
**Card:** .orbit/cards/0007-drive-forked-reviews.yaml

---

## Context

Card: *Drive's reviews fork* — 10 scenarios, goal: every drive invocation runs review-spec and review-pr as forked context-separated agents, honouring the components' declared fork contract and eliminating confirmation-bias-prone inline review at the architectural root.

Prior specs: 0 — this is the first spec for this card.

Gap: Drive's SKILL.md §5, §7, and Critical Rules instruct inline review ("read SKILL.md and follow its instructions directly within this session"). Review-spec's SKILL.md frontmatter declares `context: fork`. Review-pr's declares `context: fork, agent: general-purpose`. Drive is the violation — the component contracts already declare what should happen, drive just doesn't honour them.

Surfaced by: review-spec of `.orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` — F-01 identified drive's inline reviews as the structural cause of rally's parallel-sub-agent contradiction. Fixing at drive's layer dissolves the rally problem at its root and brings every drive invocation (not just rally sub-agents) into contract-honest behaviour.

## Q&A

### Q1: Verdict contract format
**Q:** Scenarios 6 and 7 require drive to parse a verdict from the review file. What format should be contractual?
**A:** **Strict markdown — `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK`** as the canonical line. Drive's parser locates the first line matching `^\*\*Verdict:\*\* (APPROVE|REQUEST_CHANGES|BLOCK)\s*$` and fails fast if absent or ambiguous. Aligns with existing review files; keeps reviews human-readable; no sidecar files or frontmatter blocks to maintain.

### Q2: Fork failure taxonomy
**Q:** Scenario 4 says "error, timeout, or missing verdict → retry once, then escalate." Multiple failure modes exist (agent errors pre-write, agent errors post-write, file exists but no verdict, chat and file disagree). How should they be treated?
**A:** **File-on-disk is authoritative.** The rule is minimal: drive extracts the verdict from the file. If the file doesn't exist, or has no parseable verdict line, drive retries once with a fresh fork. If the retry also fails to produce a parseable verdict, drive escalates with "review could not be completed after 2 forked attempts." The agent's chat response is not consulted. Agent-level success/failure status is irrelevant — only the artefact matters. Scenario 7's "file wins over chat" follows trivially because chat is never read.

### Q3: Re-review context on REQUEST_CHANGES
**Q:** When drive addresses REQUEST_CHANGES and re-forks, what does the new reviewer receive?
**A:** **Fully cold — only the spec/diff.** Each re-review is an independent context-separated read. The new reviewer doesn't know it's pass 2 or 3, doesn't see prior review files, doesn't know what was fixed. Pure context separation, same contract as the first review. The risk of re-flagging already-addressed findings is accepted as the cost of genuine independence — and is bounded by the budget in Q4.

### Q4: REQUEST_CHANGES internal budget
**Q:** Current drive loops on REQUEST_CHANGES without bound. With forked reviews costing a fresh agent per pass, should that loop be bounded?
**A:** **Bounded — 3 REQUEST_CHANGES cycles per stage, then escalate to synthetic BLOCK.** After the 3rd REQUEST_CHANGES without reaching APPROVE, drive converts the next verdict into a BLOCK with constraint "review converged on REQUEST_CHANGES after 3 iterations; findings have not been addressable within budget." That triggers the existing NO-GO handling: re-enter at design with the accumulated findings as the new constraint. Prevents runaway loops; aligns with disposition-bounded-by-honest-escalation. The stage-level 3-cycle budget is uniform across review-spec and review-pr — simpler than per-stage tuning.

### Q5: Migration for in-flight drives
**Q:** When drive's SKILL.md ships this change, any drive in-flight with status `review-spec` or `review` could hit the new code mid-stride. How should migration be handled?
**A:** **Finish or park in-flight drives before upgrade.** Session-context.sh already surfaces active drives at session start. The upgrade note instructs: complete or park any in-flight drive before running a drive with the new SKILL.md. No `review_mode` compatibility field in drive.yaml; no dual code paths. Matches orbit's self-hosted single-user reality — drives run one at a time, upgrades happen between drives.

---

## Summary

### Goal
Every drive invocation runs review-spec and review-pr as forked context-separated agents, honouring the components' declared `context: fork` contract. The architectural inconsistency between drive and its component skills is eliminated at the root.

### Constraints
- Verdict contract is strict markdown: `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK` as a single canonical line; drive's parser fails fast on absence or ambiguity
- File-on-disk is the only authoritative source — drive reads the verdict from the saved review file, never from the agent's chat response
- Fork failure handling is uniform: no parseable verdict → retry once → escalate with clear message
- Re-reviews after REQUEST_CHANGES are fully cold — each forked reviewer reads spec/diff with no knowledge of prior passes
- REQUEST_CHANGES loop is bounded at 3 cycles per stage; the 4th REQUEST_CHANGES is converted to synthetic BLOCK and triggers NO-GO handling
- No migration scaffolding in drive — in-flight drives are expected to finish or park before upgrade; session-context.sh surfaces the state to force the choice
- drive.yaml and review files on disk are the only state contract across fork boundaries — forked reviewers see neither drive.yaml nor chat, only the spec/diff

### Success Criteria
- drive/SKILL.md §5 and §7 describe launching review-spec / review-pr as forked general-purpose Agents via the Agent tool, not reading SKILL.md inline
- drive/SKILL.md Critical Rules no longer contains "Both reviews run inline" — it says reviews fork, citing the component frontmatter contracts
- drive's verdict-parse is specified: the regex, the fail-fast behaviour, the retry-once-on-missing policy
- drive's REQUEST_CHANGES handling documents the 3-cycle budget per stage, the synthetic-BLOCK conversion, and the constraint string used
- Every scenario in card 0007 maps to an AC with a concrete verification method
- After implementation, running `/orb:drive` on any test card produces two review files at the expected paths, written by forked agents, with drive reading only the files on disk

### Decisions Surfaced
- **Strict markdown verdict contract**: chose `**Verdict:** X` over frontmatter or sidecar. Aligns with existing reviews; no new artefact to maintain. (→ `.orbit/choices/0004-drive-verdict-contract.md`)
- **File-on-disk authoritative**: chose artefact-only over agent-success-plus-artefact. Minimal rule, uniform handling across all failure modes. Drive treats reviews as a stateless produce-a-file function. (→ `.orbit/choices/0005-drive-review-artefact-contract.md`)
- **Fully cold re-reviews**: chose pure independence over pointer-to-prior or iteration-hint. Confirmation-bias resistance is the whole point; any context bleed compromises it. Cost (possible re-flagging) is bounded by Q4. (→ `.orbit/choices/0006-drive-cold-re-review.md`)
- **Bounded REQUEST_CHANGES at 3 per stage**: chose uniform 3-cycle limit over unbounded or per-stage tuning. Matches drive's overall 3-iteration budget shape; synthetic BLOCK is the clean escalation path. (→ `.orbit/choices/0007-drive-rerequest-budget.md`)
- **No migration scaffolding**: chose finish-or-park over dual code paths or flag day. Orbit is self-hosted; single-drive-at-a-time is already enforced; simplicity wins.

### Open Questions
- **Exact Agent-tool invocation shape.** The spec will need to specify the brief template, the `subagent_type`, and how drive passes the spec path. This is a spec-level detail, not a design decision — the design is "fork via Agent tool," the spec picks the exact call.
- **Where the retry counter lives.** The fork retry (Q2) and the REQUEST_CHANGES budget (Q4) are independent counters. drive.yaml currently tracks the top-level iteration. Should the per-stage REQUEST_CHANGES count go in drive.yaml too (for resumption), or is it session-only (reset on any resume)? Probably drive.yaml for honesty; to be specified.
- **Rally dependency ordering.** Card 0006 (rally) is blocked on this card. The rally refinement spec (`.orbit/specs/2026-04-19-rally-subagent-model/spec.yaml`) has ACs that become redundant once this ships. Sequencing: ship 0007 first, then revise the rally refinement spec to collapse the now-redundant ACs. The rally v2.0 spec may also need its `review_changes` metadata updated to reference 0007 as the resolution of F-01.
