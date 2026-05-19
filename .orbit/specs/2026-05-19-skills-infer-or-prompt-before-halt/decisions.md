# Decision pack — skills infer or prompt before halt

Card: `.orbit/cards/0038-skills-infer-or-prompt-before-halt.yaml`
Spec: `2026-05-19-skills-infer-or-prompt-before-halt`
Stage: design analyst (Stage 2 of rally fan-out, parent rally `2026-05-19-agent-side-substrate-engagement-rally`)

## Card summary

Every `/orb:*` skill that requires a contextual argument (typically a spec-id) must follow the same three-step recovery — **infer → prompt → halt** — and never silently stop on a recoverable missing arg. Today the behaviour varies skill-by-skill: `implement` resolves via `spec list --status open` (no session-card read), `drive` does likewise (filtering for drive sidecars), `review-pr` and `review-spec` hard-stop with `no spec-id provided`, and `audit` runs across all specs. Uniformity is AC-04 and the disjointness driver — implementation lives "where enforcement is strongest".

## Evidence base

- **Substrate already exists.** `.orbit/.session-card` is a single-line card-slug file, written by `orbit session set-card <id>` and read via `read_session_card(layout, verb)` in `orbit-state/crates/core/src/session.rs:109`. Returns `Ok(None)` when missing or empty. Today only `session.distill` reads it (verbs.rs:4326-4336).
- **Card 0036 (depends-on)** shipped `.session-card` for session scoping; this card consumes that substrate.
- **`orbit spec list --status open`** is the open-spec enumerator (verbs.rs:1325, reused inside `overview`). Cheap, structured.
- **Skill-side variation today:**
  - `review-pr/SKILL.md:28`: hard-stop with `no spec-id provided — review-pr requires a spec-id under the orbit-state substrate`.
  - `review-spec/SKILL.md:35`: same pattern, no auto-discovery.
  - `implement/SKILL.md:52-77`: §"Input contract" — arg or `spec list --status open` with single/zero/multi branches; zero/multi halt.
  - `drive/SKILL.md:30-54`: three-branch resolution (card-path / spec-id / no-arg with drive-sidecar filter).
  - `audit/SKILL.md:13-17`: arg-or-all; no halt branch.
  - `memo/SKILL.md:24-27`: arg-or-AskUserQuestion (already does infer→prompt for a *slug*, not a spec-id).
- **The card-vs-spec mismatch.** `.session-card` binds a **card slug**, not a spec id. AC-01 says "uses the bound spec automatically", but the substrate today binds a card. A card may have zero, one, or many open specs. This is the load-bearing design question (decision 2 below).
- **Sibling cards in the rally.** 0037 (memory-gates-decisions) touches design/spec/inline-match surfaces. 0042 (act-when-authorised) edits drive/rally autonomy prose. Neither touches the per-skill input-contract sections this card edits, nor the CLI resolver this card likely lands. Disjointness looks clean if this card's surface stays in the resolver + per-skill §Usage / §Input-contract sections.

## Decisions

### Decision 1 — Where the infer → prompt → halt logic lives

**Context.** AC-04 says "implementation lives wherever enforcement is strongest (likely a shared CLI resolver, possibly with thin per-skill wrappers), not where prose interpretation is required". The choice is between (a) trusting every SKILL.md to follow a shared procedure in prose, (b) factoring the resolution into a small shell helper at `plugins/orb/scripts/`, or (c) shipping a first-class `orbit spec resolve` CLI verb that returns the resolved spec-id (or a structured "needs prompt" envelope) and that every skill calls.

**Options.**

1. **Shared prose contract in each SKILL.md.** Add a §"Input contract" block to every affected skill that names the three branches verbatim, and rely on review to catch drift.
2. **Shell helper at `plugins/orb/scripts/orbit-resolve-spec.sh`.** Encapsulates the three-step resolution; emits the resolved spec id on stdout, or a structured "prompt-needed" payload on a special exit code with the open-spec list. Each SKILL.md replaces its current branches with `orbit-resolve-spec.sh` invocation.
3. **First-class CLI verb `orbit spec resolve`.** Returns JSON `{resolved: "<id>"}` on success, or `{prompt_with: [<open-spec-ids>]}` when nothing is bound and multiple open specs exist, or `Error::unavailable` when both fallbacks fail. Implemented in `orbit-state/crates/core/src/verbs.rs` alongside `spec_list` / `spec_show`, plumbed through `crates/cli/src/main.rs`.

**Trade-offs.**

