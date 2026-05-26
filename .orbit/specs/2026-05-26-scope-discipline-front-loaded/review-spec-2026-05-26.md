# Review — spec 2026-05-26-scope-discipline-front-loaded

**Reviewer:** /orb:review-spec (Claude Opus 4.7, cold-fork)
**Date:** 2026-05-26
**Spec:** `.orbit/specs/2026-05-26-scope-discipline-front-loaded/spec.yaml`
**Card:** 0045-scope-discipline
**Tabletop:** `.orbit/specs/2026-05-26-scope-discipline-front-loaded/tabletop.md`

**Verdict:** REQUEST_CHANGES

The spec is well-shaped: four mechanisms cohere into one shipment, the dog-fooded classification (`verifies: capability` / `verifies: stand-in`) is applied to every AC's verification clause, halt/escalation/kill conditions live in the sidecar (matching the canonical seven-field Spec schema per memory `spec-skill-md-fields-vs-schema`), and the band split between blocking (`code` / `doc`) and deferring (`observation`) is clean. The block is two narrow but load-bearing gaps in AC-04: the spec invokes a "deferral-pattern spec note" convention that does not yet exist in the substrate, and the firing rule names "scenarios" as the unit of deferral when nothing on the card side guarantees scenarios are individually addressable across spec close. Both are fixable with small clarifications; neither requires re-tabletop.

---

## What's working

- **Mechanism count matches the tabletop.** Four mechanisms; nine ACs distributed across them with no orphaned scenarios. AC-01/02 land tabletop posture, AC-03 lands the /orb:spec halt rule, AC-04/05 land the audit finding family, AC-06/07 nail the canonical-source and test-coverage hygiene, AC-08/09 wire the halt-1 and halt-2 caps directly into ACs.
- **Halts converted to ACs.** The two halt conditions from the tabletop sidecar (rebloat cap, audit overfire cap) appear as AC-09 (line-count ≤124) and AC-08 (≤5 findings) rather than being left as sidecar prose. Halt-1 is a blocking `doc` AC; halt-2 is correctly typed `observation` because it only resolves post-implementation against the running audit.
- **Dog-fooding visible in the verification clauses.** Every AC carries a `verifies: capability` or `verifies: stand-in (real thing is X), accepted because Y` line, exactly the convention AC-02 and AC-03 install for future specs. AC-06 and AC-09's stand-in rationales are the kind the discipline is meant to surface — naming the real thing, naming the proxy, naming the acceptance reason.
- **Test prefix declared.** `scope_` named in the goal and threaded into AC-04, AC-05, AC-07 verifications. Test-prefix consistency lets `cargo test --workspace -- scope_` work as the AC-07 check.
- **Audit-window contract present.** AC-05 explicitly gates the new finding family by close-date, satisfying card 0045 scenario 6 ("no retroactive spam on historical narrow specs"). The test name (`scope_audit_window_excludes_pre_ship_specs`) describes the exact regression.
- **Memories reconciled.** Five memories listed with `adopted` dispositions and reasons. `tabletop-q3-cost-as-mis-staging-signal` is the load-bearing one — the spec's whole shape (front-loading vs mid-implement halt) is that memory applied.

## What needs fixing — blocking

### B1. AC-04: "deferral-pattern spec note" is an undefined convention

AC-04's firing rule depends on "the cumulative count of deferred scenarios across those specs (drawn from spec notes matching the deferral pattern)". The substrate has no such convention today:

- `grep -rn "deferred\|deferral"` across `orbit-state/crates/core/src/` returns only the `ac_type: observation` doc-comment in `schema.rs:371` — there is no parser, schema field, or written convention naming what a "deferral-pattern spec note" looks like.
- The Spec schema (per memory `spec-skill-md-fields-vs-schema` and the spec itself) carries `notes` as a free-form list. A finding family that walks `notes[]` and matches a "deferral pattern" needs the pattern itself defined somewhere — string prefix? structured key? a new `Spec.deferred_scenarios: Vec<String>` field?

AC-03 mentions that the closed-mode tabletop-note convention carries the classification; AC-03's third pick ("accept-with-rationale captured in a spec note") is where deferral notes would originate. But neither AC defines the on-disk shape of the deferral note that AC-04's parser must read.

**Required fix.** Either (a) add an AC defining the deferral-note shape (e.g. "spec notes prefixed `deferred-scenario:` are the canonical deferral marker, parsed as `<scenario-name> — <rationale>`") OR (b) introduce `Spec.deferred_scenarios: Vec<DeferredScenario>` as a typed field and update AC-04 to read from it. Without one of these, AC-04 is implementable but the implementing agent picks the convention unilaterally — exactly the kind of underspecified surface this spec exists to eliminate.

The shape choice belongs in this spec, not deferred. Picking it now is a one-line addition; deferring it creates the recursive irony of an under-verified scope-discipline spec.

### B2. AC-04 firing rule says "scenarios" but specs don't structurally guarantee scenario-level addressability

AC-04 fires when "the cumulative count of deferred scenarios across those specs … is ≥2". A "deferred scenario" is meaningful only if scenarios are the unit of work a spec addresses. The card→spec relationship is many-to-many on scenarios in practice: cards carry scenarios, specs reference cards, and an AC may exercise part of one scenario, all of one, or threads across several. Nothing in the current Spec schema names which card scenarios a spec addresses.

The tabletop sidecar's mechanism 4 says "fires on a card with 2+ deferred scenarios across closed specs and no open follow-up spec" — same gap. AC-03's tabletop-note classification is per-scenario, which is consistent, but the deferred-scenario list AC-04 reads still needs to be anchored to specific card scenarios by name or id.

