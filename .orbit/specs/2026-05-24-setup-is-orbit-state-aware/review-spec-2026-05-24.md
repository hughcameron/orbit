# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-24-setup-is-orbit-state-aware
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | content signals (schema change, cross-drive boundary, history-preserving migration); HIGH finding in Pass 1 | 2 |
| 3 — Adversarial | not triggered | — |

The spec carries a substantial scope gap between the `acceptance_criteria` field and the design captured in `interview.md` / `decisions.md`. The ACs are inherited verbatim from the card's nine scenarios and describe the *pre-rally* shape; the rally proposal added a 5th setup state (`wrapped-undotted`), a shared classifier in `orbit-state`, a `plugin_repo` config flag, a move-and-warn `decisions/` migration, a new `decisions-md-unmigrated` conformance finding family, and a flat 6-row state table — none of which appear as testable acceptance criteria. An implementer working from `orbit spec show` alone would build a different feature than the one the rally locked.

## Findings

### [HIGH] Rally-locked scope is absent from acceptance_criteria

**Category:** missing-requirement
**Pass:** 1
**Description:** The spec's `acceptance_criteria` field contains nine ACs that mirror card 0017's nine scenarios verbatim (compare `.orbit/cards/0017-setup-is-orbit-state-aware.yaml:9-53` against `spec.show` output for ac-01..ac-09). The rally proposal and the interview's *Summary → Success Criteria* and *Decisions Surfaced* sections add scope that is nowhere reflected in the ACs:

- the 5th setup state slug `wrapped-undotted` (Q1, decisions §1)
- the `git mv orbit .orbit` single-rename migration mechanism (Q2, decisions §2)
- the shared classifier in `orbit-state` callable from skill *and* `audit conformance` (Q3, decisions §3)
- the `plugin_repo: true` config flag in `.orbit/config.yaml` with default false and an empty `.orbit/topology/` + README on non-plugin projects (Q4, decisions §4)
- the `git mv decisions .orbit/decisions/` (not `.orbit/choices/`) directory rename plus inline warning (Q5, decisions §5)
- the new `decisions-md-unmigrated` conformance finding family (Q5, locked cross-drive)
- the flat 6-row state table with `brownfield-bare` / `mixed-bare` / `mixed-undotted` parallel renames (Q6, decisions §6)
- the SKILL.md `creates orbit/ directory` → `creates .orbit/ directory` description fix (decisions §6 incidental)
- the one-line edit to set `plugin_repo: true` in this repo's `.orbit/config.yaml` (Summary §Success Criteria item 7)
- the `crates/cli/tests/` parity test for the `wrapped-undotted` classifier (Summary §Success Criteria item 6)

**Evidence:** `interview.md` lines 80-96 ("Success Criteria") and 100-104 ("Decisions Surfaced") enumerate the locked scope. `spec show` output for ac-01..ac-09 carries none of these terms. The drive skill closes the spec by checking ACs — without these as ACs, the implementation could finish "all ACs green" while shipping none of the rally-approved features.

**Recommendation:** Rewrite `acceptance_criteria` so each locked Success Criterion in `interview.md` becomes a numbered, testable AC. Suggested additions (existing ac-01..ac-09 stay but most can be folded into the new shape):

