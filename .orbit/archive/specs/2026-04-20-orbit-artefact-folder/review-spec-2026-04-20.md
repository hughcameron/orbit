# Spec Review: Consolidated orbit artefact folder

**Spec:** `specs/2026-04-20-orbit-artefact-folder/spec.yaml`
**Card:** `cards/0008-consolidated-orbit-artefact-folder.yaml`
**Reviewer:** forked `/orb:review-spec` (cold read)
**Date:** 2026-04-20

**Verdict:** REQUEST_CHANGES

---

## Summary

The spec is well-structured for a cross-system refactor: clear goal, tight constraints, 20 ACs that span code/doc/gate categories, and an honest ambiguity-score (0.07) backed by an interview that explicitly records the sharpest trade-offs. Pass 1 surfaced enough concrete, medium-weight issues — brownfield partial-state handling, dirty-tree failure modes, the ac-13 regex, and two small testability gaps — that I ran Pass 2. Pass 3 was not triggered: structure is sound, the design session covered the decision space, no evidence of a missed architecture.

The "rewrite everything including quoted evidence" choice is loud but the author has already acknowledged it (interview Q6, metadata.review_notes) — I flag it at LOW with the deliberate-override noted, not higher.

None of the findings are BLOCK-level. The REQUEST_CHANGES verdict reflects that a small number of the ACs need tightening or one or two new ACs are needed before implementation can safely proceed on autopilot. All findings are fixable with spec edits — no design rework needed.

---

## Pass 1 — always-run

### Findings

#### F1 — MEDIUM: Partial-state brownfield (orbit/ AND bare dirs both present) is not modeled

`ontology_schema.setup_mode` enumerates `greenfield`, `brownfield`, `idempotent`. These are defined as disjoint by the spec text:

- greenfield: no orbit/, no bare dirs
- brownfield: bare dirs present
- idempotent: orbit/ exists, no bare dirs

But the fourth combinatorial state — **orbit/ exists AND bare cards/ (or specs/ etc.) also exists** — is unhandled. Concrete ways this arises:

1. A previous migration run aborted after `git mv cards/` succeeded but before `git mv specs/` (e.g. interrupted by ctrl-C, crash, or a `git mv` that failed mid-transaction because one sub-tree was already tracked elsewhere).
2. A user manually created `orbit/` directory first (because the CLAUDE.md snippet told them about it) but hasn't moved their existing bare dirs yet.
3. A downstream repo ran `/orb:setup` once, answered "no" to the prompt, and later ran `mkdir .orbit/choices` by hand.

Per ac-02 ("bare dirs present at root → detect + prompt"), the brownfield branch fires. Per ac-03, setup runs `git mv` of all detected bare dirs "under orbit/". That **may** work (`git mv specs orbit/` when .orbit/specs already exists will fail — git mv refuses to overwrite). So this state actually produces a *failed* migration, with unclear rollback semantics.

**Ask:** Add an AC (or extend ac-02/ac-03) explicitly covering the mixed state. Either (a) treat it as brownfield and let git mv fail loudly with a clear error, (b) merge bare dirs into orbit/ if the target subdir is absent, refuse if any collision, or (c) refuse mixed-state entirely and direct the user to finish migration manually. Option (a) + a clear error message is the minimum; (c) is the cleanest.

---

#### F2 — MEDIUM: Dirty-tree handling has concrete failure modes not addressed

ac-20 says setup does NOT refuse on a dirty tree and cites the deliberate choice. Good documentation, but the AC doesn't enumerate the actual failure modes `git mv` will hit:

- **Unstaged new files inside bare cards/ or specs/.** `git mv cards .orbit/cards` moves only tracked files. Untracked files get left behind at the old path — this creates a silent partial state where `cards/` still exists at root with the untracked file inside, defeating ac-12 ("no bare artefact dirs at root"). Then the follow-up grep gate (ac-13) also fails if the untracked file has orbit-path references.
- **Modified-but-tracked files.** `git mv` preserves modifications — this case is fine, interview Q5 is right.
- **Staged changes in the index.** `git mv` will move and re-stage; fine.
- **Files tracked but deleted in working tree (`D` status).** `git mv` on the parent dir will move the remaining tracked siblings; the deletion stays. Fine but slightly surprising.
- **A file in `.orbit/cards/` is already tracked from a prior aborted run** (overlaps with F1).

