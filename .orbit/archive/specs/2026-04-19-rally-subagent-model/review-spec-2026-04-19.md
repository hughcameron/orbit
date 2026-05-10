# Spec Review

**Date:** 2026-04-19
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-19-rally-subagent-model/spec.yaml (v1.0, 15 ACs, 9 constraints, test_prefix `rallysub`)
**Verdict:** REQUEST_CHANGES

---

## Review Depth

```
| Pass                         | Triggered by                                                                   | Findings |
|------------------------------|--------------------------------------------------------------------------------|----------|
| 1 — Structural scan          | always                                                                         | 5        |
| 2 — Assumption & failure     | Pass 1 surfaced HIGH-severity contradiction + MEDIUM test-gaps + content signal| 4        |
| 3 — Adversarial              | Pass 2 revealed structural problem (ac-07 contradicts drive SKILL.md)          | 2        |
```

Content signals triggering Pass 2: cross-skill boundary (rally ↔ drive ↔ review-*), shared config (rally.yaml schema + ontology), amendment of another spec (v2.0's ac-06 / ac-14). Pass 3 triggered by F-01 (structural contradiction between a load-bearing claim in this spec and drive's actual SKILL.md).

---

## Findings

### [HIGH] F-01 — ac-07's "drive-full spawns nested forked Agents for reviews" contradicts drive's current SKILL.md

**Category:** assumption
**Pass:** 1 (surfaced structural), deepened in 2 and 3

**Description:** ac-07 and constraint #2 both assert that drive-full's review-spec and review-pr stages "spawn their own nested forked Agents — this is recursive context separation, not self-review." The spec treats this as an existing property of drive. It is not. The current `plugins/orb/skills/drive/SKILL.md` explicitly *forbids* this pattern at §5 and §7:

- §5 line 110: "Do NOT invoke `/orb:review-spec` as a skill call. Read the SKILL.md file and follow its instructions directly within this session. The review runs inline to keep the drive's single-session model intact."
- §7 line 148: "Do NOT invoke `/orb:review-pr` as a skill call. Read the SKILL.md file and follow its instructions directly within this session. The review runs inline to preserve the full implementation context."
- Critical Rules line 327: "Both reviews run inline. Do not invoke `/orb:review-spec` or `/orb:review-pr` as skill calls..."

So the spec is premised on drive behaviour that does not exist and is actively contradicted. If the plan is to *change drive* so it forks nested reviews, that is a substantial amendment to drive's single-session model — which isn't in scope of this spec, isn't named in exit_conditions, and has no AC guaranteeing it lands. If the plan is *not* to change drive, then ac-07's claim is false and ac-06 (which requires parallel sub-agents to run `/orb:drive <card> full`) will produce sub-agents that run reviews in-context, i.e. self-review — the exact problem F-05 was meant to resolve.

**Evidence:**
- This spec, constraint #2 (line 5): "Drive-full's review-spec and review-pr stages spawn their own nested forked Agents — this is recursive context separation, not self-review."
- This spec, ac-07 (lines 48–51): "Rally SKILL.md §7 Parallel Implementation explicitly states that sub-agents run drive-full, and that drive-full's review-spec and review-pr stages spawn their own nested forked Agents"
- plugins/orb/skills/drive/SKILL.md §5 (line 110), §7 (line 148), Critical Rules (line 327) — all three state the opposite.
- plugins/orb/skills/review-spec/SKILL.md frontmatter says `context: fork` but drive currently overrides this by running the review inline.

**Recommendation:** Pick one of three paths:
- **(a) Extend scope:** Add an AC requiring drive's SKILL.md §5 and §7 to change from "run inline" to "spawn a forked Agent" (and an AC covering the `context: fork` frontmatter contract being honoured). Update drive's exit conditions and test_prefix ontology. This is a drive-level change masquerading as a rally refinement — probably deserves its own card or an explicit sub-spec.
- **(b) Contain scope:** Keep drive's inline review model. Change this spec's parallel model so the *lead* (not the sub-agent running drive-full) owns the post-implementation review. Sub-agents return implementation artefacts + a self-review draft; the lead spawns forked review-pr Agents on each card after sub-agents return. This matches F-05's recommendation (b) in the v2.0 PR review.
- **(c) Accept self-review honestly:** Admit that drive-full inside a sub-agent means the sub-agent reviews its own work, name the honesty cost, and mitigate with the batched-diff review gate at §8 as the true forked-review layer. Less pure but honest about what changes.

Until one of these is chosen, ac-07 is not actionable and ac-15's coherence claim cannot hold.

---

