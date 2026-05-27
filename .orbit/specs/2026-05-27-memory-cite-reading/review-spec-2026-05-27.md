# Spec Review

**Date:** 2026-05-27
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-27-memory-cite-reading
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals: schema changes, backwards compatibility, data migration, cross-system MCP boundary | 4 |
| 3 — Adversarial | Pass 2 surfaced a substrate mis-framing (HIGH) that cascades to verification mechanic | 1 |

## Pass 1 deterministic checks

- AC parser output: 7 ACs (ac-01..ac-07). Gates: ac-01..ac-05 (`is_gate=1`); ac-06 and ac-07 are `is_gate=0`.
- Gate-AC description rules (non-empty / non-placeholder / >=20 chars): **all five gate ACs pass**. No Pass-1 MEDIUM finding from the deterministic check.
- AC testability: every AC carries an inline `Verification:` clause naming a concrete mechanic (unit test, integration test, parity test, schema test, grep). Test prefix `cite` declared.
- Scope vs goal: goal claims diagnostic honesty under memory compression + structural detectability of cite and read; ACs deliver detect (ac-01/02), record (ac-03), enforce (ac-04), name (ac-05), surface (ac-06), regress (ac-07). No over- or under-reach against the goal sentence.
- Content signals present (schema change, backwards-compat, migration, MCP parity) — Pass 2 triggered.

## Findings

### [HIGH] ac-01 mis-frames the memory substrate as a database
**Category:** missing-requirement / failure-mode
**Pass:** 2
**Description:** ac-01 says the `cites` field "round-trips through remember -> SQLite -> show -> match without loss" and its verification clause (d) demands "database migration applies to a fixture DB whose schema predates this field without data loss." This describes a substrate that doesn't exist. Memories are YAML files at `.orbit/memories/<key>.yaml` (see `layout.rs:126 pub fn memory_file`, `layout.rs:26` layout comment, and the `key/body/timestamp/labels` shape of any file in that directory). The `Memory` struct (`schema.rs:665-673`) is plain serde with `#[serde(deny_unknown_fields)]` over a YAML file — there is no SQLite row, no schema migration, no DB-shape predating the field. The real backward-compat mechanism is `#[serde(default)]` on the new `cites` field plus deny_unknown_fields tolerance (or removal of deny_unknown_fields and accept-on-default).
**Evidence:** Inspected `orbit-state/crates/core/src/layout.rs` (memory_file resolver to YAML path), `orbit-state/crates/core/src/schema.rs:665-673` (Memory struct serde over YAML), `orbit-state/crates/core/src/verbs.rs:3934-3962` (memory_remember calls `write_atomic(layout.memory_file(...), yaml.as_bytes())`), and a sample memory file at `.orbit/memories/four-pillars.yaml` (plain YAML, four fields). No memory-table SQL anywhere in the substrate.
**Recommendation:** Rewrite ac-01's verification (d) to name the real backward-compat surface — *"the new `cites` field uses `#[serde(default)]` so existing `.orbit/memories/<key>.yaml` files lacking the field deserialise as `cites: []`; the round-trip test loads a fixture memory file missing the field, asserts it loads cleanly, asserts re-serialisation does not write `cites: []` (skip_serializing_if = Vec::is_empty) so existing files stay byte-identical on disk."* Also strike "SQLite" from the round-trip path in (a)-(c) — it should read "remember -> file -> show -> match." This is the load-bearing finding for the verdict: implementing the AC as written will produce a database migration that doesn't apply (no DB), and the test as specified can't be written against the actual substrate.

### [MEDIUM] ac-04 misdescribes the existing close-time pre-flight shape
**Category:** assumption / constraint-conflict
**Pass:** 2
**Description:** ac-04 says it "Extends the existing close-time memory reconciliation check from spec 2026-05-19-memory-gates-decisions ac-04 — does not refactor it; runs alongside as a second pass over `memories_considered`." But the existing pre-flight at `verbs.rs:2317-2363` does not iterate over `memories_considered`. It iterates over **`memory_match` corpus results filtered by `MEMORY_MATCH_THRESHOLD`**, then checks whether each matching key is *absent* from `memories_considered`. The set the new pass needs to iterate is different again — entries *present* in `memories_considered` whose referenced memory carries non-empty `cites`. The AC's "second pass over `memories_considered`" framing is closer to the truth than "extends the existing pass over the corpus," but the AC simultaneously claims to "extend" the existing check, which suggests editing one loop, and "run alongside as a second pass," which suggests adding a second loop. The kill-condition escalation in the tabletop (kill #3) captures this, but the AC text as written is ambiguous about which loop is being added and against which collection.
**Evidence:** `verbs.rs:2323-2346` shows `memory_matches` (corpus query) is the source; `reconciled_keys` is built from `memories_considered`; `unreconciled` is `memory_matches \ reconciled_keys` filtered by threshold. There is no existing iteration *over* `memories_considered` — only *into* it as a containment check.
**Recommendation:** Rewrite ac-04's description to name the loop precisely: *"adds a second pre-flight pass that iterates entries in `spec.memories_considered`, loads each referenced memory, and for any whose `cites` is non-empty, asserts every cite_path appears in that entry's `cite_evidence` list. Refusal message names the memory key and missing cite_path(s). Runs after the existing corpus-vs-considered pass at verbs.rs:2317-2363; does not modify it."* The mechanic is correct; the language drift is the load-bearing risk for the implementing agent.

