# Spec Review

**Date:** 2026-05-01
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-05-01-bead-native-cold-fork-reviews/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 9 |
| 2 — Assumption & failure | content signals (cross-system boundaries, shared-parser config, hard cutover with in-flight drives) AND >0 MEDIUM Pass-1 findings | 6 |
| 3 — Adversarial | not triggered (no cascading failure / untestable AC / unknown impact radius) | — |

## Findings

### [HIGH] Substrate parity claim is false for promoted beads — gates do not survive promotion

**Category:** failure-mode
**Pass:** 2
**Description:** The spec's central goal is "rule-coverage parity" with the spec.yaml-based reviewers, anchored on the gate-AC verification check (Q3 / ac-05 / ac-08). But `plugins/orb/scripts/promote.sh` lines 113-118 generate AC lines as `- [ ] ac-NN: <name> — <then_clause>` — there is no `[gate]` token emitted. Cards have a flat scenario list with no per-scenario gate flag. So every bead created via `promote.sh` from a card today has zero gate ACs, regardless of how many gate scenarios the card declared. The Pass-1 gate-AC verification check (rewritten by ac-05 to fire on `is_gate=1` rows) will silently no-op for production beads — the same failure mode as the orbit-6da.2 snapshot bridge that this card was supposed to fix.
**Evidence:**
- `plugins/orb/scripts/promote.sh:113-118` — AC template `f'- [ ] {ac_id}: {name} — {then_clause}'` (no `[gate]` interpolation).
- Card schema (e.g. `.orbit/cards/0016-bead-native-cold-fork-reviews.yaml`) — scenarios have `name`, `given`, `when`, `then` only; no `gate: true` field.
- ac-08 fixture cleverly works around this by piping a hand-crafted acceptance string into `parse-acceptance.sh acs --stdin` — it never exercises a `bd`-promoted bead.
- `.orbit/conventions/acceptance-field.md:55-63` shows gates in worked examples but doesn't specify how cards encode "this scenario is a gate."
**Recommendation:** Add an AC that either (a) extends `promote.sh` to read a `gate: true` field on card scenarios and emit `[gate]`, plus an AC that updates the card schema and at least one card to demonstrate, or (b) explicitly scope-limits the spec by acknowledging that gate-AC parity holds only for hand-edited bead acceptance fields and re-states the goal so it doesn't claim parity for the promote → review path. Without one of these, the spec ships the same silent no-op it claims to fix.

### [MEDIUM] Decision number 0012 may collide; existing 0011 is already double-occupied

**Category:** missing-requirement
**Pass:** 1
**Description:** ac-09 requires creating `.orbit/choices/0012-bead-acceptance-field-as-cold-fork-substrate.md`. But `.orbit/choices/` already has two files numbered 0011: `0011-beads-execution-layer.md` and `0011-design-intent-not-means.md`. The numbering convention is already broken; ac-09 hard-codes 0012 without a check or recovery rule.
**Evidence:** `ls .orbit/choices/` returns `0011-beads-execution-layer.md` and `0011-design-intent-not-means.md` — two distinct decisions sharing the 0011 prefix. There is no AC to resolve the existing 0011 collision before adding 0012.
**Recommendation:** Either (a) renumber one of the existing 0011 files as a prerequisite AC, or (b) reword ac-09 to "next available decision number" and add a verification step that picks the lowest unused integer. Hard-coding 0012 in ac-09 risks colliding with whatever the second 0011 was supposed to become, or with a parallel in-flight decision.

### [MEDIUM] Worked example block in drive SKILL.md still writes a snapshot — no AC removes it

**Category:** missing-requirement
**Pass:** 1
**Description:** Drive's `## Worked example` section (drive SKILL.md lines 754-814) at lines 787-790 contains `SNAPSHOT="orbit/reviews/$BEAD/bead-snapshot-$(date -I).md"` and the comment `# write snapshot per §1.1`, plus line 802 `# (mirrors stage 1, brief includes git diff main...HEAD + snapshot path)`. ac-01 deletes §1.1 and §3.1; ac-07 updates Completion's commit-1 description. Neither AC touches the Worked example. After the spec ships, the Worked example will reference a §1.1 that no longer exists and document a workflow that is no longer the source of truth.
**Evidence:** drive SKILL.md lines 787-790 and 802-803.
**Recommendation:** Add an AC (or extend ac-01) covering Worked example block updates: drop the SNAPSHOT line, drop the `# write snapshot per §1.1` comment, and drop the snapshot reference from the Stage 3 comment. Verification: `git grep -F 'bead-snapshot-' plugins/orb/skills/drive/SKILL.md` must return zero matches (this is what ac-01's verification already asserts — but that assertion will FAIL unless the Worked example is also updated, which means ac-01 will reject as written despite implementer following its description faithfully).

### [MEDIUM] ac-01 verification (`git grep -F 'bead-snapshot-'`) will fail if any AC's instructions are followed literally

**Category:** constraint-conflict
**Pass:** 1
**Description:** ac-01's description says "delete §1.1 and §3.1 along with their bash blocks" — the Worked example bash block at line 787-790 is NOT inside §1.1 or §3.1 (it's inside `## Worked example`). An implementer following ac-01's description literally will leave the Worked example block intact, and ac-01's verification (`git grep -F 'bead-snapshot-'` returns zero) will then FAIL on the surviving Worked example reference. The description and verification disagree.
**Evidence:** drive SKILL.md line 788: `SNAPSHOT="orbit/reviews/$BEAD/bead-snapshot-$(date -I).md"` is in the Worked example, not in §1.1 or §3.1.
**Recommendation:** Either widen ac-01's description to "remove all snapshot-write references from drive SKILL.md including the Worked example block," or narrow the verification to scope it to the §1.1/§3.1 line ranges. The two must agree.

