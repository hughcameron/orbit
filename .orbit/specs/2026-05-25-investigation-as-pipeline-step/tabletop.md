# Tabletop — Investigation as a pipeline-stage step

**Date:** 2026-05-25
**Facilitator + domain expert:** Hugh Cameron
**Scribe + driver:** Claude (claude-opus-4-7)
**Cards in scope:** 0025-codebase-mastery
**Methodology:** Card 0019 — 10-question methodology; choice 0017 — output is contract, not solution
**Output spec:** `.orbit/specs/2026-05-25-investigation-as-pipeline-step/spec.yaml`
**Predecessor specs:** `.orbit/specs/2026-05-17-code-investigate-skill/spec.yaml` (closed; shipped skill + hook)
**Input memos:**
- `.orbit/memos/2026-05-17-codebase-mastery-audit.md`
- `.orbit/memos/2026-05-25-code-investigate-stage-bound.md`
- `.orbit/memos/2026-05-25-code-investigate-empirical-audit.md`

---

## Goal (Q1, narrowed)

Investigation becomes a structural step within the orbit pipeline — fired at the right stage, at the right depth, for the right tokens — so investigation-before-edit ratio rises measurably above the pre-ship baseline (~47%) and the hook→action conversion rate stops being zero.

Card 0025 reworded inline this session: feature is now "Informed, surgical agents — investigation as a pipeline-stage step"; surgical change is the load-bearing value; token-frugality is demoted to enabler.

## Values (Q2)

**Load-bearing value: Behaviour-changing, not advice-shaped.**

The empirical audit forced this pick: skill + hook + embedded prose are three integration attempts that all failed on the same axis (610 hook fires, 0 conversions, baseline unchanged). The other candidate values fall out of getting this one right:

- *Surgical change* is what behaviour-change produces (the **so_that**).
- *Token-frugality* is the constraint that keeps behaviour-change from being annoying (the **enabler**).
- *Pipeline discoverability* is the mechanism surface — pipeline skills are where behaviour-change has reach.
- *Stage-right depth* is the calibration — wrong-depth behaviour-change burns the budget that earned the acceptance.

## Trade-offs (Q3)

The simplest cut: **pipeline-as-actuator + derived-scope + logged-bypass.** Three commitments, mechanism-agnostic.

| # | Trade-off | Classification |
|---|---|---|
| 1 | Agent autonomy at "should I investigate?" | acceptable — autonomy was producing zero investigation; trading it is the point |
| 2 | Token spend on stage-entry investigation | expensive-but-worth-it — surgical edits buy back the upfront burn; must be scoped |
| 3 | Friction at stage entry | acceptable IF small — depends on mechanism |
| 4 | Pipeline-skill coupling to the investigation surface | acceptable — orbit substrate is heavily wired already; in-grain |
| 5 | Bypass cost for legitimate one-line / typo / hot-patch edits | expensive-but-worth-it — designed bypass with logged reason is canonical |
| 6 | First-time burn on uninvestigated codebases | acceptable — broad-mode investigation IS the onboarding |
| 7 | Standalone `/orb:code-investigate` slash command becoming vestigial under some mechanisms | expensive-but-worth-it — cleanup bounded; patterns survive even if invocability changes |

## Halt conditions (Q4)

**H1 — Bypass dynamics fail.** *(ref 2026-05-25 empirical audit, 610/0 conversion rate)*
> Trigger: post-ship bypass rate dominates stage-entry investigation calls, OR the bypass log has no consumer at 4 weeks (no finding family fires on it, no skill reads it).
> Revert: tighten bypass cost (longer required reason, AUQ confirmation) OR remove logged-bypass for the affected stages.

