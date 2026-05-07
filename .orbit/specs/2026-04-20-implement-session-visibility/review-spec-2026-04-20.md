# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-20-implement-session-visibility/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

```
| Pass | Triggered by                                                                 | Findings |
|------|------------------------------------------------------------------------------|----------|
| 1 — Structural scan     | always                                                      | 2        |
| 2 — Assumption & failure| content signals: cross-card shared surfaces, shared parser, |          |
|                         | session hook modification, dependency on 0009               | 4        |
| 3 — Adversarial         | not triggered (Pass 2 findings are correctable, no cascade) | —        |
```

---

## Findings

### [HIGH] Shared-parser ownership claim is inconsistent with the upstream spec

**Category:** missing-requirement
**Pass:** 2
**Description:** The interview (D4 / Open Questions) and this spec's ac-08 both assume `parse-progress.sh` (or an equivalent labelled awk block in `session-context.sh`) is introduced as a shared helper consumed by both card 0003's resume reconcile and card 0009's "next unchecked AC" surfacing. The spec lists mission-resilience as `depends_on` so 0009 "must merge before this card so the schema is present … and the shared parser is available to reference."

However, `.orbit/specs/2026-04-20-mission-resilience/spec.yaml` does not contain any AC, constraint, or deliverable that introduces a shared progress.md parser. Mission-resilience's ac-09 imposes parser *discipline* (detours ignored), and ac-08 / ac-10 impose output requirements on `session-context.sh`, but none of its ACs require a standalone `parse-progress.sh` script or a labelled shared awk block inside `session-context.sh` that is callable from elsewhere. Its exit conditions do not include "shared parser exists."

Consequence: this spec declares a hard dependency on an artefact the upstream spec does not actually own. If 0009 ships as written, this spec's ac-08 fails because the expected helper never lands. The phrasing "OR an equivalent well-named awk block inside session-context.sh" in ac-08 does not rescue the claim — an awk block inside `session-context.sh` is not externally callable (you cannot `source` a bash file to get an awk function), so ac-08 verification (d) "grep-assert both code paths source or call the helper" is unsatisfiable in that branch.

**Evidence:**
- spec.yaml lines 55–57 (ac-08) and 111–112 (depends_on + depends_on_note)
- mission-resilience spec.yaml ACs (no shared-parser AC); exit_conditions (no shared-parser condition)
- interview.md lines 98–99 Open Questions: "Propose a single shared parser, owned and authored under 0009"

**Recommendation:** Pick one of:
1. Add a 13th AC to the mission-resilience spec that explicitly creates `plugins/orb/scripts/parse-progress.sh` (or equivalent standalone helper) and require it to be sourced by `session-context.sh`. Rebase this spec's `depends_on_note` to cite that AC.
2. Move shared-parser ownership into this spec — author `parse-progress.sh` here as part of ac-08, and downgrade the depends_on_note to cite only schema consumption. This makes ac-08 a first-party deliverable rather than a downstream consumption claim.
3. Drop the "shared parser" framing and let each consumer use its own parse logic (less good — conflicts with the stated evaluation principle "Shared parser lives once").

Without one of these, ac-08's "both code paths source or call the helper" cannot be satisfied byte-for-byte.

---

### [MEDIUM] TaskCreate sequencing vs mission-resilience pre-AC check sequence is unspecified

**Category:** assumption
**Pass:** 2
**Description:** Mission-resilience's constraints declare a fixed pre-AC check sequence: "(1) backfill Spec hash if absent (ac-12), (2) drift check if hash present and differs (ac-02), (3) gate enforcement walk (ac-06)". This card's ac-01 adds task emission "after writing progress.md in step 4". The two do not interleave cleanly:

