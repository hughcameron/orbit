# Design: Bead-native cold-fork reviews

**Date:** 2026-05-01
**Interviewer:** carson (full autonomy — agent self-answered grounded in card)
**Card:** orbit/cards/0016-bead-native-cold-fork-reviews.yaml

---

## Context

Card: *Bead-native cold-fork reviews* — 9 scenarios, goal: "review-spec and review-pr operate on the bead acceptance field with rule-coverage parity to the prior spec.yaml-based reviewers — zero regression in the cold-fork review gate's depth"

**Prior specs:** 0 (this is the first iteration).

**Position in 0.4.0 rollout:** orbit-6da.1 rewrote `/orb:implement` to read from the bead acceptance field. orbit-6da.2 collapsed `/orb:drive` to four bead-native stages with a snapshot-bridge between drive and the unchanged review skills. orbit-6da.3 collapsed `/orb:rally` onto the bead graph. **This card removes the snapshot bridge** so review-spec/review-pr read the bead directly, restoring full review depth before 0.4.0 ships to production projects (Hydrofoil, FineType, Brightfield, Arcform).

**Gap:** the snapshot bridge is a static markdown render of `bd show <bead-id> --json`. It works as a stop-gap — reviewers see the bead's description and acceptance field — but it loses the structural distinction between gate ACs and non-gate ACs (the `[gate]` marker survives as text but the parser running in the reviewer is the markdown reader, not `parse-acceptance.sh`). That makes review-spec's Pass-1 gate-AC verification check silently no-op: the rules are calibrated for `spec.yaml`'s structured `ac_type: gate` + `verification: <text>` shape, and the markdown snapshot doesn't expose that structure to a reviewer that knows how to consume it.

## Q&A

### Q1: Where does the bead read happen — in drive's Agent brief, or in the review skill itself?

**Q:** Today drive renders a markdown snapshot and passes its absolute path in the brief. Two options for the substrate change:
- **(a) Drive pre-renders.** Drive computes a structured payload (description + AC list with gate flags) and embeds it in the brief. Skill stays substrate-agnostic.
- **(b) Skill reads the bead directly.** Drive's brief carries only `<bead-id>` and an instruction to read the bead via `bd show <bead-id> --json` and `parse-acceptance.sh`. Skill becomes substrate-aware.

**A:** **(b) Skill reads the bead directly.** Three reasons:
1. Scenario 3 names `parse-acceptance.sh` as the shared parser — using it from the review skill is the only way to satisfy "AC interpretation cannot drift between implement and review."
2. Drive pre-rendering is the snapshot bridge by another name. The whole point of the card is to remove the substrate translation step.
3. Future bead schema changes (e.g. promoting `[gate]` semantics, adding metadata fields) need the review skill to re-fetch fresh data, not consume a frozen drive-side render.

The Agent brief carries: `<bead-id>`, the absolute output path for the verdict file, and the canonical verdict-line contract. Nothing else. The brief is shorter than the snapshot-bridge version, not longer.

### Q2: Where do verdict files live under the bead substrate?

**Q:** The 0.3.3 inline-mode path is `<spec_dir>/review-{spec,pr}-<date>.md`. The bead-native drive in the source repo (orbit-6da.2) writes to `orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md`. Which path contract applies to this card?

**A:** **`orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md`** — the bead-native path, named verbatim by scenario 6. Bead-native drives have no `spec_dir` concept; reviews are addressed by bead-id. Drive computes the cycle-specific path (`-v2.md`, `-v3.md` suffixes for REQUEST_CHANGES re-fork cycles) and passes the absolute path in the brief. The skill writes to the brief's path verbatim — same precedence rule as today.

This is consistent with orbit-6da.2's drive Stage 1 (`§1.2 Compute the cycle-specific verdict path`). No new path contract is invented by this card; the card just removes the snapshot path that would have been a sibling file under the same `orbit/reviews/<bead-id>/` directory.

