# /orb:code-investigate — empirical audit across 5 repos, 35 sessions

**Date:** 2026-05-25
**Predecessors:**
- `.orbit/memos/2026-05-17-codebase-mastery-audit.md` (narrowed audit at ship; "re-run when ≥20 session yamls exist")
- `.orbit/memos/2026-05-25-code-investigate-stage-bound.md` (in-repo signal + stage-bound principle)
**Card:** 0025-codebase-mastery
**Spec under audit:** 2026-05-17-code-investigate-skill (ac-07 — observation band, earliest fire 2026-06-14; this audit fires the equivalent signal early)

## The question

> *"How is orbit helping agents with efficient search and code mastery? I'm not seeing it used day to day. It's like agents don't even know about it."*

## Method

Five parallel sub-agents, one per repository, sampling the most-recent JSONL session transcripts from each. For each session: tool-call distribution, skill invocations, Edit/Write events tagged investigated-before-edit (Read or Grep on the same file within the prior 10 tool calls) vs not, hook-fire count (literal warning string `"consider /orb:code-investigate before editing"`).

Five repos: `arcform`, `orbit`, `brightfield`, `finetype` (public); plus one private repo (referred to as "repo P", no quoted content, aggregate counts only). 35 sessions sampled total, 1,725 edits, ~9,000 tool calls aggregate.

## Headline numbers

| Repo | Sessions | code-inv invocations | Hook fires | Inv-before-edit | Edits |
|---|---|---|---|---|---|
| arcform | 3 | 0 | 6 | 42.9% | 294 |
| orbit (dogfood) | 10 | 0 | 224 | 42.3% | 496 |
| brightfield (pre-ship baseline) | 2 | 0 | 0 | **47.1%** | 136 |
| repo P (private) | 10 | 0 | 247 | 40.2% | 443 |
| finetype | 10 | 0 | 133 | 41.3% | 356 |
| **Total / weighted average** | **35** | **0** | **610** | **~42%** | **1725** |

Three facts:
1. **Zero invocations.** `/orb:code-investigate` reached for zero times across 35 sessions, multiple weeks, every repo including orbit's own dogfood.
2. **Hook is inert.** 610 post-ship warnings; 0 conversions to invocation. One sub-agent's wording: "ambient noise."
3. **Baseline unchanged.** Pre-ship investigation-before-edit ratio (47.1%) is *higher* than post-ship average (~42%). The ship has not moved the behaviour it was designed to move.

## Which skills agents *do* reach for

| Skill | Total invocations across 35 sessions |
|---|---|
| `orb:drive` | 18 |
| `orb:implement` | 16 |
| `orb:review-spec` | 15 |
| `orb:spec` | 10 |
| `orb:tabletop` | 7 |
| `orb:release` | 6 |
| `orb:rally` | 6 |
| `orb:design`, `orb:distill` | 4 each |
| `orb:card`, `orb:discovery`, `orb:memo`, `orb:setup` | 2 each |
| `orb:prioritise` | 1 |
| **`orb:code-investigate`, `orb:topology`, `orb:researcher`** | **0 each** |

~95 orbit-skill invocations across 35 sessions (~2.7 per session). **Zero are investigation-class skills.** The pipeline-orchestration skills get reached for; the investigation skills do not.

## Three structural failures

The data points to three independent failure modes, each on its own causal path.

### 1. Skim-past coaching prose

`ac-04` of the shipping spec placed imperative prose call-points in `implement`/`review-pr`/`researcher` SKILL.md ("Run /orb:code-investigate (broad mode) on the module the next AC touches before proposing any non-trivial change"). The data: 16 `orb:implement` invocations across the sample, 0 follow-on `orb:code-investigate` invocations. Agents enter implement, read the prose, proceed without the call. Prose advice nested inside skill text is structurally skim-able.

### 2. Post-hoc hook is inert

The PreToolUse hook fires on `Edit|Write` *after* the agent has decided to edit. 610 fires, 0 conversions. The hook is a passive nag at the wrong moment: by the time it fires, the agent is mid-action. The warning text appears in stderr alongside the tool result; the next tool call rarely changes course.

Worse, the hook misses pure-Read sessions entirely. The session driving *this* memo did 15+ reads and zero edits before generating this paragraph — the hook never fired despite extensive code-mastery-relevant work.

