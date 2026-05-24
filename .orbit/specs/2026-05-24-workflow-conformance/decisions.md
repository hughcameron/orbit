# Decisions — 2026-05-24-workflow-conformance

Context: `orbit audit conformance` (in `orbit-state/crates/core/src/verbs.rs`, `audit_conformance_at`) currently treats `.orbit/METHOD.md` missing as a MEDIUM `missing` finding under subsystem `setup`. In a brownfield repo whose substrate lives at `orbit/` (no dot), this is misleading — the file isn't missing-because-needed, it's missing-because-the-whole-folder-is-elsewhere. Following the remediation verb (`orbit setup`) would leave two parallel folders. This drive adds a finding family that fires HIGH on the layout mismatch and suppresses the canonical-files-missing noise that flows from it.

Locked from the rally proposal (do not re-derive):
- new finding fires HIGH and suppresses canonical-files-missing findings until layout is fixed
- remediation verb is `orbit setup`

The six decisions below cover the remaining shape choices.

---

## 1. Canonical predicate for "undotted substrate is present"

**Context.** The detector needs a positive signal that substrate exists at `orbit/` rather than `.orbit/`. The arcform case had `orbit/cards/`, `orbit/choices/`, `orbit/specs/`, and `orbit/decisions/` populated. The detector must not false-positive on an arbitrary `orbit/` directory that happens to be a Rust workspace member or a build artefact.

**Options.**
- **A. Single canonical directory:** `<repo>/orbit/cards/` exists AND `<repo>/.orbit/cards/` does not.
- **B. Any-of substrate dirs:** any of `<repo>/orbit/{cards,choices,specs,memos}/` exists AND `<repo>/.orbit/cards/` does not.
- **C. Stricter — at least two of the substrate dirs:** `>=2` of `{cards,choices,specs,memos}` exist under `orbit/` AND `<repo>/.orbit/cards/` does not.

