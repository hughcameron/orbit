# Pre-Merge Review — Rally Subagent-Model Follow-Up (ac-06, ac-07, ac-14)

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Branch:** rally-subagent-followup
**PR:** https://github.com/hughcameron/orbit/pull/7
**Spec:** .orbit/specs/2026-04-19-rally-subagent-model/spec.yaml (v1.3)
**Verdict:** APPROVE

---

## Scope of This PR

Lands the three PR-#6-gated ACs from the rally-subagent-model spec plus one decision record:

| AC / Artefact | Claimed by progress.md | Kind |
|---------------|------------------------|------|
| ac-06 | §7c brief template + launch-mechanics paragraph | code (doc-verified) |
| ac-07 | §7c nested-fork contract list + Integration section cite | doc |
| ac-14 | §9 fixed label-mapping table + `[unknown]` fall-through | code (doc-verified) |
| .orbit/choices/0009-rally-parallel-drive-full.md | new | decision |

Three files changed: `.orbit/choices/0009-rally-parallel-drive-full.md` (+58/-0), `plugins/orb/skills/rally/SKILL.md` (+30/-7 by diff hunks), `.orbit/specs/2026-04-19-rally-subagent-model/progress.md` (flips the three deferred rows).

PR #6 (`drive-forked-reviews`) is confirmed merged to main (mergedAt 2026-04-20T04:35:52Z) — the first exit condition is satisfied, and the honesty principle allows these ACs to go live.

---

## Test Results

| Check | Result | Details |
|-------|--------|---------|
| Test suite | N/A | Repo has no automated test suite; this PR is SKILL.md + decision + progress only |
| AC coverage (rallysub prefix) | EXEMPT | All three ACs verified by inspection + script execution per spec verification fields |
| `plugins/orb/scripts/rally-coherence-scan.sh` | PASS (exit 0) | 3 forbidden-absent, 4 required-present, adjacency scan empty |
| Decision citation fidelity | PASS | All four titles (0004–0007) match decision file headers byte-for-byte |
| §9 escalation-trigger coverage | PASS | 5-row table covers drive §9's 4 semantic triggers + decision 0007 synthetic BLOCK |

### AC Coverage Report (prefix: rallysub)

| AC | ac_type | Status | Evidence |
|----|---------|--------|----------|
| ac-06 | code | PASS (doc-verified) | SKILL.md §7c lines 349–376: brief template + "The Agent tool is invoked with `run_in_background: true` and `subagent_type: \"general-purpose\"`; every call is in the same message so the harness dispatches all N in parallel. (ac-06)" |
| ac-07 | doc | PASS | SKILL.md §7c lines 331–336 cite all four decisions by title; Integration section line 566 repeats the range citation |
| ac-14 | code | PASS (doc-verified) | SKILL.md §9 lines 441–454: fixed 5-row label-mapping table + `[unknown]` fall-through |

**Doc-exemption note.** The spec marks ac-06 and ac-14 as `code` but their verification fields (spec.yaml:48, 92) call for scenario-driven inspection — "trigger a 2-card parallel rally", "trigger one parallel sub-agent per escalation trigger (mock five scenarios)". These scenarios cannot be executed in a repo-as-plugin context without a rally harness; the artefact this PR can actually ship is the SKILL.md specification of behaviour and the coherence-scan gate. That matches how the first-PR review treated the other code-type ACs in this spec. The spec prefix `rallysub` finds no prefixed tests, which is expected — no bare `ac06`/`ac14` tests either. Applying the `doc` exemption logic thoughtfully: these ACs are effectively doc-verified because the "implementation" is prose in SKILL.md, and the coherence-scan script is the mechanical check on that prose.

---

