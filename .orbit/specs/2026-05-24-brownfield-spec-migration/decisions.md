# Design decisions — legacy maturity auto-mapping

**Spec:** 2026-05-24-brownfield-spec-migration
**Card:** 0032-brownfield-spec-migration
**Scope (per rally proposal):** add a reconcile rule that auto-maps legacy `maturity: active` → `established` and `maturity: in_design` → `emerging` on Cards, so `orbit canonicalise --reconcile` stops aborting on brownfield cards from older orbit versions.
**Reach:** half-session. Four decisions, narrowly scoped.

The prior spec `2026-05-21-richer-reconcile-rules` added `Disposition::Synthesise` and `Disposition::WrapListElement`. This spec adds *one* rule on *one* entity (`Card.maturity`); the implementation surface is one entry in `FIELD_RULES` and one handler function. The decisions below pin which variant carries it, how unknown values behave, the v1 extensibility posture, and the log shape.

---

## D1. Which Disposition variant carries the mapping?

**Context.** Reconcile already has `Map`, `Drop`, `Quarantine`, `Transform`, `Synthesise`, and `WrapListElement` (`reconcile.rs:83-123`). The `active → established` / `in_design → emerging` rewrite is a value-level rename keyed on the field's current scalar — it is not a key rename (`Map` renames the *key*, not the value), not a key insertion (`Synthesise` fires only when the key is absent), and not a list-element wrap.

`Disposition::Transform(TransformFn)` already exists for exactly this shape: it inspects the current value, may consult the surrounding mapping, and returns either `Replace { value, sibling_writes, detail }` or `Quarantine(reason)`. The existing `reconcile_ac_type` handler at `reconcile.rs:330-419` is the canonical precedent — value-level enum routing with a quarantine fallback.

**Options.**
1. **Reuse `Transform`** with a new `reconcile_card_maturity` handler registered at `(EntityType::Card, "maturity", Disposition::Transform(reconcile_card_maturity))`.
2. **Add a new `Disposition::MapEnum(field_label, &[(from, to)])`** carrying a static mapping table directly in the variant.
3. **Inline the rewrite in `walk_and_classify`** as a special-case branch outside the registry.

**Trade-offs.**
- Option 1 ships in ~30 lines (one handler + one registry entry) and matches the established pattern. Cost: a function for a small mapping, but the function already has a precedent (`reconcile_ac_type` carries far more logic for an analogous shape).
- Option 2 is *cleaner-looking* for a pure table lookup and could be reused if more enum-rename rules emerge — but it adds a sixth variant to the public `Disposition` enum, expands the registry surface, and there is currently exactly one such rule. The richer-reconcile spec's posture (choice 0023's "follow-up spec teaches the registry the richer rule shape once a second project demands it") argues against speculative variant addition.
- Option 3 bypasses the registry entirely, which breaks the disposition-record contract and hides the rule from the run summary.

**Recommendation: Option 1.** Add `reconcile_card_maturity` as a `TransformFn`, register `(EntityType::Card, "maturity", Disposition::Transform(reconcile_card_maturity))` in `FIELD_RULES`. Matches precedent, ships smallest, leaves room for a `MapEnum` variant to emerge later if a *second* enum-rename rule materialises.

---

## D2. How does the handler treat unknown maturity values?

**Context.** The canonical `CardMaturity` enum (`schema.rs:515-521`) is `Planned | Emerging | Established`. The known brownfield drift is `active` and `in_design`. A spec.yaml could in principle carry any other string (`shipped`, `live`, `wip`, …) — what should the handler do?

Three behavioural options. Note: the routine canonical pass already fails on unknown maturity values (strict parse against `CardMaturity` rejects them). The reconcile path is the *only* place this question matters; whatever it does, routine `orbit verify` afterwards still enforces strict parse.

**Options.**
1. **Quarantine unknown values** (return `TransformResult::Quarantine(reason)`) — the field moves into the sidecar, the card is left without a `maturity:` key, the reserialise fails because `maturity` is required, and the file lands in `parse_failed`. The agent sees a disposition record naming the unknown value and a parse-failed entry pointing them at it.
2. **Default unknown values to `planned`** with a `transform_detail` noting the original value — matches `synthesise_spec_status_open`'s posture (brownfield default with explicit detail in the run summary). The card round-trips clean; the operator corrects to a more accurate maturity afterwards if they care.
3. **Pass canonical values through** (`planned`/`emerging`/`established` → no-op rewrite with a "canonical pass-through" detail line, exactly like `reconcile_ac_type` does for canonical `ac_type` values), and treat anything else as Option 1 or 2.

**Trade-offs.**
- Quarantine (Option 1) is the substrate-honesty default — never destroy or fabricate semantic content. But it converts a one-value-off problem into a parse-failed problem, defeating the rule's purpose for any brownfield maturity value not explicitly listed.
- Defaulting (Option 2) keeps the card moving but introduces fabrication: the operator might miss that a `shipped` card got rewritten to `planned`. The disposition record carries the trail, but the AC `Unknown fields default to quarantine, not silent drop` (card 0032 scenario, spec ac-02) sets a strong project-wide presumption against fabrication.
- Option 3 layers cleanly on either 1 or 2: surface a pass-through record for canonical values regardless.