**Required fix.** Either (a) state explicitly that deferred scenarios are identified by `<card-id>:<scenario-name>` string match against the card's `scenarios[].name` field (and AC-04's evidence carries those tuples), OR (b) introduce a structured `Spec.deferred_scenarios: Vec<{card: String, scenario: String, rationale: String}>` field. Pick one and write it into AC-04's evidence shape. The current "deferred scenario names" phrasing in AC-04 is too thin for the implementing agent to know what to emit in `evidence`.

This is the same gap as B1 but on the *what is named* side rather than the *where it lives* side; both want resolution before implement.

## What needs fixing — non-blocking observations

### O1. AC-08 observation is exposed to false negatives

AC-08 caps the audit at ≤5 findings on the orbit repo's substrate immediately after ship. The threshold ships in AC-04 (≥2 deferrals + closed-spec count ≥1 + no open follow-up), and AC-05's audit-window gate excludes pre-ship specs. With the audit-window gate, *all* pre-ship specs are excluded — so AC-08 is effectively asserting "0 ≤ 5" until a new spec closes under the discipline. That's a vacuously-passing observation.

If the intent is to validate threshold sanity on substrate that *would* have qualified under the new rule, AC-08 should explicitly say so — either (a) run the audit once with the window-gate disabled to count historical findings (Halt-2 input), or (b) note that AC-08 is a watch-for-soak AC that becomes meaningful as new specs close, and is not expected to fire on first run.

Not blocking — the observation band is the right place for empirical-threshold checks — but the AC reads as if it provides immediate evidence when it doesn't.

### O2. AC-04's `evidence` field is described as freeform but the schema is structured

Per `verbs.rs:1514-1529`, `ConformanceFinding.evidence` is `Option<serde_yaml::Value>` — arbitrary YAML, no schema. That works. But AC-04's verification expects a test asserting "evidence (deferred scenario names + cumulative count)" — the test will need to assert against a specific shape. The implementing agent will pick the shape unilaterally unless this spec names it.

Lighter than B1/B2 because the evidence shape is per-finding-family by design. But naming the shape here ("evidence carries `{deferred_scenarios: [String], cumulative_count: u32}`") removes one underspecified choice from implement.

### O3. AC-06 grep negative-scope list is incomplete

AC-06 asserts the discipline directives live only in `plugins/orb/skills/tabletop/SKILL.md` and `plugins/orb/skills/spec/SKILL.md`, excluding `.orbit/conventions/`, `.orbit/memos/`, `.orbit/METHOD.md`, `.orbit/STYLE.md`, `plugins/orb/README.md`, and the tabletop sidecar. Two relevant locations are missing from the negative-scope: the card 0045 file itself (which already names the discipline in scenarios) and any spec.yaml in `.orbit/specs/` (which by definition carries the discipline in description/verification text — this very spec does).

Suggest tightening to "no copies in side documents outside `.orbit/cards/` (the card-source surface), `.orbit/specs/` (spec text), and the named tabletop sidecar". The current AC is grep-runnable but the false-positive surface is wider than the exclusion list captures.

### O4. AC-07 baseline is implicit

AC-07 asserts "workspace test count rises by ≥2 relative to the pre-change baseline". The baseline isn't captured anywhere in the spec — it's "whatever `cargo test --workspace 2>&1 | grep 'test result'` returns at HEAD before the first commit on this branch". For an `ac-07` close to be auditable from the PR alone, capturing the baseline number in a spec note (e.g. `baseline_test_count: NNNN at SHA xxxxxx`) before the first commit lands is the load-bearing prerequisite.

Same shape as AC-09's explicit `baseline of 94 lines` callout — AC-09 names its baseline, AC-07 should too.

### O5. Memory `drive-substrate-recall-iter1` rationale is truncated

`memories_considered[0].reason: "the motivating failure — PR"` — clearly cut off mid-sentence. Cosmetic, but the memory is load-bearing (the recurring failure mode this whole card exists to fix), so finishing the sentence matters for future readers reconciling against the substrate.

## What I'd skip

- **No re-tabletop required.** The four-mechanism shape is sound; the values, trade-offs, halts, escalations, and kill conditions in the sidecar are all named with revert paths. The issues above are *write-up* gaps in the spec text, not gaps in the underlying design thinking. B1 and B2 are about pinning a shape the tabletop didn't need to settle but the spec does.
- **No carve renegotiation.** The single-spec carve is defended in the sidecar ("Two-spec carve … was defensible … but single-spec ships the full discipline in one PR with no half-wired intermediate state"). The four mechanisms genuinely ship together — the audit backstop only works once the upstream discipline produces deferral-tagged spec notes for it to read.

## Next step

Update AC-04's verification clause (and optionally a new dedicated AC) to:
1. Define the deferral-note shape — either a `notes` prefix convention OR a typed `Spec.deferred_scenarios` field. Pick one.
2. Define how a "deferred scenario" is identified — `<card-id>:<scenario-name>` tuple against the card's `scenarios[].name`.
3. Name the `evidence` payload shape the new test will assert against.

While there, finish the truncated `drive-substrate-recall-iter1` memory rationale (O5), capture the AC-07 baseline test count in a spec note before first commit (O4), and consider tightening AC-06's negative-scope list (O3) and AC-08's vacuous-pass framing (O1).

After those edits, re-run `/orb:review-spec` for a quick second pass, then proceed to `/orb:implement`.
