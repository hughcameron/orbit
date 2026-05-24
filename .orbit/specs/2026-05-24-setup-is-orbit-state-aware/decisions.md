# Decision pack — setup is orbit-state-aware (drive 2026-05-24-setup-is-orbit-state-aware)

Scope per rally proposal: add a 5th setup state for `orbit/`-no-dot wrappers, stop auto-seeding substrate-typed topology entries outside the orbit-plugin repo, and detect-and-warn (no auto-convert) on `decisions/` → `choices/`.

Locked upstream (do not re-derive):
- `decisions/` → `choices/`: detect-and-warn only, no auto-conversion.
- The sister drive on card 0039 ships an undotted-substrate conformance finding that suppresses the existing `canonical-files-missing` finding. These decisions assume that suppression exists.

Evidence anchors:
- Current state table: `plugins/orb/skills/setup/SKILL.md:24-31`.
- Stale skill description ("creates orbit/ directory"): `plugins/orb/skills/setup/SKILL.md:3`.
- Current brownfield migration block: `plugins/orb/skills/setup/SKILL.md:48-98`.
- Topology auto-seed entries (substrate-typed, non-conditional): `orbit-state/crates/core/src/verbs.rs:4451-4532`.
- Conformance `canonical_file_findings` (the suppression target): `orbit-state/crates/core/src/verbs.rs:4087-4140`.
- Memo source (#2 topology, #4 decisions/): `.orbit/memos/2026-05-24-brownfield-setup-friction.md`.

---

## 1. Name of the 5th setup state

**Context.** The current 4-state table is the 2×2 of (`orbit/`-with-dot present?) × (any bare `cards/`/`specs/`/`decisions/`/`discovery/` at root?). The arcform case — `orbit/` with no dot, containing the wrapped substrate — sits outside that grid because the dir name `orbit/` exists but the substrate is tool-invisible (the CLI reads `.orbit/` only). The state needs a name that survives appearing in conformance findings, error messages, and the SKILL.md table.

**Options.**
- (a) `wrapped-undotted` — descriptive, names the structural feature (substrate IS wrapped, just under the wrong dir name).
- (b) `legacy-orbit-folder` — historical, names where this layout came from (a pre-`.orbit/` plugin era).
- (c) `dotless-wrapper` — short, names the missing dot.

**Trade-offs.**
- (a) reads as a structural classifier alongside `greenfield` / `brownfield` / `mixed` / `idempotent` — all describe substrate shape, not origin. Conformance finding strings stay durable when the historical reason ("legacy") fades. Slightly clinical.
- (b) carries useful semantics for the operator ("this is a known pre-orbit-state shape") but bakes a temporal claim into the substrate. In two years "legacy" will mean something else, and the state will still exist for any operator who got there by hand. Also collides with the existing word "legacy" already used for CLAUDE.md blocks (SKILL.md §6a) and reconcile sidecars.
- (c) shorter than (a) but loses the wrapped-vs-bare distinction. Reads close enough to `mixed` to confuse a fast skim.

**Recommendation.** **(a) `wrapped-undotted`.** It composes with the existing classifier vocabulary (all structural, none historical) and avoids the "legacy" overload already in play for CLAUDE.md blocks and reconcile output. The conformance finding shipped by the sister drive on card 0039 will reference this state slug — it needs to outlast the era that created it.

---

## 2. `orbit/` → `.orbit/` migration mechanism

**Context.** Setup already uses `git mv` for the bare-dir brownfield case (SKILL.md:75-83), and the existing rationale (SKILL.md:55, "Untracked files will be left behind by `git mv` — they need to be reported to the author") is specific to that move. The new state moves a whole wrapper dir, not four sibling dirs. arcform's session moved this exactly — that commit (`176fb61`, named in the memo) used a single rename.

**Options.**
- (a) `git mv orbit .orbit` — single rename of the wrapper directory.
- (b) `git mv orbit/cards .orbit/cards && git mv orbit/specs .orbit/specs && ...` — per-subdir, matching the bare-brownfield pattern.
- (c) Fresh `mkdir .orbit && git mv orbit/* .orbit/ && rmdir orbit` — copy-and-remove rather than rename.

**Trade-offs.**
- (a) is one git operation, preserves history cleanly, fails fast if `.orbit/` already exists (which would be the "mixed-undotted" case worth refusing on). Matches the structural reality — the only thing wrong is the dir name.
- (b) makes the operation parallel to bare-brownfield's per-dir pattern, but creates N rename operations where the structure already exists as one dir. The "untracked residue" rationale weakens here — untracked files inside `orbit/` move under (a) trivially because the whole dir renames; they get stranded under (b).
- (c) loses git rename detection cleanly only at git's similarity heuristic mercy. No correctness gain over (a).

**Recommendation.** **(a) `git mv orbit .orbit`.** It's the structurally honest move (rename the dir whose name is wrong), it preserves history cleanly, and it makes untracked-residue handling a non-issue because nothing is left behind. The §3b residue scan in SKILL.md can be skipped on this path entirely. If `.orbit/` already exists alongside `orbit/`, refuse with a "mixed-undotted" error (defence in depth — the state classifier in §1 already caught this, but a `git mv` to an existing target would fail loudly anyway).

---

## 3. Where the 5th-state detection logic lives

**Context.** The current 4-state classification lives in the bash setup script's prelude (SKILL.md:22-31; `setup-method.sh` doesn't own state classification — it owns the canonical-files step only). The orbit CLI has no `orbit setup` verb today; setup is a skill that calls `setup-method.sh` and runs shell from SKILL.md prose. The sister drive on card 0039 ships a conformance finding for `wrapped-undotted` — that finding needs the same detector.