**Recommendation: Option 1 + Option 3 (pass through canonical values, quarantine everything else).** Map exactly `active → established` and `in_design → emerging`; pass through `planned`/`emerging`/`established` with a no-op rewrite + canonical-pass-through detail; quarantine any other string. This matches `reconcile_ac_type` exactly and keeps the substrate-never-fabricates posture from card 0032 scenario 2. The two known brownfield values clear the path the rally proposal calls out; unknowns surface as parse-failed with a disposition record naming the value, so the operator gets a precise pointer rather than a silent rewrite.

---

## D3. Is the mapping table extensible in v1 (project-local registry)?

**Context.** Card 0032 scenario 6 ("Mapping registry is extensible without re-shipping orbit-state") is a non-gate scenario on the parent card. Choice 0023 ("Reconcile as canonicalise mode") explicitly excluded project-local registry overrides from v1: *"v1 dispositions are `map` (rename only, value passes through), `drop`, `quarantine` … project-local override registry is deferred."* Spec 2026-05-21-richer-reconcile-rules's design note repeated that posture: *"Project-local registry overrides — choice 0023 already excluded this from v1; same posture here."*

The rally proposal calls this drive the smallest of three at half-session scope.

**Options.**
1. **Hard-code the two values in `reconcile_card_maturity`**, inline match against `"active"` / `"in_design"`. Defer project-local registry to a future spec.
2. **Add a project-local mapping file** (e.g. `.orbit/reconcile-rules.yaml`) discovered at reconcile time, loaded and merged into the in-tree registry.
3. **Lift the mapping into a constant on `reconcile.rs`** (e.g. `const CARD_MATURITY_MAP: &[(&str, &str)] = &[…]`) so future additions are a one-line edit to a literal table rather than another `if` arm — but still in-tree.

**Trade-offs.**
- Option 1 ships in minutes and matches the established choice-0023 posture. Cost: a future addition (`shipped → established`?) requires another tiny PR rather than a config edit. That cost has already been judged acceptable twice.
- Option 2 is a *real* feature (file discovery, merge semantics, precedence rules, `--help` discoverability per card 0032 scenario 6) and is not a half-session piece of work. Doing it under this rally drive blows the budget and competes for design surface with the work the proposal called out.
- Option 3 is a stylistic preference inside Option 1 — slightly cleaner for table-style mappings, slightly more ceremony for two entries.

**Recommendation: Option 1, with the table as a local constant inside the handler (light Option 3 flavour) — defer project-local registry to a future spec against card 0032.** The pattern from `reconcile_ac_type` is the model: an in-handler match expression listing the known values is the canonical shape. If a third maturity-rename emerges, lift to a `&[(&str, &str)]` table at that point. Project-local registry is its own spec and warrants its own tabletop.

---

## D4. What does the disposition record look like in the run summary?

**Context.** `DispositionRecord` (`reconcile.rs:444-462`) carries `path`, `kind`, `field`, `action`, and an optional `transform_detail`. `reconcile_ac_type` sets `transform_detail` to a human-readable phrase per outcome (e.g. `"canonical pass-through: code"`, `"typo normalisation: docs -> doc"`, `"gate-as-type with build/test description -> code + gate=true"`). The action string is the variant's `action_str()` — `"transform"` for `Transform` (or `"quarantine"` if the handler returns `TransformResult::Quarantine`).

The question: what detail string does the maturity rule write? This is small but affects the run summary's readability in any project running reconcile against legacy cards.

**Options.**
1. **Concise rename phrase per mapped value:** `"legacy rename: active -> established"`, `"legacy rename: in_design -> emerging"`, `"canonical pass-through: <value>"` for pass-through, `"unknown maturity value: \"<s>\""` for quarantine.
2. **Verbose rationale** including the orbit version provenance: `"legacy rename (pre-canonical maturity vocabulary): active -> established"`.
3. **No `transform_detail`** — leave `None`, rely on `action: "transform"` plus the structural `field: "maturity"` to be self-explanatory.

**Trade-offs.**
- Option 1 matches `reconcile_ac_type`'s house style (short prose, includes from/to) and gives the agent enough information from the run summary alone to know what changed and why.
- Option 2 is closer to a release-note tone — useful in a one-shot brownfield setup, noise in a re-run.
- Option 3 loses the from/to trail, which is the most useful information for a reader scanning the disposition list.

**Recommendation: Option 1.** Follow `reconcile_ac_type`'s phrasing. Concrete strings to write:
- `active` → `established`: `"legacy rename: active -> established"`
- `in_design` → `emerging`: `"legacy rename: in_design -> emerging"`
- `planned`/`emerging`/`established`: `"canonical pass-through: <value>"`
- anything else: `TransformResult::Quarantine("unknown maturity value: \"<s>\"")`

This keeps the disposition list grep-friendly and matches the handler's nearest precedent line for line.

---

## Decisions at a glance

| # | Decision | Pick |
|---|----------|------|
| D1 | Disposition variant | Reuse `Transform` — new `reconcile_card_maturity` handler. |
| D2 | Unknown value handling | Pass canonical through; map the two known legacy values; quarantine everything else. |
| D3 | Project-local extensibility | Defer — hard-code the table in the handler, follow choice 0023's posture. |
| D4 | Log shape | Per-outcome `transform_detail` strings matching `reconcile_ac_type`'s house style. |

These decisions together produce a tightly-scoped implementation: one entry in `FIELD_RULES`, one handler function around the same size as `synthesise_spec_status_open`, one fixture at `orbit-state/crates/core/tests/fixtures/reconcile/legacy-maturity/` covering both legacy values and a pass-through case, and one quarantine-path test for the unknown-value branch. No new `Disposition` variant, no new registry-shape extension, no new entity-type wiring (`Card` is already walked).
