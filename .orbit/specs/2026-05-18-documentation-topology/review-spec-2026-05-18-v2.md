# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-18-documentation-topology
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 (both LOW) |
| 2 — Assumption & failure | content signals (schema artefact addition + 3 envelope extensions + 4 skill surfaces + parity tests across CLI/MCP) | 1 (LOW) |
| 3 — Adversarial | not triggered (no structural concerns after pass 2 — sequencing and rollback now named in `goal`) | — |

## Cycle-1 carry-forward (responsiveness audit)

The v1 review (`review-spec-2026-05-18.md`) raised four MEDIUMs and four LOWs. v2 addresses them as follows:

| Cycle-1 finding | v2 location | Disposition |
|---|---|---|
| MEDIUM AC-02 hides three sub-changes | Split into ac-02 (Config artefact + FIELDS + drift test), ac-03 (`docs.topology` key + round-trip), ac-04 (verify wiring + parity) | Resolved |
| MEDIUM AC-04 exit-code vs envelope discrimination | ac-06 explicit: "Exit code is 0 for all three outcomes... consumers MUST NOT reach for `$?`" + symmetry with `audit drift` named | Resolved |
| MEDIUM AC-06 unverified envelope keys | ac-08 cites the actual envelope (handover / item_bound / memories / next_step / open_specs) "confirmed via `orbit --json session prime` against this repo on 2026-05-18" — verified live (handler at `verbs.rs:3143`+ uses `item_bound`) | Resolved |
| MEDIUM AC-07 substring-match noise floor | ac-09 specifies case-insensitive word-boundary regex `\b<subsystem>\b` + minimum subsystem-name length ≥ 5 chars | Resolved |
| LOW AC-05 brownfield-stub clobber risk | ac-07 explicit: brownfield-accept creates stub at target if absent, does NOT overwrite an existing file (byte-compare check called out) | Resolved |
| LOW AC-09 nudge negative test | ac-11 adds "Manual smoke (negative)" line | Resolved |
| LOW AC-08 release output dump | ac-10 specifies N=10 truncation rule with verbatim summarisation wording | Resolved |
| LOW sequencing & rollback notes | `goal` carries explicit "Sequencing: ac-02 → ac-03 → ac-04 close before ac-06, ac-07, ac-08, ac-09" + "Backout" sentence | Resolved |

All eight cycle-1 findings closed.

## Findings

### [LOW] ac-02 still bundles two artefacts (Config struct + schema-drift test) under one AC

**Category:** test-gap
**Pass:** 1
**Description:** Cycle-1 recommended a 3-way split of the old AC-02 (Config artefact / `docs.topology` key / verify wiring). v2 lands a 2-of-3 split — ac-03 and ac-04 are now standalone, but ac-02 still bundles three sub-changes inside one AC: (a) the `Config` struct with `deny_unknown_fields`, (b) the `Config::FIELDS` constant, (c) the `schema_drift_config` unit test. Each is independently verifiable, but the AC's `checked` boolean flips on all three at once. Sequencing depends only on the struct existing; the test and FIELDS const land in the same commit by construction (the test reads the const). This is acceptable as written — the three sub-items co-located in `schema.rs` form one editable unit and the cycle-1 concern (downstream verbs needing a single boundary to depend on) is satisfied because ac-04 is now the seam, not ac-02. Filing as LOW rather than re-raising MEDIUM.
**Evidence:** ac-02 description: "Config struct… Config::FIELDS constant. Schema-drift unit test added alongside the existing tests at the bottom of schema.rs… asserting Config::FIELDS matches the struct's serde representation". Cycle-1 recommendation accepted ac-02a as the artefact-creation slice; v2 ac-02 matches that slice plus the drift-test commitment.
**Recommendation:** Accept as-is. The schema.rs convention already co-locates struct + FIELDS const + drift test (e.g. `Spec::FIELDS` at `schema.rs:56` + drift test at `schema.rs:596`); separating them across ACs creates spec-prose friction without test-coverage benefit.

### [LOW] ac-08 `topology_drift` shape duplicated across ac-06 / ac-08 / ac-09 with no normative reference