### [MEDIUM] Drive REQUEST_CHANGES return paths still say "write a fresh snapshot"

**Category:** missing-requirement
**Pass:** 1
**Description:** drive SKILL.md §1.6 (line 290-291) on REQUEST_CHANGES says "return to §1.1 to write a fresh snapshot and re-fork". §3.4 (line 411) says "return to §3.1 for the next cycle". After ac-01 deletes §1.1 and §3.1, these cross-references are dangling pointers, AND the prose still says "write a fresh snapshot" — exactly what the spec is removing. No AC updates these REQUEST_CHANGES paths.
**Evidence:** drive SKILL.md lines 285-291 and 408-412.
**Recommendation:** Add an AC for the §1.6 and §3.4 prose updates: cross-refs renumber per ac-01's renumbering rule, "write a fresh snapshot" prose drops to "re-fork" (no fresh artefact under bead-native).

### [MEDIUM] Drive Completion PR-body line still references the snapshot path

**Category:** missing-requirement
**Pass:** 1
**Description:** drive SKILL.md §Completion step 3 (line 495) says "Body references the bead-id, snapshot path, and review files". ac-07 updates commit-1's description string but does not touch the PR-body line. After the spec, the PR-body will still reference a snapshot path that doesn't exist.
**Evidence:** drive SKILL.md line 495.
**Recommendation:** Extend ac-07's description to include this PR-body line, or add a separate AC. Verification: `git grep -F 'snapshot path' plugins/orb/skills/drive/SKILL.md` returns zero matches (this would also catch the Worked example ref as a bonus).

### [MEDIUM] Inline-mode output-path block still references `.orbit/specs/<topic>/` — no AC updates it

