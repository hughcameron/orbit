## Design: Mission Resilience

**Date:** 2026-04-20
**Interviewer:** nightingale (rally sub-agent)
**Card:** orbit/cards/0009-mission-resilience.yaml

---

## Context

Card: *Mission resilience — agents stay anchored to the spec through disruptions* — 5 scenarios, goal: agents maintain spec fidelity through mid-flight disruptions without requiring the author to pull them back on track.

Prior specs: 0 — this is the first spec against card 0009. Discovery note `orbit/discovery/mission-resilience.md` documents the observed failure (AC sequence skipped after a legitimate data-quality detour) and catalogues what the current system provides vs. what is missing.

Gap: no mechanism says "you just finished a detour — re-read the spec and progress.md, what is the next AC?" `ac_type: gate` exists as a classifier in the schema but has no runtime blocking semantics. `session-context.sh` surfaces hard constraints but not the next unchecked AC. progress.md is written but nothing forces re-consultation after unplanned work. Mid-flight spec edits bypass review. This session is about wiring those pieces into a single, belt-and-braces anchoring system and fixing this card as the schema owner of `progress.md` extensions for the rally.

## Q&A

### Q1: Re-anchoring locus
**Q:** The "what AC comes next?" reminder could live in `progress.md` (state), `session-context.sh` (surface), or the implement skill (behaviour). Discovery Q1 floated a fourth option: all three, each with a distinct job. Which?
**A:** All three — option D, belt, braces, suspenders. Each failure phase from the discovery note needs a different safety net: phase 2 "attention capture" is interrupted by the skill rule, phase 3 "momentum continuation" needs a file to re-anchor against, and session-death + resume needs the hook to re-prime the next AC. Non-overlapping responsibilities: `progress.md` is the authority (carries `Current AC:` + `## Detours` with `Return to:`), `session-context.sh` is the notifier (reads progress.md on resume and prints the next unchecked AC + any blocking gate), implement SKILL.md prescribes the loop-back behaviour (after any detour, re-read progress.md and the AC list before choosing what to do next).

### Q2: Gate AC representation
**Q:** `ac_type: gate` already exists in the schema and appears on real ACs, but today it only tells `/orb:audit` "don't expect a test" — it does not block. Do we overload the existing enum (option A), add an orthogonal `gate: true` boolean (option B), or overload plus an optional `blocks: [ac-05, ac-07]` dependency list (option C)?
**A:** Option A — reuse `ac_type: gate` with an implicit-ordering rule. A gate AC blocks every AC that follows it in declaration order until marked complete in progress.md. This matches the discovery example (`AC-01 (gate) → Do not proceed to AC-05 until verified`) and avoids a redundant axis (option B) or speculative dependency lists (option C, which no current spec uses). No schema churn — the YAML stays exactly as it is today; the blocking rule is enforced at the implement-skill level.

### Q3: Drift detection mechanism
**Q:** Mid-flight spec edits went through without re-review. Three detection candidates: implement-skill hash check before each AC (option A), session-context git-status comparison at session start (option B), or Monitor tool watching the spec file in real time (option C). Which?
**A:** Option A (implement-skill hash) as primary, with option B (session-context surfacing the same hash on resume) as a secondary belt. The implement skill computes `sha256(spec.yaml)` at session start, writes it to `progress.md` as `Spec hash:` metadata, and re-hashes before each AC; on mismatch it surfaces a REQUEST_CHANGES-style notice ("spec modified since implementation started, re-review recommended") and checkpoints. The session-context hook reads the recorded hash on resume and surfaces the same warning. Monitor (option C) is too heavy for a single-file concern — deferred.

### Q4: Detour log placement
**Q:** Where should detour entries live: inside `progress.md` as a `## Detours` section (option A), or in a separate `findings.md` (option B)?
**A:** Option A — `## Detours` section inside progress.md. progress.md is already the single file session resume knows about (consumed by `/orb:review-pr`, the session hook, and the implement skill). Splitting state into two files forces the `Return to:` field to cross files and gives the session hook two places to read. "Keeps progress.md clean" is cosmetic against a real integrity concern; single source-of-truth wins. Each detour entry carries a `Return to: <ac-id>` field — that field is the re-anchoring mechanism that survives session death.

