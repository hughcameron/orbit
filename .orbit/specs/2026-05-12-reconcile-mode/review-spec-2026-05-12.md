# Spec Review

**Date:** 2026-05-12
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-12-reconcile-mode
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | content signals (schema migration, cross-system substrate) + Pass-1 MEDIUM findings | 4 |
| 3 — Adversarial | not triggered | — |

Pass 1 #5 deterministic gate-description check: all four gate ACs (ac-01, ac-03, ac-04, ac-08) have non-empty descriptions well above 20 characters and none match a placeholder token. The gate-description check passes.

---

## Findings

### [HIGH] AC-03 mirrors a dry-run shape that does not exist
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-03 says the `--reconcile --dry-run` JSON envelope "mirrors `orbit audit drift`'s existing dry-run shape". `orbit audit drift` has no dry-run mode. `AuditDriftArgs` is empty (`orbit-state/crates/core/src/verbs.rs:393`) and the verb walks the substrate unconditionally — every invocation is read-only and emits the same `{drift: [...]}` envelope regardless. There is no flag to mirror.
**Evidence:**
- `verbs.rs:392-394`: `#[serde(deny_unknown_fields)] pub struct AuditDriftArgs {}` — no fields.
- `verbs.rs:2333-2420`: `audit_drift()` body has no `dry_run` branch; result is one `DriftEntry` per unknown field.
- `cli/src/main.rs:809`: CLI wires `AuditAction::Drift` to `AuditDriftArgs::default()` with no flag passing.
**Recommendation:** Reword AC-03 to specify the envelope shape directly. Either (a) name the per-entry fields the dry-run output must carry (`path`, `kind`, `field`, `proposed_disposition`) and the top-level keys (`ok`, `dry_run`, `would_rewrite`, `entries`) — mirroring `run_canonicalise`'s hand-rolled JSON shape at `cli/src/main.rs:497-516`, not `audit drift`'s — or (b) drop the "mirrors" clause and say "the per-entry record reuses `DriftEntry`'s fields". `run_canonicalise`'s JSON shape is the closer precedent because it already has a `dry_run` flag.

### [MEDIUM] AC-02 leaves "map" disposition value-semantics unspecified
**Category:** test-gap
**Pass:** 1
**Description:** AC-02 says the registry maps "known legacy field names onto their canonical equivalents", but says nothing about what happens to the field's *value*. Cases the implementer will hit on day one: (1) rename only — `date_opened` → `date_created`, value passes through; (2) enum value shift — legacy `status: in_progress` mapped onto `status: open`; (3) value drop — `version: "0.0.3"` mapped onto nothing because the canonical Spec has no version field (already covered by "drop", but ambiguous if listed under "map"); (4) shape change — `predecessor_evidence` (string or list?) mapped onto a notes-array entry. The current registry contract `(EntityType, field_name) → Disposition` carries no value-transform function, so cases 2–4 are unreachable without extending the registry shape.
**Evidence:** Interview implementation notes §"default mapping registry is a Rust constant inside `reconcile.rs`, e.g. `pub const FIELD_RULES: &[(EntityType, &str, Disposition)] = &[...]`" — no value-transform slot. The five seed fields listed in AC-05 (`version`, `date_opened`, `predecessor_evidence`, `constraints`, `exit_conditions`) include at least two (`date_opened` rename, `predecessor_evidence` shape) that need value-level handling.
**Recommendation:** Extend the AC to name the registry's expressiveness — either restrict v1 to drop-only and rename-only and defer value transforms (cases 2 and 4) to a follow-up, or widen the registry shape to `(EntityType, &str, Disposition, Option<ValueTransform>)`. Either is fine; leaving it ambiguous forces the implementer to pick mid-task.

### [MEDIUM] AC-04 sidecar covers top-level fields only; inner-shape drift is silently uncaught
**Category:** failure-mode
**Pass:** 1
**Description:** The permissive read (per interview §"uses `serde_yaml::Value` — matches `audit drift`'s pattern") classifies *top-level* keys against `FIELDS`. A spec.yaml whose `acceptance_criteria[]` entries carry a legacy inner field (e.g. `predecessor_evidence` inside each AC), or a card.yaml whose `scenarios[]` entries have an extra field, parses past the top-level allow-check (key is known: `acceptance_criteria` or `scenarios`) — then handing the partially-classified `Value` to the canonical writer goes through strict parse, which fails. The promise "post-run `orbit verify` is clean" (interview Success Criteria, also implicit in card 0032 goal) does not hold for inner-shape drift. The downstream research project's five legacy fields are all top-level; the spec assumes top-level is the full failure surface.
**Evidence:**
- `schema.rs:97`, `schema.rs:124`, `schema.rs:201`, `schema.rs:240`, `schema.rs:251`, `schema.rs:275`: every non-root struct (`AcceptanceCriterion`, `Scenario`, `Relation`, `Choice`, `NoteEvent`, `TaskEvent`) also carries `#[serde(deny_unknown_fields)]`. The strict surface is not just root.
- `verbs.rs:2342-2391`: `audit drift`'s scan loop walks `mapping.keys()` at the root only — confirms the documented pattern is top-level-only.
**Recommendation:** Either (a) add an explicit AC stating scope is top-level-only and inner-shape drift remains a strict-parse failure the user must hand-edit, or (b) extend the reconcile walker to recurse into list-of-struct fields (`acceptance_criteria`, `scenarios`, `relations`) and apply the same allow-check at each nesting level. Option (a) is the smaller change and matches the documented design intent; option (b) is more thorough but doubles the scope.