### 3. Tabletop, drive, and rally have no call-points

Prose audit of stage SKILL.md files:

| Skill | code-investigate mentions |
|---|---|
| `implement` | 1 (imperative) |
| `review-pr` | 1 (imperative) |
| `researcher` | 1 (imperative) |
| `tabletop` | **0** |
| `drive` | **0** |
| `rally` | **0** |

Three of the six load-bearing pipeline skills don't reference the investigation skill at all. Tabletop is especially load-bearing: Q8 is literally "adjacent code" — the explicit moment for code-investigate to fire — and the question's prose doesn't mention it. Drive orchestrates the whole pipeline. Rally fans out drives. None route to investigation.

## Where the integration surface actually is

The reachable surface is **the orbit pipeline skills**, not the standalone `/orb:code-investigate` slash command. Agents enter drive/implement/tabletop/review-spec regularly; they don't browse the skill catalogue and type code-investigate. The integration question is therefore not "how do we get agents to invoke code-investigate" — it's "how do the pipeline skills cause investigation to happen at the right stage, at the right depth, for the right tokens".

This reframes the candidate mechanisms held in reserve in the 2026-05-25 stage-bound memo:

| Mechanism | Shape | What the data suggests |
|---|---|---|
| **A — Orchestration** | Parent skill's prose step *N* literally invokes `/orb:code-investigate <mode> --scope <derived>`. Agent doesn't decide. | Strongest fit for the data — agents reach for parent skills, parent skills currently don't reach further. Token cost is bounded because the parent skill scopes the invocation. |
| **B — Marker-gate** | Parent skill pre-flight reads `.orbit/.code-investigate-recent`; AUQ-blocks if absent for relevant files. (a) run now, (b) waive. | Forcing function; risk of waive-defaulting given 610-zero conversion track record on the existing nag. |
| **C — Composition** *(new — surfaced by the data)* | Fold the investigation discipline directly INTO the pipeline skills' prose + behaviour. No standalone slash command to invoke. The hook tracks "did adequate investigation occur this session/stage" rather than "did /orb:code-investigate fire". | Closest to what agents already do — investigation via Read/Grep/Bash. Drops the indirection layer. Cost: code-investigate-as-a-skill becomes a library of patterns the parent skills reference, not an invocable surface. |

The standalone slash command's value was always its discoverability. The data shows discoverability is not the problem — invocability is. Even with 610 nudges to invoke it, agents didn't.

## What this memo does NOT recommend

- **Not a card change.** Card 0025's `goal` and scenarios are intact; the gap is mechanism, not framing.
- **Not a spec yet.** The orchestrate / gate / compose pick needs a tabletop on card 0025 with this memo as input.
- **Not a hook tweak.** Adjusting the warning text or matchers will not close a 610-to-0 conversion gap. The post-hoc shape is the failure, not the prose.
- **Not waiting for ac-07's 2026-06-14 audit.** The signal is already conclusive. ac-07 can fire on schedule with longer-window data; it will refine, not reverse, this read.

## Next move

`/orb:tabletop 0025` with three inputs:
1. `.orbit/memos/2026-05-17-codebase-mastery-audit.md` — narrowed audit at ship
2. `.orbit/memos/2026-05-25-code-investigate-stage-bound.md` — stage-bound principle
3. This memo — empirical audit with 5-repo data

The tabletop's job:
- Pick A / B / C (or hybrid)
- Scope the spec(s) that result — likely one per affected pipeline skill, or one cross-cutting spec depending on the chosen mechanism
- Define success measurement — investigation-before-edit ratio target, hook-fire→action conversion target, or a different metric if the chosen mechanism makes those obsolete
- Halt conditions and kill conditions per the tabletop methodology

## Method notes (for re-running)

- Source: `~/.claude/projects/<encoded-repo-path>/<session-id>.jsonl`
- Volume: 148 jsonl files / 636 MB across 5 repos
- Parse via python (jq works but slower for nested tool_use blocks)
- Aggregate per-session before crossing repos — per-repo behaviour varies (arcform skews much more edit-heavy than orbit's dogfood)
- Privacy: hydrofoil's content stays inside the sub-agent's working memory; only aggregate counts surface
- The 5 sub-agent reports are not persisted as artefacts — re-run from source if needed
