# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-24-port-acceptance-shim
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | Pass-1 medium findings + content signal (cross-surface CLI/MCP parity, semantic-mismatch on shared helper) | 2 |
| 3 — Adversarial | not triggered (Pass-2 surfaced no structural cascade — issues are localised to AC wording / surface-extraction work) | — |

## Findings

### [HIGH] AC-05 mis-names the substrate it wraps — there is no `spec update --ac-check` verb body to rewrap

**Category:** missing-requirement
**Pass:** 1
**Description:** AC-05 reads "alias or rewrap of the existing `orbit spec update --ac-check` verb body — same idempotency: returns ok on first check, `Error::not_found` for unknown AC, `Error::conflict` for already-checked AC". That framing implies a callable core verb already encodes the per-AC flip logic. It doesn't. The `--ac-check` / `--ac-uncheck` read-mutate-write lives in the **CLI layer** at `orbit-state/crates/cli/src/main.rs:1037-1080` (inside `SpecAction::Update` → `build_request`). Core's `spec_update` (`verbs.rs:1866`) accepts a pre-mutated `acceptance_criteria: Option<Vec<AcceptanceCriterion>>` — it is a full-list replacement primitive, not an idempotent per-AC flipper. MCP today reaches `spec.update` directly (per `crates/mcp/src/...:184`), so MCP callers would already need to do the read-mutate-write themselves; `--ac-check`'s idempotency contract is a CLI-only concession.

**Evidence:**
- `orbit-state/crates/cli/src/main.rs:1037-1080` — the `Some(_) | Some(_) → mutually exclusive`, AC-existence check, `already {state} → Error::conflict` ladder lives here.
- `orbit-state/crates/core/src/verbs.rs:1866-1908` — `spec_update` body: replaces `acceptance_criteria` wholesale when `Some(acs)` provided, no per-AC semantics.
- AC-07 requires byte-equal envelope assertions for the five new verbs against CLI/MCP **on a shared fixture spec**. If `spec check` is implemented purely as CLI sugar like `--ac-check` is today, MCP parity for `spec.check` requires the same read-mutate-write logic landing in core first.

**Recommendation:** Rewrite AC-05 to name the real shape of the work: "`orbit spec check <id> <ac-id>` ships as a **first-class core verb** (`spec_check` in `verbs.rs`) — own the read-mutate-write + idempotency logic that today lives in the CLI's `--ac-check` flag, returning `Error::not_found` for unknown AC, `Error::conflict` for already-checked AC, `Ok(())` on first check. The existing CLI `--ac-check` flag re-routes through the new verb (and `--ac-uncheck` either follows the same pattern or is deferred — name the choice)." Add a sibling note about the disposition of `--ac-check` / `--ac-uncheck` flags post-port: leave as sugar over the new verb, or deprecate?

---

### [HIGH] AC-06's "single source of truth" claim conflates two different AC-traversal semantics

**Category:** constraint-conflict
**Pass:** 1
**Description:** AC-06 says the helpers `next_unblocked`, `blocking_gate`, `has_unchecked` "are reused by `spec_close`'s AC pre-flight — single source of truth, no duplication between the close-time check and the new verbs". The shim's helpers (and therefore the new verbs per AC-01..AC-04) traverse on the **`gate: bool`** field. `spec_close`'s AC pre-flight (`verbs.rs:1980-1990`, per spec 2026-05-16-ac-taxonomy ac-02) filters on **`ac_type.blocks_close()`** — the Code/Config/Doc vs Ops/Observation taxonomy. These are orthogonal predicates: a Code AC may or may not be `gate: true`; a Doc gate AC blocks close on both axes; an Observation gate AC blocks the shim's `next-ac` but does **not** block `spec_close`. There is no single helper that both call sites can share without first deciding which predicate they actually want.

**Evidence:**
- `orbit-state/crates/core/src/verbs.rs:1980-1990` — `spec_close`'s pre-flight: `.filter(|ac| !ac.checked && ac.ac_type.blocks_close())` and `.filter(|ac| !ac.checked && !ac.ac_type.blocks_close())` for `deferrable_open`. No reference to the `gate` field for *blocking* decisions; `gate` only annotates the error suffix (`gate: {ids}`).
- `plugins/orb/scripts/orbit-acceptance.sh:115-126` — shim's `next-ac` traverses on `ac.get('gate')` to set the `blocked` flag, never reads `ac_type`.
- The shim was written before `ac_type` shipped (spec 2026-05-16-ac-taxonomy); the two predicates diverged when the taxonomy landed and the shim wasn't updated.

