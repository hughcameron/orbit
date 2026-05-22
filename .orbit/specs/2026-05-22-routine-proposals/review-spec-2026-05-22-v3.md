# Spec Review

**Date:** 2026-05-22
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-22-routine-proposals
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-system substrate, new verb, write paths, audit family); v2 review history | 3 |
| 3 — Adversarial | not triggered (Pass-2 gaps are localised substrate-pinning, no cascade) | — |

Spec id supplied verbatim via the calling brief (`2026-05-22-routine-proposals`); `orbit --json spec resolve` not consulted per skill instructions. `drive.yaml` shows `review_spec_cycle: 2` — this is the third review on this spec (v1 + v2 both REQUEST_CHANGES on 2026-05-22). Output path supplied by brief verbatim: `review-spec-2026-05-22-v3.md`.

**Gate-AC description check (Pass 1 deterministic rule):** AC-01 reports `gate=true`. Description is non-empty (passes rule 1), trimmed value is not a placeholder token (passes rule 2), and is well over 20 chars (passes rule 3). Rule passes.

**Status of v2 findings.** All four v2 findings are addressed in the v3 spec:

| v2 finding | Resolution in v3 |
|---|---|
| [HIGH] AC-09 path-determinism (auto-name vs path lookup) | AC-04 adds `chain_id` (front-matter field); AC-09 pivots to content-addressed lookup by `chain_id`, path-independent; rename case covered by fixture (a)+(b) |
| [MEDIUM] AC-06 audit-mutates-skill invariants | AC-06 splits the write off audit onto a new `orbit routine verify` verb (atomic-write, sole writer); AC-07 explicitly carves the audit as read-only ("does not mutate routines"); the split is consistent across both ACs |
| [MEDIUM] AC-02 exact-ordering vs tabletop relaxation | AC-02 now names the ≤1-skipped-step rule for length-3+ and the longest-consistent-chain pick; fixture cases (a/b/c) cover all three |
| [LOW] AC-07 finding-family slug undefined | AC-07 names subsystem slug `routines`, state slugs `stale` + `broken_refs`, remediation verbs for each |

The v2 review's structural concerns are closed. The findings below are new — they emerged from a fresh pass on the v3 spec, all in the same shape: substrate the v3 spec presupposes is defined, but isn't yet pinned in the AC text.

## Findings

### [HIGH] AC-01's path (c) is undefined — "agent-side session record" has no substrate referent

**Category:** assumption
**Pass:** 2

**Description:** AC-01 (the gate) offers three paths the implementing agent may pick to prove chain-detection substrate. Path (c) reads:

> prove via passing unit test against a JSONL fixture that existing rows plus an agent-side session record suffice

"Agent-side session record" is not defined anywhere in the spec, the tabletop, the linked cards, or the existing substrate. The current `SkillInvocation` struct (`orbit-state/crates/core/src/schema.rs:418`) carries outcome metadata (`worked / partial / didnt-apply / incorrect`) but no session id, no position, no ordering field. There is no other substrate the phrase could be pointing at.

Concrete consequences:

1. **The path is unselectable.** The implementing agent cannot evaluate (c) against an unknown substrate — they will pick (a) or (b) by elimination, making (c) decorative. If the spec author intended (c) as the cheapest path, the cheapest path is also the most fragile to ship.
2. **If picked, (c) reinvents (a)/(b).** Whatever "session record" means concretely (a new file? a new field on `SkillInvocation`? an in-memory construct?), the agent must invent it — and that invention is a substrate decision the tabletop's escalation trigger #5 specifically reserved for the author.
3. **Chain ordering requires position.** AC-04 and AC-09 both presuppose an "ordered skill_id sequence" the routine encodes. Existing `SkillInvocation` rows are timestamped but the spec doesn't say timestamp-ordering is enough (within a session two invocations can share a timestamp; across sessions ordering across sessions is semantically odd for a chain). Path (c) must address ordering substrate or the downstream ACs hang on an unsourced premise.

