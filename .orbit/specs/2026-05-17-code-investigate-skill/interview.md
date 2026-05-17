# Interview — `/orb:code-investigate` skill design

**Date:** 2026-05-17 (initial), 2026-05-18 (design pass 2)
**Spec:** `2026-05-17-code-investigate-skill`
**Card:** `0025-codebase-mastery`

## What good looks like

The agent owns the code — nobody else is better placed, so it should operate from mastery rather than caution. The discipline is simple: investigate before you change, and `/orb:code-investigate` is how that becomes cheap enough to default to. Narrow mode locates an implementation in a flash, names a call site, quotes an accurate stat; broad mode surfaces the structures that aren't immediately visible — where complexity clusters, what sits adjacent to the change surface, where existing patterns already live. The agent reaches for ast-grep, tree-sitter, ripgrep, and rtk-wrapped variants by default — that is what fluent expertise looks like, and it folds efficiency and best practice into the work itself rather than into the review afterwards. When it edits without first knowing the ground, a soft warning catches it as a peer prompt, not a gate. Tagged memories accrete back into the skill prose, so each session compounds the last.

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

## Design pass 2 — 2026-05-18

Residual UX/intent decisions on ac-02 / ac-03 / ac-04 / ac-05. Hook implementation path and atomic-write mechanics are routed to Implementation Notes rather than asked.

### Q1 — Nudge frequency (ac-03)

> How should the soft Edit/Write warning decide whether to fire — per-file, session-once, or time-bounded TTL?

**Answer:** Per-file, with broad-mode scope coverage. An entry in the marker file is either a literal file path or a scope prefix; the hook resolves either. Investigating a directory in broad mode marks every file under it.

Reason: "investigate before you change" is per-change, not per-session. Session-once goes silent regardless of next-file familiarity; TTL fires by clock rather than by what the agent is actually editing. Per-file ties the nudge to the decision moment, which is the discipline the 0037/0038 cluster argues for. Noise concern self-resolves because the agent doing its job naturally quiets the warning.

### Q2 — Call-point prose strength (ac-04)

> How directive should the embedded `/orb:code-investigate` call-points read in `/orb:implement`, `/orb:researcher`, and `/orb:review-pr`?

**Answer:** Strong default. Imperative voice across all three calling skills — e.g. "Run /orb:code-investigate (broad mode) before proposing any non-trivial change." Treats reach as the expected behaviour rather than a tool-in-the-box, and matches the mastery framing in the intent paragraph. Conditional phrasing was rejected because it relies on the agent correctly judging its own familiarity — which is the failure mode cards 0037/0038 were filed to address.

### Q3 — Memory-write strength (ac-05)

> How strong should the closing memory-write instruction be inside `/orb:code-investigate`?

**Answer:** Heuristic. Quality-gated: write a memory only when something non-obvious surfaced — a tool that worked where another failed, a query shape worth reaching for again, a structural insight worth keeping. Strong-directive was rejected because the learning loop wants signal, not log volume; distillation has to be able to find patterns without wading through trivial entries.

The split with Q2 is intentional. Call-points use strong directive because the cluster's concern is agents skipping the *discipline* (the investigate-before-change action). Memory-write strength is about *quality signal* for the learning loop, where convention-style softness is the right shape — judgement here is OK to delegate because the failure mode (a thin memory) is cheap, whereas a skipped investigation is the expensive failure.

## Decisions surfaced

| Axis | Decision |
|------|----------|
| Marker semantics (ac-03) | Per-file with broad-mode scope coverage. Entries are `(timestamp, kind=file\|scope, path)`. Hook resolves either: an Edit on file X passes if X is a literal marker entry or sits under any scope-prefix entry. Session is the natural lifetime — no clock TTL. |
| Call-point voice (ac-04) | Imperative. "Run /orb:code-investigate ..." across `/orb:implement`, `/orb:researcher`, `/orb:review-pr`. Strong default, not hedged. |
| Memory-write voice (ac-05) | Heuristic. Quality-gated closing instruction — write a memory only when something non-obvious surfaced. Reduces noise in the learning corpus; relies on agent judgement (cheap-failure surface, OK to delegate). |

## Implementation notes

Routed here rather than asked — these failed the implementation-question filter or surfaced from the evidence scan as starting context for the implementing agent.

- **No `plugins/orb/hooks/` directory exists yet.** Plugin manifest at `plugins/orb/.claude-plugin/plugin.json` is minimal (name, description, version, author — no `hooks` array). ac-02 needs both a hook script and registration in whichever surface the Claude Code plugin loader expects (plugin manifest hooks field vs `.claude/settings.json` hooks block — implementing agent to confirm against current Claude Code plugin format).
- **Marker file format.** `.orbit/.code-investigate-recent`, newline-delimited entries shaped `<unix-timestamp>\t<kind>\t<path>` where kind is `file` or `scope`. Atomic write via the existing orbit-state `write_atomic` convention (referenced in `plugins/orb/skills/setup/SKILL.md`). Marker is session-scoped; fresh-session detection via `.orbit/.session-id` comparison, or cleanup at session start.
- **Hook resolution.** When the hook fires on Edit/Write for file X, it reads the marker and considers X investigated if (a) X appears as a literal `file` entry, or (b) X is under a `scope` entry's prefix (e.g. scope entry `plugins/orb/skills/` covers `plugins/orb/skills/implement/SKILL.md`). Path matching is prefix-based on the repo-relative path.
- **Hook warning text.** "consider /orb:code-investigate before editing <file>" — grep-able per ac-02 verification. Stable phrasing is the load-bearing property; tone-tuning later if audits show it's being ignored as wallpaper.
- **`.orbit/.gitignore` entry.** Add `.code-investigate-recent` alongside the existing `.session-id` and `.session-card` entries.
- **ac-05 heuristic phrasing example.** "If something non-obvious surfaced — a tool that worked where another failed, a query shape you'd reach for again, or a structural insight worth keeping — write a short memory via `orbit memory remember` with tag `code-investigate`, capturing (a) the query or scope, (b) the tools reached for, (c) what was non-obvious." Closing instruction in the skill prose; not every invocation produces a memory.
- **ac_type per AC.** ac-01/02/03 default to `code` (functional smoke is the close evidence). ac-04/05/06 are `doc` (prose artefacts). ac-07 is `observation` (4-week audit window). Spec already reflects these — no changes required.

## Open questions

None at intent level. Plugin-format detail for ac-02 hook registration is the only outstanding item and is implementation-shaped — implementing agent to resolve.

## Cluster context

This spec is the third leg of an agent-side substrate-engagement cluster filed today:

- **Card 0037 (memory-gates-decisions)** — when the agent encounters a decision, it reconciles against relevant memories.
- **Card 0038 (skills-infer-or-prompt-before-halt)** — when a skill is invoked missing context, it infers from substrate, prompts via menu, halts only as last resort.
- **Card 0025 (codebase-mastery), this spec** — when the agent investigates code, it reaches for AST-aware tools by default.

All three address agent-side substrate engagement (pillar #2). The cluster may eventually warrant a synthesis card; not opening one preemptively.
