# Decision Pack — 0009 Mission Resilience

**Date:** 2026-04-20
**Card:** `.orbit/cards/0009-mission-resilience.yaml`
**Primary source:** `.orbit/discovery/mission-resilience.md`
**Rally:** orbit UX uplift via Claude Code tools (co-cards: 0003 implement session visibility, 0007 drive forked reviews, 0008 artefact folder)

This pack is a design-time record of choices that bound the forthcoming `spec.yaml`. It is not a spec and does not enumerate ACs. Seven decisions are recorded; the card's five scenarios are covered.

---

## Context at a glance

- **The failure mode is attention capture, not misunderstanding.** Discovery `§ Anatomy of Drift` identifies three phases: legitimate detour → attention capture → momentum continuation. The spec never "got wrong" — it got forgotten.
- **The current system has two half-mechanisms.** `plugins/orb/skills/implement/SKILL.md` step 5 tells the agent to update `progress.md` after each AC but does not force re-consultation after a detour. `plugins/orb/scripts/session-context.sh` (lines 215–261) surfaces hard constraints from the in-flight spec but does not read `progress.md` and does not surface *which AC is next*.
- **`ac_type: gate` already exists in the schema** but has no runtime meaning. It is declared in `plugins/orb/skills/spec-architect/SKILL.md` (line 36), documented in `plugins/orb/skills/audit/SKILL.md` (line 39), referenced by `plugins/orb/skills/review-pr/SKILL.md` (line 38), and appears on real ACs in `.orbit/specs/2026-04-19-rally/spec.yaml` (lines 27, 54, 65), `.orbit/specs/2026-04-20-orbit-artefact-folder/spec.yaml` (lines 72, 77, 102, 107), and `.orbit/specs/2026-04-20-drive-forked-reviews/spec.yaml` (line 109). Today `gate` only tells `/orb:audit` "don't expect a test". It does not block subsequent ACs.
- **Schema ownership (rally coordination).** This card owns the `progress.md` schema extensions. Card 0003 (implement session visibility, TaskCreate/Update/List) consumes them: its session task list mirrors the checklist and "current AC" marker defined here.

---

## Decision 1 — Re-anchoring locus

**Context.** The discovery note (Q1) asks where the "what AC comes next?" reminder should live: in `progress.md`, in `session-context.sh`, or as a rule in the implement skill. The author's apparent preference is **D. All three — belt, braces, suspenders**.

**Options considered.**

```
| Option | Summary                                             | Coverage                                         |
|--------|-----------------------------------------------------|--------------------------------------------------|
| A      | `progress.md` only — add a `current_ac:` pointer    | Passive; only helps if agent reads progress.md   |
| B      | `session-context.sh` only — surface next AC at start| Fires at session boundary only; misses mid-session|
| C      | implement skill only — prescribe re-anchoring rule  | Relies on agent discipline; no state on disk     |
| D      | All three, each with a distinct job                 | Every failure path caught by at least one layer  |
```

**Trade-offs.** D is more surface area (three touchpoints to keep in sync), but the failure modes are genuinely different. Phase 2 "attention capture" is interrupted by the skill rule (C). Phase 3 "momentum continuation" needs a file to re-anchor against (A). Session death + resume needs the hook to re-prime the next AC (B). Picking any single option leaves one failure path unguarded.

**Decision.** **Adopt option D.** Each layer has a non-overlapping responsibility:

- **`progress.md` (state)** carries a `Current AC:` pointer and a `## Detours` log with `Return to:` fields. This is the source of truth that survives session death.
- **`session-context.sh` (surface)** reads `progress.md` on session start and prints the next unchecked AC plus any blocking gate.
- **implement SKILL.md (behaviour)** prescribes "after any detour, re-read `progress.md` and the spec's AC list before choosing what to do next".

`progress.md` is the authority; the hook is a notifier; the skill rule is the loop-back behaviour. Naming them by role keeps implementation crisp.

---

## Decision 2 — Gate AC representation

**Context.** Discovery Q2 weighs `ac_type: gate` vs a new `gate: true` boolean. Scan results (above) confirm `ac_type: gate` is already defined and in use. Today it is a *classifier* ("no test expected") consumed only by `/orb:audit` (`plugins/orb/skills/audit/SKILL.md` line 39) and `/orb:review-pr` (`plugins/orb/skills/review-pr/SKILL.md` line 38). It is not yet an *enforcer*.

