# Design: AC taxonomy — typed acceptance criteria

**Date:** 2026-05-16
**Interviewer:** Claude Opus 4.7
**Card:** .orbit/cards/0035-ac-taxonomy.yaml
**Mode:** open

---

## What good looks like

When I write an AC, I want to tell the substrate what kind of evidence will close it — a passing test, a config change, an operator's sign-off, a metric I'll only see seven days from now — and have the rest of orbit honour that distinction. spec.close shouldn't block on an AC that's waiting for the world; /orb:review-pr shouldn't ask for a unit test on a post-deploy observation; /orb:drive shouldn't route an ops sign-off through the implement-and-test loop. I declare the type once, in the spec, and every downstream verb reads it. The taxonomy lives in the canonical schema so brownfield projects with their own AC kinds can absorb cleanly rather than re-fragmenting, and the type makes the verification path legible from the spec itself rather than reconstructed by whichever agent reads it next.

---

## Context

Card: *Typed acceptance criteria* — 6 scenarios (3 gate), 5 feeders (cards 0034, 0030, 0032, 0020, 0026), 0 prior specs.

Prior work touching this domain: card 0034 shipped the narrow precursor `time_gated: bool` in spec `2026-05-13-spec-close-ac-preflight` (orbit 0.4.13, released 2026-05-14). That bool flips spec.close's blocking set for the single "legitimately defer at close" case. Card 0035 generalises one bit to a categorical that signals close semantics, verification path, and review expectation in one declaration.

Live trigger: spec `2026-05-16-memos-own-folder` ac-12 was authored as a release-window beelink smoke. Its verification line asserted a `memos` field on `session prime`'s envelope that does not exist. The smoke functionally passed (binary worked against new layout); the AC was unprovable as written. A typed AC (kind: observation/ops) would have steered the verification language toward "binary boots, command exits 0 on new layout" instead of inventing a structured-field assertion.

