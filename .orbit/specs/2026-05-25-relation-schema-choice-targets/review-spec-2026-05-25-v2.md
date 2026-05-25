# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-relation-schema-choice-targets
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (schema change, byte-equal backward-compat, cross-system CLI/MCP parity) | 2 |
| 3 — Adversarial | not triggered | — |

This is a re-review. v1 (`review-spec-2026-05-25.md`) raised five findings on the prior spec text; all five landed in the current spec (ac-09 now names the type-system audit explicitly, ac-10 uses the "bump from working-tree" rule with a CHANGELOG path pin, ac-02 carries both validator options with descriptive error, ac-05(f) was reworded to the symmetric no-`card:` / no-`choice:` form). v2 focuses on what's new or remained unflagged.

## Findings

### [MEDIUM] Choice-target relation uses full-slug form, breaking the `relation.X` ≡ `X.id` parallelism

**Category:** assumption
**Pass:** 2
**Description:** Cards 0005 and 0006 are written with `relations: [{choice: 0020-shell-scripts-to-rust-verbs, ...}]` (full slug). But `Choice.id` is bare-numeric — per `.orbit/conventions/id-conventions.md` and the schema (`pub id: String` populated as `'0020'` across every choice file). For card-target relations the parallelism holds: `relation.card: 0019-tabletop` matches `Card.id: 0019-tabletop`. For choice-target relations the spec breaks it: `relation.choice: 0020-shell-scripts-to-rust-verbs` does not match `Choice.id: '0020'`.

Lookup tolerates both forms (`resolve_numeric_slug` at `verbs.rs` does filename-prefix-match against the choices_dir), so functionally this works — but it codifies an asymmetry into substrate. Every future choice-edge author has to decide which form to write, and the convention will drift.

Two paths forward, both consistent:

- **(a) Choice-edges carry bare-numeric** (`choice: '0020'`). Matches `Choice.id`. Asymmetric vs `relation.card` but each side mirrors its own entity's id form. This is what id-conventions.md implies by saying "Choice id is numeric prefix only".
- **(b) Choice-edges carry full-slug** (`choice: 0020-shell-scripts-to-rust-verbs`). Matches `relation.card`'s slug form. Asymmetric vs `Choice.id` but consistent within the relations block.

Pre-recommendation: **(a)**. The id-conventions doc is the substrate's standing answer on choice identity, and the prose-reference style at line 37 (`choice 8`, `choice 0008`) confirms the numeric form is the canonical handle. Following the bare-numeric form keeps the relation field's value identical to the choice file's `id` field, so a grep for `'0020'` finds both the choice and every relation pointing at it.

**Evidence:**
- `.orbit/conventions/id-conventions.md:25` — "Choice `id` numeric prefix only (`'0021'`) — the slug is in the title"
- `.orbit/choices/*.yaml` headers — all 8 sampled choices use `id: '00NN'`
- Spec ac-07/ac-08 — both write the full-slug form
- `resolve_numeric_slug` in `verbs.rs` — accepts both forms via prefix-match, so this is convention-only, not a parse failure

**Recommendation:** Either pick (a) and rewrite ac-07/ac-08's relation entries to `choice: '0020'` (with quotes — Choice.id is a quoted-string in every file because `0020` would otherwise parse as integer), or pick (b) and add a sentence to id-conventions.md documenting that relations carry full-slug for both card and choice targets. Don't ship without making the call — the v1 of choice-edges sets the precedent for every future one.

---

### [MEDIUM] ac-02's custom-Deserialize path silently loses `deny_unknown_fields` unless re-implemented

**Category:** failure-mode
**Pass:** 2
**Description:** ac-02 pre-recommends option (a) — a custom `Deserialize` impl on `Relation` that validates the card-XOR-choice invariant inside `deserialize`. The current `Relation` derives `Deserialize` and carries `#[serde(deny_unknown_fields)]`, which means a parse of `{card: x, type: depends-on, reason: r, bogus: 1}` fails because `bogus` is unknown.

The serde gotcha: **`#[serde(deny_unknown_fields)]` only takes effect on `Deserialize` impls that serde *derives***. A hand-written `impl<'de> Deserialize<'de> for Relation` reads exactly the fields it asks for and silently ignores everything else — `deny_unknown_fields` is a code-generation directive, not a runtime check serde calls into. ac-05(e) (*"parse with an unknown sibling field still rejected — `deny_unknown_fields` preserved"*) will fail under the most-natural option-(a) implementation.

Workarounds exist (manually visiting the map and erroring on unknown keys; using `#[serde(deny_unknown_fields)]` on a private `RelationRaw` derive-target and then calling the validator), but the spec pre-recommends (a) without naming this trap.

**Evidence:**
- `schema.rs:547-554` — current Relation carries `#[serde(deny_unknown_fields)]` on a derived `Deserialize`
- ac-05(e) — explicitly tests the `deny_unknown_fields` contract is preserved
- serde docs / community knowledge — `deny_unknown_fields` is a derive-only attribute; hand-rolled `Deserialize` impls must re-implement unknown-field rejection manually

**Recommendation:** Update ac-02 to name the gotcha and the standard workaround:

> Option (a) — custom `Deserialize` on `Relation`. Note: `#[serde(deny_unknown_fields)]` is a derive-time attribute and does not fire on hand-rolled `Deserialize` impls. The custom impl must either (i) define a private `#[derive(Deserialize)] #[serde(deny_unknown_fields)] struct RelationRaw { ... }` matching the public shape, deserialise into `RelationRaw`, then run the card-XOR-choice validation; or (ii) implement `Visitor` manually and reject unknown keys in `visit_map`. ac-05(e) is the regression test.

That keeps option (a) on the table and arms the implementer against the trap.

---

### [LOW] ac-06 leaves "at least one verb" undefined — implementer may pick a verb that doesn't carry relations

**Category:** test-gap
**Pass:** 1
**Description:** ac-06 says *"at least one verb that surfaces a card's relations (e.g. `card.show` or `card.tree`)"*. Both examples do carry relations, but the wording lets the implementer pick another verb that doesn't, then claim closure on a vacuous parity test. The example list is the right answer — making it the required list closes the loophole.

**Evidence:** ac-06 wording; `card.show` and `card.tree` both surface relations per `verbs.rs:3822-3839, 5685-5691`.
**Recommendation:** Reword the parenthetical to a required-list: *"a verb that surfaces a card's relations — `card.show` is the simplest; `card.tree` exercises the recursive expansion path and is preferred"*. Or just pin `card.show` as the canonical surface.

---

## Honest Assessment

The structural shape is right and v1's findings landed cleanly. The remaining risks are convention-level, not correctness-level: (1) the choice-edge id-form decision will set substrate precedent for every future `relations:respects → choice` edge — make the call before shipping rather than after, (2) the custom-Deserialize trap will burn 30-60 minutes of implementer time if not flagged in the AC, and ac-05(e) will be the test that catches it. The biggest risk is item (1) — substrate-shape decisions made silently during implement get hard to reverse. Pin it in the spec, then ship.
