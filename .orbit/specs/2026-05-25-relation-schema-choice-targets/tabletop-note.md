# Tabletop Note: Relation schema gains choice-target support + `respects` kind

**Date:** 2026-05-25
**Cards:** .orbit/cards/0020-orbit-state.yaml, .orbit/cards/0005-drive.yaml, .orbit/cards/0006-rally.yaml
**Mode:** closed
**Choice:** .orbit/choices/0020-shell-scripts-to-rust-verbs.yaml — substrate-shaped shell scripts under `plugins/orb/scripts/` migrate into orbit Rust verbs

---

## What good looks like

Cards can carry `relations:` entries pointing at choice files, not just other cards — and the relation kind can be `respects` (a card honours a choice's policy), not just the four card-to-card kinds (`depends-on`, `feeds`, `supersedes`, `superseded-by`). The schema bump is the smallest possible: additive optional field, additive enum variant, custom validation that exactly one of `card:` / `choice:` is set per entry. Existing card-only relations parse byte-identically; new relations on cards 0005 + 0006 reference choice 0020 to honour the policy clause that has been promising the edges since 2026-05-09. The follow-up edge write on card 0017 (when `setup-method.sh` ports) becomes a one-line addition with no further schema work.

## Pinned approach

- Schema change is **additive**: `Relation.card` softens from `String` to `Option<String>` with `#[serde(default, skip_serializing_if = "Option::is_none")]` (existing on-disk shape `card: <id>` parses + serialises byte-equal — `Option::is_none` skip plus `default` keeps both directions clean). A new `choice: Option<String>` field joins it with the same serde discipline. A new `Respects` variant lands on `RelationKind`. Custom validation enforces exactly-one of `card`/`choice` per entry.
- Sibling PR sequence: this lands AFTER PRs #32 (0.4.34, port-acceptance-shim) and #33 (0.4.35, port-promote-sh) merge. Rebases are clean — the changes touch `schema.rs` (untouched by #32/#33) and add two card-yaml edges (untouched files).
- The edge writes for cards 0005 + 0006 are in scope here. Card 0017's edge (for the future `setup-method.sh` port) is deferred to that port's own spec.

## Deferred items

- Card 0017's `relations:respects → choice 0020` edge — lands when `setup-method.sh` ports as the third opportunistic migration under choice 0020.
- A generic `RelationTarget` enum that could carry other-kind targets in future (specs, memos, topology entries) — over-engineering for v1. Current scope: just `card` and `choice` as alternatives. If a fourth target type appears later, refactor to an enum then.
- Any `Card.relations[].card` consumer code that breaks when the field becomes `Option<String>` — discovered at compile time; fix-as-found during implement.
- Topology / graph / overview / audit verbs that iterate `card.relations` — most read `relation.card` directly. They need to skip choice-target relations OR widen their projection. Either is a one-liner; mark as detour escalation during implement if larger than expected.

## Implementation notes

- **Verb-surface fork — pick during implement, both clean:**
  - **(a) Two optional fields + post-parse validation.** `card: Option<String>` and `choice: Option<String>`, both `default` + `skip_serializing_if`. Validation function `validate_relation_target(r: &Relation) -> Result<()>` invoked from `parse_yaml::<Card>` (or via a custom `Deserialize` impl on `Relation`) returns `Error::malformed` if both/neither set. Preserves `deny_unknown_fields` on `Relation`.
  - **(b) `#[serde(untagged)] + #[serde(flatten)]` with a `RelationTarget` enum.** More idiomatic serde shape (the type system encodes "exactly one"). Conflicts with `Relation`'s `deny_unknown_fields` — either drop it from `Relation` (the wrapper) and keep it on the variants, or accept that unknown sibling fields are silently dropped.
  - **Pre-recommendation: option (a)** — preserves `deny_unknown_fields`, keeps the `Relation.card` Option pattern small, and the validation function is 5 lines. Untagged-flatten is elegant but the deny-unknown interaction is fiddly.
- **Backward compat verification:** `orbit verify` must return clean after the schema change (round-trips every canonical card through parse → serialise → byte-equal). Add a parity test for an existing card-target relation AND a new choice-target relation.
- **`Respects` variant:** add to `RelationKind` enum at `orbit-state/crates/core/src/schema.rs:558`, kebab-case-serialised as `respects` (per `rename_all = "kebab-case"` on the enum).
- **Edge writes on cards 0005 + 0006** — direct YAML edits, then `orbit canonicalise` to normalise byte form. The new entries:
  ```yaml
  - choice: 0020-shell-scripts-to-rust-verbs
    type: respects
    reason: drive's promote stage consumes the orbit spec promote Rust verb that this choice authorised the shell-script migration to (per choice 0020 Consequences)
  ```
  (And the mirror entry on card 0006 with `rally's promote stage` substituted.) Each card gets one new entry appended; existing relations untouched.
- **Consumer audit:** grep for `\.card\.` and `relation\.card` and `Relation \{ card` across the codebase — every consumer needs to handle `Option`. Most will be:
  ```rust
  if let Some(card_id) = &relation.card { ... }
  ```
  Audit-only consumers may need to skip choice-target relations entirely (they don't currently model them). Mark each touched site in the spec's note stream as you go.
- **Version bump:** 0.4.35 → 0.4.36 (stacked on #33). CHANGELOG entry under [0.4.36] names the schema bump, the `respects` variant, the two card edges, and the consumer audit's findings.
- **Recommended `ac_type` distribution:**
  - `code` — schema field additions, `Respects` enum variant, validation function, parity tests (existing card-target + new choice-target), consumer-audit fixes (count discovered during implement).
  - `doc` — card 0005 + 0006 yaml edits, choice 0020 prose update to mark the edge writes done, CHANGELOG entry.
  - No `ops` / `observation`.
