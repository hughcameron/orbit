# Codebase-mastery audit (narrowed)

**Date:** 2026-05-17
**Subject:** Card 0025 (codebase-mastery) — promise vs reality
**Why narrowed:** session-handover only landed yesterday; consumer repos have zero session yamls accumulated. The qualitative session-trace evidence the full audit needs does not yet exist. This audit covers what *is* auditable cheaply right now: shipped surface area, skill coaching, consumer-memo signal, and one recent incident as a single qualitative data point.

## Per-scenario classification

| # | Scenario (paraphrased)                                              | Status        | Evidence |
|---|---------------------------------------------------------------------|---------------|----------|
| 1 | rtk wraps command interactions by default                           | aspirational  | No `ops/RTK.md` exists at the path the card references. No `rtk` mention in `plugins/orb/skills/**` or `orbit-state/crates/**`. Only hit is a passing line in archived spec `tabletop.md` ("rtk unaffected"). |
| 2 | tree-sitter answers structural queries                              | aspirational  | `tree-sitter` appears once across the repo — in `.orbit/memos/2026-05-08-cross-reference-integrity-gap.md` arguing it's *not* the right tool for that case. Zero integration in `orbit-state/`. |
| 3 | ripgrep returns focused matches via a formalised skill              | partial       | `plugins/orb/skills/keyword-scan/SKILL.md` formalises an `rg`-preferred file-list (`-l`) pattern for substrate scans only. No skill coaches `rg` for source-code exploration. `/orb:implement` and `/orb:researcher` mention "Read" and "grep" only incidentally. |
| 4 | ast-grep handles structural patterns                                | aspirational  | Zero `ast-grep` references anywhere in repo source, skills, or substrate. |
| 5 | Token-frugality is the default constraint                           | partial       | The principle is stated in 0025 and inherited indirectly via the `-l`-only ripgrep pattern in `keyword-scan`. No skill enforces it for code reads; no harness-level token guard. Card 0033 (`see-the-tree`) explicitly cites 0025 as its "sibling on the code-domain side" but ships only substrate verbs. |
| 6 | Karpathy's four principles are operable                             | aspirational  | The principles are named in 0025's `so_that` and referenced in a Karpathy-skills repo URL; no tooling instantiates them. `/orb:implement` and `/orb:researcher` describe a workflow but don't reach for tree-sitter / ast-grep / rg in the structural sense. |
| 7 | Code-domain verbs (`orbit code stats / grep-ast / rg`) in v0.2+     | aspirational  | `orbit-state/crates/cli/src/main.rs` `Command` enum: `Spec`, `Task`, `Memory`, `Card`, `Choice`, `Session`, `Skill`, `Overview`, `Audit`, `Graph`, `Verify`, `Canonicalise`. No `Code` variant. The "namespace was reserved at v0.1 design time" claim is not visible in the CLI surface. |

Net: **1 partial (#3, substrate-scoped only), 1 partial-by-citation (#5), 5 aspirational.** Card maturity `planned` reflects this honestly — nothing here contradicts the stated state. The audit point is the *gap between claim and pull-weight*, not a mismatch with the maturity label.

## Memo-read findings — tool-reach signal from consumer repos

- `/home/hugh/github/meridian-online/finetype/.orbit/memos/` — 17 memos, all dated 2026-04-27. Keyword scan for `ripgrep|rg|tree-sitter|ast-grep|rtk|grep|find` returns three files; manual inspection shows the hits are the words "find" and "find out" used as English verbs ("can find it via docs", "should not need to resolve the taxonomy to find out"), not tool invocations. **Zero memos discuss tool choice, search strategy, or token cost of investigation.**
- `/home/hugh/github/hughcameron/hydrofoil/.orbit/memos/` — empty directory.
- `/home/hugh/github/meridian-online/orbit/.orbit/memos/` — one substantive 0025-adjacent reference: the cross-reference-integrity-gap memo argues tree-sitter and ast-grep are *not* shaped for a substrate-wide reference audit. That's a contraindication, not a use.

**Read:** consumer memos so far show no evidence agents are reaching for 0025's tools, and no evidence they're noticing the absence. The absence-of-evidence is itself thin — 17 memos from one repo on one day, none from the other.

## The recent incident — one data point

In a sibling private project this week, an agent hit an OOM blocker partway through a long replay. It invented a chunked-with-overlap workaround instead of using the underlying tool's first-class streaming mode. A relevant memory pointing at the streaming-mode documentation existed but was not consulted. The agent later self-reported that none of the in-session design decisions went through `/orb:design` or `/orb:spec` — all were resolved in chat. Roughly half a day was lost to the wrong fix.

What an agent under pressure reached for: ad-hoc chat-driven design and ad-hoc workarounds. **Not** the tools 0025 promises, **not** the memory layer, **not** orbit's design/spec ramps. One data point, but it's the shape of failure 0025 was meant to prevent.

## Gaps the evidence suggests

What 0025 does not cover that the audit-visible behaviour suggests agents need:

1. **A "before you reach for chat" tripwire.** 0025 assumes the agent picks the tool when it picks. The incident shows the agent doesn't always reach the picking step — it stays in chat. Tooling that's not *invoked* is not token-frugal; it's just not used.
2. **Memory-consult discipline.** A relevant memory existed and was not opened. 0025 is silent on memory-as-first-stop; this audit suggests the gap is upstream of tool selection.
3. **Coaching at the skill layer, not just the card layer.** `keyword-scan` is the *only* skill that formalises a search tool choice, and it's substrate-scoped. `/orb:implement`, `/orb:researcher`, `/orb:review-pr` do not coach `rg`/tree-sitter/ast-grep for code exploration. The card promises capabilities; the skills don't route to them.
4. **A defined "token-frugal default" mechanism.** #5 in the card states the principle. Nothing checks or enforces it. There is no `tokens-spent-this-turn` signal, no `rg | head` convention codified, no per-skill token budget.
5. **An ops/RTK.md the card references.** The reference is dangling. If rtk is load-bearing for scenario 1, its absence as a documented reference is itself evidence the card has not been pulled through.
6. **A bridge from incident to spec.** The recent incident produced no spec, no card update, no choice. The substrate has no record it happened. 0025's pillar (agents reach for the right tool) needs a feedback loop the audit cannot see in operation.

## Status

**Narrowed audit complete.** Full audit deferred until session yamls accumulate in consumer repos — the qualitative trace of *what an agent actually reached for* across many sessions is the evidence that would let us judge scenario #5 ("token-frugality as the default constraint") empirically rather than by inspection. Re-run when ≥20 session yamls exist across consumer repos.

No card changes proposed in this memo, per audit discipline.
