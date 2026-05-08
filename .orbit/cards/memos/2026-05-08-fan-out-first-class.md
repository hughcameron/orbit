# Fan-out as a first-class concept in orbit

**Date:** 2026-05-08
**Source:** Hugh, in conversation, after pausing finetype's cron-driven autonomy. Reference: `/home/hugh/reference/agentic/fan-out.md`.

## The realisation

Long-running R&D — pillar 4 — is not delivered by a single agent running for longer. It's delivered by **fan-out × per-agent autonomy time × durable state**. A single agent can't do "a full session's work" alone because the bottleneck is serial reasoning. The cron loop in finetype tried to extend single-agent autonomy time and ran flat for 7 cycles; it failed because depth without breadth produces the *appearance* of work, not the substance.

Rally already encodes fan-out at one level: across drives. But fan-out is a bigger idea than rally, and treating it as a rally-internal pattern under-uses it.

## Where fan-out shows up in orbit today

- **Rally** (card 0006) — six fan-out techniques: map, fork-join, type-fanout, speculative, tree-of-thoughts, cold-fork-review. All operate **across specs / drives**.
- **Cold-fork review** (card 0007) — fan-out at review time, fresh-context agents reading work without the implementer's confirmation bias.
- **Sub-agents** (the Task tool) — fan-out within a session for context isolation. Used ad-hoc; not codified in any card.
- **Worktrees** — fan-out across sessions for filesystem isolation. Mentioned nowhere in cards.
- **Headless `claude -p`** — process-level fan-out for batch work. Mentioned nowhere in cards.

The rally card treats fan-out as "what rally does." The reference doc at `/home/hugh/reference/agentic/fan-out.md` treats fan-out as **a substrate concept** that operates at sub-agent, worktree, headless, fork, custom-agent, and routing levels. Orbit currently picks up only the rally-and-review slice of that surface.

## Three levels of fan-out orbit should make first-class

### 1. Spec-level (across drives) — what rally already does

N specs handed to N agents. Rally is the orchestrator. This is in place.

### 2. Drive-internal (within a single spec)

A single drive can fan out:

- **Per-AC** — when ACs are genuinely independent, spawn one sub-agent per AC. Each returns a structured result; the drive synthesises.
- **Per-investigation** — during research-heavy ACs, spawn parallel investigation sub-agents (different angles, different data sources). Each returns a brief; the drive picks.
- **Speculative implementation** — when an approach is uncertain, spawn N implementation sub-agents with different approaches; pick the one that passes the gate AC fastest.
- **Tree-of-thoughts mid-drive** — already named in rally's scenarios but rally-flavoured. The same primitive applied within a single drive avoids escalating to a rally for a one-level branch.

This level is not currently codified. Drives default to single-agent execution. Fan-out within a drive is left as an emergent capability if the implementing agent thinks of it.

### 3. Research-mode (across investigation streams)

`/orb:researcher`, `/orb:discovery`, `/orb:distill` are exploration-shaped, not execution-shaped. They benefit from fan-out differently:

- **Discovery fan-out** — N parallel investigation sub-agents exploring different angles of a vague idea, each returning a structured brief, the orchestrator merging.
- **Distill fan-out** — when distilling a large body of source material (e.g., a whole project), one sub-agent per source file, each extracting candidate cards in a fixed schema, the orchestrator deduping.
- **Research multi-source synthesis** — pattern D from the reference doc — schema-defined returns merged into a comparative document.

Research fan-out is barely mentioned anywhere in orbit. `/orb:distill` already works on multiple files but does so serially. `/orb:researcher` returns a single agent's investigation.

## Why this needs to be a separate card from rally

Rally is **the orchestrator across drives**. The card that this memo proposes is **the substrate concept that rally is one consumer of**. Other consumers:

- Drive-internal fan-out — spec-level execution
- Research-mode fan-out — exploration-mode execution
- Skill-internal fan-out — any skill can fan out for its own work (e.g., `/orb:audit` fanning out per-spec)

Conflating "fan-out" with "rally" forces every fan-out use case through rally's orchestration overhead, which is wrong for drive-internal and research-mode cases.

## Coupling to the four pillars

| Pillar | How fan-out serves it |
|--------|------------------------|
| Long-running R&D | The load-bearing one. Full-session-of-work = fan-out depth × autonomy time. Without fan-out, autonomy time hits the single-agent reasoning ceiling fast. |
| Agent state-persistence | Fan-out without durable state is unrecoverable on session death. Specs, drive state, and rally state are what make fan-out resumable. |
| Agent self-learning | Multiple agents accumulate facts in parallel; memory-loop merges them. Fan-out without self-learning loses the gains. |
| Executive interaction | Fan-out is what enables "do a full session's work before checking in." Without fan-out, the agent has to serialise and check in more often. |

Fan-out touches all four pillars but is the load-bearing mechanism for pillar 4.

## What good looks like

A card — `0029-fan-out.yaml` (or similar) — that:

1. **Defines fan-out as a substrate concept**, not a rally feature.
2. **Names the three levels** (spec / drive-internal / research-mode) with the criteria for each.
3. **Sets the default to sequential.** Fan-out is opt-in. Specs declare fan-out potential at design time; otherwise drives execute single-agent.
4. **Names the anti-patterns** (see below) so fan-out theatre doesn't pass review.
5. **Couples to the pillars card** — cites pillar 4 as primary, pillars 2/3/1 as secondary.

## Anti-patterns to head off

- **Fan-out theatre** — three sub-agents doing one agent's work in parallel because "parallel = fast." Fan-out has overhead (orchestrator context, return synthesis, rate limits); below a threshold it loses to sequential execution.
- **Shared-state fan-out** — sub-agents writing to the same files without worktree isolation. The reference doc names this as the most common conflict source.
- **Verbose returns** — sub-agents that return long results consume orchestrator context. Fan-out only pays off if returns are constrained (structured schema, word cap).
- **Fan-out for sequential work** — if B depends on A's output, parallel is wrong. The `/orb:design` step should declare "this work is sequential" or "this work is parallelisable" explicitly.
- **Untriaged escalations** — fan-out produces N escalation candidates; the orchestrator must triage before surfacing to the author. Otherwise pillar 1 (executive interaction) breaks.

## Where this couples

- **`/orb:design`** — adds "fan-out potential" as a question: per-AC parallelism? Speculative? Sequential?
- **`/orb:spec`** — specs carry a `fan_out:` field declaring the chosen pattern (or `sequential`).
- **`/orb:implement`** / drive — consumes the field; spawns sub-agents when declared, runs single-agent otherwise.
- **`/orb:rally`** — depends on this card (rally is the spec-level consumer of the fan-out primitive).
- **`/orb:researcher`**, **`/orb:discovery`**, **`/orb:distill`** — research-mode fan-out patterns.

## Tradeoff worth flagging

Making fan-out first-class risks every spec sprouting a "should this fan out?" debate when most work is genuinely sequential. The card needs a **sharp default** (sequential) and a **named bar** (fan-out is justified when N independent ACs exist OR when an investigation has ≥3 angles OR when a single approach is uncertain enough that race-and-pick is cheaper than deliberation).

## Status

Memo only. To be distilled into a card. Closely paired with the four-pillars memo (`2026-05-08-four-pillars.md`) — neither makes full sense without the other.

## Related

- 0006-rally — one consumer of the fan-out primitive
- 0007-drive-forked-reviews — the cold-fork-review fan-out pattern
- 0011-cron-driven-execution — the *failed* attempt to deliver pillar 4 without fan-out (depth-without-breadth)
- `/home/hugh/reference/agentic/fan-out.md` — the technical reference for the six mechanisms
