# Decision Pack — 2026-05-19-memory-gates-decisions

**Card:** `.orbit/cards/0037-memory-gates-decisions.yaml`
**Spec:** `2026-05-19-memory-gates-decisions`
**Date:** 2026-05-19

This pack frames the 5 design decisions the card's ACs force. The card already
declares the *what* (memories surface at decision moments, spec records what
was considered, spec.close blocks on unreconciled memories, memory shape
favours mechanism over state, behavioural reminders alone are insufficient).
The decisions below are the *how* — they pick the substrate surfaces, the
matching semantics, and the enforcement seams.

Rally context: sibling specs `2026-05-19-skills-infer-or-prompt-before-halt`
and `2026-05-19-act-when-authorised` also live under the same rally. The
disjointness analysis at the end of this pack names every file/symbol this
spec proposes touching so the rally lead can decide which drives serialise.

---

## D1 — Matching surface: extend `memory.search` vs add a new `memory.match` verb

### Context
ac-01 and ac-02 require a callable surface that returns "memories matching
the current work" — both at design time (inside `/orb:design`) and inline
mid-conversation. Today `orbit memory search <query>` exists
(`MemorySearchArgs { query: String }` at `verbs.rs:324`) and does
case-insensitive substring over body + labels (`verbs.rs:2417`). Substring is
crude — false negatives on synonyms ("halt" vs "stop"), false positives on
common words. The question is whether to upgrade `memory.search`'s ranking
or introduce a sibling verb scoped to the decision-moment use case.

### Options
- **D1a — Extend `memory.search`**: keep one verb, add an optional `topic` /
  `context` arg that accepts a richer signal (a card slug, a spec id, free
  text describing the proposed approach). Return ranked matches with an
  overlap score so callers can threshold.
- **D1b — Add `memory.match`**: introduce a new verb (`memory.match` /
  `MemoryMatchArgs`) explicitly framed as "surface memories relevant to this
  decision". Keep `memory.search` as the operator-facing keyword tool.
  Implement matching as a thin wrapper today (still substring + label
  overlap) but reserve the namespace for stronger ranking later.
- **D1c — Reuse session.prime's overlap heuristic**: the prime path already
  ranks memories by label-overlap with open-spec labels
  (`verbs.rs:4172-4181`). Expose that same scoring as a query verb taking
  arbitrary labels/tokens, no new struct beyond a `tags: Vec<String>` arg
  added to `MemorySearchArgs`.

### Trade-offs
- D1a is the smallest API expansion but conflates two use cases under one
  name. Operators using `orbit memory search foo` for keyword discovery get
  different ranking behaviour than they expect; the verb has to overload.
- D1b is the cleanest semantic split and matches the card's vocabulary
  (the scenario is literally named "Inline memory match"). Cost: one extra
  verb to canonicalise, document, and version-pin. Gain: future ranking
  work (embeddings, label-DAG walking) lands behind a stable name without
  re-shaping the operator-facing verb.
- D1c is the smallest implementation but ties decision-moment matching to
  the prime-relevance heuristic, which is already labelled "v1 — cheap
  heuristic, no LLM classification" in spec 2026-05-15-agent-learning-loop
  ac-06. The card's ac-06 explicitly demands a *callable surface*; making
  it a `search --tags` variant works mechanically but blurs the semantic.

### Recommendation
**D1b — add `memory.match`.** The card uses the word "match" in two of its
gate scenarios and contrasts it with the existing prime auto-injection. The
distinct verb cleanly absorbs future ranking improvements without disturbing
`memory.search`'s operator contract. Concrete shape:

```rust
#[derive(Serialize, Deserialize)]
pub struct MemoryMatchArgs {
    pub topic: String,                // free text / card slug / approach snippet
    #[serde(default)] pub labels: Vec<String>,   // optional label overlap hint
    #[serde(default = "default_match_limit")] pub limit: usize, // default 10
}
pub struct MemoryMatchResult {
    pub matches: Vec<MemoryMatch>,    // ranked
}
pub struct MemoryMatch {
    pub memory: Memory,
    pub score: f32,                   // 0.0..=1.0 — token + label overlap
    pub reason: String,               // short phrase: "label overlap on 'drive'"
}
```

