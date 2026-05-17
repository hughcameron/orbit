# Read the cited source — memory summaries are pointers, not substitutes

**Date:** 2026-05-17
**Triggered by:** A session in a sibling project where an agent encountered a memory whose body cited an external doc as the fix for a known problem. The agent read the memory's summary, designed a workaround based on the summary's framing, and lost roughly half a day to a wrong fix. The memory had pointed at the right answer — the agent never opened the doc.

Companion to the memo distilled into card 0037 (memory-gates-decisions). That card addresses *whether* the agent encounters the relevant memory at the decision moment. This memo addresses *how* the agent processes the memory once encountered.

## What surfaced

The memory in question cited a specific section of an external authoritative doc as the fix. The agent treated the memory's one-line summary as the actionable instruction and designed around it. The summary was directionally correct but lacked the mechanical detail the doc carried — and the workaround the agent built (a chunked-with-overlap scheme) was structurally incompatible with what the doc actually described (a streaming-mode primitive native to the underlying tool).

The agent's own retrospective:

> when an orbit memory cites an external doc as the fix, read that doc directly before designing around the diagnosis

This is the discipline rule. The memo names it so it survives the session that learned it.

## Why this matters as its own discipline

Memory entries are compressed by design — they have to be, because they are loaded into every session's context window. When a memory cites an external doc, the cite is **load-bearing**: it exists because the doc carries detail the memory could not fit. Acting on the summary alone is acting on the compression artefact, not on the knowledge the memory is pointing at.

The failure mode is structurally distinct from "memory was ignored":

- **Card 0037 (memory-gates-decisions) failure**: memory existed, agent never reconciled against it.
- **This memo's failure**: memory existed, agent *did* reconcile against it — but only against its summary, designing a workaround the summary's framing made plausible. The cited doc, which would have shown the workaround was unnecessary, was never opened.

Both failures end with the agent building the wrong thing. Their fixes are different.

## What would close it

Three candidate moves, ordered by force:

1. **Memory-format convention.** Memories that cite an external doc must do so in a recognisable form (e.g. a `cites:` field, or a leading `See: <path>` line) so agents and tooling can detect "this memory has a load-bearing external reference."

2. **Reconciliation includes the cite.** When an agent reconciles a memory at a decision moment (per 0037's mechanisms), the reconciliation explicitly includes reading any cited external source. The `memories_considered` field on `spec.yaml` (or its inline equivalent) records the cite-read as part of the reconciliation evidence, not separately.

3. **Skill-prompt rule for memory consumption.** `/orb:design`, `/orb:spec`, and any inline memory-match surface emit an instruction along the lines of: "for each surfaced memory that names an external doc, read the doc before proposing an approach that diverges from it." Lightweight but behavioural — likely insufficient alone, sufficient as reinforcement on top of (1) and (2).

(1) is the cheapest substrate change and the enabling condition for the other two. Without a detectable cite shape, (2) and (3) cannot fire reliably.

## Relation to card 0037

This memo is best treated as input to 0037's design pass rather than a separate card. The mechanisms above are refinements to 0037's "reconciliation" step — they specify what *good* reconciliation looks like when the memory contains a citation. If 0037's design absorbs them as additional acceptance criteria, no new card is needed. If the cite-shape convention turns out to be substantial enough on its own (a memory-format change that touches `orbit memory remember`, the SQLite schema, and the format docs), it may warrant a separate card at design time.

## Adjacent observation

The companion of "summaries are not substitutes for cited sources" is **memories should cite sources where they exist**. A memory that names a fix without citing where the fix is documented is harder to verify and degrades faster — the next agent has no anchor to confirm the memory is still accurate. A memory-shape convention should encourage citation, not only enable detection of it.

## Status

Memo only. No card filed. To be folded into the 0037 design pass (or split out as its own card if the cite-shape convention proves substantial). Until then, the discipline lives in this memo and in the agent's own session-end memory in the sibling project.
