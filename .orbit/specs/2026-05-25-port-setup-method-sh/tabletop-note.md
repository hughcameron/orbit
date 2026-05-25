# Tabletop Note: Port setup-method.sh to native `orbit setup` verb

**Date:** 2026-05-25
**Cards:** .orbit/cards/0017-setup-is-orbit-state-aware.yaml, .orbit/cards/0020-orbit-state.yaml
**Mode:** closed
**Choice:** .orbit/choices/0020-shell-scripts-to-rust-verbs.yaml — substrate-shaped shell scripts under `plugins/orb/scripts/` migrate into orbit Rust verbs

---

## What good looks like

`/orb:setup` stops shelling out to a 244-line python3-in-bash script for the load-bearing §6 work (CLAUDE.md legacy-block migration, METHOD.md + STYLE.md copy + byte-compare drift detection, idempotent @-import appends). The work lands as a single native verb `orbit setup files <project-root>` that performs all three steps atomically — same-batch legacy migration when legacy blocks are present, copy + drift-prompt when they're not. Interactive prompts move to the CLI layer; the core verb takes pre-resolved answers via typed args so MCP callers and tests get the same surface without stdin gymnastics. Same-commit decommission: shim and its dedicated test (`test-setup-method.sh`) delete in one commit with the SKILL.md rewrite. Card 0017 picks up the deferred `relations:respects → choice 0020` edge.

## Pinned approach

- Choice 0020 names the direction; the policy has shipped two precedent ports this session (`orbit-acceptance.sh` → PR #32 / 0.4.34, `promote.sh` → PR #33 / 0.4.35). Mechanical to mirror.
- Decommission discipline per choice 0020's "Test-vs-prod parity": the shell shim stays in place until grep on every consumer SKILL.md returns clean; the shim and its dedicated test then delete in one commit with the last call-site rewrite. Both prior ports took the wrapper-skip single-PR path (verbs ship + rewrite + delete in one commit); same shape recommended here.
- Card 0017's `relations:respects → choice 0020` edge lands in this spec — the Relation schema added `choice:` targets and `Respects` kind via PR #34 (0.4.36, merged), so the edge write is now mechanical (no schema work).

## Deferred items

- `code-investigate-mark.sh` discrimination — a separate `/orb:choice` (does it port at all?) before any port spec; post-dates choice 0020 and sits on the substrate-vs-tooling boundary.
- `test-sidecar-layout.sh` cleanup — already-broken test now also references the deleted `promote.sh` from PR #33; separate hygiene spec.
- Migration of the four other shell tests under `plugins/orb/scripts/tests/` that reference setup-method.sh (`test-setup-method.sh` is the dedicated one; the other three may have incidental refs — check during implement). Per choice 0020 test-vs-prod parity, only the dedicated test moves with the shim's deletion.
- Any /orb:setup §1-§5 work that still shells out (the `/orb:setup` skill has multiple phases; §6 is what this spec ports). Other phases that may still use scripts get their own ports.
- A possible `orbit setup` verb family with sibling verbs (`orbit setup init`, `orbit setup diagnose`) — out of scope; this spec ships the one verb the shim covers, leaving the family open for future expansion.

## Implementation notes

- **Verb-surface fork — pick during implement, single-verb preferred.** Two coherent shapes:
  - **(a) ONE combined verb** `orbit setup files <project-root>` that performs all three §6 sub-steps (legacy detect+migrate, copy canonicals with drift prompt, ensure @-imports) atomically. Matches the shim's atomic-batch contract verbatim; one call site in SKILL.md rewrites cleanly to one verb invocation. **Pre-recommendation.**
  - **(b) Three sub-verbs on an `orbit setup` family** — `setup migrate-legacy`, `setup copy-canonicals`, `setup ensure-imports`. More flexible but the SKILL.md call site is sequential (the shim does all three in one invocation); the family shape adds surfaces without buying composability the shim doesn't already give.
- **Interactive prompts: CLI layer, not core.** The shim reads from stdin when `--answer-*` flags aren't passed. The Rust port should keep that strict separation:
  - Core verb takes pre-resolved typed args (`SetupFilesArgs { project_root: PathBuf, legacy_action: LegacyAction, method_drift_action: DriftAction, style_drift_action: DriftAction }` where `LegacyAction::{Migrate, Refuse, Skip}` and `DriftAction::{Overwrite, Keep, Skip}` are enums).
  - CLI layer runs interactive prompts (via the `dialoguer` crate or hand-rolled stdin reads) when flags aren't passed, then calls the core verb with the resolved actions. The existing `--answer-legacy / --answer-method-drift / --answer-style-drift` flags map directly to the core args.
  - MCP callers always pass typed args (no prompts) — same surface as the CLI's non-interactive path.
- **Atomic legacy migration.** The shim's Python heredoc strips three legacy section blocks from CLAUDE.md (`## Workflow (orbit)`, `## Orbit vocabulary`, `## Current Sprint`) via regex and rewrites the file. Rust port uses `regex` crate (already a workspace dep at `orbit-state/Cargo.toml:46`); the pattern shape is `(^|\n)<marker>\s*\n.*?(?=\n##\s|\n#\s|\z)` per the shim. The migration is atomic in the shell sense: if legacy is detected and user declines migration, the entire setup REFUSES — no canonical files copied, no @-imports added. Preserve that contract.
- **Byte-compare drift detection.** For each canonical (METHOD.md, STYLE.md), if destination exists and differs from source by `cmp`-equivalent comparison, prompt before overwriting. Default action when no prompt answer: keep existing (no-op). Rust port: use `std::fs::read` + bytewise equality, NOT `same_file::is_same_file` (which checks inode identity, not content).
- **@-import idempotency.** The shim appends `@.orbit/METHOD.md` and `@.orbit/STYLE.md` to CLAUDE.md if they're not already present. Each line checked via `grep -Fxq`. Rust port: read CLAUDE.md, split on lines, check for exact-line presence, append with leading blank line if file ends with non-blank content. Preserve trailing newline handling (the shim ensures `\n` line ending before appending).
- **Source-of-truth lifts:** `plugins/orb/scripts/setup-method.sh` lines 100-244 for the three sub-step bodies. Existing CLI/MCP precedent: `orbit-state/crates/cli/src/main.rs` for the subcommand dispatch + interactive arg-resolution pattern (look at how `spec close --force` handles flags); `orbit-state/crates/core/src/verbs.rs` for the verb impl pattern (look at the AC verbs added in PR #32 as the most recent template).
- **Card 0017 relation edge.** The Relation schema now supports `choice:` targets and the `Respects` kind (PR #34, 0.4.36, merged). Card 0017 gains `{choice: '0020', type: respects, reason: "setup's canonical-files step uses the orbit setup Rust verb that this choice authorised the shell-script migration to (per choice 0020 Consequences)"}` appended to its existing relations. Choice id form bare numeric per id-conventions.md.
- **Recommended `ac_type` distribution:**
  - `code` — verb impl + CLI dispatch + interactive prompt layer + core arg-struct typing + parity tests (legacy-migrate / drift-overwrite / drift-keep / @-import-idempotent / refuse-on-legacy-decline branches) + SKILL.md call-site rewrite (verified by `rg setup-method\.sh plugins/orb/skills/ | wc -l == 0`).
  - `doc` — choice 0020 table update (mark setup-method.sh row migrated), CHANGELOG entry under [0.4.37], shim + test deletion commit, card 0017 relations-edge write.
  - No `ops` / `observation` ACs expected.