### [MEDIUM] AC-06 idempotency on existing sidecars lacks merge semantics
**Category:** failure-mode
**Pass:** 2
**Description:** AC-06 says "A tree with existing quarantine sidecars where the same content would be re-quarantined → the sidecar is not rewritten". Two cases the AC doesn't address:
1. **Accumulation:** if `<name>.legacy.yaml` already contains field `foo` from a prior run, and this run discovers field `bar` (newly added to the canonical file since last reconcile), the sidecar must merge — not overwrite. Spec doesn't say whether the sidecar accumulates or each run starts from scratch.
2. **Byte-equality vs structural-equality:** `serde_yaml::Value`'s reserialisation may not be byte-stable if anchors, aliases, or comment-adjacent whitespace differ. Idempotency under byte-equality is fragile; under structural-equality requires defining "same content" precisely.
**Evidence:** Interview §"when a sidecar already exists with content matching what would be re-quarantined, the verb does not rewrite the sidecar" — uses informal "matching" without a definition.
**Recommendation:** Add a sentence to AC-06 (or as a new AC): "When the canonical file has a new unknown field on a re-run, the sidecar is read, the new key is merged in, and the sidecar is rewritten; existing keys are preserved unchanged. Equality is checked at the parsed-`Value` level, not byte level." If accumulation isn't desired, say so explicitly — the implementer should not have to guess.

### [MEDIUM] AC-08(a) wire into /orb:setup brownfield has no insertion point in the current skill
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-08(a) says `/orb:setup`'s brownfield path "invokes or instructs the agent to invoke `orbit canonicalise --reconcile` when legacy field content is detected (e.g. by a prior `orbit audit drift` showing non-empty drift)". The current `/orb:setup` brownfield path (§3 of `plugins/orb/skills/setup/SKILL.md`) operates on a *pre-migration* tree where the legacy layout has bare `cards/`, `specs/`, `decisions/`, `discovery/` directories — there is no `.orbit/` tree yet for `orbit audit drift` to walk. The audit-drift signal can only be meaningful *after* §3d's `git mv` transaction.
**Evidence:**
- `plugins/orb/skills/setup/SKILL.md:48-100`: §3 is layout migration only; no step references audit drift or schema strict-parse.
- `plugins/orb/skills/setup/SKILL.md:24-29`: brownfield is detected by bare-dir presence, not by spec content.
**Recommendation:** Specify where the wire lands. Two options: (a) add a new §3g "field-shape reconcile" that runs *after* §3d (the layout migration completes), invokes `orbit audit drift`, and offers `orbit canonicalise --reconcile --dry-run` → confirm → `--reconcile` if drift is non-empty; (b) wire it into §5 (idempotent state) for cases where a project's `.orbit/` already exists but has accumulated field drift. The spec or the implementer needs to choose; "instructs the agent" is a softer wire that defers the choice, but the skill prose still needs new text somewhere. Add a sub-bullet to AC-08(a) naming the section (§3g or §5) the implementer is committing to edit.

### [LOW] Choice 0023 number is correct as of this review but races against concurrent work
**Category:** assumption
**Pass:** 2
**Description:** AC-08(c) names `.orbit/choices/0023-reconcile-as-canonicalise-mode.yaml` as the next free choice. Current state shows choices go up to 0022, so 0023 is free. Not load-bearing if a concurrent change lands a different 0023 first — the implementer can bump — but flag it for awareness.
**Evidence:** `ls .orbit/choices/` shows 0001 through 0022 present.
**Recommendation:** No change required. The implementer should re-check before authoring the choice file.

### [LOW] `canonicalise` is a CLI subcommand short-circuited before the verb envelope
**Category:** assumption
**Pass:** 2
**Description:** Interview implementation notes describe `reconcile.rs` as called from `canonicalise.rs`'s entry point. That's correct, but: per `cli/src/main.rs:360-368`, `Command::Canonicalise` is *short-circuited* before `build_request()` — it's a hygiene/admin command, not a verb. The implementer should expect to thread `--reconcile` as a new CLI arg on the existing `Command::Canonicalise { dry_run, reconcile }` enum variant, *not* as a new entry in `VerbRequest`. The run-summary expansion in AC-02 also has to happen in `run_canonicalise`'s hand-rolled JSON writer (`cli/src/main.rs:496-516`), not via the typed envelope.
**Evidence:** `cli/src/main.rs:367-368` and `cli/src/main.rs:830-833` (Canonicalise variant unreachable from build_request).
**Recommendation:** No spec change required. Surface this in the implementer's pre-flight: the disposition list must extend `CanonicaliseReport` and the hand-rolled JSON, not the verb envelope.

---

## Honest Assessment

The plan is structurally sound. The choice to land reconcile as a mode on `canonicalise` rather than a sibling verb is well-defended (interview Q2, choice 0023 to be authored); the registry-driven map/drop/quarantine semantics match how `audit drift` already classifies content; the ac-01 preservation strategy (permissive read isolated in `reconcile.rs`, struct-level `deny_unknown_fields` untouched) is the right shape.

The biggest risk is **AC-03's referent does not exist** — the implementer will discover this immediately and either improvise a shape or block. Worth fixing in the spec rather than letting the drive/implement pass figure it out. The next-largest risk is **AC-02 and AC-04 under-specify the disposition surface** — both top-level-only scope and value-transform semantics need a sentence each. AC-08(a)'s missing insertion point in `/orb:setup` is the third edit. None require redesign; all are wording precision.

Once those four edits land, this is implementable in a single pass.