- *AC: classifier in `orbit-state` returns `wrapped-undotted` for `orbit/`-present-`.orbit/`-absent layouts.* Evidence: Rust unit test in `crates/core/src/verbs.rs` siblings; parity test in `crates/cli/tests/` covering the wrapped-undotted case.
- *AC: setup migrates `orbit/` → `.orbit/` via a single `git mv` from the `wrapped-undotted` state, preserving history.* Evidence: integration test asserting `git log --follow` on a moved file traces to its `orbit/` ancestor.
- *AC: setup in a non-plugin project (no `plugin_repo: true`) scaffolds `.orbit/topology/` empty with a README, writes no substrate-typed seed entries; `orbit audit topology` reports `configured: false, drift: []`.*
- *AC: setup in the orbit-plugin repo (where `plugin_repo: true` is set) seeds the 5 substrate-typed topology entries unchanged; `orbit audit topology` reports `configured: true`.*
- *AC: setup `git mv`-s `decisions/` to `.orbit/decisions/` (not `.orbit/choices/`) and prints a one-paragraph warning naming the MD→YAML conversion requirement.*
- *AC: `orbit audit conformance` returns a `decisions-md-unmigrated` finding when `.orbit/decisions/` contains `.md` files without matching `.orbit/choices/<slug>.yaml`.*
- *AC: setup SKILL.md state table at §1 is the 6-row form (`greenfield` / `idempotent` / `brownfield-bare` / `wrapped-undotted` / `mixed-bare` / `mixed-undotted`); `brownfield` and `mixed` are renamed.*
- *AC: SKILL.md line 3 description reads `creates .orbit/ directory` (not `creates orbit/ directory`); migration-prompt example reads `decisions/ → .orbit/decisions/ (MADR files; manual MD→YAML conversion needed)` (not `decisions/ → .orbit/choices/`).*
- *AC: this repo's `.orbit/config.yaml` carries `plugin_repo: true`.*
- *AC: setup refuses cleanly on the `mixed-undotted` state (`.orbit/` present AND `orbit/` present) with a message naming both paths; no filesystem mutation occurs.*

Each existing ac-01..ac-09 should be reviewed against the new shape. ac-06's prompt text in particular contradicts decisions §5 (see next finding).

### [HIGH] ac-06 prompt text contradicts decisions §5

**Category:** constraint-conflict
**Pass:** 1
**Description:** ac-06 reads: *"the prompt names what will move (orbit/ → .orbit/, decisions/ → choices/, MD → YAML)"*. Decisions §5 (interview.md lines 49-53; decisions.md lines 107-110) explicitly overrules this — the rally-locked decision is `decisions/ → .orbit/decisions/` (directory rename, no content conversion) with the SKILL.md migration-prompt example updated to read `decisions/ → .orbit/decisions/ (MADR files; manual MD→YAML conversion needed)`. The AC's prompt text describes the *old* design.

**Evidence:** `spec show` ac-06 string; `interview.md:49-53`; `decisions.md:107-110`. The contradiction is concrete: an implementer following ac-06 would write the prompt the rally proposal locked *out*.

**Recommendation:** Rewrite ac-06 to reflect the locked decision. Suggested phrasing: *"Brownfield with bare dirs prompts before migrating — the prompt names what will move, including `decisions/ → .orbit/decisions/ (MADR files; manual MD→YAML conversion needed)`; on yes, the migration runs idempotently via `git mv`."* Pair it with the new `decisions-md-unmigrated` conformance-finding AC (above) so the durability half of decisions §5 has its own gate.

### [MEDIUM] Cross-drive dependency on `undotted_substrate` finding is unstated in the spec

**Category:** missing-requirement
**Pass:** 1
**Description:** The interview (lines 25-26 and 142-144) names a sister drive on card 0039 that ships an `undotted_substrate` conformance finding which "suppresses the existing `canonical-files-missing` finding" — *"this drive's design assumes that suppression exists"*. The spec's `acceptance_criteria` and the spec record itself surface no dependency wire (no `relations:` block in the spec, no AC asserting the sister-drive finding is in place). If the sister drive lands later or in a different shape, the canonical-files-missing finding fires on every `wrapped-undotted` repo after migration starts but before `.orbit/METHOD.md` lands — exactly the failure mode the suppression exists to prevent.

**Evidence:** `interview.md:25-26` ("Cross-drive: the sister drive on card 0039 ships an `undotted_substrate` conformance finding that suppresses canonical-files-missing — this drive's design assumes that suppression exists.") and `interview.md:147-148` (deferred questions). `spec show` carries no `cards: [0039-...]` entry and no AC referencing the suppression.

**Recommendation:** Add either (a) an AC stating the dependency explicitly (*"After `wrapped-undotted` migration completes, `orbit audit conformance` does not fire `canonical-files-missing` for the migrated repo until `.orbit/METHOD.md` lands — relies on the sister drive's `undotted_substrate` suppression."*), or (b) a `relations:` entry on the spec wiring this spec to the sister drive's spec id, or (c) both. Without one of these, the dependency is review-time tribal knowledge.

### [MEDIUM] `OrbitConfig` schema change has no AC for the parse contract

