# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-tabletop
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals present (skill deletion, METHOD.md three-copy cascade, conformance engine touch); v2 prior-review HIGHs needed re-verification | 4 |
| 3 — Adversarial | Pass 2 surfaced an unverifiable baseline artefact AND a formula-direction bug that interact under the same gate (ac-10) | 1 (amplifies Pass 2 HIGH) |

---

## Findings

### [HIGH] ac-10 baseline interview artefact does not exist on disk

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-10 names the baseline as "the existing 2026-05-21-session-start-priority-synthesis interview + spec" scored using the `/orb:spec` §2 formula. The closed spec is on disk, but the sidecar folder contains only `drive.yaml`, `notes.jsonl`, `review-pr-2026-05-21.md`, `review-spec-2026-05-21.md`, `review-spec-2026-05-21-v2.md`, `spec.yaml` — no `interview.md`, no `design-note.md`. The probe's baseline depends on an artefact that was not persisted (likely because card 0043's spec ran via `/orb:drive` which doesn't always emit an interview sidecar). A context-separated agent told to score "interview + spec" will either fabricate a retroactive interview reading, score the spec alone (changing the formula's input set), or treat `notes.jsonl` (the drive's working scratch) as a stand-in. Three different agents will produce three different baselines, and the GO/NO-GO gate that controls ac-11's irreversible cascade is sitting on top of that variance.
**Evidence:** `find .orbit/specs/2026-05-21-session-start-priority-synthesis -type f` returns no `interview.md` and no `design-note.md`. ac-10 prose: "the baseline — a retroactive ambiguity score computed against the existing 2026-05-21-session-start-priority-synthesis interview + spec".
**Recommendation:** Pick one of (a), (b), or (c) and write it inline in ac-10:
(a) **Score against the spec only** — rewrite the baseline clause as "computed against `.orbit/specs/2026-05-21-session-start-priority-synthesis/spec.yaml`" and accept that the tabletop session's score must be computed against `spec.yaml` of its output spec to be comparable. This is the lightest fix.
(b) **Pick a different baseline spec** — name a spec that does have a persisted interview, e.g. `2026-05-19-workflow-conformance` if it carries one. Requires checking which closed specs in `.orbit/specs/` have `interview.md` on disk.
(c) **Reconstruct the interview** — direct the implementing agent to write a retroactive interview sidecar from `notes.jsonl` + the card scenarios before scoring. Highest cost, restores the artefact, but the reconstructed interview is the prober's invention.

The current AC asks the prober to read an artefact that doesn't exist; pick a recoverable form before the probe can claim parity.

### [MEDIUM] ac-10 formula written as clarity, but the GO test compares as ambiguity

**Category:** constraint-conflict
**Pass:** 2
**Description:** ac-10 names the formula as `goal * 0.4 + constraints * 0.3 + criteria * 0.3`. The actual `/orb:spec` SKILL.md §2 formula is `ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)`. ac-10 dropped the `1 -` prefix. With the formula as written, the metric is **clarity** (higher = better); with the `1 -` prefix, the metric is **ambiguity** (lower = better). ac-10's GO test is "tabletop score ≤ baseline score is GO" — that test is correct against ambiguity (lower is better) and **backwards against clarity** (lower clarity would mean tabletop is worse, yet ac-10 treats it as GO). An implementing agent who reads ac-10 literally — sums the three products without the `1 -` prefix — will compute clarity and then apply the ambiguity-direction test, flipping the GO/NO-GO call. The skill name in the spec ("ambiguity-floor probe") matches the intent; the formula prose contradicts it.
**Evidence:** `rg -n "ambiguity = 1" plugins/orb/skills/spec/SKILL.md` returns `36:** ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)`. ac-10 prose: "the `/orb:spec` §2 formula (`goal * 0.4 + constraints * 0.3 + criteria * 0.3`)" — missing `1 -`. ac-10 prose: "Parity-or-better (tabletop score ≤ baseline score) is GO" — direction-correct only for ambiguity.
**Recommendation:** Restate the formula in ac-10 verbatim from `/orb:spec` SKILL.md §2 as `ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)`. Either include the `1 -` prefix and keep the "tabletop ≤ baseline = GO" rule, or rephrase the formula as clarity (drop the `1 -`) and flip the rule to "tabletop ≥ baseline = GO". Pick one and ship the matching pair — the spec gate that controls ac-11 must not be ambiguous about its direction-of-improvement.

### [MEDIUM] CLAUDE.md narrative pipeline phrase is stale post-cascade and not caught by ac-11's grep

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-11(2)'s canonical detection is `rg -l "/orb:design" plugins/orb/skills/*/SKILL.md CLAUDE.md`. That catches CLAUDE.md line 19 (`/orb:design` in the skills list). It does **not** catch CLAUDE.md line 3: `Sessions here are about workflow refinement — improving how orbit guides the card → design → spec → implement → review pipeline.` After the cascade, that pipeline phrase reads stale — the live pipeline is `card → tabletop → spec → implement → review`, matching the METHOD.md update in ac-11(3). The README's narrative gets a parallel sweep via the broader `rg "(/orb:)?design" README.md`, but CLAUDE.md does not. A literal-reading implementing agent runs the canonical grep, edits CLAUDE.md line 19, marks ac-11 closed, leaves line 3's pipeline phrase stale.
**Evidence:** `rg "design" CLAUDE.md` returns two hits — line 19 (`/orb:design` form, caught by canonical grep) and line 3 (`card → design → spec` narrative phrase, missed). METHOD.md's pipeline-diagram update is named in ac-11(3) but CLAUDE.md's narrative pipeline phrase is not.
**Recommendation:** Either (a) broaden the CLAUDE.md grep in ac-11(2) to `rg "(/orb:)?design" CLAUDE.md`, with a one-line note that intentional uses (e.g. quoting the historical v1 design failure) stay; or (b) add an explicit step under ac-11 naming CLAUDE.md's pipeline phrase: "CLAUDE.md line 3's `card → design → spec → ...` pipeline phrase becomes `card → tabletop → spec → ...` to match METHOD.md". Either suffices; the failure mode is one specific phrase the canonical grep is blind to.

### [LOW] ac-12 ordering vs ac-10 GO/NO-GO is unspecified

**Category:** failure-mode
**Pass:** 2
**Description:** ac-12 commits to writing two MADR choice files: `tabletop-replaces-design` (status accepted) and `tabletop-contract-sidecar` (status accepted). Neither has explicit ordering relative to ac-10. If the implementing agent writes ac-12 before ac-10 runs, `tabletop-replaces-design` is marked accepted before the probe gates the retirement — that's an inverted dependency (the choice claims the design skill is retired, but ac-11 might not fire). If ac-10 returns NO-GO and option (c) is picked (revert ac-01..ac-09), the two accepted choices are now describing a reverted feature — the implementing agent has to mark them superseded or delete them, which ac-12 doesn't direct.
**Evidence:** ac-12 names "next free ids drawn at implement time" but no ordering against ac-10's GO/NO-GO outcome. ac-10's NO-GO option (c) names "revert ac-01..ac-09 and revisit card 0019" — silent on ac-12 cleanup.
**Recommendation:** Add one clause to ac-12: "ac-12 lands AFTER ac-10's GO/NO-GO decision. On GO, both choices write with status accepted (current shape). On NO-GO, the `tabletop-replaces-design` choice is either deferred (not written until a re-design returns GO) or written with status proposed and a NO-GO summary in its consequences section." This pins the dependency and removes the dangling-choice failure mode on NO-GO.

### [LOW] Cascade-stage atomicity unstated

**Category:** failure-mode
**Pass:** 3
**Description:** ac-11 is a six-step cascade across skill folder deletion, three METHOD.md copies, verbs.rs + tests, and README. If `cargo test -p orbit-state` fails mid-cascade (step 4), partial state is on disk: design skill possibly deleted, METHOD.md half-updated, verbs.rs half-rewritten. There is no "single commit / single PR" atomicity rule in ac-11 — implementing-agent discipline plus drive's commit pattern handle this in practice, but the AC framing doesn't pin it. Recovery requires `git reset --hard <pre-cascade>` or selective revert, both of which depend on the implementing agent having staged the pre-cascade state cleanly.
**Evidence:** ac-11 lists six steps without naming a single-commit boundary. The skill deletion (step 1) is irreversible from the working tree without git history.
**Recommendation:** Add one line to ac-11: "The cascade lands as a single commit or a single PR — all six steps stage together; if any fails (notably `cargo test -p orbit-state` in step 4), the implementing agent reverts the staged changes before retrying." Lightweight, removes the partial-state failure mode.

---

## Honest Assessment

The v2 review's three HIGH findings have all landed cleanly — the conformance engine (`verbs.rs`) is now in ac-11(4) with explicit format-string, docstring, body-string, and test-assertion targets; CLAUDE.md is in the canonical grep; the README detection broadened to `rg "(/orb:)?design"`. The methodology core (ac-01..ac-09) is well-shaped; the four gate ACs all pass the deterministic Pass-1 check. The two new choice files cite 0017 correctly as the load-bearing antecedent.

The remaining risk is concentrated, again, in **ac-10** — this time on two interacting issues:

1. The baseline artefact ac-10 names (the session-start-priority-synthesis interview) does not exist on disk. The closed spec was driven without a persisted interview sidecar. A context-separated prober told to score "interview + spec" will improvise. Three improvisations will produce three baselines.
2. The formula ac-10 quotes is the clarity form (no `1 -` prefix); the GO test is the ambiguity-direction rule (lower is better). The pair is internally inconsistent. An agent that reads the formula literally will sum clarity and then compare it as ambiguity, flipping GO/NO-GO.

These two together amplify each other: a fabricated baseline scored under a directionally-confused formula produces a GO/NO-GO call that has no relationship to the actual quality of `/orb:tabletop`. The cascade in ac-11 is one-shot — skill deletion, METHOD.md across three copies, conformance engine edits, README updates — so a wrong GO is a one-shot wrong cascade.

Fix the baseline (pick form (a) — score against `spec.yaml` only) and restate the formula verbatim from `/orb:spec` §2 (include `1 -`, keep "tabletop ≤ baseline = GO"). Both are sentence-sized edits. The MEDIUM CLAUDE.md narrative phrase and the two LOW findings (ac-12 ordering, cascade atomicity) are worth landing in the same pass — they're all sentence-sized — but the spec is implementable without them once the ac-10 pair is fixed.

The structural pattern this review keeps surfacing is the same one v2 found: ac-10/ac-11 are the load-bearing pair, and small ambiguities in either turn into one-shot cascade risks. The methodology halves of the spec are sound; the retirement gate needs one more pass to harden.