**Evidence:**
- `spec.yaml` ac-01 path (c) — "agent-side session record" undefined
- `orbit-state/crates/core/src/schema.rs:418` — `SkillInvocation` struct has no session/position fields
- `tabletop.md:66-69` (escalation trigger #5) — explicitly reserves substrate decisions for the author
- v2 review Finding 1 raised the path-determinism shape for AC-09; the same shape now applies to AC-01 path (c)

**Recommendation:** Pick one:

1. **Drop path (c).** Leave AC-01 with paths (a) and (b) — the two concrete substrate shapes the tabletop's escalation trigger #5 already enumerates. If the agent's evidence is "existing rows suffice," that's path (b)'s aggregator-verb shape (the verb just turns out to be trivial), not a third path.
2. **Pin "session record" substrate.** Name what it is concretely — a new file path (`.orbit/sessions/<id>.jsonl`?), a new schema field, or an in-memory join. Name what "ordering" means. Then path (c) becomes selectable.

Either resolves the gap. Leaving (c) as written guarantees AC-01 closure is either non-mechanical (any of three undefined alternatives) or collapses back to (a)/(b) in practice, in which case the AC over-specifies the option space.

### [HIGH] AC-04's `chain_id` algorithm is unpinned — determinism claim cannot be verified

**Category:** missing-requirement
**Pass:** 2

**Description:** AC-04 introduces `chain_id` as "hex hash of the ordered skill_id sequence the routine encodes" and asserts "chain_id is deterministic across sessions for the same ordered skill sequence." AC-09's archive-state lookup is content-addressed on `chain_id` and depends on this determinism for correctness.

The AC does not pin the hash algorithm or the canonical input format. Concrete consequences:

1. **Algorithm choice is implementation-defined.** SHA-256 truncated? Blake3? FNV-1a? Each produces different hex strings for the same input. A v1 implementation that picks SHA-256 (32 chars hex), a v2 refactor that picks Blake3 (16 chars hex), and the AC-09 lookup runs against routines authored by both — the archive check silently misses cross-version matches.
2. **Input encoding is implementation-defined.** Even with a pinned hash, the canonicalisation matters. Is the input `["release","commit","push","reload"]` JSON-encoded? Or null-separated? Or newline-separated? With or without a trailing separator? Different choices produce different hex.
3. **The determinism claim is the load-bearing one.** AC-04's "deterministic across sessions" is what makes AC-09's content-addressed lookup work. Without a pinned algorithm, two implementing agents on the same chain produce different `chain_id`s — AC-09's "do not re-author for the same chain" branch silently produces duplicates, which is exactly the failure mode v2 review Finding 1 surfaced for path-determinism.

This is the same shape as v2 Finding 1, transposed from path-naming to hash-naming.

**Evidence:**
- `spec.yaml` ac-04 — "hex hash" with no algorithm pin, no encoding pin
- `spec.yaml` ac-09 — "consults the curator state by chain_id (content-addressed, not path)"; AC-09 closure depends on AC-04 determinism holding
- v2 review Finding 1 — same shape, addressed for path-naming; pattern recurs here

**Recommendation:** Add to AC-04: "Hash algorithm is **SHA-256 truncated to 16 hex chars**" (or Blake3 / whatever the spec author prefers — the specific choice is less important than pinning *one*). And: "Hash input is the ordered skill_id sequence JSON-encoded as an array of strings (e.g. `["orbit:release","orbit:commit","orbit:push","orbit:reload"]`) with no trailing whitespace." A two-line addition; closes the determinism gap surgically.

### [MEDIUM] AC-05's chain-vs-DAG decision rule is unwritten — the detector cannot route shapes

**Category:** test-gap
**Pass:** 2

**Description:** AC-05 splits routine bodies into two shapes:

- **Chain-shaped body** for strictly sequential chains (no branches, no parallelism).
- **DAG-shaped body** for chains with branching, parallelism, or conditional skips.

AC-05 is the only AC naming the distinction. AC-02 (the detector) describes only chain matching — linear sequences with the ≤1-skipped-step relaxation. Nothing in AC-02 or AC-05 names the algorithm that decides which shape to author from invocation evidence.

Concrete consequences:

1. **AC-02 fixture has no DAG case.** AC-02's three test cases are exact-length-2, length-3+ with one skipped step, longest-chain pick — all linear. The detector never proves it can emit a DAG. AC-05's fixture is supposed to cover both shapes, but AC-05 closes on the *body shape* of an authored SKILL.md, not on the *detection input* that produced it. So the test can pass by hand-crafting a DAG body without ever exercising DAG detection.
2. **Tabletop escalation #1 names this as runtime escalation, not spec gap.** Trigger #1 ("Chain-shape ambiguity") fires when "the agent detects recurrence but can't cleanly decide between chain and DAG." That's a runtime safety net for ambiguous cases; AC-05 implies the spec already knows how to call the shape when it *can* be called cleanly. The decision rule for clean cases is unwritten.
3. **Implementing-agent improvisation surface.** Faced with AC-02 (linear) + AC-05 (chain OR DAG), the agent will either (a) skip DAG detection in v1 and let escalation trigger #1 cover every DAG case (under-delivers vs AC-05's promise), or (b) invent a DAG-detection rule unilaterally (drift from author intent), or (c) escalate at first implementation (round-trip cost).

**Evidence:**
- `spec.yaml` ac-02 — linear matching only, three linear fixture cases
- `spec.yaml` ac-05 — names two shapes, no decision rule for how the detector picks
- `tabletop.md:50-53` (escalation trigger #1) — runtime safety net for ambiguous cases, not a spec mechanism

**Recommendation:** Pick one:

1. **Defer DAG detection to v2.** Restrict AC-05 to chain-shaped bodies only; explicitly say DAG detection lands in a follow-up spec, and accept that v1 emits no DAG routines. This is the cleanest cut and matches the tabletop's "v1 mechanism is the simplest cut" framing (`tabletop.md:25`).
2. **Pin the v1 decision rule.** Extend AC-02 with a DAG-detection clause naming what evidence triggers DAG-shape emission (e.g. "if the same prefix appears with two or more distinct continuations across sessions, emit DAG"). Add a fixture case to AC-02 covering DAG detection. AC-05's "tested by fixture covering both shapes" then has an upstream detection path.

Either resolves the gap. Leaving AC-05 with an unwritten decision rule means the implementing agent owns a substrate decision the tabletop explicitly reserved (escalation #1).

---

## Honest Assessment

The v3 spec resolves v2 cleanly. The three v2 blockers — path-determinism for AC-09, audit-mutates invariants for AC-06, exact-ordering contradiction for AC-02 — are surgically addressed. The shape of the spec is now correct: gate AC up front, write/read split between the verify verb and the audit aggregator, chain_id as content-addressed substrate flowing from AC-04 to AC-09, AC types tagged where the tabletop intended.

The three v3 findings cluster around one shape — *load-bearing substrate referenced but not pinned in the AC text*. AC-01 path (c) names "session record" without a referent; AC-04's `chain_id` claims determinism without naming the hash; AC-05 splits chain vs DAG without naming the detection rule. None is architectural. Each is a single-paragraph edit the spec author can land in minutes. The pattern is the same as v2's: tabletop intent didn't fully survive promotion into AC prose.

The biggest residual risk is AC-04's `chain_id` algorithm (Finding 2). Like v2's path-determinism finding, this is the one whose failure mode is silent — two agents on the same chain produce different `chain_id`s and AC-09's deduplication silently produces duplicates. AC-01 path (c) and AC-05 chain/DAG will surface as escalations on first implementation; AC-04 can ship and fail in production. Worth fixing before drive proceeds.

Spec is one cycle from APPROVE.
