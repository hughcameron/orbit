# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-relation-schema-choice-targets
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (additive schema change, byte-equal backward-compat, CLI/MCP parity, multi-site consumer audit) | 1 (LOW) |
| 3 — Adversarial | not triggered | — |

This is cycle 3. v1 raised one HIGH (consumer-audit gap), two MEDIUMs (field-order, version-pin), two LOWs (parse-hook ambiguity, CHANGELOG path) — all five landed. v2 raised two MEDIUMs (choice-id form, custom-Deserialize trap) and one LOW (vacuous parity verb) — all three landed: ac-07/08 carry bare numeric `'0020'`, ac-02 flipped pre-recommendation to (b) the call-site sweep with a written rationale for why custom Deserialize is the more expensive path, ac-06 pinned to `card.show`. The spec is materially ready.

## Findings

### [LOW] ac-02's "~3-5 sites" sweep estimate is roughly half the real count

**Category:** assumption
**Pass:** 2
**Description:** ac-02 pre-recommends option (b) — invoke `validate_relation_target` from a wrapper around every `parse_yaml::<Card>` call site — and parenthesises the cost as "~3-5 sites currently". `rg 'parse_yaml.*Card' orbit-state/crates/core/src/` returns eight Card parse sites: `canonicalise.rs:104`, `index.rs:199`, `verbs.rs:2318`, `verbs.rs:2837`, `verbs.rs:3697`, `verbs.rs:3797`, `verbs.rs:3952`, `verbs.rs:5736`. The wrap-everywhere approach is still cheaper than re-implementing `deny_unknown_fields` by hand (ten or so insertions, each a one-line `validate_relation_target(&card)?;` after the parse), but the count estimate is off by ~2x and a wary implementer reading the AC may second-guess the recommendation.

Two clean shapes for the sweep, both fine:

- **(b.i) Wrap at each call site.** Add `validate_relation_target(&card)?;` after each of the eight `parse_yaml::<Card>(...)?` lines. Each insertion is mechanical.
- **(b.ii) Introduce `parse_card_yaml(text) -> Result<Card>` in `canonical.rs`** that wraps `parse_yaml::<Card>` and runs the validator. The eight sites swap `parse_yaml(&text)` for `parse_card_yaml(&text)` and the validator lives in exactly one place. Slightly larger diff but single-point-of-truth.

**Evidence:**
- `rg 'parse_yaml.*Card' orbit-state/crates/core/src/` — 8 hits across canonicalise.rs, index.rs, verbs.rs
- ac-02 text — "costs a call-site sweep (~3-5 sites currently)"

**Recommendation:** Correct the count in ac-02's parenthetical to "~8 sites" and optionally mention shape (b.ii) as the slightly-larger-but-cleaner variant. Not blocking — the pre-recommendation still holds and the implementer will discover the real count at the first `cargo build`. Land as a small clarification or address inside implement.

---

## Honest Assessment

The spec is ready. The structural shape is correct (additive `Option<String>` field, `skip_serializing_if` for byte-equal backward-compat, enum-variant addition, two card edges, type-system + read-site audit, version bump, CHANGELOG entry). Each AC names the file + line where the change lands. The two cycle-2 substrate-shape calls — bare-numeric choice id and call-site sweep over custom Deserialize — are both correct against the substrate's standing conventions (id-conventions.md for the id form; ac-05(e)'s `deny_unknown_fields` regression test as the disambiguator on the validation hook).

The only remaining finding is the parenthetical site-count estimate in ac-02. It doesn't change the chosen path and won't block implementation — the implementer adds the validator at every site flagged by `cargo build`'s type errors, and the count converges to whatever it converges to.

Verification spot-checks performed against the working tree:
- `schema.rs:549` — `Relation` struct present with `card: String`, `deny_unknown_fields`, exactly the shape ac-01 modifies
- `schema.rs:144-146` — `Relation::FIELDS = &["card", "type", "reason"]` matches the ac-09 type-system audit reference
- `schema.rs:1245-1257` — `relation_fields_matches_struct` drift test matches the ac-09 reference
- `verbs.rs:3849-3857` — `relation_kind_str` exhaustive `match` matches the ac-09 reference; adding `Respects` is a one-arm change
- `verbs.rs:3822 / 3839 / 5653 / 5656 / 5685 / 5691` — six `relation.card` / `r.card` read sites that need the `Option` widen (the `args.card` matches at 5536/6819 etc. are CLI arg reads, not Relation reads — correctly excluded by the implementer's manual filter)
- `.orbit/choices/0020-shell-scripts-to-rust-verbs.yaml` header — `id: '0020'` confirms the bare-numeric form ac-07/08 now uses
- `orbit-state/Cargo.toml:10` — `version = "0.4.35"` confirms this branch is already rebased on #33; ac-10's dynamic bump rule yields 0.4.36
- Top-level `CHANGELOG.md` exists and matches the path pin in ac-10

Ship it.
