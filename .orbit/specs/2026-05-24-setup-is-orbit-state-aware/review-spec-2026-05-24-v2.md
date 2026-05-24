# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-24-setup-is-orbit-state-aware
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | content signals (schema change, cross-drive boundary, history-preserving migration); MEDIUM findings in Pass 1 | 1 |
| 3 — Adversarial | not triggered | — |

The v2 spec absorbs the bulk of v1's findings. The AC count grew from 9 to 18; the rally-locked scope (5th state `wrapped-undotted`, single-rename `git mv`, shared classifier in orbit-state, `plugin_repo` flag with default false, move-and-warn `decisions/` handling, `decisions-md-unmigrated` conformance finding, 6-row state table, SKILL.md description-line fix, repo-self `plugin_repo: true` edit, parity test) is now reflected in ac-06, ac-10..ac-18. v1's HIGH findings (#1 missing rally scope, #2 ac-06 contradiction) are closed. v1's MEDIUM #4 (`OrbitConfig` parse contract) is adequately covered by ac-12's "optional...defaulting to false" plus ac-17's verification. v1's LOW (#6 topology in greenfield) is closed by ac-03's "Topology scaffolding follows the plugin_repo gate (ac-12)".

Two gaps from v1 remain, and one new gap surfaces from Pass 2.

## Findings

### [MEDIUM] No AC asserts refusal behaviour on `mixed-bare` or `mixed-undotted`

**Category:** missing-requirement
**Pass:** 1
**Description:** ac-10 enumerates `mixed-bare` and `mixed-undotted` as rows in the 6-row state table, but neither row's *runtime behaviour* is testable from the ACs alone. The decisions doc (§2 ¶51, §6 ¶131-141) and interview Q6 lock both states as **refuse states** with specific contracts — a message naming the collision, no filesystem mutation, exit non-zero. An implementer could ship ac-10 by listing the two rows in the SKILL.md table without any code path actually refusing — the table-row check passes, the runtime check has no AC. This is v1 review finding #5 carried forward; the v2 spec adds the table-row coverage but not the runtime-refusal coverage.

**Evidence:** `spec.yaml` ac-10 description names only the table contents (`§1 lists exactly six states`). `decisions.md:51` ("If `.orbit/` already exists alongside `orbit/`, refuse with a 'mixed-undotted' error"). `interview.md:65-66` (state-table rows annotated "refuse"). No AC string in `spec.yaml` contains the word `refuse` in connection with `mixed-bare` or `mixed-undotted` (ac-01 / ac-05 / ac-13 use `refuse` for unrelated states).

**Recommendation:** Add one AC covering both refuse states. Suggested phrasing: *"Setup refuses on `mixed-bare` and `mixed-undotted` with a message naming both substrate paths and the collision; no filesystem mutation occurs (`git status --porcelain` compares equal pre- and post-invocation); exit non-zero. Verifiable by integration test against fixtures for each shape."* Mark it gated if the rally treats categorical refusal as load-bearing — `mixed-undotted` in particular is the failure mode `git mv orbit .orbit` would otherwise blow up on with an opaque git error.

### [MEDIUM] Cross-drive suppression assumption is wired through ac-11 only, not asserted as a post-condition

**Category:** assumption
**Pass:** 1
**Description:** v1 finding #3 (cross-drive `undotted_substrate` suppression dependency) is partially addressed — ac-11 names the sister drive (`2026-05-24-workflow-conformance`) and the shared classifier helper, so the *shape-sharing* contract is explicit. But the *suppression* contract — "after `wrapped-undotted` migration completes (or during it), `canonical-files-missing` does not fire" — is still implicit. Interview lines 25-26 and 142-144 name the suppression as a design assumption this drive carries. If the sister drive ships the helper but not the suppression (or the suppression has a different trigger condition), `wrapped-undotted` repos hit `canonical-files-missing` during the migration window and every audit pass between `git mv orbit .orbit` and `.orbit/METHOD.md` landing — exactly the failure mode the suppression exists to prevent. The spec's `cards:` field lists only `0017-...`; no `relations:` wire to the sister drive's spec id.

**Evidence:** `spec.yaml` carries no AC string containing `canonical-files-missing` or `suppression`. `interview.md:25-26` ("the sister drive on card 0039 ships an `undotted_substrate` conformance finding that suppresses canonical-files-missing — this drive's design assumes that suppression exists"). `decisions.md:7` ("These decisions assume that suppression exists.").

**Recommendation:** Add an AC stating the dependency as a verifiable post-condition. Suggested phrasing: *"`orbit audit conformance` does not fire `canonical-files-missing` against a `wrapped-undotted` repo during or after migration — relies on the sister drive's `undotted_substrate` finding suppression at `orbit-state/crates/core/src/verbs.rs::audit_conformance_at`. Verifiable by integration test fixture that classifies a `wrapped-undotted` shape, runs `orbit audit conformance --json`, and asserts no `canonical-files-missing` finding in the output."* Alternatively, add a sister-spec relation to the spec record so drive's input-contract surfaces the dependency.

### [MEDIUM] ac-06 covers prompt-before-migrating for `wrapped-undotted` only; `brownfield-bare` prompt-and-migrate behaviour has no AC after the rename

**Category:** test-gap
**Pass:** 2
**Description:** ac-10 renames `brownfield` → `brownfield-bare`, ac-14 covers `decisions/` migration in the `brownfield-bare` case, but no AC asserts the full prompt-and-migrate runtime behaviour for the `brownfield-bare` state itself (the prompt text, the per-subdir `git mv` block, the untracked-residue scan, the dirty-tree tolerance). Today's SKILL.md §3 covers `brownfield-bare` (under its old name `brownfield`) — the implementation can preserve that block verbatim while only renaming the state slug. That's likely the intent, but the spec doesn't say so. An implementer doing a careful read of just the ACs could conclude the `brownfield-bare` block is also being rewritten and accidentally diverge from the working code. Pass-1's structural scan rated this lower because there's an existing working implementation to preserve; Pass-2's "what happens when this assumption is wrong" analysis raises it: if the implementer treats the rename as a rewrite trigger, the brownfield-bare runtime contract drifts.

**Evidence:** `spec.yaml` ACs ac-06 (wrapped-undotted prompt) and ac-14 (decisions/ migration mechanism) are the only ACs naming brownfield migration runtime behaviour. The existing `brownfield`-state block at `plugins/orb/skills/setup/SKILL.md:48-98` is not name-checked as "preserved verbatim except for state slug" in any AC. `interview.md:124-125` ("§3 brownfield migration block (lines 48-98): extended to handle `wrapped-undotted` via the single-rename path; existing bare-dir path preserved") states the intent in interview prose only.

**Recommendation:** Add a one-line AC pinning the preservation contract. Suggested phrasing: *"The existing `brownfield-bare` migration block at `plugins/orb/skills/setup/SKILL.md:48-98` is preserved structurally (prompt text, per-subdir `git mv` enumeration, residue scan, dirty-tree tolerance); only the state slug rename (`brownfield` → `brownfield-bare`) and the `decisions/` migration-target update (ac-14) apply."* Alternatively, fold the preservation note into ac-10's description as an explicit clause.

### [LOW] ac-17 verification mechanism is grep-only; idempotency on re-run is not asserted

**Category:** test-gap
**Pass:** 1
**Description:** ac-17 (`plugin_repo: true` in this repo's `.orbit/config.yaml`) names its verification as "grepping the config file for the literal `plugin_repo` true line". That works once, but says nothing about idempotent behaviour if `/orb:setup` runs in this repo after the edit lands. Does setup notice the flag and skip writing? Does it overwrite or re-confirm? The greenfield/idempotent path runs through ac-03 + ac-12, and ac-07 covers "fully-initialised" idempotency at the substrate-layout level — but neither speaks to config-file idempotency once the flag is set.

**Evidence:** `spec.yaml` ac-17 names verification mechanism; ac-07 covers fully-initialised no-op without naming config content. `decisions.md:87` ("The one-time cost of setting the flag on this repo is paid once; the safety property holds forever.") implies idempotency intent but doesn't name the mechanism.

**Recommendation:** Optional — add a clause to ac-07 or ac-12 specifying "if `.orbit/config.yaml` already contains `plugin_repo: true`, setup does not modify the file". Pass-1 rated this low because the typical idempotency boilerplate at the file-write layer covers it; flagging only because the `plugin_repo` field is the load-bearing safety property and silent overwrites would risk regressing the flag.

---

## Honest Assessment

The v2 spec is materially closer to implementation-ready than v1. The rally-locked scope is now reflected in concrete, testable ACs; the v1 HIGH findings are both closed; the v1 schema-parse and topology-scaffolding MEDIUM/LOW findings are absorbed cleanly into ac-12 and ac-03. The substrate is in good order: 18 ACs, 7 gates, clean coverage of the classifier helper, parity tests, repo-self config edit, and the conformance-finding cross-drive surface.

The remaining gaps are smaller and concentrated in two places: (1) the **refuse states** (`mixed-bare`, `mixed-undotted`) have table-row coverage but no runtime-behaviour AC, and (2) the **suppression assumption** wired through the sister drive (`canonical-files-missing` non-firing during the migration window) is named in interview/decisions but not asserted as a post-condition. Both are testable additions, not redesign requests. The Pass-2 finding on `brownfield-bare` preservation is a small clarification that prevents an implementer-side over-interpretation of the rename.

Biggest risk is the suppression dependency: it's the one finding that couples this drive's correctness to another drive shipping the right shape, and the coupling is currently review-time tribal knowledge rather than substrate. Add it as an AC (or a `relations:` wire to the sister spec id) and the spec is ready to implement.

Once these three MEDIUM findings are addressed (the LOW is optional polish), the spec is implementation-ready.
