Discovered during the brownfield-migration-hardening rally on 2026-05-24: the rally skill's parallel-worktree implementation pattern (Stage 5c) is structurally incompatible with the current Claude Code agent harness. All three drives in the rally parked with `tool_surface_incomplete`.

## The wall

The rally lead launches drive sub-agents via the Agent tool with `run_in_background: true`. Each sub-agent's brief tells it to run `/orb:drive <spec-id>` inside its worktree. Drive then performs review-spec → implement → review-pr internally, with review-spec and review-pr running as **nested forked Agents** (the cold-fork pattern that prevents review-context pollution from the implementation context).

In this harness, **the Agent tool is not surfaced inside Agent-spawned sub-agents.** So the nested cold-fork that `/orb:drive` requires cannot launch. Drive correctly parks the spec with `reason_label: tool_surface_incomplete` — the skill's documented reason for exactly this case.

Drive A returned that verdict at 148s of runtime. Drive C was stopped before it returned (its last output trace was "No Agent tool surfaced. Let me look for it explicitly." — confirming it was hitting the same wall).

## What the rally skill says

The rally skill's NO-GO Handling section names `tool_surface_incomplete` as one of six fixed reason_labels:

> Agent tool unavailable for cold-fork                   tool_surface_incomplete

And the skill says the rally continues with remaining cards when one parks. But in this harness, **every drive launched this way parks for the same reason** — there is no remaining-cards path. The skill anticipates the failure mode but doesn't have a workaround documented.

## What this means

The rally skill's three high-value gates (proposal alignment, consolidated decision pack, consolidated design review with disjointness) all worked perfectly. The rally produced excellent design substrate. Where it broke is the implementation orchestration — specifically the assumption that sub-agents can themselves spawn sub-agents.

The rally's value before implementation: three decision packs (~390 lines), three interviews (~600 lines), disjointness analysis, ordering wired via `dep_predecessors`. All committed at `b2b8ba9`. Specs are open, drive sidecars are clean (Drive A's at stage: review-spec; B and C never started). The user resumes via `/orb:drive <spec-id>` in fresh top-level sessions where the Agent tool IS available.

## Architectural options for the rally skill

1. **Inline reviews inside the sub-agent** — change drive's contract under rally-sub-agent execution to inline review-spec and review-pr rather than cold-fork. This loses review-context isolation, which is the explicit reason drive cold-forks (`/orb:drive §1.3 disallows inline review`). Trade quality for executability.

2. **Lead-orchestrated reviews** — the rally lead (which DOES have Agent tool access) takes over the cold-fork dispatch for each sub-agent's review stages. Sub-agents do only implementation; the lead spawns review-spec / review-pr forks against the sub-agent's worktree state. Increases lead-side complexity but recovers the cold-fork pattern.

3. **Sequential drives in the main checkout** — abandon parallel-worktree dispatch entirely. The lead invokes `/orb:drive` directly via the Skill tool, serially, one card at a time. Loses parallelism but works in any harness that supports the Agent tool at the top level.

4. **Wait for harness support** — Anthropic's harness may eventually surface Agent tool access to sub-agents, at which point the existing rally pattern works as designed.

Option 3 is the cheapest immediate fix. Option 2 is the most architecturally interesting (preserves cold-fork isolation while routing dispatch through the only context that has Agent tool access). Option 4 is the right answer if the harness gap is short-lived.

## Cards affected

- **Card 0006 (rally)** — the parallel-worktree pattern in Stage 5c is the load-bearing assumption that breaks. A future spec against 0006 should pick one of the four options above and wire it into the skill.
- **Card 0009 (mission-resilience)** — drive's halt-and-park behaviour worked as designed. Drive A correctly identified the limitation, named the right reason_label, and left a clean drive sidecar at stage: review-spec for resumption. The mission-resilience pattern is intact; the orchestration layer above it has the gap.
- **Card 0005 (drive)** — drive's cold-fork dependency on the Agent tool is documented (`/orb:drive §1.3`). No card-level change needed; the rally is the surface that breaks, not drive.

## Resumption

When the user is ready to ship the three parked drives:

```
/orb:drive 2026-05-24-setup-is-orbit-state-aware
# then, after A closes:
/orb:drive 2026-05-24-workflow-conformance
# C is independent — can run any time:
/orb:drive 2026-05-24-brownfield-spec-migration
```

Each drive picks up from its spec's current state. Drive A's sidecar is at stage: review-spec already (drive sub-agent got that far before parking). The three rally branches (`rally/setup-is-orbit-state-aware`, `rally/brownfield-spec-migration`, `rally/workflow-conformance`) are at the same commit as main and can be reused or ignored — drive will create whatever it needs.

## Source

Rally 2026-05-24-brownfield-migration-hardening-rally, parked at phase: complete with all three children carrying `card_phase: parked, park_reason: PARKED: [tool_surface_incomplete] ...`. Full state in `.orbit/specs/2026-05-24-brownfield-migration-hardening-rally/rally.yaml`. Sub-agent verdicts captured in spec notes via `orbit spec note`.