**Recommendation:** Pick one of:
- **(a)** Narrow AC-06 to name the *one* helper that actually has dual use — e.g. `has_unchecked_blocking(acs: &[AcceptanceCriterion]) -> bool` over `ac_type.blocks_close()` (close-time pre-flight) AND a separate `has_unchecked(acs)` over raw `!checked` (drive's implement-loop termination). Document that `next_unblocked` / `blocking_gate` are gate-axis helpers **not** shared with `spec_close`, which is taxonomy-axis.
- **(b)** Declare a unified predicate (e.g. an "is blocking right now" function parameterised by axis) that both consumers call with their axis token. More work, but genuinely single-source.
- **(c)** Drop the `spec_close` reuse claim from AC-06 and accept duplication on this small surface. Cleanest if (a) and (b) are over-engineering.

Whichever path, the spec needs to say so unambiguously — the implement agent will otherwise spend a half-session realising the helpers don't naturally compose.

---

### [MEDIUM] AC-08 + AC-10 leave the shim's role during the migration window under-specified

**Category:** missing-requirement
**Pass:** 1
**Description:** AC-08 says the shim "collapses to a thin compat wrapper that shells into the new verbs — preserves the existing CLI signature ... so any not-yet-rewritten call site keeps working through the migration window". AC-10 says the shim deletes in the same commit as the last call-site rewrite. Neither AC pins the ordering, and the "migration window" is implicit. Two valid orderings:
- **Wrapper-first:** ship new verbs → land shim-as-wrapper → rewrite SKILL.md sites incrementally → final commit deletes wrapper + test. Wrapper genuinely earns its keep.
- **Rewrite-then-delete:** ship new verbs → rewrite all SKILL.md sites in one go → delete shim + test. The wrapper step is dead code that ships only to be removed in the same PR.

The tabletop note (line 18) implies wrapper-first ("the shell shim stays in place as a wrapper over the new verbs until grep on every consumer SKILL.md returns clean"), but no AC says "intermediate commit must show the shim as a working wrapper" or "AC-08 and AC-10 may collapse into a single deletion if all rewrites land first".

**Evidence:**
- AC-08 says wrapper exists; AC-10 says wrapper deletes; no AC says the wrapper must exist for >0 commits.
- 31 call sites across 5 SKILL.md files (`rg orbit-acceptance.sh plugins/orb/skills/ | wc -l == 31`) — non-trivial rewrite surface, so an intermediate wrapper period is plausibly useful for splitting the PR.

**Recommendation:** Add one sentence to AC-08 or a new AC clarifying the ordering. Simplest: "AC-08's wrapper may be skipped (the shim deletes in the same commit as the verbs ship and the SKILL.md sites rewrite) if the implementer chooses a single-PR migration. AC-10's invariant — no orphaned wrapper, no partial decommission — holds either way." This explicitly authorises both orderings and removes a temptation to ship dead wrapper code purely to satisfy AC-08's literal reading.

---

### [LOW] No AC for behaviour of `spec.acs` against an empty or non-existent spec

**Category:** test-gap
**Pass:** 2
**Description:** AC-01 says "Empty acceptance_criteria → empty stdout exit 0". It does not say what happens for a non-existent spec id or a malformed spec file. The shim today shells `orbit spec show` and exits 2 with a stderr message on failure. The Rust verb path will naturally return `Error::not_found` (matching `spec.show`'s behaviour at `verbs.rs:6619`) — but the parity test in AC-07 won't cover it unless the fixture includes the failure case.

**Evidence:** `orbit-state/crates/core/src/verbs.rs:6619` — `spec_show_missing_id_is_not_found` test exists for the parallel case. AC-07 cites only "shared fixture spec" (singular).

**Recommendation:** Either widen AC-07 to "shared fixture specs including a missing-id case", or add a one-liner to AC-01 stating "missing spec id returns `Error::not_found`". Low priority — the verb authors will likely do the right thing — but the AC currently doesn't assert it.

---

### [LOW] AC-12 bundles three independent verifications under one criterion

**Category:** test-gap
**Pass:** 2
**Description:** AC-12 chains "`cargo test ...` passes; `orbit verify` returns zero drift; CHANGELOG.md entry added". These are three separable gates — if `orbit verify` fires drift after canonicalise (it shouldn't, but if), the AC can't be partially checked. Two of the three are mechanical (cargo + CHANGELOG); the `orbit verify` one is the substrate-correctness gate.

**Evidence:** The spec's other ACs are single-claim. AC-12 is the outlier.

**Recommendation:** Split into AC-12a / AC-12b / AC-12c, OR accept the conjunction and let the implement agent tick once all three are observed. Low impact — the implement skill handles compound ACs fine — but cleaner separation lets `spec.next-ac` flag exactly which sub-gate isn't met. Not a blocker.

---

## Honest Assessment

The plan is shipping-grade in shape — choice 0020 is the right north star, the verb-surface fork was correctly resolved to five thin sub-verbs in the tabletop note, and the decommission discipline (AC-09's `rg ... | wc -l == 0` gate + AC-10's atomic deletion) is well-formed. The two HIGH findings are real but local: AC-05 and AC-06 both wave at substrate that doesn't exist in the shape they describe, and a fresh implement agent reading these would either build the wrong thing or stall waiting for clarification. Fix the wording — particularly AC-06's helper-sharing claim, which currently asserts an architectural property that the underlying `ac_type` vs `gate` semantics don't support — and the spec is ready to drive. The biggest risk in flight is the implement agent attempting to "share" `next_unblocked` with `spec_close`'s pre-flight, discovering the predicate mismatch mid-implementation, and either over-engineering a unifier or silently building a parallel path that contradicts AC-06's claim. Resolve AC-06 first.
