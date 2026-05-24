Sweep of stale "bead" references across the live substrate. Found while running the brownfield-setup-friction distill (card 0017's slug `setup-is-bead-aware` was the trigger). Card 0017 itself has been renamed to `0017-setup-is-orbit-state-aware`; this memo captures the remaining surface for a follow-up cleanup pass.

## Stale references in live cards

- **`.orbit/cards/0016-bead-native-cold-fork-reviews.yaml`** — both slug and body. Card describes review-spec/review-pr operating against "the bead's structured acceptance field" via `bd show` and `parse-acceptance.sh`. The cold-fork-review mechanism has migrated to orbit-state; the bead-native framing is the *prior* mechanism. Likely needs a full rewrite — possibly retired-and-replaced rather than renamed.

- **`.orbit/cards/0008-consolidated-orbit-artefact-folder.yaml:60`** — notes entry: "The hidden directory mirrors how .git/ and .beads/ work — tool-managed state in one place." Stale analogy; .beads/ is no longer the comparison point. Swap for orbit-state's `.orbit/state.db` or drop the analogy.

- **`.orbit/cards/0010-objective-functions.yaml:52`** — references `.orbit/discovery/beads-flow.md` as a source. Discovery file is historical; reference is fine but worth noting.

- **`.orbit/cards/0020-orbit-state.yaml:32`** — body: "no separate database sync, no Dolt push/pull, no .beads/ reinit dance." Historical positioning — correct context (this IS the card that replaced beads), so leave.

## Stale references in live SKILL.md files

- **`plugins/orb/skills/card/SKILL.md:52`** — "the corresponding bead AC blocks all subsequent ACs by declaration order". Stale — describes how ACs flow into bead substrate, but the substrate is now orbit-state.
- **`plugins/orb/skills/card/SKILL.md:75`** — "propagates to bead AC as [gate] via promote.sh". Stale mechanism.
- **`plugins/orb/skills/tabletop/SKILL.md:51`** — references `0017-setup-bead-aware` (note: also wrong slug format — missing "is"). Now doubly stale after the rename to `0017-setup-is-orbit-state-aware`.

## MADR records — historical context, leave alone

- **`.orbit/choices/0011-beads-execution-layer.yaml`** — the decision that chose beads as the execution substrate. Per MADR convention, retired decisions stay as records; verify `status: superseded` is set and the supersedence pointer is correct (probably to choice 0015 orbit-state-architecture).
- **`.orbit/choices/0013-bead-acceptance-field-as-cold-fork-substrate.yaml`** — same; check status field, leave content.
- **`.orbit/choices/0015-orbit-state-architecture.yaml`** — names beads as the predecessor it replaces. Correct historical context.

## Functionally correct — leave alone

- **`orbit-state/crates/cli/src/bin/migrate.rs`** (21 mentions) — this IS the bead-to-orbit-state migration binary. `.beads/ → .beads-archive/` references are operationally correct for projects still on the old layout.

## Historical artefacts — leave alone

- `.orbit/archive/specs/*` with bead in name or body — explicit archive.
- `.orbit/discovery/beads-flow.md`, `.orbit/discovery/beads-trial-findings.md` — discovery records of the bead trial; historical, not live state.
- Old open specs (2026-05-09-orbit-method-md, 2026-05-16-memos-own-folder) and choice 0020-shell-scripts-to-rust-verbs.yaml with old slug references — text-context only, not parsed; either update in passing when next touching them or leave.

## Suggested disposition

One small cleanup spec, likely against an existing card (0020-orbit-state or a new "documentation hygiene" card if there isn't one). ACs would be:
- card 0016 rewritten or retired
- card 0008 notes line updated
- card/SKILL.md prose lines updated  (×2)
- tabletop/SKILL.md slug reference updated
- choices 0011/0013 status fields verified as `superseded`

Not urgent. The substrate is functionally correct; this is naming hygiene that becomes more important as new agents onboard against the substrate and read these files as definitions.

## Source

Discovered 2026-05-24 during /orb:distill on `2026-05-24-brownfield-setup-friction.md`. Triggered by user flagging card 0017's `setup-is-bead-aware` slug as stale during rally-planning conversation.