- On a fresh `/orb:implement` run, progress.md is freshly written in §4, then tasks are emitted. The pre-AC check sequence runs before the first AC starts. Where does TaskCreate sit relative to that sequence? The spec does not say.
- On a session where progress.md already exists (resume), the resume reconcile (ac-03) runs from `session-context.sh`. But `session-context.sh` also runs mission-resilience's drift check and next-AC surfacing. If drift is detected (ac-03 of mission-resilience prints the notice), the resume reconcile's "rebuild from progress.md" may rebuild tasks against a stale progress.md whose Spec hash is about to be overwritten by the interactive ac-02 acknowledge path. The tasks would then reflect the old AC list if the acknowledged spec has different ACs.
- In the non-interactive path (mission-resilience ac-02: exit status 1), the implement skill halts before §4d. Does ac-01 still fire? The spec doesn't say.

The two specs are claimed to layer "additively" but the runtime call graph (who runs first, who blocks whom) is under-specified.

**Evidence:**
- spec.yaml ac-01 "After writing progress.md in step 4, the /orb:implement skill emits one pending task…" (line 20)
- mission-resilience spec.yaml constraint "Pre-AC check sequence is fixed: (1) backfill … (2) drift check … (3) gate enforcement" (line 18)
- mission-resilience spec.yaml ac-02 non-interactive path: exit status 1 before any AC starts (lines 28–29)

**Recommendation:** Add a constraint naming where §4d's TaskCreate loop sits in the fresh-run sequence (most natural: after §4 progress.md write, before the first AC's pre-AC check; tasks emission depends only on the parsed progress.md file). On resume, state whether the resume reconcile runs before or after mission-resilience's drift notice, and what the reconcile does when drift is detected but not yet acknowledged. Add a test fixture for the "drift detected on resume" interaction.

---

### [MEDIUM] Resume rebuild uses TaskDelete semantics that are nowhere declared

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-03 says "full-rebuilds (delete + recreate the filtered tasks from scratch)". ac-04 verification asserts spec-A tasks were "rebuilt (TaskCreate × 3)" after drift, implying the pre-existing 3 tasks were removed. But the spec's ontology_schema only mentions `task_emission` via TaskCreate and the TaskUpdate rule — no TaskDelete operation is described. The constraint set says "TaskCreate (pre-flight) and TaskCreate/TaskUpdate (resume rebuild) derive from the parsed file" (line 7), explicitly omitting TaskDelete.

If the rebuild is implemented as "TaskUpdate all stale tasks to cancelled + TaskCreate fresh ones", ac-04 verification holds (no TaskDelete needed). If implemented as an actual TaskDelete, it contradicts constraint line 7 which enumerates only TaskCreate and TaskUpdate for resume. Either way the spec is internally inconsistent between the constraint list, the ontology, and the AC text.

**Evidence:**
- Constraint line 7: "TaskCreate (pre-flight) and TaskCreate/TaskUpdate (resume rebuild) derive from the parsed file" (no TaskDelete)
- ac-03 description line 30: "full-rebuilds (delete + recreate the filtered tasks from scratch)"
- ac-04 verification line 36: "the 2 spec-B tasks were untouched (no TaskUpdate/TaskCreate/TaskDelete on them)" — names TaskDelete, but only in a negative assertion
- ontology `task_emission` (line 62) and `resume_reconcile` (line 68) — neither describes a deletion operation

**Recommendation:** Either (a) add TaskDelete (or TaskUpdate status=cancelled) to the constraint list and ontology, explicitly stating how stale tasks are disposed, or (b) if the tool primitive is `TaskUpdate status=cancelled` rather than a hard delete, spell that out in ac-03's description and in a new ontology field. Update ac-04 verification to check the disposition of the stale spec-A tasks, not only the count.

---

### [LOW] ac-01 verification does not scope by metadata.spec_path before counting tasks

**Category:** test-gap
**Pass:** 1
**Description:** ac-01 verification says "TaskList returns 7 new pending tasks". In a real session, TaskList returns every task in the session, including non-orbit-implement tasks the user or other skills may have created. The test asserts task count as 7 but does not scope the count to `metadata.spec_path == <fixture spec path>`. A fresh simulation may give a clean count, but the fixture contract is weaker than it should be given the rest of the spec's emphasis on spec-path scoping (ac-03, ac-04).

