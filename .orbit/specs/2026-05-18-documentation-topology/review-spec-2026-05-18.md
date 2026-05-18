# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-18-documentation-topology
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | content signals (cross-system: schema + 3 verb envelopes + 4 skill surfaces + parity tests); MEDIUM finding in pass 1 | 4 |
| 3 — Adversarial | structural concerns (sequencing dependency on uncreated `.orbit/config.yaml` substrate, cascade risk if `Config` artefact lands wrong) | 2 |

## Findings

### [MEDIUM] AC-02 hides at least three sub-changes behind one acceptance criterion
**Category:** test-gap
**Pass:** 1
**Description:** AC-02 conflates (a) introducing `.orbit/config.yaml` as a wholly new orbit-state artefact with its own schema, `FIELDS` constant, and `deny_unknown_fields`, (b) adding a `docs.topology` key on that artefact, and (c) wiring `orbit verify` to validate the file when present and tolerate absence. Each is independently testable and each has its own failure modes (schema-drift tests, reconcile coverage, verify exit-code surface). Bundling them into a single AC removes the granularity that the rest of the spec relies on — AC-04 / AC-05 / AC-06 / AC-07 all consume "the docs.topology pointer", but if AC-02 ships in three commits, the downstream verbs can't sequence cleanly against a single "AC-02 closed" boundary.
**Evidence:** Spec AC-02 ("Config pointer mechanism — …schema as a new top-level artefact… with #[serde(deny_unknown_fields)] and a corresponding FIELDS constant. orbit verify validates… The pointer is read by /orb:topology, orbit audit topology, /orb:setup, and the spec.close / release warning emitters"). The schema-drift mechanism it references is real and load-bearing (`orbit-state/crates/core/src/schema.rs` lines 56–125 hold the `FIELDS` constants for Card/Spec/Choice/Memory/Scenario/Relation; `reconcile.rs` lines 496–549 dispatch on them). A `Config` artefact added incorrectly here cascades into every reconcile path that touches the layout.
**Recommendation:** Split AC-02 into three: AC-02a "Config artefact added to orbit-state schema with FIELDS constant and deny_unknown_fields, schema-drift test added alongside existing ones" (config); AC-02b "docs.topology key parses against the schema and round-trips through canonicalise" (config); AC-02c "orbit verify validates Config when present, tolerates absence, exits non-zero on invalid types — parity test on CLI + MCP" (config). Each gets its own verification line. Downstream ACs (04/05/06/07) then have a single named boundary to depend on.

