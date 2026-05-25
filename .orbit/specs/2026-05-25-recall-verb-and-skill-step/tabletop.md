# Tabletop — Substrate recall: verb + skill-entry step

**Date:** 2026-05-25
**Facilitator + domain expert:** Hugh Cameron
**Scribe + driver:** Claude (Opus 4.7)
**Cards in scope:** .orbit/cards/0044-substrate-recall.yaml
**Methodology:** Card 0019 — 10-question methodology (compact, single-pass; Q1–Q3 + Q5–Q6 already covered in `.orbit/specs/2026-05-25-recall-capability/interview.md`); choice 0017 — output is contract, not solution
**Output spec:** .orbit/specs/2026-05-25-recall-verb-and-skill-step/spec.yaml
**Source discovery:** .orbit/specs/2026-05-25-recall-capability/interview.md

---

## Values

**Load-bearing value: surfacing the right substrate artefact at the moment that matters.** Pillar 2 (agent self-learning) is downstream of this — agents only accumulate competence across sessions if substrate accreted by past sessions is actually surfaced when current work touches its topic. Today an agent has to *know* which artefact type to query and *remember* to query it; recall removes both burdens.

Substrate-cite rate is the downstream measure, not the value itself.

## Trade-offs

- **Fan-out across existing per-type verbs vs new unified ranker** — acceptable to fan out. The verb wraps `orbit memory match` + `orbit card search` + `orbit choice search` + new substring search for specs and memos. Trade-off: rankers are heterogeneous (memory has token+label overlap; card/choice have substring). Result merging normalises scores per-type. The BM25 spec (2026-05-20-memory-match-bm25, deferred) will improve memory ranker independently; the unified verb inherits whatever's there.
- **Mandatory skill-entry recall vs opt-in** — accepted as mandatory. 0037 already declared "skill-prompt-only enforcement is insufficient" — but at *skill entry* the prompt is structural (the SKILL.md prose is the contract). Same shape as `/orb:code-investigate` orchestrated per spec 2026-05-25-investigation-as-pipeline-step.
- **New top-level verb `orbit recall` vs subcommand `orbit substrate recall`** — accepted as top-level. Recall is conceptually distinct from substrate-classify (which is layout-shape inspection); a top-level verb mirrors the per-type verbs' position. Operator surface: 1 new top-level command (`orbit recall`).
- **Per-AC scope derivation vs explicit `<topic>` argument** — accepted as agent-typed explicit argument. Same shape as the investigation-orchestration spec — the agent picks scope at the call site from substrate they're already reading (card slug, spec id, AC description text, PR diff). Auto-derivation is deferred.
- **Including specs + memos in recall vs memory + choice + card only** — accepted as substrate-wide (5 types). The discovery answer was substrate-wide; the spec implements that. Cost: two new substring search paths (spec.yaml goals + AC descriptions; memo .md bodies).

The cut is the simplest cut that holds the load-bearing value.

## Halt conditions

- **Regression on existing per-type verbs.** Trigger: any `orbit memory match` / `orbit card search` / `orbit choice search` test in the workspace test suite turns red as a side effect of the recall verb's implementation. Revert path: `git restore .` on the affected verb's source; recall fan-out shells out to per-type verbs rather than refactoring shared code.
- **Workspace test suite regression.** Trigger: `cargo test --workspace` drops below the 555 baseline (post verify-surfaces-migration-errors close). Revert path: bisect via `git rebase --interactive` on the spec's commit range; restore green before continuing.
- **Skill SKILL.md edits break existing skill invocation.** Trigger: invoking any updated pipeline skill (`/orb:implement`, `/orb:tabletop`, etc.) after the SKILL.md edit lands fails to load or halts before its work loop. Revert path: `git restore plugins/orb/skills/<name>/SKILL.md` on the offending skill; re-test in isolation before re-applying.

## Escalation triggers