**Evidence:** spec.yaml lines 20–21 (ac-01 and verification).

**Recommendation:** Tighten verification to "TaskList filtered by metadata.spec_path == <fixture> returns 7 new pending tasks, and no tasks without that tag exist in the fixture." Aligns with the ac-04 pattern and prevents a future test from masking a missing-tag bug.

---

### [LOW] ac-07 diff-baseline "pre-card-0009 merge" is ambiguous

**Category:** test-gap
**Pass:** 1
**Description:** ac-07 verification (d) says "Negative: diff against main (pre-card-0009 merge) must NOT alter §1, §2, §3, §4a, §4b, §4c — only adds §4d and §5 rules. Assert via a line-count check on the unchanged sections."

The spec's `depends_on` requires 0009 to merge first. Once 0009 is merged, "main" includes 0009's additions to §4 (Spec hash line, Current AC, ## Detours, gate annotation). So the diff baseline for this card is main-after-0009, not main-before-0009. The verification phrasing "pre-card-0009 merge" is the opposite of the dependency order. Description text already says "the shipped skill plus card 0009's additions" but the verification test name contradicts it.

**Evidence:** spec.yaml lines 50–51, cross-referenced with lines 111–112 (`depends_on`).

**Recommendation:** Reword ac-07 verification (d) to: "diff against main (post-card-0009 merge) must NOT alter §1, §2, §3, §4a, §4b, §4c — only adds §4d and §5 rules." Or equivalently: "the sections present before this card starts (i.e. main + 0009) must be byte-identical after this card merges, except for the new §4d block and the three new §5 rules."

---

### [LOW] Canonical "out of sync" warning is not declared as a single-source constant

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-03 specifies the warning string "orbit: task list out of sync with progress.md, rebuilt from scratch" verbatim. Mission-resilience takes the stronger position for its canonical drift-notice by declaring it as a single named constant in `implement/SKILL.md` that `session-context.sh` includes literally, and mandates that any change to the wording routes through the schema-owning card. This card does not impose the same single-source discipline on its warning, even though the same failure mode (string diverges between the hook and the skill that wrote it) applies.

Lower severity because this warning is not parsed by any downstream tool (unlike the drift-notice, which fixtures grep for) — it is authored once in `session-context.sh` and surfaces to stderr for humans.

**Evidence:** spec.yaml ac-03 lines 30–31; contrast mission-resilience spec.yaml constraint on drift-notice lines 10–11.

**Recommendation:** Either (a) add a constraint declaring the warning string is authored once in `plugins/orb/scripts/session-context.sh` (or a shared header if one exists) and that test fixtures grep the source of truth, or (b) downgrade the ac-03 verification assertion from "warning string appeared verbatim" to "any non-empty stderr warning appeared" to weaken the literal contract. Option (a) is preferred for consistency with the mission-resilience discipline.

---

## Honest Assessment

This spec is thorough, well-constrained, and internally coherent at the individual-AC level. The verifications are specific, the constraint list is unusually rigorous for a cross-card layering exercise, and the depends_on note signals awareness that the card cannot ship alone.

The dominant risk is at the seam with mission-resilience. The spec claims a shared parser will exist, but the upstream spec does not actually own that deliverable per its own AC list (HIGH finding). If both specs merge as written, this spec's ac-08 cannot be satisfied. The TaskCreate-vs-pre-AC-check sequencing question (MEDIUM) is a related seam problem — the two specs are layered independently but their runtime interleaving is never stated. Both are fixable with text changes; no architectural rework is needed. The remaining findings are tightening recommendations.

The biggest risk, if shipped without changes, is discovering during implementation that the "shared parser" slot does not exist and having to decide mid-implementation whether to grow this spec's scope or bounce back to mission-resilience. Addressing the HIGH finding before implement begins is cheap now and expensive later.
