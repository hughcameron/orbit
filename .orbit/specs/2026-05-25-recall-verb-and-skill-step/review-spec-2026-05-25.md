# Spec Review — 2026-05-25-recall-verb-and-skill-step

**Date:** 2026-05-25
**Reviewer:** Claude (Sonnet 4.6)
**Spec:** `.orbit/specs/2026-05-25-recall-verb-and-skill-step/spec.yaml`
**Verdict:** **REQUEST_CHANGES**

---

## Load-bearing claim

Ship substrate-wide recall: `orbit recall <topic>` fans across all five artefact types in a
single ranked response, and six pipeline skills run it as a mandatory structural step at
entry. The verb is composable from existing per-type verbs plus two new substring-search
paths (specs, memos). Pillar 2 (agent self-learning) is the downstream beneficiary.

The claim is coherent and compressible. The load-bearing test: an agent that doesn't think
to look is now structurally pulled to look. That test drives every AC.

---

## Contract pieces (tabletop sidecar)

Present and complete. Tabletop at `.orbit/specs/2026-05-25-recall-verb-and-skill-step/tabletop.md` carries:
- Values: surfacing at the moment that matters, pillar 2 downstream
- Trade-offs: 5 named (fan-out vs unified ranker, mandatory vs opt-in, top-level vs subcommand, agent-typed scope, 5 types vs 3)
- Halt conditions: 3 (regression on per-type verbs, workspace suite, skill invocation break)
- Escalation triggers: 3 (confusing merged results, non-trivial spec+memo search, cascading inconsistency across skills)
- Kill conditions: 3 (noisy recall, skill-entry skipping despite prose, cite-rate as flat signal)

No gaps. The tabletop sidecar satisfies the halt/escalation/kill requirement.

---

## AC-by-AC findings

### ac-01 — `orbit recall <topic>` top-level verb

**Gradeable from disk:** Yes. `cargo test --workspace rcall_returns_substrate_wide_results` + smoke.

**Implementation grounding:** Fan-out to `orbit memory match` (exists), `orbit card search` (exists), `orbit choice search` (exists) is confirmed in the codebase. Spec search (against `spec.yaml` goal + AC descriptions) and memo search (`.md` bodies) do not exist yet — both are real new work. Tabletop escalation E2 covers this budget risk explicitly. The fan-out plan is sound.

**Min-max normalisation edge case (MEDIUM):** The spec states "each type's results are re-scored to a 0.0-1.0 scale via min-max within that type's matches." It does not specify the behaviour when a type returns zero matches. With zero matches the type's result set is empty and there is nothing to normalise — the behaviour is implicitly correct (empty → no tuples emitted for that type) but the AC should name this explicitly so the test fixture can assert it. Add: "When a type returns no matches, it contributes no tuples to the merged result; no normalisation is applied for that type."

**Default tie-breaking order** (memory > choice > card > spec > memo) is specified. Worth noting that this order differs from the type-grouped escalation pivot in tabletop E1 — if the ranking produces confusing interleaving, the escalation path is type-grouping, not score-only. The AC doesn't need to address this (it is the escalation trigger's job), but the implementing agent should read E1 before writing the merge step.

**Status:** APPROVE with minor prose amendment (zero-match edge case).

---

### ac-02 — `--type` filter flag

**Gradeable from disk:** Yes. `cargo test --workspace rcall_filter_by_type` + smoke with `--type memory,choice`.

**Implementation grounding:** Standard clap multi-value flag pattern. `Error::malformed` shape is the canonical error type (confirmed in use elsewhere). Empty-list-treated-as-default is specified and testable. No gaps.

**Status:** APPROVE.

---

### ac-03 — `--json` envelope + CLI/MCP parity

**Verb name convention violation (HIGH):** The spec says `verb name "recall"` in the JSON envelope. Every existing verb in the codebase uses dotted namespace notation: `memory.match`, `card.search`, `choice.search`, `spec.list`, `task.open`, etc. — all registered via `#[serde(rename = "namespace.action")]` in `VerbRequest` and `VerbResponse` enums. A bare `"recall"` would break the naming convention and MCP tool routing (MCP maps `{name: "noun.verb"}` → `{"verb": "noun.verb", ...}`). The correct verb name is `"substrate.recall"` to follow the established pattern.

**Envelope shape is otherwise correct:** `data.result.matches[]` is consistent with `memory.match`'s actual wire shape (`{"data":{"result":{"matches":[...]},"verb":"memory.match"},"ok":true}`). The `result.matches[]` key name is the right choice given that `memory.match` already uses it.

**Smoke test syntax is correct:** `orbit --json recall recall | jq .data.result` addresses the right path. If the verb name changes to `substrate.recall`, the CLI invocation stays `orbit recall <topic>` (CLI subcommand name is decoupled from the serialised verb name) — but the spec should name the wire verb as `"substrate.recall"` so the parity test can assert the envelope field.

**Status:** REQUEST_CHANGES — name the wire verb as `"substrate.recall"` (or justify the bare name explicitly).

---

### ac-04 — No regression on existing per-type verbs

**Gradeable from disk:** Yes. `cargo test --workspace` with baseline 555 is confirmed (ran during this review, 555 tests pass, 8 suites, 2.43s).

**Scope constraint is correctly stated:** "recall verb composes these — it does not refactor their shared code paths." This is the right guard given the fan-out implementation strategy.

**Status:** APPROVE.

---

### ac-05 — Six pipeline SKILL.md files gain "Recall pre-flight" section

