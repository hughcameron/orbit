# Spec Review

**Date:** 2026-05-21
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-21-richer-reconcile-rules
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | Pass 1 findings + content signals (cross-system validation, data migration) | 2 |
| 3 — Adversarial | not triggered (one structural concern, no cascade) | — |

## Findings

### [HIGH] AC-04 cites wrong line range in `cli/src/main.rs`

**Category:** missing-requirement
**Pass:** 1
**Description:** AC-04 says the JSON envelope at `cli/src/main.rs:496-516` gains the additive optional `next_step` field. That line range is wrong on both branches the AC is talking about.

**Evidence:** In the current tree at `orbit-state/crates/cli/src/main.rs`:
- Lines 496-516 are inside `SpecAction::Close` / `MigrateLayout` enum definitions — unrelated to the canonicalise envelope.
- The routine `canonicalise` envelope is built at lines 689-708 (inside `run_canonicalise`).
- The `--reconcile` envelope is built at lines 742-774 (inside `run_reconcile`).

The two envelopes are constructed by separate functions with separate string buffers — there is no single edit site. AC-04's "the JSON envelope" singular treats them as one location, and the cite points at neither.

**Recommendation:** Update the cite to `cli/src/main.rs:689-708 (run_canonicalise) and 742-774 (run_reconcile)`, and rephrase as "both envelopes gain an additive optional `next_step` field". Without this, the implementing agent either edits the wrong region or picks one of the two envelopes and ships a divergent surface.

### [HIGH] Scalar-AC wrapping is a list-element-shape rule, not an inner-field rule

**Category:** assumption
**Pass:** 1, 2
**Description:** AC-02's transform operates on a list **element** (scalar → mapping), but the existing registry shape `(EntityType, structural_path) → Disposition` only fires at two scopes: top-level field on the entity, and inner field inside a list-of-struct. AC-03 claims "wrap-scalar-ac runs at inner scope" — but inner-field handlers don't reach scalar list elements; they reach inner fields of mapping elements.

**Evidence:** `orbit-state/crates/core/src/reconcile.rs:562-566`:
```rust
for (idx, item) in seq.iter_mut().enumerate() {
    let inner_map = match item.as_mapping_mut() {
        Some(m) => m,
        None => continue,
    };
```
A scalar list element falls into the `None => continue` branch and is never visited. The wrap-scalar transform must intercept at the **list level** (between mapping access and `recurse_inner`'s `iter_mut`) — a third registry scope category. The spec's "inner scope" framing under-describes what the implementing agent must build.

**Recommendation:** Add an implementation note (and tighten AC-03's "well-defined order" prose) to say wrap-scalar-ac fires at a new **list-element scope** — a structural-path of shape `<list>[]` (no `.<name>` suffix), positioned before `recurse_inner` so scalar entries become mappings the recurse step can then walk. Either name the registry-shape extension in a new AC or call it out in the design note's implementation-notes block. Without this, the implementing agent will discover the gap mid-build and stop.

### [MEDIUM] Top-level field synthesis is a new disposition kind, not an existing one

**Category:** missing-requirement
**Pass:** 2
**Description:** AC-01 leans on the registry's `transform` disposition to synthesise a missing top-level `id`. The existing `Disposition::Transform` (introduced by spec 2026-05-16-ac-taxonomy ac-05) operates on a field that already exists — it rewrites a value, optionally with sibling writes. Synthesising `id` from the parent folder requires firing a rule when the field is **absent** from the mapping.

**Evidence:** `walk_and_classify` in `reconcile.rs:514` iterates `for key in keys` — keys collected from the existing mapping. A missing required field has no key, so no rule can fire under the current registry shape. The transform handler also takes `surrounding: &serde_yaml::Mapping` for sibling reads — for filename derivation it needs the spec's **filesystem path**, which isn't currently threaded through.

**Recommendation:** Extend either the transform signature (add a `context: TransformContext` carrying the file path) or add a pre-walk "synthesise" phase that fires registered top-level synthesisers before `walk_and_classify` runs. Either is fine — but it's a registry-shape extension, not a registry-entry addition, and the spec should say so.

### [LOW] AC-04 contradicts the design note on which path emits the breadcrumb

**Category:** constraint-conflict
**Pass:** 1
**Description:** AC-04 says the breadcrumb is "identical across `--reconcile` and routine `orbit canonicalise` paths". The design note (implementation-notes block) says "when `--reconcile` rewrites a file and post-rewrite the strict parse still fails ... Same line on the routine `orbit canonicalise` path when strict parse fails without `--reconcile`." Both touch the same paths but the framings differ in nuance — AC-04 treats the registry as having already applied (so the breadcrumb fires when reconcile's reach is exceeded), while the design note's wording "post-rewrite" implies the file was rewritten then failed. Routine canonicalise doesn't rewrite parse-failed files at all today.

**Evidence:** `cli/src/main.rs:683-728` shows `run_canonicalise` (non-reconcile path) already reports `parse_failed` for files that fail strict parse; no rewrite occurs. The breadcrumb on this path is fine — it just nudges the agent toward conformance findings. On the `--reconcile` path the parse-failed list contains files where the registry didn't cover the drift. Same breadcrumb text reads sensibly on both, but the design note's "post-rewrite" framing should be tightened to "post-registry" or removed.

**Recommendation:** Either (a) update the design note's framing to match AC-04 ("when any file remains in `parse_failed` after the registry's full rule set has applied"), or (b) accept this is just prose drift and move on. Not blocking; flag for the implementing agent to read AC-04 as authoritative.

---

## Honest Assessment

The spec's goal is sharp and the validation strategy (ac-05 against the named brownfield repo with a recorded baseline) is genuinely well-shaped. The two `transform`-rule additions are the right shape for the registry pattern. But two of the four blocking ACs (01, 02) lean on the existing `Disposition::Transform` machinery as if it already covers their case, and on close inspection it doesn't — `id`-synthesis fires when a key is absent (not a known transform shape) and scalar-AC wrapping fires at a list-element scope the registry doesn't recognise. The implementing agent will hit both gaps within the first hour and need to extend the registry's shape, not just add entries. That's still a feasible spec — it's a half-day's extra work, not a re-design — but the spec should name the shape extensions up front rather than presenting them as "additive against `FIELD_RULES`" (design note line 18).

The AC-04 line-range cite is the most directly actionable fix: a 30-second edit that prevents the implementing agent from editing the wrong region of the file.

Biggest risk: AC-03's "well-defined order ... commutative" claim rests on the two rules operating at disjoint scopes (top-level synthesis vs list-element wrap). That's true, but the spec describes them as top-level-vs-inner-scope, which obscures whether they share the registry shape. Worth tightening before the implementing agent goes deep.
