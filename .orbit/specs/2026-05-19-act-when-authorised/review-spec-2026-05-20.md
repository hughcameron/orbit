# Spec Review

**Date:** 2026-05-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-19-act-when-authorised
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (hook infrastructure, cross-system boundary into Claude Code's `PreToolUse` surface) + Pass 1 testability concerns | 3 |
| 3 — Adversarial | not triggered | — |

## Findings

### [MEDIUM] ACs are rule-restatements, not verification statements
**Category:** test-gap
**Pass:** 1
**Description:** All five ACs read as restatements of the rule the spec wants to instil ("severity is reviewer-language…", "memory plus contract is sufficient authorisation…") rather than as verification statements that name *what artefact* or *what observable behaviour* closes them. The approved design (`decisions.md`, `interview.md`) pins each decision to concrete artefacts — D1 to `plugins/orb/hooks/three-question-test.sh` + `plugin.json` registration + a `drive/SKILL.md` prose section; D3 to a paragraph at `drive/SKILL.md` §1.6; D5 to paragraphs at `.orbit/STYLE.md` and `.orbit/cards/0026-executive-communication.yaml` — but the ACs themselves do not reference those artefacts. At `/orb:review-pr` time the reviewer will have to infer the verification surface from the design rather than read it off the AC.
**Evidence:** spec.yaml lines 8–27; contrast with decisions.md "Files this spec is likely to touch" (lines 162–172) and interview.md "Disjointness map" (lines 80–97), which both enumerate concrete files. None of those file paths appear in spec.yaml's AC text.
**Recommendation:** Tighten each AC's description so it names the verification surface explicitly. Suggested rewrites:
- **ac-01** (gate, currently rule-shape): *"A PreToolUse hook at `plugins/orb/hooks/three-question-test.sh`, registered in `plugins/orb/.claude-plugin/plugin.json` against `AskUserQuestion`, prints the three substrate-typed questions to stderr when the calling agent is inside drive/rally autonomy; under `ORBIT_NONINTERACTIVE=1` it exits non-zero to suppress the halt. `plugins/orb/skills/drive/SKILL.md` carries a prose pointer to the same test."* — covers D1 + D2 wording. (If D1 pre-flight had failed and the CLI-verb fallback applied, this would point at `orbit autonomy authorised?` instead; the brief confirms it has not.)
- **ac-02**: *"`plugins/orb/skills/drive/SKILL.md` §1.6 contains a clarifying paragraph stating severity is reviewer-language and does not change autonomy routing; the D1 hook reinforces the same rule when it fires mid-cycle under guided/full autonomy with a non-APPROVE verdict."* — covers D3.
- **ac-03**: *"`.orbit/STYLE.md` and `.orbit/cards/0026-executive-communication.yaml` each carry a paragraph distinguishing the closing-recommendation frame from the in-flight imperative-single-action form, with a back-reference to the three-question test."* — covers D5.
- **ac-04**: *"`plugins/orb/skills/drive/SKILL.md` documents that pre-commit halts named in spec text apply only to the stage in which they were registered; the D1 hook reads the current pipeline stage from `drive.yaml` and treats stage-cross widening as a violation of the authorisation question."* — covers D4.
- **ac-05**: *"The D1 hook's question 3 (`Does the contract authorise me?`) names `drive.yaml.autonomy`, memory `mid-session-autonomy-contract-default-to-action-halt`, and the spec's `halt-conditions` as the three substrate sources; presence of any one is treated as load-bearing authorisation."* — covers D2's substrate-typed phrasing and pins ac-05's "memory + contract" claim to a concrete check.

### [MEDIUM] `ac_type` field missing — defaults to `code` for all five
**Category:** missing-requirement
**Pass:** 1
**Description:** Per the METHOD.md `ac_type` taxonomy, ACs that close on prose edits (CLAUDE.md, card text, MADR, skill docs) are `doc`-typed and ACs that close on config or external-system-state changes verifiable by grep / file inspection are `config`. Several ACs here close on `drive/SKILL.md` / `STYLE.md` / `0026-executive-communication.yaml` prose edits (ac-02, ac-03, partly ac-04), and ac-01's `plugin.json` registration is config-shape. Defaulting all five to `code` (the implicit default for an untyped AC) misrepresents the close-time evidence each one expects and will route them through `spec.close`'s code-evidence path rather than the doc / config path.
**Evidence:** spec.yaml lines 8–27 (no `ac_type` field on any AC); METHOD.md table "Acceptance-criterion `ac_type`".
**Recommendation:** Add explicit `ac_type` to each AC. Suggested mapping: ac-01 `code` (hook script + plugin.json + prose; hook is the load-bearing artefact and is testable), ac-02 `doc`, ac-03 `doc`, ac-04 `doc` (prose-only with hook reinforcement; the closing evidence is the prose, not the hook behaviour), ac-05 `code` (substrate-typed phrasing inside the hook + matching prose). All five remain in the blocking band.

### [MEDIUM] Hook scope-gating is implicit, not pinned to an AC
**Category:** failure-mode
**Pass:** 2
**Description:** D1 says the hook fires "when the calling agent is inside drive/rally autonomy" and uses `ORBIT_NONINTERACTIVE=1` as the kill-switch. But no AC requires the hook to gate itself to that context. A PreToolUse hook on `AskUserQuestion` that fires unconditionally would interrupt every interactive Claude Code session that uses AskUserQuestion — a much wider blast radius than the spec intends. The failure mode "hook misfire = false-positive halt suppression" is named in interview.md D1 trade-off cell but does not appear as an acceptance criterion.
**Evidence:** interview.md D1 trade-off table (line 33 of decisions.md): "adds a new failure surface (hook misfire = false-positive halt suppression)"; spec.yaml has no AC covering the scope-gating contract.
**Recommendation:** Either extend ac-01 to include the scope-gating clause ("…fires only when `ORBIT_NONINTERACTIVE=1` and a `drive.yaml` is present in the working tree…") or add a sixth AC pinning the no-misfire behaviour explicitly. The simpler path is to fold it into ac-01's rewrite above.

### [LOW] D6 (card-boundary disjointness) is asserted, not verified by any AC
**Category:** test-gap
**Pass:** 2
**Description:** D6 names "different surfaces" as the boundary between 0037 / 0038 / 0042 and explicitly flags that the assumption is path-level disjointness. The disjointness map in interview.md is the verification surface for rally Stage 4, not for this spec — but if the assumption is wrong (e.g., symbol-level rather than path-level disjointness), implementation could collide with siblings at merge. No AC carries this risk; it lives in the rally lead's hands.
**Evidence:** interview.md D6 (lines 73–78) — "Confidence: Medium — assumes rally lead's disjointness check is path-level not symbol-level."
**Recommendation:** Not required for this spec — the rally is the right place to verify disjointness. Mentioned here only so the implementer knows the assumption is parked at the rally level and should not be re-litigated mid-implementation. No spec change needed.

### [LOW] ac-04's hook-reinforcement path is asserted in design but not in ACs
**Category:** test-gap
**Pass:** 2
**Description:** D4's recommendation is "prose-only with hook reinforcement" — the hook is supposed to read the current pipeline stage from `drive.yaml` and match it against the halt-condition's stage. ac-04 as written only restates the stage-vs-surface rule; it does not require the hook to perform the matching. If the implementer reads only the AC and not the design, they might ship the prose half and miss the hook half. The rewrite proposed under MEDIUM #1 above covers this; logging here as a distinct shape for completeness.
**Evidence:** decisions.md D4 (lines 90–110); spec.yaml ac-04 (lines 22–24).
**Recommendation:** Subsumed by MEDIUM #1's ac-04 rewrite. No standalone action.

---

## Honest Assessment

The spec's bones are coherent: every approved decision (D1–D6) maps to an AC, the gate AC (ac-01) is the right load-bearing item, and the rally context is well-handled (strict dep on 0037's `memory.match` already on the branch; soft proximity dep on 0038's drive/SKILL.md edit deferred to merge). The design is unusually thorough — six decisions with named substrate, a disjointness map, a pre-flight already cleared.

The weakness is at the AC layer. The ACs were written by lifting the card's scenario `then:` text verbatim into the spec, which means they read as the *rule* the discipline encodes rather than as the *verification statement* that closes the work. The design has the artefacts pinned — the ACs don't reference them. The biggest risk is at `/orb:review-pr` time: a reviewer reading the AC list alone cannot say what evidence closes each AC without cross-referencing decisions.md. That is exactly the substrate-engagement failure mode this card is trying to eliminate.

Recommended fix: rewrite ac-01..05 to name the artefacts (per MEDIUM #1), add `ac_type` (per MEDIUM #2), and fold hook scope-gating into ac-01 (per MEDIUM #3). Three local edits; no design rework needed.

**Verdict:** REQUEST_CHANGES

---

# Spec Review — Cycle 2

**Date:** 2026-05-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-19-act-when-authorised
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (hook infrastructure, cross-system into Claude Code's PreToolUse surface) — re-run to confirm cycle-1 P2 findings are closed | 0 new |
| 3 — Adversarial | not triggered | — |

## Cycle-1 finding disposition

| Cycle-1 finding | Severity | Status | Evidence in spec.yaml |
|-----------------|----------|--------|------------------------|
| ACs are rule-restatements, not verification statements | MEDIUM | **Closed** | All five ACs now name concrete substrate paths. ac-01 cites `plugins/orb/hooks/three-question-test.sh`, `plugins/orb/.claude-plugin/plugin.json`, `plugins/orb/skills/drive/SKILL.md`. ac-02 cites `plugins/orb/skills/drive/SKILL.md §1.6`. ac-03 cites `.orbit/STYLE.md` and `.orbit/cards/0026-executive-communication.yaml`. ac-04 cites `plugins/orb/skills/drive/SKILL.md` and `drive.yaml`. ac-05 cites `drive.yaml.autonomy`, memory key `mid-session-autonomy-contract-default-to-action-halt`, the spec `halt-conditions` field, and `plugins/orb/skills/drive/SKILL.md`. |
| `ac_type` field missing | MEDIUM | **Closed** | ac-02 / ac-03 / ac-04 carry `ac_type: doc`. ac-01 and ac-05 deliberately omit the field (defaulting to `code`) — matches the cycle-1 recommended mapping verbatim: hook script + `plugin.json` registration on ac-01 is testable code-shape; ac-05's substrate-typed phrasing lives inside the hook implementation. All five remain in the blocking band. |
| Hook scope-gating implicit, not pinned to an AC | MEDIUM | **Closed** | ac-01 now includes the gating clause: *"The hook is scope-gated — it only fires when `ORBIT_NONINTERACTIVE=1` AND a `.orbit/specs/<id>/drive.yaml` is present in the working tree; under those conditions it exits non-zero to suppress the halt"*. Folded into ac-01 as recommended; sixth AC not needed. |
| D6 disjointness not verified by any AC | LOW | Acknowledged in cycle 1, no spec change required (rally-level concern). Unchanged in cycle 2 — parked at the rally lead. |
| ac-04 hook-reinforcement path not in ACs | LOW | **Closed (subsumed)** | The rewritten ac-04 now contains: *"The D1 hook reads the current pipeline stage from `drive.yaml` and treats stage-cross widening as a violation of the authorisation question."* Hook half is now load-bearing in the AC text. |

## Pass 1 — Structural Scan

1. **AC testability.** All five ACs name observable artefacts or behaviours: hook file existence + registration + stderr behaviour + exit code (ac-01); a paragraph at a named section of `drive/SKILL.md` (ac-02, ac-04); paragraphs at two named files (ac-03); substrate-typed phrasing reproduced verbatim across hook + skill (ac-05). Every AC has a clear `/orb:review-pr` evidence surface.
2. **Constraint conflicts.** None. ac-02/04 prose pointers in `drive/SKILL.md` are coherent with ac-01's hook (skill prose mirrors hook behaviour), matching D3's "skill prose explains, hook enforces" rationale.
3. **Scope vs goal.** Goal pins the three-question test to halt-temptation moments. ACs cover the five approved decisions (D1–D5) without overshoot; D6 is correctly parked at rally level.
4. **Obvious gaps.** Hook failure-mode (misfire = false-positive halt suppression) is now covered by ac-01's scope-gating clause. Pre-flight verification (D1: confirm Claude Code's PreToolUse surface accepts `AskUserQuestion`) is acknowledged in interview.md "Open items" as the gate before implementation begins — the brief confirms it has not failed, so the hook path stands.
5. **Gate-AC description check (deterministic).** ac-01 is the only gate (`is_gate=1` per parser output). Description is non-empty (722 chars), not a placeholder token, ≥20 chars. **Pass.**
6. **Content signals.** Hook infrastructure + cross-system into Claude Code's `PreToolUse` surface remain present; Pass 2 re-triggered for confirmation.

**Pass 1 findings: 0.**

## Pass 2 — Assumption & Failure Re-run

Re-checked under cycle-2 edits:

1. **Hook misfire blast radius.** ac-01 now requires both `ORBIT_NONINTERACTIVE=1` AND a `drive.yaml` present. This double-gate keeps interactive Claude Code sessions untouched (they don't set `ORBIT_NONINTERACTIVE`) and keeps non-drive autonomy contexts untouched (no `drive.yaml`). Failure mode cleanly bounded.
2. **PreToolUse surface assumption.** Interview.md §"Open items" mandates pre-flight verification. The brief confirms the pre-flight has not failed; if it had, the fallback (CLI verb `orbit autonomy authorised?`) would re-shape ac-01. Routing this as design-time concern, not a spec-level finding.
3. **Test adequacy per AC.** ac-01 closes on a runnable hook + grep-checkable registration + grep-checkable prose. ac-02..04 close on grep-checkable prose at named paths (`doc` ac_type). ac-05 closes on a runnable hook whose stderr text matches the named substrate sources. All five have unambiguous evidence surfaces.

**Pass 2 findings: 0 new.**

## Honest Assessment

The cycle-1 fixes are clean. Each AC now names the artefact that closes it, the `ac_type` mapping matches the cycle-1 recommendation verbatim (doc on the three pure-prose ACs, code on the two hook-bearing ones), and ac-01's scope-gating clause keeps the hook's blast radius bounded to drive/rally contexts. The substrate-engagement failure mode the card was built to prevent — *AC list does not name the verification surface* — is no longer present here.

Two assumptions remain parked outside this spec: D1 pre-flight (Claude Code's PreToolUse surface accepts `AskUserQuestion`) and D6 path-level disjointness across rally siblings. Both are correctly owned upstream — the pre-flight is named in interview.md "Open items" and confirmed cleared by the brief; the disjointness check belongs to the rally lead's Stage 4. Neither blocks implementation.

Ready for `/orb:implement`.

**Verdict:** APPROVE