**Options.**
- (a) Detection in the SKILL.md bash prelude only — the skill classifies, the CLI doesn't.
- (b) Detection in a new `orbit setup detect` (or `orbit audit substrate-layout`) CLI verb — single source of truth callable from both the skill and the conformance audit.
- (c) Detection in both — bash mirrors the CLI but duplicates the predicate.

**Trade-offs.**
- (a) keeps the existing structure (skill owns state machine, CLI owns substrate verbs). But the sister drive's conformance finding needs to fire from `orbit audit conformance` — running the audit shouldn't require shelling out to bash. If detection only lives in bash, the audit needs its own re-implementation, and the two can drift. The arcform memo names exactly this drift class as "the trust contract breaks".
- (b) puts the predicate in one place — Rust, schema-aware, testable. The skill calls `orbit ...` to get the classification; the audit calls the same predicate internally. Single test surface (`crates/core/src/verbs.rs` siblings to `audit_conformance_at` at line 3902). Adds one verb, but it's a thin reader.
- (c) is the worst of both — double maintenance, guaranteed drift. The arcform incident is the case study.

**Recommendation.** **(b) one detector in `orbit-state`, called from both the skill and the conformance audit.** Place it next to `audit_conformance_at` so the substrate-layout finding family stays colocated with the other finding families. The CLI surface can be `orbit audit substrate-layout --json` or rolled into `audit_conformance` directly as a new finding family (the sister drive on 0039 likely already names the shape). The skill replaces its bash classification block with a call to this verb and routes on its output. This is the same direction choice 0020 (`shell-scripts-to-rust-verbs`) already points the substrate.

---

## 4. Topology auto-seed scoping: how setup recognises the orbit-plugin repo

**Context.** Memo item #2 surfaces this as the deepest of the topology bugs: the 5 seed entries (`cards`/`choices`/`memories`/`specs-substrate`/`topology` — see `verbs.rs:4494-4532`) point at `orbit-state/crates/core/src/schema.rs`, a path that only exists in this repo. In any other project they're a category error — the substrate types are owned by the plugin, not the project. Setup needs to decide whether seeding makes sense, which requires recognising "this is the orbit-plugin repo itself".

**Options.**
- (a) Detect by presence of `plugins/orb/` AND `orbit-state/crates/core/Cargo.toml` at the repo root.
- (b) Detect by presence of `.claude-plugin/` (the marketplace manifest dir at repo root, unique to plugin repos).
- (c) Detect by a config marker — operator sets `plugin_repo: true` in `.orbit/config.yaml` (or similar) on the plugin repo itself; default `false` everywhere.