### Q5: progress.md schema extensions (authoritative)
**Q:** This card is the schema owner for rally coordination — card 0003's session task list mirrors whatever shape we fix here. What fields does progress.md gain, and what is the authoritative template?
**A:** Four additions, authoritative template below:

1. `**Spec hash:**` metadata field — `sha256:<hex>` of the spec at session start, enables Decision 3's drift check.
2. `**Current AC:**` pointer — the AC ID the agent is working against; updated on AC advance and when a detour closes (set to the `Return to:` target, not the last AC in flight).
3. `## Detours` section — inserted between Hard Constraints and Acceptance Criteria. Entries are date-stamped, carry a short description, end with `Return to: <ac-id>`. Closed detours are retained, not removed — the audit trail matters.
4. Gate markers in the AC checklist — `- [ ] ac-01 (gate): <description>` as a human-readable tag; the authoritative signal remains `ac_type: gate` in `spec.yaml`.

```markdown
# Implementation Progress

**Spec:** orbit/specs/YYYY-MM-DD-<topic>/spec.yaml
**Spec hash:** sha256:<hex>
**Started:** YYYY-MM-DD
**Current AC:** ac-01

## Hard Constraints
- [ ] <constraint text>

## Detours
<!-- Empty until a detour is recorded. Append newest last. -->
- YYYY-MM-DD: <short description of unplanned finding and its resolution>
  Return to: ac-01

## Acceptance Criteria
- [ ] ac-01 (gate): <description>
- [ ] ac-02: <description>
- [ ] ac-03: <description>
```

AC list order is stable — detours never reorder ACs, because the implicit gate-ordering rule relies on declaration order. Card 0003's consumer contract: one task per hard constraint, one task per AC (gate flag surfaced in label), one pinned task `"Current AC: <id>"`. Detours are context, not tasks.

### Q6: Gate enforcement locus
**Q:** Once gates have blocking semantics (Q2), who enforces "don't start ac-NN while a preceding gate is open"? Implement skill at runtime (A), spec-architect at design time (B), review-spec at review time (C), or all three (D)?
**A:** Primary = A (implement skill enforces at runtime). Secondary = C (review-spec Pass 1 adds a structural check). Skip B — spec-architect already emits `ac_type: gate` correctly and needs no new logic. The runtime rule: before starting any AC, the implement skill walks the AC list in order and asserts that for every preceding `ac_type: gate` AC, the progress.md checkbox is `[x]`. If any gate is open, the skill refuses to start the current AC and surfaces the blocking gate's ID. The author can unblock by completing the gate or by explicitly authorising the skip — which gets recorded as a detour entry with `Return to:` pointing at the gate. Review-spec Pass 1 adds: "If `ac_type: gate` is used, the verification field must describe what being 'complete' means" — prevents vague rubber-stampable gates.

### Q7: Rally-level AC progress visibility
**Q:** Should `rally.yaml` track per-card AC progress — defer entirely (A), add per-card AC counters like `3/8` (B), or mirror each card's `Current AC:` pointer into rally.yaml (C)?
**A:** Option A — defer. The session-context hook already surfaces per-card status in the rally display; AC-level granularity from rally.yaml would duplicate what `progress.md` already holds. Option C introduces two-way sync (stale-pointer failure mode) for marginal benefit; option B still requires rally.yaml to update on every AC tick. Rally continues to track card status only (`queued`, `in-progress`, `complete`, `parked`). If a future rally tops out at 5+ cards and drift becomes visible only at the AC level, reopen with evidence. The card's five scenarios do not include a rally-level one — keep it that way.

---

## Summary

### Goal

Agents maintain spec fidelity through mid-flight disruptions — spec edits, unplanned findings, session death — without requiring the author to pull them back on track. The failure mode being addressed is attention capture and momentum continuation after legitimate detours, not misunderstanding of the spec.

### Constraints

