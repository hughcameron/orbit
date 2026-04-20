---
name: implement
description: Pre-flight spec check — extract ACs and constraints as a tracked checklist before writing any code
---

# /orb:implement

Read a spec, extract its acceptance criteria and constraints as a checklist, and track progress throughout implementation. This is the pre-flight equivalent of `/orb:review-pr` — it ensures the spec is loaded before any code is written.

## Usage

```
/orb:implement <spec_path>
```

Where `<spec_path>` is the path to a `spec.yaml` file (e.g. `orbit/specs/2026-04-04-topic/spec.yaml`).

## Why This Exists

Without a pre-flight check, implementing agents treat the codebase as ground truth and miss spec-prescribed patterns. This skill forces the spec into working memory before implementation begins, and keeps it there throughout.

> **Reference incident:** An agent treated the codebase as ground truth and missed a spec-prescribed entrypoint pattern.

## Instructions

### 1. Read the Spec

Read the spec YAML file provided via `$ARGUMENTS`.

- If no argument is provided: look for the most recent `orbit/specs/*/spec.yaml`
- If no spec exists: stop and tell the author — there's nothing to implement against

Extract:
- `goal` — the primary objective
- `constraints` — the hard limitations (these are non-negotiable)
- `acceptance_criteria` — every `ac-NN` with its description
- `metadata.test_prefix` — if present, use this prefix in test function names (e.g., `remat_ac01_*`)
- `deliverables` — what files need to be created or modified

### 2. Search for Related Code

Before writing anything, run a keyword scan (see `/orb:keyword-scan`) against the project source using terms from the spec's goal and AC descriptions. This surfaces existing code, patterns, and tests the implementation should build on — rather than reimplementing what already exists.

### 3. Present the Checklist

**Before any code is written**, present the full checklist:

```
## Pre-Flight Checklist

**Goal:** <goal from spec>
**Test prefix:** <test_prefix from metadata, or "none">

### Hard Constraints
- [ ] <constraint 1>
- [ ] <constraint 2>

### Acceptance Criteria
- [ ] ac-01: <description>  →  test: <prefix>_ac01_*
- [ ] ac-02: <description>  →  test: <prefix>_ac02_*
- [ ] ac-03: <description>  →  test: <prefix>_ac03_*

### Deliverables
- <path 1>: <description>
- <path 2>: <description>
```

If `test_prefix` is present, all test function names for this spec must use the prefix (e.g., `test_remat_ac01_creates_structure`). This prevents AC ID collisions across specs in multi-spec projects.

Then proceed immediately to writing the progress file and implementing — do not wait for confirmation.

### 4. Write the Progress File

Create `progress.md` in the same directory as the spec. This is the **authoritative template** — the schema is owned by card 0009 (mission-resilience) and consumed by card 0003 (implement session visibility). Every field and section below is load-bearing.

```markdown
# Implementation Progress

Spec path: <path to spec.yaml>
Spec hash: sha256:<hex>
Started: <today's date>
Current AC: <ac-id | none>

## Hard Constraints
- [ ] <constraint text>

## Detours

## Acceptance Criteria
- [ ] ac-01 (gate): <description>
- [ ] ac-02: <description>
```

The metadata fields (`Spec path:`, `Spec hash:`, `Started:`, `Current AC:`) are **plain text, not bold** — this is a deliberate contract so the `Spec hash:` line matches the ac-01 regex `^Spec hash: sha256:[0-9a-f]{64}$` exactly, and so the `session-context.sh` parser's literal grep stays simple.

**Field semantics (non-negotiable — card 0009 owns this schema):**

