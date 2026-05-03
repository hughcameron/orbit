# Tabletop sessions and the autonomy contract

**Date:** 2026-05-02
**Source:** Cross-project session deriving a recommendation discipline (ops decision 0029) and noticing the per-session ask-cap creates its own blocker.

## Observation

A recent agent session surfaced a structural problem: even with a "lead with single recommendation" rule and a "one ask per cycle" cap, a cron-driven agent firing every two hours can find a way to consume its ask budget every cycle. Twelve cycles a day × one ask each = twelve hand-backs per day. The autonomy collapses to "wait for the human."

The diagnosis: the agent treats every uncertain decision as ask-worthy because it doesn't have *pre-committed answers*. The ask cap is treating a symptom; the cause is the absence of a decision tree the agent can evaluate observed state against.

The unlock is to **pre-commit the decisions**. A tabletop-derived decision tree sits between the goal and the agent. Most decisions are pre-resolved by the tree. Gaps default to *hold-and-log*, never *ask-and-wait*. The ask surface shrinks dramatically; the cap becomes a fallback rather than the load-bearing rule.

## The autonomy contract

The artefact a tabletop session produces. Five sections, all enumerated explicitly:

1. **Objective function.** Numeric targets defining success this sprint. (Example: weekly drawdown ≤ X, monthly net ≥ Y, action-rate ≤ Z per day.)
2. **Standing orders.** Actions taken every cycle regardless of state. (Read state, check halt conditions, log heartbeat.)
3. **Branch table.** Observed state → prescribed action. The decision tree proper. *"If <gate condition> holds, take action A; if it doesn't, file an observation memo and continue waiting on the upstream variable."* Audited via a **HAZOP-style coverage matrix** (state parameters × guide words: NO / MORE / LESS / AS WELL AS / PART OF / REVERSE / OTHER THAN). Each cell is filled with a branch entry, an explicit hold-and-log annotation, or a justified N/A. The matrix converts the branch table from a freeform list into a provably-considered surface.
4. **Halt conditions.** Pre-committed kill switches with explicit revert paths. (Examples: budget-exhaustion triggers; metric-drift triggers; consecutive-no-progress triggers.)
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

A discussion-based exercise — NIST SP 800-84's term — between the human owner and the project's primary agent. Roles split per NIST: the human is **facilitator + domain expert**; the agent is **scribe / data collector**. The output is the autonomy contract for one goal / one sprint / one autonomous-execution context. The contract serves as the *After-Action Report* NIST mandates as the durable artefact of every exercise — no tabletop ends without it written.

Session structure (target ~1 hour for amendment passes; first-pass sessions on a new context may run longer):

1. **Brief.** Facilitator restates objective function, standing orders, recent halt-condition history.
2. **Scenario injection.** Some fraction of scenarios drawn from *real past cycle logs*, not imagined — Google SRE Game Master discipline ("a real one taken from the annals of history"). Imagined scenarios are permitted but explicitly flagged. Projects with no cycle history rely on imagined scenarios for pass 1; pass 2 onwards is increasingly log-grounded.
3. **Branch-table population.** Walk the HAZOP matrix (state parameters × guide words) cell by cell. Each gets a branch entry, hold-and-log, or justified N/A.
4. **Premortem block (halt conditions only).** Klein's prospective hindsight applied surgically: imagine the agent has destroyed the project objective in the next two weeks; why? Each answer becomes a halt condition with a revert path. The premortem is a tool here, not the methodology's spine — a non-anxious agent on a cron has no optimism bias to overcome, so the technique is used narrowly.
5. **Escalation trigger enumeration.** Narrow, listed; each specifies the required content of the ask (state snapshot + proposed action).
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

