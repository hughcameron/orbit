# Design: Rally Approval is a Structured Prompt

**Date:** 2026-04-20
**Interviewer:** nightingale (rally sub-agent)
**Card:** orbit/cards/0006-rally.yaml

---

## Context

Card: *Rally — coordinated multi-card delivery* — 11 scenarios, goal: "agents coordinate multi-card delivery with fewer human touchpoints per card than serial drives".
Prior specs: 2 — `orbit/specs/2026-04-19-rally/spec.yaml` shipped the initial rally skill (0.2.19), and `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` (v1.3) hardened the sub-agent execution model with the thin-card guard (ac-01) and the pre-qualification retry framing (ac-04).
Gap: The §2b proposal-approval gate already uses `AskUserQuestion` with a three-option surface, but the card scenario (line 60) and the shipped skill disagree on the canonical labels, and the skill leaves four interaction details unresolved — label set, how modifications are collected, where per-card rationale lives, and whether the thin-card guard re-runs on the modify loop. This session closes those four gaps.

## Q&A

### Q1: Canonical option labels
**Q:** The card scenario names `approve-all / select-subset / decline`; the shipped skill §2b names `Approve as-is / Modify the list / Reject the rally`. One has to win. Do we (A) adopt the card's triple verbatim, (B) adopt the skill's triple and rewrite the card, or (C) take a hybrid `approve-all / modify-list / decline` that matches drive's hyphenated verb-style verdict labels and covers both add and remove paths?
**A:** Option C — `approve-all / modify-list / decline`. The add path is a real documented branch in §2b ("author names cards to add or remove"), so `select-subset` under-describes the interaction. `modify-list` matches drive 0005's `request-changes` / `read-full-review` hyphenated action-verb shape. Update card scenario line 60 so card and skill stay in lock-step; the card scenario is maturity:planned intent, so rewriting the then-clause is cheaper than mislabelling the surface on every rally run.

### Q2: How `modify-list` collects the edit
**Q:** `AskUserQuestion` returns one chosen option (plus an optional "Other" free-text). A modify response needs card paths or numbers, which is inherently free-form. Do we (A) fold the modification into the first AskUserQuestion's "Other" field, (B) use a second AskUserQuestion prompt (no pre-populated options, free-form) after the verdict is `modify-list`, or (C) drop out of AskUserQuestion and wait for the author's next conversational message?
**A:** Option B — two prompts. The verdict stays machine-parseable; the follow-up is an explicit labelled surface the author cannot miss. Distill's spec review (orbit/specs/2026-04-04-distill/review-spec-2026-04-04.md §42–46) already set this precedent — structured verdict, free-form follow-up. "Fewer touchpoints" is measured per-card, not per-decision: a proposal gate with a bounded second step is still one rally-level gate. The §2b modify loop becomes trivially re-enterable: verdict → modify instructions → revised list → verdict → ...

### Q3: Per-card rationale placement
**Q:** Rationale can live in (A) the markdown preview block only (status quo), (B) AskUserQuestion option `description` fields only (single surface, but breaks at 4+ cards because of description length caps), or (C) both — preview block owns card rationale, AskUserQuestion options stay terse and action-focused. Which placement?
**A:** Option C with strict roles. Preview block = per-card rationale (the evidence). AskUserQuestion option `description` = one-line action summary (e.g. "Proceed with all N candidates", "Add or remove cards before proceeding", "Abort the rally; offer individual drive as alternative"). No card rationale in the option descriptions. Rally's principle is "maximum clarity based on the best available evidence" at each gate — the preview block is the evidence, the AskUserQuestion is the decision surface. Collapsing them (B) breaks visibly at N=4+, and scenarios 7 and 9 on the card contemplate rallies of ~5–6 cards.

