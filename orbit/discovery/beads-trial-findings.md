# Beads trial replay: UX uplift rally findings

**Date:** 2026-04-30
**Method:** Replayed the 4-card UX uplift rally (2026-04-20) as beads in a scratch repo. Parent epic with four child beads, dependency edges matching original serial/parallel structure, decisions seeded via `bd remember`.

## What worked

**Ready-work computation is better than orbit's.** `bd ready` returned the correct set at every stage — three beads ready initially (0009, 0005, 0006), card 0003 correctly blocked by 0009, auto-unblocked the instant 0009 closed. No scanning, no YAML parsing, no shell script. The dependency graph is the truth and the query is idempotent.

**Parallel-to-serial mid-flight is trivial.** `bd dep add B A --type blocks` — one command, immediate effect. Card B disappears from ready queue, reappears when A closes. The original rally required a design-review disjointness check, rally.yaml update, and implementation_order field. The beads approach is strictly superior here.

**Decision persistence via `bd remember` replaces decisions.md cleanly.** Six decisions seeded, all surfaced in `bd prime` output at session start. No separate file to manage, no path to remember, no drift between file and reality.

**Atomic claim prevents race conditions.** `bd update --claim` is compare-and-swap. Orbit has no equivalent — two agents could theoretically start the same card in a rally if timing is unlucky.

**Close reasons provide the audit trail.** Each bead's close reason records what shipped. This replaces the rally.yaml per-card status + PR field, with the advantage that it's queryable.

## What was missing

**No AC-level tracking within a bead.** Beads tracks status at the bead level (open/in_progress/closed). Within a bead, the description can contain a checklist, but there's no `Current AC:` pointer, no structured parsing of checklist state, and no `bd show` output that says "you're on ac-04 of 12." An agent resuming a bead mid-implementation would need to read the description and figure out where it left off.

This is the single largest gap. Orbit's progress.md with its `Current AC:` pointer, gate enforcement, and detour tracking is more granular than anything beads offers natively.

**No gate AC concept.** Beads checklists are unordered. Orbit's `ac_type: gate` enforces "don't start ac-05 until ac-04 is checked off." Without this, an agent could skip a design-decision gate and go straight to implementation. Gates were the most frequently exercised mission-resilience mechanism in the original rally.

**No cold-fork review.** Beads has no concept of spawning a fresh-context reviewer who reads the spec and diff cold. This is orbit's primary quality mechanism (per decisions 0004–0007). The merge-approval step is ceremony; the cold-fork review itself catches real defects.

**No iteration budget or escalation.** Beads has no concept of "you've tried three times, stop and escalate." Orbit's three-iteration budget with semantic reason labels (recurring_failure, contradicted_hypothesis, diminishing_signal) prevents infinite loops. This would need to be layered as convention on top of beads.

**No disjointness detection.** Beads handles the *result* of a parallel-to-serial discovery (add a dependency edge) but not the *detection*. The original rally's disjointness check extracted files, symbols, and references from each card's design and computed the intersection. That analysis still needs to happen somewhere.

**Epic appears in `bd ready`.** The parent epic showed up as ready work alongside its children. An orchestrating agent might accidentally claim the epic instead of working a child bead. In orbit, rally.yaml is never "work" — it's state.

**`bd prime` output is large.** The full prime output is ~150 lines of context including the command reference, session close protocol, and all memories. Orbit's session-context.sh is more targeted — it only surfaces what's active (current drive/rally state, next AC, drift notices). For a cron-based agent running hourly, the prime overhead matters.

## Spec schema: what was load-bearing

The beads-flow document asked: "If you find yourself wanting spec fields back, that tells you precisely which schema discipline was load-bearing."

I wanted these back:

1. **Acceptance criteria with `ac_type`** — gate/code/doc/config typing drives both runtime enforcement (implement skill) and review discipline (review-spec Pass 1 checks gate verification fields). Without types, the agent can't distinguish "design decision that blocks subsequent work" from "test coverage requirement."

2. **Hard constraints as a separate section** — constraints are non-negotiable boundaries (platform, compatibility, performance). ACs are the work. Mixing them in a freeform bead body loses the distinction that review-spec Pass 1 relies on.

3. **Verification field per AC** — the review-spec structural check validates that gate ACs have non-empty, non-placeholder verification approaches (≥20 chars). This is a deterministic quality gate that catches under-specified gates before implementation starts.

I did **not** want these back:

- **Spec metadata** (test_prefix, interview_ref) — low value, rarely used
- **Deliverables section** — the code is the deliverable; listing paths upfront is speculative
- **Full spec YAML schema** — the overhead of maintaining a schema file per work item is the problem beads solves

## Verdict

**Beads replaces orbit's rally/drive orchestration layer cleanly.** The dependency graph, auto-ready query, atomic claim, and `bd remember` are strictly better than rally.yaml, drive.yaml, progress.md scanning, and decisions.md.

**Beads does not replace orbit's intra-bead execution discipline.** Gate ACs, typed acceptance criteria, cold-fork review, and iteration budgets are load-bearing. These need to be layered on top of beads as orbit conventions, not abandoned.

**The migration is a hybrid:** beads as the orchestration substrate (replaces rally/drive state management), with orbit conventions for execution discipline within each bead (replaces spec.yaml schema with something lighter but still structured).

## Open decisions for spec

1. **AC structure within beads.** Options: (a) checklist in bead description with a convention for gate markers, (b) sub-beads for each AC with dependency edges encoding gate ordering, (c) a companion file (lighter than spec.yaml) that the orbit skill reads. Option (b) maps naturally to beads but may be too granular — 12 sub-beads for one card is a lot of graph noise.

2. **Cold-fork review: keep, adapt, or drop.** The trial doesn't answer this — it's a process decision. Options: (a) keep cold-fork as-is (agent spawns fresh reviewer), (b) replace with objective-function check + test coverage as the merge gate, (c) keep cold-fork but make it optional per autonomy level. The merge-approval-ceremony memo says the review is the gate, not the merge step.

3. **`bd prime` vs targeted context injection.** Options: (a) use `bd prime` as-is (accept the context overhead), (b) customize PRIME.md to strip the command reference and only show active work + memories, (c) keep session-context.sh as a complementary layer that reads from beads rather than YAML files.

4. **Epic-in-ready noise.** Options: (a) convention — agents ignore epics in ready queue, (b) use `bd ready --type task` to filter, (c) pin the epic (pinned status removes it from ready).
