# Brief: Orbit refactor for autonomous execution

## Why now

I'm moving away from being at the keyboard for eight hours a day. Project objectives haven't softened and the quality bar hasn't moved. What needs to change is the workflow: agents need to carry more of the execution autonomously, with structured checkpoints rather than constant approvals.

Orbit currently solves the spatial axis well (sprints, cards, scenarios, rally). It does not yet solve the temporal axis: how work pushes forward when I'm not watching, and how I re-enter the loop efficiently when I am. This refactor adds the temporal axis without breaking what already works.

## Orbit's own objective function

I spend less than two hours per day managing agents, while every active project's objective function continues to show measurable progress.

Both halves matter. Cutting management time at the cost of quality fails. Holding quality at the cost of management time also fails. Treat this as the success metric for the refactor itself.

## What stays load-bearing

- The human/agent division of labour. I own vision and decisions; agents handle execution and derivation. The whole framework rests on this.
- Cards as commitments and memos as evidence and rationale. These remain the artefacts I actually read.
- Agents stop and ask before making silent decisions. The cron model raises the stakes on this rule, it does not relax it.

## What changes

**Project objective functions become first-class.** Every active project gets a measurable, agent-checkable objective (e.g. private-project PL positive, FineType validation >= 95%) plus at least one paired guardrail to prevent Goodharting. The objective function is the agent's primary self-test for whether work is on-track and the primary stop condition when it cannot be predicted.

**Specs get rationalised.** I'm not reading them. Either they shrink to terse acceptance criteria and rationale moves into memos, or they're replaced. Worth studying [beads](https://gastownhall.github.io/beads/) for the queryable-state pattern even if we don't adopt the tool. `bd ready` is the kind of primitive crons can hit cleanly.

**Crons join the model.** Agents run on cadences (hourly to six-hourly) between daily planning sessions with me. Each cron run checks state, picks up ready work against the objective function, and emits a thin shift report. Shift reports are a new artefact type and need designing carefully so they don't become the next bloat problem.

**Daily planning sessions become the linchpin.** Vision, prioritisation, and unblocking happen here. Quality of planning sets the ceiling on the next twenty-four hours of autonomous work. The session protocol needs the same care as the cron design.

## Open questions for you

- Adopt beads, mirror its primitives in markdown, or stay closer to current orbit shape? What does `ready` look like in orbit's vocabulary?
- What's the shape of a shift report? Length, structure, and what it must contain to be useful in my discovery time.
- How do crons recognise decision boundaries cleanly? What's the agent's stop condition when the objective function can't predict the impact of a choice?
- How do specs collapse? Minimum-viable acceptance criteria attached to cards, or replaced entirely by something terser?
- What's the daily planning session protocol? Inputs, outputs, expected duration, what artefacts get touched.
- How does cron-generated evidence flow back into discovery and vision modes without me having to chase it?

## Out of scope

- Multi-agent coordination beyond what already exists.
- Replacing the rally model.
- Any change to card or memo formats themselves.
- Tooling integrations beyond Claude Code.

## First step

Before writing any code or skills, draft a short design memo answering the open questions above. I want to read your thinking before any refactor lands. The memo itself is a test: if it's verbose, the refactor will be too.
