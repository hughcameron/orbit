# Spec Review

**Date:** 2026-05-07
**Reviewer:** Context-separated agent (fresh session)
**Bead:** n/a — pre-substrate spec, reviewed from `.orbit/specs/2026-05-07-orbit-state-v0.1/spec.yaml` directly. Prior reviews: v1 (REQUEST_CHANGES, 17 findings); v2 (REQUEST_CHANGES, 1 HIGH + 6 MEDIUM + 1 LOW). This is the v3 pass against the further-amended spec.
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | content signals (deployment, data migrations, cross-system boundaries, schema changes) | 0 (no structural concerns surfaced) |
| 3 — Adversarial | not triggered (no contradicted load-bearing assumptions; no cascading failure modes) | — |

## Pre-flight: prior review reconciliation (v2 → v3)

The v2 review issued 1 HIGH, 6 MEDIUM, and 1 LOW. The amended spec materially incorporates all of them:

```
v2 finding                                            Severity   Status in v3 spec
----------------------------------------------------  --------   ----------------------------------
CI round-trip gate omits schema-version (ac-16)       HIGH       FIXED — ac-16 now scopes to specs,
                                                                 cards, choices, memories, AND
                                                                 schema-version. Tasks excluded
                                                                 explicitly per append_only_excluded.
                                                                 values.enforcement.substrate_written
                                                                 also lists schema-version.
ac-21 skeleton-binary link scope                      MEDIUM     FIXED — ac-21 now requires linking
                                                                 the same C-dependency chain
                                                                 (rusqlite at minimum), with nm/otool
                                                                 SQLite-symbol verification.
Main-branch migration timing unpinned                 MEDIUM     FIXED — ac-22 added: pre-dogfood
                                                                 cutover as single atomic commit
                                                                 against main.
Skill parity bar fuzzy for judgement skills           MEDIUM     FIXED — ac-14 differentiates state-
                                                                 mutating (byte-identical state) vs
                                                                 judgement-emitting (verdict equality
                                                                 + HIGH-finding-coverage equality).
ac-05 reset semantics undefined                       MEDIUM     FIXED — ac-05 pins
                                                                 `git stash` + `rm state.db && orbit
                                                                 verify --rebuild`.
Migration A YAML body round-trip fixture missing      MEDIUM     FIXED — ac-01 enumerates four
                                                                 multiline-body fixtures (code fence,
                                                                 trailing blank line, hard tabs,
                                                                 CRLF). ac-12 cross-references them.
Dogfood-not-converging kill missing                   MEDIUM     FIXED — k-8 added: three consecutive
                                                                 dogfood restarts → pivot to greenfield
                                                                 dogfood (per K5 path).
ac-13 hash-set source-side snapshot implicit          LOW        FIXED — ac-13 verification pins
                                                                 source-side snapshot captured BEFORE
                                                                 migration runs and BEFORE
                                                                 .beads → .beads-archive rename.
```

All v2 findings are addressed at the contract level. The findings below are fresh-eyes observations from a v3 read; none are ship-blockers.

## Findings

### [LOW] ac-01 fixture (iv) CRLF round-trip is ambiguous about expected behaviour
**Category:** test-gap
**Pass:** 1
**Description:** ac-01 requires four choice-fixture cases to pass byte-identical round-trip, including "(iv) a body with CRLF line endings." For a CRLF fixture to round-trip byte-identical, either the canonical writer must emit CRLF (mixed-platform pain — the file is then platform-non-portable), or the parser must reject CRLF at parse time. The spec doesn't pin which. If the implementer interprets "byte-identical" strictly, they'll preserve CRLF on output and create a write-platform vs read-platform asymmetry. If they interpret it as "normalise then round-trip," fixture (iv) silently fails the byte-identical claim.
**Evidence:** ac-01 verification, fixtures (i)-(iv).
**Recommendation:** Either (a) clarify that CRLF input is rejected at parse time with a clear error (most defensible — canonical files are LF-only), and reword fixture (iv) as "a CRLF body MUST fail parse with a clear error"; or (b) explicitly accept that the canonical writer preserves input line endings, with a note that this is intentional. Recommend (a) — keeps the canonical output deterministic across platforms.