### Q4: Thin-card guard on the modify loop
**Q:** When the author picks `modify-list` and names additions/removals, does the thin-card guard (§2a) fire (A) before every re-prompt, (B) only at the final `approve-all`, or (C) only on additions during the modify step? Per-addition, per-revised-list, or once at the end?
**A:** Option A — re-run before every re-prompt. This establishes the invariant "every candidate list shown to the author has passed the guard". Option B wastes a decision: the author picks `approve-all` and is then told the list is invalid. Option C splits add-path and remove-path handling for negligible saving (the guard is a cheap scenario-count scan). The guard is a *pre-qualification gate*, not a *decision* — re-running it is not a rally-level strike, matching the ac-04 framing in the sub-agent-model spec ("pre-qualification retry, NOT a rally-level strike"). Constraint #1 in that spec uses *any* / *unconditional* language that supports the broader invariant.

---

## Summary

### Goal
Tighten the §2b proposal-approval gate in `plugins/orb/skills/rally/SKILL.md` so the approval surface is a single, consistently-labelled AskUserQuestion with a deterministic follow-up path for modifications and a pre-qualified candidate list on every re-prompt. Keep the card scenario and the shipped skill in lock-step.

### Constraints
- Scope is the §2b approval gate only. Do not redesign the proposal flow upstream, the thin-card guard at §2a (only its re-run behaviour), or the decision gate at §5.
- Canonical option labels are `approve-all / modify-list / decline`. No other labels on this surface.
- Modification collection is a second `AskUserQuestion` with no pre-populated options; empty response cancels the modification.
- Preview block owns per-card rationale; AskUserQuestion option `description` fields stay terse and describe actions only.
- Thin-card guard re-runs before every re-prompt in the modify loop; guard invocations are pre-qualification, not rally-level strikes.
- Card scenario 0006 line 60 then-clause must be updated from `select-subset` to `modify-list` so card and skill agree.

### Success Criteria
- §2b presents exactly the three canonical labels on every proposal.
- On `modify-list`, the lead issues exactly one free-form `AskUserQuestion` collecting the modification, then applies edits, re-runs the thin-card guard, and re-presents the revised candidate list with the same three labels.
- No candidate list is shown to the author unless it has passed the thin-card guard in the current loop iteration.
- The preview block above `AskUserQuestion` contains per-card rationale; the option descriptions contain only the action summary.
- Card 0006 scenario at line 60 reads `approve-all / modify-list / decline` end-to-end.

### Decisions Surfaced
- **D1 — Canonical option labels:** chose `approve-all / modify-list / decline` over the card's `approve-all / select-subset / decline` and the skill's `approve-as-is / modify-list / decline`, because the hyphenated action-verb shape matches drive 0005 and `modify-list` covers both the add and remove paths that §2b already documents.
- **D2 — Modification collection mechanism:** chose a second `AskUserQuestion` (free-form, no pre-populated options) over single-prompt "Other" free-text and conversational fall-through, because it keeps the verdict machine-parseable and the modification surface labelled, following distill's structured-verdict-plus-free-text precedent.
- **D3 — Per-card rationale placement:** chose both preview block (for card rationale) and AskUserQuestion option descriptions (for action summary) with strict roles, because rally's "maximum clarity based on best available evidence" principle means the evidence surface and the decision surface should not be collapsed, and option B's single-surface approach breaks at 4+ cards.
- **D4 — Thin-card guard on modify loop:** chose re-run before every re-prompt over final-gate-only and additions-only, because the invariant "every list shown has passed the guard" preserves the pre-qualification framing and avoids wasting author decisions on invalid lists. Guard re-runs are pre-qualification retries and do not count as rally-level strikes.
- **Card-scenario update (derived):** line 60 of `orbit/cards/0006-rally.yaml` moves from `select-subset` to `modify-list` so the card's then-clause matches the canonical label set chosen in D1.

### Open Questions
None blocking.

- Bounded-loop soft cap: if the modify loop becomes a ping-pong on thin-card additions, a soft cap of ~5 iterations before the lead offers `/orb:card` for the offending cards would match rally's other single-strike patterns. Future implementation detail — out of scope for this card-level scenario; belongs in the implementation spec if and when the pattern shows up in practice.