- **`Spec hash: sha256:<hex>`** — the sha256 hex digest of `spec.yaml` computed over **raw bytes as read from disk in binary mode**. No line-ending conversion, no trimming, no YAML canonicalisation. Cross-platform checkouts may produce occasional false drift; this is the accepted trade-off.
- **`Current AC: <ac-id | none>`** — the AC currently in flight. Updated when advancing and immediately after a detour append (set to the detour's `Return to:` target).
- **`## Detours`** — positioned strictly between `## Hard Constraints` and `## Acceptance Criteria`. Entries are atomic and terminal — appended only at the moment unplanned work is resolved. Format:
  ```
  YYYY-MM-DD: <one-line description>
  Return to: <ac-id>
  ```
  Never edit or remove an entry after writing. There is **no** `Status:` field, **no** open/closed distinction — "closing a detour" and "appending the entry" are the same event. A separate `findings.md` is forbidden.
- **Gate annotation** — an AC whose `ac_type: gate` is rendered with the human-readable tag `(gate)`: `- [ ] ac-NN (gate): <description>`. The authoritative signal remains the `ac_type` value in `spec.yaml`; the annotation is documentation only.

All items start as `- [ ]` (pending). Constraint items mirror each constraint from `spec.yaml`.

#### 4a. Canonical drift-notice string

The single canonical source for the drift-notice string lives in this skill:

> **DRIFT_NOTICE** = `spec modified since implementation started, re-review recommended`

This exact string is emitted by the implement skill on drift (ac-02) and referenced by literal inclusion in `plugins/orb/scripts/session-context.sh` (ac-03). Test fixtures for `mrl_ac02` and `mrl_ac03` import or copy the same string. Any change to the wording is a schema change that routes through card 0009.

#### 4b. Pre-AC check sequence (fixed)

Before starting any AC, the implement skill runs **three checks in this exact order**. The ordering is pinned so user-visible log causality is deterministic across implementations.

1. **Backfill Spec hash if absent (ac-12).** If the `Spec hash:` line is missing from `progress.md` (a legacy in-flight file from before ac-01), compute `sha256(spec.yaml raw bytes)`, insert the `Spec hash:` line in the file header, and emit the log line `Backfilled Spec hash for existing progress.md`. Do **NOT** emit the drift notice on this write — the hash was never recorded, so there is nothing to disagree with. Once backfilled, the file is indistinguishable from a fresh write.
2. **Drift check (ac-02).** If the `Spec hash:` line is present, recompute `sha256(spec.yaml raw bytes)` and compare. On mismatch, emit the canonical `DRIFT_NOTICE` string (stderr/stdout), then branch by interactivity:
   - **Interactive path** (TTY on stdin **AND** `ORBIT_NONINTERACTIVE` is not set to `1`): call `AskUserQuestion` with exactly two options:
     - `Acknowledge drift and proceed (records new hash)` — overwrite the `Spec hash:` field with the new hash and continue. **This write is the acknowledgement.**
     - `Stop — re-run /orb:review-spec on the modified spec` — halt with exit status 0, leave the stale hash in place.
   - **Non-interactive path** (no TTY on stdin **OR** `ORBIT_NONINTERACTIVE=1`): do **NOT** call `AskUserQuestion`. Leave the `Spec hash:` field unchanged, halt with exit status 1. The autonomous harness (e.g. `/orb:drive`) is responsible for surfacing the non-zero exit as a checkpoint.
3. **Gate enforcement (ac-06).** Walk the AC list in declaration order. If any preceding AC with `ac_type: gate` is not marked `[x]`, refuse to start the current AC and emit a refusal message naming the blocking gate's id. Non-gate preceding ACs do **not** block — an unchecked `ac-02 (code)` does not prevent `ac-03 (code)` from starting.

The acknowledgement contract for the interactive drift path is machine-checkable: **`progress.md`'s `Spec hash:` field equals `sha256(spec.yaml raw bytes)` after `Acknowledge` and only after `Acknowledge`.** No other path modifies it.

#### 4c. Detour append and re-anchor (ac-04, ac-05, ac-07)

When unplanned work is resolved:

1. Append a single atomic entry to `## Detours` in the format `YYYY-MM-DD: <description>` followed by `Return to: <ac-id>`. No open/closed state; the append IS the resolution.
2. Immediately set `Current AC:` to the entry's `Return to:` target.
3. Re-read `progress.md` and the spec's AC list. Select the next action from the **first unchecked AC in `## Acceptance Criteria`** — not from any in-memory context left over from the unplanned work. The parser ignores `## Detours` content when determining AC status (ac-09).

### 5. Implement — Tracking as You Go

**The pre-flight phase is over. Now write code.** Implement the deliverables from the spec, working through the acceptance criteria. After completing work that addresses an AC or satisfies a constraint:

1. Update `progress.md` to mark the item done: `- [x] ac-01: <description> — <brief note>`
2. State which AC(s) were addressed in your response

**Critical rules during implementation:**

- **Spec over codebase.** If the spec prescribes a pattern the codebase doesn't have, implement what the spec says. Do not work around missing code — create what the spec requires.
- **Surface unspecced decisions.** When you encounter a choice not covered by the spec that has meaningful consequences, **stop and ask** using AskUserQuestion. Present 2-3 options with trade-offs. Never choose silently.
- **Constraints are non-negotiable.** If you find yourself about to violate a constraint, stop and flag it. Either the constraint needs updating or the approach needs changing.
- **Assumption reversals require escalation.** When implementation evidence (phase results, benchmarks, test outcomes) contradicts a spec assumption, **stop immediately**. Do not silently adjust and continue. Instead: (1) Document the finding with exact numbers in `progress.md`, (2) State which spec assumption is invalidated and why, (3) Checkpoint with the author before proceeding. A spec built on a false assumption produces implementation that diverges silently — this is worse than stopping.
- **Derive from evidence, don't ask for gut calls.** When you encounter a parameter or approach question during implementation, check whether prior research, phase results, or benchmarks answer it. If the data prescribes the answer, use it. Only escalate to the author when evidence is genuinely silent or contradictory. The author sets goals and constraints; you derive implementation from evidence.

### 6. Final Check

When implementation is complete:

1. Review `progress.md` — all items should be marked done
2. If any items remain pending, explain why
3. Suggest next step: `/orb:review-pr` to verify the implementation

### 7. When a Spec Produces a NO-GO

Not every spec ships code. Some produce evidence that an approach doesn't work — that's a valid outcome, not a failure. When results invalidate the spec's hypothesis:

1. **Record the finding in `progress.md`** with exact numbers and evidence. This is the spec's deliverable.
2. **Mark the relevant ACs** with the result (NO-GO, with data).
3. **Do NOT "close the card."** Cards describe capabilities, not work items — they are never closed. The spec is the unit that completes, not the card.
4. **Update the card's `goal`** to reflect what was learned. The goal may narrow, shift, or be refined based on evidence.
5. Suggest next step: `/orb:design` to reassess the gap with the new evidence, or `/orb:review-pr` if there is code to merge.

## Integration with Other Skills

- **SessionStart hook** surfaces hard constraints even if this skill is never invoked — see `session-context.sh`
- **`/orb:review-pr`** reads `progress.md` to cross-reference AC coverage against the implementation record
- **`/orb:review-spec`** should have run before this skill — progressive review catches issues proportional to complexity

---

**Next step:** After implementation, run `/orb:review-pr` to verify AC coverage.
