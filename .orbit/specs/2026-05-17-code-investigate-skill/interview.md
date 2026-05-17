# Interview — `/orb:code-investigate` skill design

**Date:** 2026-05-17
**Spec:** `2026-05-17-code-investigate-skill`
**Card:** `0025-codebase-mastery`

## Origin

Card 0025 (codebase-mastery) was filed as `planned` in early May 2026 with seven scenarios promising rtk-wrapped commands, tree-sitter for AST queries, ripgrep/ast-grep for search, token-frugal defaults, and Karpathy's four principles operable in tooling. The audit at `.orbit/memos/2026-05-17-codebase-mastery-audit.md` found five of the seven scenarios aspirational, two partial, and no consumer-repo evidence that agents were reaching for these tools or noticing their absence. The referenced `ops/RTK.md` did not exist; `tree-sitter` appeared once in the repo (in a memo arguing against it); `ast-grep` appeared zero times.

The framing correction at the heart of this spec: 0025 is an **agent-equipment** capability, not a user-invokable surface. The user benefit is indirect — stronger advice before implementation, fewer wrong fixes, less rework — and the user shouldn't have to type any new commands. The audit's "no consumer demand" finding is evidence the capability hasn't been delivered, not evidence it doesn't matter.

A recent session in a sibling private project provided a sharp data point: under pressure, the agent skipped the design and memory layers entirely, designed a workaround in chat, and lost roughly half a day. The memory pointing at the correct fix was present but unread. That incident sits behind cards 0037 (memory-gates-decisions) and 0038 (skills-infer-or-prompt-before-halt) and informs this spec — the spec's purpose is to add the third leg of the agent-side substrate-engagement cluster.

## Interview record

### Q1 — Scope shape

> What's the minimum viable wedge — coaching prose only, coaching + thin orbit code wrappers, or coaching + a shared sub-skill?

**Answer:** Coaching + a shared sub-skill. Reason given: "we aren't yet clear on the best ways to investigate code — hoping that a skill will help us learn this."

The sub-skill is intentionally a **learning surface**. It encodes what we know now (modest) and has explicit hooks for what we learn over time (more important than initial completeness).

### Q2 — Invocation trigger

> How does the calling skill know to invoke `/orb:code-investigate` — at specific decision moments inside skills, always at the start of code-touching skills, or agent-callable any time?

**Answer:** Embedded in other skills AND available as go-to before any non-trivial code change. The user drew an analogy to Claude Code's Read-before-Edit pattern: "much like reading the markdown file before editing — code-investigate could reveal relationships to make changes better & sharper."

That analogy is structurally important. Read-before-Edit is a hard tool-level gate. The user's framing signals they want comparable force.

### Q3 — Enforcement strength

> Hard hook gate (Read-before-Edit analogue), soft hook nudge, prose convention only, or hard hook with explicit bypass?

**Answer:** Soft hook nudge. PreToolUse hook warns when Edit/Write is invoked without a recent `/orb:code-investigate` call, but doesn't block. The edit proceeds; the warning accumulates as a signal that gets observed over time.

Choice pairs cleanly with the "skill as learning surface" framing — if the warning fires often, we learn the threshold is wrong; if it stops firing because agents reach for the skill naturally, we learn it's becoming habit.

### Q4 — Skill scope

> Narrow query-shaped tool picker, multi-step investigation against a goal, or broad pre-change context gathering?

**Answer:** Both narrow and broad. Reason: "I see a case for the narrow query-shape regularly, I also think we need to meet the broad case so agents can be aware of the repo scale."

Two modes, same skill:
- **Narrow mode** — agent passes a specific query ("find the X matching pattern Y"). Skill picks the right tool (ast-grep, tree-sitter, rg, etc.) and returns matches.
- **Broad mode** — agent invokes for repo/neighbourhood context. Returns a synthesised picture: directory structure, hot files, where complexity lives, what's near the area being changed.

### Q5 — Learning loop

> How does the skill learn over time — append-only memories tagged 'code-investigate', skill self-edits its own examples, sibling notes file, or defer to a follow-on spec?

**Answer:** Append-only memories tagged `code-investigate`. Same mechanism card 0023 already provides; no new substrate.

Loop shape: invocations → tagged memories → periodic distillation lifts patterns into the skill prose. The skill itself stays stable on day one; the learning happens in memories and only periodically promotes into the skill prose.

## Decisions in summary

| Axis | Decision |
|------|----------|
| Shape | Shared sub-skill + embedded call-points |
| Trigger | Embedded in skills + go-to before non-trivial code changes |
| Enforcement | Soft hook nudge (warn, don't block) |
| Modes | Narrow (query) and broad (neighbourhood) |
| Learning | Append-only memories tagged `code-investigate` → periodic distillation |
| Name | `/orb:code-investigate` (working name; alternatives `/orb:investigate`, `/orb:scope` considered and rejected as either too broad or ambiguous) |

## Cluster context

This spec is the third leg of an agent-side substrate-engagement cluster filed today:

- **Card 0037 (memory-gates-decisions)** — when the agent encounters a decision, it reconciles against relevant memories.
- **Card 0038 (skills-infer-or-prompt-before-halt)** — when a skill is invoked missing context, it infers from substrate, prompts via menu, halts only as last resort.
- **Card 0025 (codebase-mastery), this spec** — when the agent investigates code, it reaches for AST-aware tools by default.

All three address agent-side substrate engagement (pillar #2). The cluster may eventually warrant a synthesis card; not opening one preemptively.
