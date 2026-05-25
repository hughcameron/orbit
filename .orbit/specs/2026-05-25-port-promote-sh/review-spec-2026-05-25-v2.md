# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-port-promote-sh
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signal (cross-system version sequencing in AC-11) + LOW finding in Pass 1 | 1 |
| 3 — Adversarial | not triggered — Pass 2 surfaced no cascade or rollback concerns | — |

## What changed since v1

The v1 review (2026-05-25, REQUEST_CHANGES) raised one HIGH and three lower-severity findings. The revised spec resolves three of the four cleanly:

- **v1 HIGH (AC-11 silently embedded a schema change)** — resolved. The relations-edge writes on cards 0005/0006 are now explicitly deferred to a follow-up spec, both in the `goal` field and via deletion of the old AC-11. The new AC-11 is a clean test/canonicalise/CHANGELOG gate.
- **v1 MEDIUM (AC-05 dropped the `--root` flag)** — resolved. AC-05 now reads "the verb honours the CLI's standard `--root` flag for layout-root selection (matches the shim's `--root <path>` arg shape)".
- **v1 MEDIUM (AC-08 grep tally miscounted)** — resolved. AC-08 now states "Three SKILL.md files, six call sites per pre-flight grep: drive 3, card 2, rally 1", which matches live grep against the current tree (drive: 3, card: 2, rally: 1; six total).
- **v1 LOW (AC-12 CHANGELOG version pin)** — **partially resolved**. The spec now acknowledges the stack ("0.4.34 → 0.4.35, stacked on top of PR #32's 0.4.34 bump") but the underlying sequencing risk remains live — see new HIGH below.

The deferred-items language in the `goal` field, plus the explicit AC-11 about the deferred follow-up, is the right shape and matches the port-acceptance-shim precedent. The verb surface, dry-run contract, idempotency error, and the wrapper-vs-delete choice in AC-07 are all well-shaped.

## Findings

### [HIGH] Version sequencing assumes PR #32 has merged — it hasn't
**Category:** assumption
**Pass:** 1
**Description:** AC-11 specifies "bumped version (0.4.34 → 0.4.35, stacked on top of PR #32's 0.4.34 bump)". This requires that PR #32 (port-acceptance-shim, which performs the 0.4.33 → 0.4.34 bump) is merged into `main` before this spec lands, so the CHANGELOG and `plugin.json` already read 0.4.34 when this spec opens its edits.

Reality at review time:

- `gh pr view 32 --json state,mergedAt` returns `state: OPEN, mergedAt: null` — PR #32 has not merged.
- `plugins/orb/.claude-plugin/plugin.json` reads `"version": "0.4.33"`.
- `CHANGELOG.md` contains zero matches for `0.4.34` or `0.4.35` — the topmost entry is `[0.4.33] - 2026-05-24`.
- Most-recent git log on `main`: `fc279c4 Bump version to 0.4.33`. The two new commits on top (`c8872ff spec(...port-acceptance-shim)` and `d2e43f9 spec(...port-promote-sh)`) only ship spec files; neither bumps version.

Three failure modes:

1. **PR #32 lands cleanly first** → AC-11's "0.4.34 → 0.4.35" pin is correct. Best case.
2. **This spec's drive completes before PR #32 merges** → the implementer must either (a) bump 0.4.33 → 0.4.34 themselves (collides with PR #32's bump when it merges later) or (b) write a CHANGELOG entry under 0.4.35 when `plugin.json` still reads 0.4.33 (semver gap, broken release). Neither is recoverable cleanly.
3. **PR #32 changes shape during review and ships under a different version** → AC-11's pin is wrong silently.

The spec assumes case 1 without naming it as a precondition. The implement skill will hit this the moment it tries to close AC-11.

**Evidence:**
- `gh pr view 32 --json state,mergedAt` — `state: OPEN, mergedAt: null`.
- `plugins/orb/.claude-plugin/plugin.json` — `"version": "0.4.33"`.
- `head -10 CHANGELOG.md` — topmost entry `[0.4.33] - 2026-05-24`; no 0.4.34/0.4.35 entries exist.
- `git log --oneline -20` — no version bump commit after `fc279c4`.