**File list check (PASS):** The six named skills are:
`plugins/orb/skills/tabletop/SKILL.md`, `implement/SKILL.md`, `spec/SKILL.md`,
`review-spec/SKILL.md`, `review-pr/SKILL.md`, `researcher/SKILL.md`.

All six directories exist in `plugins/orb/skills/`. No `/orb:design` — that skill has no
directory, consistent with discovery interview summary line 61 naming "design" alongside the
other six but card 0044 scenario 2 clarifying it as aspirational. The spec's list of six
correctly omits `design`. No mismatch.

**None of the six SKILL.md files currently contain "Recall pre-flight" or "orbit recall"**
(confirmed via grep) — correct, this is pre-implementation review.

**Scope-derivation rules per skill are named and differentiated (PASS):**
- tabletop: cluster cards' references[] + goal/scenarios
- implement: AC description + tabletop.md sidecar
- spec: interview.md or tabletop.md being crystallised
- review-spec: spec's goal + AC descriptions
- review-pr: PR's changed paths + spec id
- researcher: topic argument

These mirror the investigation-orchestration spec's per-skill scope derivation in
`2026-05-25-investigation-as-pipeline-step` ac-01 through ac-05, with the investigation
Skill call swapped for `orbit recall`. The mirror claim holds.

**"Immediately before their existing pre-flight / work-loop content" is implementable** but
requires the implementing agent to read each SKILL.md's current structure to find the
insertion point. The spec does not name the insertion point line number (by design — it would
be stale immediately). This is acceptable; the verification clause (grep returns the six
files + per-file grep returns ≥1 `orbit recall` match) is sufficient.

**Verification clause completeness (MEDIUM):** The AC says "each skill file passes a smoke
read — invoking the skill against a test argument loads cleanly (no broken prose, no dangling
section headers)." This is a human smoke read, not a machine-assertable test. Acceptable for
a `doc` AC type, but the implementing agent should interpret this as: load each updated
skill file in a fresh session and confirm the Recall pre-flight section appears before the
skill's first substantive step.

**Status:** APPROVE.

---

### ac-06 — `path` field resolves to correct repo-relative filesystem path

**Gradeable from disk:** Yes. `cargo test --workspace rcall_path_field_format` with fixture
asserting path shapes per type.

**Path table is complete and unambiguous:**
- memory → `.orbit/memories/<key>.yaml`
- card → `.orbit/cards/<slug>.yaml`
- choice → `.orbit/choices/<slug>.yaml`
- spec → `.orbit/specs/<id>/spec.yaml`
- memo → `.orbit/memos/<filename>.md`

Consistent with the canonical-file references used by `orbit audit conformance`. Repo-relative
(not absolute) is the right choice — matches conformance finding `path` fields.

**Gate: false** — correct. Path field is a usability enhancement, not a load-bearing
correctness requirement.

**Status:** APPROVE.

---

## Cross-cutting findings

### AC IDs (PASS)
Sequential ac-01 through ac-06. No gaps or duplicates.

### Test prefix (PASS)
`rcall` applied consistently. ac-01 → `rcall_returns_substrate_wide_results`; ac-02 → `rcall_filter_by_type`; ac-03 → `rcall_json_envelope_parity`; ac-06 → `rcall_path_field_format`. ac-04 checks the existing baseline (no new `rcall_` prefix needed). ac-05 is `ac_type: doc` (no code tests expected). Prefix is clean.

### ac_type assignments (PASS)
- ac-01 through ac-04, ac-06: default `code` — correct (cargo tests + smoke)
- ac-05: `doc` — correct (SKILL.md prose changes)
- No `observation` or `ops` ACs — appropriate; this spec ships capability, not a measurement window

### Goal compressibility (PASS)
The goal statement is compressible to: "Make `orbit recall <topic>` a first-class verb that fans across all five substrate types, and make six pipeline skills run it at entry as a mandatory structural step." Every AC traces to that.

### Grounding against prior art (PASS)
- Discovery interview (6 questions) → spec goal, 5-type scope, mandatory pull, broad-first-push staging all carry through without drift
- Card 0044 scenarios → all seven scenarios map to spec ACs (scenarios 1, 6, 7 → ac-01/02/06; scenario 2 → ac-05; scenario 5 → ac-04; scenarios 3/4 are spec 2 and spec 3 — correctly out of scope)
- investigation-as-pipeline-step pattern → ac-05's scope-derivation rules mirror exactly, with investigation Skill call swapped for `orbit recall` invocation

---

## Required changes before implementation

1. **ac-03 verb name** — replace `verb name "recall"` with `verb name "substrate.recall"` to follow the `namespace.action` convention. Update the parity test assertion accordingly. The CLI invocation (`orbit recall <topic>`) is unchanged — only the wire verb name changes.

2. **ac-01 zero-match edge case** — add one sentence: "When a type returns no matches, it contributes no tuples to the merged result; normalisation is skipped for that type."

---

## Summary

Five of six ACs are clean. Two targeted amendments required:

- ac-03: verb name convention (`"recall"` → `"substrate.recall"`) — HIGH, affects MCP tool routing and parity contract
- ac-01: zero-match normalisation is implicit, not stated — MEDIUM, will produce test-fixture ambiguity if not named

Neither finding requires redesign. Both are one-sentence amendments to AC prose. Verdict is REQUEST_CHANGES rather than BLOCK because implementation cannot begin correctly with ac-03's naming gap unresolved, but the underlying design is sound.
