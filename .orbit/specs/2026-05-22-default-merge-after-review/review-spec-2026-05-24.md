# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-22-default-merge-after-review
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | content signals (deployment, GitHub auth, repo settings) + Pass 1 mis-typing finding | 3 |
| 3 — Adversarial | not triggered (Pass 2 surfaced no cascading or untestable ACs) | — |

## Findings

### [HIGH] Six ACs default to `ac_type: code` but verify only SKILL.md prose
**Category:** test-gap
**Pass:** 2
**Description:** ac-01 through ac-07 omit `ac_type`, defaulting to `code` per METHOD.md. But every one of those ACs' verification clauses reduces to "the SKILL.md prose says X" (e.g. ac-01: "the SKILL.md prose orders the steps as commit-1 → … → spec close"; ac-03: "drive SKILL.md prose places the merge step inside the APPROVE branch only"; ac-04: "SKILL.md prose explicitly numbers the steps"). The actual change in this spec is prose-only — drive is a SKILL.md skill, no Rust / no test harness branch. ac-08 is correctly typed `doc`; the other six should be too.

The consequence: at `spec.close`, the close pre-flight will treat ac-01..07 as code ACs needing test/commit evidence and will block until each is ticked. The implementing agent will tick them based on "the SKILL.md change landed", which is exactly what `ac_type: doc` is designed to encode. Mis-typing here doesn't cause functional failure but adds friction and weakens the audit trail's signal.
**Evidence:** spec.yaml ac-01..07 carry no `ac_type` field (defaulting to `code`); ac-08 explicitly carries `ac_type: doc`; ac-01's verification text contains only SKILL.md prose checks plus one happy-path manual run. METHOD.md ac_type table: `doc` = "A written artefact (CLAUDE.md edit, card text, memo, MADR)".
**Recommendation:** Add `ac_type: doc` to ac-01, ac-02, ac-03, ac-04, ac-06, ac-08 (already doc). ac-05 and ac-07 could stay `code` if the spec wants to require a manual reproduction of the failure / crash scenario; otherwise type them `doc` too. ac-01's "happy-path manual run shows `gh pr view --json autoMergeRequest` returning non-null" is the one verification clause that genuinely needs a real run — consider splitting it into a paired `observation`-typed AC ("first full-autonomy drive under this spec captures `gh pr view --json autoMergeRequest` output and shows non-null").

### [HIGH] AC-07 recovery shape depends on a `drive.yaml.stage` distinction that doesn't exist
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-07 says drive "inspects drive.yaml.stage plus `gh pr view --json …` and detects the partial state (PR exists, autoMergeRequest is null). It re-attempts the merge step from the same PR." Today the drive resume table (SKILL.md L789-798) terminates at `stage: complete = "Already done — report status"`. There is no stage value that means "PR created, merge not attempted". Without one, the only thing the resume logic can do is rely entirely on `gh pr view --json autoMergeRequest`, with `drive.yaml.stage` providing no help (or actively misleading the resume table — if the drive crashed after `gh pr create` but before `orbit spec close`, `stage` is still `review-pr`, which the table routes back to Stage 3, not §Completion).

The spec doesn't say whether to introduce a new stage value (e.g. `merge-pending`) or extend the resume table to land at §Completion under specific conditions. The implementing agent will be guessing.
**Evidence:** SKILL.md L789-798 resume table has stages `review-spec | implement | review-pr | complete | escalated`. ac-07 references "drive.yaml.stage" as a discriminator but the spec defines no new stage and does not amend the resume table.
**Recommendation:** Either (a) add a sub-AC introducing a new stage value (e.g. `pr-created` between `review-pr` and `complete`) and updating the resume table, OR (b) rewrite ac-07 to drop the `drive.yaml.stage` clause and rely solely on the `gh pr view` inspection (the inspection alone is sufficient for the merge re-attempt). Pick one explicitly.

### [MEDIUM] AC-05's "PR-already-exists recovery prose pattern" doesn't appear to exist
**Category:** missing-requirement
**Pass:** 1
**Description:** ac-05 says the failure-branch shape "mirrors the existing PR-already-exists recovery prose pattern". A grep of plugins/orb/skills/drive/SKILL.md for `PR already`, `pr exists`, `gh pr view`, `already exists` returns zero hits — the named referent doesn't exist. The implementing agent will read this AC, search for the pattern, find nothing, and have to either invent the pattern or ask the author what was meant. Either path adds a cycle.
**Evidence:** grep across plugins/orb/skills/drive/SKILL.md for the named pattern returns no matches. The §Recovery section (L778-811) handles stage-based resumption but contains no PR-existence handling.
**Recommendation:** Either point to the actual reference (e.g. "mirrors §Recovery's stage-table pattern"), drop the comparative clause and describe the shape directly ("on non-zero exit: log → spec note → close-comment → spec close → exit 0"), or — if the pattern truly should exist as common substrate — add a sub-AC that creates it.