**Category:** missing-requirement
**Pass:** 1
**Description:** Interview's Implementation Notes (lines 117-118) name a schema change: *"`OrbitConfig` schema (file at `.orbit/config.yaml`) gains an optional `plugin_repo: bool` field, default `false`."* Adding a field to a parsed schema requires (a) backwards compatibility on existing configs (parsing a config without the field must succeed and default to false), and (b) acceptance the field is plumbed all the way to the topology-seed gate. No AC names either.

**Evidence:** `interview.md:117-118` lists the change as an implementation note, not a verifiable criterion. `spec show` ACs do not reference `OrbitConfig` or `plugin_repo`.

**Recommendation:** Add an AC: *"`OrbitConfig` parses an existing `.orbit/config.yaml` without `plugin_repo` as `plugin_repo: false` (default); parses with `plugin_repo: true` as the seed gate. Existing configs without the field must continue to parse cleanly — backwards compatibility test in `crates/core/src/`."*

### [MEDIUM] `mixed-undotted` refusal lacks a dedicated AC

**Category:** missing-requirement
**Pass:** 2
**Description:** The 6-row state table (interview Q6, decisions §6) includes `mixed-undotted` (`.orbit/` present AND `orbit/` present) as a refuse state. Decisions §2 also names a defence-in-depth check: *"If `.orbit/` already exists alongside `orbit/`, refuse with a 'mixed-undotted' error"*. No AC asserts the refusal happens or names the error message shape. The existing `mixed`-state coverage in `plugins/orb/skills/setup/SKILL.md:123-138` covers the `mixed-bare` case only.

**Evidence:** `interview.md:64-66` (state-table row); `decisions.md:51` (defence-in-depth note); `spec show` ACs do not name `mixed-undotted` or this refusal.

**Recommendation:** Add an AC: *"Setup refuses on `mixed-undotted` (both `.orbit/` and `orbit/` present at root) with a message naming both paths and the collision; no filesystem mutation occurs; exit non-zero."*

### [LOW] Greenfield AC (ac-03) does not name topology scaffolding

**Category:** test-gap
**Pass:** 2
**Description:** ac-03 enumerates greenfield setup actions: create cards/choices/specs/memos, run `orbit init`, write METHOD.md, ensure CLAUDE.md @-import. It does not mention `.orbit/topology/`, but the rally scope (interview Q4, decisions §4) requires greenfield setup to scaffold either an empty `.orbit/topology/` + README (non-plugin) or the 5 substrate-typed seed entries (plugin repo). Without coverage in ac-03 or a new AC, a greenfield install in a non-plugin repo could land without the empty topology dir + README and no AC catches it.

**Evidence:** `spec show` ac-03 text; `interview.md:91-92` (Success Criteria items 2 and 3); `SKILL.md:210-236` (existing §6d topology scaffolding text).

**Recommendation:** Either (a) extend ac-03's description to include topology scaffolding routed by `plugin_repo`, or (b) add a separate AC covering both branches (plugin-repo seeds vs non-plugin empty README). The non-plugin case is the load-bearing one — the rally's framing was that the plugin-repo behaviour stays; the non-plugin case is the new safety property.

---

## Honest Assessment

The design work in `interview.md` and `decisions.md` is solid — six rally questions resolved with reasoned trade-offs, evidence anchors named to file:line, cross-drive dependencies surfaced. The problem is the design didn't propagate into the spec's `acceptance_criteria` field. The spec record currently mirrors card 0017's nine scenarios (pre-rally) rather than the rally-locked scope. An implementer running `/orb:drive` against this spec hits the implement skill, sees ac-01..ac-09, and could ship a feature that closes all ACs while shipping none of the rally additions — the substrate's contract with the rally proposal is broken at the spec layer.

The biggest risk is the silent mismatch: the interview-vs-spec drift is invisible to anyone reading `orbit spec show` alone, and ac-06's contradiction with decisions §5 is a concrete trap (an implementer following ac-06's prompt-text constraint actively writes the wrong prompt). Both should be fixed before implementation begins.

Once the ACs are rewritten to cover the locked scope (HIGH findings #1 and #2 above), the cross-drive dependency is surfaced (MEDIUM finding #3), and the schema-parse and `mixed-undotted` ACs are added (MEDIUM findings #4 and #5), the spec is implementation-ready. The underlying design is sound and the supporting context is in good order — this is a transcription gap, not a re-think.