**Ask:** Either add an AC that says setup runs `git status --porcelain -- cards/ specs/ decisions/ discovery/` pre-migration, warns about untracked files in those trees, and either aborts or prompts; or explicitly document in ac-20 that untracked files will be left at the old path and the author is responsible for them. The current "proceeds with the migration" is correct by convention but under-specifies what happens to untracked content.

---

#### F3 — MEDIUM: ac-13 regex has known false-positive and false-negative cases

The verification command:

```
rg -n -t md -t yaml -t sh -t json -e '(^|[^/a-z])(cards|specs|decisions|discovery)/'
```

The negative-class `[^/a-z]` is doing two jobs: reject ".orbit/cards/" (via the leading `/`) and reject words like "scorecards/" (via the leading `a-z`). Spot-tested:

- **False-positive risk:** prose sentences that start a line or follow punctuation+space with "cards/" — e.g. a bullet beginning `cards/memos was renamed to .orbit/cards/memos` would flag. After the full rewrite these shouldn't exist, but the AC verification command will report them as violations during implementation iteration, confusing the signal.
- **False-negative risk:** capitalised references like `Cards/` (unlikely but possible in headings), or `CARDS/` (never). Also the character class excludes only `[/a-z]` — digits, underscores, slashes-other-than-forward are also fine. A reference like `_cards/` would not match — rare in a normal repo but a test fixture or generated path could contain it.
- **Scope confusion:** the verification asserts "zero hits against tracked files" but `rg` by default respects `.gitignore`, not git's tracked set. If an untracked file (e.g. a local scratch) contains `cards/`, `rg` will find it and report a miss. Use `git ls-files | xargs rg …` or explicit path filters for true tracked-only semantics.

**Ask:** Tighten the grep invocation. A safer form might be:

```
git ls-files -z \
  | xargs -0 rg -n -t md -t yaml -t sh -t json \
      -e '(^|[^/A-Za-z0-9_])(cards|specs|decisions|discovery)/'
```

Plus an explicit allow-list for expected hits (e.g. test fixtures, this review file, CHANGELOG entries documenting the migration itself, the migration decision record) — zero hits is not realistic because the CHANGELOG per ac-16 must *describe* the rewrite. ac-16 + ac-13 are in tension: the changelog entry naming "bare cards/ dirs" would match the grep unless quoted inside backticks in a way the regex tolerates (which it does — the leading backtick is `[^/a-z]`, so `` `cards/` `` matches). Worth explicitly allow-listing CHANGELOG.md for this one line or rewording.

---

#### F4 — MEDIUM: ac-05 idempotency test ignores a brownfield-then-setup regression path

ac-05 verifies idempotency by running `/orb:setup` twice on an already-migrated repo. Good. But the real regression risk is: migrate a brownfield repo, then run setup again immediately — does the second run now see a clean repo and no-op, or does it re-prompt? The spec text says migrated repos are "idempotent" state, but the detection logic as written (detect "any bare cards/specs/decisions/discovery at the root") doesn't explicitly rule on the interaction between "orbit/ exists" and "no bare dirs" (the idempotent state). ac-05 covers this implicitly via the two-run sequence but does not test brownfield-then-idempotent — run /orb:setup with bare dirs, confirm, then run again. This is the closest approximation to real upgrade flow and should be an explicit AC.

**Ask:** Add an AC or extend ac-05's verification to test the migrate-then-idempotent sequence, not only the already-migrated path.

---

#### F5 — LOW: ac-19 is a deferred post-ship AC with no gating discipline

