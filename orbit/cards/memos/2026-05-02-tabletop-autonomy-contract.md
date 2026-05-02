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
3. **Branch table.** Observed state → prescribed action. The decision tree proper. *"If sweep verdict landed and PF≥1.0 and trail/spread ≥3×, ship to demo; if PF<1.0, file collapse memo and continue four-family wait."*
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
                           DO NOT ASK.
```

Step 4's hold-and-log default is the load-bearing piece. Unmatched state never asks; it accumulates as observation a future cycle (or the next tabletop) can act on.

## The tabletop session

A 1-hour collaborative scenario-enumeration session between Hugh and the project's primary agent. The output is the autonomy contract for one goal / one sprint / one autonomous-execution context.

The leverage ratio is the strategic point: 1 hour of focused human attention → N hours of autonomous execution downstream. If cron fires every 2 hours and a contract lasts 2 weeks, that's ~168 cycles of leverage from one session.

Three distinct payoffs:

- **Engineering throughput** — the surface metric. More cycles per day actually move the needle.
- **Predictability** — the underlying win. Agents become trustworthy in autonomous mode because they execute pre-committed branches, not de-novo strategy.
- **Card distillation evidence** — the subtle but important payoff. A tabletop scenario with no clean home in any existing card is a signal: either the cards are stale or the product has grown a capability that hasn't been distilled. Tabletops feed back into `/orb:distill`.

Tabletops compound. Pass 1 fills the obvious branches. Pass 2 (after real cycles run) fills the gaps observed state surfaced. By pass 3–4 the contract is largely settled and most cycles run untouched.

## Prior art (research in flight)

A research agent is consulting five sources (2026-05-02): Klein's pre-mortem (HBR 2007), NIST SP 800-84 on tabletop exercises, Google SRE game-day material, Klein's Recognition-Primed Decision-Making (Sources of Power 1999), and HAZOP / FMEA guide-word patterns. Findings will be merged into this memo and cited in the eventual ops decision recording the methodology.

The hypotheses worth testing against prior art:

- **Pre-mortem ≠ tabletop.** Pre-mortem is failure-imagination (narrower); tabletop covers the full state tree (broader).
- **Tabletop has canonical structure (NIST).** Scenario injection, decision points, hot-wash debrief. Likely adopt-as-is.
- **RPD validates pre-committed branches.** The autonomy contract operationalises Klein's recognition-primed decision-making: "recognition" = matching observed state to a pre-committed branch.
- **HAZOP guide-words could fill gaps.** NO / MORE / LESS / AS WELL AS as systematic prompts during the scenario-enumeration step of a tabletop session.

## Open questions

- **Where does the contract live?** Per-card metadata? A new `orbit/contracts/<card-id>.yaml` artefact? Embedded in card scenarios via the existing `gate: true` mechanism (decision 0011 / 0013 in orbit)?
- **Cadence.** Per sprint? Per major capability? On a trigger ("cron is about to turn on for X — tabletop first")?
- **Who facilitates.** Hugh + project agent collaboratively? A dedicated facilitator skill (`/orb:tabletop`)? The skill could enforce the five-section structure and prompt for HAZOP-style gap-filling.
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