- **Where does the contract live?** ~~Per-card metadata? A new `orbit/contracts/<card-id>.yaml` artefact? Embedded in card scenarios via the existing `gate: true` mechanism?~~ **Resolved (first instance, 2026-05-02):** `<project>/orbit/contracts/<date>-<scope>.yaml` — sibling to `cards/`, `decisions/`, `reviews/`, `specs/`. Contracts are durable structured artefacts, not memos; YAML over markdown so the agent can parse the contract data fields (objective, halts, branches, escalation triggers) directly, with narrative sections (rationale, hot-wash, provenance) embedded as YAML multi-line strings. Single file, dual purpose: machine-read at cycle start, human-read at next tabletop. Mis-filing the first draft into `cards/memos/` exposed why a separate top-level category matters: `memos/` is rough-idea territory and category drift is a real failure mode.
- **Cadence.** Per sprint? Per major capability? On a trigger ("cron is about to turn on for X — tabletop first")? Plus a staleness-expiry rule — when does a contract need re-tabletop-ing? Cycle count? Calendar? Drift in objective-function reality vs. target?
- **Coverage threshold.** The HAZOP matrix gives a coverage metric (% of cells filled with branch-or-N/A vs. left blank). What threshold ships? 100% is probably wrong (forces N/A justifications for genuinely irrelevant cells); 80% may be right; the empirical first instance will tell us.
- **Severity / likelihood scoring on halt conditions** (FMEA gap from prior art). When two halt conditions fire simultaneously, which wins? Adopt FMEA's S × O × D scoring? Or simpler ordinal priority?
- **Roles when more humans are present.** NIST prescribes facilitator / participants / data collector / evaluator. We've specified human + agent. When a second human joins, who plays which role? Risk: drift if not specified.
- **Skill (`/orb:tabletop`) shape.** A facilitator skill could enforce the five-section structure, prompt for HAZOP-style gap-filling, and emit the contract in a canonical format. Defer until the methodology has run at least once and we know what a useful skill does.
- **Interaction with `/orb:distill`.** Tabletop output flags gaps in the card register; distill runs to fill them; cards refined; next tabletop benefits. The two skills are complementary — distill creates capabilities from source material, tabletop creates contracts from goals + cards.
- **Versioning.** A contract is a living document — branches added as cycles surface gaps. Versioning convention? Diff review per amendment?

## First instance findings (2026-05-02)

The methodology ran for the first time on 2026-05-02 against an active sprint. Three findings worth promoting from session-specific to general methodology, before distillation into a skill:

**1. The load-bearing test for cutting halts.** *"What failure mode does this halt catch that the branch table cannot?"* Halts earn their place when they protect against state the branches structurally cannot handle: long-tail breakages the table doesn't enumerate, faster-than-cycle-cadence transitions, or states implying the branch table itself is wrong (e.g. the assumptions every branch sits on are violated). Any halt that fails this test is ceremony — the branch table at cycle cadence already covers it. Surfaced when a candidate "absolute equity floor" halt was rejected on first principles: equity-trajectory observation belongs at branch cadence, not as a separate kill switch. Promoted to the session protocol — Step 4 (premortem block) now applies this test to each candidate before accepting it.

**2. Halts that name engine-layer fields require schema back-pressure.** When a halt's tripwire references a field in an engine event payload, audit the actual schema before accepting the halt. A first-pass halt named a field whose semantics didn't apply to half the configuration space the contract had to govern; only a rename pass exposed it. Add as a session-protocol checkpoint at Step 5 (escalation enumeration): for each halt and escalation, name the schema field it depends on and verify it exists across all live and proposed configs. If a needed field doesn't exist, the contract surfaces a follow-up engineering bead — the halt either ships with a documented agent-side fallback computation or waits for the field.

**3. Catalysing-scenario priority.** When a scenario walk surfaces a question about the methodology itself rather than producing a branch, abandon the scenario queue and walk the spine question to completion. The displaced scenarios downgrade to matrix-audit fodder for the scribe pass rather than discussion-protocol fodder. The first instance ran Brief → Scenario 1 → halt-condition deep dive → wrap-up rather than the prescribed Brief → all scenarios → premortem → wrap-up; the catalysing scenario produced more leverage than the five queued scenarios it displaced. Adopt as a session-protocol amendment: Step 2 (scenario injection) explicitly permits the facilitator to pivot to spine-question mode mid-walk, with the remaining queue surfacing in scribe-pass output rather than the live session.

**4. Posture observation.** The "single recommendation" posture (lead with `recommend X because Y` rather than offer a 2–3 option menu) produced higher-quality reframes than option-menus in the first instance. The format invites pushback in a way option-menus don't — the facilitator corrects the recommendation rather than picking among options. Compatible with the recommendation-discipline rule emerging in user-level CLAUDE.md.

## Second instance findings (2026-05-03)

The methodology ran for the second time on 2026-05-03 against the FineType project (facilitator: Hugh; scribe: Nightingale). Pass-1 produced the contract `orbit/contracts/2026-05-03-gittables-90-percent-roundtrip.yaml` and the seven hot-wash observations below. None contradict the methodology as written; all are pass-2 fodder for evolving it.

**1. Probe "what do you want the cron to OWN" early.** The original session brief proposed scoping the contract to `validate-corpus iter-5+`. The facilitator's reframe — full GitTables, no PR merges, retrain authority — was an order-of-magnitude larger than the bottom-up brief assumed. Bottom-up briefs assembled from recent spec dirs systematically miss this scope question. Methodology amendment: facilitator should explicitly probe ownership scope ("what do you want the cron to OWN?") before scenario walks begin, not let the brief default to the most recent spec context.