**Category:** missing-requirement
**Pass:** 1
**Description:** Three ACs prescribe the same envelope-entry shape `{ subsystem: string, drift_kind: string }` (ac-06 for `orbit audit topology`, ac-08 for `session prime`, ac-09 for `spec.close`). Each AC restates the shape independently. If implementation diverges across the three surfaces (e.g. ac-09 emits `kind` instead of `drift_kind`, or adds a fourth category), the spec gives no normative seat — each AC is independently correct against its own text.
**Evidence:** ac-06: "ok envelope with a topology_drift array" (no shape detail). ac-08: "Each entry is an object { subsystem: string, drift_kind: string } where drift_kind matches the three categories from ac-06 (stale_pointer, missing_entry, shape_drift)". ac-09: "as { subsystem, drift_kind } entries". Categories enumerated only in ac-06.
**Recommendation:** Cheap fix: add one sentence to ac-06 explicitly naming the entry shape `{ subsystem: string, drift_kind: "stale_pointer" | "missing_entry" | "shape_drift" }`, then ac-08 and ac-09 can reference "the entry shape from ac-06" rather than restating. Acceptable to defer to implementation if the implementing agent is the same one who owns all three — file as LOW.

### [LOW] ac-09 spec-text concatenation rule is permissive about ordering

**Category:** assumption
**Pass:** 2
**Description:** ac-09's "subsystems touched" detection reads "the spec's spec.yaml and (when present) interview.md, then for each topology entry whose subsystem name is ≥ 5 characters long, test for case-insensitive word-boundary match (regex `\b<subsystem>\b`) in the concatenated spec text." Two assumptions worth surfacing:
  1. The regex `\b<subsystem>\b` interpolates a subsystem name that may contain regex metacharacters (`/`, `-`, `.`, `_`). For names like `orb:topology` or `audit.drift`, raw interpolation will misparse. Most realistic subsystem names are alphanumeric+hyphen+slash so the risk is bounded, but the spec doesn't say to regex-escape the interpolation.
  2. "Concatenated spec text" — concatenation order (spec.yaml first vs interview.md first) is irrelevant to word-boundary match results, so the spec is correct, but the implementer might over-interpret and add a separator that splits a real match across a boundary. Cheap to clarify.
**Evidence:** ac-09: "regex \b<subsystem>\b in the concatenated spec text". No mention of `regex::escape` or equivalent; no mention of how the two files are concatenated.
**Recommendation:** Add one sub-clause to ac-09: "subsystem names are regex-escaped before interpolation; the concatenation joins with a single newline." Both are obvious to a careful implementer; naming them removes the ambiguity. Filing as LOW because the implementing agent will almost certainly get this right by default.

---

## Honest Assessment

This is a v2 that absorbed cycle-1 feedback cleanly. All four cycle-1 MEDIUMs are closed; all four LOWs are closed; the sequencing and rollback notes are now in `goal` where downstream consumers (drive, implement) will read them. The three new findings here are all LOW paper-cuts — none of them changes the implementation contract, none of them blocks the spec from going to implementation. The verdict is APPROVE rather than REQUEST_CHANGES because re-cycling for LOW-grade tightening costs more than letting the implementing agent resolve them in passing.

The plan's biggest residual risk is not a spec-shape concern but an integration concern: ac-06 / ac-08 / ac-09 all extend existing envelope shapes (`audit topology`, `session prime`, `spec.close`) and require parity tests on both CLI and MCP. That's four parity-test additions across two crates plus three handler edits. The implementing agent should sequence ac-02 → ac-03 → ac-04 first as the spec instructs, then build ac-06's verb (because ac-08 and ac-09 both reuse its drift-detection logic), then add the surfaces. The spec already implies this; the `goal` sequencing line nails it.

One observation, not a finding: ac-13 (the 4-week observation-band audit) is well-formed and correctly classifies as `observation` per the AC-taxonomy. spec.close will defer it; the audit memo path is named; the data sources are enumerated. This is exactly the shape an observation-band AC should take.
