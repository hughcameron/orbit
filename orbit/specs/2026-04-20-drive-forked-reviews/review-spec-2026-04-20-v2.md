# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** /home/hugh/github/hughcameron/orbit/specs/2026-04-20-drive-forked-reviews/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (cross-system boundary, state migration, schema change) + MEDIUM Pass 1 finding | 3 |
| 3 — Adversarial | structural concern surfaced in Pass 2 (budget mechanics inconsistency between ac-10/constraint #6 and ac-11) | 2 |

## Findings

### [HIGH] Internal inconsistency in REQUEST_CHANGES budget semantics

**Category:** constraint-conflict
**Pass:** 2
**Description:** The spec describes the 3-cycle REQUEST_CHANGES budget with two incompatible semantics. Implementation will have to pick one, and either choice violates the other AC.

- **Constraint #6** (spec.yaml line 9): *"After the 3rd REQUEST_CHANGES on a stage, drive does not launch a 4th review."* This says 3 REQUEST_CHANGES verdicts are received, and then the **next fork** (the 4th review) is suppressed and converted to synthetic BLOCK. That is: 3 actual reviews run; the 4th is synthetic.
- **ac-10** (spec.yaml line 67): *"When review_cycles.<stage> reaches 3 on a stage and the most recent review returned REQUEST_CHANGES, drive does not launch a 4th forked review for that stage. Instead, drive synthesises a BLOCK..."* Consistent with constraint #6 — 3 reviews run, the 4th is synthesised.
- **ac-11** (spec.yaml line 73): *"If a session dies mid-stage with review_cycles.review_spec == 2... simulate a 3rd REQUEST_CHANGES. Verify drive triggers synthetic BLOCK per ac-10."* This says the 3rd REQUEST_CHANGES **itself** is the trigger for synthetic BLOCK — i.e. only 2 actual reviews run before the 3rd cycle is synthesised.

The ac-10 description and ac-11 verification reach opposite conclusions about what "the 3rd REQUEST_CHANGES" means:

- If ac-10 is correct: counter goes 1 → 2 → 3 (all real REQUEST_CHANGES verdicts), then the 4th fork is suppressed. Three full review cycles execute.
- If ac-11 is correct: counter is 2 (two real REQUEST_CHANGES received), and drive synthesises BLOCK before running a 3rd review. Only two full review cycles execute.

ac-19's verification example is also inconsistent: *"Trigger a drive where review-spec returns REQUEST_CHANGES twice then APPROVE. Verify three files exist: `review-spec-<date>.md`, `review-spec-<date>-v2.md`, `review-spec-<date>-v3.md`."* That example has **3 reviews** (2 REQUEST_CHANGES + 1 APPROVE) producing 3 files. It does not reach the synthetic-BLOCK boundary, but it shows drive expects to run up to 3 reviews — consistent with ac-10, not ac-11.

**Evidence:** spec.yaml constraint #6; ac-10 (lines 66-68); ac-11 (lines 71-73); ac-19 (lines 115-117).

**Recommendation:** Pick one semantics and update the other AC to match. Suggested: keep ac-10/constraint #6/ac-19 (budget = 3 real reviews, 4th is synthetic BLOCK) and rewrite ac-11's verification example to match:

> *"Start a drive, let review-spec return REQUEST_CHANGES three times, then kill the session. Verify drive.yaml shows review_cycles.review_spec == 3. Restart and invoke drive; verify drive does not launch a 4th fork, synthesises BLOCK per ac-10, and re-enters at design."*

This also raises the implicit question of when `review_cycles` is incremented — before or after the synthetic-BLOCK check. ac-09 says the counter advances on a "successfully-parsed REQUEST_CHANGES verdict", which suggests increment-before-check. Worth stating explicitly so the implementation and both test scenarios agree.

---

### [MEDIUM] Brief must override review-spec/review-pr SKILL.md default save path; no AC guarantees the override is honoured

**Category:** missing-requirement
**Pass:** 2
**Description:** review-spec/SKILL.md line 123 says *"Save the review to: `orbit/specs/YYYY-MM-DD-<topic>/review-spec-<date>.md`"*. The forked agent, reading the SKILL.md cold, will follow this default path. Drive's cycle-ordinal scheme (ac-19) requires forks on cycles 2 and 3 to write to `-v2.md` / `-v3.md`. This override must come from drive's brief, but:

1. No AC requires review-spec/review-pr SKILL.md to mention that drive-brief paths take precedence over the documented default.
2. No AC verifies that the forked agent actually writes to the brief-specified path when it differs from the SKILL.md default.
3. ac-04 and ac-05 verify that drive's brief *mentions* the cycle-specific path, but not that the resulting file lands at that path.

If the forked agent follows the SKILL.md default, drive will find no file at `-v2.md`, trigger retry (per ac-07), the retry will do the same thing, and drive will escalate — not because the review failed, but because of a brief/skill path conflict. This failure mode is entirely invisible from the spec's ACs.

**Evidence:** review-spec/SKILL.md line 123; review-pr/SKILL.md line 113; spec.yaml ac-04, ac-05, ac-19.

**Recommendation:** Either

- Add an AC to ac-01's group: "review-spec/SKILL.md and review-pr/SKILL.md state that when invoked as a forked Agent the brief-provided output path takes precedence over the default save path documented in §6/§7", **or**
- Amend ac-17's verification to cover the multi-cycle case: "Run a drive that hits REQUEST_CHANGES once on review-spec then APPROVE. Verify review-spec-<date>-v2.md exists at the drive-computed path with an APPROVE verdict written by the forked agent."

Without one of these, ac-19 is untestable end-to-end — it only verifies drive's internal path computation, not the contract with the forked reviewer.

---

### [MEDIUM] drive.yaml schema change not reflected in downstream consumers

**Category:** missing-requirement
**Pass:** 3
**Description:** ac-21 adds `review_cycles` to the drive.yaml schema on fresh creation. ac-20 treats its absence as a refusal trigger. The spec does not call out that other code reading drive.yaml must be updated (or verified unchanged):

- `session-context.sh` (named in ac-16 as surfacing active drives) reads drive.yaml.
- rally SKILL.md (ac-18) may introspect drive.yaml for sub-agent state.
- The drive skill's §11 Resumption reads drive.yaml extensively — ac-20 covers one resumption path but the resumption logic in §11 isn't explicitly updated to read review_cycles on every resumed stage, only on entry.

**Evidence:** spec.yaml ac-20, ac-21; drive/SKILL.md §11 (resumption table does not yet include review_cycles); no AC touches session-context.sh or other consumers.

**Recommendation:** Add an AC (doc-type) requiring `drive/SKILL.md §11 Resumption` to include review_cycles in its state-reconstruction steps, with specific behaviour when the field is absent (per ac-20) vs present. Optionally add a verification step to ac-21 confirming that session-context.sh still parses the extended drive.yaml without error (or explicitly document that no change is needed).

---

### [MEDIUM] Retry semantics unclear when a stale review file pre-exists at the cycle-specific path

**Category:** failure-mode
**Pass:** 2
**Description:** ac-07 says the retry "writes to the same cycle-specific path, overwriting any partial file from the failed attempt." But consider the resumption scenario:

- Session crashes after the forked agent wrote a file containing a valid verdict (e.g. APPROVE) but before drive parsed it and updated drive.yaml. On resumption, drive sees counter unchanged (increment happens on successful parse per ac-09) and status still at review-spec; it will then launch a fresh fork for the same cycle, overwriting the good file.
- Conversely, if the prior fork wrote a malformed file, drive's retry per ac-07 launches a fork. If the retry also writes malformed, drive escalates — but the first post-crash fork is effectively the retry of the pre-crash fork, from drive's point of view it's the "first" fork after resume. So drive gets 2 more attempts post-resume, potentially 3 real fork attempts for the same cycle.

Neither behaviour is wrong per se, but the spec doesn't say which one drive picks or whether post-crash resumption counts as a retry (ac-07 budget = 1) or a fresh first-fork.

**Evidence:** spec.yaml ac-07, ac-09, ac-11; no AC addresses "file exists at cycle path on resumption."

**Recommendation:** Add a clarifying sentence to ac-07 or ac-11: either "On resumption, drive treats any pre-existing file at the cycle path as authoritative and attempts to parse it before launching any fork — if parseable, drive uses that verdict; if not, drive launches a single retry fork per the ac-07 budget" or "On resumption, drive ignores pre-existing files at the cycle path and launches a fresh first-fork, getting the full ac-07 retry budget." The former is safer (idempotent resumption); the latter is simpler.

---

### [LOW] ac-18 verification lives outside this spec's shippable scope

**Category:** test-gap
**Pass:** 3
**Description:** ac-18 is a `gate`-type AC verifying that rally's SKILL.md no longer contains parallel-specific review plumbing after the rally refinement spec is updated. This cannot be verified by implementing *this* spec alone — it depends on a separate future spec being shipped and rally/SKILL.md being edited. The exit condition "End-to-end drive on a test card passes ac-17" gates merge on ac-17 but ac-18 is unverifiable at merge time.

**Evidence:** spec.yaml ac-18 (lines 109-111); exit_conditions (lines 157-164).

**Recommendation:** Either reclassify ac-18 as a `doc`-type follow-up note in the PR description (covered by ac-16 expansion), or explicitly mark it as a post-ship verification in the exit_conditions — e.g. "ac-18 is verified separately when the rally refinement spec ships; not a merge gate for this spec." As written, it risks either blocking merge on an unrelated spec or being silently ignored.

---

### [LOW] Structural scan: synthetic BLOCK constraint string is long and must be exact

**Category:** test-gap
**Pass:** 1
**Description:** ac-10 specifies the synthetic BLOCK constraint string verbatim: *"review converged on REQUEST_CHANGES after 3 iterations; findings have not been addressable within budget"*. ac-15 requires drive/SKILL.md to document this exact string. A typo between the two locations will silently drift — no AC cross-checks the strings are byte-identical.

**Evidence:** spec.yaml ac-10, ac-15.

**Recommendation:** Add a verification step to ac-15: "Grep drive/SKILL.md for the exact string specified in ac-10; a byte-level match is required." Low severity because the test is trivial to add and the risk is purely cosmetic, but easy cleanup.

---

### [LOW] Date-boundary edge case in cycle-specific path computation

**Category:** failure-mode
**Pass:** 2
**Description:** ac-19's path format includes `<date>`. If a drive is long-running (e.g. overnight) and review-spec cycle 1 produces `review-spec-2026-04-20.md` but cycle 2 is computed after midnight, drive might produce `review-spec-2026-04-21-v2.md`. That breaks ac-19's implicit assumption that same-stage same-day cycles share a date prefix.

**Evidence:** spec.yaml ac-19.

**Recommendation:** Specify that `<date>` is computed once per stage (at cycle 1) and reused for subsequent cycles of the same stage. Or accept the drift — a mixed-date naming scheme is readable, just slightly surprising. Minor.

---

## Honest Assessment

The spec is well-structured, well-motivated, and the core fork-and-parse architecture is sound — it honestly addresses an architectural inconsistency (drive violating its component skills' `context: fork` contracts) and picks a minimal, defensible implementation path. Decision-surfacing is strong: 4 decisions are explicitly flagged for `orbit/decisions/` capture, and the constraints section reads like a contract rather than aspirational text.

The biggest risk is the **HIGH budget-semantics inconsistency** between ac-10/constraint #6 (3 reviews, 4th synthetic) and ac-11's verification example (2 reviews, 3rd synthetic). An implementer following ac-10 will fail ac-11's test; an implementer following ac-11 will fail constraint #6 and ac-19's 3-file example. This must be resolved before implementation; the fix is mechanical (rewrite ac-11's example) but the choice matters.

The **MEDIUM brief-vs-SKILL.md path contract** gap is the second-most-likely failure mode in practice — nothing in the spec's ACs would catch a forked agent that ignores the brief's path and honours its SKILL.md default, yet that is exactly the kind of contract drift that will bite on cycles 2 and 3. Adding one AC linking the skill's output section to the brief-override behaviour closes the gap cleanly.

The remaining findings are smaller: a schema-change blast radius check, a resumption edge case around pre-existing cycle files, a scope question on ac-18, and two LOW hygiene notes. None are blocking individually, but the HIGH finding prevents approval.

Once the budget-semantics ambiguity is resolved and the brief-vs-skill-default path contract is made explicit, this spec is ready to implement. The design is honest, the scope is clean, and the downstream simplification of rally (ac-18) is a genuine win — worth the ~1-2 hour revision to land it cleanly.