- **Fan-out verb returns confusingly-merged results across types.** Condition: a recall query returns results where the ranking interleaves types in a way that obscures the most-relevant artefact (e.g. a low-overlap card outranks a high-overlap memory because the rankers' score scales aren't normalised). Surface: the test fixture's query + expected vs actual top-3 + the per-type raw scores. Action: AUQ author — (a) lock the result presentation to type-grouped (memories first, then choices, then cards, etc.) with per-type ranking only; (b) attempt a global score normalisation; (c) defer the ranking concern to BM25 spec and ship type-grouped for now.
- **Spec + memo search verbs are non-trivial to add.** Condition: implementing per-spec or per-memo substring search reveals it requires a new index, schema field, or substantial new layout code (>200 LoC each). Surface: the file paths + the proposed shape + the existing per-type-verb shapes for comparison. Action: AUQ author — (a) ship recall with the 3 existing types only (memory + card + choice), defer spec/memo search to a follow-up spec; (b) extend scope and absorb the LoC inside spec 1.
- **SKILL.md update across 7 skills creates cascading inconsistency.** Condition: midway through editing the 7 pipeline SKILL.md files, the recall-step prose needs different shape per-skill (e.g. /orb:tabletop wants different scope than /orb:implement), and a uniform stanza no longer fits. Surface: the diff of each SKILL.md edit so far + the inconsistencies + a draft per-skill stanza. Action: AUQ author — (a) accept per-skill stanzas, (b) revert to a uniform stanza and tolerate the suboptimal fit, (c) defer some skills to spec 2.

## Kill conditions

- **K1: substrate-wide recall claim.** If the fan-out verb returns too-noisy / too-irrelevant results across types to be useful (operator complaint or measurable: average top-5 result is irrelevant >50% of queries), the substrate-wide claim is dead. Pivot: ship recall as memory + card + choice only (the 3 types with mature rankers); spec/memo search becomes a follow-up.
- **K2: skill-entry pull-only claim.** If pipeline skills consistently skip the recall step despite SKILL.md prose (mirroring 0037's "skill-prompt-only is insufficient" outcome at the open-time), the structural-pull claim is dead. Pivot: accelerate the mid-flow PreToolUse hook (spec 2) and make it the primary surfacing mechanism; the verb stays but the skill-entry obligation downgrades to optional.
- **K3: cite-rate-as-success-signal claim.** If post-ship measurement shows substrate-cite rate is flat or noisy (no clear signal from the audit verb), the cite-rate metric is dead as the success signal. Pivot: switch to qualitative per-incident retrospectives (memo channel) for the +4w audit and treat the verb itself as the deliverable.

## Hot-wash

- **recurred**: the investigation-orchestration spec's structural-pull-at-skill-entry pattern is the natural shape here too. Same SKILL.md anchor position, same agent-typed-scope-at-call-site rule. Worth flagging as a transferable pattern — "structural skill-entry orchestration" might become a card of its own once it lands a third time.
- **surprised**: discovery converged in 6 questions on a clear pull+push hybrid with substrate-wide scope. The user's confidence on Q3 (substrate-wide) and Q6 (broad first push) was higher than I expected — both rejected the narrower laterals (memory-only, pilot-on-one-skill). That confidence justifies a slightly bigger first spec than I'd otherwise commit to.
- **friction**: spec + memo search verbs don't exist today. Adding them is real implementation cost — the discovery answer (substrate-wide) commits to that, but the budget for spec 1 widens.
- **meta-patterns-for-future-tabletops**: when discovery has already covered Q1/Q2/Q3/Q5/Q6, tabletop should default to a compact sidecar focused on Q4/Q7/Q8/Q10 + hot-wash. The full 10-question walk would re-litigate decisions already made. This is the second time today I've taken the compact route (see also `.orbit/specs/2026-05-25-verify-surfaces-migration-errors/tabletop.md`); worth filing as a tabletop SKILL.md tightening — a `discovery-pre-empts-tabletop` mode that runs Q4+Q7+Q8+Q10 only when the discovery interview is present.
