The current workflow requires Hugh to explicitly approve every PR merge. In practice this has become ceremony — the agent presents a status summary, waits for "merge it," and Hugh rubber-stamps.

The real quality gate is review-pr: a cold-fork context that reads the spec and diff without implementation bias. That's where defects get caught. The merge-approval step after review-pr adds latency without adding signal.

Observed pattern: agent completes implementation, opens PR, runs review-pr (cold fork catches real issues), then... waits. Hugh says "merged" or "merge it." The approval is a formality at that point — the substantive review already happened.

What this suggests:

1. **Auto-merge after passing review-pr.** If the cold-fork reviewer approves, the agent should merge without waiting for Hugh. The review is the gate, not the merge approval.
2. **Hugh still sees the PR.** Visibility isn't the problem — being a blocking step in the pipeline is. Notify, don't block.
3. **Preserve the override.** Hugh can still reject or hold any PR. The change is default-merge-on-approval, not remove-human-from-loop.
4. **This compounds with research iteration speed.** If the research workflow gets lighter-weight iteration (see orbit-heavy-for-research memo), removing merge-approval ceremony from the engineering side reduces total operator bottleneck across both domains.
