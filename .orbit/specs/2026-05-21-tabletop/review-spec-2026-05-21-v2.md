# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-tabletop
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | Pass 1 found MEDIUM finding; content signals present (skill deletion, METHOD.md cascade across 3 copies, cross-crate touch surface) | 3 |
| 3 — Adversarial | Pass 2 surfaced an unverifiable post-cascade gate condition with cross-system impact | 1 |

---

## Findings

### [HIGH] ac-11 cascade scope omits the conformance engine (`verbs.rs`) — ac-11(5) is unreachable as written

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-11(5) requires `orbit audit conformance --json` to run clean post-cascade. The conformance engine at `orbit-state/crates/core/src/verbs.rs` hard-codes `/orb:design <numeric-id>` as the `remediation.verb` value emitted for planned-empty-card findings. After the design skill is deleted, the conformance audit will continue to emit `/orb:design N` as the remediation for any planned-empty card the auditor encounters — pointing the agent at a skill that no longer exists. The audit's textual output may report "clean" (no plugin-canonical-file drift), but it will simultaneously be telling agents to invoke a deleted skill. ac-11's scope language ("agent-facing live surface: SKILL.md files, METHOD.md, README") doesn't reach into the crate — yet the crate is what emits the live agent prose the audit surfaces.
**Evidence:** `rg "/orb:design" orbit-state/crates/core/src/verbs.rs` returns five hits, including `verb: format!("/orb:design {numeric_id}")`, the docstring `/// /orb:design <numeric-id>`, the body string `"use orbit memory match before /orb:design"`, and `assert_eq!(f.remediation.verb, "/orb:design 99")` in a test. Deleting `plugins/orb/skills/design/SKILL.md` without updating these leaves the conformance verb broken.
**Recommendation:** Extend ac-11 to add a step (4a): "`orbit-state/crates/core/src/verbs.rs` updated to emit `/orb:tabletop <numeric-id>` in place of `/orb:design <numeric-id>`; associated tests updated; `cargo test -p orbit-state` runs green". Without this, ac-11(5) cannot be honestly closed — the audit's output will still name a dead verb.

### [HIGH] ac-11 cascade scope omits `CLAUDE.md` — live agent-facing prose with `/orb:design` reference

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-11 declares the cascade scope as "SKILL.md files, METHOD.md, README". The project root `CLAUDE.md` (the top-level instructions every Claude Code session reads in this repo) contains the line: ``orbit is a Claude Code plugin that provides specification-driven workflow skills (`/orb:card`, `/orb:distill`, `/orb:design`, `/orb:spec`, `/orb:implement`, `/orb:review-pr`, etc.)``. This is unambiguously live agent-facing prose — it loads into every session here — but it isn't in the cascade scope. A literal-reading implementing agent runs the canonical grep, gets the SKILL.md list, edits those, marks done, leaves CLAUDE.md stale.
**Evidence:** `rg -l "/orb:design" CLAUDE.md` returns the project CLAUDE.md. The canonical grep query named in ac-11(2) is `rg -l "/orb:design" plugins/orb/skills/*/SKILL.md` — narrow to SKILL.md only and will not surface CLAUDE.md.
**Recommendation:** Broaden the canonical grep in ac-11(2) to `rg -l "/orb:design" plugins/orb/skills/*/SKILL.md CLAUDE.md README.md`, or restate the scope as "tracked SKILL.md files plus the project `CLAUDE.md` plus the project `README.md`". The 3-file delta is small; the stale-prose risk in CLAUDE.md compounds across every future session.

### [HIGH] README contains six bare-`/design` references the canonical grep will miss

**Category:** failure-mode
**Pass:** 2
**Description:** ac-11(4) commits to README workflow flowchart and skills table being updated. But the canonical detection in ac-11(2) — `rg "/orb:design"` — misses the README's actual usage pattern. The README writes the skill in shortened form throughout: `/design`, not `/orb:design`. Six such bare references exist in the README; the canonical grep returns zero hits in README, so the implementing agent who relies on the grep alone will mark ac-11 closed with the README still naming `/design` in its workflow narrative.
**Evidence:** `rg "design" README.md` surfaces lines like ``a card ready for `/design`, then `/spec```; ``opens a focused session`` referring to `/design`; ``return to `/design` for the same card``; ``Design["/design"]`` in the mermaid flowchart; the skills table row ``| `/design` | Refine a card into technical decisions ...``. `rg "/orb:design" README.md` returns zero hits — the canonical grep query is blind to the bare form. ac-11(2)'s grep and ac-11(4)'s README requirement don't align.
**Recommendation:** Amend ac-11(4) to name an explicit second grep for the README: `rg "(/orb:)?design" README.md`, or list the README touch surface inline ("workflow flowchart node `Design["/design"]` becomes `Tabletop["/tabletop"]`; skills-table row `/design` becomes `/orb:tabletop`; narrative prose `/design` becomes `/orb:tabletop`"). The current AC asks for a substrate cleanup the canonical grep cannot verify.

### [MEDIUM] Gate flag drift between card scenarios and spec ACs