- (1) is cheapest to write but loses AC-04's "enforcement is strongest" test — prose drifts and the same divergence we have today recurs. The card explicitly names this as not where enforcement lives.
- (2) factors logic out of prose but lives in shell — fragile parsing, no schema, and the prompt path still requires the skill to interpret a structured payload from a shell helper. Marginal hardening over (1).
- (3) gives the resolver a typed wire shape, a single test surface in Rust, and one call site per skill. The skill still owns the `AskUserQuestion` call (the prompt is an agent-action; the CLI can't issue it), but the resolver returns the menu data structurally. Cost: one new verb in verbs.rs/main.rs/schema.rs and propagation through the help / JSON-mode parity tests. Aligns with the `spec.list` / `spec.show` neighbours.

**Recommendation.** **Option 3 — `orbit spec resolve`.** AC-04's "implementation lives wherever enforcement is strongest" reads directly as "in the CLI, not in prose". The existing pattern (`spec.list`, `spec.show`, `spec.close`) makes this the natural home. The skill-side change is a one-liner per skill that replaces the divergent branches; the resolver's three-step contract is unit-testable in `core/src/verbs.rs`.

### Decision 2 — How a card-binding resolves to a spec-id

**Context.** AC-01 reads: "Skill uses the bound spec when no argument is supplied — it uses the bound spec automatically". But `.session-card` holds a **card slug**, not a spec id (per `read_session_card` in session.rs:109 and the verb `session.set-card` at verbs.rs:4362). A card may have zero, one, or many open specs in its `specs[]` array. The resolver needs a deterministic rule for "given a bound card, which spec do I pick?".

**Options.**

1. **Single-open-spec-per-card rule.** If the bound card has exactly one open spec, use it. If zero or multiple, fall through to the prompt branch (which lists open specs scoped to the card, or all open specs if none under the card).
2. **Most-recent-open-spec-per-card rule.** If the bound card has any open specs, pick the most recently created (spec ids are date-prefixed, so this is a sort). Fall through only when the card has zero open specs.
3. **Bind a spec, not a card.** Introduce `.orbit/.session-spec` (or extend `.session-card` to carry both) and write a `session set-spec` verb. Inference reads the spec binding directly; the card binding is decoupled.

**Trade-offs.**

- (1) is the most conservative — multiple open specs on one card is genuinely ambiguous, and silently picking one risks the "skill auto-picked the wrong spec" failure mode AC-03 warns against. The cost is more prompt-branch traffic when a card has 2+ open specs.
- (2) is the most ergonomic — the agent's hot path almost always has one active spec — but it makes a silent choice when the assumption breaks. The "surface which spec it picked" clause in AC-01 partly compensates, but the agent has to read the surfaced output and intervene.
- (3) is the cleanest substrate but is a card-0036 amendment, not a 0038 implementation. It also splinters the binding surface — operators now manage two files. The card lists 0036 as `depends-on`, not as something to extend.

**Recommendation.** **Option 1 — single-open-spec-per-card.** Matches `implement/SKILL.md`'s existing single/zero/multi triage (lines 71-77) and gives AC-01 a deterministic semantics that AC-03's "halts only when both fallbacks fail" can sit on top of. When the card has multiple open specs, the resolver returns the scoped list to the prompt branch — the agent gets a card-narrowed menu, which is strictly better than today's project-wide menu. Defer option 3 to a future card if the prompt-branch traffic proves painful in practice.

### Decision 3 — Scope: which skills are "affected"

**Context.** AC-04 says "every affected skill" but doesn't enumerate. The codebase has 23 skills (`plugins/orb/skills/`); only some take a required contextual argument. Naming the set precisely matters for the disjointness check at Stage 4 and for the spec's AC-04 verification.

**Options.**

1. **Spec-id-consumer skills only.** `implement`, `review-pr`, `review-spec`, `audit`, `drive`. These are the skills where `$ARGUMENTS` is a spec-id-or-equivalent and the missing-arg branch is the failure mode the card describes.
2. **Spec-id-consumers + arg-taking skills with prompt fallbacks.** Adds `memo` (slug), `card` (topic), `distill` (scope), `design` (card-or-topic). These already partially infer or prompt but don't share a contract.
3. **Every skill with any optional arg.** Includes `rally`, `setup`, `discovery`, etc. Broadest interpretation.

**Trade-offs.**

- (1) keeps the card's "spec-id" framing tight and aligns with the load-bearing substrate (`.session-card` → spec resolution). The other arg shapes (slugs, topics, scopes) need different inference sources, not the same resolver.
- (2) is consistent in shape but mixes resolution domains. A "memo slug" can't be inferred from `.session-card`; the inference branch collapses to "ask the user" — which is just AskUserQuestion, not the three-step recovery.
- (3) over-fits. Most non-spec-id args don't have a `.session-*` binding to infer from and don't have an open-list to enumerate; AC-01 and AC-02 don't apply.

**Recommendation.** **Option 1 — spec-id consumers only.** The five named skills (`implement`, `review-pr`, `review-spec`, `audit`, `drive`) share the same input shape and the same substrate (`spec list --status open` + `.session-card` → card → specs). Other arg-taking skills (memo, card, distill, design) are a follow-up card if the "infer or prompt before halt" discipline generalises — but the card's goal text ("typically a spec-id") signals spec-id consumers are the intended set.

### Decision 4 — Prompt surface: AskUserQuestion shape

**Context.** AC-02 says "the skill lists the open specs as a single choice (one AskUserQuestion), uses the selected spec, and proceeds — without a multi-step research expedition". The decision is the exact shape of that AskUserQuestion call — what the choices look like, whether the spec goal is included, and what the cancel/abort branch does.

**Options.**

1. **Bare spec-id list.** Each choice is `<spec-id>` literally. Minimal noise, but the agent / operator has to recognise specs by date-slug alone.
2. **Spec-id + goal one-liner.** Each choice is `<spec-id> — <first line of goal>`. Costs one extra `spec show` per candidate (or batched via `spec list`), but the menu is self-describing.
3. **Spec-id + status + age.** Adds `(opened YYYY-MM-DD)` or `(N ACs unchecked)` to give recency / progress signals. Heavier; arguably leaks substrate state into the prompt.

**Trade-offs.**

- (1) is the smallest payload and the fastest implementation, but operators in a many-spec project (or agents resuming a stale session) will struggle to pick the right one — defeating "one round-trip resolves what would otherwise force me to re-read docs".
- (2) is the highest-leverage middle ground. `spec list` can return goals inline (one extra field), so no per-candidate `spec show` is needed. The menu describes itself; the agent's decision is visible to the human reading the prompt afterwards.
- (3) over-specifies. Status is implicit (we're listing open specs only). Age signals are noisy in this codebase where every spec is dated within the last few weeks.

**Recommendation.** **Option 2 — spec-id + goal one-liner.** Extend `spec.list`'s response schema to include the spec's `goal` (or a truncated `goal_first_line`) so the resolver returns prompt-ready labels. Per-skill prompt code stays a single AskUserQuestion call. The cancel branch (operator dismisses the menu) maps to AC-03's halt path — the skill exits with the same "no spec to act on" message as the empty-list case.

### Decision 5 — Halt message contract

**Context.** AC-03 demands "halts with a clear 'no spec to act on' message — not a silent stop the user can mistake for a broken skill". Today the messages differ: `review-pr` says `no spec-id provided — review-pr requires a spec-id under the orbit-state substrate`, `review-spec` says the same, `implement` says `No open spec. Create one with /orb:spec or pick an existing spec with orbit spec list.`, `drive` says "halt with usage". The contract needs unifying.

**Options.**

1. **Single message template, parameterised by skill name.** `no spec to act on for /orb:<skill> — both fallbacks failed (.session-card is unbound and no open specs exist). Create one with /orb:spec or set a binding with orbit session set-card <id>.`
2. **Two-tier messages: terminal vs recoverable.** Distinguish "no bound spec and no open specs" (terminal — needs `/orb:spec`) from "bound card has no open specs" (recoverable — points at the bound card and suggests creating a spec under it).
3. **Skill-specific messages preserved.** Keep today's per-skill prose. The "clarity" AC-03 demands is met by the unified prose contract, not by literal text identity.

**Trade-offs.**

- (1) gives AC-04's uniformity test something concrete to grep against. One message, one test. Mild loss of per-skill nuance.
- (2) is more informative — the recoverable case is genuinely different from the terminal case and deserves a different next-step. Costs two templates and a branch in the resolver.
- (3) preserves today's variance and makes AC-04 harder to verify mechanically.

**Recommendation.** **Option 2 — two-tier messages.** AC-03's "clear" plus AC-01's "uses the bound spec" together imply the user cares about the bound-card-with-no-open-specs case. The two templates land in the resolver alongside Decision 1's CLI verb and are reused verbatim across all affected skills. Verification: grep that no skill emits a halt message other than the two canonical templates.

## Disjointness note (informational, for Stage 4)

Files / symbols this card's implementation touches:

- `orbit-state/crates/core/src/verbs.rs` — new `spec_resolve` verb alongside `spec_list` / `spec_show`. New args/result types in the same file.
- `orbit-state/crates/core/src/schema.rs` — possible new `goal_first_line` field on the `SpecListItem` schema if Decision 4 lands as recommended.
- `orbit-state/crates/cli/src/main.rs` — wire the new verb through the `spec resolve` subcommand.
- `plugins/orb/skills/implement/SKILL.md` — replace §"Input contract" branches with `orbit spec resolve` call.
- `plugins/orb/skills/review-pr/SKILL.md` — replace §1 spec-id hard-stop with `orbit spec resolve`.
- `plugins/orb/skills/review-spec/SKILL.md` — same.
- `plugins/orb/skills/audit/SKILL.md` — same.
- `plugins/orb/skills/drive/SKILL.md` — replace §"Input contract" branches with `orbit spec resolve` (preserving the card-path branch and drive-sidecar filter as a wrapper).

Sibling cards 0037 (memory-gates-decisions) and 0042 (act-when-authorised) edit different surfaces — memory matching / design-skill prose for 0037, drive/rally autonomy contract prose for 0042 — so parallel execution looks clean. The one shared file is `drive/SKILL.md`: 0042 edits the autonomy / halt-temptation prose while 0038 edits the §"Input contract" block. If both land simultaneously, sequence 0038 first (input-contract is upstream of the autonomy contract) or accept a small merge cost.