### Q3: How does the gate-AC verification rule (review-spec Pass 1, step 5) map to the bead acceptance convention?

**Q:** The Pass-1 deterministic check today reads `ac_type: gate` and `verification: <text>` from spec.yaml. The bead acceptance convention (`orbit/conventions/acceptance-field.md`) has no separate `verification` field — each AC is a single line `- [ ] ac-NN [gate]: <description>`. Three options:
- **(a) Extend the convention.** Add a multi-line per-AC format with optional `verification:` continuation, update parse-acceptance.sh, document migration.
- **(b) Treat the AC description as the verification text.** The promote.sh template (`<scenario name> — <then-clause>`) produces a description that IS the verification statement — apply the deterministic rules (non-empty, not-placeholder, ≥20 chars) directly to the description text for ACs where `is_gate=1`.
- **(c) Drop the gate-AC check.** Accept the regression.

**A:** **(b) Treat the AC description as the verification text.** Reasoning:
- The bead acceptance convention is intentionally one-line-per-AC for parser simplicity. Extending it (option a) is a substrate change that exceeds this card's scope and would force re-promotion of every in-flight bead.
- The promote.sh template already concatenates the scenario name with the `then`-clause (the verification statement) as the AC description. The semantic content of "verification text" is preserved in the description; only the structural separation was lost in the substrate change.
- Option (c) directly contradicts scenario 8 ("Coverage parity with 0.3.3 spec.yaml flow … every rule that fired under 0.3.3's spec.yaml flow … fires against the bead substrate with equivalent or better coverage").

**Concrete mapping for the rule:**

| spec.yaml field        | Bead substrate equivalent                                                |
|------------------------|--------------------------------------------------------------------------|
| `ac.ac_type == "gate"` | `parse-acceptance.sh acs <bead-id>` row where `is_gate == 1`             |
| `ac.verification`      | The description text (column 3 of `parse-acceptance.sh acs` output)      |

The three deterministic rules (non-empty / not-placeholder / ≥20 chars) fire against the description text for every `is_gate=1` row. The "silently no-op" failure mode named in scenario 2 is fixed: parse-acceptance.sh exposes `is_gate` as a column, so the reviewer can find the gate ACs and run the checks against their text.

### Q4: What about review-pr's progress.md / `ac_type` / `test_prefix` cross-reference?

**Q:** review-pr's Phase 2 cross-references AC IDs against tests using `ac_type: code` (only code ACs require tests) and `metadata.test_prefix` (prefix for test discovery). Both are spec.yaml fields. Three options:
- **(a) Mirror Q3's approach.** Map `ac_type: code` to "AC where `is_gate=0` and the description doesn't match a doc/config keyword regex"; drop test_prefix and require bare `ac<NN>` test names.
- **(b) Use bead metadata.** Stuff `ac_type` and `test_prefix` into bead `--set-metadata` fields and have review-pr read them.
- **(c) Treat all ACs as code-typed.** Search for `ac<NN>` test names against every AC; report missing tests as findings the reviewer must contextualise.

**A:** **(c) Treat all ACs as code-typed.** Reasoning:
- The bead acceptance convention has no `ac_type`. Inventing one (option b) extends the convention scope past what this card is doing.
- The current `ac_type: doc | gate | config | code` field in spec.yaml exists to suppress false-positive test-coverage findings, but a reviewer with the bead's description in hand can read each AC and judge whether it's a code AC. The deterministic check becomes "search for tests; report what you find," not "search for tests; fail if any required AC has no test."
- `test_prefix` was never universally adopted — most cards omit it and rely on bare `ac<NN>` names. Dropping it removes a piece of metadata the reviewer rarely had anyway.
- progress.md is already removed from `/orb:implement` (orbit-6da.1) — the bead acceptance field's check status IS the progress tracker. review-pr should run `parse-acceptance.sh acs <bead-id>` to get current AC status, then cross-reference against `git diff main...HEAD`.