Brownfield evidence base (compiled mid-session at Hugh's request):

| Source | ACs | `ac_type` field | Values seen |
|---|---|---|---|
| meridian-online/arcform (public) | 128 | yes, on 123 | `code: 122`, `doc: 5`, `gate: 1` |
| meridian-online/finetype (public) | 325 | yes, on ~293 | `code: 176`, `gate: 60`, `doc: 52`, `config: 5`, `docs: 2` |
| A private R&D repo | ~61 | none structural | verification prose splits into pytest (~10), operator/sign-off (~7), monitoring/soak/post-cutover (~4), MADR/spec text (~3) |

Key finding: the brownfield `ac_type: gate` collides with orbit's existing `gate: true` boolean. arcform uses it once for "cargo build succeeds" (= a `gate: true` `code` AC). The 60 uses in finetype wrap "training completes" / "eval ≥ N/M" — semantically these are `observation`-shaped (a measurement on a running process), and usually also blocking-importance. Reconcile-mode must split these into the orthogonal axes: `ac_type` for the kind, `gate: true` for the blocking-importance.

Gap to close: the bool's information content (defer-at-close vs block-at-close) becomes one axis of a five-value enum that also tells review-pr and drive what evidence to expect.

## Q&A

### Q1: AC vocabulary
**Q:** Is `code / ops / observation / research` (from the card's notes) the canonical set, or do the actual brownfield repos suggest a different shape?

**A:** Scan the brownfield first. Evidence ratified the proposed five-value enum:

`code` · `config` · `doc` · `ops` · `observation`

- **code** — closes on a passing test / referenced commit / functional artefact. Blocks close.
- **config** — closes on a config or external-system-state change verifiable by grep / file inspection / external query. Blocks close.
- **doc** — closes on a written artefact (CLAUDE.md edit, card text, memo, MADR). Blocks close. (The card's hypothesised "research" collapses here — they're the same kind: closure by a written artefact.)
- **ops** — closes on operator action with captured log line / signoff / dashboard check. Legitimately deferred at close.
- **observation** — closes on a dated window of empirical measurement (post-cutover N-day soak, eval-run output, training-completes-and-produces-metrics). Legitimately deferred at close.

Two close-time bands: `code / config / doc` block spec.close when unchecked; `ops / observation` are listed under deliberate-deferral and do not block.

### Q2: Intent paragraph
**Q:** Accept the drafted "what good looks like" paragraph, or edit?

**A:** Accepted as drafted.

### Q3: Slicing
**Q:** One spec covering all six scenarios, or stack of three (substrate / skills / brownfield)?

**A:** Single spec, all wires at once. Atomic correctness over multiple-cycle overhead — the card's "wired into the framework" scenario lands fully in one go.

### Q4: Migration policy
**Q:** Default for ACs that omit `ac_type`, and what happens to today's `time_gated: bool`?

**A:** Default = `code`. Drop `time_gated` cleanly in one canonicalise pass; the field is removed from the schema; every `time_gated: true` in orbit's own corpus is rewritten to `ac_type: observation`. Forward-only, no compat shim. Confidence: high; rests on brownfield repos tolerating `canonicalise --reconcile` on upgrade, which is consistent with card 0032's stated on-ramp.

---

## Summary

### Goal
`AcceptanceCriterion` carries a five-value `ac_type` enum (`code / config / doc / ops / observation`) with `code` as the default. `spec.close` blocks on unchecked `code/config/doc` ACs and lists `ops/observation` ACs separately as deliberate-deferral candidates. `/orb:review-pr` expects type-appropriate evidence (test/commit for `code`, file diff for `config/doc`, operator log line for `ops`, dated metric window for `observation`). `/orb:drive` routes each AC into the matching workflow. `/orb:design` prompts the author for `ac_type` as ACs are shaped. Brownfield migration via `canonicalise --reconcile` maps legacy `ac_type` values onto the canonical enum, splitting the `gate`-as-type collision into orthogonal `ac_type` + `gate: true`. `time_gated: bool` is removed from the schema in the same release; orbit's own corpus migrates in one canonicalise pass.

### Constraints
- Forward-only — no compat shim, no coexistence period for `time_gated` and `ac_type`.
- Single spec, all wires at once. The card's "wired into the framework" scenario lands atomically.
- Brownfield migration is not lossy: `gate`-as-type splits into `ac_type: code` (or `observation`) + `gate: true`; `docs` typo maps to `doc`; unknown values quarantine into `spec.legacy.yaml` per card 0032 ac-02.
- `code` is the default-or-omitted value — keeps existing untyped corpora parseable on upgrade.
- Schema-version bumps for the field addition + `time_gated` removal.

### Success Criteria
- `AcceptanceCriterion::FIELDS` includes `ac_type` and excludes `time_gated`.
- `spec.close` honours the enum: `code/config/doc` block unchecked, `ops/observation` defer.
- `/orb:review-pr`, `/orb:drive`, `/orb:design` SKILL.md each cite the enum in their evidence/routing/prompting logic.
- Canonical schema doc (card 0030 destination) names the enum and the two close-time bands.
- `canonicalise --reconcile` against arcform and finetype maps the 480 typed brownfield ACs without losing semantic content; `gate`-as-type is split orthogonally.
- The live-trigger pattern is detectable: had spec `2026-05-16-memos-own-folder` ac-12 been written under typed ACs, `/orb:design` would have routed it to `ac_type: observation` and the verification template would have steered the author away from inventing envelope fields.

### Decisions Surfaced
- **Canonical AC vocabulary**: `code / config / doc / ops / observation`. Chose this five-value set over the card's hypothesised four (code/ops/observation/research) because brownfield evidence shows `doc` is the actual idiom (research is a special case of doc), `config` is a small but real distinct kind (5 ACs in finetype), and the `gate`-as-type collision needs to be resolved by the migrator rather than canonised. → record as a choice (MADR) during /orb:spec.
- **Close-time bands**: two-axis — `code/config/doc` block; `ops/observation` defer. Chose two-band over per-value policy because spec.close only needs the boolean "blocks at close" answer; the kind carries the rest of the signal for downstream verbs. → record as a choice.
- **Default behaviour**: `code` is the default for ACs that omit `ac_type`. Chose forgiving-default over required-field because card 0032's on-ramp goal (the schema is reachable from any starting point) requires existing untyped specs to parse on upgrade; the default matches what every untyped AC was implicitly assuming.
- **time_gated retirement**: clean cut in the same release the enum lands; one canonicalise pass rewrites the single live `time_gated: true` in orbit's corpus to `ac_type: observation`. Chose forward-only over coexistence because the bool shipped two days ago, has one live use, and coexistence violates card 0030's canonical-schema goal.
- **Slicing**: single spec, atomic ship. Chose one-spec over a stack-of-three because the card's wired-into-the-framework scenario depends on all wires being live for the field to demonstrate value.

### Implementation Notes
- Substrate location: `AcceptanceCriterion` at `orbit-state/crates/core/src/schema.rs:152` — add `ac_type: AcType` field with `#[serde(default)]` (Default impl returns `AcType::Code`); remove `time_gated: bool`. Add `pub enum AcType { Code, Config, Doc, Ops, Observation }` with `#[serde(rename_all = "snake_case")]`. Add `impl AcType { pub fn blocks_close(&self) -> bool { matches!(self, Self::Code | Self::Config | Self::Doc) } }` — single source of truth for the two-band split.
- `AcceptanceCriterion::FIELDS` const at `schema.rs:109-112` adds `"ac_type"`, removes `"time_gated"`. The `acceptance_criterion_fields_matches_struct` test fixture at `schema.rs:752` updates accordingly.
- `spec.close` at `orbit-state/crates/core/src/verbs.rs` (the time_gated read site introduced by spec 2026-05-13-spec-close-ac-preflight) — switch the unchecked-blocking computation from `!ac.time_gated` to `ac.ac_type.blocks_close()`. Output envelope: rename `time_gated_open` to a name matching the enum semantics (`deferrable_open` or `non_blocking_open` — settle at /orb:spec).
- Canonicaliser: rewrite-pass for orbit's own corpus reads every spec.yaml, sets `ac_type: observation` on ACs that carried `time_gated: true`, drops the `time_gated` field. The single live target is spec `2026-05-16-memos-own-folder` ac-12.
- `canonicalise --reconcile` registry adds three mappings: brownfield `ac_type: docs` → `doc`; brownfield `ac_type: gate` where the description matches "build succeeds" or "test passes" → `code` + set `gate: true`; brownfield `ac_type: gate` where the description matches "eval" / "training completes" / "score ≥ N" → `observation` + set `gate: true`. The split heuristic is a regex pass per card 0032's registry-extensibility AC.
- Schema-version bump: 0.1 → 0.2 (struct field added AND removed — minor bump per orbit-state versioning convention).
- `/orb:design` SKILL.md change: when shaping ACs in §6, prompt for `ac_type` per AC. Default to `code` if unspecified. Surface the two-band close-time consequence so the author understands what they're declaring.
- `/orb:review-pr` SKILL.md change: per-AC walk reads `ac_type` and selects the evidence-expectation template (code → passing test + commit ref; config → grep + file diff; doc → grep + content check; ops → operator log line + signoff quote; observation → dated metric window + metric reference).
- `/orb:drive` SKILL.md change: the implement step routes ACs by `ac_type` — `code` enters the implement-and-test loop; `config/doc` enter the file-edit loop; `ops` escalates to operator-handoff; `observation` registers a deferred checkpoint and proceeds.
- README.md + .orbit/STYLE.md + .orbit/METHOD.md mirrors need an entry on the enum and the two close-time bands. Vocabulary table addition.
- `orbit audit drift` already enforces FIELDS-vs-struct parity (per spec 2026-05-12-tree-views) — the enum addition flows through that gate automatically once `FIELDS` is updated.

### Open Questions
None at the intent level. Implementation-level questions (envelope field renames, drive's deferred-checkpoint registry shape, the regex heuristics in the reconcile registry) are routed to /orb:spec.

---

**Next step:** `/orb:spec` against this design session — the spec will materialise the seven AC clusters above (struct + enum, close, canonicaliser pass, brownfield reconcile registry, three skill edits, schema-doc additions) into numbered ACs.