The v1 ranker is `(token-overlap on body) + 2 * (label-overlap on labels)`,
normalised. This stays cheap, files-canonical, and inspectable in `git
diff` of the verb output. Evidence: prime already uses label overlap and
shipped behind ac-06 of agent-learning-loop with no scaling problem.

---

## D2 — Where the design-time surfacing lives (ac-01)

### Context
ac-01 requires matching memories to surface inside `/orb:design`
"before the agent commits to an approach". The design SKILL.md already has
a §2 "Load the Evidence Base" step that explicitly reads "Check `.orbit/memos/`
for related memos" (SKILL.md:63) but does NOT currently read memories from
the substrate. The mechanism choice is: SKILL.md prose instruction, a new
CLI verb the skill calls, or a substrate-emitted block on a prime-like verb
the skill already calls.

### Options
- **D2a — SKILL.md prose only**: add a step "run `orbit memory match
  <card-slug>` and surface the results in the §2 evidence load" to
  `plugins/orb/skills/design/SKILL.md`. No new verb beyond D1.
- **D2b — New `design.prime` verb (or `card.prime`)**: substrate-side
  composite verb that the design skill calls — given a card slug, it
  returns prior specs (already in `CardSpecsResult`), prior memos (via a
  new `memo.search` shape), and matching memories. One call, structured
  output.
- **D2c — Add `matching_memories` to `card.show` / `card.specs`**: extend
  an existing verb's response with the matched-memory block. The skill
  already calls these; no new round trip.

### Trade-offs
- D2a is fast to ship and stays consistent with how the skill currently
  loads evidence (prose-described commands). Risk: the card's ac-06 calls
  out that "skill-prompt-only enforcement is insufficient" — D2a alone
  satisfies the literal AC (the memories *are* surfaced when the skill is
  followed) but exposes the same anti-pattern the card warns against. The
  surfacing happens, but only because the skill remembered to run the
  command.
- D2b creates a clean composite but adds a verb whose only caller is one
  skill. The composition isn't reusable elsewhere; the cost-to-value ratio
  is poor.
- D2c is the most ergonomic for the skill (one verb, one structured
  block, no extra plumbing) and is precedented — `session.prime` already
  composes specs + memories + handover + topology_drift into a single
  envelope (`SessionPrimeResult` at `verbs.rs:4217-4224`). Cost: `card.show`
  / `card.specs` are read-only "just give me the card" verbs today; adding
  a contextual `matching_memories` block widens their contract.

### Recommendation
**D2a + a thin enforcement seam.** Update `plugins/orb/skills/design/SKILL.md`
§2 to call `orbit memory match <card-slug>` as a non-optional step (sits
alongside the existing prior-specs and references reads). The
*structural* gate fires at D4 below (spec.close block on
`memories_considered`), so the design-time call is best-effort: if the
agent skips it, the spec still cannot close. Treating the design-time
surfacing as the *encouragement* and the close-time block as the
*enforcement* matches the card's ac-06 contract directly — a callable
surface (memory.match, from D1) plus a structural gate (spec.close
block, D4), not behavioural reminders alone.

D2b is over-engineering for one caller. D2c contaminates `card.show`'s
read-only shape. Both are rejected.

---

## D3 — Where `memories_considered` lives on the spec (ac-03)

### Context
ac-03 requires the spec to record which memories were considered, with
adoption status per memory (adopted / partially adopted / NA, with
reason). The current `Spec` struct (`schema.rs:211`) has six fields:
`id, goal, cards, status, labels, acceptance_criteria`. Adding a new
field requires extending `Spec::FIELDS` (`schema.rs:56`) and the
`spec_fields_matches_struct` drift test.

### Options
- **D3a — Top-level `memories_considered: Vec<MemoryReconciliation>`**:
  add a sibling field to `acceptance_criteria`. Each entry names the
  memory key, the adoption status (enum: `Adopted`, `PartiallyAdopted`,
  `NotApplicable`), and a reason string.
- **D3b — Per-AC `memories_considered`**: hang the reconciliation off
  individual `AcceptanceCriterion` entries. Each AC's reconciled
  memories travel with the AC; closure of an AC implies reconciliation
  for *its* memories.
- **D3c — Sidecar file**: `.orbit/specs/<id>/memories.yaml` — separate
  artefact that the spec doesn't directly contain. `spec.close` reads it
  and enforces the block.

### Trade-offs
- D3a is the closest fit to the card's prose ("a `memories_considered`
  field"). The field shape is uniform across the spec; reconciliation
  is a spec-level concept, not an AC-level one. One round-trip test, one
  drift-test entry. Cost: every spec gains one optional field even when
  no memories matched.
- D3b distributes reconciliation across ACs, which would be appropriate
  if memories were AC-scoped — but the card frames reconciliation as
  spec-scoped ("the spec must reconcile each matching memory before it
  can close"). Wrong granularity.
- D3c separates concerns but adds a new file shape, a new round-trip
  test, and a new drift check. The benefit (the spec stays small) is
  weak because the `memories_considered` field is opt-in and skip-on-
  default.

### Recommendation
**D3a — top-level `memories_considered` on `Spec`.** Concrete shape:

```rust
pub struct Spec {
    pub id: String,
    pub goal: String,
    pub cards: Vec<String>,
    pub status: SpecStatus,
    pub labels: Vec<String>,
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memories_considered: Vec<MemoryReconciliation>,  // NEW
}