### [MEDIUM] ac-05 SKILL.md prose mechanic doesn't close the tabletop halt-condition it inherits
**Category:** test-gap
**Pass:** 2
**Description:** The tabletop's Halt condition #2 reads: *"Cite-read evidence not actually captured. Evidence records 'cite read' but the agent never opened the file. Halt: verification of memories_considered cite-evidence entries must include a content-derived value (excerpt or hash), not just a boolean."* ac-05's verification mechanic is a three-part grep — (a) "cites/cite_evidence within a directive line", (b) "reference to the spec.close pre-flight gate", (c) "reference to the excerpt + read_at shape". None of those grep targets verifies that the SKILL.md prose tells the author the excerpt must be **drawn from the cite's file contents** (the "1-3 line excerpt" referenced in ac-03 is the right shape, but ac-05's prose-test doesn't enforce that the SKILL teaches authors to extract excerpts from the file rather than synthesise them). The halt-condition's "content-derived value" requirement is the bar the AC must clear; the grep mechanic as written can be satisfied by SKILL.md prose that names the shape without teaching the discipline.
**Evidence:** Tabletop halt #2 (line 37); ac-05 description's grep mechanic (the three (a)-(c) clauses); ac-03's "excerpt drawn from the cite" wording does land the discipline in the substrate field's docs but ac-05 does not require SKILL.md to repeat it.
**Recommendation:** Add a fourth grep target to ac-05's mechanic: *"(d) prose explicitly directs the author to draw the excerpt from the file contents at `cite_path`, not paraphrase from memory."* Or restate the halt-condition closure mechanic directly in the AC text. Without this, the spec ships ac-05 with the structural discipline named in passing rather than gated.

### [MEDIUM] Regression AC is gate=false but is the only proof of backward-compat halt-condition
**Category:** test-gap
**Pass:** 2
**Description:** ac-07 (regression suite scans existing `.orbit/memories/` corpus, asserts all load with `cites: []`-or-absent, asserts spec.close pre-flight passes against fixture specs referencing each) is marked `gate: false`. The tabletop's Halt condition #3 reads: *"Backward-compat break. Old cite-less memories fail to load or get spuriously flagged. Halt: regression suite must run green against the current `.orbit/memories/` corpus."* ac-07 *is* that regression suite — it's the load-bearing structural proof that the backward-compat claim in the spec goal and in ac-01's description ("continue to operate exactly as before this spec") holds. Putting it outside the close-gate ring lets the spec close with the regression suite unwritten or red.
**Evidence:** `orbit spec acs` output: `ac-07 ... 0` (is_gate=0); tabletop line 38 halt-condition #3. Predecessor spec 2026-05-19-memory-gates-decisions had its analogous ac-06 ("skill-prompt-only enforcement is insufficient") as gate=false, but that AC was a discipline observation, not a regression suite — different shape.
**Recommendation:** Promote ac-07 to `gate: true`. The predecessor's ac-06 is the wrong analogy; the analogy is the predecessor's ac-04 (close-block on unreconciled memories), which was gate=true. A regression suite that proves a halt-condition cannot be a non-gate.

### [LOW] ac-06 MCP parity gate posture inherited verbatim from predecessor
**Category:** assumption
**Pass:** 3
**Description:** ac-06 (MCP parity) is `gate: false`, mirroring the predecessor's posture on MCP-shaped ACs. This is consistent with how the closed predecessor shipped, but worth flagging in a cite-reading review specifically because the MCP surface is the *agent-facing* shape — agents reconciling memories through `/orb:drive`, `/orb:rally`, and other forked-context skills will call MCP, not the CLI. If MCP parity slips, the cite-discipline lives only behind the CLI. Not load-bearing for the verdict (the predecessor closed clean with the same posture), but worth a moment's thought before re-confirming.
**Evidence:** AC parser is_gate=0 on ac-06; predecessor spec ac-02 (inline memory match) was gate=true but was the CLI surface, while MCP wiring was implicit in that spec's design pack (D1b verb registration).
**Recommendation:** Either keep `gate: false` (explicit decision, ship as-is) or promote to gate=true (load-bearing for agent reconciliation). Acceptable either way; just confirm the choice is deliberate.

---

## Honest Assessment

The plan is structurally clean — every AC carries an inline verification clause per 0.4.38 scope discipline, the tabletop's halt/escalation/kill matrix is complete, and the technical-dependency defence for one-spec scope is sound. The reason this is REQUEST_CHANGES and not APPROVE is the HIGH finding: ac-01 was authored against an assumed SQLite-backed memory substrate, and the actual substrate is YAML-file-canonical. An implementing agent following ac-01 verbatim will write a database-migration test that has no database to migrate, and the agent's escape route (reinterpret the AC) breaks the spec.close contract that says the AC text is authoritative. Fix ac-01's substrate framing, tighten ac-04's loop description, harden ac-05's grep mechanic to close the halt-condition it inherits, and promote ac-07 to gate=true. The biggest risk after those edits is the one the tabletop already names as kill-condition #2 — excerpt evidence being fluently faked — and the spec correctly leaves that as an in-flight escalation rather than pre-solving it. Good shape overall; fix the substrate mis-framing before drive opens.
