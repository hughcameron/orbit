# Mission Resilience — Staying on Spec When Reality Intervenes

**Date:** 2026-04-19
**Origin:** Observed drift during a multi-phase spec implementation (AC-01 through AC-06)

---

## Problem Statement

During long implementation sessions, real-world findings surface that demand attention — a data quality issue, a dependency incompatibility, an infrastructure problem. These findings are legitimate and need resolving. But resolving them pulls the agent off the spec's AC sequence, and the agent often doesn't return.

The failure mode isn't wrong assumptions (the implement skill already handles that). It's **attention capture** — the agent gets absorbed in the side quest and forgets where it was in the spec. After the detour, it continues in whatever direction it was heading rather than re-anchoring against the AC sequence.

**Goal:** Agents maintain spec fidelity through mid-flight disruptions without requiring the human to pull them back on track.

## Observed Failure

An agent was implementing a spec with six ordered ACs, including gate ACs that blocked later work:

```
AC-01 (gate) → Prerequisite verification — "Do not proceed to AC-05 until verified"
AC-02 (code) → Data + infrastructure setup
AC-03 (code) → Validation parity checks
AC-04 (doc)  → Configuration calibration
AC-05 (code) → Core computation
AC-06 (gate) → Quality gate on outputs
```

During AC-02, a data quality issue surfaced. This was a real problem that needed fixing. The agent spent significant effort resolving it — modifying the pipeline, re-running batch jobs, verifying outputs. All legitimate work.

After resolving the issue, the agent continued in the "infrastructure" direction and then proposed jumping straight to the core computation (AC-05), skipping:

- AC-01: The gate AC that explicitly blocked AC-05
- AC-03: Validation parity checks
- AC-04: Configuration calibration

The spec had also been modified mid-flight (new constraints, AC-02 updates) without going back through review. The original review's REQUEST_CHANGES verdict was never formally resolved.

The human had to intervene twice:
1. Pulling attention back to the skipped calibration step (AC-04)
2. Recognising the spec itself had drifted and needed re-review

## Anatomy of Drift

The failure has three distinct phases:

### Phase 1: Legitimate detour
A finding surfaces that genuinely needs attention. The agent is right to address it. This is not a bug — it's the agent being responsive to reality.

### Phase 2: Attention capture
The detour consumes working memory. The agent's context fills with the side quest's details — error logs, retry attempts, verification steps. The spec's AC sequence fades from active attention. There's no mechanism reminding the agent where it was.

### Phase 3: Momentum continuation
After resolving the detour, the agent continues in the direction it was heading rather than returning to the spec. The most recent work feels like the natural next step. The AC that should come next (the gate) is several context windows back.

## What the Current System Provides

### Implement skill (SKILL.md)
- **"Spec over codebase"** — but this is about implementation patterns, not AC ordering
- **"Assumption reversals require escalation"** — handles contradicted hypotheses, not drift
- **"Surface unspecced decisions"** — handles design choices, not sequence
- **progress.md tracking** — the mechanism exists, but nothing forces the agent to consult it after a detour

### Session-context hook
- Surfaces hard constraints from the in-flight spec
- Surfaces active drive status and next workflow step
- Does NOT surface which AC is next
- Does NOT read progress.md to detect drift
- Does NOT detect mid-flight spec modifications

### Drive skill
- Tracks stage-level state (design → spec → implement → review) in drive.yaml
- Does NOT track AC-level progress within the implement stage
- File-presence state machine detects stage completion, not AC completion

## The Gap

There is no mechanism that says: "You just finished a detour. Before proceeding, re-read the spec and progress.md. What is the next AC in sequence?"

The spec's AC ordering — especially gate ACs — is a design-time decision that gets lost during implementation. Gate ACs are supposed to block subsequent ACs, but nothing enforces this during execution.

## Design Questions

### 1. Where should re-anchoring happen?

Options:
- **A. In progress.md itself.** Add a "current AC" marker. After any detour, the agent reads progress.md and sees which AC is next. Simple, passive — relies on the agent to check.
- **B. In the session-context hook.** The hook already reads the spec for constraints. It could also read progress.md and surface the next unchecked AC at session start. Automatic, but only fires at session start — doesn't help mid-session.
- **C. In the implement skill.** Add an explicit rule: "After resolving any unplanned work, re-read progress.md and the spec's AC list before choosing what to do next." Prescriptive, relies on agent discipline.
- **D. A combination.** Hook surfaces next AC at session start (B). Skill rule prescribes re-anchoring after detours (C). Progress.md tracks current position (A). Belt, braces, and suspenders.

### 2. How should gate ACs be represented?

The observed spec used `(gate)` as a prose annotation. This has no mechanical meaning — nothing in the implement skill or session-context hook recognises it.

Options:
- **A. Spec YAML field.** Add a `gate: true` field to AC definitions. The implement skill can then enforce ordering: "AC-05 is blocked until gate AC-01 is marked done."
- **B. Convention only.** Keep `(gate)` as prose and add a skill rule: "Before starting any AC, check that all preceding gate ACs are complete." Simpler, less enforceable.

### 3. What triggers a re-review?

The spec was modified during implementation — new constraints added, ACs expanded — without going back through `/orb:review-spec`. The original review's REQUEST_CHANGES verdict was never formally closed.

Options:
- **A. Implement skill detects spec changes.** On first read, hash the spec. Before each AC, re-read and compare. If the spec changed, flag it: "Spec modified since implementation started. Re-review recommended."
- **B. Git-based detection.** Session-context hook compares spec file's git status. If modified since the last review-spec-*.md, surface a warning.
- **C. Manual discipline.** Add a skill rule: "If you modify the spec during implementation, record that a re-review is needed in progress.md."

### 4. How should detours be recorded?

When the agent goes off-script to handle a finding, there's currently no record of the detour — it exists only in conversation history. If the session dies, the detour's context is lost.

Options:
- **A. Detour log in progress.md.** A section for unplanned work:
  ```markdown
  ## Detours
  - 2026-04-17: Data quality issue in dataset X — scanned, verified clean.
    Return to: AC-01 (prerequisite gate)
  ```
  The "Return to" field is the re-anchoring mechanism.
- **B. Findings file.** Separate `findings.md` alongside progress.md for discoveries made during implementation. Keeps progress.md clean but adds another file to track.

### 5. How does this interact with rally?

Rally (see `.orbit/discovery/rally.md`) packs multiple cards between human gates. Mission resilience becomes more critical in a rally — if one card hits a detour, it could pull the lead agent's attention away from the other cards' progress.

The rally state file (`rally.yaml`) could track per-card AC progress, making drift visible at the rally level. But this adds complexity to an already-new concept. Worth considering but probably a second-order concern.

## Severity

This is not a theoretical risk. The observed case demonstrates it in production:

- **Time lost:** Multiple sessions spent on infrastructure before the human noticed the sequence violation
- **Spec integrity:** Spec modified mid-flight without re-review, original review verdict unresolved
- **Trust cost:** The human had to intervene twice to re-anchor the agent, reducing confidence in unattended operation

The irony: the implement skill was created because of a prior incident where an agent missed a spec-prescribed pattern. The skill solved the "spec not loaded" problem. This discovery is about the "spec loaded but forgotten" problem.

## Next Steps

- [ ] Decide whether this is a skill refinement (update implement skill + session-context hook) or a new capability (new card)
- [ ] If skill refinement: update implement/SKILL.md with re-anchoring rules, gate AC enforcement, and detour logging
- [ ] If new capability: write a card with scenarios covering detour → return, gate enforcement, mid-flight spec change detection, and session resumption after drift