#[derive(Serialize, Deserialize)]
pub struct MemoryReconciliation {
    pub key: String,                         // memory key (e.g., "drive-autonomy-default-to-action")
    pub disposition: ReconciliationDisposition,
    pub reason: String,                      // short — one sentence
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReconciliationDisposition {
    Adopted,
    PartiallyAdopted,
    NotApplicable,
}
```

`#[serde(default, skip_serializing_if = "Vec::is_empty")]` keeps
existing specs byte-identical on disk — the field only materialises
when memories were actually considered. Same skip-on-default discipline
as `AcType::is_code` (`schema.rs:258`).

Extend `Spec::FIELDS` to include `"memories_considered"` and the
`spec_fields_matches_struct` test fixture (per the drift-gate pattern
at `schema.rs:740-768`). Schema-version bumps from current to next
minor; migration is structural no-op (additive field).

---

## D4 — How `spec.close` decides "memories are unreconciled" (ac-04)

### Context
ac-04 requires `spec.close` to refuse closure when memories match the
spec but `memories_considered` doesn't cover them. The matching set has
to be computed *at close time* from a topic signal. The question is:
what topic does spec.close match against, and how does it decide a memory
is "covered"?

### Options
- **D4a — Match by spec goal + card slugs**: build the match query from
  `spec.goal` (free text) plus `spec.cards` (slug list, used as label
  hints). Memory is "covered" iff its key appears in
  `memories_considered` regardless of disposition.
- **D4b — Match by labels only**: compute the matching set from labels
  that overlap between the spec's `cards`/`labels` and the memories'
  `labels`. Tighter signal, fewer false positives.
- **D4c — Spec declares its own matching scope**: spec.close runs against
  whatever the agent declared at design time — the spec stores a
  `match_query: String` field alongside `memories_considered`. The agent
  who shaped the spec picks the scope.

### Trade-offs
- D4a is automatic and matches the card's intent ("every memory matching
  active work"). Risk: `spec.goal` is a long free-text paragraph; token
  overlap will surface noisy matches. Mitigated by D1b's ranker
  (label-overlap is weighted higher than body-overlap).
- D4b is precise but brittle — it relies on memories carrying
  card-aligned labels. Memories recorded under skill or topic labels
  (`drive`, `code-investigate`, `topology`) won't surface for a spec
  whose card has different labels. The card's ac-01 demands
  "matching memories" — labels alone are a partial signal.
- D4c puts the agent in charge but reintroduces the failure mode the
  card warns about — the agent who skipped checking memories at design
  time is the same agent who'd leave `match_query` empty.

### Recommendation
**D4a — match by `spec.goal + spec.cards` using the D1b ranker, with a
score threshold (default 0.3) below which a memory is not "matching".**
Concrete spec.close additions:

1. After the AC pre-flight (currently `verbs.rs:1597-1642`) and BEFORE
   the unfinished-tasks check, call the same matching primitive as
   `memory.match` (D1b) with `topic = spec.goal`, `labels = spec.cards`.
2. Filter results to `score >= MEMORY_MATCH_THRESHOLD` (constant in
   the verbs module, default `0.3` — tunable).
3. Build the unreconciled set: matching memory keys that don't appear
   in `spec.memories_considered`.
4. If unreconciled is non-empty AND `--force` is not passed, return
   `Error::conflict` naming the unreconciled keys, mirroring the
   existing AC-blocking message at `verbs.rs:1632-1641`.
5. `--force` bypasses (same affordance as the AC pre-flight) and is
   recorded in the response payload (`forced_unreconciled: Vec<String>`,
   parallel to `forced_unchecked` at `:1643`).

This wires the gate into the same control surface as the existing AC
pre-flight — single code path, single test pattern, single response
field. The threshold being a constant (not a per-spec setting) keeps
the policy uniform across the project; tuning is a substrate change,
not a per-spec choice the agent could fudge.

---

## D5 — Mechanism-over-state memory shape enforcement (ac-05)

### Context
ac-05 says memories should lead with mechanism ("use X for Y") not
state ("Y is hard"). This is a *content quality* constraint, not a
schema constraint — the `Memory` struct (`schema.rs:332` area) has
`{key, body, timestamp, labels}` and `body` is opaque to the substrate.
The question is whether enforcement is mechanical (lint at write-time)
or behavioural (skill prose + audit).

### Options
- **D5a — Lint at `memory.remember`**: reject bodies that match a state-
  shape regex (leading "X is", "Y is hard", "the problem is", etc.) and
  return `Error::malformed` with a suggested rewrite. Strict.
- **D5b — Warn at `memory.remember`**: same detection, but return a
  warning string on the result envelope (mirrors the topology-label
  nudge at `verbs.rs:2401-2405`) and store anyway. Soft.
- **D5c — Audit-only**: add a `mechanism-shape` finding to `audit
  conformance`'s envelope. The substrate never blocks; the audit
  surfaces non-conforming memories as a finding the agent can address.
  No write-time check.

### Trade-offs
- D5a is the strongest enforcement but the most false-positive-prone.
  A regex on natural-language openings will reject legitimate memories
  ("FineType is uv-based" is mechanism-shaped despite "is"). Hugh has
  to fight the linter; agents work around it. Likely net-negative.
- D5b is the topology-nudge precedent — already-shipped pattern, already-
  tested skip-on-default discipline, doesn't block the write but raises
  the signal. The agent sees the warning *at the moment they wrote the
  memory* and can rephrase. Low false-positive cost: the memory still
  stored.
- D5c is the loosest. The nudge fires later, possibly never if the audit
  isn't run. Doesn't connect the agent to the moment they wrote the
  problem memory.

### Recommendation
**D5b — warn at `memory.remember`, mirroring the topology nudge.**
Concrete shape: extend `MemoryRememberResult` with an optional
`shape_warning: Option<String>` (parallel to `nudge`). Detection is a
small heuristic on the body's first sentence — leading
state-verb patterns ("X is …", "the problem is …", "Y proved
difficult") fire the warning with a suggested rephrase:

> "memory body leads with state ('X is …'); decision-moment surfacing
>  works better when the body leads with mechanism ('use X for Y',
>  'prefer X when Y'). Consider rephrasing — the memory is stored as
>  written."

A `--no-warn` flag (parallel to `--no-nudge` at
`MemoryRememberArgs.no_nudge`) suppresses for the cases where the
agent knows better. The heuristic regex is tunable in one place; false
positives don't block the write.

This satisfies ac-05's intent ("favours mechanism over state") without
introducing the false-positive failure mode of strict rejection. The
topology-nudge precedent is the proof-of-concept that nudge-shaped
warnings change agent behaviour without blocking it.

---

## Disjointness map (read by rally lead)

This spec proposes touching the following files and symbols. Sibling
specs in the same rally should compare against this list to decide
whether to serialise:

**Substrate (Rust):**
- `orbit-state/crates/core/src/verbs.rs`
  - NEW: `memory_match` fn, `MemoryMatchArgs`, `MemoryMatchResult`,
    `MemoryMatch` (D1b)
  - NEW: variant `VerbRequest::MemoryMatch`, `VerbResponse::MemoryMatch`
  - MODIFY: `spec_close` — add memory-reconciliation block before
    unfinished-tasks check (D4)
  - MODIFY: `memory_remember` — add `shape_warning` heuristic (D5b)
  - NEW: constant `MEMORY_MATCH_THRESHOLD`
- `orbit-state/crates/core/src/schema.rs`
  - MODIFY: `Spec` struct — add `memories_considered` field (D3a)
  - MODIFY: `Spec::FIELDS` const — add `"memories_considered"`
  - MODIFY: `spec_fields_matches_struct` test fixture
  - NEW: `MemoryReconciliation`, `ReconciliationDisposition`
  - MODIFY: `MemoryRememberArgs` — add `no_warn: bool` (D5b)
  - MODIFY: `MemoryRememberResult` — add
    `shape_warning: Option<String>` (D5b)
- `orbit-state/crates/cli/src/main.rs` — wire `memory match` subcommand
- `orbit-state/crates/mcp/src/main.rs` — register `memory.match` verb
  (search by "memory.search" registration at `main.rs:330`)
- `.orbit/schema-version` — bump minor (additive: `memories_considered`)

**Skill prose:**
- `plugins/orb/skills/design/SKILL.md` — §2 evidence-load: add
  `orbit memory match <card-slug>` call (D2a)
- `plugins/orb/skills/spec/SKILL.md` — add §X "Record memories
  considered" (uses D3a field)

**Documentation:**
- `plugins/orb/PRIME.md` — add `orbit memory match` to the Decisions
  section alongside `memory search` / `memory remember`

**Sibling cards in same rally:**
- **0038 (skills-infer-or-prompt-before-halt)** — its likely surface is a
  shared CLI resolver for missing-arg recovery. No file overlap with
  this spec.
- **0042 (act-when-authorised)** — its likely surface is `/orb:drive`
  pre-halt instrumentation and the drive autonomy contract. ac-05 of
  0042 ("memory + contract is sufficient authorisation") *reads* from
  the memory surface this spec defines but does not modify it. The
  callable surface (D1b `memory.match`) is consumed by 0042, not
  contested. Designs are disjoint at the file level; if 0042 takes a
  dependency on `memory.match`, ordering is: this spec ships first,
  then 0042.

---

## Open items the lead may want to surface

- D1b's score threshold is a single project-wide constant. Reasonable
  default is `0.3` but the value isn't load-bearing on ship; if the
  audit at session close shows the gate firing too noisily, the
  threshold bumps in a follow-up.
- D5b's state-shape heuristic — exact regex set is an implementation
  detail the implementing agent picks; this pack only fixes the
  contract (warn, don't block; suggested rephrase; `--no-warn`
  affordance).
- Card 0023 (memory-loop) sits upstream — the prime auto-injection still
  happens; this spec adds the decision-moment surfacing without
  changing prime. No conflict.
