# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-port-promote-sh
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 4 |
| 2 — Assumption & failure | content signal (cross-system schema change in AC-11) + MEDIUM finding in Pass 1 | 2 |
| 3 — Adversarial | not triggered — Pass 2 surfaced no cascade or rollback concerns once AC-11 is split off | — |

## Findings

### [HIGH] AC-11 silently embeds a non-trivial schema change as a "first sub-task"
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-11 asks cards 0005 and 0006 to gain `relations:` entries `{ choice: '0020-shell-scripts-to-rust-verbs', type: respects, reason: ... }`. Direct inspection of `orbit-state/crates/core/src/schema.rs:549-563` shows the current `Relation` struct hardcodes `pub card: String` (no `choice:` field) and `RelationKind` exposes only `DependsOn / Feeds / Supersedes / SupersededBy` — **neither** a `choice:` target field **nor** a `respects` kind exists today. The AC's parenthetical "extend the variant set as the first sub-task of this AC and add a round-trip test; otherwise just write the edges" treats this as a maybe-trivial conditional. It isn't trivial:

1. The `Relation` struct currently uses a flat `card: String` field, not an enum-tagged target. Adding `choice:` means either (a) adding a sibling field with serde validation that exactly one is set, or (b) rewriting `Relation` to a tagged enum — a schema migration touching every existing `relations:` entry in the substrate, every card-validation site in core, and every consumer that destructures `relation.card`.
2. Adding `RelationKind::Respects` propagates through serde round-trip tests, every match site on `RelationKind`, plus any rendering surface (`orbit card show`).
3. The precedent spec (2026-05-24-port-acceptance-shim) declined to bundle the analogous edge-writes despite choice 0020's Consequences clause asking for them — its AC-11 touched only the choice table, leaving the relations work for a separate spec. This spec re-imports that deferred work plus a schema change, inside one AC labelled `code` by implication.

**Evidence:**
- `orbit-state/crates/core/src/schema.rs:549-563` — current `Relation` and `RelationKind` definitions, no `choice:` / `Respects` variants.
- Real cards under `.orbit/cards/*.yaml` (e.g. `0009-mission-resilience.yaml:51`) use `relations: [{card: ..., type: depends-on, ...}]` — confirms the flat `card:` field is load-bearing across the substrate.
- Precedent spec `2026-05-24-port-acceptance-shim/spec.yaml` AC-11 omits the relations-edge writes despite the same clause in choice 0020.

**Recommendation:** Split AC-11 in two:
- **New AC (schema)** — `code`, `gate: true`: extend `Relation` to accept choice targets (either an enum-tagged target or a sibling `choice:` field with mutually-exclusive validation) and add `RelationKind::Respects`. Round-trip test against a fixture card with a `choice:` relation. Land before any card edits.
- **Current AC-11 (edge writes)** — `doc`, gates on the schema AC: writes the actual `{ choice: 0020-shell-scripts-to-rust-verbs, type: respects, reason: ... }` entries to cards 0005 and 0006.

If the author prefers to keep AC-11 in this spec, the schema extension belongs as its own AC with a gate dependency, not a parenthetical "first sub-task". Alternatively, defer AC-11 to a follow-up spec (matches the port-acceptance-shim precedent) and have this spec land only the verb port + decommission, citing choice 0020's table update in AC-10.

### [MEDIUM] AC-05 drops the `--root` flag the shim accepts
**Category:** missing-requirement
**Pass:** 1
**Description:** The shim (`plugins/orb/scripts/promote.sh` lines 30, 81-88) accepts `--root <path>` to operate against a non-CWD layout, and passes it through to `orbit spec create` and `orbit canonicalise`. The spec's verb surface (`orbit spec promote <card-path> [--dry-run]`) and `SpecPromoteArgs { card_path: String, dry_run: bool }` make no mention of root selection. Other `orbit` verbs accept `--root` at the CLI layer (`orbit_root_args` in the shim), so the verb itself may not need a field — but the spec should say so explicitly, otherwise a path-validation AC that rejects absolute paths "outside the layout root" becomes ambiguous when the layout root is non-default.
**Evidence:** `plugins/orb/scripts/promote.sh:30, 47, 81-88, 153-156` — shim's `--root` plumbing. AC-05 references "layout root" without specifying how the verb knows what that root is.
**Recommendation:** Add one line to AC-05 or the implementation-notes block clarifying that the verb takes the layout from the CLI's standard `--root` flag (no new field on `SpecPromoteArgs`), so the path-rejection semantics tie to that resolved root.

### [MEDIUM] AC-08's grep verification regexes against the wrong corpus
**Category:** test-gap
**Pass:** 1
**Description:** AC-08 says "verified by `rg --no-heading 'promote\.sh' plugins/orb/skills/ | wc -l` returning 0. Five SKILL.md files implicated per the pre-flight audit: drive 3, card 2, rally 1". A live `rg promote\.sh plugins/orb/skills/` from this review returned **6 matches across 3 files**, not 5 across 5:
- `card/SKILL.md:52` — a `gate: true` mention containing the word "promote.sh" as part of prose ("propagates to bead AC as [gate] via promote.sh") — **not a call site**, just documentation prose.
- `card/SKILL.md:75` — same: a comment in a card YAML example.
- `drive/SKILL.md:110` — actual call site (`SPEC_ID=$(plugins/orb/scripts/promote.sh ...)`).
- `drive/SKILL.md:113` — prose mention, not a call site.
- `drive/SKILL.md:735` — actual call site (`NEW_SPEC=$(plugins/orb/scripts/promote.sh ...)`).
- `rally/SKILL.md:231` — actual call site.