**Category:** missing-requirement
**Pass:** 1
**Description:** review-spec SKILL.md line 135 and review-pr SKILL.md line 119 both contain an "Inline invocation" block prescribing the output path `.orbit/specs/YYYY-MM-DD-<topic>/review-{spec,pr}-<date>.md`. Constraint #7 changes the inline argument to a bead-id — under bead-native there is no `<topic>` and no `.orbit/specs/<topic>/` directory for the bead. The default output path is incoherent post-cutover. No AC fixes this.
**Evidence:** review-spec SKILL.md lines 133-136; review-pr SKILL.md lines 117-120.
**Recommendation:** Add an AC updating the inline-invocation default to `orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md` (matches Q2's bead-native path contract). Verification: literal string `.orbit/specs/YYYY-MM-DD` does not appear in either skill.

### [MEDIUM] Cycle-history bleed: bead `[x]` rows leak prior REQUEST_CHANGES context into the cold fork

**Category:** assumption
**Pass:** 2
**Description:** Decision 0011 D2 commits to "cold-fork stays" — fork sees no shared conversation history. Under spec.yaml substrate, the fork read a snapshot of the spec at fork time (no AC status). Under bead substrate, the fork runs `bd show <bead-id> --json` and `parse-acceptance.sh acs <bead-id>` — which include each AC's `[x]` checked status from prior cycles. On a cycle-2 review-spec re-fork after a cycle-1 REQUEST_CHANGES + implementer edits, the fork will see ACs already marked complete (or partially so) by the prior cycle's implement work. That's cycle history leaking into a "cold" fork. The brief's "do not include conversation context" prohibition is honoured; the substrate quietly carries the leak.
**Evidence:** `parse-acceptance.sh` `acs` subcommand emits `<status>` column (`[ ]` or `[x]`) per row. `bd show --json` returns the bead's current acceptance_criteria string with whatever check marks the implement skill last wrote.
**Recommendation:** Either (a) document the leak as accepted (review-spec runs before implement, so cycle-2 review-spec post-implement is rare) and add a sentence to constraint #1 noting that "cold fork" means "no conversation history" not "no AC status state," or (b) add an AC making the review-spec skill strip `[x]` markers before evaluation (a one-line awk in the skill). Option (a) is probably honest; option (b) is honest-and-complete. Either way, the silent inheritance is undesirable.

### [LOW] Review-pr loses the "this AC was implemented in <commit>" signal that progress.md provided

**Category:** test-gap
**Pass:** 2
**Description:** ac-06 removes the progress.md cross-reference from review-pr Phase 1 step 3. progress.md was the implementer's self-attestation linking ACs to commits/files. Under bead substrate, `parse-acceptance.sh acs` gives `[x]` status but no provenance — review-pr can verify "is this AC checked off" but not "where in the diff was it implemented." That's a real fidelity loss the spec partially acknowledges (in Q4) but doesn't contextualise as a finding the reviewer should call out.
**Evidence:** review-pr SKILL.md line 31; ac-06 description (no replacement provenance mechanism).
**Recommendation:** Acceptable as-is if Hugh accepts the loss (the diff plus AC text is usually enough for a fresh-context reviewer). Worth surfacing in the MADR 0012 consequences section so future readers see it.

### [LOW] ac-02/ac-03 brief-example literal-string assertions assume placeholder syntax `<bead-id>` not `$BEAD`

**Category:** test-gap
**Pass:** 1
**Description:** ac-02's verification asserts the literal substring `bd show <bead-id> --json` is present in the brief example. drive SKILL.md's existing Worked example uses `$BEAD` style (line 807: `bd show $BEAD --json`). An implementer who writes the brief example consistently with the rest of the doc will use `$BEAD` and break the literal-string assertion. The verification is workable but brittle to a stylistic choice.
**Evidence:** drive SKILL.md line 191 vs line 807 — both styles already coexist in the file.
**Recommendation:** Either accept that the brief example must use the `<bead-id>` placeholder verbatim (and note this in implementation_notes), or relax the verification to a regex like `bd show (<bead-id>|\$BEAD|\$bead_id) --json`.

### [LOW] No AC updates the Resumption table cross-reference to renumbered §1.3

**Category:** missing-requirement
**Pass:** 1
**Description:** drive SKILL.md line 652 contains a cross-reference `(idempotent §1.3 check skips fork if file already valid)`. After ac-01's renumbering, §1.3 → §1.2 — the table cross-ref becomes stale. Same for any other internal §-references inside drive SKILL.md not enumerated by ac-01.
**Evidence:** drive SKILL.md line 652.
**Recommendation:** Add a verification rider to ac-01: "After deletion + renumbering, run `git grep -nE '§(1|3)\.[1-9]' plugins/orb/skills/drive/SKILL.md` and audit each match for staleness." Or list the known cross-refs explicitly in implementation_notes.

### [LOW] ac-04 verification (b) "spec.yaml does not appear in §1" may pass by accident if the section gets shorter

**Category:** test-gap
**Pass:** 1
**Description:** Asserting absence of a string is fragile — it passes when the section is correctly rewritten, but also passes if the section is accidentally deleted or empty. No positive assertion that §1 still describes "Gather the bead" with the right content.
**Evidence:** spec.yaml line 31.
**Recommendation:** Strengthen ac-04 verification with a positive substring: "the literal phrase `Gather the Bead` (or equivalent — `Gather the spec via bd show`) appears as the §1 header." Defensive but cheap.

### [LOW] Decision 0002 (ac-test-prefix) is silently superseded by ac-06

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-06 removes `metadata.test_prefix` lookup from review-pr §3. Decision 0002 (`.orbit/choices/0002-ac-test-prefix.md`) establishes the test_prefix convention. Removing the runtime use without superseding the decision leaves the decision register out of sync with the skill's behaviour.
**Evidence:** `ls .orbit/choices/0002*` exists; ac-06 removes the field reference; no AC updates decision 0002's status.
**Recommendation:** Add an AC (or extend ac-09) updating decision 0002's status to `superseded by 0012` (or whichever number resolves the 0012 collision finding above).

---

## Honest Assessment

The spec is well-structured, the substrate-mapping table in the interview is crisp, and ac-08's hand-crafted fixture cleverly proves the parser+rule pipeline. The hard-cutover discipline is appropriate.

**The biggest risk is HIGH-1: the gate-AC parity claim is structurally false for any bead created via `promote.sh`.** Cards don't carry per-scenario gate flags, and `promote.sh` doesn't emit `[gate]` markers — so every promoted bead has zero gate ACs, and the deterministic gate-AC check rewritten by ac-05 will silently no-op for production beads. The card's stated goal ("zero regression in the cold-fork review gate's depth") is not met against the actual artefact pipeline; it's only met against synthetic acceptance fields. This is the same silent-degradation failure mode the snapshot bridge had, just relocated.

Secondary risks are unscoped edits across drive SKILL.md (Worked example, REQUEST_CHANGES return paths, Completion PR-body line, internal §-cross-refs) and inconsistencies in the inline-mode output-path blocks. Several of these will cause ac-01's literal-grep verification to fail on a faithful implementation — the spec's description and verification disagree about scope.

The spec is fixable with: (1) a card-schema/`promote.sh` AC that propagates gate semantics from card to bead, OR an explicit scope-limit acknowledgement; (2) a cleanup AC covering Worked example, REQUEST_CHANGES paths, Completion PR-body, and internal cross-refs; (3) an inline-mode output-path AC; (4) resolving the 0011 → 0012 numbering collision. With those, REQUEST_CHANGES → APPROVE on the next pass.