### [HIGH] F-02 — ac-04's verification mechanism is undefined and conflicts with the ambient repo state

**Category:** test-gap / missing-requirement
**Pass:** 1

**Description:** ac-04 says the lead "checks for any files written outside that spec_dir" but never specifies how. The interview's Open Question #1 explicitly lists three options (git status scan, pre/post filetree snapshot, sub-agent self-report) and punts to the spec — and the spec does not resolve it. This is not an implementer-derivable detail because the three options have materially different behaviours:

- **git status scan** will flag any file the author touched in the shell while a sub-agent was running, producing false positives in a live repo.
- **pre/post filetree diff** requires capturing a snapshot before Agent-tool launch; there's no API for this at rally-level and implementing it adds surface area the spec explicitly tries to minimise (principle #4, minimality).
- **sub-agent self-report** is trust + trust; it doesn't actually verify anything the sub-agent wouldn't have written honestly anyway.

The parallel case is worse: sub-agents run in separate git worktrees, so `git status` in the main checkout won't see a sub-agent's worktree writes at all (worktrees share .git, not working tree). ac-04's verification method must also specify whether it's scoped to the main worktree, each sub-agent's worktree, or both.

**Evidence:**
- Spec ac-04 (lines 32–35): "checks for any files written outside that spec_dir" — mechanism unspecified.
- Interview Open Questions (line 79): "This is a spec-level detail, not a discovery decision." The spec failed to close it.
- Constraint #1 says "trust + post-verify — the brief names the spec_dir, the lead verifies on return" — restates the intent but does not pick a verification primitive.

**Recommendation:** Pick the primitive and encode it. Recommend a hybrid: (1) sub-agent brief requires the sub-agent to return a list of files it wrote (self-report); (2) lead cross-checks the claimed files exist at the expected path; (3) lead runs `git status --porcelain` scoped to the spec_dir's parent and flags any unexpected entries. Add this to ac-04's verification, or make it an explicit new AC — either way the "verification pass" contract must name the primitive.

---

### [HIGH] F-03 — ac-01 thin-card guard cannot know "parallel is expected" at proposal time

**Category:** failure-mode
**Pass:** 1, deepened in 2

