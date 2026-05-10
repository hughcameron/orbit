# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-20-drive-live-visibility/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (cron, session lifecycle) | 2 |
| 3 — Adversarial | not triggered | — |

## Findings

### [MEDIUM] Cron prompt body does not enforce the read-only contract

**Category:** missing-requirement
**Pass:** 1
**Description:** Constraint line 3 and ac-02 require heartbeat prompts to be read-only — no drive.yaml mutation, no Agent launches. But ac-01 specifies only that the prompt body "instructs the agent to read drive.yaml, read progress.md if present, and emit the one-line heartbeat string." There is no requirement that the prompt text *itself* state the read-only contract to the agent that will service the check-in.
**Evidence:** ac-01 verification describes the scheduling mechanics; it never requires the prompt body to include the read-only instruction. When the cron fires, it becomes a new user message; without an explicit in-prompt contract, an agent could "helpfully" update drive.yaml or launch tools.
**Recommendation:** Strengthen ac-01 (or add a new AC) so that the cron prompt body explicitly tells the firing agent: "This is a read-only observability check. Do not modify drive.yaml. Do not launch Agents. Emit the single heartbeat line and stop."

### [MEDIUM] CronCreate failure at §2 has no documented handling

**Category:** failure-mode
**Pass:** 2
**Description:** ac-05 covers CronDelete failure on terminal paths (non-fatal, log and continue). But the symmetric case — CronCreate failure at §2 initialise/resume — is undefined. If CronCreate errors (rate limit, disallowed in the current harness, transient failure), should drive abort, proceed silently, or retry? The spec is silent.
**Evidence:** Constraints enumerate the CronList-then-CronCreate flow but not its error path. The scenario "Drive schedules a check-in at start" asserts the task is created, but drives running on harnesses without cron support would otherwise fail hard on first initialise.
**Recommendation:** Add an AC or extend ac-01: if CronCreate fails at §2, log one line (`heartbeat unavailable: <reason>`) and continue without the heartbeat. The heartbeat is an observability affordance; its absence must not block the pipeline.

### [LOW] Resumption announcement "cron:" line is additive beyond the interview

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-03 adds a new line to §11's resumption announcement block (`cron: active` / `cron: created`). This is a reasonable visibility surface but was not explicitly called out in the interview or decisions.md — D3 and D6 discussed cron lifecycle and hook-layer visibility, not the resumption-announcement format.
**Evidence:** interview.md Q3 (D3) covers the CronList/CronCreate reconciliation; Q6 (D6) explicitly puts SessionStart cron inspection out of scope. The resumption-announcement text is drive §11 territory, not explicitly mentioned.
**Recommendation:** Either keep ac-03 as-is (the addition is small and coherent with the visibility goal of this card) or explicitly reference the interview provenance. A single-line note in SKILL.md §11 suffices. Not a blocker.

### [LOW] Heartbeat firing during the 30-second escalation window

**Category:** failure-mode
**Pass:** 2
**Description:** §9 flow: emit escalation summary → schedule one-shot ping at +30s → at §10/§9 cleanup, CronDelete the recurring heartbeat. During the 30-second window before the escalation ping fires, the recurring heartbeat may still tick once and emit a line like `drive: iter=3/3 stage=escalated ac=- elapsed=...`. This is not wrong — it reflects reality — but it could mildly obscure the prominence of the escalation message the ping is trying to amplify.
**Evidence:** Timing analysis of §9 → CronDelete(heartbeat) → +30s escalation ping. Spec does not require CronDelete(heartbeat) to precede the escalation ping schedule.
**Recommendation:** Specify order in §9: (1) emit escalation summary, (2) CronDelete the recurring heartbeat (non-fatal on failure per ac-05), (3) schedule one-shot escalation ping at +30s. Updating the constraint text (or adding a bullet to ac-04) pins the order. Small fix.

---

## Honest Assessment

The spec is well-scoped and faithfully tracks the interview/decisions pack. The four card scenarios each map to concrete ACs (ac-01/02/03/09 heartbeat; ac-04 escalation ping; ac-03 resume survival; ac-06/07/08 four-option verdict), and the non-negotiable discipline (5-minute hardcoded interval, strict one-line format, four-option labels verbatim) is carried through.

The two MEDIUM findings are the real blockers: without the in-prompt read-only contract, the "safe to fire mid-fork" constraint rests on agent good behaviour rather than instruction, which is fragile; without a CronCreate failure mode, the drive becomes brittle on harnesses that don't support the tool primitive. Both are one-sentence fixes in SKILL.md plus one new AC each. The two LOW findings can be rolled into the same revision pass cheaply.

Re-review after those four items are addressed should land at APPROVE.