The verdict logic stays the same: AC coverage findings are reported in the standard table format, and the reviewer's honest assessment paragraph contextualises which uncovered ACs are doc/gate/config-type (visible from each AC's description) and which are genuine test gaps. This is a small fidelity loss against the spec.yaml flow but is in the "equivalent or better coverage" spirit of scenario 8 — a human-judged exemption is more accurate than a metadata-tagged exemption that depends on the spec author remembering to set the field.

### Q5: Risk appetite — backward compatibility with snapshot-bridge drives in flight?

**Q:** The orbit-6da.2 snapshot bridge has been live for ~10 days. Three options:
- **(a) Hard cutover.** This card's implementation removes the bridge entirely. In-flight drives that were started under orbit-6da.2 finish under the bridge code; the next drive uses the new direct-read path.
- **(b) Dual-mode with feature flag.** Skills accept either a snapshot path or a bead-id; drive picks based on a flag. Removed in 0.5.0.
- **(c) Auto-detect.** Skill checks if the brief argument is a file path or a bead-id and routes accordingly.

**A:** **(a) Hard cutover.** Reasoning:
- Decision 0011 D2 commits to the cold-fork reading from beads, not from a translated artefact. Dual-mode preserves the bridge as a maintained code path indefinitely.
- The orbit-6da.2 bridge was explicitly a stop-gap shipped to keep the cold-fork working through the substrate migration. The card scenarios make removal of the bridge an explicit acceptance criterion (scenario 7).
- In-flight drives during the upgrade window finish under the version of the skill they started with — same migration discipline as the forked-reviews migration shipped earlier (drives initialised before forked reviews refuse to resume under the new code).
- Hugh's projects (Hydrofoil, FineType, Brightfield, Arcform) have not yet migrated to bead-native orbit, so there's no in-flight bridge-mode drive to protect at the production sites — only orbit's own self-development drives.

**Operational note:** the next drive launched against any card after this ships uses the direct-read path. orbit's own in-flight drives (this very drive included, if it were started under the bridge — but it wasn't, this card hasn't shipped yet) finish under their start-time skill version.

---

## Summary

### Goal
review-spec and review-pr read the bead acceptance field directly via `bd show <bead-id> --json` and `parse-acceptance.sh`, with the same depth of structural review they had against spec.yaml. Drive's snapshot-bridge files (`bead-snapshot-<date>.md`, `bead-snapshot-<date>-pr.md`) are removed from the pipeline. Verdict files stay at `orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md`.

### Constraints
- **Cold-fork architecture preserved.** Reviewers run as forked general-purpose Agents with no shared conversation history (decision 0011 D2). The brief carries only the bead-id, output path, and verdict-line contract.
- **Verdict contract unchanged.** Canonical line `**Verdict:** APPROVE | REQUEST_CHANGES | BLOCK` at the same paths drive expects. Drive's strict-regex parser is not modified.
- **Shared parser.** Both review-spec and review-pr use `plugins/orb/scripts/parse-acceptance.sh` for AC enumeration. AC interpretation cannot drift from `/orb:implement`.
- **No backward compatibility for snapshot-mode drives.** Hard cutover; in-flight drives finish under their start-time skill version. No `review_mode` flag, no dual code paths.
- **No convention extension.** The bead acceptance convention (`orbit/conventions/acceptance-field.md`) is unchanged — one line per AC, optional `[gate]` marker, no separate verification field. The gate-AC verification check fires against the AC description text for `is_gate=1` rows.

