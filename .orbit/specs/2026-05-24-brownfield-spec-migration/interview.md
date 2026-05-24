---
date: 2026-05-24
interviewer: claude
card: .orbit/cards/0032-brownfield-spec-migration.yaml
mode: rally
rally: 2026-05-24-brownfield-migration-hardening-rally
---

# Design: reconcile rule for legacy Card.maturity values

## What good looks like

`orbit canonicalise --reconcile` against a brownfield project carrying older `maturity: active` or `maturity: in_design` card values rewrites them to the canonical `established` and `emerging` respectively, in one pass — no hand-edits, no parse-failed cards blocking the run. The disposition list names each transform with from/to clarity. Canonical maturity values pass through clean. Unknown values quarantine cleanly with the unknown string named in the reason, so the operator gets a precise pointer rather than a silent rewrite.

---

## Context

Card: *brownfield-spec-migration* — 7 scenarios, maturity emerging, goal: `orbit canonicalise --reconcile` brings legacy spec.yaml shapes into the canonical schema with substrate-never-fabricates semantics.

Prior specs: 3 — `2026-05-12-reconcile-mode` (initial mode), `2026-05-16-ac-taxonomy` (Disposition::Transform added for ac_type), `2026-05-21-richer-reconcile-rules` (Disposition::Synthesise + WrapListElement; four rules registered).

