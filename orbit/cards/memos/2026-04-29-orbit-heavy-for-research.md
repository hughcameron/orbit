Orbit's Card → Design → Spec → Implement → Review → Ship pipeline is calibrated for engineering work with architectural consequences. It's over-procedural for research operations — iterative loops where the goal is "test a hypothesis, measure the outcome, decide what's next."

Observed symptoms in a research-heavy project:

- **Cards describe engineering capabilities, not research hypotheses.** The card taxonomy covers plumbing (adapters, pipelines, signal engines) but has no representation for "we believe X; here's the evidence." Research iterations don't have long-term shape consequences — they're disposable experiments.
- **Approval gates designed for production changes slow down experimental iteration.** On a demo/sandbox environment, every approval is a delay between iterations. The bottleneck becomes operator availability rather than compute or quality.
- **Sprint goals measured activities, not outcomes.** "Run the pipeline end-to-end" and "one sweep per day" are activity metrics. A team could perfectly satisfy both while the actual target metric moved in the wrong direction. Sprint goals shaped for plumbing produce plumbing.
- **Session-start rituals read specs and decisions but not the outcome metric.** The agent reads the spec, reads the decisions folder, reads the card taxonomy — and never looks at the thing it's trying to improve. The goal stays abstract because nothing in the startup ritual grounds it in reality.
- **Infrastructure drift goes undetected.** Multiple consecutive sessions can ship operator-productivity fixes (docs, config, refactors) with zero movement on the research objective. There's no stop-loss: "if the last N PRs were infra, the next must move the target metric."

What research operations need instead:

1. **Lightweight iteration records** — hypothesis, params, result, decision — not a full card/spec/review cycle per experiment.
2. **Session-start grounded in the outcome metric** — read the current state of the thing you're trying to improve before reading any specs.
3. **Sprint goals in outcome terms** — "target metric improves by X within Y time" not "run pipeline daily."
4. **Relaxed approval gates for sandbox experiments** — approval still required for production-affecting changes; not required for experimental iterations in safe environments.
5. **Infra-drift stop-loss** — after N consecutive non-research PRs, the next one must move research economics with a measured target.
6. **Decoupled research workflow** — reserve orbit's full ceremony for engineering work. Research gets its own lighter-weight loop.

This isn't a bug in orbit — it's a scope gap. Orbit was built for engineering specification work and it does that well. Research operations are a different domain with different iteration cadence, different risk profile, and different success metrics.
