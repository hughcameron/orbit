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
- `implementation_notes` — means-level leads from the design session. These are starting context, not constraints. Use them as a head start, but override with evidence when your analysis contradicts them.
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

#### 4d. Emit tasks — first-class AC/constraint visibility (card 0003 ac-01)

Immediately after writing `progress.md` (§4) and BEFORE running the first AC's pre-AC check sequence (§4b), the skill emits the checklist as a set of Claude Code tasks so the author has a live, structured view of the session's work. Task emission is derived from the parsed `progress.md` file (the single source of truth per constraint #3) using `plugins/orb/scripts/parse-progress.sh`:

1. Call `parse-progress.sh spec-path <progress.md>` to obtain the spec path string — this value becomes the `metadata.spec_path` on every emitted task.
2. Call `parse-progress.sh constraints <progress.md>` to enumerate hard-constraint strings. For each, call **TaskCreate** with:
   - `subject`: the constraint text, verbatim
   - `status`: `pending`
   - `metadata.spec_path`: the spec path from step 1
3. Call `parse-progress.sh acs <progress.md>` to enumerate `(ac-id, status, description, is_gate)` tuples. For each, call **TaskCreate** with:
   - `subject`: `ac-NN: <description>` (verbatim — **no** `(gate)` suffix; the annotation lives on the `progress.md` line only)
   - `status`: `pending` if `[ ]`, `completed` if `[x]` (pre-completed ACs in resumed sessions)
   - `metadata.spec_path`: the spec path from step 1

**Task emission is FLAT** — `addBlockedBy` / `addBlocks` are **never** set by this skill. Dependency wiring for gate ACs is reserved for a future card 0009 successor; leaving the slot unwired lets that wiring layer in additively without reshaping task creation.

**Scoping discipline.** Every orbit-implement task carries `metadata.spec_path`. The resume reconcile (§5) and any future orbit tooling filter by this tag and MUST NOT touch tasks without it or with a different value.

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

#### TaskUpdate rule — same tool-call turn as AC checkbox flip (card 0003 ac-02)

When the agent edits `progress.md` to flip a checkbox from `- [ ] ac-NN` to `- [x] ac-NN`, it **MUST** call **TaskUpdate** with `status: completed` on the corresponding task — the one whose `subject` starts with `ac-NN:` and whose `metadata.spec_path` matches the current spec — **in the same tool-call turn** as the `progress.md` edit. Not in the next turn, not after the next phase, not in a deferred batch. The edit and the `TaskUpdate` ride together in one tool-call batch.

This is an agent-side skill rule — it is **not** a PostToolUse hook, **not** a file watcher, **not** a batched checkpoint. Failing to emit the `TaskUpdate` in the same tool-call turn as the edit is a **protocol violation** of the §5 rule and produces a divergent task list. The same rule applies symmetrically to constraint tasks: when the agent ticks `- [ ] <constraint>` to `- [x] <constraint>`, it emits `TaskUpdate status: completed` on the matching constraint task in the same turn.

#### Resume reconcile — cancel-then-recreate on drift (card 0003 ac-03, ac-04)

On session resume, the `session-context.sh` hook surfaces whether a reconcile is pending. The agent's **first action** on resume is to execute the reconcile algorithm:

1. Call `TaskList` filtered to only include tasks whose `metadata.spec_path == <current spec path>`. Tasks without the tag and tasks with a different `metadata.spec_path` value are untouched — never read, never mutated (ac-04).
2. Build the **expected set** from `progress.md` using `plugins/orb/scripts/parse-progress.sh` (the `acs` and `constraints` subcommands — `## Detours` content is ignored per card 0009 ac-09).
3. Compare: if the filtered task count equals the expected count AND every `ac-NN` task's status matches the `progress.md` `[ ]` / `[x]` state AND every constraint task's status matches, **perform zero Task mutations and emit no warning** — the session is in sync.
4. Otherwise, rebuild: for each filtered task, call `TaskUpdate status: cancelled`. Then emit fresh tasks for every item in the expected set via `TaskCreate` (per §4d). Finally emit a single stderr warning whose text is the canonical single-source constant below.

**Canonical resume-rebuild warning (single source of truth, card 0003 constraint #12):**

> **RESUME_REBUILD_WARNING** = `orbit: task list out of sync with progress.md, rebuilt from scratch`

This exact string is emitted by the agent on rebuild, copied literally from this declaration. Test fixtures grep `plugins/orb/skills/implement/SKILL.md` for the constant and assert the emitted warning matches byte-for-byte. No `TaskDelete` / `TaskStop` is used — cancellation via `TaskUpdate` is the disposal primitive (card 0003 constraint #11).

If card 0009's non-interactive drift halt (exit status 1) fired before the hook reached the reconcile surface, the agent never reaches this rule — the session is aborted before any Task mutations.

#### Monitor-for-tests heuristic (card 0003 ac-05)

Long-running test invocations — expected duration over **60 seconds**, or full-suite runs (e.g. `cargo test` without a filter, `pytest` at the repo root, `npm test`) — **MUST** be launched via the **Monitor** tool with the command piped through a line-buffered failure-marker filter. The canonical filter is:

```
grep --line-buffered -E 'FAIL|ERROR|AssertionError|Traceback'
```

Short targeted tests (< 60 seconds, a named subset or single test) continue to use the `Bash` tool as before.

**Unfiltered Monitor on a test suite is forbidden** — every stdout line becoming a notification swamps the agent. The `grep --line-buffered` wrapper ensures only failure markers surface as streamed events while the suite runs to completion.

#### First-failure checkpoint — interactive / non-interactive split (card 0003 ac-06)

On the **first streamed** line from Monitor that matches the failure-marker regex `FAIL|ERROR|AssertionError|Traceback`, the agent's behaviour branches by interactivity, mirroring card 0009 ac-02's `AskUserQuestion` discipline.

**Interactive path** — stdin is a TTY AND `ORBIT_NONINTERACTIVE` is unset or not equal to `1`:

The agent MUST pause mid-run, acknowledge the failure inline, and call `AskUserQuestion` with exactly two options:

- `Fix the failure now (I will investigate and re-run)`
- `Let the suite finish, then triage`

Subsequent failure lines in the same Monitor run are surfaced but do NOT re-prompt. The `first` semantics is per-Monitor-invocation: a new test run resets the gate.

**Non-interactive path** — no TTY on stdin OR `ORBIT_NONINTERACTIVE=1` (this is `/orb:drive`, rally, cron, CI):

The agent MUST NOT call `AskUserQuestion`. On the first matching failure line, emit the canonical non-interactive marker string below to stderr, stop consuming further Monitor output, and halt with **exit status 2**. The upstream orchestrator (drive) uses the exit-2 convention to route to a checkpoint distinct from a clean test-suite failure (exit 1).

**Canonical non-interactive first-failure marker (single source of truth, card 0003 constraint #9):**

> **FIRST_FAILURE_NONINTERACTIVE_MARKER** = `orbit: first-failure checkpoint skipped (non-interactive); halting for upstream triage`

This exact string is emitted verbatim. Test fixtures grep this file for the constant and assert the emitted marker matches byte-for-byte.

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
