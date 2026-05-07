# Tabletop — orbit-state v0.1 cluster

**Date:** 2026-05-07
**Facilitator + domain expert:** Hugh
**Scribe + driver:** Carson (manual execution; /orb:tabletop skill not yet on disk)
**Cards in scope:** 0020 (orbit-state), 0021 (tasks), 0008 (.orbit/ folder), 0017 (setup-is-bead-aware)
**Methodology:** Card 0019 — ten right questions; decision 0017 — output is contract, not solution
**Output:** orbit/specs/2026-05-07-orbit-state-v0.1/spec.yaml

---

## Cluster maturity at session start

```
Card                                       Maturity   Specs
0020  orbit-state                          planned    0
0021  tasks                                planned    0
0008  .orbit/ consolidated folder          planned    1  (pre-orbit-state spec, stale)
0017  /orb:setup is bead-aware             planned    0
```

The 0008 prior spec (2026-04-20-orbit-artefact-folder) predates decision 0015's substrate pick; treated as superseded by this tabletop's output.

---

## Cluster shape

```
0020 orbit-state          ← the substrate (Rust + SQLite + MCP, files canonical)
  ├─ feeds → 0021 tasks   ← session-scoped sub-spec units (append-only JSONL)
  └─ feeds → 0008 .orbit/ ← the directory layout the substrate writes into
                  ↑
                  └─ feeds → 0017 setup ← the bootstrap that creates it all
```

---

## Q1 — Goal narrowing

Hugh's pre-committed goal: ship orbit-state v0.1 — bd verb parity, single-repo only, dogfooded in orbit itself first. Cross-repo (`--global`) is v0.2+. 4 weeks. Halt if MVP scope creeps past bd verb parity.

Goal accepted as-stated; no narrowing required.

---

## Q2 — Values

**Question:** Among five candidate values from card 0020 (git-trackable, branch-aware, recoverable, inspectable, queryable at scale, format-integrity), rank-order or name the load-bearing one.

**Answer:** Format integrity — declared load-bearing because it makes the other values easier to reach.

**Captured reframe:** Format integrity is the SUBSTRATE value, not a peer of the others. Once it holds, the other four land naturally on top because every file is well-formed by construction. They become downstream consequences, not parallel goals.

---

## Q3 — Trade-offs

**Question:** With format integrity load-bearing, three trade-offs become live: (a) no raw-YAML escape hatch, (b) MCP server on critical path day one, (c) schema is a public API. Which are acceptable, expensive-but-worth-it, or halt-triggers?

**Answer:** All three acceptable. Schema-version migration runner ships in v0.1.

**Captured creative addition:** Hugh added the README early-release notice as the policy counterweight for trade-off (c):

> Early Release — Orbit is under active development. Expect breaking changes to schemas, CLI arguments, library APIs, and model formats between releases. Pin to a specific version if stability matters for your use case.

Pinned as v0.1 ship-blocker.

---

## Q3-extension — Verb surface (the simplest-way contract cut)

**Initial agent cut (rejected):** specs ≈ bd issues; cards/choices "untouched, file-only"; gate included; 22 verbs across 4 entity types.

**Hugh's reframes:**
1. Cards and choices need to be parsed/indexed/searched — they're load-bearing for all stages; specs_array is important for work completion
2. Tasks are the better analog for bd issues (specs have multiple task components — specs are containers, tasks are work units)
3. Gate not load-bearing; defer until clear use case

**Revised v0.1 verb surface:**

```
Entity     Verbs                                   Role
-----------------------------------------------------------------------------
tasks      ready, show, list, claim, update, done  work units (bd-issue analog)
specs      create, show, list, update, close, note contract containers
cards      show, list, search                      capability descriptions (indexed)
choices    show, list, search                      architectural decisions (indexed)
memories   remember, list, search                  persistent agent knowledge
session    prime                                   session-start
```

~22 verbs across 6 entity types.

**Pinned behaviours:**
- spec.close updates linked cards' specs_array as a serde side-effect (no separate verb)
- spec.close requires all child tasks done (no `--force`)

---

## Q3-second-extension — Self-learning & skill development scope

**Question (Hugh):** Does v0.1 cover self-learning & skill development?

**Answer:** Yes via existing entities; no new entity types.
- Self-learning: `memory.remember` / `memory.list` / `memory.search` (already in cut). Memory loop (card 0023, auto-injection) is v0.2+.
- Skill development: workflow on top of tasks via `label=skill-author`. Requires `labels: list[str]` schema field on tasks and specs. Curator stays a skill, reads SKILL.md frontmatter directly.

**Pinned schema additions:**
- `tasks.labels: list[str]`
- `specs.labels: list[str]`

---

## Q4 — What could go wrong (failure modes)

Seven themes enumerated. Three classified as halt-worthy (automatic stop), four as engineering hygiene (pin as ACs).

```
Theme                                             Class       Pinned as
1. Format-integrity bug locks every agent         halt        h-1, ac-01, ac-16
2. Concurrency race on same file                  hygiene     ac-03
3. Index drift between rebuild and incremental    hygiene     ac-02, ac-17
4. Migration locks the orbit repo                 halt        h-4, ac-12
5a. Scope creep during dogfood                    halt        h-5a
5b. Memo overflow (added by Hugh)                 halt        h-5b
6. Verb-surface mismatch with real workflow       hygiene     e-1
7. MCP ergonomics fail; agents revert to bash     hygiene     e-4
```

