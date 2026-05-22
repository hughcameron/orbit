# Spec Review

**Date:** 2026-05-22
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-22-routine-proposals
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | content signals (cross-system substrate, write path, audit integration); Pass 1 findings ≥ MEDIUM | 3 |
| 3 — Adversarial | not triggered | — |

Spec resolved via `orbit --json spec resolve --skill review-spec` returned `outcome=prompt` with two candidates — the inline brief named `2026-05-22-routine-proposals` so the prompt round-trip was skipped. `drive.yaml` confirms `review_spec_cycle: 0` (first review on this spec). Default sidecar path used per drive convention.

Gate-AC description check (Pass 1 deterministic rule): all five ACs report `gate=false`; rule does not apply.

## Findings

### [HIGH] Substrate adequacy assumed, not proved
**Category:** assumption
**Pass:** 2
**Description:** The spec assumes the existing `SkillInvocation` JSONL stream (`.orbit/skills/<skill_id>.invocations.jsonl`) is sufficient to detect recurring *chains* of skill invocations. It isn't, on its current shape. The struct (`orbit-state/crates/core/src/schema.rs:418`) records one row per *manually recorded outcome* (`worked` / `partial` / `didnt-apply` / `incorrect`), written by the agent via `orbit skill record-invocation`. There is no row-per-invocation log of which skills ran in what order within a session — the substrate captures *correctional* events, not invocation sequences. AC-01 ("agent surfaces a routine proposal on recurring chain") presupposes the agent can reconstruct chains; nothing in the spec commits the implementing agent to verify that presupposition first or define what counts as "recurrence evidence" in concrete substrate terms.

The tabletop's escalation trigger #5 names this exact risk ("substrate-shape question for chain detection"), but escalation triggers are runtime safety nets — they fire *after* the implementing agent has discovered the gap. None of the ACs prove the substrate question is answered, and none of them gate on a chain-detection mechanism existing.

**Evidence:**
- `orbit-state/crates/core/src/schema.rs:418-432` — `SkillInvocation` shape (no `position`, no `sequence_id`, no per-call log).
- `.orbit/conventions/skill-self-improvement.md:36-46` — convention scopes rows to ≥2 same-skill, same-outcome recurrences; written *on outcome*, not *on every invocation*.
- Tabletop escalation trigger #5 (lines 66-69 of `tabletop.md`) — names the substrate gap as the most likely halt condition.
- Spec ACs 01-05 — none names "chain detection mechanism exists" or "SkillInvocation extended" as a closure condition.

**Recommendation:** Add an explicit AC (e.g. `ac-00`) that gates closure on a documented chain-detection substrate. Two acceptable shapes:
1. The substrate is extended (new field on `SkillInvocation` capturing per-invocation position/sequence, or a new aggregator verb that reconstructs chains from existing rows + timestamps + sessions) — name which, with a passing test or referenced commit.
2. The implementing agent confirms that existing substrate suffices and the AC closes on a passing chain-detection unit test against a JSONL fixture.

Either resolves the assumption. Leaving it implicit guarantees the spec hits escalation trigger #5 on first run.

### [HIGH] No AC for the verification mechanism (`last_verified` / audit-driven freshness)
**Category:** missing-requirement
**Pass:** 2
**Description:** The tabletop walks `last_verified` extensively (Implementation Notes; halt condition 2; kill condition K5 — "Freshness signal reliability"). The mechanism is treated as load-bearing for correctness: a routine without freshness verification is a drift trap, and K5 explicitly says "post-ship observation that audit doesn't flag staleness within 30 days" kills the whole approach. Yet none of the five ACs require the verification mechanism to ship — no AC for the audit finding family, no AC for the `last_verified` field write, no AC for the conformance integration. The spec promises a halt condition and a kill condition for a mechanism the spec doesn't commit to building.

**Evidence:**
- `tabletop.md:42-44` (halt 2 — "Routine drift undetected"), `tabletop.md:91-94` (K5), `tabletop.md:100` (Implementation Notes on `last_verified`), `tabletop.md:104` (AC types — "Routine drift soak = observation (defers)").
- `spec.yaml` ACs 01-05 — no mention of `last_verified`, no `observation` AC for drift soak, no AC for audit integration.

**Recommendation:** Add two ACs:
- `code` AC: routine SKILL.md write path records `last_verified` from a successful audit cross-check (existence + non-retirement of every `/orb:<verb>` referenced in the body).
- `code` AC: `orbit audit conformance` emits a finding family for routines whose `last_verified` is older than the configured threshold OR whose referenced skills no longer resolve.

The `observation` (deferring) AC for post-ship drift soak is optional but consistent with what the tabletop scoped.

### [MEDIUM] Author lever (approve / edit / reject) underspecified at the substrate
**Category:** test-gap
**Pass:** 2
**Description:** AC-04 names the author's lever but the simplest-cut tabletop pattern (`tabletop.md:25`) inverted it — "agent writes `.claude/skills/<name>/SKILL.md` directly … author sees on commit, edits inline, or archives via curator. No separate proposal artefact." AC-01, by contrast, says "it surfaces a proposal artefact … and an approve / edit / reject prompt." The spec doesn't say which mechanism wins:
- AC-01 implies a proposal artefact + interactive author prompt (heavy).
- AC-04 says "approve scaffolds the routine skill or extends an existing meta-skill" which reads more like artefact-first.
- Tabletop simplest-cut says no proposal artefact — write directly, surface via commit message.

