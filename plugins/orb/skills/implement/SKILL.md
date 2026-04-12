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

Where `<spec_path>` is the path to a `spec.yaml` file (e.g. `specs/2026-04-04-topic/spec.yaml`).

## Why This Exists

Without a pre-flight check, implementing agents treat the codebase as ground truth and miss spec-prescribed patterns. This skill forces the spec into working memory before implementation begins, and keeps it there throughout.

> **Reference incident:** McGill hydrofoil — agent treated codebase as ground truth, missed S3 entrypoint pattern from spec.

## Instructions

### 1. Read the Spec

Read the spec YAML file provided via `$ARGUMENTS`.

- If no argument is provided: look for the most recent `specs/*/spec.yaml`
- If no spec exists: stop and tell the author — there's nothing to implement against

Extract:
- `goal` — the primary objective
- `constraints` — the hard limitations (these are non-negotiable)
- `acceptance_criteria` — every `ac-NN` with its description
- `deliverables` — what files need to be created or modified

### 2. Search for Related Code

Before writing anything, run a keyword scan (see `/orb:keyword-scan`) against the project source using terms from the spec's goal and AC descriptions. This surfaces existing code, patterns, and tests the implementation should build on — rather than reimplementing what already exists.

### 3. Present the Checklist

**Before any code is written**, present the full checklist:

```
## Pre-Flight Checklist

**Goal:** <goal from spec>

### Hard Constraints
- [ ] <constraint 1>
- [ ] <constraint 2>

### Acceptance Criteria
- [ ] ac-01: <description>
- [ ] ac-02: <description>
- [ ] ac-03: <description>

### Deliverables
- <path 1>: <description>
- <path 2>: <description>
```

Then proceed immediately to writing the progress file and implementing — do not wait for confirmation.

### 4. Write the Progress File

Create `progress.md` in the same directory as the spec:

```markdown
# Implementation Progress

**Spec:** <path to spec.yaml>
**Started:** <today's date>

## Hard Constraints
- [ ] <constraint text>

## Acceptance Criteria
- [ ] ac-01: <description>
- [ ] ac-02: <description>
```

All items start as `- [ ]` (pending).

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

## Integration with Other Skills

- **SessionStart hook** surfaces hard constraints even if this skill is never invoked — see `session-context.sh`
- **`/orb:review-pr`** reads `progress.md` to cross-reference AC coverage against the implementation record
- **`/orb:review-spec`** should have run before this skill on HIGH-tier specs

---

**Next step:** After implementation, run `/orb:review-pr` to verify AC coverage.