**Options considered.**

```
| Option | Summary                                                        | Schema impact            |
|--------|----------------------------------------------------------------|--------------------------|
| A      | Overload `ac_type: gate` with blocking semantics               | None — reuses existing   |
| B      | Add a new `gate: true` boolean orthogonal to ac_type           | Adds a second field      |
| C      | Overload `ac_type: gate` + optional `blocks: [ac-05, ac-07]`   | Extends existing AC      |
```

**Trade-offs.** Option B introduces a redundant axis: a `gate` AC by definition is a manual/process gate (that is already its meaning in `audit/SKILL.md`). The fact that today the only consequence is "exempt from test expectation" is an under-use, not a conflict. Option C (named dependency list) is more expressive, but none of the four existing specs declare dependencies between ACs — they rely on ordinal sequence. Adding an optional dependency list today is speculative.

**Decision.** **Adopt option A with an implicit-ordering rule.** An `ac_type: gate` AC blocks every AC that follows it in declaration order until it is marked complete in `progress.md`. This matches the discovery example (`AC-01 (gate) → Do not proceed to AC-05 until verified`) and matches the author's preference in the discovery note (Q2 option A).

**Exact YAML (unchanged from current schema).**

```yaml
acceptance_criteria:
  - id: ac-01
    ac_type: gate
    description: "Prerequisite verification complete for dataset X"
    verification: "All preflight checks pass; author has signed off in progress.md"
  - id: ac-05
    ac_type: code
    description: "Core computation produces expected parity with reference run"
    verification: "Unit test ac05_* matches reference output within tolerance"
```

The blocking rule is enforced at the implement-skill level (Decision 6), not encoded in a new field.

**Forward compatibility note.** If future specs need non-adjacent gating ("gate blocks ac-05 but not ac-02"), option C (`blocks: [...]`) is an additive extension that does not break option A.

---

## Decision 3 — Drift detection mechanism

**Context.** Discovery Q3 asks how spec modifications during implementation should be detected. Three candidates: implement-skill hash check, git-status in SessionStart, Monitor tool watching the spec file.

**Options considered.**

```
| Option | Summary                                                 | Detection window   | Dependencies          |
|--------|---------------------------------------------------------|--------------------|-----------------------|
| A      | implement-skill hash: record hash at start, recheck     | Before each AC     | None (shell)          |
| B      | session-context git-status: compare mtime vs review file| Session start only | git, review-spec-*.md |
| C      | Monitor tool watching the spec file                     | Real-time          | Monitor tool, session |
```

**Trade-offs.**

- **A** is the simplest. It catches the exact scenario from the discovery note (spec modified mid-flight without re-review) at the natural checkpoint (before starting each AC). It requires no git assumptions and survives outside repos.
- **B** ties detection to session start — useless if the spec is modified mid-session, which is the exact observed case. It also silently depends on the existence of a `review-spec-*.md` file, which may not exist if review was skipped.
- **C** is real-time but introduces a harness dependency (Monitor) and requires a persistent watcher across tool calls, which is heavy for a single-file concern.

**Decision.** **Adopt option A (implement-skill hash) as primary, with option B (session-context hash surfacing) as a secondary belt.** Scenario 1 of the card ("Mid-flight spec modification is detected") aligns with A; the card's own `source_lines` quote mentions B. Running both is cheap and symmetric with Decision 1's "three layers, three roles":

- **implement skill** computes `sha256(spec.yaml)` at the start of the session and writes it to `progress.md` as `Spec hash:` metadata. Before each AC it re-reads the file, re-hashes, and if the hash differs it surfaces a REQUEST_CHANGES-style notice: `"spec modified since implementation started, re-review recommended"` and checkpoints with the author before continuing.
- **session-context hook** on resume reads the recorded hash from `progress.md` and compares against the current file. If mismatched, surface the same warning so the resuming agent sees it immediately.

Monitor (option C) is explicitly deferred — too heavy for this first pass.

---

## Decision 4 — Detour log placement

**Context.** Discovery Q4 weighs an in-`progress.md` detour section vs a separate `findings.md`. Author preference (option A) is stated in the discovery note.

**Options considered.**