### [MEDIUM] AC-06's `merge state` value `merged` is unlikely to ever be observed at close-comment time
**Category:** assumption
**Pass:** 2
**Description:** ac-06 specifies three values for the merge-state field: `queued`, `merged`, `deferred-<reason>`. `gh pr merge --auto` enqueues the merge — actual completion waits on required checks. The close-comment is written immediately after the merge call, in the next step of §Completion. Unless the drive polls `gh pr view` after the merge call (which the spec doesn't ask for), the close-comment will write `queued` essentially every time — `merged` will never appear in the wild. ac-06 reads as if `merged` were a normally-reachable state.
**Evidence:** AC-06 enumerates `queued / merged / deferred-<reason>`; spec contains no polling step between `gh pr merge --auto` and `orbit spec close`. `gh pr merge --auto` documented behaviour: enables auto-merge, returns immediately, does not wait for checks.
**Recommendation:** Either (a) drop `merged` from the enumeration and document `queued` as the universal success case, OR (b) add an explicit polling step ("after merge, run `gh pr view --json mergeStateStatus` once to detect immediate-merge — synchronous merges of green PRs return non-`AUTO_MERGE`"). Option (a) matches the tabletop's "trust the queue" stance.

### [LOW] AC-04 introduces `git push` as a new step but frames it as pre-existing
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-04's ordering "(1) commit implementation, (2) propose card updates and commit, (3) push, (4) gh pr create, (5) gh pr merge --auto, (6) spec close" introduces step (3) push. The current §Completion (L533-597) contains no explicit `git push` step — the sequence today is commit-1 → commit-2 → "Create the PR" → close. AC-04 reads as if "push" were already there, but it isn't. No AC explicitly says "drive must run `git push` before `gh pr create`" — that requirement only lives implicitly inside AC-04's ordering clause.

Additionally, AC-04 does not say whether push happens at all autonomy levels (it should — `gh pr create` can't run without it). Minor scope clarification but worth nailing down.
**Evidence:** SKILL.md L533-557 §Completion enumerates 4 steps, none of which is push. AC-04 enumerates 6 with push as step 3.
**Recommendation:** Add a one-line clause to AC-04: "step (3) push runs at all autonomy levels — it is not full-only — and is a new explicit step not present in today's §Completion".

### [LOW] AC-05 and AC-06 write overlapping records on degradation
**Category:** test-gap
**Pass:** 2
**Description:** On merge degradation, AC-05 writes `orbit spec note <spec-id> "merge deferred — manual action required — <reason>"` and AC-06 surfaces `merge state = deferred-<reason>` in the close-comment payload. The two carry overlapping information; the spec doesn't say which is canonical or whether both should reference the same `<reason>` string. Risk is divergence (e.g. note says "auto-merge not enabled", payload says "deferred-no_automerge"). Minor.
**Evidence:** AC-05 and AC-06 both describe the deferral surface; neither references the other.
**Recommendation:** Add a one-line clause to AC-06: "the `deferred-<reason>` value uses the same `<reason>` string as the AC-05 spec note".

### [LOW] `gh pr ready` draft check from tabletop Q4 #4 was dropped without rationale
**Category:** missing-requirement
**Pass:** 2
**Description:** Tabletop Q4 #4 named "PR draft check before merge — `gh pr ready` if draft; at full autonomy a PR shouldn't be draft anyway" as an engineering-hygiene failure mode. This didn't survive into the ACs. The note "at full autonomy a PR shouldn't be draft anyway" suggests the author judged it redundant — fair. But if a future drive evolution opens drafts (e.g. opening a draft for early CI feedback), `gh pr merge --auto` will fail and the path falls into AC-05's graceful degradation. That's a safe failure, just an unexpected one. Worth a one-line acknowledgment.
**Evidence:** tabletop.md L56 enumerates the draft check; spec.yaml ac-01..09 contain no `gh pr ready` reference.
**Recommendation:** Either add a sub-AC ("drive does not call `gh pr ready` — relies on the convention that full-autonomy drives never open drafts; draft PRs fall through to AC-05 graceful degradation") OR drop the clause from the tabletop as deliberately out of scope. Currently neither — the tabletop and spec disagree silently.

---

## Honest Assessment

The spec is well-scoped, the tabletop reasoning is crisp, and the kill condition K1 with `ac_type: observation` is a textbook example of using the deferrable band correctly. The four-option-prompt preservation in ac-02 / ac-03 is well-aligned with the existing §Four-option verdict prompt section.

The biggest risk is not in the design — it's in the AC typing. Six ACs marked as `code` describe prose changes; the implementing agent will hit confusion at `spec.close` pre-flight when it tries to map "evidence" to "the SKILL.md change". This is a friction trap that doesn't endanger the work but does endanger the audit trail's clarity. Fix the `ac_type` fields and AC-07's stage-discriminator gap, address the dangling "PR-already-exists" reference, and this spec is ready for implement.

Recommended next move — author addresses the three HIGH/MEDIUM findings (ac-01..07 typing, ac-07 stage shape, ac-05 referent gap), drops a one-line clause on AC-04 (push universality), and ships.