## Rally Coherence Scan

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
```

Exit 0. All four required phrases (including "recursive context separation" and "nested forked Agents", which are load-bearing for ac-07) are present; the three forbidden phrases are absent. The adjacency scan has no enforcement verbs to chase in the rewritten §7c — consistent with the first-PR note.

---

## Decision Keyword Scan — Compliance with 0004–0010

- **0004-drive-verdict-contract** (Drive's Verdict Contract: Strict Canonical Markdown Line): cited at SKILL.md §7c line 331 by exact title. Decision 0009 §"Decision Outcome" names it by filename + title. Compliance: OK.
- **0005-drive-review-artefact-contract** (Drive's Review Artefact Contract: File-on-Disk Authoritative): cited at SKILL.md §7c line 332 by exact title. Decision 0009 repeats the citation. Compliance: OK.
- **0006-drive-cold-re-review** (Drive's Re-Review Context: Fully Cold): cited at SKILL.md §7c line 333 by exact title. Compliance: OK.
- **0007-drive-rerequest-budget** (Drive's REQUEST_CHANGES Budget: 3 Cycles Per Stage): cited at §7c line 334, §9 line 451, and decision 0009. The synthetic BLOCK semantics (constraint string, NO-GO re-entry) in §7c line 334 faithfully paraphrase 0007 without contradicting it. Compliance: OK.
- **0008-rally-subagent-path-discipline**: referenced from decision 0009's Related block; not materially changed by this PR. Compliance: OK.
- **0009-rally-parallel-drive-full** (new): status `accepted`, MADR format correct, Considered Options section has three options with explicit rejection reasons for A and C. Option B's trade-offs (nested fork cost: up to 2 × 4 = 8 reviewer invocations per rally sub-agent) are named honestly. Compliance: OK.
- **0010-rally-thin-card-guard**: referenced from decision 0009's Related block; not materially changed. Compliance: OK.

No violations found. The rally SKILL.md's §7c pointer list directly honours decision 0003's "rally as a separate skill from drive" boundary — rally points to drive's contracts rather than duplicating them.

---

## Edge-Case Probe Results

**1. Are the four §7c decision citations accurate by title?** Yes. Grep of each `.orbit/choices/000[4-7]-*.md` file's header line produces an exact string match against SKILL.md §7c lines 331–334. No typos, no paraphrasing that would mislead a cold reader.

**2. Is §9's label-mapping table complete relative to drive's escalation triggers?** Yes. Drive SKILL.md §9 (line 354–356) names four semantic triggers: budget exhaustion, recurring failure, contradicted hypothesis, diminishing signal. Decision 0007 adds the synthetic BLOCK after 3× REQUEST_CHANGES. Rally §9's table has exactly these five rows, in the same order as drive's enumeration plus the decision-0007 synthetic at the end, with a `[unknown]` fall-through for unrecognised labels — honest rather than silent-absorb.

**3. Does the coherence scan still pass?** Yes (exit 0, re-run on tip of branch). All forbidden phrases absent; all required phrases present. The "recursive context separation" phrase that ac-07 requires appears once at §7c line 327.

**4. Was the dependency-note callout actually removed (ac-06 / ac-07 pre-condition)?** Yes. Diff line 327 replaces the dependency-note callout verbatim with the Recursive context separation callout. The Integration section (line 566) replaces its "when PR #6 merges" forward reference with the concrete citation range `0004-drive-verdict-contract` through `0007-drive-rerequest-budget`.

**5. Does progress.md accurately reflect the shipped state?** Yes, with a small inconsistency noted in findings — progress.md flips all three ACs and constraints #2/#8 from `[--]` to `[x]`, updates the verification-evidence block, and records `.orbit/choices/0009` as created. The mutation-matrix prose was dropped from the verification evidence in the follow-up — that was useful context for first-PR review and could arguably have been preserved, but is not required.

**6. Fresh-reader sanity on §7c.** A cold reader now gets (a) what recursive context separation is, (b) the four drive-internal contracts by decision ID and title, (c) a complete sub-agent brief including the `reason_label` vocabulary, (d) the exact launch-mechanics sentence naming `run_in_background: true`, `general-purpose`, and single-message dispatch. No forward references, no aspirational caveats.

---

## Findings

### [LOW] Stale "once PR #6 is live" caveat in §7c completion-handling prose

**Category:** edge-case (documentation coherence)
**Description:** The follow-up PR's stated scope is to eliminate PR-#6 forward references now that PR #6 has merged. §7c line 386 still reads:

> "3. Update rally.yaml for that card: `complete` on APPROVE, `parked` on escalation (with `parked_constraint` constructed per ac-14 once PR #6 is live)."

The parenthetical "once PR #6 is live" is now false-historical — PR #6 is live, and ac-14's label-mapping table is shipped in this same PR four sections below. A cold reader encountering this sentence would infer either (a) ac-14 is still aspirational, contradicting §9's actual table, or (b) rally is waiting on something that has already happened.

**Evidence:** `plugins/orb/skills/rally/SKILL.md:386` — verbatim quoted above. No other occurrences of "PR #6" / "once PR" / "become live" remain in the file (grep confirmed).

**Recommendation:** Replace the parenthetical with a direct §9 cross-reference, e.g. `(with parked_constraint constructed per §9's label-mapping table, ac-14)`. Trivial edit; does not block merge.

### [LOW] Dropped mutation-matrix evidence in progress.md verification block

**Category:** test-gap (reviewer ergonomics)
**Description:** The first PR's progress.md verification section included a 7-row mutation matrix documenting the coherence-scan's behaviour under each mutation (forbidden-present, required-absent, verb without citation, etc.). The follow-up PR's progress.md replaces that matrix with a prose paragraph. The scan itself still passes (ac-15 / ac-17 verified), but subsequent auditors lose the auditable mutation evidence without re-deriving it.

**Evidence:** Diff of `.orbit/specs/2026-04-19-rally-subagent-model/progress.md` removes lines 66–80 of the previous verification block (the "Mutation matrix" table) and the YAML parse check.

**Recommendation:** Optional. If the matrix was intentionally trimmed because "ac-15 verified in first PR" is the accurate claim, the current wording is fine. If the matrix was dropped inadvertently, restoring it costs nothing.

---

## Honest Assessment

This is a clean, small, correctness-focused follow-up PR that does exactly what it claims: lands the three ACs that were explicitly gated on drive-forked-reviews (PR #6) merging, adds the one decision record (0009) that depends on 0004–0007 existing on main, and flips the self-reported progress tracker accordingly. The changes are surgical — 125 lines added, 51 lines removed, all in three files — and every citation I spot-checked is accurate by title, filename, and semantic content. The §9 label-mapping table is genuinely complete against drive's actual escalation triggers, with an `[unknown]` fall-through that honours the spec's honesty principle (drift visible, not silently absorbed). The rally-coherence-scan still passes on the tip. The only finding of any substance is a single stale "once PR #6 is live" caveat at line 386 that should have been caught by the same cold-reader pass that rewrote §7c and §9; it is a LOW-severity documentation inconsistency that does not block merge but would tighten the artefact. Verdict: approve; fix the stale caveat in a follow-up commit or next PR.

**Verdict:** APPROVE