Three real call sites and three prose mentions. The grep check conflates them. Rewriting the two prose mentions in `card/SKILL.md` to say `orbit spec promote` is a docs nicety, not a functional requirement — but the AC's pass/fail gate treats them identically. The "5 SKILL.md files (drive 3, card 2, rally 1)" tally also overcounts: it appears to be counting matches-per-file rather than files (3 files, 6 matches).
**Evidence:** Live grep against the current tree returned 6 matches in 3 files, not "five SKILL.md files" as the AC asserts.
**Recommendation:** Either (a) tighten the regex to actual invocations (`rg 'plugins/orb/scripts/promote\.sh'`) and accept that prose mentions in `card/SKILL.md` are part of the same rewrite, or (b) keep the broad regex but correct the AC's pre-flight tally to "3 SKILL.md files, 6 matches (3 invocations + 3 prose mentions)". Option (a) is cleaner because it matches what AC-09's invariant actually cares about — no functional dependence on the shim.

### [LOW] AC-12 CHANGELOG entry pins the wrong version
**Category:** missing-requirement
**Pass:** 1
**Description:** AC-12 says "bumped version (0.4.34 → 0.4.35)". The current installed plugin is **0.4.33** per the skill cache path and the most recent commit (`fc279c4 Bump version to 0.4.33`). The port-acceptance-shim precedent spec mentions "v0.4.34, PR #32" in the tabletop note, but if that bump landed on `main` already the working tree should reflect it. If 0.4.34 has not yet been published, this spec lands at 0.4.34, not 0.4.35.
**Evidence:** Recent commits show `fc279c4 Bump version to 0.4.33` as the latest version bump; `19ab025` and `b96d02d` close rallies/specs without bumping. Skill cache path `/home/hugh/.claude/plugins/cache/orbit/orb/0.4.33/` confirms 0.4.33 is the latest published version.
**Recommendation:** Verify the current version before implement starts (likely 0.4.33 or 0.4.34 depending on whether port-acceptance-shim's CHANGELOG entry is already cut). Rewrite AC-12's version range to "next bump" or to whichever number `plugins/orb/.claude-plugin/plugin.json` actually reads at implement start, so the spec doesn't ship a CHANGELOG entry under a phantom version.

### [LOW] Spec slug derivation lossy on multi-card-id cards (Pass 2)
**Category:** assumption
**Pass:** 2
**Description:** AC-01 says the verb materialises the spec at `.orbit/specs/<today-iso>-<slug-without-NNNN>/`. The shim derives the slug via `re.sub(r"^\d+-", "", basename)` against `os.path.basename(card_path)` — works for `0005-drive.yaml` → `drive`. The verb is expected to mirror this. Failure mode: if a card filename ever drops the `NNNN-` prefix (e.g. someone hand-creates `.orbit/cards/manual-card.yaml`), the regex no-ops and the derived spec id becomes `2026-05-25-manual-card`, which is fine. But if two cards in the same day promote to slugs that collide (e.g. `0005-drive.yaml` and `0042-drive.yaml`), the second call hits AC-04's `Error::conflict` correctly — so this is handled. Assumption validated; flagging for awareness only.
**Evidence:** `promote.sh:80-86` regex + AC-04 conflict semantics.
**Recommendation:** None required — AC-04 covers the failure mode. Worth a one-line acknowledgement in the implementation notes if the verb's tests don't already include a slug-collision case.

### [LOW] AC-07 lets the implementer choose between two paths but doesn't capture the choice in the spec record (Pass 2)
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-07 offers two paths (a) compat-wrapper or (b) delete-with-rewrites, with (b) "preferred when a single PR can land verbs + rewrites + deletion cleanly". This is fine for the implementer's autonomy, but the spec's `acceptance_criteria` array doesn't carry a `notes:` field to record which path was actually taken. Review-pr will have to infer from the diff. Cheap fix: when implement closes AC-07, note the chosen path in the spec's labels or in a closing comment. Not blocking — just a small substrate-shape nicety.
**Evidence:** AC-07 text.
**Recommendation:** Optional. If you want machine-checkable provenance, add a `labels:` entry like `path:b-delete-with-rewrites` when implement closes AC-07. Otherwise let review-pr read the diff.

---

## Pass 1 gate-AC text check (deterministic rules)

Three gate ACs in this spec: `ac-01`, `ac-06`, `ac-08`. All three pass the deterministic rules (non-empty, not a placeholder token, ≥20 chars trimmed). No MEDIUM finding from rule 5.

## Honest Assessment

The verb port itself is mechanical — the shim's logic is small, the precedent spec landed cleanly yesterday, the source-of-truth lifts are all named in the tabletop. ACs 01–10 and 12 are well-shaped, testable, and align with the port-acceptance-shim pattern.

The biggest risk is **AC-11**. It bundles a schema change (extending `Relation` to accept choice targets, adding `RelationKind::Respects`) inside an AC that reads as if writing a YAML entry is the main task. The schema extension is the *real* work — touching the core crate, every match site on `RelationKind`, serde round-trips, and the validation rules that currently keep `card:` as the only relation target — and it's currently hiding behind a parenthetical. The precedent spec sidestepped this by leaving the equivalent edge-writes for a follow-up. This spec should either (a) split AC-11 so the schema change is its own AC with `gate: true`, ahead of the edge-writes; or (b) defer AC-11 entirely to a dedicated "relations: choice targets" spec, matching the precedent. Pick one before implement starts. The other findings are local cleanups — AC-05 wants one line of clarification, AC-08 wants its grep + pre-flight tally aligned with reality, AC-12 wants a version sanity-check.
