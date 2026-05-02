# Tabletop sessions and the autonomy contract

**Date:** 2026-05-02
**Source:** McGill exit-mechanic resweep transcript (2026-05-01) + Carson session 2026-05-02 deriving the recommendation discipline (ops PR #3, hydrofoil PR #216) and noticing the per-session ask-cap creates its own blocker.

## Observation

A McGill session on 2026-05-01 surfaced a structural problem: even with a "lead with single recommendation" rule and a "one ask per cycle" cap, a cron-driven agent firing every two hours can find a way to consume its ask budget every cycle. Twelve cycles a day × one ask each = twelve hand-backs per day. The autonomy collapses to "wait for Hugh."

The diagnosis: the agent treats every uncertain decision as ask-worthy because it doesn't have *pre-committed answers*. The ask cap is treating a symptom; the cause is the absence of a decision tree the agent can evaluate observed state against.

The unlock — proposed by Hugh, sharpened in the same session — is to **pre-commit the decisions**. A war-gamed (renamed: tabletop) decision tree sits between the goal and the agent. Most decisions are pre-resolved by the tree. Gaps default to *hold-and-log*, never *ask-and-wait*. The ask surface shrinks dramatically; the cap becomes a fallback rather than the load-bearing rule.

## The autonomy contract

The artefact a tabletop session produces. Five sections, all enumerated explicitly:

1. **Objective function.** Numeric targets defining success this sprint. (Example: Hydrofoil's cloud-era weekly DD ≤ AUD 100, monthly net ≥ 0 over 30 days, trade-rate ≤ 50/UTC day.)
2. **Standing orders.** Actions taken every cycle regardless of state. (Read account, check halt conditions, log heartbeat.)
3. **Branch table.** Observed state → prescribed action. The decision tree proper. *"If sweep verdict landed and PF≥1.0 and trail/spread ≥3×, ship to demo; if PF<1.0, file collapse memo and continue four-family wait."* Audited via a **HAZOP-style coverage matrix** (state parameters × guide words: NO / MORE / LESS / AS WELL AS / PART OF / REVERSE / OTHER THAN). Each cell is filled with a branch entry, an explicit hold-and-log annotation, or a justified N/A. The matrix converts the branch table from a freeform list into a provably-considered surface.
4. **Halt conditions.** Pre-committed kill switches with explicit revert paths. (Hydrofoil's *infra-drift stop-loss* and *divergence halt* are halt conditions in this language.)
5. **Escalation triggers.** Narrow, listed; require proposed action when fired. The only states that legitimately consume the ask surface.

The agent's session protocol becomes:

```
1. Read the contract.
2. Observe state.
3. Match state against the tree:
   - Branch hit?         → take prescribed action.
   - Halt condition hit? → halt and report.
   - Escalation trigger? → escalate with proposed action.
   - None of the above?  → log observation, hold, continue.
                           DO NOT ASK. DO NOT IMPROVISE.
```

Step 4's hold-and-log default is the load-bearing piece. Unmatched state never asks; it never invents; it accumulates as observation a future cycle (or the next tabletop) can act on. This is the explicit divergence from SRE practice — Google's incident-response framework permits improvisation when runbooks don't resolve issues, on the assumption that a competent human is at the keyboard. Our agent isn't. The contract is the only competence it has.

## The tabletop session

A discussion-based exercise — NIST SP 800-84's term — between Hugh and the project's primary agent. Roles split per NIST: Hugh is **facilitator + domain expert**; the agent is **scribe / data collector**. The output is the autonomy contract for one goal / one sprint / one autonomous-execution context. The contract serves as the *After-Action Report* NIST mandates as the durable artefact of every exercise — no tabletop ends without it written.

Session structure (target ~1 hour for amendment passes; first-pass sessions on a new context may run longer):

1. **Brief.** Facilitator restates objective function, standing orders, recent halt-condition history.
2. **Scenario injection.** Some fraction of scenarios drawn from *real past cycle logs*, not imagined — Google SRE Game Master discipline ("a real one taken from the annals of history"). Imagined scenarios are permitted but explicitly flagged. Projects with no cycle history rely on imagined scenarios for pass 1; pass 2 onwards is increasingly log-grounded.
3. **Branch-table population.** Walk the HAZOP matrix (state parameters × guide words) cell by cell. Each gets a branch entry, hold-and-log, or justified N/A.
4. **Premortem block (halt conditions only).** Klein's prospective hindsight applied surgically: imagine the agent has destroyed the project objective in the next two weeks; why? Each answer becomes a halt condition with a revert path. The premortem is a tool here, not the methodology's spine — a non-anxious agent on a cron has no optimism bias to overcome, so the technique is used narrowly.
5. **Escalation trigger enumeration.** Narrow, listed; each specifies the required content of the ask (account state + proposed action).
6. **Hot-wash debrief.** Immediate meta-observation pass before the contract is finalised: what kept coming up? What was the agent confused about? Captured fresh, before formal write-up sanitises it.
7. **Contract written.** As the AAR.

The leverage ratio is the strategic point: 1 hour of focused human attention → N hours of autonomous execution downstream. If cron fires every 2 hours and a contract lasts 2 weeks, that's ~168 cycles of leverage from one session.

Three distinct payoffs:

- **Engineering throughput** — the surface metric. More cycles per day actually move the needle.
- **Predictability** — the underlying win. Agents become trustworthy in autonomous mode because they execute pre-committed branches, not de-novo strategy. The branch table is **compensation for missing expertise** — Klein's Recognition-Primed Decision-making says human experts retrieve actions from a tacit pattern library; the agent has no such library, so the contract substitutes externalised recognition for the missing tacit knowledge.
- **Card distillation evidence** — the subtle but important payoff. A tabletop scenario with no clean home in any existing card is a signal: either the cards are stale or the product has grown a capability that hasn't been distilled. Tabletops feed back into `/orb:distill`.

Tabletops compound. Pass 1 fills the obvious branches. Pass 2 (after real cycles run) fills the gaps observed state surfaced — the unmatched-state log from hold-and-log is the input to contract evolution. By pass 3–4 the contract is largely settled and most cycles run untouched.

## Prior art

Research synthesis from a 2026-05-02 background agent across five sources: NIST SP 800-84 (CSRC glossary + final pub), Klein's pre-mortem (HBR 2007 + gary-klein.com), Google SRE Workbook incident-response chapter, Klein's Recognition-Primed Decision-Making (Wikipedia + decision-science overviews), and HAZOP / FMEA literature.

**Adopted verbatim:**

- **The term "tabletop exercise"** with NIST's canonical definition: *"a discussion-based exercise where personnel with roles and responsibilities... meet to validate the content of the plan by discussing their roles during an emergency and their responses."* The term carries the right baggage: discussion-based, scenario-driven, validates an existing plan.
- **Facilitator + scenario injection + question-prompt loop** (NIST). Facilitator briefs, walks scenarios, may inject additional questions periodically; data-collector documents.
- **After-Action Report as mandatory output** (NIST: *"every TT&E activity — without exception — should produce an after-action report"*). The autonomy contract IS our AAR.
- **HAZOP guide words** (IEC 61882): NO, MORE, LESS, AS WELL AS, PART OF, REVERSE, OTHER THAN. Applied to state parameters they generate deviations systematically.

**Adapted, not adopted:**

- **Pre-mortem** (Klein, HBR 2007) used surgically for halt-condition population only, not as the methodology's spine. The premortem's mechanism is psychological — overcoming optimism bias via prospective hindsight — and a non-anxious agent has no such bias to overcome.
- **SRE Wheel of Misfortune.** The exercise (GM throws a real-historical scenario at the on-call victim) maps to: human plays GM, agent plays victim, scenarios from past cycle logs. The output differs — SRE trains humans; we capture branch-table entries.
- **Recognition-Primed Decision-making** (Klein, *Sources of Power* 1999) reframed: the branch table doesn't make the agent more expert than de-novo decision-making; it *substitutes externalised recognition* for the tacit pattern library a human expert would have. Operational claim, not general superiority claim.
- **NIST exercise lifecycle** (design → conduct → evaluate → improve) reframed as contract revision cadence: the unmatched-state log from hold-and-log is the input to the next tabletop.

**Explicitly rejected:**

- **Pre-mortem as the dominant frame.** Importing the full ritual would be ceremony without mechanism. Use it surgically (halt conditions), not generally.
- **SRE's improvisation tolerance.** Google's framework explicitly supports "trying new recovery options in a methodical manner" when runbooks fail. We reject this entirely. Unmatched state hits hold-and-log, never invents. SRE assumes a competent human at the keyboard; we assume a competent human only at tabletop time.

**Gaps the literature flags (open work):**

1. **Scenario sourcing discipline.** SRE draws from real history; we have no equivalent rule. Adopted in the session structure above (step 2) — first-pass projects without cycle logs are explicitly noted as a degenerate case.
2. **Roles when more humans are present.** NIST splits *facilitator, participants, data collector, evaluator*; we've specified human + agent. Listed in open questions below.
3. **Coverage measurement.** HAZOP tracks *node × parameter × guide word* coverage as a matrix. Adopted as the §3 audit artefact.
4. **Severity / likelihood scoring (FMEA).** Halt conditions are currently flat — all kill switches feel equal. When two fire simultaneously, which wins? Listed in open questions.
5. **Contract drift / staleness.** NIST assumes plans go stale and exercises re-validate them. We have no expiry trigger. Listed in open questions.

Sources cited:

- [NIST SP 800-84](https://csrc.nist.gov/pubs/sp/800/84/final), [CSRC glossary on tabletop exercise](https://csrc.nist.gov/glossary/term/tabletop_exercise)
- [Klein, "Performing a Project Premortem" (HBR Sep 2007)](https://hbr.org/2007/09/performing-a-project-premortem); [gary-klein.com PreMortem method](https://www.gary-klein.com/premortem)
- [Google SRE Workbook — Incident Response](https://sre.google/workbook/incident-response/); [Wheel of Misfortune (dastergon)](https://github.com/dastergon/wheel-of-misfortune)
- [Wikipedia — Recognition-Primed Decision](https://en.wikipedia.org/wiki/Recognition_primed_decision)
- [Wikipedia — Hazard and Operability Study](https://en.wikipedia.org/wiki/Hazard_and_operability_study)

## Open questions

- **Where does the contract live?** Per-card metadata? A new `orbit/contracts/<card-id>.yaml` artefact? Embedded in card scenarios via the existing `gate: true` mechanism (decision 0011 / 0013 in orbit)?
- **Cadence.** Per sprint? Per major capability? On a trigger ("cron is about to turn on for X — tabletop first")? Plus a staleness-expiry rule — when does a contract need re-tabletop-ing? Cycle count? Calendar? Drift in objective-function reality vs. target?
- **Coverage threshold.** The HAZOP matrix gives a coverage metric (% of cells filled with branch-or-N/A vs. left blank). What threshold ships? 100% is probably wrong (forces N/A justifications for genuinely irrelevant cells); 80% may be right; the empirical first instance will tell us.
- **Severity / likelihood scoring on halt conditions** (FMEA gap from prior art). When two halt conditions fire simultaneously, which wins? Adopt FMEA's S × O × D scoring? Or simpler ordinal priority?
- **Roles when more humans are present.** NIST prescribes facilitator / participants / data collector / evaluator. We've specified Hugh + agent. When a second human joins (e.g. Hugh + Pedro at a future tabletop), who plays which role? Risk: drift if not specified.
- **Skill (`/orb:tabletop`) shape.** A facilitator skill could enforce the five-section structure, prompt for HAZOP-style gap-filling, and emit the contract in a canonical format. Defer until the methodology has run at least once and we know what a useful skill does.
- **Interaction with `/orb:distill`.** Tabletop output flags gaps in the card register; distill runs to fill them; cards refined; next tabletop benefits. The two skills are complementary — distill creates capabilities from source material, tabletop creates contracts from goals + cards.
- **Versioning.** A contract is a living document — branches added as cycles surface gaps. Versioning convention? Diff review per amendment?
- **First instance.** Hydrofoil's exit-mechanic resweep + four-family verdict is the natural first tabletop. Run it before card 0011 (cron-driven execution) ships, so card 0011 is informed by the methodology rather than imposing on it.

## Status

Held as a memo. Once prior-art research lands and the first hydrofoil tabletop runs, distill into:

- An **orbit card** describing the capability (orbit provides tabletop methodology + autonomy contract pattern).
- An **ops decision** (0030, after the recommendation-discipline 0029) recording the methodology with prior-art citations.
- An **ops playbook** (`docs/tabletop-facilitators-guide.md`) — operational guide for running a session.
- A future **`/orb:tabletop` skill** — operationalises the session structure once the methodology has run at least once and we know what the skill needs to do.

The first consumer is hydrofoil's card 0011 (cron-driven execution, planned). Do not author 0011 until the tabletop pattern is concrete; the card's scenarios should be a worked example of the methodology, not a guess at it.