**Category:** constraint-conflict
**Pass:** 1
**Description:** Card 0019 marks four scenarios as `gate: true` — scenario 1 (goal-scoped, one-to-many), 2 (right questions / methodology), 3 (multi-card), 5 (contract-not-solution). The spec carries 12 ACs but only two are marked `gate: true` (ac-01 — methodology; ac-11 — retirement cascade). The mapping is unclear: scenario 1's "goal-scoped, one-to-many" gate maps to ac-03 (sidecar fan-out shape) and ac-04 (goal-string mode), neither of which is gated. Scenario 5's "contract-not-solution" gate maps to ac-02 (sidecar pattern) which carries the load-bearing contract shape — but ac-02 is not gated either. The retirement cascade is not on the card as a gate scenario; ac-11 carrying `gate: true` is a spec-level promotion not anchored to the card.
**Evidence:** `orbit card show 0019` shows scenarios 1, 2, 3, 5 with `gate: true`. Parser output for the spec: only ac-01 and ac-11 are gate=1. The gate-promotion logic at card→spec time appears to have collapsed four card gates into one spec gate (ac-01) plus added a fresh spec gate (ac-11). Worth noting explicitly so reviewers and the implementing agent know which gates are load-bearing.
**Recommendation:** Either (a) mark ac-02, ac-03 (and optionally ac-08, which carries the methodology's AUQ-prose hybrid for scenario 2) as `gate: true` to track the card's contract-not-solution and multi-card gates onto the spec; or (b) add a one-line note to the spec body explaining why only ac-01 and ac-11 are gated despite four card scenarios being gated (e.g. "ac-01 absorbs scenarios 1, 2, 3, 5 because all four are SKILL.md-prose content"). Pick one and write it inline.

### [MEDIUM] ac-10 probe-runner identity is unspecified — same-context confirmation bias

**Category:** failure-mode
**Pass:** 2
**Description:** ac-10 describes a probe that compares a retroactive baseline (scoring the existing 2026-05-21-session-start-priority-synthesis design output) against a fresh tabletop session against card 0043 or similar. The probe artefact "land[s] in" `.orbit/specs/2026-05-21-tabletop/ambiguity-floor-probe.md`. The AC is silent on who scores the two artefacts. If the implementing agent — the same context that wrote `/orb:tabletop` SKILL.md and ran the fresh tabletop session — also scores both, confirmation bias is baked in. The `/orb:review-spec` skill itself enforces context separation precisely because same-context review is unreliable; the ac-10 probe carries the same structural weakness.
**Evidence:** ac-10 prose says "computed by the prober" and "scored the same way" without naming the prober's session identity. The probe is the gate that lets ac-11 (skill deletion) fire — a load-bearing decision under same-context scoring is exactly the failure mode the rest of orbit guards against.
**Recommendation:** Add one clause to ac-10: "The two scoring passes run in a fresh context-separated agent (mirroring `/orb:review-spec`'s pattern) — neither pass shares context with the agent that implemented `/orb:tabletop` SKILL.md or ran the fresh tabletop session." This costs one extra fork but matches the orbit pattern for load-bearing review decisions.

### [LOW] Pass-3 cascade — ac-10 NO-GO branch sidecar artefact unspecified

**Category:** missing-requirement
**Pass:** 3
**Description:** ac-10's NO-GO branch directs the implementing agent to file `.orbit/memos/<date>-tabletop-nogo.md` and surface a single decision (a/b/c) to the author. The shape of that AUQ — what the agent surfaces, which option is the default, what state snapshot accompanies — is unspecified. The decision is consequential (revert ac-01..ac-09, accept parallel operation indefinitely, or re-design tabletop) but the AC framing leaves the decision-surface unstructured. The author will have to invent the AUQ structure mid-flight, which the spec elsewhere (ac-08) explicitly tries to prevent.
**Evidence:** ac-10 prose names the memo and the three options but doesn't pin the surfacing pattern (AUQ vs prose) or which option is the agent's recommendation.
**Recommendation:** Add one sentence: "The decision is surfaced via AskUserQuestion with the three options as picks, the gap analysis from the memo as the question body, and option (a) re-design as the default." This costs a sentence; it removes a mid-flight UX invention from the NO-GO branch.

---

## Honest Assessment

The methodology core (ac-01..ac-09) is solid — every prior-review finding on those ACs has landed cleanly. The sidecar/closed-mode contract is now reconciled, the AUQ-refusal fallback is pinned, the card-inference algorithm is explicitly left best-effort with the AUQ as the safety valve. The two new MADR choices are framed correctly against choice 0017.

The remaining risk is concentrated in **ac-11 (the retirement cascade)** and its gating dependency on **ac-10**. Three HIGH findings, all in the cascade scope:

1. The conformance engine (`verbs.rs`) emits `/orb:design N` as a remediation verb — the audit will be telling agents to run a deleted skill. ac-11(5)'s "runs clean" is structurally unreachable until verbs.rs ships in the cascade.
2. CLAUDE.md is live agent-facing prose with a stale `/orb:design` reference; ac-11 scope language doesn't reach it.
3. The README uses `/design` (no `/orb:` prefix); the canonical grep query in ac-11(2) is blind to the bare form, so a literal-reading implementing agent will close ac-11 with the README still naming `/design` in six places.

These are mechanical fixes — extend the cascade scope, broaden the grep, add a verbs.rs step — but they're not optional. The cascade is a one-shot deletion: post-deletion, agents that read CLAUDE.md, follow README's narrative, or act on conformance-audit remediations will hit dead references.

The MEDIUM findings (gate-flag drift, probe-runner identity) are about pre-committing decisions the implementing agent would otherwise have to invent. The LOW finding (NO-GO branch UX) is the same shape. None block on their own; together they suggest one more pass to tighten the cascade scope and the probe's structural protection before implementation begins.

Fix the three HIGH findings and approve. The two MEDIUM and one LOW findings are worth landing in the same pass — they're sentence-sized — but the spec is implementable without them.