**Recommendation:** Pick one of three:
- **(a)** Add an explicit precondition to the spec preamble or AC-11: "this spec opens after PR #32 merges; if PR #32 has not merged when implement starts, halt and escalate." Cheapest fix; matches the actual constraint.
- **(b)** Rewrite AC-11 to compute the version dynamically: "CHANGELOG.md entry under the next bumped version (`plugin.json` version + patch increment at implement start)". Removes the dependency on PR #32's number but loses precision.
- **(c)** Rebase this spec's drive onto PR #32's branch so the two ship together. Heaviest but eliminates the gap.

Option (a) is the cleanest signal — name the dependency, let drive's halt-guard catch it.

### [LOW] AC-08 grep regex catches prose mentions in card SKILL.md (Pass 1)
**Category:** test-gap
**Pass:** 1
**Description:** AC-08 verifies via `rg --no-heading 'promote\.sh' plugins/orb/skills/` returning zero lines. The current corpus has 6 matches in 3 files, but two of those are prose (card SKILL.md line 52 docs the `gate` field "propagates to bead AC as [gate] via promote.sh"; line 75 is a YAML-comment example with the same wording; drive SKILL.md line 113 is a prose sentence describing what `promote.sh` does). Three are actual shell-invocation call sites (drive:110, drive:735, rally:231).

For AC-09's "no orphaned wrapper" invariant, the prose mentions are harmless — they don't keep the shim alive. But the AC-08 grep treats them as failures, forcing the implementer to rewrite documentation prose alongside actual call-site rewrites. That's a docs nicety masquerading as a functional gate.

**Evidence:** `rg 'promote\.sh' plugins/orb/skills/` returns 6 matches; of those, only 3 are `$(plugins/orb/scripts/promote.sh ...)` shell invocations. The other 3 are documentation prose mentioning the script by name.

**Recommendation:** Acceptable as-is if the author wants the prose rewritten too (consistent post-port substrate). Otherwise tighten the regex to actual invocations: `rg 'plugins/orb/scripts/promote\.sh' plugins/orb/skills/` (which catches all three real call sites but skips the bare `promote.sh` prose mentions). Either choice works — name it explicitly so the implementer doesn't second-guess at close time.

### [LOW] AC-07 lets the implementer choose path (a) or (b) but doesn't capture the choice in the spec record (Pass 2)
**Category:** missing-requirement
**Pass:** 2
**Description:** Carried over from v1 — AC-07 offers two paths (compat-wrapper vs delete-with-rewrites), with (b) "preferred when a single PR can land verbs + rewrites + deletion cleanly". The spec's `acceptance_criteria` array doesn't carry a `notes:` or `path_chosen:` field to record which path was taken. Review-pr has to infer from the diff. Cheap fix: when implement closes AC-07, note the chosen path in a closing comment or label.
**Evidence:** AC-07 text.
**Recommendation:** Optional. If you want machine-checkable provenance, add a `labels:` entry like `path:b-delete-with-rewrites` when implement closes AC-07. Otherwise let review-pr read the diff. Not blocking.

---

## Pass 1 gate-AC text check (deterministic rules)

Three gate ACs in this spec: `ac-01`, `ac-06`, `ac-08`. All three descriptions are non-empty, not placeholder tokens (`TBD/TODO/FIXME/PLACEHOLDER/XXX/???`), and ≥20 chars trimmed. No MEDIUM finding from rule 5.

## Honest Assessment

The structural revisions from v1 are clean and effective. The schema-change bundling that made v1 a HIGH-severity REQUEST_CHANGES is gone; the `--root` flag is named; the grep tally matches reality.

What remains is a single live-environment risk: AC-11 pins a version (0.4.35) that assumes a sibling PR has already merged, and that PR is still OPEN as of this review. The implement skill will hit this the moment it tries to write the CHANGELOG entry. The cheap fix is one sentence naming the precondition; the more thorough fix is dynamic-version language in AC-11. Either way, the dependency should be explicit before implement starts, so drive's halt-guard catches the sequencing rather than the implementer discovering it mid-close.

Everything else is in good shape — the verb surface, the dry-run contract, the idempotency error, the wrapper-or-delete choice in AC-07, and the deferred-relations posture in the goal all match the port-acceptance-shim precedent that landed cleanly yesterday.