**Hugh amendment to Theme 5:** Use memos, not specs, for dogfood-discovered gaps (specs are heavyweight; memos are lightweight). New sub-failure-mode 5b: memo overflow risk if memos become a backup todo list. Mitigation: distillation cadence at 5–10 memos.

---

## Q5 — Lateral approaches considered

Three already considered and rejected in decision 0015 (Options A, B, the chosen C).

Three new laterals named:
- L1: Slim v0.1 (specs + tasks only) — held in reserve as Option C/D fallback
- L2: Fork bd, contribute upstream — rejected (Dolt removal makes it a rebuild)
- L3: Defer v0.1 entirely — rejected (reverses decisions 0014, 0015, 0017)

---

## Q6 — Success criteria

7 criteria pinned (see spec.yaml `success_criteria`). Each binary, measurable, traces to a value or trade-off.

**Four ambiguous things resolved:**
- (i) Dogfood window: 1 week
- (ii) "No fallback to bd": strict (any `bd X` invocation = criterion 4 failure)
- (iii) Publishing: internal-only for v0.1 (git tag, no GitHub release / crates.io)
- (iv) Multi-machine: both mac and beelink required (cross-compile mature; better to surface platform bugs in week 4 than week 6)

---

## Q7 — Escalation triggers

7 escalations pinned (see spec.yaml `escalation_triggers`). Each carries trigger + state snapshot + proposed action. "Ask Hugh, I'm stuck" is non-actionable and rejected.

E1 (verb-surface mismatch), E2 (schema field-type), E3 (malformed migration input), E4 (MCP ergonomics — week-1 only), E5 (awkward-middle gap), E6 (multi-machine divergence), E7 (distillation reveals scope change).

---

## Q8 — Adjacent code

Three layers touched:

**Layer 1 — Plugin skills:**
- 6 substantive rewrites: drive, implement, rally, review-spec, review-pr, audit
- 8 path-only updates: card, distill, memo, discovery, interviewer, spec, spec-architect, keyword-scan
- design/ skill stays (retired post-tabletop-skill ship, not in v0.1 cluster)
- release/ no-op for v0.1

**Layer 2 — Repo files:**
- CLAUDE.md (rewritten — bd → orbit verbs)
- README.md (early-release notice + install instructions)
- .gitignore + .orbit/.gitignore
- .beads/ retained during dogfood, removed after v0.1 ships clean (Hugh confirmed)

**Layer 3 — External tooling:**
- Claude Code MCP registration
- bd remains installed (other repos use it)
- rtk unaffected (MCP isn't shell)
- chezmoi distributes new MCP config + skill updates
- Other plugin installs unaffected (they continue with pre-orbit-state plugin until upgrade)

**Two distinct migrations pinned separately:**
- Migration A — Layout (orbit/ → .orbit/, decisions/ → choices/, MD → YAML)
- Migration B — Substrate (bd issues/notes/memories → orbit-state files)

**Decision (Hugh):** All 6 substantive skill rewrites ship in v0.1 (strict no-fallback policy bites).

---

## Q9 — Budget

**Initial agent estimate:** 33–43 working days (over the 4-week ~20-day budget). Recommendation: Option D (sequence as v0.1 series).

**Hugh pushback:** Agent estimates are ALWAYS overestimated.

**Recut at Claude-execution pace:** 13–17 working days. Comfortable inside 4 weeks. Confidence at 4 weeks jumps from ~30% to ~80–85%.

**Updated recommendation (accepted):** Option A — hold the line at 4 weeks, full enumerated scope.

**Captured contract addendum:**

> Estimate-inflation guard. Agent-authored effort estimates default toward conservative-engineering quotes, not Claude-execution reality. The contract uses recut estimates (13–17 days) as the planning baseline, not the inflated original (33–43). If the real burn rate trends toward the inflated number inside week 1, that's a Theme 5a halt-trigger condition (scope/architecture wrong, not execution slow).

---

## Q10 — Kill conditions

6 kills pinned (see spec.yaml `kill_conditions`). Each is the failure of one specific load-bearing claim, with a named pivot path.

```
K1 → format-integrity claim         (load-bearing value)
K2 → MCP-as-operator claim          (trade-off (b) acceptance)
K3 → bd-verb-parity claim           (the goal)
K4 → files-canonical claim          (decision 0015's architecture)
K5 → dogfood-in-orbit-self claim    (success criterion 4)
K6 → cluster premise                (strategic)
```

---

## Hot-wash debrief

Captured in `spec.yaml` under `hot_wash:` — recurred / surprised / friction / meta-patterns-for-future-tabletops.

Key meta-observations from Hugh:
- AskUserQuestion tool wasn't used; would suit closing picks (Q6 four ambiguous things, Q9 budget options) but would have foreclosed creative reframes at Q2/Q3/Q4. Hybrid pattern recommended (prose opens, AUQ closes).
- Tabletop methodology felt different from card 0026 (executive-communication) — confirmed: 0026 governs deliverables and closing recommendations; 0019 governs working sessions and opening questions.

---

## Session captures

- Memory: `agent-estimate-inflation-guard` — recut at Claude-execution pace before treating as planning input
- Memory: `tabletop-auq-hybrid-pattern` — AUQ closes forks, prose opens them
- Card 0019 amendment: one-paragraph note clarifying 0026 overlap