### Success Criteria
1. `git grep -F "bead-snapshot-" plugins/orb/skills/drive/SKILL.md` returns zero matches (Stage 1 §1.1 and Stage 3 §3.1 are deleted).
2. The Agent brief in drive's Stage 1 (review-spec) and Stage 3 (review-pr) contains only the bead-id, the absolute verdict output path, and the verdict-line contract — no snapshot path.
3. `plugins/orb/skills/review-spec/SKILL.md` Step 1 instructs reading from `bd show <bead-id> --json` and parsing ACs via `parse-acceptance.sh acs <bead-id>`. The Pass-1 gate-AC verification check fires against the description text of every `is_gate=1` row.
4. `plugins/orb/skills/review-pr/SKILL.md` Phase 1 instructs reading the bead via `bd show <bead-id> --json` and parsing ACs via `parse-acceptance.sh acs <bead-id>`. AC coverage check uses bare `ac<NN>` test names.
5. The four-option / two-option / three-option AskUserQuestion gates in supervised and guided modes work unchanged — they consume the verdict file's findings, which still come out of the review skills.
6. A test bead promoted from a card with ≥1 gate scenario triggers the deterministic gate-AC verification rules under the new flow.

### Decisions Surfaced
- **Skill is substrate-aware, brief is minimal** (Q1, option b): the review skill calls `bd show` and `parse-acceptance.sh` itself; drive's brief carries only the bead-id. → not yet a MADR — this is a refinement of decision 0011 D2.
- **AC description = verification text** (Q3, option b): the bead substrate has no separate verification field; the deterministic rules apply to the description text for `is_gate=1` rows. → candidate MADR (0012?) — record as part of this drive's commit.
- **No `ac_type` in beads; review-pr judges code-vs-doc per-AC** (Q4, option c): test coverage check searches for bare `ac<NN>` test names and lets the reviewer's honest-assessment paragraph contextualise gaps. → candidate MADR — record alongside the implementation.
- **Hard cutover** (Q5, option a): no dual-mode; in-flight bridge drives finish under their start-time skill version. → already implied by decision 0011 D2.

### Implementation Notes
- The bridge code in `plugins/orb/skills/drive/SKILL.md` is concentrated in §1.1 (write snapshot for review-spec) and §3.1 (write fresh snapshot for review-pr). §1.4 and §3.3 reference the snapshot path in their Agent brief. All four sections need editing.
- The Stage 1 brief example block (§1.4) currently reads `Run /orb:review-spec on the spec snapshot at <absolute snapshot path>. The snapshot is the authoritative spec for this review.` — replace with `Run /orb:review-spec on bead <bead-id>. Read the bead via bd show <bead-id> --json and parse ACs via parse-acceptance.sh acs <bead-id>.`
- The Stage 3 brief example block (§3.3) currently includes `Spec snapshot for AC cross-reference is at <absolute snapshot path>.` — replace with `Bead acceptance field is at bead-id <bead-id>; read via bd show / parse-acceptance.sh.`
- review-spec/SKILL.md Step 1 ("Gather the Spec") today reads "If a spec file path is provided via $ARGUMENTS: read it" — needs rewriting to handle a bead-id argument and call bd/parse-acceptance.
- review-pr/SKILL.md Step 2 ("Read the Diff") references reading the spec — needs rewriting similarly. Step 3's AC coverage check already references parse-acceptance — keep that, just change the AC-to-test discovery to bare `ac<NN>` names.
- The bead `acceptance_criteria` field is exposed as a single string in `bd show --json`. parse-acceptance.sh consumes that via the `get_acceptance` helper. The skill calls `parse-acceptance.sh acs <bead-id>` and gets one tab-separated row per AC.
- The Completion section (§Completion in source SKILL.md) currently says `commit message: feat: <bead title>` and lists `bead snapshots, and the review files` — drop "bead snapshots" from the commit-1 description; the snapshots no longer exist.
- Decision-record draft: 0012 — bead acceptance field as the cold-fork review substrate. Rationale: closes the bridge that orbit-6da.2 left as a stop-gap; preserves cold-fork separation; trades one fidelity loss (per-AC `ac_type` / `verification` fields) for full parser parity with implement.

### Open Questions
None — the card scenarios resolved all five intent-level questions. Implementation is well-defined.