**Description:** ac-01 says the proposal gate refuses cards with <3 scenarios "AND the disjointness check is expected to allow parallel implementation." But the rally SKILL.md is explicit that disjointness is determined only after designs exist (§2 line 91: "Do not attempt a lightweight heuristic disjointness check — the definitive check happens after designs exist (§6)"; v2.0 constraint #7: "the author's approval at the proposal gate is the only pre-design independence check"). So at proposal time the lead has no basis for predicting parallel eligibility.

This leaves three possible interpretations of ac-01, none of which the spec commits to:
1. **Pessimistic:** the guard fires whenever any card has <3 scenarios, regardless of serial-vs-parallel — in which case ac-02 (serial rallies accept thin cards with warning) is unreachable for any rally mixing thin and thick cards, because you'd refuse at proposal before ever reaching the disjointness check.
2. **Pre-design prediction:** the lead performs some heuristic independence check at proposal (but v2.0 explicitly forbids this).
3. **Late-firing guard:** the "refuse" actually happens at post-design once parallel is confirmed, not at proposal — contradicting ac-01's text.

The interview's Q3 answer says "Refuse at proposal. Earliest possible failure, cleanest mental model" — implying interpretation 1 (pessimistic), but ac-02 then contradicts that.

**Evidence:**
- Spec ac-01 (line 18): "if any candidate card has fewer than 3 scenarios AND the disjointness check is expected to allow parallel implementation"
- Spec ac-02 (line 23): "Serial rallies (those the author pre-declares as serial, or those where the post-design disjointness check forces serial ordering) accept cards..." — introduces "pre-declared serial" which doesn't exist in rally's usage (§1 in SKILL.md: the author picks guided/supervised, not serial/parallel).
- Interview Q3 (line 33): "Refuse at proposal" (earliest-fail framing).
- v2.0 constraint #7: post-design is the only disjointness check.

**Recommendation:** Resolve the timing contradiction. Options:
- **(a) Pessimistic-at-proposal:** Refuse any rally containing a <3-scenario card at proposal. Drop ac-02's carve-out for serial. This is honest — the author can still run thin cards via individual `/orb:drive` guided/supervised.
- **(b) Post-design enforcement:** Move the guard to the consolidated-design-review gate (§6c), run it only when the disjointness check proposes parallel, and park/refuse the thin card there. Rewrite ac-01 as a post-design guard. ac-02's "warning on serial" still works. This is more surgical and matches v2.0's single-disjointness-check principle.
- **(c) Add an explicit "pre-declared serial" mode:** add a rally invocation flag (e.g. `/orb:rally <goal> [guided|supervised] [--serial]`) and rewrite ac-02 around it. This is new surface area the spec has not acknowledged.

Either pick (a) or (b). The current text blends them and no implementer can derive which wins.

---

### [MEDIUM] F-04 — Drive escalation conflates three different triggers; parked_constraint loses the distinction

**Category:** missing-requirement
**Pass:** 2

**Description:** ac-14 says "When a parallel sub-agent's drive-full exhausts its 3-iteration budget, the sub-agent returns an escalation summary. The lead treats this as a NO-GO park..." But drive's §9 defines *three* escalation triggers, not one:
1. Budget exhaustion (mechanical).
2. Recurring failure mode (semantic).
3. Contradicted hypothesis (semantic — goal unreachable).
4. Diminishing signal (semantic).

(4 triggers, actually.) The spec only names #1. Does #2/#3/#4 also get parked as a single strike? Probably yes given the single-strike absorb argument — but if the sub-agent escalates because of *contradicted hypothesis* ("the underlying goal is unreachable"), parking it with the one-line reason and moving on loses signal: this card doesn't need reruning with a constraint, it needs human rethinking at the card level. The spec should either (a) distinguish park reasons in `parked_constraint` or (b) add a new status like `escalated` alongside `parked` that surfaces "this card needs re-cards, not just re-drive."

Also: constraint #8 says "Drive-full's internal 3-iteration budget exhaustion... is recorded as a park." Same narrow framing. The v2.0 ontology's `parked_constraint` is typed as string — a richer type (`parked_reason: budget | recurring | contradicted | diminishing | external`) might be worth it for completion-summary usefulness.

**Evidence:**
- Spec ac-14 (line 88) and constraint #8 (line 11) — both only name budget exhaustion.
- drive/SKILL.md §9 (lines 205–238) — four escalation paths.
- Interview Open Question #3 (line 81) — flagged budget path; didn't raise the semantic paths.

**Recommendation:** Extend ac-14 to cover all four triggers explicitly and decide whether the lead records a structured `parked_reason` or just concatenates drive's escalation reason into `parked_constraint`. At minimum, state "all drive-full escalation paths (budget + semantic) are absorbed as a single-strike park; the park record includes drive's escalation reason type verbatim."

---

### [MEDIUM] F-05 — ac-15 coherence gate lacks a verifier and is likely to regress

**Category:** test-gap
**Pass:** 1

**Description:** ac-15 is a gate AC: "After this spec's changes land, the rally SKILL.md and .orbit/specs/2026-04-19-rally/spec.yaml are internally consistent." The verification method is "cross-read... confirm each claim about sub-agent orchestration has a matching AC, and that v2.0's ac-06 and ac-14 have been amended per ac-05 and ac-07 here." This is a manual cross-read with no machine check, no reviewer designated, and no trigger other than "after changes land."

Practical problems:
1. No process designates *who* runs it or *when*. If implement-stage runs inline (as drive does) the implementer self-verifies, which defeats the forked-review intent.
2. The check-list ("each claim has a matching AC") requires extracting claims from SKILL.md prose — same extraction problem as v2.0's F-04 on disjointness.
3. Rally SKILL.md will drift as the skill evolves; the coherence gate has no re-verification cadence.

**Evidence:**
- Spec ac-15 (lines 93–96) — verification is prose cross-read only.
- No exit condition or metadata hook to re-run the gate after future changes.

**Recommendation:** Either (a) downgrade ac-15 to an implementation checklist item in exit_conditions (honest — this is a one-shot check, not a standing AC), or (b) upgrade it by adding a mechanical check: e.g. a short script or keyword-scan recipe that greps SKILL.md for the forbidden phrases ("tools allow-list", "drive's stage logic inline" in the parallel section) and asserts they are absent. Option (b) is more defensible and fits the `keyword-scan` skill that already exists in this plugin.

---

### [MEDIUM] F-06 — ac-12 "warning" has no surfacing channel defined

**Category:** missing-requirement
**Pass:** 2

**Description:** ac-12 distinguishes "missing field on implementing card = validation error" from "worktree directory gone = recoverable warning." The warning path is important but the spec doesn't say where the warning lands:
- rally.yaml validation runs at every read (v2.0 §12). If the warning prints to stdout every read, the lead's terminal output floods.
- If the warning is squashed, the lead loses the signal that something is structurally wrong (the worktree the lead expected is gone).
- If the warning lands in a log only, the author won't see it during a live rally.

Interview didn't cover this. The spec should name a channel.

**Evidence:**
- Spec ac-12 (lines 75–78) — names the warning tier but not its surface.
- Rally SKILL.md §12 Validation (lines 394–402) — validation path is fail-fast; no "warning" tier exists yet.

**Recommendation:** Specify: "the warning is emitted once, to the lead's user-visible context at rally resumption or the next rally.yaml-writing transition; subsequent reads suppress the warning if the worktree is still missing." Or equivalently: a `worktree_missing` flag is set on the card in rally.yaml, inspected at resumption.

---

### [MEDIUM] F-07 — ac-11 "null worktree for serial cards" loses information the lead needs

**Category:** missing-requirement
**Pass:** 2

**Description:** ac-11 says `cards[].worktree` is null for serial cards (and pre-launch parallel cards). But serial cards still execute *somewhere* — the main checkout in the rally lead's session. If the lead session dies during serial implementation and a new session resumes, the lead needs to know which branch/checkout to operate on. Currently:
- `cards[].branch` exists (e.g. `rally/<slug>`) — fine for branch name.
- The lead is expected to operate in the main worktree — unstated.

"null worktree" conflates two different "nulls":
- **serial, main checkout:** worktree is the main repo root (not null in a meaningful sense).
- **parallel, pre-launch:** worktree is genuinely not-yet-created.

These should surface differently. When the lead resumes a serial rally, if worktree is null it can't distinguish "this card runs in main" from "we haven't launched this card yet." It probably doesn't need to because serial implies main-checkout, but encoding convention this way is exactly the kind of implicit contract the spec's honesty principle warns against.

**Evidence:**
- Spec ac-11 (lines 71–73) — "null for serial cards or before launch."
- Principle #1 (lines 107–108) — honesty requires naming things as they are.

**Recommendation:** Either use an explicit sentinel (`"main"` or the repo root path for serial cards; `null` only for pre-launch), or split into two fields (`worktree: string|null` + `worktree_state: "main" | "launched" | "pending" | "missing"`). Small ontology, meaningful gain.

---

### [MEDIUM] F-08 — Honesty principle at 0.35 weight is declarative, not measurable

**Category:** test-gap
**Pass:** 2

**Description:** Evaluation principles exist to let post-implementation review rank against something. The honesty principle — "describe what Claude Code actually provides, not what an ideal architecture would provide" — is aspirational and aligned with the spec's intent, but no AC provides a measurement. Contrast with minimality (principle #4, weight 0.20) which has crisp exit conditions ("extends the ontology by one field and amends two ACs") the reviewer can tick off.

Without measurement, the 0.35-weighted principle either (a) collapses into reviewer judgement at review-pr time (so the weight is just rhetoric) or (b) gets unevenly applied across future rally changes. ac-03, ac-05, ac-07 are the closest to measurable honesty checks — they require specific phrases to appear or disappear in SKILL.md — but they don't generalise.

**Evidence:**
- Spec principle #1 (lines 107–108), weight 0.35.
- No AC explicitly verifies "a claim matches an actual Claude Code primitive." ac-03 checks the phrase "tools allow-list" is absent; ac-07 checks "recursive context separation" is present. Both are keyword checks, not honesty checks.

**Recommendation:** Either downweight the honesty principle to signal "this informed design, not verification," OR add a concrete honesty AC such as: "Every mechanism SKILL.md §4a, §7, §11 claims to enforce is either (a) backed by a named Claude Code primitive cited inline, or (b) explicitly marked as a convention/trust + post-verify pattern." That's auditable and fits the keyword-scan pattern.

---

### [LOW] F-09 — v2.0's constraints around the "tools allow-list" may need amendment too

**Category:** missing-requirement
**Pass:** 2

**Description:** Constraint #9 of this spec says "The earlier spec's ACs... remain in force where not directly amended here." Exit conditions name ac-06 and ac-14 of v2.0 for amendment. But v2.0's `review_changes` metadata (.orbit/specs/2026-04-19-rally/spec.yaml line 218) states:
> "HIGH: PreToolUse hook resolved — tools allow-list for design sub-agents, worktree isolation for parallel implementation (ac-06, ac-14)"

This sentence in v2.0's metadata asserts the aspirational mechanism that this spec is specifically repudiating. If left unamended, a cold reader of v2.0 still sees the tools allow-list claimed as a resolution. This isn't an AC, it's metadata — but it's part of the coherence ac-15 claims.

**Evidence:**
- This spec constraint #9 and exit conditions name ac-06 and ac-14 only.
- v2.0 `review_changes` text (line 218) independently asserts tools allow-list as the resolution mechanism.

**Recommendation:** Add an exit condition: "v2.0's metadata.review_changes line on the tools allow-list is updated to reflect trust + post-verify." Or accept the drift and note in ac-15's verification that metadata is out of scope. Either is fine, but silence causes ac-15 to partially fail on day one.

---

### [LOW] F-10 — ac-04 "re-briefed once" is undefined in context of rally's single-strike policy

**Category:** missing-requirement
**Pass:** 2

**Description:** ac-04 says a design sub-agent that writes outside spec_dir is "re-briefed once; on second failure the card is parked." But constraint #6 of v2.0 (carried forward by this spec's constraint #9) is "Single-strike NO-GO — a card that fails any review is parked immediately, no iteration retries within the rally." These are compatible only if "re-brief" is defined as a pre-qualification retry (before counting as a strike), not as a NO-GO retry. The spec implies this but doesn't say so.

Also: what does "re-brief" mean operationally? Re-launch a sub-agent with the same brief? Same brief with an added warning? A stricter variant? Implementation-derivable but worth a line.

**Evidence:**
- Spec ac-04 (line 34) — "re-briefed once; on second failure the card is parked."
- v2.0 constraint #6 — single-strike NO-GO.

**Recommendation:** Add to ac-04: "Re-brief is a pre-qualification retry, not a drive iteration, and does not consume any single-strike budget. The re-brief adds an explicit warning: 'your previous return wrote outside <spec_dir>; do not write outside <spec_dir>.' On second violation, the card is parked as a single strike with parked_constraint 'sub-agent violated path discipline.'"

---

### [LOW] F-11 — ac-06 "uses the already-produced interview.md at <spec_dir>/interview.md" requires strong assumption about worktree ↔ spec_dir mapping

**Category:** assumption
**Pass:** 3

**Description:** ac-06 tells the sub-agent to run drive-full using the interview.md. Drive-full then re-enters design (§3 of drive). But drive-full is supposed to produce its own interview.md during design (drive §3 writes interview.md). If the sub-agent's worktree at `../<repo>-rally-<slug>/specs/.../` already contains an interview.md (authored by the rally lead's design phase), what does drive-full do?