```
| Option | Summary                                             | Files to track | Return-to field |
|--------|-----------------------------------------------------|----------------|-----------------|
| A      | `## Detours` section inside progress.md             | 1              | Inline          |
| B      | Separate `findings.md` alongside progress.md        | 2              | Cross-file ref  |
```

**Trade-offs.** `progress.md` is the single file session resume already knows about (consumed by `/orb:review-pr` per `implement/SKILL.md` line 128, and — per this pack — by the session hook and the implement skill). Keeping detours there preserves the single source-of-truth invariant. A separate `findings.md` splits state: the "Return to" field must now cross files, and the session hook has two places to read. The only upside is "keeps progress.md clean" — a cosmetic concern against a real integrity concern.

**Decision.** **Adopt option A.** Detours go in a `## Detours` section of `progress.md`. Each detour entry carries a `Return to:` field naming the AC ID to resume against.

**Schema.** See Decision 5 for the exact template.

---

## Decision 5 — `progress.md` schema extensions (authoritative)

**Context.** This card is the schema owner for rally coordination. Card 0003's session task list mirrors this schema. Today the implement skill emits a minimal progress template (`implement/SKILL.md` step 4): spec path, started date, hard constraints checklist, AC checklist. This pack extends it.

**Required additions.**

1. `**Spec hash:**` metadata field — holds `sha256:<hex>` of the spec file as of session start. Enables Decision 3's drift check.
2. `**Current AC:**` pointer — holds the AC ID the agent is currently working against (`ac-01`, `ac-02`, …). Updated whenever the agent advances to a new AC. Empty or `none` when nothing is in-flight.
3. `## Detours` section — inserted between the Hard Constraints checklist and the Acceptance Criteria checklist. Entries are date-stamped, carry a short description, and end with `Return to: <ac-id>`.
4. Gate markers in the AC checklist — the gate annotation is preserved as a human-readable tag next to the AC description: `- [ ] ac-01 (gate): <description>`. This mirrors the prose convention already seen in the discovery note while the authoritative signal lives in `spec.yaml`'s `ac_type`.

**Authoritative template.**

