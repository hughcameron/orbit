# Discovery: Recall as a first-class orbit capability

**Date:** 2026-05-25
**Interviewer:** Claude (Opus 4.7)
**Cards (prior art):**
- .orbit/cards/0023-memory-loop.yaml — write + auto-inject ("the recall half")
- .orbit/cards/0037-memory-gates-decisions.yaml — gate at decisions ("the enforcement half")
**Open spec (prior art):** .orbit/specs/2026-05-20-memory-match-bm25/spec.yaml — ranker upgrade, empty ACs, deferred
**Memos in scope:**
- .orbit/memos/2026-05-17-read-the-cited-source.md — processing discipline (cite-reading), unfolded
**Mode:** discovery

---

## Context

The author framed recall as a major lever for pillar 2 (agent self-learning). The opening four-layer model — **authorship → retrieval → surfacing → processing** — is the spine. Card 0023 covers write + auto-inject; card 0037 covers decision-time gating; the BM25 spec covers ranking; the cited-source memo covers processing discipline. None of the existing substrate provides a *unified* recall surface across artefact types.

## Q&A

### Q1: Goal — what does "good recall" look like?
**Q:** Right memory at right moment, processing discipline, durable authorship, or the whole system?
**A:** Right memory at right moment — surfacing is the load-bearing layer.