- It might see interview.md exists and resume at §4 (spec) — drive's resumption logic (§11) does file-presence override. This is probably the intent.
- Or it might rewrite interview.md in full mode, destroying the rally-approved design decisions.
- Or it might complain.

The spec assumes the former without asserting it. And worktree/spec_dir layout interaction matters: the interview.md lives at `<main checkout>/specs/<spec_dir>/interview.md`; the worktree at `../<repo>-rally-<slug>/` — will drive-full in the worktree see a `.orbit/specs/<spec_dir>/interview.md` directory *inside the worktree's working tree*, or does the lead need to copy it in?

**Evidence:**
- ac-06 (lines 44–46).
- drive/SKILL.md §3 (writes interview.md) and §11 (resumption via file presence).
- Rally v2.0 §7 Parallel Implementation uses worktrees but doesn't address .orbit/specs/ directory provisioning.

**Recommendation:** Add an AC or amend ac-06: "Before launching the sub-agent, the lead ensures the card's spec_dir (including the approved interview.md) exists inside the sub-agent's worktree — either by committing interview.md to the card's branch before worktree creation, or by copying interview.md into the worktree post-creation. The sub-agent's drive-full detects interview.md and resumes at spec per drive §11, skipping the design stage."

---

## Honest Assessment

This spec is trying to rescue two honest-but-hard findings from the v2.0 PR review, and the authoring discipline shows — decisions register entries are pre-allocated, principles are weighted, and the amendment posture is clear. But the most load-bearing new claim — that drive-full spawns nested forked Agents for its reviews (F-01) — is simply not how drive works today, and this spec doesn't bring drive along. Without either extending scope to amend drive's review execution model or switching the parallel implementation to a lead-owned forked-review-per-card pattern, ac-06/ac-07/ac-15 all fail the cold read. F-02 (ac-04 verification mechanism unspecified) and F-03 (ac-01 timing paradox) are second-tier but real — each blocks a fresh implementer.

Biggest risk: the spec is written as if drive-full is already compatible with sub-agent execution, when it is in fact designed around interactive gates AND inline reviews. Resolving F-01 likely means either amending drive or redesigning parallel implementation, both bigger moves than the "refinement" framing suggests. Once F-01 is resolved honestly, the rest of the findings are tightening work rather than re-think.
