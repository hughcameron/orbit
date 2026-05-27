# Tabletop — Memory cite-reading

**Date:** 2026-05-27
**Cards in scope:** 0037-memory-gates-decisions
**Output spec:** .orbit/specs/2026-05-27-memory-cite-reading/spec.yaml
**Input:** .orbit/memos/2026-05-17-read-the-cited-source.md (to be consumed on spec close)

---

## Capability ambition

Card 0037's first spec (2026-05-19) hardened how memories *surface* and *block* at decision moments — but it assumed the memory's body carries everything actionable. This work extends 0037 to make memory reconciliation *robust against compression*: when a memory points at an external authoritative source, the agent reads the source itself, and the substrate makes both the citation and the read structurally detectable.

One sentence: *When a memory cites an external doc as the load-bearing fix, the agent reads that doc before reconciling the memory — and the substrate makes both the cite and the read detectable.*

One-spec defence: moves (2) cite-reading and (3) close-gate enforcement depend on move (1) detectable cite shape. Without (1), the other two cannot fire reliably. This is one cohesive spec via technical dependency.

## Values

**Load-bearing:** diagnostic honesty under memory compression — when a memory cites an authoritative source, reconciliation reflects the source's content, not the summary's framing.

Subordinate:
- **Detectability** — cite-shape is machine-readable; enabling condition for any gate.
- **Structural over behavioural** — closed ac-06 of 2026-05-19 established this for card 0037; this spec must hold the same posture.
- **Backward compatibility** — existing memories without cites continue to function; cite-shape is opt-in additive.

## Trade-offs

- **Schema churn** — `cites:` field on memory record touches `orbit memory remember`, SQLite schema, format docs. One-time cost. *Acceptable.*
- **Reconciliation latency** — reading the cited doc adds time; mitigated by per-reconciliation cache. *Expensive-but-worth-it.*
- **Forward-only convention** — old memories don't carry `cites:`; reconciler can't retro-detect without a heuristic pass. Old memories operate under closed-spec rules. *Acceptable.*
- **False-negative risk** — author writes a cite in prose, not the formal field. Reconciler misses it. *Acceptable*, same forward-only posture; convention spreads via skill-prompt examples.

## Halt conditions

- **Cite-shape exists but reconciler doesn't read it.** Pure ceremony — field gets populated, behaviour unchanged. Halt: verify the read path fires on at least one fixture before any AC ticks.
- **Cite-read evidence not actually captured.** Evidence records "cite read" but the agent never opened the file. Halt: verification of memories_considered cite-evidence entries must include a content-derived value (excerpt or hash), not just a boolean.
- **Backward-compat break.** Old cite-less memories fail to load or get spuriously flagged. Halt: regression suite must run green against the current `.orbit/memories/` corpus.
- **Scope creep into memory-hygiene-at-write-time.** Author rejected hygiene-nudge in Q1. Halt: any AC that requires authors to add cites to *new* memories is out of scope; surface for re-tabletop.

## Escalation triggers