**2. Step-1 metric-gameability check before any scenario walks.** The non-trivial-prediction floor only got pinned because the scribe challenged "validation rate >90%" as undefined. A cron agent optimising raw rejects has a trivial winning move (push every prediction to `plain_text`). Promote to a methodology Step 0 / Step 1 checkpoint: *"Is the proposed metric game-able by the contracted agent?"* Run before scenario injection. The contracted agent will optimise whatever you measure — verify that the optimisation aligns with intent.

**3. Explicitly ask "what value comes from data the gate doesn't see?"** The dual-metric framing (gate metric on frozen 2k-file holdout + corpus value metric on the remaining ~1.016M files) emerged only from facilitator pushback that a frozen holdout left the bulk of the corpus invisible to the contract. The answer surfaced three additional uses of the broader corpus: failure-mechanism discovery, training-data harvest, taxonomy coverage map. Methodology amendment: when the gate is a sampled subset, ask explicitly what value comes from data the gate doesn't see — prevents subset-only contracts that under-use available signal.

**4. Cross-cycle invariants always become halts.** Three halts in this contract share a structural shape — branches structurally cannot bridge cycles, so cross-cycle observations must materialise as halt invariants:

- `H13 holdout_stagnation` (gate Δ < 0.1% over 8 cycles)
- `H08 failure_log REVERSE` (append-only count drops)
- `H09 coverage_log REVERSE` (visited count drops)

The cron has no inter-cycle memory mechanism other than halts that read prior-cycle log state. Promote to a methodology pattern: *"cross-cycle invariants always become halts; branches cannot bridge cycles."* Pair with the §3 branch-table walk so candidate branches that depend on cross-cycle observation get re-routed to §4 by construction.

**5. Contract write step files auxiliary artefacts AND engineering beads, not just the contract YAML.** The FineType contract surfaced a sibling load-bearing-paths registry, plus engineering work (gate harness build-out, content-hash dedup, lockfile mechanism, append-only log integrity tooling, eval report header schema with model SHA + tag + filename). All filed as beads alongside the contract. Methodology amendment: the contract write step explicitly enumerates *"what auxiliary artefacts and engineering work does this contract require to operate?"* and files them — auxiliary YAML/registry files in the contracts dir, engineering deltas in the project's bead tracker.

**6. The orbit `schedule` skill is online-only — wrong tool for local-machine cron contracts.** The scribe reflexively reached for `/schedule` when scoping the pass-2 tabletop trigger and had to be redirected. `schedule` provisions remote cloud agents in Anthropic's infrastructure; it does not wire local cron jobs. For tabletop-derived contracts whose cron-firing agent must run on the user's machine (data locality, GPU access, local repo state), local equivalents are the right tool: `CronCreate` (in-session, REPL-bound) for low-stakes reminders; `launchd` / system cron for unattended autonomous firing — the latter is infrastructure-team work. Methodology amendment: when authoring a contract, state explicitly whether the cron-firing agent runs local or remote, and route to the appropriate scheduling tool from the start. Add a §2 standing-orders entry naming the scheduling mechanism so future tabletops can audit it.

**7. `CronCreate`'s `durable: true` was silently ignored on this harness.** Both attempts produced session-only crons (no `.claude/scheduled_tasks.json` file created). For multi-day reminders, never trust a session-bound mechanism. The fallback that worked: write the next-tabletop prompt to a discoverable file at a sibling path (e.g. `orbit/contracts/<date>-pass2-prep-prompt.md`) and let the human invoke it manually OR have infrastructure (launchd) wire it in. Methodology amendment: the contract write step always files the next-tabletop prompt as a sibling artefact, even if a cron tool also schedules it. The artefact is the single source of truth; the cron is the convenience layer.

Lessons 6 and 7 are particularly load-bearing for any future tabletop in this ops setup — they prevent the same scribe error recurring in subsequent instances.

## Status

Held as a memo. First instance complete; findings above promoted to methodology. Continue toward:

- An **orbit card** describing the capability (orbit provides tabletop methodology + autonomy contract pattern).
- A **decision** in the consuming project (or in a cross-project ops repo) recording the methodology with prior-art citations.
- A **playbook** — operational guide for running a session.
- A future **`/orb:tabletop` skill** — operationalises the session structure once the methodology has run at least once and we know what the skill needs to do.

Don't author a consuming project's cron-execution card until the tabletop pattern is concrete; the card's scenarios should be a worked example of the methodology, not a guess at it.
