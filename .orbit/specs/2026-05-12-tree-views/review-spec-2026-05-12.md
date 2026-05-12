# Spec Review

**Date:** 2026-05-12
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-12-tree-views
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | content signals (cross-system boundaries: CLI + parity + session-prime + multiple SKILL.md files) + MEDIUM Pass-1 findings | 3 |
| 3 — Adversarial | not triggered — Pass 2 issues are local to specific ACs, no cross-AC cascades | — |

## Findings

### [HIGH] AC-05 depends on a canonical schema document that does not yet exist
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-05 says `orbit audit drift` flags fields "absent from the canonical schema (per card 0030's drift-detection scenario)". Card 0030 is `Planned` and its wire-up gate (a SCHEMA.md at a stable path, linked from README/CLAUDE.md, cited by skills) is not yet shipped. `find` for `SCHEMA.md` / `schema*.md` / `glossary.md` in the repo returns nothing. Card 0030's two closed specs (`2026-05-09-card-vs-choice-discrimination`, `2026-05-10-card-id-field-and-conventions`) delivered the discrimination decision tree and the id-field convention — neither produced a canonical schema document. The only mechanical schema today is the Rust struct `Card` in `orbit-state/crates/core/src/schema.rs`, which already uses `#[serde(deny_unknown_fields)]` and rejects unknown fields at parse time. So the verb either (a) reads a doc that doesn't exist, (b) reflects the Rust struct (in which case the verb is a no-op against the existing strict-parse invariant — every drift is already a parse error), or (c) reads some third source the spec doesn't name.
**Evidence:** `orbit card show 0030` → `maturity: Planned`. Card 0030 scenario "Wired into the framework" (gate: true) names SCHEMA.md as the required output and is unmet. `orbit-state/crates/core/src/schema.rs:152` declares `#[serde(deny_unknown_fields)]` on `Card`. Repo-wide find returns no `SCHEMA.md` or `glossary.md`.
**Recommendation:** Either (1) drop AC-05 from this spec and depend it on a prior spec that ships card 0030's SCHEMA.md, or (2) name the substitute schema source explicitly in AC-05 ("walks the Rust struct field set in `orbit-state/crates/core/src/schema.rs` for `Card`, `Spec`, `Choice`, treating any field present in a YAML file but not in the struct as drift") and explain how that differs from the existing strict-parse error path. As written, AC-05 is not implementable.

### [MEDIUM] AC-06 parity-test scope contradicts the CLI-only goal
**Category:** constraint-conflict
**Pass:** 2
**Description:** AC-06 requires `orbit-state/crates/cli/tests/parity.rs` to be extended to cover "each new verb's `--json` envelope against MCP". The parity contract is that the CLI's `--json` stdout matches the canonical envelope produced by `orbit_state_core::envelope_ok`, and the MCP test (`crates/mcp/tests/parity.rs`) checks the same expected envelope from its surface — i.e. parity requires the verb to exist on both surfaces. The spec's goal and AC-01..05 describe CLI verbs (`orbit card tree`, `orbit card specs`, `orbit overview`, `orbit graph`, `orbit audit drift`) with no mention of MCP. Either MCP surface is implicit scope (and a missing AC), or AC-06 cannot deliver "parity" because there's no MCP counterpart to compare to.
**Evidence:** `orbit-state/crates/cli/tests/parity.rs:1-11` documents the cross-surface contract. AC-06 description quotes "against MCP" verbatim. Goal names "ship navigation and synthesis verbs" without naming MCP. AC-01..05 enumerate CLI verb names only.
**Recommendation:** Make a choice and pin it in the AC. Option A: add an AC requiring each new verb to ship on both CLI and MCP surfaces, with parity covering both. Option B: scope parity in AC-06 to the canonical-envelope-only side (CLI `--json` matches `envelope_ok`), drop the "against MCP" phrasing, and defer MCP surface to a later spec. Option B is the lighter cut and matches the "no new tables" frugality the goal pre-commits to.

