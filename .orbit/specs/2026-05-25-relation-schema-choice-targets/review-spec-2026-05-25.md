# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-relation-schema-choice-targets
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (schema change, cross-system CI-validated round-trip, version-pin reasoning) | 3 |
| 3 — Adversarial | not triggered | — |

## Findings

### [HIGH] AC-09's consumer audit query misses two compile-blocking call sites
**Category:** test-gap
**Pass:** 2
**Description:** AC-09 names the consumer audit as `rg '\.card\b|relation\.card|Relation \{ card' orbit-state/`. Running that pattern catches `relation.card` / `r.card` reads at `verbs.rs:3822, 3839, 5653, 5656, 5685, 5691`, which the implementer will indeed have to handle. But two further sites *also* break the build and aren't in the audit's natural output:

1. **`Relation::FIELDS` constant** at `orbit-state/crates/core/src/schema.rs:144-146` is `&["card", "type", "reason"]`. The drift-test `relation_fields_matches_struct` (schema.rs:1245-1257) serialises a fully-populated `Relation` and asserts the resulting YAML keys equal `Relation::FIELDS`. After this spec lands, a fully-populated `Relation` serialises to `{card, choice, type, reason}` — so `Relation::FIELDS` must gain `"choice"` or the FIELDS-drift test fails. The test also has to populate `choice` with a `Some`.
2. **`relation_kind_str()` at `verbs.rs:3849`** is a non-exhaustive `match` over `RelationKind`. Adding the `Respects` variant turns this into a `match` that the compiler will reject (or warn, depending on flags) — the function needs a `RelationKind::Respects => "respects"` arm. Same applies to any other exhaustive match over `RelationKind` (none others found in this audit, but the spec should name the pattern, not just the regex).

A third audit-adjacent surface: `reconcile.rs:856` references `Relation::FIELDS` for legacy-field disposition routing. Adding `choice` to the FIELDS list is consistent with the existing reconcile pattern and doesn't need new code, but it should be noted in the consumer-audit findings.

**Evidence:** schema.rs:144-146, schema.rs:1245-1257, verbs.rs:3849-3857, reconcile.rs:856.
**Recommendation:** Widen AC-09's audit verb to one of:
- Add a second regex: `rg 'RelationKind::|Relation::FIELDS' orbit-state/` and require the implementer to handle exhaustive-match additions and the FIELDS constant as part of the same AC.
- Or split into AC-09a (read-site audit) and AC-09b (type-system audit: `Relation::FIELDS` + every exhaustive `RelationKind` match arm). The latter is cleaner.

The current AC-09 text technically lets the implementer claim closure without touching `Relation::FIELDS` or `relation_kind_str()`, even though both are mandatory for the build to compile and tests to pass. Add them by name.

---

### [MEDIUM] AC-05 (f) asserts a serialisation order the schema can't guarantee
**Category:** assumption
**Pass:** 2
**Description:** AC-05 (f) says: *"serialise a choice-target relation produces canonical YAML shape (`choice: <id>` first, then `type:`, then `reason:`)"*. Two problems:

1. **Field order is determined by struct declaration order, not the YAML's preferred order.** AC-01 says `Relation.card` softens to `Option<String>` and a new `choice: Option<String>` field is added — but doesn't say *where* in the struct. If `choice` is appended after `card` (the natural reading of "softens its card and adds a choice"), the serialised output for a choice-target relation is `{choice: ..., type: ..., reason: ...}` — `card` is skipped via `skip_serializing_if`. That matches the AC. But if the implementer puts `choice` *before* `card`, output is the same. The AC's claim happens to hold for both orderings only because `card` is `None` for choice-target relations and gets skipped.
2. **The canonical writer in `canonicalise.rs` doesn't re-order Relation fields** — there's no per-field sort for `Relation` in canonicalise.rs (grep returned zero matches for `Relation`/`relation`). So the order is purely struct-declaration order out of serde.

The AC will pass on both struct orderings, so it's not strictly broken — but it tests an *emergent* property (the `None` gets skipped, leaving choice/type/reason) rather than the load-bearing one (round-trip byte-equality + skip_serializing_if behaviour). A failed assertion would point at a different cause than the AC's wording suggests.

**Evidence:** AC-05 (f); schema.rs:547-554 (current Relation struct, card before kind before reason); canonicalise.rs grep (no Relation handler).
**Recommendation:** Reword AC-05 (f) to assert the actual contract: *"serialise a choice-target relation produces YAML with no `card:` key (skip_serializing_if elides the None), and a `card`-target relation produces YAML with no `choice:` key"*. That's the byte-equal backward-compat guarantee. Test field order separately if it matters (it probably doesn't — `orbit canonicalise` is the byte-shape contract, and Relation isn't currently re-ordered there).

---