Gap, from `.orbit/memos/2026-05-24-brownfield-setup-friction.md` (item #3): a downstream project (arcform) had five cards with `maturity: active` (for shipped work) and `maturity: in_design` (for active design) — neither in the canonical set `planned/emerging/established`. Reconcile aborted on each; the operator hand-edited per card before reconcile would proceed. The rally proposal calls this drive the smallest of three at half-session scope.

---

## Q&A

### D1: Disposition variant
**Q:** Which Disposition variant carries the value-level enum rename — reuse `Transform`, add a new `MapEnum`, or inline in `walk_and_classify`?
**A:** Reuse `Disposition::Transform` with a new `reconcile_card_maturity` handler registered at `(EntityType::Card, "maturity", Disposition::Transform(reconcile_card_maturity))`. Matches `reconcile_ac_type` precedent line-for-line. Choice 0023's standing posture argues against speculative variant addition for a single rule; if a second enum-rename emerges later, lift to `MapEnum` then.

### D2: Unknown values
**Q:** Quarantine, default to `planned`, or pass canonical through plus quarantine unknown?
**A:** Pass canonical values through (`planned`/`emerging`/`established` → no-op rewrite with `canonical pass-through: <value>` detail); map the two known legacy values; quarantine anything else with `unknown maturity value: "<s>"` reason. Matches `reconcile_ac_type` exactly and honours card 0032 scenario 2 ("Unknown fields default to quarantine, not silent drop"). The two known brownfield values clear the path; unknowns surface as parse-failed with a precise disposition record rather than a silent rewrite.

### D3: Extensibility
**Q:** Hard-code in v1 or ship project-local override registry?
**A:** Hard-code in `reconcile_card_maturity` as an inline match expression. Project-local override registry (card 0032 scenario 6) is a real feature — file discovery, merge semantics, precedence — and warrants its own tabletop. Choice 0023's deferral posture has been confirmed twice already (richer-reconcile-rules spec design note). This drive's half-session scope does not include it.

### D4: Disposition log shape
**Q:** Concise rename phrase, verbose rationale, or no transform_detail?
**A:** Concise per-outcome `transform_detail` matching `reconcile_ac_type`'s house style:
- `active` → `established`: `"legacy rename: active -> established"`
- `in_design` → `emerging`: `"legacy rename: in_design -> emerging"`
- canonical values: `"canonical pass-through: <value>"`
- anything else: `TransformResult::Quarantine("unknown maturity value: \"<s>\"")`

---

## Summary

### Goal

Add one reconcile rule keyed on `(EntityType::Card, "maturity")` that auto-maps the two known legacy values (`active → established`, `in_design → emerging`), passes the three canonical values through, and quarantines anything else. Ships in `Disposition::Transform` form (no new variant). Logged with per-outcome `transform_detail` strings matching `reconcile_ac_type` house style.

### Constraints

- One new entry in `FIELD_RULES` at `orbit-state/crates/core/src/reconcile.rs`: `(EntityType::Card, "maturity", Disposition::Transform(reconcile_card_maturity))`.
- One new handler function `reconcile_card_maturity: TransformFn` next to `reconcile_ac_type`.
- No new `Disposition` variant. No registry-shape extension. No project-local override file.
- Mapping table is hard-coded inside the handler as an inline match expression.
- Unknown values produce `TransformResult::Quarantine("unknown maturity value: \"<s>\"")`. No silent default.
- `transform_detail` strings match `reconcile_ac_type`'s precedent format.

### Success Criteria

- A card with `maturity: active` running through `orbit canonicalise --reconcile` rewrites to `maturity: established` with a disposition record naming `legacy rename: active -> established`. Subsequent `orbit verify` is clean against that card.
- A card with `maturity: in_design` rewrites to `maturity: emerging` with the matching disposition record.
- A card with `maturity: planned` (or `emerging`, `established`) round-trips unchanged with a `canonical pass-through: <value>` disposition record.
- A card with `maturity: shipped` (or any other unknown value) quarantines with reason `unknown maturity value: "shipped"`. The card lands in `parse_failed` because the maturity field becomes empty after quarantine; the operator gets a precise pointer via the disposition record.
- Routine `orbit verify` after a successful reconcile pass over a fixture containing all three classes (legacy / canonical / unknown) is clean for the canonical and legacy-rewritten cards, and reports parse-failed for the quarantined ones.
- Fixture lives at `orbit-state/crates/core/tests/fixtures/reconcile/legacy-maturity/` containing at least four cards (`active`, `in_design`, `planned` canonical pass-through, `shipped` unknown-value quarantine).
- The handler function is around the same size as `synthesise_spec_status_open` and `reconcile_ac_type` — one function, one match expression, no new types.

### Decisions Surfaced

- **Reuse `Disposition::Transform`.** Matches `reconcile_ac_type` precedent; smallest diff; consistent with choice 0023's standing posture against speculative variant addition. (D1)
- **Pass canonical through + quarantine unknowns.** Honours card 0032 scenario 2's substrate-never-fabricates posture; converts known brownfield values without introducing fabrication for unknown strings. (D2)
- **Hard-code mapping in v1.** Project-local registry deferred to its own future spec; ships in minutes vs blowing half-session budget. (D3)
- **Per-outcome `transform_detail` strings matching `reconcile_ac_type` house style.** Grep-friendly run summary; from/to trail is the most useful information for a reader scanning the disposition list. (D4)

### Implementation Notes

**Codebase leads:**

- Reconcile registry: `orbit-state/crates/core/src/reconcile.rs::FIELD_RULES` — one new entry. Variant: `Disposition::Transform`.
- Handler precedent: `reconcile_ac_type` at `reconcile.rs:330-419`. Exact shape model for the new function — value-level enum routing with quarantine fallback and per-outcome `transform_detail`.
- Disposition variants: `Disposition::Transform(TransformFn)` already exists at `reconcile.rs:103`. No new variant. `TransformResult` already has the `Replace` / `Quarantine` constructors needed.
- Schema constants: `CardMaturity` enum at `orbit-state/crates/core/src/schema.rs:515-521`. Known values: `Planned | Emerging | Established`. The handler does not import this enum — it pattern-matches on strings.
- Fixture pattern: `orbit-state/crates/core/tests/fixtures/reconcile/<feature>/` — recent example is the `richer-reconcile-rules` fixture tree.

**New surface:**

- `reconcile_card_maturity: TransformFn` in `reconcile.rs`. Signature matches `reconcile_ac_type`.
- One new entry in `FIELD_RULES`.
- Fixture tree at `orbit-state/crates/core/tests/fixtures/reconcile/legacy-maturity/` with four cards covering the four outcomes.
- Tests in `reconcile.rs::tests` (inline) — one test per outcome (active, in_design, canonical pass-through, unknown-value quarantine).

**Memory dispositions:**

- `private-projects-genericised-in-artefacts` — *not applicable*. arcform is meridian-online family.

**Out of scope:**

- Project-local mapping registry (card 0032 scenario 6). Deferred to a future spec.
- Spec.maturity, Choice.status, or any other entity's enum field. This rule fires on `EntityType::Card` only; `maturity` is Card-specific.
- A `Disposition::MapEnum` variant. Deferred to a future spec that registers more than one value-level enum-rename rule.
- Cross-drive coupling — this drive is fully independent of Drives A and B; no shared files or symbols expected.