```markdown
# Implementation Progress

**Spec:** .orbit/specs/YYYY-MM-DD-<topic>/spec.yaml
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

**Mutation rules (for implement skill and session-context hook).**

- `Current AC:` is updated when the agent starts work on a new AC and when a detour is closed (set to the `Return to:` target, not the last AC in flight).
- A detour entry is appended when the agent decides unplanned work is legitimate; it is closed (not removed) when resolved, and the `Return to:` field stays as the audit trail.
- The Acceptance Criteria list order is stable — adding detours never reorders ACs, because the implicit gate-ordering rule (Decision 2) relies on declaration order.

**Card 0003 consumer contract.** The session task list in card 0003 is built by mapping this file: one task per hard constraint (status mirrors `[ ]`/`[x]`), one task per AC (status mirrors `[ ]`/`[x]`, gate flag surfaced in task label), plus a single pinned task `"Current AC: <id>"` reflecting the pointer. Detour entries are not tasks — they are context, not work items.

---

## Decision 6 — Gate enforcement locus

**Context.** Once `ac_type: gate` has blocking semantics (Decision 2), *something* must enforce "don't start ac-NN while a preceding gate is open". Candidates: implement skill, spec-architect validation, review-spec, or the session-context hook.

**Options considered.**

```
| Option | Summary                                             | Enforced when           |
|--------|-----------------------------------------------------|-------------------------|
| A      | implement skill — check before starting each AC     | Runtime (every AC step) |
| B      | spec-architect — validate gate placement at spec gen| Design time             |
| C      | review-spec — Pass-1 structural check flags gates   | Review time             |
| D      | All three, each at its natural phase                | Design + review + runtime|
```

**Trade-offs.** B and C are about *spec validity* ("does the spec declare gates sensibly?"), which is a different concern from A ("is the agent honouring the gate right now?"). The observed failure in the discovery note is a runtime failure — AC-01's gate was declared and was still skipped. So A is load-bearing. B and C are cheap additions once gates have blocking semantics: spec-architect already emits `ac_type`; review-spec Pass 1 already scans constraint conflicts and would naturally extend to "gate without prerequisites declared".

**Decision.** **Primary = A (implement skill enforces at runtime). Secondary = C (review-spec Pass 1 notes gate placement).** Skip B for now — spec-architect does not need new logic, it just needs to keep emitting `ac_type: gate` correctly (which it already does).

**Runtime rule (implement SKILL.md).** Before starting any AC, the skill walks the AC list in order and asserts: *for every AC preceding the current one with `ac_type: gate`, the progress.md checkbox is `[x]`*. If any gate is open, the skill refuses to start the current AC and surfaces the blocking gate's ID. The author can unblock by completing the gate or by explicitly authorising the skip (which gets recorded as a detour entry with `Return to:` pointing at the gate).

**Review-spec note (Pass 1).** Add a structural check: "If `ac_type: gate` is used, the verification field must describe what being 'complete' means." This prevents gates that are vague enough to be rubber-stamped.

---

## Decision 7 — Rally-level AC progress visibility

**Context.** Discovery Q5 asks whether `rally.yaml` should track per-card AC progress. The discovery note itself calls this "probably a second-order concern".

**Options considered.**

```
| Option | Summary                                                      | Complexity added   |
|--------|--------------------------------------------------------------|--------------------|
| A      | Defer — rally tracks card-level status only (as today)       | None               |
| B      | Extend rally.yaml with per-card AC counters (e.g. 3/8)       | Moderate           |
| C      | Extend rally.yaml to mirror each card's `Current AC:` pointer| High — two-way sync|
```

**Trade-offs.** The session-context hook (lines 83–108) already surfaces per-card status in the rally display. AC-level granularity from rally.yaml would duplicate what `progress.md` already holds. Two-way sync (option C) creates a new failure mode (stale pointer) for marginal benefit — the author can read `progress.md` directly when they care. Option B (counters) is cheaper but still requires rally.yaml to be updated on every AC tick, dragging complexity across two files.

**Decision.** **Adopt option A — defer.** Rally continues to track card status (`queued`, `in-progress`, `complete`, `parked`). The session hook's rally block surfaces one line per card; if the author wants AC detail they open the card's `progress.md`. If a future rally tops out at 5+ cards and drift becomes visible only at the AC level, reopen this decision with evidence.

**Recommendation for card wording.** The mission-resilience card's five scenarios do not include a rally-level one. Keep it that way. If rally interaction surfaces a need, it belongs on the rally card, not here.

---

## Summary table

```
| # | Decision                                       | Recommended                   | Discovery Q |
|---|------------------------------------------------|-------------------------------|-------------|
| 1 | Re-anchoring locus                             | All three layers (D)          | Q1          |
| 2 | Gate AC representation                         | Reuse `ac_type: gate` (A)     | Q2          |
| 3 | Drift detection mechanism                      | implement-hash + hook surface | Q3          |
| 4 | Detour log placement                           | `## Detours` in progress.md   | Q4          |
| 5 | progress.md schema extensions (authoritative)  | Template above                | (new)       |
| 6 | Gate enforcement locus                         | implement (runtime) + review  | (new)       |
| 7 | Rally-level AC progress visibility             | Defer                         | Q5          |
```

## Scenario coverage

```
| Card scenario                                      | Covered by                                     |
|----------------------------------------------------|------------------------------------------------|
| Mid-flight spec modification is detected           | Decision 3 (hash check) + Decision 1 (hook)    |
| Gate ACs block subsequent ACs                      | Decision 2 (schema) + Decision 6 (enforcement) |
| Detours are recorded in progress.md                | Decision 4 + Decision 5 (schema)               |
| Agent re-anchors after detour                      | Decision 1 (belt+braces) + Decision 5 (pointer)|
| Session resume surfaces next AC                    | Decision 1 (hook) + Decision 5 (Current AC)    |
```

All five scenarios are covered. The spec author can now lift these decisions into ACs, constraints, and verification methods without revisiting the design questions.

---

## Coordination notes for the rally

- **0003 (implement session visibility)** consumes Decision 5's schema. Its session task list mirrors the hard constraints + AC checklist + `Current AC:` pointer. Detours are **not** tasks.
- **0007 (drive forked reviews)** is unaffected; gate blocking runs inside the implement phase, after review-spec has already produced its verdict.
- **0008 (artefact folder)** is the path/layout story; mission resilience only adds one more file section to `progress.md`, which already lives under `.orbit/specs/<slug>/`. No path change required.
- **Schema authority:** any change to `progress.md`'s structure after this pack lands must route through this card (or a successor spec under `.orbit/specs/2026-04-20-mission-resilience/`). Card 0003's spec should reference this pack as a hard dependency.
