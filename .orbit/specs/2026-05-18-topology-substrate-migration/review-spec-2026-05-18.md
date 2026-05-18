# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-18-topology-substrate-migration
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (schema change, cross-system parity, backwards-compat envelope continuity) | 4 |
| 3 — Adversarial | not triggered (gaps are concrete and patchable in spec text, not cascading or rollback-shaped) | — |

## Findings

### [MEDIUM] ac-04 misses two known `docs.topology` call sites in plugin skills
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-04 enumerates the wiring rewire surface — `DocsConfig::topology`, `.orbit/config.yaml` schema docs, `/orb:setup` §6d — but does not name the two other plugin skills that reference `docs.topology` today. Dropping the config key without touching those leaves dangling prose that contradicts the new substrate shape.
**Evidence:**
- `plugins/orb/skills/distill/SKILL.md:178` — "If `.orbit/config.yaml` is present and the `docs.topology` key is configured, ask: did any of the newly-written cards correspond to a *subsystem-level capability*…" — gates the topology-update step on the dropped config key.
- `plugins/orb/skills/release/SKILL.md:69` — "Run `orbit audit topology` (only when `.orbit/config.yaml` exists and `docs.topology` is configured — the audit auto-detects this and exits cleanly…)" — names the dropped predicate; with ac-02's new "configured iff `.orbit/topology/` exists AND ≥ 1 entry" rule, the prose drifts on day one of the migration's ship.
**Recommendation:** Extend ac-04 to include `plugins/orb/skills/distill/SKILL.md` and `plugins/orb/skills/release/SKILL.md` in the "prose updates to the new shape" list, with grep verification asserting `docs.topology` is removed from both. Re-anchor each skill's predicate to "`.orbit/topology/` exists and is populated" per ac-02's canonical predicate.

### [MEDIUM] Brownfield setup behaviour unaddressed for repos with previously-wired `docs.topology`
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-04 drops `docs.topology` and the `DocsConfig::topology` field. The spec's goal asserts "the just-shipped capability has zero adopters and there is no existing topology.md to import" — true for this repo (no `docs/` dir present). But spec 2026-05-18-topology-substrate-wires shipped a `/orb:setup` brownfield-accept path; any repo that ran `/orb:setup` on or after 2026-05-18 may now carry `.orbit/config.yaml` with `docs.topology: docs/topology.md` and a stub doc at that path. Post-migration, `serde_yaml::from_str::<Config>` on those files will fail with `unknown field topology` (ac-04 asserts `DocsConfig::topology` absent — `deny_unknown_fields` propagation is the existing pattern across schema.rs). The spec does not name how those repos transition.
**Evidence:**
- `orbit-state/crates/core/src/schema.rs:1059` — "ac-03 verification: inner DocsConfig also rejects unknown fields" (existing test) — confirms the rejection behaviour will trigger.
- Design-note "Deferred items" §2 names "pre-existing consumer-repo `docs/topology.md` migration" as out of scope for the migration verb, but does not address the **config-load failure** that follows the field drop.
**Recommendation:** Pick one and write it into the spec:
- **Option A — graceful field removal.** Add an explicit AC: `docs.topology` is removed from the deny-unknown-fields fence (e.g. via `#[serde(default, deserialize_with = "drop_legacy_topology")]` or by leaving `DocsConfig` permissive for that one key with a deprecation note logged at parse time). Verification: a fixture `.orbit/config.yaml` carrying `docs.topology: docs/topology.md` still parses post-migration; a deprecation warning is emitted once.
- **Option B — `orbit topology setup` cleans up.** Extend ac-05 so the new setup verb, when run on a repo whose `.orbit/config.yaml` carries `docs.topology`, removes that key from the config (and optionally relocates the old `docs/topology.md` into a quarantine path or simply leaves it for the operator to delete).
- **Option C — explicit non-goal.** State in the goal that consumer-repo upgrade requires manual `docs.topology` removal; document that path in the spec, not just the design-note. Pair with a CHANGELOG/release-note AC so the breaking change is communicated.

The recommendation is Option A or B — Option C makes the migration breaking on field-parse alone for any repo that ran `/orb:setup` since 2026-05-18.