**Trade-offs.**
- (a) reads what's actually there — the plugin source IS those two paths. Zero new substrate. But "presence of paths" is implicit; any project that incidentally has both names (unlikely but possible) would trip the heuristic and reintroduce the seed problem.
- (b) `.claude-plugin/` is the marketplace's own marker for "this directory contains a plugin". Almost identical signal to (a) with less surface area to check. Still implicit-by-filesystem.
- (c) explicit opt-in. Substrate-bearing — operator declares the role. Survives reorganisation of the plugin layout. Costs one config field and one migration step (this repo needs to set the flag on first orbit-version bump after this lands). The arcform-style failure mode (seed pointers garbage in a non-plugin repo) is impossible because seeding is gated on an explicit opt-in.

**Recommendation.** **(c) explicit `plugin_repo` config flag.** The memo's framing is "category error" — the seed entries are conceptually wrong for any project that isn't the plugin source. Categorical-error prevention deserves an explicit signal, not a filesystem heuristic that could misfire. Default is `false`; setup in non-plugin projects scaffolds `.orbit/topology/` as an empty directory (per memo's suggested fix) and a one-line README pointing at `/orb:topology` for opt-in entry authoring. Setup in the plugin repo (where `plugin_repo: true` is set) seeds the 5 substrate-typed entries. The one-time cost of setting the flag on this repo is paid once; the safety property holds forever.

Secondary check: combine (c) with a `validate` step that refuses to write seeds whose `canonical_code` paths don't exist in the working tree. This catches the failure mode of an operator copying the flag into the wrong repo and gives a clean error rather than 21 silent drift entries.

---

## 5. What setup emits when it finds `decisions/`

**Context.** Locked upstream: detect-and-warn only, no auto-conversion. The MADR-to-choice-schema mapping is non-trivial; the rally proposal locked it out of scope. What's open is the emission shape — where the signal lands, who consumes it, what voice it uses. The existing brownfield prompt already names the move (SKILL.md:62: `decisions/   → .orbit/choices/`) but the prompt's promise is currently a lie because `git mv decisions .orbit/choices` (SKILL.md:81) moves the directory verbatim without converting MD → YAML.

**Options.**
- (a) Inline warning during setup migration: setup prints "found N MADR files in `decisions/`; copying to `.orbit/choices/` but `orbit choice list` will return empty until you convert MD → YAML by hand. See <pointer>." Single transactional message at the time of the migration.
- (b) Conformance finding only: setup leaves `decisions/` in place (no auto-move), and the sister drive's substrate-layout finding family adds a `decisions-not-migrated` finding that the agent surfaces on next `orbit audit conformance`.
- (c) Both: setup migrates `decisions/` → `.orbit/decisions/` (renames the dir under the dot but doesn't convert content), prints the inline warning, AND the conformance audit fires on the unmigrated content so the signal survives session loss.

**Trade-offs.**
- (a) cheapest. The signal exists exactly when the operator is looking at the output. But the operator who doesn't act on it in-session loses the signal — there's no durable surface that says "this repo has MADR files that should be YAML choices". The arcform memo names exactly this failure: setup migrated `orbit/` → `.orbit/` but "left `.orbit/decisions/` untouched" with no durable trace.
- (b) durable. The agent picks it up on every audit pass. But if setup doesn't move the files into `.orbit/`, they stay invisible to every other orbit verb (and to file-system grep that respects `.orbit/` scoping). The arcform case was specifically that the files lived at `.orbit/decisions/` (post-migration) and `orbit choice list` was empty. So "leave them at the source" loses the migration benefit.
- (c) the migration moves the dir into `.orbit/` (so subsequent layout is canonical-shaped), names the limitation inline (so the operator sees it now), AND the conformance audit catches it next session (so it survives context loss). The cost is one finding-family addition and one inline message — both small.

**Recommendation.** **(c) move-and-warn, with a conformance finding for durability.** Specifically:
- During brownfield migration (both the existing bare-dir path and the new `wrapped-undotted` path), `decisions/` gets `git mv`-ed to `.orbit/decisions/` (not `.orbit/choices/` — the directory rename should be honest about the content shape; conversion is the renaming step the operator does by hand).
- Setup prints a one-paragraph warning naming the unmigrated content and the conversion task. Update the SKILL.md migration-prompt example (currently shows `decisions/   → .orbit/choices/`) to read `decisions/   → .orbit/decisions/  (MADR files; manual MD→YAML conversion needed)`.
- A new conformance finding family — `decisions-md-unmigrated` — fires on the presence of `.orbit/decisions/` with `.md` content and no matching `.orbit/choices/<slug>.yaml`. `remediation.verb` points at a docs page or a future `orbit choice import-madr` verb (not in this drive's scope; the finding is the placeholder).

This composes with the locked decision (no auto-conversion) without leaving the substrate in the broken half-state the arcform session hit.

---

## 6. State-machine shape after the 5th state lands

**Context.** The current table (SKILL.md:24-31) is the 2×2 of `orbit/`-with-dot × bare-dirs. Adding `wrapped-undotted` means a 3rd axis: `orbit/`-no-dot also present. The total state count under (3 axes × 2 values each, minus impossible combinations) needs to be enumerated cleanly so the table is exhaustive and reviewable.

**Options.**
- (a) Keep a flat table, enumerate all reachable combinations (estimated 5-6 rows).
- (b) Reframe as a flowchart in prose — "first, check undotted; if found, route to §3'; otherwise fall through to the existing 2×2".
- (c) Two tables — one for the new orthogonal undotted axis, one for the existing 2×2 (unchanged) — with a routing note at the top.

**Trade-offs.**
- (a) the table grows but stays scannable. The "no other state" claim at SKILL.md:31 stays a sentence the reader can verify by counting rows. Easy to extend (a future 6th state slots in as a row).
- (b) prose loses the exhaustiveness check. The author has to trust the prose instead of counting rows. The arcform session is the case study for what happens when the state machine's coverage is opaque.
- (c) two tables fragment the surface — the reader has to hold two tables and a routing rule in head. Slightly worse than (a) for the same coverage.

**Recommendation.** **(a) flat table, all reachable combinations enumerated.** Specifically:

| State | Condition |
|-------|-----------|
| `greenfield` | none of `.orbit/`, `orbit/`, or bare dirs present |
| `idempotent` | `.orbit/` present, neither `orbit/` nor bare dirs present |
| `brownfield-bare` | bare dirs present, neither `.orbit/` nor `orbit/` |
| `wrapped-undotted` | `orbit/` present (wrapped substrate), `.orbit/` absent, no bare dirs |
| `mixed-bare` | `.orbit/` present AND bare dirs present (refuse) |
| `mixed-undotted` | `.orbit/` present AND `orbit/` present (refuse) |

Six rows; each is a unique condition; the union is exhaustive over the three independent axes. The existing `brownfield` row gets renamed to `brownfield-bare` for parallelism with `mixed-bare` / `mixed-undotted`. The existing `mixed` row gets renamed to `mixed-bare`. The SKILL.md description line (currently SKILL.md:3) also needs updating from "creates orbit/ directory" to "creates `.orbit/` directory" — that's a stale-substrate fix the drive picks up incidentally.

---

## Open questions deferred to implementation

- Does `orbit init` (the SQLite-index step in ac-03 of the spec) need any change for the `wrapped-undotted` path, or does it run after the rename completes? **Default: runs after.** The rename produces a `.orbit/` indistinguishable from a fresh one for init's purposes.
- The sister drive on card 0039 owns the `wrapped-undotted` conformance finding's exact severity / remediation text. This drive ships the detector; the finding wire is the sister's surface.
- Whether the `plugin_repo: true` flag's introduction warrants a migration note for THIS repo. Yes — the drive's implementation lands the flag in `.orbit/config.yaml` of the orbit-plugin repo itself as a one-line edit during the implementation pass.
