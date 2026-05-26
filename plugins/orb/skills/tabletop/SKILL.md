---
name: tabletop
description: Front-loaded pre-spec thinking — walk values, trade-offs, halt conditions and kill conditions across a card or cluster to produce aligned specs
argument-hint: "[card-id | card-id-1 card-id-2 ... | \"goal string\"]"
disable-model-invocation: true
allowed-tools: Bash Read Edit Write AskUserQuestion
---

# /orb:tabletop

Tabletop is the canonical pre-spec session for substantive R&D. One session, one card or a cluster in scope, one or more specs out. The output is the spec's **contract** — values, trade-offs, halt conditions, escalation triggers, kill conditions, hot-wash — *never* the spec's solution.

Choice 0017 (`tabletop-output-is-contract`) pins the load-bearing rule: tabletop captures *what to optimise for and what would stop us*; the spec captures the AC contract; the drive captures the implementation. Conflating these is the failure mode this skill exists to prevent.

Agent prose follows the discipline in `.orbit/STYLE.md` (see card 0026 — `.orbit/cards/0026-agent-prose-discipline.yaml`).

@.orbit/STYLE.md

## Usage

```
/orb:tabletop [card-id | card-id-1 card-id-2 ... | "goal string"]
```

A goal-string invocation infers the card cluster from `orbit card list`, surfaces the inferred set with one-line rationales, and confirms via AskUserQuestion (`approve` / `modify` / `extend`) before alignment work begins.

## When to use

- **Substantive R&D** where alignment cost would compound across the work
- **Cross-cutting work** touching two or more cards in one cluster
- **Goal-scoped work** where the cluster of cards isn't pre-determined

## Trivial-skip advisory

When the work appears trivial — single-line change, typo fix, single-AC scope — the skill surfaces a prose nudge at session open:

> *"This looks trivial; a direct `/orb:spec` is recommended. Tabletop is reserved for substantive R&D where alignment cost would otherwise compound. Proceed anyway? (y/N)"*

The advisory is **prose, not AskUserQuestion** — the operator chooses by typing or by continuing. The skill does NOT refuse. Per card 0019 scenario 10.

## Before Q1 — name the capability ambition

State the **underlying capability ambition** of the card cluster in one sentence before scope-carving begins. Carving multiple specs out of one cluster requires a concrete defence drawn from {technical dependency, hard budget, operator-bandwidth, parallelism}. "Manage risk" and "ship incrementally" are explicitly rejected as standalone defences — name the dependency, the budget, or the parallelism, or carve as one spec.

## The 10 questions

Walk these in declared order. Prose opens each question; AskUserQuestion closes the pick when the fork is discrete. Reach for past run-logs before inventing scenarios; flag imagined ones inline with `[imagined]`.

1. **Q1 — Goal narrowing.** Lock the goal in one sentence the author would say to a colleague.
2. **Q2 — Values.** Name what the work is optimising for; surface the load-bearing one.
3. **Q3 — Trade-offs.** Enumerate what the work trades against the chosen values; confirm the cut is the simplest cut that holds them.
4. **Q4 — Failure modes.** Enumerate ways the work fails; classify each as halt-worthy or engineering-hygiene.
5. **Q5 — Lateral approaches.** Name the alternatives that *aren't* being picked, with reasons. Held in reserve as fallback paths.
6. **Q6 — Success criteria.** Pin binary, measurable criteria that trace to a value or trade-off.
7. **Q7 — Escalation triggers.** Name when the agent should halt and surface to the author mid-flight, with the proposed action.
8. **Q8 — Adjacent code.** Enumerate which layers/modules the work touches; route file-level questions to Implementation Notes in the sidecar.
9. **Q9 — Budget.** Name the working-day budget at Claude-execution pace, not at conservative-engineering quotes.
10. **Q10 — Kill conditions.** Name the failure of each load-bearing claim, with a named pivot path.

### Hot-wash debrief

After Q10, capture meta-observations in prose — what kept coming up, what was unclear, what reframes surfaced. Two to five bullets per category: `recurred`, `surprised`, `friction`, `meta-patterns-for-future-tabletops`. Fresh, before formal write-up sanitises the signal.

### Per-scenario verification classification

Every scenario the resulting spec will cover carries one verification classification line in the tabletop sidecar — `verifies: capability` (the test exercises the underlying capability directly) or `verifies: stand-in (real thing is X), accepted because Y`. **No third option.** The classification lives in a dedicated **Verification posture** section in the sidecar (or, if compact, inline in Trade-offs). /orb:spec carries this verbatim into each AC (see spec SKILL.md halt rule).

## Output

One session, N specs (N ≥ 1); each spec gets its own folder under `.orbit/specs/<date>-<slug>/` with its own `tabletop.md` sidecar.

```markdown
# Tabletop — <Topic>

**Date:** YYYY-MM-DD
**Cards in scope:** <cluster>
**Output spec:** .orbit/specs/<spec-id>/spec.yaml

---

## Values
<from Q2 — the load-bearing value and what falls out of it.>

## Trade-offs
<from Q3 — each named trade-off classified acceptable / expensive-but-worth-it / halt-trigger.>

## Halt conditions
<from Q4 halt-worthy entries — each names a measurable trigger and a revert or pivot path. Anti-patterns like "halt and reassess" or "ask if confused" get rewritten before they land.>

## Escalation triggers
<from Q7 — each names a condition, the state snapshot to surface, and the proposed action.>

## Kill conditions
<from Q10 — each names the specific load-bearing claim being killed and a pivot path.>

## Hot-wash
<recurred / surprised / friction / meta-patterns-for-future-tabletops.>
```

---

**Next step:** `/orb:spec` against each output spec folder to crystallise the AC contract from the tabletop's values, trade-offs, halt conditions, escalation triggers, kill conditions, and acceptance criteria.