### [LOW] Schema field types in ac-01 leave id-or-path resolution undefined
**Category:** test-gap
**Pass:** 2
**Description:** ac-01 declares `decision_record (optional, list of choice ids or paths)` and similar mixed-type latitude on `operational_doc` and `test_surface` ("paths or test-target identifiers"). The dangling-pointer drift check in ac-02 verification depends on whether these strings resolve to filesystem paths. The spec doesn't say whether the schema accepts an opaque string (no resolution) or requires the parser to resolve choice ids to paths before drift-checking. The two behaviours yield different drift findings on the same input.
**Evidence:** ac-02 verification names "dangling canonical_code pointer" as a drift case but does not name how a "choice id" (rather than a path) resolves to a check-against-fs target.
**Recommendation:** ac-01 should pin one of: (i) all fields store opaque strings and drift detection is path-only on `canonical_code` / `operational_doc` / `test_surface`, with `decision_record` ids resolved through `Layout::choice_path_for(id)`; or (ii) the schema canonicalises to paths at parse time and drift detection is uniform across fields. Either is fine; the choice changes the round-trip and drift tests in ac-01/ac-02.

### [LOW] `DocsConfig` empty-struct fate unspecified
**Category:** test-gap
**Pass:** 2
**Description:** Today `DocsConfig` has exactly one field — `topology`. ac-04 says "the DocsConfig struct in orbit-state… lose the topology field". With the only field removed, `DocsConfig` becomes a zero-field struct and `Config::docs` becomes a struct-with-no-fields wrapper. The spec doesn't say whether `DocsConfig` is deleted entirely (and `Config::docs` along with it) or kept as a forward-extension hook.
**Evidence:**
- `orbit-state/crates/core/src/schema.rs:480` — `pub docs: Option<DocsConfig>` — sole field's removal leaves `Config::docs` semantically empty.
- ac-04 verification greps for `DocsConfig::topology` absent but does not assert anything about `DocsConfig` itself.
**Recommendation:** Pick one and add a verification line:
- **Delete `DocsConfig` and `Config::docs`** — cleaner, matches "convention beats config" rationale; existing `.orbit/config.yaml` files carrying a `docs:` block fail to parse (couples with Finding 2's Option A/B).
- **Keep `DocsConfig` as `#[serde(deny_unknown_fields)] struct {}`** as a forward hook — uglier but lets the `docs:` block round-trip without error if anything else wants to extend it later.

### [LOW] `parse_topology_doc` removal not explicitly verified
**Category:** test-gap
**Pass:** 1
**Description:** ac-02 verification reads "assert markdown header-scanning code is removed". The canonical name of that code is `parse_topology_doc()` at `verbs.rs:2910` (and its caller `load_topology_entries` at :2975). The spec relies on "markdown header-scanning" as the grep target; a stricter reviewer would expect the function names to be the assertion target.
**Evidence:** `orbit-state/crates/core/src/verbs.rs:2910` `fn parse_topology_doc(text: &str) -> Vec<TopologyEntry>` — the parser the spec is removing.
**Recommendation:** Tighten ac-02 verification: "grep `verbs.rs` for `fn parse_topology_doc` and `fn load_topology_entries` and assert absent". Also call out the existing test at `verbs.rs:7237` ("config-present-but-docs.topology-absent") as needing removal or rewrite — it tests a predicate that no longer exists.

---

## Honest Assessment

The plan is structurally sound and the supersession story is clean — choice 0025's rationale is well-served, the AC sequencing in the design-note (schema → parser → skill → wiring → setup verb) maps cleanly onto ac-01..ac-05, and the parity-test contract preserves the substrate-engagement envelopes shipped by the predecessor spec. The biggest risk is **silent breakage on `.orbit/config.yaml` parsing in any repo that adopted `/orb:setup` since 2026-05-18** (Finding 2). The goal's "zero adopters" framing is true for this repo today but the field drop is unforgiving — any repo that ran the just-shipped setup carries `docs.topology` in its config, and removing the field from the deny-unknown-fields-fenced struct turns config-load into a hard failure on next session prime. The other findings are tighten-the-spec issues rather than risks; Finding 1 is the most likely to bite during implementation because grep-driven cleanup will catch the distill/release references whether they're in the AC or not, but the verification line should name them explicitly so the implementer has a closed checklist.