### [MEDIUM] AC-04 confuses exit code with envelope shape — three drift categories all map to "exit 0"
**Category:** test-gap
**Pass:** 1
**Description:** AC-04 says "exit 0 when clean, exit 0 with non-empty array when drift exists (drift is informational, not an error)" and separately "topology capability not configured" envelope (not an error). All three paths return exit 0 with no discriminator at the exit-code layer. That's intentional per Q4 of the interview (warn, don't block) and consistent with the existing audit verbs — but it leaves operators with no way to detect drift via shell exit code alone. The release skill (AC-08) is described as "operator can proceed regardless", which is fine, but `/orb:drive` / CI integration paths have no machine-parseable signal short of parsing the JSON envelope. Either commit to envelope-only signalling (and say so explicitly so consumers don't reach for `$?`) or carve a distinct exit code for drift-present.
**Evidence:** AC-04 verification line: "Returns ok envelope with populated topology_drift on a fixture with each of the three drift categories injected. Returns 'topology capability not configured' envelope on a fixture without .orbit/config.yaml." No mention of exit-code discrimination. Existing pattern: spec close emits warnings in-envelope (AC-07 follows that pattern explicitly). Question is whether `orbit audit topology` is symmetric with `orbit audit drift` (presumably exit 0 even on drift) — not confirmed in the spec text.
**Recommendation:** Add a clarifying line to AC-04 verification: "Exit code is 0 for all three outcomes (clean / drift / not-configured); discrimination is via the envelope `topology_drift` field and the envelope error shape only." If exit-code carve-out is desired for CI consumers, raise it as a separate AC; don't bury it.

### [MEDIUM] AC-06 self-acknowledges envelope-shape uncertainty but doesn't gate spec writing on resolution
**Category:** assumption
**Pass:** 1
**Description:** AC-06 verification says "Pre-write the envelope key check against the binary per memory spec-verification-against-real-envelope-shape — confirm the actual key name before encoding it into tests." That's a self-aware caveat — the spec author knows the existing envelope key names (`item_bound`, `memories`, `next_step`, `open_specs`) are claimed without confirmation. I sampled `orbit-state/crates/core/src/session.rs` and could not confirm those exact key names exist as written in the spec. The AC is shipped as if those names are facts when they're hypotheses. The verification line places the burden on implementation-time discovery, but the AC text itself encodes assumed names — risking later spec-vs-reality drift when implementation finds the keys are spelled differently.
**Evidence:** AC-06 description: "The field is added to the existing envelope (item_bound / memories / next_step / open_specs) — additive, not breaking." `grep -n "item_bound\|open_specs" orbit-state/crates/core/src/session.rs` returned zero matches against those literal strings. Either the keys live elsewhere (verbs.rs, mcp surface) or they are spelled differently. The memory `spec-verification-against-real-envelope-shape` cited in the verification line exists at `.orbit/memories/spec-verification-against-real-envelope-shape.yaml` and explicitly addresses this failure mode.
**Recommendation:** Before promoting the spec to implementation, run `orbit session prime --json` against this repo, paste the actual envelope into AC-06 description, and lock the field name there. Otherwise the implementer either (a) discovers the spec is wrong and re-edits it (drift between as-designed and as-built), or (b) ships the named field anyway and breaks consumers who depend on the actual envelope shape.

### [MEDIUM] AC-07's "subsystems touched" heuristic is under-specified for what counts as a hit
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-07 says "subsystems touched is detected by reading the spec's interview.md / spec.yaml for path mentions and intersecting against topology entries — initially a coarse heuristic (substring match on the subsystem name in spec text)". Substring match against subsystem names is going to fire wildly: a subsystem called `audit` or `memory` will match practically every spec in this repo. The verification line says "precision improvements are out of scope" — fine, but the spec should at least name what the false-positive floor looks like and confirm operators can suppress / disable, or the warning surface becomes background noise the operator learns to ignore (the precise failure Q4 wanted to avoid by choosing "warn at gate moments" over "block at gate moments").
**Evidence:** AC-07: "substring match on the subsystem name in spec text; precision improvements are out of scope for this spec". No discussion of casing, word boundaries, allowlist/blocklist of stop-subsystem-names, or operator suppression. Q4 of interview explicitly rejected "block at gate moments" because "hard gates would create false-positive friction" — false-positive warnings degrade the same trust if the heuristic is too loose.
**Recommendation:** Tighten AC-07 with one of: (a) word-boundary match instead of substring, (b) require a minimum subsystem-name length (e.g. ≥ 5 chars) to fire, or (c) accept the noise but add an explicit AC-07b "operator can suppress topology_warnings for a given subsystem via a config entry". Choose one and name it before promoting.

### [LOW] AC-05 default doc path collides with consumer-owned content
**Category:** assumption
**Pass:** 2
**Description:** AC-05 has greenfield setup write `docs/topology.md` as a stub. Many existing repos already have a `docs/` directory with their own conventions — `/orb:setup` writing into it on greenfield is fine (greenfield means the repo is being set up), but the brownfield path (AC-05 second paragraph) prompts to add the `docs.topology` key without saying whether it also creates `docs/topology.md` or merely points at a (possibly nonexistent) path. If brownfield creates the stub, it can clobber an existing file at that path; if it doesn't, the pointer can name a file that doesn't exist and `orbit audit topology` will need to handle that case (which AC-04 does — "stale pointer" — but the warning will fire immediately on first session prime after setup, which is wrong-noise).
**Evidence:** AC-05 description: "Brownfield path detects whether .orbit/config.yaml exists and prompts to add the docs.topology key if missing (operator can decline; declining leaves the topology capability unconfigured but the rest of orbit still works)." No mention of stub-file creation behaviour on the brownfield accept path.
**Recommendation:** Add one sentence to AC-05 specifying brownfield-accept behaviour: either (a) brownfield-accept also creates the stub if no file exists at the target path, suppressing first-prime drift noise, or (b) brownfield-accept refuses to wire a pointer at a nonexistent path and instructs the operator to create it first. Either is fine; the spec just needs to pick.

### [LOW] AC-09 and AC-10 nudges have no test for "fires too often"
**Category:** test-gap
**Pass:** 2
**Description:** AC-09 (distill nudge) and AC-10 (memory nudge) both prescribe nudges, both note they should be quality-gated / conditional, but neither has a verification line for the negative case (does the nudge correctly NOT fire on irrelevant distillations / non-topology-labelled memories?). AC-10 is better — its verification says "Without the label, no nudge fires" — but AC-09 only says "Manual smoke: invoke /orb:distill on a memo that produces a subsystem-flavoured card; observe the agent reaches for /orb:topology after the write phase." There's no test that distilling a non-subsystem-flavoured memo does NOT prompt for topology. Risk: nudge fatigue degrades the same way false-positive warnings do.
**Evidence:** AC-09 verification: only positive case (subsystem-flavoured card produces the nudge). No negative case (non-subsystem memo does not).
**Recommendation:** Add to AC-09 verification: "Manual smoke (negative): invoke /orb:distill on a memo that produces a non-subsystem card (e.g. a style/prose card); observe the agent does NOT reach for /orb:topology." This is prose-only verification, low cost to add.

### [LOW] AC-08 release-skill integration has no operator-out documented
**Category:** missing-requirement
**Pass:** 2
**Description:** AC-08 says "Operator can proceed regardless (release is not gated by topology drift per Q4 of the interview)" but doesn't say what the release skill prose actually shows the operator. "Surfaces the audit output verbatim with a one-line framing" is fine, but if the topology has dozens of drift entries the output dump may be larger than the rest of the release pre-bump checklist and drown the signal it was supposed to surface. Either truncate by default with a "see full output via orbit audit topology" hint, or accept that release-time audit output can be long and document why that's the right tradeoff.
**Evidence:** AC-08 description: "surfaces the audit output verbatim with a one-line framing".
**Recommendation:** Add a sentence to AC-08: "If topology_drift exceeds N entries (suggest N=10), the release skill prose summarises rather than dumps — '12 topology drift items; run `orbit audit topology` for the full list'." This is skill-prose-only; no binary change.

### [LOW] No sequencing AC — Config artefact lands before any verb depends on it
**Category:** constraint-conflict
**Pass:** 3
**Description:** Five ACs (04, 05, 06, 07, plus indirectly 02b/c) all read `.orbit/config.yaml` via the `docs.topology` pointer. AC-02 establishes that pointer. The spec doesn't say AC-02 must close before 04–07 are tested — and because the spec is going to be implemented by a single agent in /orb:drive (presumably), this might never bite, but if the spec is parallelised or implemented out of order, the consuming ACs will fail in confusing ways. Worth one sentence in the spec goal or a top-level note.
**Evidence:** Spec ACs 04, 05, 06, 07 all reference "the docs.topology pointer" or `.orbit/config.yaml`. AC-02 establishes the schema. No ordering constraint named.
**Recommendation:** Add a "Sequencing" note at the spec level (or to AC-02's description): "AC-02 closes before AC-04, AC-05, AC-06, AC-07 can be tested — the Config artefact must exist in the schema before any verb can read its pointer."

### [LOW] Rollback / backout path not documented for the Config schema addition
**Category:** missing-requirement
**Pass:** 3
**Description:** The spec adds a new top-level artefact to the orbit-state schema. Rollback (if AC-02 ships broken) means reverting schema.rs, reconcile.rs dispatch, verify.rs, plus removing any `.orbit/config.yaml` files that consumers may have created during the broken window. Not catastrophic — orbit treats `.orbit/config.yaml` as opt-in (per AC-02) so absence is tolerated — but the spec doesn't say "if the Config artefact ships broken, here's the backout". For a schema-level change, that's worth a sentence.
**Evidence:** No rollback section in spec; AC-02 ships a permanent schema change without naming the recovery path.
**Recommendation:** Add a one-line note to AC-02 or spec-level: "Backout: revert the Config artefact addition; absence-of-config-file is already a tolerated state per AC-02, so consumer repos that created .orbit/config.yaml during the broken window remain valid (file just becomes inert)."

---

## Honest Assessment

This is a thoughtful, well-scoped spec with strong design lineage — the interview answers map cleanly to the AC list, the four pillars are visible (agent self-learning via the update-on-learning loop, state-persistence via the substrate doc), and the cluster-fit framing as "architecture-level analogue of /orb:code-investigate" is concrete enough to constrain implementation choices.

The biggest risk is AC-02 — it's a schema-level change that downstream ACs depend on, and it's bundled with three sub-changes that each deserve their own AC boundary. Splitting it is the single change that most reduces implementation risk. The second-biggest risk is AC-06's unverified envelope-key claim; that's a known pattern (per the cited memory) and should be resolved by sampling `orbit session prime --json` against this repo before promotion.

Everything else is paper-cut tightening — AC-04 exit-code semantics, AC-07 substring-match noise floor, AC-09 nudge negative test, AC-08 output-truncation rule, AC-05 brownfield stub-creation rule, plus two pass-3 cleanups (sequencing and rollback notes). None of them block implementation; all of them sharpen what "done" looks like.

The plan is implementable as written; the verdict is REQUEST_CHANGES rather than APPROVE because three of the findings (AC-02 split, AC-06 envelope verification, AC-07 noise floor) materially change the implementation contract and are cheaper to fix in the spec than in code.