- **Cited source unreadable** during reconciliation. Surface: which memory, cite path, reason. Action: `(a) record-as-unreadable + continue` / `(b) update memory cite-path` / `(c) drop from memories_considered with NA`.
- **Cite-read evidence ambiguous on applicability.** Agent reads the cite but can't tell if it supports or contradicts the current approach. Surface: cite excerpt, current approach. Action: `(a) adopt` / `(b) partial-adopt with named divergence` / `(c) NA — cite irrelevant`.
- **Excerpt-evidence shape proves fakeable.** Implementation reveals excerpts are too easily synthesised without reading. Surface: instance of suspected fake-evidence. Action: pivot to hash-verification (kill-condition #2).
- **`memories_considered` field shape can't carry cite-evidence cleanly.** Existing field schema from 2026-05-19 was sized for adopted/partial/NA only. Surface: current shape, proposed extension. Action: extend in this spec OR carve as a prerequisite spec.

Pre-decided (not an escalation): forward-only migration. Old memories without `cites:` are not retro-migrated. Per Q3 trade-off.

## Kill conditions

- **Structured `cites:` field is not meaningfully distinct from prose paths.** Kill if URL/path detection in prose proves equally reliable across the corpus. Pivot: ship cite-detection as a parser pass; drop the schema field.
- **Excerpt evidence does not prove cite-read.** Kill if review of evidence shows fluent fakes pass undetected. Pivot: hash-based content verification (cite-read evidence carries `sha256(file_contents_at_read_time)`).
- **`spec.close` pre-flight cannot extend cleanly.** Kill if the closed 2026-05-19 ac-04 implementation shape can't accommodate cite-evidence checks without refactor. Pivot: introduce a separate `pre_close_check_memory_cite_evidence` pass alongside the existing one.
- **One spec is the wrong scope.** Kill if cite-shape schema work alone is a session of effort. Pivot: carve into cite-shape spec + cite-evidence spec; this tabletop's sidecar then drives only the first.

## Verification posture

Every scenario the spec covers carries one classification:

- *`cites:` round-trips through remember→match* — `verifies: capability`
- *`orbit memory match` exposes cite-shape on output* — `verifies: capability`
- *`/orb:spec` reads cited source + records evidence* — `verifies: stand-in (real thing is agent internalising the cite), accepted because evidence-of-read is the most we can structurally enforce; internalisation is a downstream behavioural concern outside this spec's gate*
- *`spec.close` refuses on missing cite-read evidence* — `verifies: capability`
- *Existing cite-less memories regression-free* — `verifies: capability`

## Implementation notes

(Routed from Q8 — adjacent code; file-level only, no contract content.)

- Memory record: extend struct in `orbit-state/crates/core/src/memory.rs` (or canonical location) with `cites: Option<Vec<Cite>>` where `Cite { path: String, // resolves to local file relative to repo root for v1 }`.
- SQLite schema: new column (JSON-encoded `cites` blob is simpler than relational; matches existing pattern for `labels`).
- CLI surface: `orbit memory remember --cite <path>` (repeatable for multiple cites).
- `orbit memory match` JSON output: each result carries `cites: []` (empty for cite-less memories).
- `orbit memory show`: surface cites in human output.
- `Spec.memories_considered` entry extension: add optional `cite_evidence: [{ cite_path: String, excerpt: String, read_at: timestamp }]` shape. Empty / absent for memories without `cites:`.
- `orbit spec close` pre-flight: extend the existing memories-reconciliation check (from 2026-05-19 ac-04) with a second pass — for each considered memory carrying `cites:`, verify `memories_considered.cite_evidence` covers each cite.
- `/orb:spec` SKILL.md: prose addition naming the cite-read step and the `cite_evidence` shape; per-AC verification classification convention applies (already shipped 0.4.38).
- MCP server: parity for `--cite` flag on remember and the new field on match output.
- Tests: round-trip (cite written → stored → loaded → matched → cite intact); match output exposes cite-shape; spec.close refuses on missing cite-evidence; regression against existing memory corpus.

## Hot-wash

- **recurred:** "structural over behavioural" appeared in Q2, Q4, Q5, Q7 — every option that drifted toward prose-only enforcement got rejected. Consistent with card 0037's closed ac-06 finding.
- **surprised:** the Q1 broaden-to-hygiene option was a real lateral; rejecting it cleanly showed the 0.4.38 scope discipline is biting in live use.
- **friction:** distinguishing "cite-shape detection" from "cite-read enforcement" — conceptually separable, technically coupled (you can't enforce reading what you can't detect). Kept coupled; the technical-dependency defence held.
- **meta-patterns-for-future-tabletops:** prior closed-spec findings become first-class inputs to new tabletops. Card 0037's ac-06 was cited as load-bearing rationale multiple times — should this be a structural step in the tabletop walking (e.g. read the card's closed specs before Q1)?