### [LOW] ac-22 commit composition is mildly under-specified for a gitignored archive
**Category:** missing-requirement
**Pass:** 1
**Description:** ac-22 says "Cutover commit on main contains both migrations (Migration A layout move + Migration B substrate move) in a single commit, with `.beads-archive/` introduced in the same commit." But `.beads-archive/` is gitignored (per ac-13). The gitignore entry is what the commit introduces, not the directory contents. A reader could interpret "introduced in the same commit" as "the contents are committed" and either (a) try to `git add -f` the archive (creating a large historical artefact and contradicting the gitignore) or (b) realize the contents are local-only and worry the spec is internally inconsistent.
**Evidence:** ac-22 verification ("`.beads-archive/` introduced in the same commit"); ac-13 description ("renamed to `.beads-archive/` and gitignored at this AC").
**Recommendation:** Reword ac-22's phrasing to: "the cutover commit contains the `.beads/ → .beads-archive/` rename as a deletion of `.beads/` plus the `.gitignore` entry for `.beads-archive/`. The archive contents themselves are NOT committed (they live only in the operator's working tree, which is what K7's local-rollback path depends on)."

### [LOW] K7 rollback path implicitly assumes the operator's local working tree
**Category:** assumption
**Pass:** 1
**Description:** k-7's pivot "(1) Revert to bd: restore .beads-archive/ → .beads/ (gitignored contents preserved)" works only on a working tree that has `.beads-archive/` populated. A fresh clone at v0.1.0 doesn't carry the archive (gitignored). If the post-ship critical defect surfaces on a different machine than the cutover ran on, or after an aggressive working-tree reset, the rollback path is unavailable. The 14-day window mostly covers this in practice (the operator's machine retains state), but the assumption isn't pinned.
**Evidence:** k-7 pivot step (1); ac-13 (`.beads-archive/` gitignored); ac-22 (cutover commit).
**Recommendation:** Either (a) take a one-time snapshot of `.beads/` to a non-gitignored path during cutover (e.g., `archive/beads-snapshot-pre-orbit-state.tar.gz` committed to main, redacted of any secrets) so K7's rollback survives any clone within 14 days; or (b) add a one-line note to k-7 acknowledging the local-only assumption: "K7 rollback runs from the operator's working tree; if `.beads-archive/` is not present locally, restore from a pre-cutover commit (`git checkout <pre_cutover_hash> -- .beads/`)." Recommend (b) — cheap, no commit-history bloat, matches the "operator within 14 days" framing.

### [LOW] ac-15 grep verification is judgement-laden
**Category:** test-gap
**Pass:** 1
**Description:** ac-15 verification: "Grep CLAUDE.md for `bd ` (with trailing space) returns 0 results that refer to verb invocations." The "that refer to verb invocations" qualifier means a pure grep with non-zero output isn't an automatic fail — the verifier must classify each match (e.g., a match in a code-block illustrating historical context vs. a match in a live workflow instruction). Without categorization rules, two reviewers could disagree on whether the AC passes.
**Evidence:** ac-15 verification.
**Recommendation:** Tighten to a deterministic check: "Grep CLAUDE.md for `bd ` (trailing space) returns 0 matches." If retaining illustrative `bd ` references in CLAUDE.md is intentional, move them under a clearly-labelled historical/migration-notes section that the grep can exclude with `--exclude-line` or similar, and pin the exclusion rule in verification. Recommend dropping the qualifier and forcing a clean grep — keeps the contract testable without judgement.

---

## Honest Assessment

The v2 amendments incorporate all eight prior findings cleanly, and the spec is now materially complete. The remaining findings are low-severity polish items: an ambiguous CRLF round-trip case, two phrasing tightening's around the .beads-archive lifecycle, and one judgement-laden grep verification. None invalidate the architecture, the contract, or the readiness-to-implement claim.

The structural discipline that emerged across three review cycles is impressive in its own right: every load-bearing claim has a halt or kill condition, every halt has an entry/exit criterion, every kill has a named pivot, and the migration cutover is now an atomic single-commit operation with a 14-day post-ship rollback window. The format-integrity claim is policed by CI on every entity that's user-writable; the substrate-written entities (specs, tasks, memories, schema-version) have appropriate enforcement (CI round-trip for the rewritable ones; append-only validation at write time for tasks).

The 4-week budget at 13–17 working days remains the load-bearing planning claim and is outside this review's scope. The structural discipline that would let it hold is in place: indicative subtotals enable Theme 5a early-warning before the overall budget bites; ac-21 forces the cross-compile risk class to surface in week 1; ac-22 makes the cutover atomic so partial-state limbo isn't possible.

Recommend APPROVE. The four findings are pre-implementation polish — worth a 30-minute amendment pass before /orb:implement, but not a blocker. None of them gate any AC's verification or any halt-trigger's revert path.