### [MEDIUM] AC-03 aggregation contracts are under-specified
**Category:** test-gap
**Pass:** 2
**Description:** AC-03 names five outputs for `orbit overview` but two are not deterministically testable as written. "Most-connected card (highest relation degree)" — which edges count? Outgoing `relations:` only? Outgoing + incoming? Do `specs:` array entries count as edges (card 0033's relations field today has 2 outgoing edges; its specs has 1; a card with specs:[] but 5 outgoing relations vs a card with specs:[5 items] and 1 outgoing should produce different rankings depending on the rule). "Orphans (cards with no specs AND no incoming relations)" — incoming-relation detection requires walking every card to build a reverse index, which is fine but worth pinning. Ties (two cards equally connected) are not addressed. Without a precise rule, two correct implementations can produce different output and the AC can't be tested deterministically.
**Evidence:** AC-03 description (spec.yaml:18-21) and card 0033 scenario "Single-screen project state" (cards/0033-see-the-tree.yaml:14-18) both use the phrase "most-connected" without defining the edge set. AC-04's mermaid graph names "cards-choices-specs" as the node set — implying choices and specs are also nodes — but AC-03 says "most-connected card" which suggests cards only.
**Recommendation:** Pin three rules in AC-03: (a) the edge set ("outgoing `relations:` + counted incoming `relations:` from other cards; `specs:` array entries do not count toward connectivity"), (b) the tie-break ("lowest numeric id wins on ties"), (c) orphan definition ("a card with `specs: []` AND zero incoming `relations:` references from any other card"). The implement skill needs these to write a test that fails when the rule is wrong, not just when the output is unreadable.

### [MEDIUM] AC-07 surfacing wire names the wrong skills
**Category:** test-gap
**Pass:** 1
**Description:** AC-07 requires the new verbs to be named in "at minimum `setup`, `drive`, `rally`" SKILL.md files "where browsing the tree is the natural next move". Setup runs once per project; drive and rally are pipeline drivers that already have a fixed remit (promote → review-spec → implement → review-pr) — none of these is the obvious "browse the tree" site for an interactive author. The natural sites are `/orb:card`, `/orb:design`, `/orb:distill`, and the session-prime output itself. Picking three skills that don't naturally browse the tree risks shallow wires ("here's the verb, FYI") that fail card 0033's gate "without this wire the verbs exist but nobody reaches for them".
**Evidence:** Card 0033 scenario "Wired into the framework" (cards/0033-see-the-tree.yaml:44-48) repeats the same three-skill list, so the spec is faithful to the card, but the card itself names a weak wire. `session prime` already references open specs and memories — adding `orbit overview` to its output is the load-bearing wire; SKILL.md additions are secondary.
**Recommendation:** Rephrase AC-07 to put the session-prime wire as the primary gate and the SKILL.md wires as supporting. Replace "at minimum `setup`, `drive`, `rally`" with "at least one author-facing skill where the tree view is the genuine next step — candidates: `/orb:card` (after `show`, suggest `card tree`), `/orb:distill` (before producing a new card, suggest checking the existing tree). Setup is the wrong site (one-shot)". This is a card 0033 amendment as much as a spec change; flag it as both.

### [LOW] AC-04 filter semantics under-specified
**Category:** test-gap
**Pass:** 1
**Description:** AC-04 names `--filter` flags with one example (`--card 0028 --depth 2`) but doesn't enumerate the filter set. Is `--card` the only filter? Are `--choice`, `--spec`, `--maturity`, `--pillar` planned? Does `--depth` apply only with `--card` or globally? Without bounds the verb could grow unboundedly during implement; with bounds it stays focused.
**Evidence:** AC-04 description, spec.yaml:22-25.
**Recommendation:** Either explicitly bound the filter set ("supported flags: `--card <id>` and `--depth <N>`, default depth 2; other filters are out of scope for this spec") or list the full set in the AC. Tightening this also rules out scope creep at implement-time.

### [LOW] No rollback plan; no monitoring; no obvious failure-handling AC
**Category:** missing-requirement
**Pass:** 1
**Description:** The spec ships five new CLI verbs that read files. Rollback is trivial (revert the binary) and monitoring is N/A for read-only verbs, so the omission is defensible. But there's no AC covering behaviour on substrate inconsistency: what does `orbit card tree 99` do when card 99 doesn't exist? What does `orbit card specs 0033` do when a referenced spec file is missing? What does `orbit graph` do when a card cites a relation to a card that doesn't exist? Each verb should fail with a structured error envelope, not a panic.
**Evidence:** No AC covers error envelopes for the new verbs.
**Recommendation:** Add one AC (gate=true) requiring each new verb to emit a structured error envelope (matching the existing `--json` error shape used by `orbit card show <unknown-id>`) for the three obvious error paths: unknown id, broken referenced file, malformed YAML in a walked file. One test per path is sufficient.

---

## Gate-AC description check

All five gate ACs (ac-01, ac-03, ac-06, ac-07) pass the deterministic structural rules: non-empty, no placeholder tokens, all far above the 20-character minimum. No deterministic findings from this rule. (Pass 2 findings above name semantic issues in some of those same gate ACs — orthogonal to the structural check.)

---

## Honest Assessment

The spec is well-scoped against card 0033 and cleanly carves out the file-walking / no-new-tables constraint. The five-verb shape is right and AC-01 / AC-02 are tight. The two material problems are AC-05 (depends on a SCHEMA.md that doesn't exist yet — card 0030 is still Planned and its wire-up gate is unmet) and AC-06 (parity scope quietly assumes MCP surface that the rest of the spec doesn't deliver). The third (AC-03 aggregation rule) is sharpening, not rework. Fix AC-05 by either deferring it to a follow-up spec or naming the Rust struct as the schema source explicitly; fix AC-06 by trimming "against MCP" or adding an MCP-surface AC; tighten AC-03 with explicit edge-set and tie-break rules. The biggest risk is shipping AC-05 against a non-existent reference and quietly delivering a no-op verb against the strict-parse path; the spec needs to commit to its substitute or step back.
