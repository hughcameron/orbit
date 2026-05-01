# Spec Review

**Date:** 2026-05-01
**Reviewer:** Context-separated agent (fresh session)
**Spec:** orbit/specs/2026-05-01-bead-native-cold-fork-reviews/spec.yaml
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | content signals (cross-system boundaries, shared parser, schema change, hard cutover with in-flight drives) | 2 |
| 3 — Adversarial | not triggered (no cascading failure modes, no untestable ACs, impact radius bounded by named files + scoped MADRs) | — |

## Findings

### [LOW] AC ordering between ac-13 (renumber) and ac-09 (new MADR) is implicit

**Category:** missing-requirement
**Pass:** 1
**Description:** ac-09 creates `0013-bead-acceptance-field-as-cold-fork-substrate.md` and asserts "the next free integer after the renumber" (constraint #11). ac-13 performs the renumber that frees up integer 0013. Neither AC declares an explicit ordering dependency. Implementer could write ac-09 first while `0011-design-intent-not-means.md` still occupies 0011 — at that point 0012 is the next free integer, not 0013, and ac-09's filename assertion would be defensible at write-time but would land out-of-order against ac-13's renumber that pushes 0011-design-intent-not-means to 0012. The spec resolves this only by author goodwill — the author has to read constraint #11 and infer the ordering.
**Evidence:** spec.yaml line 12 (constraint #11), line 57 (ac-09 references "0013"), line 77 (ac-13 description states "After this AC closes, the next free decision number is `0013`"). The acceptance_criteria array in spec.yaml is not interpreted as a strict ordering by the bead substrate (`parse-acceptance.sh next-ac` walks declaration order but no `[gate]` markers exist on ac-13 or ac-09 to enforce sequencing).
**Recommendation:** Either (a) annotate ac-13 with `[gate]` so that under bead substrate, parse-acceptance.sh's gate-blocking semantics force ac-13 to complete before any later AC including ac-09 can start; or (b) add a one-line implementation_note explicitly stating "ac-13 must land before ac-09 — the 0013 number is only free post-renumber." Since this spec adds gate semantics to the pipeline (ac-11), using `[gate]` on ac-13 also doubles as a real-world dogfood test of the gate-propagation pipeline.

### [LOW] ac-11(a) treats "card schema documentation" as conditional ("if it exists")

**Category:** missing-requirement
**Pass:** 1
**Description:** ac-11(a) says the new `gate: true` scenario field gets "recorded in any project-level card-format documentation if it exists, plus inline as a YAML comment in card 0016." The card-format documentation does in fact exist — it's at `plugins/orb/skills/card/SKILL.md` lines 36–40 and 58–62, which authoritatively defines the per-scenario YAML shape (`name`, `given`, `when`, `then`). Phrasing it as "if it exists" risks the implementer skipping the canonical card SKILL.md update and relying only on the YAML comment in card 0016. Future card authors invoking `/orb:card` will not see `gate: true` in the documented schema and will not know to use it — the propagation pipeline (ac-11b/c) lands but stays undiscoverable through the normal authoring path.
**Evidence:** plugins/orb/skills/card/SKILL.md §3 documents the scenario shape with no `gate` field. ac-11(a) verification is positive only on card 0016's `gate: true` literal — it does not assert anything about plugins/orb/skills/card/SKILL.md.
**Recommendation:** Strengthen ac-11(a) to name the file: "the card SKILL.md (`plugins/orb/skills/card/SKILL.md`) Step 2 question 5 and Step 3 YAML template gain a documented optional `gate: true` field per scenario." Add a verification rider: `git grep -F 'gate:' plugins/orb/skills/card/SKILL.md` returns ≥1 match in §3's YAML template. Cheap defensive add and removes a discoverability gap.

### [LOW] ac-04 verification (a) is absence-only; positive content guard is partial

**Category:** test-gap
**Pass:** 1
**Description:** ac-04 verification (e) was added to address the cycle-1 review's concern about absence-only assertions ("the section header is the literal string `Gather the Bead`"). That guards against a wholesale empty rewrite. But verification (a) "the literal string `spec.yaml` does not appear in §1" still passes if the section is correctly rewritten OR if the section is reduced to just the header line plus one sentence that omits substrate detail. The minimum-viable §1 satisfying ac-04 is `### 1. Gather the Bead\n\nRun bd show <bead-id> --json. Run parse-acceptance.sh acs <bead-id>.` That clears (a)–(f) but lacks the procedural cues a fresh-context reviewer needs (what to look for in metadata, how to handle missing arg, fallback behaviour).
**Evidence:** spec.yaml lines 32–33; current §1 in plugins/orb/skills/review-spec/SKILL.md lines 26–31 has 4 procedural lines including "If neither exists, report that no spec was found."
**Recommendation:** Add a verification rider: §1 must contain a missing-argument fallback clause (e.g. `git grep -F 'no bead-id provided' plugins/orb/skills/review-spec/SKILL.md` returns ≥1 match), which mirrors the implementation_note's prescription on line 84 ("If not: report 'no bead-id provided — review-spec requires a bead-id under the bead-native substrate'"). Optional but tightens the spec/implementation contract.

### [LOW] ac_type / test_prefix references survive in audit, spec, spec-architect, implement skills

**Category:** missing-requirement
**Pass:** 1
**Description:** ac-09's MADR (0013) marks decision 0002 (ac-test-prefix) as `superseded by 0013`. ac-06 removes `ac_type` and `test_prefix` references from review-pr. But `ac_type` and `test_prefix` are still alive in `plugins/orb/skills/audit/SKILL.md` (lines 23, 39, 41, 56, 66, 142, 146, 152), `plugins/orb/skills/spec/SKILL.md` (lines 49, 59, 83), `plugins/orb/skills/spec-architect/SKILL.md` (lines 37, 41, 61, 71, 74, 79, 90), and the spec.yaml format generator. Decision 0002 is marked superseded but four skills still implement its convention. This is in scope-creep territory for this card (which is about cold-fork review substrate, not about audit/spec authoring), so I am NOT recommending the spec absorb those edits. The risk is only that the decision register is technically inconsistent post-merge: 0002 says superseded, but the convention is still live in non-review-skill code paths.
**Evidence:** `grep -nE "test_prefix|metadata\\.test_prefix|ac_type" plugins/orb/skills/{audit,spec,spec-architect}/SKILL.md` returns 24 matches across 4 files; none are removed by this spec.
**Recommendation:** Either (a) narrow ac-09's supersession scope to "decision 0002's runtime use is superseded for the cold-fork review substrate; the test_prefix / ac_type convention remains active for spec.yaml-mode authoring (audit, spec, spec-architect, implement)"; or (b) note in MADR 0013's consequences that the supersession is review-pr-scoped and the broader retirement of test_prefix is deferred to a follow-up spec (one for each skill that still uses it). Either keeps the decision register honest. No new ACs needed.

### [LOW] ac-11(b) verification (`git grep -F '[gate]' promote.sh`) is brittle to escaping

**Category:** test-gap
**Pass:** 2
**Description:** ac-11(b) verification asserts `git grep -F '[gate]' plugins/orb/scripts/promote.sh` returns ≥1 match. The string `[gate]` is a literal that will be interpolated into the f-string. But promote.sh today emits AC lines via `f'- [ ] {ac_id}: {name} — {then_clause}'` — adding a literal `[gate]` requires an f-string like `f'- [ ] {ac_id}{gate_marker}:'` where `gate_marker = ' [gate]' if s.get('gate') else ''`. If the implementer chooses a different idiom (e.g. building the marker as `marker = " [gate]" if scenario.get("gate") else ""` outside the f-string), the literal `[gate]` IS in the source — just on a different line than the f-string. The grep passes either way, so the verification is fine. But there's a less obvious failure mode: if the implementer uses a constant like `GATE_MARKER = " [gate]"` and then references the constant, the grep still passes. Good. However, if the implementer uses string concatenation `marker = " [" + "gate" + "]"` (unlikely but possible if avoiding the literal for some reason), the grep fails despite correct behaviour. Low-probability edge case.
**Evidence:** spec.yaml line 68; current promote.sh lines 113-118.
**Recommendation:** Already mitigated by ac-11(c): `promote.sh --dry-run` against card 0016 must contain `[gate]` on the line for ac-02. That assertion is behavioural and immune to source-level idiom choices. The grep in ac-11(b) is a reasonable belt-and-suspenders. No change required; flagging for awareness.

### [LOW] Brief example placeholder syntax `<bead-id>` collides with existing `$BEAD` style elsewhere in drive SKILL.md

**Category:** test-gap
**Pass:** 2
**Description:** This was flagged in the cycle-1 review (LOW). The cycle-2 spec addresses it via implementation_notes (line 82) and verification (e) on ac-02 and ac-03 (`example block does not use $BEAD placeholder syntax`). Solid. The remaining residual risk: drive SKILL.md elsewhere uses `$BEAD` style (the Worked example at line 807). After this spec lands, drive SKILL.md will have BOTH styles in different blocks — the brief example uses `<bead-id>`, the Worked example uses `$BEAD`. That's intentional (brief is reviewed by literal-string assertions; Worked example is bash-executable code) but is a documentation inconsistency a future reader may flag. Acceptable but worth noting in implementation_notes.
**Evidence:** spec.yaml lines 23, 28, 82; plugins/orb/skills/drive/SKILL.md line 807.
**Recommendation:** No spec change needed. The dual-style is justified by the dual purpose of the blocks. Optional: add a one-line note to drive SKILL.md after the brief example explaining why this block uses `<bead-id>` rather than `$BEAD` (so future maintainers don't "fix" the inconsistency).

---

## Honest Assessment

This is a clean cycle-2 spec. The cycle-1 HIGH finding (gate-AC parity is structurally false because cards don't carry `gate: true` and `promote.sh` doesn't emit `[gate]`) is fully addressed by ac-11, with both code (`promote.sh` update) and behavioural (test fixture + dry-run dogfood) coverage. The MEDIUM cycle-1 findings (Worked example, REQUEST_CHANGES return paths, Completion PR-body, inline-mode default path, decision 0011 collision, decision 0002 supersession, cycle-history bleed) are each picked up by a named AC or an explicit constraint with documentation falling to MADR 0013. The hard-cutover discipline holds: no dual paths, no `review_mode` flag, no auto-detect.

The spec is large — 13 ACs, two decision-record changes, two test scripts, and a card-schema extension — but each AC has a tight description-plus-verification contract. The verifications are mostly literal-string greps, file-existence checks, or `--dry-run` behavioural tests. ac-08 and ac-11(d) provide regression guards for the parser+rule pipeline and the gate-propagation pipeline respectively, both of which are the substrate-parity claim's load-bearing mechanisms.

Five LOW findings remain. None block. The most worth absorbing pre-implementation is the implicit ordering between ac-13 and ac-09 — adding `[gate]` to ac-13 would dogfood the very pipeline this spec is shipping (gate semantics from card to bead) and removes the only ordering dependency that isn't structurally enforced. The card-schema documentation gap (ac-11a) is real but narrow — flag and continue.

The biggest residual risk is not in the spec itself but in its scope discipline: decision 0002's supersession is correctly recorded but the convention it documents is still live in audit/spec/spec-architect/implement skills. That's a follow-up spec waiting to happen, not a blocker for this one. The spec is APPROVE-able and ready to implement.