### Q2: Failure moments — where does surfacing fail today?
**Q:** Session start (prime ranker misses), skill entry (no memory match at open), mid-flow (momentum carries past), sub-agent forks (no memory context)?
**A:** Skill entry AND mid-flow. Both are "agent didn't think to call memory match." Skill entry is the structural version (we can prescribe), mid-flow is the behavioural version (we can't).

### Q3: Scope — what counts as the recall surface?
**Q:** Memories only, memories + choices, + cards, or substrate-wide?
**A:** Substrate-wide. Memories + choices + cards + specs + memos. Recall is an aggregation capability, not a memory feature.

### Q4: Surface style — how invasive?
**Q:** Pulled at structural moments, pulled + pushed mid-flow, always-on injection, or pulled minimum-intervention?
**A:** Pulled + pushed hybrid. Skill SKILL.md mandates recall step at entry (like /orb:code-investigate now); PreToolUse hook observes agent inputs and emits a "consider: <matched-artefact-pointer>" nudge when topic-match fires mid-flow. The 0037 "callable surface + structural gates" pattern extended substrate-wide.

### Q5: Success signal — how do we know recall improved?
**Q:** Zero "answer existed and was ignored" incidents, substrate-cite rate in agent decisions, reduced rediscovery, or agent-reported friction drops?
**A:** Substrate-cite rate. Mechanically measurable — grep agent output (PR bodies, spec goals, design notes, AC verifications) for `[[memory-key]]` / `choice NNNN` / `card NNNN` / spec-id patterns. Pre-ship baseline + post-ship trend is the audit shape.

### Q6: First-spec staging — what's the minimum viable cut?
**Q:** Verb + skill obligation broad, pilot on one skill, verb-only, or hook-first?
**A:** Verb + skill obligation — broad first push. Spec 1 ships `orbit recall <topic>` (substrate-wide fan-out, existing rankers) AND updates pipeline-skill SKILL.md prose to mandate a recall step at entry. Mid-flow PreToolUse hook is spec 2. BM25 stays its own deferred spec. Cite-rate baseline + audit is spec 3.

---

## Summary

### Goal
The agent gets the relevant substrate-wide answer (memory / choice / card / spec / memo) at the moment that matters — skill entry by mandatory pull, mid-flow by substrate push. Measured by substrate-cite rate trending up in agent decisions post-ship.

### Constraints
- Recall surface is **substrate-wide**, not memory-only. Existing per-type verbs (`orbit memory match`, etc.) coexist; the new `orbit recall` verb fans out across them.
- Existing artefact shapes (cards, choices, memories, specs, memos) stay unchanged — no write-side discipline added in this push. Cite-shape for memories (cited-source memo) is a separate follow-up under card 0023.
- Pull is mandatory at skill entry per SKILL.md prose. Push is hook-mediated; PreToolUse nudges, agent decides.
- Existing rankers (body-token overlap, label-overlap) ship first; BM25 upgrade is a parallel deferred spec.
- The recall verb returns results tagged by artefact type so the agent can filter / drill down.

### Success Criteria
- After ship: substrate-cite rate in agent output rises significantly against the pre-ship baseline (measurement window: N sessions or N PRs).
- Each pipeline skill (tabletop, implement, design, spec, review-pr, review-spec, researcher) carries a mandatory recall step at entry per SKILL.md prose.
- `orbit recall <topic>` returns results across memories, choices, cards, specs, memos in a single response.
- Substrate-cite-rate audit verb (or grep pattern) measurable from the substrate; baseline captured at ship.

### Decisions Surfaced
- **Substrate-wide vs memory-only**: chose substrate-wide. Recall is an aggregation capability, not a memory feature. Existing 0023 (memory-only narrow scope) coexists — recall layers across it. Likely warrants a new card.
- **Pull-push hybrid vs pull-only**: chose hybrid. Skill-prompt-only is insufficient (per 0037 scenario 6); structural pull at entry + substrate push during mid-flow mirrors the 0037 enforcement pattern.
- **Broad first push vs pilot-on-one-skill**: chose broad. Spec 1 covers verb + skill obligation across all pipeline skills. (Per the investigation-orchestration spec just shipped, halt-revert to phased rollout remains available if the broad push burns the budget.)
- **Substrate-cite rate vs zero-ignored-memory rate**: chose cite-rate. Mechanically measurable from agent output; aligns with the investigation-orchestration audit pattern (per-repo investigation-before-edit ratios at +4w).

### Implementation Notes
- `orbit recall <topic>` likely lives as a new top-level CLI verb with `--type` filter (memory, choice, card, spec, memo) and `--limit N`. Fans out across existing match/search verbs. Returns canonical-yaml-friendly result tuples (id, type, score, snippet).
- PreToolUse hook (spec 2): observe `Skill`, `Bash`, `Edit` tool inputs. Topic extraction from args + first N lines of conversation turn. Match against substrate index (the SQLite index already exists). Emit "consider: <id>" nudges via stderr, agent decides.
- Skill SKILL.md updates (spec 1): each pipeline skill gains a "Recall pre-flight" section in the canonical position (between "Pre-flight" and the work loop). Mirrors the structural-investigation pattern from the 2026-05-25 spec.
- Substrate-cite rate audit (spec 3): grep PR bodies, spec goals, design notes for `card NNNN` / `choice NNNN` / `[[memory-key]]` / spec-id patterns. Pre-ship baseline captured for the orbit repo and consumer repos before spec 1 lands.
- BM25 spec (already open, deferred): parallel track. The recall verb uses whatever ranker the memory-match verb uses, so BM25 ships transparently when ready.
- Existing reach-points to consider: `orbit memory match`, `orbit card search`, `orbit choice` (no search yet — would need one), `orbit spec list/show`. The recall verb is the umbrella; per-type verbs stay for specific lookups.

### Open Questions
- **Card creation**: substrate-wide recall is a new capability per METHOD.md's decision tree (new capability the product provides → card). Should we file a new card or extend 0023's scope? *Recommendation: new card. 0023's "project-scoped facts not codified elsewhere" boundary is intentionally narrow.*
- **Boundary with 0037**: 0037 covers decision-time surfacing via `memories_considered`. Does the new recall verb supersede 0037's inline-match pattern or coexist? *Recommendation: coexist. 0037's `memories_considered` is the close-time enforcement gate; recall is the open-time visibility verb. Different timing.*
- **Cite-shape for memories** (from cited-source memo): does substrate-wide recall force write-side discipline (e.g. `cites:` field on memory records)? *Recommendation: no — keep memory writes simple in this push. Cite-shape can be a follow-up under 0023.*
- **PreToolUse hook performance**: how heavy is the topic-match-on-every-tool-input? Cost budget for the hook. (Implementation detail — falls out at tabletop.)
- **Substrate-cite rate baseline measurement window**: how many sessions / PRs for pre-ship baseline? (Tabletop question.)