If the simplest cut wins, AC-04's "reject discards the proposal" is meaningless (there's no proposal to discard, only an already-shipped SKILL.md to archive). If the proposal-artefact path wins, the tabletop's simplest-cut analysis was rejected without being marked as such.

This is also untestable as written: "agent does not re-surface the same chain in subsequent sessions" needs concrete state — where is the rejection recorded? A `.orbit/skills/<id>.rejected` marker? An entry in the SkillInvocation stream with a new outcome? Without a named substrate, the AC can't be verified.

**Evidence:**
- `spec.yaml` ac-01 ("proposal artefact … approve / edit / reject prompt") vs ac-04 ("approve scaffolds the routine skill") vs `tabletop.md:25` ("No separate proposal artefact").
- No substrate is named for storing rejected chains.

**Recommendation:** Pick one path (artefact-first vs commit-first) and update both ACs to agree. If commit-first (the tabletop's preferred simplest cut), AC-04's "reject" branch needs a named persistence substrate so a future session can know "this chain was already rejected" — e.g. an entry in a new `.orbit/skills/rejected.jsonl` keyed by chain hash, with a closing AC that the chain-detection mechanism consults it.

### [MEDIUM] AC descriptions embed the scenario's "given" clause as prose, producing untestable mixed statements
**Category:** test-gap
**Pass:** 1
**Description:** Each AC reads as a free-form paragraph that fuses the precondition, the action, and the expected result. AC-05 illustrates the problem in pure form: its description opens "Threshold mirrors the failure-routing convention — it does not — …" — the "it does not" is a leftover from the scenario template's `then` clause being concatenated against the scenario's `name`. Read literally, the AC's first verifiable claim is that the threshold *does not* mirror the convention; read in context, the claim is the opposite. This isn't a content bug (the threshold *is* ≥2, consistent with the convention) but the AC text is not what you'd write a test against.

The same shape repeats in ACs 01-04 to a lesser degree — each AC is one long sentence with em-dashes serving as both clause separators and clause connectives. For a reviewer or implementer asking "what passing artefact closes this AC," the answer requires re-parsing the prose every time.

**Evidence:**
- `spec.yaml:24-27` ac-05 — opens with a self-negating clause.
- `spec.yaml:8-23` ac-01 through ac-04 — each is one sentence with three or more em-dash-separated clauses.

**Recommendation:** Re-shape each AC into the standard `given … when … then` triple, with the `then` clause being a single declarative statement of what evidence closes it. The scenario fields on card 0013 already carry this shape — propagating them as separate fields (or at least cleaning up the concatenation) would produce ACs an implementer can drive a test against directly. At minimum, fix AC-05's "it does not" leftover so the literal reading matches the intended claim.

### [MEDIUM] AC-types not assigned (`ac_type` field absent)
**Category:** missing-requirement
**Pass:** 1
**Description:** The tabletop Implementation Notes (`tabletop.md:104`) explicitly maps work to AC types per card 0035: "Detection + write path + audit integration = `code`. Front-matter convention compliance = `config`. Routine drift soak = `observation` (defers)." The spec's ACs carry no `ac_type` field, so all five default to `code` — `spec.close` will block on every unchecked AC. The tabletop already noted that drift soak should be `observation` (defers); shipping without the types means the spec can't close at the boundary the tabletop intended.

**Evidence:**
- `tabletop.md:104` — explicit type assignment.
- `.orbit/METHOD.md` — describes the `ac_type` field and the `code` default.
- `spec.yaml` — no `ac_type` on any AC.

**Recommendation:** Add `ac_type:` to each AC per the tabletop assignment. If a drift-soak AC is added per Finding 2, mark it `observation`.

### [LOW] Cross-card scope expansion (card 0022 amendment) not committed either way
**Category:** constraint-conflict
**Pass:** 2
**Description:** Tabletop escalation trigger #4 ("Cross-card scope expansion") flags that implementing this spec may require amending card 0022's front-matter convention to add `last_verified`. The tabletop offers three branches (extend 0022 inline / keep 0013-only / defer to v2) but the spec doesn't pre-commit to one. The implementing agent will hit this on first AC and either escalate (costing a round-trip) or pick unilaterally (potentially diverging from author intent). Cheap to fix at spec-time, expensive at implement-time.

**Evidence:**
- `tabletop.md:62-65` (escalation trigger #4).
- `spec.yaml` — no constraint or note pre-committing the branch.

**Recommendation:** Either add a constraint to the spec ("`last_verified` lives on the routine SKILL.md front-matter local to this card; card 0022 is not amended in this spec") or split the 0022 amendment as a precursor spec. Either resolves the ambiguity before implementation starts.

---

## Honest Assessment

The tabletop is unusually rigorous — values, trade-offs, halt conditions, escalation triggers, kill conditions, all walked and confirmed via AUQ. The spec is the weak link. It captures the card's scenarios faithfully but doesn't translate the tabletop's mechanism choices into concrete, type-tagged, substrate-aware ACs. As written, the spec is open on three of the load-bearing decisions the tabletop *thought* it had settled: which substrate makes chain detection possible (escalation trigger #5 territory), whether the verification mechanism is in-scope (K5 territory), and which author-interaction path wins (simplest cut vs proposal-artefact).

The biggest risk is the implementing agent reading the spec, starting with AC-01, immediately hitting the "where does chain evidence come from" gap, and either escalating (one round-trip cost) or guessing (silent drift from tabletop intent). All five findings are cheap to fix at spec-rewrite time and expensive to discover at implement-time. Returning the spec for revision is the right call now.