### [MEDIUM] AC-10 version-bump logic enumerates branches but doesn't pin the assumption that this branch is rebased on top of #33
**Category:** assumption
**Pass:** 2
**Description:** AC-10 says: *"orbit-state workspace version bumped by one patch from the working-tree Cargo.toml at implement-stage start (0.4.35 → 0.4.36 if rebased on PR #33, or 0.4.34 → 0.4.35 if rebased on PR #32 only, or 0.4.33 → 0.4.34 if rebased on bare main)"*. The workspace Cargo.toml currently shows `version = "0.4.35"`, which means this branch is already rebased on top of PR #33 (or has #33's changes merged in). PRs #32 and #33 are both still open on GitHub. Two risks:

1. **If PR #33 gets squash-merged into main with a different commit hash than what this branch carries, the rebase will not be clean** — and the implementer arrives at implement-stage with a working-tree version that's neither 0.4.35 nor 0.4.36, but something like 0.4.34 (from a partial rebase) or a conflict.
2. **The AC reads as "pick the right branch at implement-time"** but doesn't say what to do if the working-tree version doesn't match any of the three named cases. Implementer guesses or halts.

**Evidence:** `gh pr list --state open` shows #32 and #33 both open as of review time. Current `orbit-state/Cargo.toml` is `version = "0.4.35"`, matching the "rebased on #33" branch in AC-10.
**Recommendation:** Simplify AC-10's version logic to: *"workspace version bumped by one patch from whatever the working-tree Cargo.toml shows at implement-stage start"* — and let the implementer read the version and bump. The three-case enumeration is brittle. Add an explicit halt-trigger if the working-tree version is unexpected (not `0.4.33`, `0.4.34`, `0.4.35`, or `0.4.36`).

---

### [LOW] AC-02 names a `validate_relation_target` helper but doesn't pin where the validation runs
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-02 says the helper is "called from `parse_yaml::<Card>`". But `parse_yaml` is the generic at `canonical.rs:28` over `T: DeserializeOwned` — it doesn't know about `Card` specifically and has no per-type validation hook. The actual call sites that parse a `Card` are at `index.rs:199`, `canonicalise.rs:104`, and other entry points. Calling validation "from `parse_yaml::<Card>`" is either (a) a custom `Deserialize` impl on `Relation` (the spec's option (b)), (b) wrapping every `parse_yaml::<Card>` call site to also call the validator, or (c) a new `Card::parse_yaml` method. The spec pre-recommends (a) — meaning add the call to *every Card parse site*, three of them — but the wording "called from `parse_yaml::<Card>`" implies one site.
**Evidence:** canonical.rs:28-46 (`parse_yaml` is generic), index.rs:199 (Card parse), canonicalise.rs:104 (Card parse).
**Recommendation:** Reword AC-02 to: *"validation invoked on every `Card` parse — implementer's choice between (a) a `validate_relations` post-parse helper called from each `parse_yaml::<Card>` call site, or (b) a custom `Deserialize` impl on `Relation`. Both meet the contract."* Or pick one. The custom `Deserialize` option (b) is the cleanest because it validates at deserialisation time without scattering calls, but requires care to preserve `deny_unknown_fields`.

---

### [LOW] CHANGELOG.md location not pinned
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-10 names a CHANGELOG.md entry. There are likely two CHANGELOG.md files in this repo — the plugin one at `/home/hugh/github/meridian-online/orbit/CHANGELOG.md` (or under `plugins/orb/`) and the orbit-state crate's. The spec says "CHANGELOG.md entry under that bumped version" but doesn't pin which file. Since the version bump is on the orbit-state workspace, the orbit-state CHANGELOG (if one exists) or the top-level one (if it tracks orbit-state versions) is the target.
**Evidence:** Spec mentions "CHANGELOG.md" without a path.
**Recommendation:** Pin the path. From prior PR #32/#33 patterns (commit `cf41790 feat(0.4.35): port promote.sh...`), the orbit-state changes flow into the top-level CHANGELOG. Confirm and write the path into AC-10.

---

## Honest Assessment

The plan is solid — additive schema bump with `skip_serializing_if` keeping backward compat is the right shape, and the implementation fork (validation helper vs custom `Deserialize`) is genuinely fungible. The biggest risk is the consumer audit (AC-09): the named regex catches most read sites but misses two compile-mandatory ones — `Relation::FIELDS` and exhaustive `RelationKind` matches in `relation_kind_str()`. Without those in scope, the implementer either rediscovers them at `cargo build` time (cheap detour) or, worse, claims AC-09 closure before realising the FIELDS-drift test fails (forces a re-open). Tightening AC-09 to name the type-system audit explicitly is the highest-value change. The version-bump enumeration in AC-10 is the second risk — open PRs in the stack could shift the working-tree version under the implementer's feet between spec write and implement start.