ac-19 says downstream migration works, verified on the first real downstream /orb:setup invocation post-ship, and deferred execution is expected. That's sensible — you can't easily test against a repo that doesn't exist yet — but the exit_conditions list includes ac-19 verification as a shipping condition, which contradicts "deferred execution is expected". Either:

- ac-19 gates shipping → you need a staged downstream repo to test against before merge (e.g. a fixture or a sibling repo in `~/github/`).
- ac-19 is post-ship verification only → remove it from exit_conditions or tag it as a deferred commitment with a named tracking location (progress.md line item, or a followup card).

**Ask:** Clarify ac-19's enforcement model. Either set up a fixture for pre-merge testing, or move the ac-19 verification to a deferred-commitment list and say explicitly that shipping proceeds before it is verified.

---

#### F6 — LOW: ac-01's "discovery/ not created at setup time" introduces a genuine inconsistency

ac-01 says setup does NOT create `discovery/` (it's created ad-hoc by /orb:discovery). This matches today's setup behaviour. But the spec's ontology_schema.subdirs lists all four including `discovery`, and the CLAUDE.md snippet (ac-06) names `.orbit/cards/, .orbit/specs/, .orbit/choices/` but not `.orbit/discovery/`. Good so far. The issue: ac-02's brownfield detection includes `discovery` in the detected set ("On a brownfield repo with any of bare cards/, specs/, decisions/, or discovery/ at the root"). So setup can *move* an existing bare `discovery/` but will never *create* one. That's internally consistent — but worth a one-line note in the setup SKILL rewrite so a future reader doesn't see "we create 3 dirs but detect 4" as a bug.

**Ask:** Add a sentence to the SKILL.md rewrite that discovery/ is created ad-hoc, detected during brownfield migration, and never created eagerly. The ontology_schema.subdirs description already covers it; this is a docstring-hygiene ask.

---

#### F7 — LOW: Quoted-evidence rewrite is deliberate, noted

Constraint #3 rewrites quoted evidence in shipped review-pr-*.md and progress.md. This silently alters what prior reviews asserted they observed. The author flagged this as deliberate in interview Q6 and in metadata.review_notes. I agree it's a legitimate trade-off for a repo that uses itself and wants a clean end-state. Fidelity loss is real but small: the migration commit is the audit trail.

No ask. Noting for the record.

---

#### F8 — LOW: ac-20 verification is a grep for the word "dirty" — weak

ac-20 verifies by grepping SKILL.md for the word "dirty" or "uncommitted". That catches the documentation but doesn't verify the *behaviour*. A paired behavioural test (run setup in a repo with a dirty working tree, confirm migration proceeds and commits / leaves things how the SKILL says) would be stronger. Optional.

---

### Pass 1 content signals

- Cross-system refactor (every SKILL rewritten): **yes**
- Backwards compatibility (downstream users on bare-root layout): **yes**
- Filesystem migration with dirty-tree handling: **yes**
- Any finding ≥ MEDIUM: **yes (F1, F2, F3, F4)**

Content signals trip. Pass 2 warranted.

---

## Pass 2 — triggered

### Probe: Brownfield state-machine completeness

Covered under F1 — partial state missing from the model. The `setup_mode` enum should be 4 states, not 3; or the brownfield branch needs an explicit sub-state for "orbit/ present with collisions". The spec's all-or-nothing prompt model assumes clean separation between pre-migration and post-migration worlds; reality has intermediate states.

### Probe: Plugin distribution semantics

The plugin installs via Claude Code marketplace. A downstream user:

1. Runs `/plugin update` — receives new skill, new scripts, new session-context.sh gate.
2. Between step 1 and their next `/orb:setup` run, **session-context.sh now gates on `-d "orbit"`** — their repo has no `orbit/`, so the hook goes silent. They lose outstanding memo surfacing, rally/drive detection, and spec status lines until they migrate.

This is a real UX regression window. Duration is "until the user runs /orb:setup next", which could be one session or weeks. Possible mitigations:

- Keep the hook gate permissive for one release: `-d "orbit" || -d "specs" || -d "cards"`, with an added warning "detected legacy layout; run /orb:setup to migrate".
- Or fire a one-time message on SessionStart when legacy layout is detected: "orbit: legacy layout detected. Run /orb:setup to migrate."

**Ask:** Add an AC covering the legacy-layout nudge on SessionStart, or explicitly accept the silent-hook window as a trade-off and document it. The current spec is silent; downstream users will be confused.

### Probe: Consumers of the old gate

Grep confirms `-d "specs"` gate appears only in `session-context.sh`. No other script or skill uses the pattern. Good.

### Probe: Hook memo scan post-migration

The hook currently scans `cards/memos/*.md`, then reads `cards/*.yaml` looking for `cards/memos/` references in lists to determine outstanding. After migration all four path literals inside the script rewrite to `.orbit/cards/memos/` / `.orbit/cards/*.yaml`. ac-08 covers this. The cross-reference search `grep 'cards/memos/' cards/*.yaml` becomes `grep '.orbit/cards/memos/' .orbit/cards/*.yaml` — card bodies must use the new prefix when referencing memos. ac-13's rewrite covers that. Internally consistent.

### Probe: Evidence rewrite — sharpest edge

Noted under F7. Author acknowledged deliberate. No escalation.

### Probe: ac-13 regex correctness

Covered under F3. Medium concern on both false-positive and scope-interpretation fronts.

### Probe: Missing ACs implied by goal

The goal asserts "Every skill, script, hook, and shipped artefact in this repo references orbit/<subdir>/ paths". ACs cover: skills (ac-10), scripts (ac-08, ac-09), hooks (ac-07, ac-08), shipped artefacts (ac-13, ac-15, ac-16). But nothing explicitly covers:

- **Decision records under decisions/** — presumably they live as-is (content describes events at time of writing) but must reference orbit/-prefixed paths in any future writes. ac-13's sweep catches them, but no AC explicitly names the decision register as rewrite-scope.
- **cards/*.yaml internal references** — cards like 0001-memos.yaml reference `cards/memos/` in scenarios (grep confirmed: `cards/0001-memos.yaml:9`). These need rewriting. Covered by ac-13 implicitly; worth naming explicitly given cards are living documents per CLAUDE.md.
- **The card that produced this spec** (`cards/0008-*.yaml`) — references `.orbit/cards/memos/`, `.orbit/specs/`, `.orbit/choices/` already (prose in the then: clause, line 28). Already correct.
- **Specs' own `specs:` arrays in cards.** After migration `specs: [specs/2026-04-20-...]` becomes `specs: [.orbit/specs/2026-04-20-...]`. This is a specs-array-integrity concern — the ac-13 sweep catches it, but the authored spec paths in all cards will change. Worth a short note that the specs-array format accepts the new prefix without schema change.

**Ask:** Optional but high-value: extend ac-13 or add a specific AC that names decisions/, cards/*.yaml scenario text, and the specs: arrays in cards as part of rewrite scope.

---

## Pass 3 — not triggered

No structural concerns surfaced in Pass 2. Verdict-level escalation not needed. Findings are localised to AC wording and two new-AC recommendations.

---

## Recommendations summary

Before implementation starts:

1. **F1** — Model the orbit/+bare mixed state in setup_mode or add an AC for it.
2. **F2** — Explicitly handle or document untracked-files-in-target-dirs during dirty-tree migration.
3. **F3** — Tighten the ac-13 grep invocation; use `git ls-files`; allow-list CHANGELOG.md entry that names legacy paths; widen the negative character class.
4. **F4** — Add an explicit brownfield-then-idempotent run AC.
5. **F5** — Clarify ac-19's enforcement vs deferred commitment status.
6. **Pass-2** — Add a downstream-upgrade-window AC: either legacy-gate-preserved-one-release, or legacy-layout-detection nudge on SessionStart.

F6, F7, F8 are LOW and advisory — optional fixes.

---

**Verdict:** REQUEST_CHANGES
