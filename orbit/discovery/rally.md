# Rally — Maximising Agent Throughput Between Human Gates

**Date:** 2026-04-19
**Origin:** Observed behaviour during a parallel design session across three independent cards

---

## Problem Statement

The orbit pipeline has two stages where human interaction is essential:

1. **Ideation** — design sessions where the author makes architectural choices
2. **Assurance** — spec review and PR review where the author validates quality

Everything between these gates — spec generation, implementation, test writing — is agent work. The current `/orb:drive` skill processes one card at a time, meaning agent throughput is bottlenecked by serial human interaction: the author finishes one card's design before the next begins.

**Goal:** Maximise the volume of agent work that runs between each human touchpoint, without sacrificing the quality that human gates provide.

## Observed Pattern

During a session, the lead agent identified three independent cards that shared a subsystem but had no cross-dependencies. It ran three parallel design sessions as background agents, then presented a consolidated review. The sequence was:

```
Cards identified
  │
  ├─ Human gate: rally approval ("drive 0015–0017 as a group?")
  │
  ▼
Parallel design (3 background agents)
  │
  ├─ Human gate: consolidated design review
  │
  ▼
Sequential spec + implement (informed by cross-design findings)
  │
  ├─ Human gate: PR reviews
  │
  ▼
Ship
```

### What went right

- **Batch identification was sharp.** 10 unshipped cards filtered to 3 with clear rationale (pipeline runtime essentials, all independent, all needed for the reference pipeline).
- **Design parallelised cleanly.** Three agents read source files independently and produced coherent designs without coordination.
- **Consolidated review caught cross-cutting concerns.** All three designs converged on Engine trait signature changes. The lead agent spotted this and proposed an implementation order (0016 → 0015 → 0017) that avoids conflicting trait changes.
- **Simplifications surfaced.** The lead agent recommended dropping two v1 features (exit code filtering, hook state recording) — editorial judgment that only works with visibility across all three designs.

### What went wrong

- **Design outputs landed in an improvised `designs/` folder** instead of the prescribed `orbit/specs/` structure. The parallel agents weren't running `/orb:design` as a skill — they were general-purpose agents briefed to "produce a design document." Without the skill's instructions, they picked a reasonable-sounding location.
- **The "parallel teammates" pitch became sequential.** The initial framing was 3 parallel implementation streams (~2 hours vs ~5). The design phase revealed shared trait changes that require serial implementation. The time estimate was honest in retrospect, but the initial pitch oversold parallelism.
- **No artefact records the rally plan.** The implementation order (0016 → 0015 → 0017) and the rationale exist only in conversation. If the session dies, a resuming agent has no record of the rally or its sequencing.

## The Throughput Model

Human attention is the scarce resource. Agent compute is abundant. The question is: how many agent-hours can we pack between each human decision?

```
                    Human gates (scarce)
                    ─────────────────────
                    │                   │
        ┌───────── Ideation    Assurance ──────────┐
        │           │                   │           │
        │     ┌─────┴─────┐      ┌─────┴─────┐     │
        │     │ Design Q&A │      │ PR review │     │
        │     └─────┬─────┘      └─────┬─────┘     │
        │           │                   │           │
        │    Agent work (abundant)      │           │
        │    ────────────────────       │           │
        │    │ Spec generation  │      │           │
        │    │ Implementation   │      │           │
        │    │ Test writing     │      │           │
        │    │ Spec review      │      │           │
        │    └──────────────────┘      │           │
        │                               │           │
        └───────────────────────────────┘           │
```

**Single-card drive:** One design session → one spec → one implementation → one PR review. Human touches the pipeline 2–3 times per card.

**Rally:** N design sessions (parallel) → consolidated review (1 human touch) → N specs + implementations (serial or parallel) → N PR reviews (grouped). Human touches the pipeline 2–3 times per *rally*.

The multiplier is `N` — the number of independent cards in the rally.

## Key Design Questions

### 1. What qualifies cards for a rally?

The observed heuristic: independent cards that share a subsystem. More precisely:

- **No cross-card dependencies** — card A's implementation doesn't block card B's design
- **Shared codebase area** — same module or crate, so the lead agent can reason about interactions
- **Common prerequisite** — all needed for the same downstream goal (e.g., "reference pipeline readiness")

Open question: should rally identification be a skill, or is it better left to agent judgment? The current agent definition says "recommend splitting for 2+ hours with 2+ independent streams" — this heuristic worked.

### 2. How do parallel designs stay on the artefact path?

The current problem: parallel design agents don't run `/orb:design` as a skill (drive explicitly says "do not invoke sub-skills"). They're briefed agents that may not know orbit's file conventions.

Options:
- **A. Include output instructions in the agent brief.** The lead agent tells each design agent exactly where to save: `orbit/specs/YYYY-MM-DD-<slug>/interview.md`. Simple, relies on the lead getting it right.
- **B. Each design agent runs `/orb:design` as a skill.** This keeps them on the prescribed path but contradicts drive's "inline, not sub-skill" rule.
- **C. Design agents return content, lead agent saves it.** Agents produce markdown as their return value; the lead writes files to the correct locations. Centralises file management.

### 3. What artefact records the rally plan?

A rally needs a state file analogous to `drive.yaml` but at a higher level. Something like:

```yaml
# rally.yaml — lives in orbit/specs/ or at repo root
rally: pipeline-runtime-essentials
cards:
  - path: orbit/cards/0015-execution-resilience.yaml
    status: designing
    spec_dir: orbit/specs/2026-04-19-execution-resilience/
  - path: orbit/cards/0016-pipeline-parameterisation.yaml
    status: designing
    spec_dir: orbit/specs/2026-04-19-pipeline-parameterisation/
  - path: orbit/cards/0017-lifecycle-hooks.yaml
    status: designing
    spec_dir: orbit/specs/2026-04-19-lifecycle-hooks/
implementation_order: [0016, 0015, 0017]
order_rationale: "All three touch Engine trait; 0016 changes signature first"
autonomy: guided
started: 2026-04-19T10:00:00Z
```

This survives session death and tells a resuming agent the full plan.

### 4. When does parallel become sequential?

The consolidated design review is where this decision gets made. The lead agent examines all designs for:

- **Shared type/trait changes** — same struct or interface modified by multiple cards
- **Execution order dependencies** — card A's implementation creates the interface card B builds on
- **Test infrastructure overlap** — shared test fixtures or data

If conflicts exist, the lead proposes an implementation order. If not, parallel implementation with worktree isolation is viable.

### 5. How does assurance scale with rally size?

Single-card drive: one PR review per card, each one a human gate.
Rally options:

- **Individual PR reviews** — N separate reviews. Safe but defeats the throughput goal.
- **Batched PR review** — one review session covering all N PRs. Higher throughput but higher cognitive load on the author.
- **Stacked PRs** — each card's PR targets the previous card's branch. Author reviews the stack as a sequence.

## Scope Boundary

This brief is about **orbit workflow mechanics** — how `/orb:drive` could support rallying. It is not about:

- Agent teaming infrastructure (TeamCreate, worktree management) — that's Claude Code platform
- Specific card content (0015–0017 are examples, not the subject)
- Changing the human gates themselves — ideation and assurance remain interactive

## Next Steps

- [ ] Decide whether this warrants a card (it does if we want to build it; it doesn't if the pattern works well enough as agent judgment)
- [ ] If carding: the card's scenarios should cover rally identification, parallel design, consolidated review, sequential implementation, session resumption, and the failure case (what happens when one card in the rally gets a NO-GO)
