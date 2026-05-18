# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-17-code-investigate-skill
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (cross-system Edit/Write hook fires in every session in this and consumer repos; shared `.claude/settings.json` registration; concurrent-fan-out marker file) | 4 |
| 3 — Adversarial | not triggered | — |

## Findings

### [HIGH] ac-02 plugin-format registration is unresolved at spec time
**Category:** missing-requirement
**Pass:** 1 (gap) → 2 (failure-mode amplifier)
**Description:** ac-02 says the PreToolUse hook is "installed in `plugins/orb/hooks/` (or wherever the plugin loader expects)" and the implementation notes explicitly defer "plugin manifest hooks field vs `.claude/settings.json` hooks block — implementing agent to confirm". A spec is the wrong artefact to carry an *unresolved registration target* for the load-bearing primitive (the hook is what delivers the discipline; without registration it never fires). The "Open questions" section claims "none at intent level" and routes this to implementation — but the implementing agent is being asked to make a substrate-shape choice that affects every consumer repo, not a tactical detail.
**Evidence:**
- spec ac-02 description, lines reading "installed in `plugins/orb/hooks/` (or wherever the plugin loader expects)"
- interview.md "Implementation notes" §1 — *No `plugins/orb/hooks/` directory exists yet… implementing agent to confirm against current Claude Code plugin format*
- prior memory `missing-hooks-json-smoke-test`: orbit shipped 0.4.5 with no `plugins/orb/hooks/hooks.json` and the plugin loader reported "0 hooks · 0 plugin MCP servers" — i.e. the plugin format **does not** deliver hooks. The only working surface today is `.claude/settings.json` (already used for SessionStart/PreCompact/Stop).
- ac-02 verification: "Grep `.claude/settings.json` (or wherever the plugin registers hooks) confirms the registration" — hedge present in the verification clause itself.
**Recommendation:** Resolve before implementation. Either (a) pin the spec to `.claude/settings.json` registration (consistent with the existing PreCompact / SessionStart / Stop hooks and the prior memory's finding) and treat per-consumer-repo opt-in as the v1 reality, accepting that adoption gate; or (b) open a sub-spec / choice MADR for "plugin-shipped hooks shape" if the intent is plugin-shipped delivery to every consumer repo with no manual setup. Choosing the wrong path here is hard to walk back — agents in consumer repos hand-editing `.claude/settings.json` is a different rollout shape from a plugin-manifest entry.

### [MEDIUM] ac-02 hook lacks graceful-degradation contract for non-orbit and clean-clone environments
**Category:** failure-mode
**Pass:** 2
**Description:** The PreToolUse hook fires on every Edit/Write. ac-02 specifies its behaviour when `.orbit/.code-investigate-recent` exists; it does not specify what happens when (a) the marker file is missing on a fresh session, (b) `.orbit/` doesn't exist (a consumer repo that has loaded the plugin but not run `/orb:setup`), or (c) the hook runs in a CI / non-interactive environment where surfacing the warning is noise. The current text would imply the hook warns on every Edit/Write in any non-orbit project where the plugin happens to be loaded — likely the noisiest possible failure mode for a soft nudge.
**Evidence:** ac-02 description, ac-03 description. Neither pin behaviour when `.orbit/` is absent or the marker file hasn't been written yet.
**Recommendation:** Add to ac-02: hook silently skips when `.orbit/` is absent; when `.orbit/` exists but the marker file is absent, the hook still warns (this is the "session with no investigation" case the interview describes). Optionally: add an env-var or config knob to disable in CI. One sentence in ac-02 closes this.

### [MEDIUM] ac-02 doesn't bound the warning surface to code edits
**Category:** test-gap
**Pass:** 2
**Description:** The hook fires on every Edit/Write — including edits to cards, memos, the spec itself, CLAUDE.md, settings.json. The interview pass-2 Q1 noted that "noise concern self-resolves because the agent doing its job naturally quiets the warning" — but this only holds when the warning is firing for actual code edits. Doc/substrate edits don't have a /orb:code-investigate counterpart; they're a permanent warning baseline that the ac-07 audit will count under "warning-fire count" alongside genuine code-edit fires. The 4-week audit signal becomes hard to interpret.
**Evidence:** ac-02 description (no path-filter); ac-07 verification table includes "warning-fire count from the ac-02 hook" with no segmentation between code and non-code paths.
**Recommendation:** Either (a) ac-02 pins a default path-filter (extensions like `.rs/.ts/.py/.sh/.go/...`, or "skip when the file lives under `.orbit/` or matches `*.md`"), or (b) ac-07 specifies that the warning-fire count is broken down by file kind so the audit can distinguish nudge effectiveness from doc-edit baseline. Either resolves the audit-interpretation gap.

### [MEDIUM] ac-03 verification is out of sync with the design-pass-2 decision
**Category:** constraint-conflict
**Pass:** 1
**Description:** The design pass 2 interview Q1 explicitly chose "session-scoped — no clock TTL" for the marker file. ac-03's verification clause still reads "is cleaned up by Stop hook (or expires by TTL — design pass chooses)". The hedge is stale — the design pass already chose. Either ac-03 should cite the Stop-hook cleanup (which means amending the existing Stop hook in `.claude/settings.json` to also `rm -f .orbit/.code-investigate-recent`), or the cross-session detection should be via `.orbit/.session-id` comparison (which the description mentions as an alternative). Pinning either is a one-line change; leaving both is an open seam.
**Evidence:**
- ac-03 description: *"entries from prior sessions are ignored via `.orbit/.session-id` comparison (or cleared at session start)"* — two options listed
- ac-03 verification: *"is cleaned up by Stop hook (or expires by TTL — design pass chooses)"* — three options listed
- interview pass-2 decisions table: *"Session is the natural lifetime — no clock TTL"* — TTL ruled out
- `.claude/settings.json` Stop hook currently deletes `.session-id` and `.session-card` — natural place to amend
**Recommendation:** Pick one of the two surviving options (Stop-hook cleanup vs `.session-id` comparison) in ac-03's description, drop the TTL hedge from the verification clause, and — if Stop-hook cleanup — add to the AC: "`.claude/settings.json` Stop hook updated to include `.orbit/.code-investigate-recent` in the cleanup list."

### [LOW] ac-07 audit-memo path hard-codes a date that depends on ship date
**Category:** assumption
**Pass:** 2
**Description:** ac-07 sets earliest fire date "2026-06-14 (4 weeks after this spec lands)" and hard-codes the audit memo path `.orbit/memos/2026-06-14-code-investigate-4-week-audit.md`. The spec hasn't shipped yet (today is 2026-05-18); if it lands later, the date and path diverge from the AC text. Observation-band ACs are deferrable per ac-taxonomy, so this won't block `spec.close` — but the audit memo created at the actual +4-week mark will sit at a different path than the AC predicts.
**Evidence:** ac-07 description and verification both pin `2026-06-14`.
**Recommendation:** Soften to "earliest fire date: 4 weeks after this spec ships; audit memo lands at `.orbit/memos/<ship-date+4w>-code-investigate-4-week-audit.md`". One-line change.

---

## Honest Assessment

The plan is sound at intent level — the agent-equipment framing is well-argued, the two modes (narrow / broad) map cleanly to the work, soft-nudge enforcement is the right strength for a learning surface, and the memory-tag learning loop reuses substrate that already exists. The interview record (initial + design pass 2) does real work; few specs in this repo are this well-grounded at the intent level.

The biggest risk is the unresolved hook-registration question in ac-02. It looks tactical but it isn't — it determines whether the discipline ships to consumer repos at all, or only fires in repos that hand-edit `.claude/settings.json`. The implementing agent should not be choosing between those two rollout shapes mid-implementation; either the spec pins `.claude/settings.json` (and accepts per-repo opt-in as the v1 reality), or it spawns a choice MADR for plugin-shipped hooks before implementation starts. The prior memory `missing-hooks-json-smoke-test` already establishes that the plugin format does not deliver hooks today — that finding should land in the spec.

The remaining four findings are tighten-up work: pin the hook's degradation behaviour (MEDIUM), bound the warning surface (MEDIUM), sync ac-03's marker-lifecycle hedge with the design-pass-2 decision (MEDIUM), and soften ac-07's hard-coded date (LOW). None block once the HIGH is resolved; all are one-to-two-line AC edits.

Verdict is REQUEST_CHANGES rather than BLOCK because the design is right — the spec just needs to commit on the load-bearing seam before the implementing agent is asked to.