**H2 — Scope derivation produces wrong or empty scope.** *(ref 2026-05-25 empirical audit's 50% blind-edit rate in private repo)*
> Trigger: post-investigation marker covers files different from what the spec/PR actually touched in a substantial fraction of close cycles, OR tabletop stage produces empty derived-scope payloads materially often.
> Revert: per-stage derivation re-cut, OR fall back to agent-typed scope with sensible defaults.

**H3 — Token spend compounds unsustainably.**
> Trigger: mean session token spend rises materially above pre-ship baseline within 1 week of ship, attributable to stage-entry investigation calls.
> Revert: gate behind heavier triggers (first-entry-per-session, Edit-bound stages only) OR reduce broad-mode default depth.

Engineering-hygiene items moved to Implementation Notes — centralised entry point, per-stage scope-derivation heuristics, inlined investigation results back into agent context, session-bridging marker persistence.

## Lateral approaches (Q5)

Five laterals named; spec picks among A/B/C, with D and E as fallback paths.

- **A — Orchestrate.** Pipeline skill's prose step *N* literally invokes `/orb:code-investigate <mode> --scope <derived>`. Agent doesn't decide. *Held in reserve as the leading candidate.*
- **B — Marker-gate.** Pipeline skill's pre-flight reads `.orbit/.code-investigate-recent`; AUQ-blocks if absent for relevant files. *Held in reserve — risk of waive-defaulting given 610-zero conversion track record on the existing nag.*
- **C — Compose.** Fold investigation discipline directly into pipeline skills' prose + behaviour. No standalone invocable surface. Hook tracks "did adequate investigation occur" rather than "did /orb:code-investigate fire". *Held in reserve — closest to what agents already do; drops indirection layer.*
- **D — Defer entirely** (wait for ac-07's 2026-06-14 fire and a consumer-repo dataset). *Rejected — in-repo signal across 35 sessions in 5 repos is already conclusive; ac-07 will refine, not reverse.*
- **E — Contained scope** (pick just `/orb:implement` and prove the pattern before fanning out). *Held in reserve as a phased-rollout option for the spec.*

## Success criteria (Q6)

Binary, measurable, each traces to a Q2 value or Q3 trade-off:

1. **Investigation-before-edit ratio.** Sustained lift above the pre-ship baseline (47.1%, brightfield), measured over a 2-week post-ship window across orbit + at least 2 consumer repos. *Traces to Q2 (behaviour-changing).*
2. **Hook→action conversion.** Conversion rate moves above zero materially. The current 610/0 stops being zero. *Traces to Q2.*
3. **Memory-write closing instruction fires.** At least one `code-investigate`-labelled memory captured *from* an investigation (not *about* the skill) per active development week. *Traces to Q2 + closing instruction in existing skill prose.*
4. **Token spend stability.** Mean session token spend stays within a reasonable band of pre-ship baseline (no compounding). *Traces to Q3 #2.*

## Escalation triggers (Q7)

Condition + state snapshot + proposed action.

- **E1 — Empty derived-scope cascade.** Trigger: chosen mechanism's "investigation step" produces empty scope across >5 consecutive pipeline-skill entries. Surface: stage type, derived-scope payload, marker-after content. Action: AUQ — (a) fall back to agent-typed scope with broad-mode default, (b) halt and re-cut derivation logic.
- **E2 — Over-aggressive gating.** Trigger: AC-traversal during implement halts on missing investigation marker more than 3 times in one cycle. Surface: marker state, file paths, current AC. Action: AUQ — (a) lower gate to soft warning, (b) tighten scope-derivation to fewer files.
- **E3 — Bypass storm at ship.** Trigger: ship-day or +1 day shows bypass rate already >50% of stage entries. Surface: rate, sample bypass reasons. Action: AUQ — (a) raise bypass cost, (b) halt deployment.

## Kill conditions (Q10)

Per load-bearing claim, with named pivot.

- **K1 — Behaviour-changing claim.** If after ship + 2w the investigation-before-edit ratio hasn't moved measurably, the chosen mechanism is dead. *Pivot: try a different mechanism from Q5 laterals A/B/C; or conclude that the integration shape itself is wrong and revisit the cut.*
- **K2 — Pipeline-as-actuator claim.** If pipeline-skill entries don't reach often enough to carry the discipline (e.g. agents stop entering /orb:implement after the change), the actuator surface is dead. *Pivot: move discipline to a different surface — SessionStart with stage-bound suppression, or smarter PreToolUse triggers.*
- **K3 — Derived-scope claim.** If derivation produces wrong/empty scope in a majority of cases, scope-derivation is dead. *Pivot: fall back to agent-typed scope with stage-appropriate defaults, OR re-shape stages to carry scope explicitly.*

## Adjacent code (Q8) — layer-level

- `plugins/orb/skills/{tabletop,implement,review-pr,researcher,drive,rally}/SKILL.md` — pipeline skills gain integration points; current state: implement / review-pr / researcher have 1 imperative mention each; tabletop / drive / rally have 0.
- `plugins/orb/skills/code-investigate/SKILL.md` — role changes depending on mechanism pick (orchestration target / marker-gate dependency / pattern library).
- `plugins/orb/hooks/code-investigate-nudge.sh` — may be modified or retired.
- `plugins/orb/scripts/code-investigate-mark.sh` — marker writer; likely unchanged.
- `orbit-state/crates/cli/src/main.rs` — may add an `orbit code investigate` verb if EH1 (centralised entry point) routes through Rust.
- Test surface in `orbit-state/crates/core/src/` and CLI parity tests.

File-level routing belongs to Implementation Notes (below).

## Budget (Q9)

Conservative-engineering quote: **3-5 working days** (six SKILL.md edits + mechanism implementation + tests + smoke + audit-instrumentation).
Inflation-guard recut (÷3): **1-2 working days at Claude-execution pace.**

Theme-5a halt: if real burn trends toward the inflated estimate inside the first day, halt and reassess scope/architecture — likely candidate is to drop to lateral E (contained scope, /orb:implement only) and ship that as the first spec, with the others as follow-up specs.

## Hot-wash

**Recurred:**
- "Behaviour-changing not advice-shaped" returned in every Q. Empirical data (610/0 conversion) made it impossible to drift back to a coaching framing — useful guardrail.
- Stage-bound principle: from the first reframe ("code-investigate has a *specific role*") through to Q8's layer enumeration, the same idea kept compressing the option set.

**Surprised:**
- How cleanly the empirical audit collapsed the design space. Standard tabletop walks through Q1-Q5 with multiple plausible cuts; here Q1's narrowing was forced by the data, Q2's load-bearing-value pick was forced by 610/0, Q3's cut emerged in one pass.
- Q1's narrowing surfaced as a card reword (not just a session-scoped narrowing). The wrong-shape goal was *in the card*, not just in the session.

**Friction:**
- I drifted into implementation detail at Q4 (specific thresholds, specific bypass mechanics, EH1-EH4 spelling out particular ACs). User intervention reset me to contract altitude. Pattern for future tabletops: when surfacing halts with measurable triggers, name the *category of measurement* (rate-of-X, presence-of-Y) and let the spec pick the cut-off.

**Meta-patterns:**
- Author reframes during what feels like Q1 are usually Q2 input. "We should reword the goal — it's really about X" is naming the load-bearing value, not narrowing scope. Treat as Q2 closing signal and ask whether to lock it there.
- When card-level prose is wrong-shaped, update the card *during* the tabletop rather than at the end. Avoids the spec inheriting a stale parent.
- Empirical predecessor work (the 2026-05-25 audit) compressed Q1-Q3 into ~30 minutes. Tabletops with strong empirical priors are faster *and* tighter; pattern worth replicating where the data exists.

---

## Implementation Notes

For the implementing agent (engineering hygiene items + file-level routing):

- **EH1 — Centralised entry point.** Whatever the mechanism, pipeline skills should call one verb / orchestration target (candidate: a new `orbit code investigate <scope>` Rust verb that wraps the existing `code-investigate-mark.sh` + tool-routing logic) so refactor surface stays at one location.
- **EH2 — Per-stage scope-derivation heuristics.** `tabletop` Q8 → broad on cluster cards' adjacent code; `implement` pre-flight → narrow on the spec's `adjacent_files` or the AC's named files; `review-pr` → narrow on PR's changed paths; `researcher` → broad on the topic argument.
- **EH3 — Inlined results.** Investigation results surface back into the stage's working context (skill quotes the marker/result at the agent), not just written to marker for the hook to maybe read.
- **EH4 — Session-bridging scope persistence.** Marker re-derives from spec/PR adjacent_files at session start so first-time burn doesn't recur every session.
- **Recommended `ac_type` per AC band:** mechanism wiring → `code`; SKILL.md prose changes → `doc`; success-criteria measurements → `observation` (4-week post-ship window); hook retirement / config flips → `config`.
- **Mechanism pick:** the spec author should make the orchestrate / marker-gate / compose call upfront in the spec's goal, and ideally file a `.orbit/choices/NNNN-*.yaml` MADR that pins the rationale. The audit's `610/0` conversion is the load-bearing evidence; the choice should cite it.
- **Phased rollout (lateral E).** If the spec runs over budget on the full fan-out, drop to `/orb:implement` only as the proving ground; spawn follow-up specs per remaining pipeline skill.