- **This card owns the `progress.md` schema; future changes route through here.** Any structural change to `progress.md` after this spec lands must route through card 0009 (or a successor spec under `orbit/specs/2026-04-20-mission-resilience/`). Card 0003 (implement session visibility) consumes this schema as a hard dependency and must not fork it.
- Three-layer implementation is non-negotiable: progress.md (state authority) + session-context.sh (notifier on resume) + implement SKILL.md (mid-session loop-back behaviour). Each layer owns a distinct failure path.
- No spec YAML schema changes. `ac_type: gate` is reused as-is; blocking semantics are enforced at the implement-skill layer via implicit declaration-order ordering.
- Detours live inside `progress.md` (single source of truth). A separate `findings.md` is explicitly rejected.
- Drift detection is hash-based (`sha256` of `spec.yaml`), stored in `progress.md` as `Spec hash:`, recomputed before each AC by the implement skill and on resume by the session-context hook. Monitor-tool real-time watching is deferred.
- Closed detour entries are retained, not removed — the audit trail is load-bearing.
- AC declaration order is stable; detour logging must never reorder ACs (gate blocking depends on order).
- Rally-level AC-progress visibility is out of scope — `rally.yaml` tracks card status only.

### Success Criteria

- All five card scenarios are covered by ACs in the resulting spec:
  1. Mid-flight spec modification detected — hash mismatch surfaces re-review notice and checkpoints the author.
  2. Gate ACs block subsequent ACs — implement skill refuses to start any AC while a preceding `ac_type: gate` is `[ ]`, surfacing the blocking AC's ID.
  3. Detours recorded in progress.md — `## Detours` section populated with `Return to: <ac-id>` on every unplanned work item.
  4. Agent re-anchors after detour — after resolving a detour, agent re-reads progress.md and the AC list, sets `Current AC:` to the `Return to:` target, and resumes there rather than continuing in the detour's direction.
  5. Session resume surfaces next AC — SessionStart hook reads progress.md and prints the next unchecked AC (including any blocking gate) alongside hard constraints and the drift warning if the hash differs.
- `progress.md` template in Q5 lands verbatim as the authoritative schema.
- Card 0003's consumer contract (tasks mirror hard constraints + AC checklist + `Current AC:` pointer; detours are not tasks) is explicitly referenced in the spec as a cross-card dependency.
- Review-spec Pass 1 gains the "gate verification must describe completeness" structural check.

### Decisions Surfaced

- **D1 Re-anchoring locus:** chose three-layer (progress.md + session-context.sh + implement SKILL.md) over any single-layer option — each layer guards a non-overlapping failure phase (→ decisions.md §Decision 1).
- **D2 Gate AC representation:** chose `ac_type: gate` reuse with implicit declaration-order blocking over a new `gate:` boolean or a `blocks: [...]` dependency list — no schema churn; expressiveness not yet needed (→ decisions.md §Decision 2).
- **D3 Drift detection mechanism:** chose implement-skill sha256 hash (primary) + session-context hook surfacing on resume (secondary) over Monitor real-time watching — cheapest symmetric coverage (→ decisions.md §Decision 3).
- **D4 Detour log placement:** chose `## Detours` section inside progress.md over a separate `findings.md` — preserves single-source-of-truth invariant (→ decisions.md §Decision 4).
- **D5 progress.md schema extensions:** fixed the authoritative template with `Spec hash:`, `Current AC:`, `## Detours`, and gate markers in the AC checklist; card 0009 is the schema owner for rally coordination and card 0003 consumes it (→ decisions.md §Decision 5).
- **D6 Gate enforcement locus:** chose implement skill at runtime (primary) + review-spec Pass 1 structural check (secondary); spec-architect needs no change (→ decisions.md §Decision 6).
- **D7 Rally-level AC progress visibility:** deferred — `rally.yaml` continues to track card-level status only; reopen with evidence if needed (→ decisions.md §Decision 7).

### Open Questions

None blocking. One forward-compatibility note carried forward from Decision 2: if future specs need non-adjacent gating ("gate blocks ac-05 but not ac-02"), option C (`blocks: [ac-05, ac-07]`) is an additive extension that does not break the option A implicit-ordering rule chosen here. No action required now; revisit only if a concrete spec needs it.
