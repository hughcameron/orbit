# /orb:code-investigate — stage-bound, not session-wide

**Date:** 2026-05-25
**Predecessor:** `.orbit/memos/2026-05-17-codebase-mastery-audit.md`
**Card:** 0025-codebase-mastery
**Spec shipped:** 2026-05-17-code-investigate-skill (8 days ago)
**Reopens:** ac-07 (4-week usage audit, earliest fire 2026-06-14) — in-repo signal is already conclusive without waiting

## The author's question

> *"How is orbit helping agents with efficient search and code mastery? I'm not seeing it used day to day. It's like agents don't even know about it."*

## In-repo signal (2026-05-25)

| Probe | State |
|---|---|
| `.orbit/.code-investigate-recent` marker, this session | Absent — zero invocations across ~15 reads + greps + memory queries |
| Memories carrying label `code-investigate` | 3, all *about* the skill (hook plumbing, rg shim, misleading rg prose). Zero are products *of* using it |
| Hook registration (`code-investigate-nudge.sh`, `Edit\|Write` matcher) | Live (confirmed 2026-05-20 session-close) |
| Hook trips this session | Zero — Read/Bash-only session; post-hoc hook only fires on Edit |

Eight days of operation; zero memories that record an investigation closing back to the substrate; one substantive session that didn't reach for the skill.

## Why the plumbing doesn't move behaviour

1. **Opt-in by name.** The agent has to type `/orb:code-investigate`. Skills aren't browsed.
2. **Post-hoc hook.** `PreToolUse Edit|Write` fires *after* the agent decided to edit. Adds warning noise; doesn't change pre-edit behaviour. Read/grep-heavy sessions don't trip it at all.
3. **Skim-past coaching.** ac-04 placed imperative prose in `implement`/`researcher`/`review-pr` SKILL.md ("Run /orb:code-investigate ..."). Prose advice gets skimmed.
4. **Call-points only fire when those skills are entered.** Most main-context work (this session included) doesn't traverse `/orb:implement`. The coaching prose is unreachable.
5. **Memory-write closing instruction unused.** The "if something non-obvious surfaced, write a memory" reach has fired zero times in 8 days.

Continues the 2026-05-17 audit's read: tooling that's not *invoked* is not token-frugal; it's just not used.

## The stage-bound principle (the load-bearing claim)

SessionStart auto-fire was the wrong recut: it conflates "agent opens terminal" with "agent is about to change code". Different sessions, different stages, different token budgets.

**Code-investigate belongs to *workflow stages*, not sessions.** The orbit pipeline has natural moments where investigation pays:
- **tabletop Q8** ("adjacent code") — the explicit question is what to investigate
- **implement pre-flight** — Karpathy's "think before coding"; needs the surface known before any non-trivial change
- **review-pr call-site check** — call sites, test coverage, related-doc presence are by definition investigation
- **researcher** — before opening any new thread that touches code

The current shape has these stages *coach* the call in prose. The mechanism that would move the dial is for the parent skill to **orchestrate** the call as a structural step (or **gate** entry on the marker), not advise it.

Token-frugality is preserved by stage-boundedness: investigation fires when the workflow stage genuinely needs it, not on every session.

## Mechanisms held in reserve (pick in follow-up tabletop)

| Mechanism | Shape | Trade-off |
|---|---|---|
| **Orchestrate** | Parent skill's prose step *N* literally invokes `/orb:code-investigate <mode> --scope <derived>`. Agent doesn't decide. | Zero friction at stage entry; loses agent judgment on scope. Risk: orchestrated invocation may be noisy / off-scope without operator-tunable cues. |
| **Marker-gate** | Parent skill's pre-flight reads `.orbit/.code-investigate-recent`; AUQ-block if marker absent for the relevant files. (a) run now, (b) waive with reason. | Forcing function; introduces AUQ friction on every stage entry; agents may default to waive. |
| **Sharper coaching** | Tighten ac-04 prose to imperative-with-block ("STEP N: run code-investigate before STEP N+1"). | Lowest cost; same skim-past failure mode. |

Recommendation deferred to tabletop on card 0025.

## What this memo is NOT

- Not a card update — card 0025 maturity (`emerging`) and goal already name `/orb:code-investigate` as the delivery vehicle; the gap is mechanism, not framing
- Not a new spec — the contract for a follow-up needs the orchestrate-vs-gate pick first, which is tabletop's job
- Not a replacement for ac-07's 4-week audit — the in-repo signal is enough to act on *now*; ac-07's consumer-repo data will refine, not reverse, this read

## Next move

`/orb:tabletop 0025` with this memo + 2026-05-17-codebase-mastery-audit.md as inputs. The tabletop's job: pick orchestrate / marker-gate / hybrid, and scope the spec.