**Trade-offs.**
- A is the simplest predicate and matches the most common brownfield shape (cards always exist if anything does). It misses an exotic case where someone has `orbit/specs/` but no `orbit/cards/` yet — improbable, but possible mid-migration.
- B catches every shape but widens the false-positive surface. If a future card adds a `orbit/` symlink for some unrelated reason, or the repo has a `orbit/` workspace crate (this repo doesn't, but `orbit-state/` is close to that pattern), the detector fires noise. The `.orbit/cards/` negative guard mitigates that risk substantially — once `.orbit/` is canonical, the noise window closes.
- C is defensive but encodes a heuristic that's hard to explain in prose. The seed memo and arcform precedent both had four+ subdirs populated, so the two-of-four floor is satisfied in the observed case but not load-bearing.

**Recommendation.** Option B. Match any of `orbit/{cards,choices,specs,memos}/`. Rationale: the negative guard (`.orbit/cards/` absent) is the real filter — once canonical substrate exists, the finding is suppressed regardless. Option B's wider net handles partial-migration mid-states (someone moved cards but not specs) without a separate finding family. The implementation reads each directory's `.exists()` and short-circuits on the first hit; cost is four `stat` calls.

The detector function lives next to `canonical_file_findings` (verbs.rs ~line 4091) as a sibling `undotted_substrate_finding(layout) -> Option<ConformanceFinding>` returning `Option` (single finding or none), called from `audit_conformance_at` before the per-file canonical check.

---

## 2. Suppression mechanism — emission-time skip vs post-processing filter

**Context.** The locked decision says the new finding suppresses canonical-files-missing findings. The existing `pin_dominates` precedent in `audit_conformance_at` (verbs.rs ~3920) gates `canonical_file_findings` behind `if !pin_dominates` — a branch that prevents the emission rather than a filter after the fact.

**Options.**
- **A. Emission-time skip (mirror `pin_dominates`):** add a `layout_dominates: bool` boolean alongside `pin_dominates`; wrap `canonical_file_findings` in `if !pin_dominates && !layout_dominates`.
- **B. Post-processing filter:** emit every finding, then drop canonical-files-missing entries when the layout finding is present.
- **C. Separate suppression-aware wrapper:** introduce a `compose_findings(layout, today) -> Vec<Finding>` helper that owns the suppression matrix.

**Trade-offs.**
- A is minimal and mirrors the existing precedent. The branch is one line; the test surface stays close to what's already proven for `pin_behind` / `pin_ahead`. The reader following the existing code shape understands it immediately.
- B is more general — if the suppression matrix grows beyond two suppressors, post-processing scales. But every finding-builder already runs unconditionally on the codebase; running the byte-compare against `.orbit/METHOD.md` when `.orbit/` doesn't exist will hit the `missing` branch for every canonical file, only to be filtered out. That's wasted work, but more importantly it leaks state from a downstream symptom.
- C is over-engineered for two suppressors. Defer until a third arrives.

**Recommendation.** Option A. Add `let layout_dominates = undotted_substrate_finding(layout).is_some()` (or derive from a stored `Option<ConformanceFinding>`) before the existing `pin_dominates` check, and extend the `if !pin_dominates` guard to `if !pin_dominates && !layout_dominates`. Same precedent, same test pattern, smallest diff.

---

## 3. Finding-family identity — extend `setup` subsystem or add a new one

**Context.** `ConformanceFinding.subsystem` currently takes the values `cards`, `memos`, `setup`, `routines`. The pin-state findings live under `setup` (subject `.orbit/config.yaml`, state `pin_behind` / `pin_ahead`). Canonical-files-missing findings also live under `setup` (subject `.orbit/METHOD.md` etc., state `missing` / `byte_drift`). The new finding is layout-shaped.

**Options.**
- **A. Subsystem `setup`, new state slug:** subject = `orbit/` (the offending folder), state = `non_canonical_layout` (per decision 5), under existing `setup` subsystem.
- **B. New subsystem `layout`:** subject = `orbit/`, state = `non_canonical_layout`, subsystem = `layout`.
- **C. Subsystem `setup`, subject = repo root, state encodes the gap:** state = `non_canonical_layout`, subject = `.` (repo root) — finding is "about the repo", not "about a directory".

**Trade-offs.**
- A keeps the subsystem axis stable. Every finding whose remediation is `orbit setup` already lives under `setup`; the new finding shares that property exactly. The state slug carries the discriminator, which is what `state` is for.
- B introduces a new subsystem for one finding. Adds a value to the documented set (`ConformanceFinding` docstring at verbs.rs:1227 lists subsystems). Worth doing only if the subsystem axis is the natural query — e.g. if `/orb:prioritise` filters findings by subsystem. It doesn't today.
- C makes the subject `.` which loses the diagnostic value of pointing at the offending folder. `/orb:prioritise` surfaces the subject; "orbit/" is more actionable prose than ".".

**Recommendation.** Option A. Subsystem `setup`, state `non_canonical_layout`, subject = `orbit/` (the literal directory name, repo-relative). This is consistent with how the pin findings sit under `setup` with `.orbit/config.yaml` as subject, and the byte-drift findings sit under `setup` with the file path as subject. The new finding fits the same shape.

---

## 4. Severity floor — always HIGH, or conditional

**Context.** The locked decision says HIGH-severity. The question is whether HIGH is unconditional, or whether some shape of "no actual data under `orbit/`" downgrades it. The seed memo's case had 22 cards + 16 decisions in the wrong location — losing visibility on that volume of work is genuinely high-impact.

**Options.**
- **A. Always HIGH:** any time the predicate fires, severity is `high`.
- **B. HIGH only when substrate is non-empty:** if `orbit/cards/` exists but contains no files, the finding emits at `medium`.
- **C. HIGH always, with `evidence.substrate_volume` populated:** severity stays HIGH unconditionally, but the evidence map carries `{cards_count, choices_count, specs_count, memos_count}` so downstream consumers can apply their own threshold.

**Trade-offs.**
- A is unambiguous. The locked decision said HIGH; this honours it without conditional shape.
- B introduces a count-the-files step on emission and a severity branch that's hard to defend in prose. An empty `orbit/cards/` directory is still a layout mismatch and still means `orbit setup` will fire the migration prompt — the agent's next action is identical regardless of substrate volume.
- C gives `/orb:prioritise` and other downstream consumers structural visibility into the scale of the migration. The cost is four `read_dir` calls at audit time, all of which are cheap on substrate-sized directories.

**Recommendation.** Option C. HIGH unconditionally, with `evidence` carrying the four counts. The severity stays simple (matches the locked decision); the evidence enriches the finding for any consumer that wants to surface "this is a 22-card migration, not an empty scaffold". Implementation reads each `orbit/{cards,choices,specs,memos}/` directory with the existing `list_yaml_files` / `list_md_files` helpers (or inline `read_dir` if the helpers don't apply against a non-`.orbit/` path). Total cost: four `read_dir` calls, gated on the predicate already having fired.

---

## 5. State slug name — `non_canonical_layout` vs `undotted_substrate` vs `legacy_orbit_folder`

**Context.** State slugs appear in CLI/JSON output, agent prose, and `/orb:prioritise` summaries. The slug should read naturally in a remediation summary, and the agent should be able to grep for it across code, prose, and history.

**Options.**
- **A. `non_canonical_layout`:** generalist — describes the gap (layout doesn't match canonical) without naming the specific shape of the mismatch.
- **B. `undotted_substrate`:** specific — names exactly what's wrong (folder is missing its dot).
- **C. `legacy_orbit_folder`:** historical framing — names the cause (old folder shape) rather than the gap.

**Trade-offs.**
- A is open-ended. If a future finding under "layout doesn't match canonical" needs to fire (e.g. `orbit/` exists alongside `.orbit/` — though spec'd as "mixed" in setup §4), the slug doesn't generalise; we'd add a sibling state for that case. The generalist slug invites confusion about which shape is firing.
- B is precise and matches the seed memo's diagnostic language ("undotted substrate"). The slug is literal: anyone grepping for "undotted" in code, prose, or git log finds the full story. It carries the diagnostic over from the memo to the code without translation.
- C is backward-looking. "Legacy" implies historical, which is true today but ages poorly — the slug will still fire in 2027 when nobody remembers the dotted/undotted transition was a thing.

**Recommendation.** Option B — `undotted_substrate`. Precise, matches the memo's framing, greppable across code and prose. Subject = the offending folder name (`orbit/`); state = `undotted_substrate`; evidence = the four counts (decision 4). Reads in CLI output as `[high] setup/undotted_substrate orbit/` — the diagnostic is immediate.

---

## 6. Suppression scope — only canonical-files-missing, or also pin-behind / memo-staleness / card-state

**Context.** The locked decision suppresses canonical-files-missing. The question is whether other finding families also suppress during this state. The arcform case surfaced multiple downstream symptoms: METHOD.md/STYLE.md missing, but also potentially card-state findings (no cards visible because they're at `orbit/cards/` not `.orbit/cards/`) and memo-staleness (same reason).

**Options.**
- **A. Suppress only canonical-files-missing (the locked-decision minimum):** the new finding emits; canonical-files-missing skip; everything else emits as today.
- **B. Suppress all `.orbit/`-dependent findings:** during `undotted_substrate`, skip canonical-files-missing AND card-state AND memo-staleness AND pin-state (all of which read from `.orbit/`).
- **C. Suppress canonical-files-missing + the pin-state finding only:** the pin-state read is also `.orbit/`-dependent (reads `.orbit/config.yaml`), but card-state and memo-staleness fail naturally to empty (no files under `.orbit/cards/` or `.orbit/memos/`) so they don't need explicit suppression.

**Trade-offs.**
- A honours the locked decision literally but doesn't address that the rest of the audit is misleading too. If `.orbit/` doesn't exist, the pin-state finding reads "unpinned" (no config to read), card-state reads "no cards" (nothing under `.orbit/cards/`), memo-staleness reads "no memos". None of those are findings, but they aren't useful either — and the agent reading the envelope might draw wrong conclusions.
- B is the most consistent: when the layout is wrong, the audit's verdict is "fix the layout first, then re-run me." Every other finding family is structurally meaningless until `.orbit/` is canonical.
- C splits the difference: suppress what can produce a misleading finding (pin-state can emit `unpinned` or could in theory misfire if `.orbit/config.yaml` is read from a half-migrated tree), keep what naturally empties out.

**Recommendation.** Option B. When `undotted_substrate` fires, suppress canonical-files-missing AND card-state AND memo-staleness AND pin-state. Implementation: extend the existing `if !pin_dominates && !layout_dominates` pattern to wrap every finding-builder except routine-findings (which read from `.claude/skills/`, not `.orbit/`, and remain meaningful regardless of substrate location). Aggregated `audit_drift` and `audit_topology` continue to run — they fail-empty naturally when `.orbit/` is absent and the agent already knows how to read their `configured: false` and empty-vec states.

This delivers on `/orb:prioritise`'s single-verb-remediation contract: when the layout is wrong, the agent sees exactly one finding (`undotted_substrate`, remediation `orbit setup`), runs it, then re-invokes the audit and sees the real state of the (now-canonical) substrate.

---

## Summary — recommended shape

- Predicate: any of `orbit/{cards,choices,specs,memos}/` exists AND `.orbit/cards/` does not (decision 1)
- Emission: single-finding helper called before the existing canonical-files-missing branch; suppression via the existing `pin_dominates` pattern extended to `layout_dominates` (decision 2)
- Identity: subsystem `setup`, state `undotted_substrate`, subject = `orbit/` (decisions 3, 5)
- Severity: HIGH unconditionally, with `evidence` carrying counts under each of the four `orbit/<subdir>/` directories (decision 4)
- Remediation: `orbit setup` (locked)
- Suppression scope: canonical-files-missing, card-state, memo-staleness, pin-state (decision 6); routine-findings + aggregated drift/topology continue to run

Implementation surface: one new function `undotted_substrate_finding(layout) -> Option<ConformanceFinding>` next to `canonical_file_findings` in `orbit-state/crates/core/src/verbs.rs` (~line 4091); one new `layout_dominates` branch in `audit_conformance_at` (~line 3920); one CLI parity test in `crates/cli/tests/parity.rs`; one MCP parity test in `crates/mcp/tests/parity.rs`; the four `*_findings` builders gated on `!layout_dominates` (in addition to their existing `!pin_dominates` guard where applicable).
