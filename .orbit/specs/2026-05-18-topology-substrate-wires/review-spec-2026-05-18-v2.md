# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-18-topology-substrate-wires
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (CLI + MCP parity, shared `.orbit/config.yaml`, additive envelope shape across three verbs) | 0 |
| 3 — Adversarial | not triggered | — |

## Cycle-2 disposition — prior review findings

Reviewing the v2 spec against the v1 review's five findings. The v1 verdict was REQUEST_CHANGES on one MEDIUM + three LOW + one INFO. Each is checked against the current spec text:

| v1 finding | Severity | Cycle-2 status | Evidence in current spec |
|------------|----------|----------------|--------------------------|
| ac-03 heuristic does not read design-note.md | MEDIUM | Addressed | ac-03 description now reads *"read the spec's spec.yaml and (when present) interview.md and design-note.md"*; design-note.md justification cited inline (*"canonical sidecar substrate per the artefact table in METHOD.md"*); verification adds the dedicated fixture *"spec close on a fixture spec where the subsystem name appears ONLY in design-note.md (not spec.yaml or interview.md) returns topology_warnings populated — proves the design-note.md scan is wired"*. |
| ac-02 "not configured" predicate underspecified | LOW | Addressed | ac-02 now binds the predicate exactly: *"Omit the topology_drift key iff `audit_topology(...).configured == false` (canonical predicate defined at verbs.rs:863-865 — true iff `.orbit/config.yaml` exists AND `docs.topology` is set)"*. Verification adds the fourth fixture *"a fixture where `.orbit/config.yaml` exists but `docs.topology` is unset returns the envelope without a topology_drift key (skip-on-default fires on configured==false, not only on file-absent)"*. Substrate cross-check: verbs.rs:863-865 confirms the cited line range and the canonical wording. |
| ac-01 brownfield-accept parent-dir absence | LOW | Addressed | ac-01 description adds *"If the target path's parent directory does not exist, /orb:setup creates the parent directory tree before writing the stub"*; verification adds *"Brownfield-accept on a fixture where docs.topology=docs/architecture/topology.md and docs/architecture/ does not exist; assert both the directory tree and the stub are created"*. |
| ac-04 flag name and render target left to implementation | LOW | Addressed | ac-04 locks the flag name (*"suppressible via the --no-nudge flag — mirrors --no-edit / --no-verify naming convention"*) and the render target (*"CLI renders the nudge to stderr in human mode (stderr is the conventional channel for advisory output; stdout is reserved for the primary verb output)"*). Verification asserts the stderr contract explicitly: *"CLI human mode renders the nudge to stderr (assert via captured stderr stream; stdout must NOT contain the nudge text)"*. |
| ac-03 self-reference on this spec's own close | INFO | Addressed | ac-03 now states the design intent inline: *"A spec that modifies a subsystem will typically mention that subsystem's name and trigger a warning at its own close — this is intended; the warning is non-blocking, and self-reference is exactly the design intent"*. |

All five v1 findings resolved. The MEDIUM was the load-bearing one; the LOWs were tightening tasks; all are now closed in-text with both a description change and a matching verification step.

## Findings

No new findings in Pass 1 or Pass 2.

**Pass 1 — structural scan, zero findings:**

- All ACs carry concrete verification methods naming fixture states and grep assertions. None is "works correctly" / "handles errors gracefully" shape.
- No constraint conflict. The skip-on-default contract (`configured == false` ⇒ key omitted) is consistent across ac-02 and ac-03.
- Scope matches goal one-to-one: the five surfaces named in the goal (`/orb:setup`, session prime, spec.close, memory.remember, audit) map to ac-01 through ac-05.
- Error / rollback / monitoring posture: all four work-ACs are additive envelope additions on stable structs (`SessionPrimeResult`, `SpecCloseResult`, `MemoryRememberResult`) — rollback is field removal; blast radius is contained by parity tests. The observation AC defers correctly per ac-taxonomy.
- Gate-AC deterministic checks: ac-02 (gate=1, description 873 chars, no placeholder token) — PASS. ac-03 (gate=1, description 1693 chars, no placeholder token) — PASS.

**Pass 2 — assumption & failure analysis, zero findings:**

- Substrate-dependency assumptions are all named and verifiable. `TopologyDriftEntry` reuse is called out twice (ac-02 *"do NOT redeclare the type; import or re-export it"*, ac-03 *"same DRY rule as ac-02"*); verbs.rs:873 confirms the type exists. `audit_topology(...).configured` predicate cited at verbs.rs:863-865 — confirmed present at exactly that range.
- `regex::escape` carry-forward from the parent spec's cycle-2 review is explicit in ac-03 (*"MUST be passed through regex::escape before interpolation … unescaped name would silently mis-match"*) and reinforced by a dedicated verification fixture (*"a fixture where a topology entry has a regex-metacharacter-bearing subsystem name (e.g. \"foo.bar\") returns the expected match/no-match outcome — proves regex::escape is applied (test fails if escape is missing)"*).
- Failure modes covered: short-name false positives (≥5-char filter, with negative fixture); substring-vs-word-boundary (negative fixture); idempotency on /orb:setup re-invocation (positive fixture); brownfield decline path (operator-can-decline, leaves rest of orbit working).
- Test adequacy: every AC's verification probes each branch of the contract, including the negative paths (no-prompt, no-nudge, no-overwrite). CLI + MCP parity is asserted on the three envelope-extending ACs.

## Honest Assessment

The v1 review's MEDIUM and three LOWs have each been resolved with both a description change and a matching verification step — not a token edit, but a real tightening. The design-note.md inclusion in ac-03 is the most important: it closes the gap where the heuristic would have under-flagged drift on this very spec's own drives, and the new fixture proves the scan is wired rather than asserted-only-in-prose. The skip-on-default predicate is now bound to `audit_topology(...).configured` directly with a verified line citation, removing the ambiguity that could have produced three different valid implementations. ac-01's parent-dir clause and ac-04's flag + stderr locks are small but real spec-vs-implementation boundary clarifications.

Five ACs, four of similar shape (verbs.rs envelope add + parity test on CLI + MCP), one observation deferred to 2026-06-15. Substrate cited is correct (verified line ranges in verbs.rs). Implementation sequencing in the design note is sensible. No new risks surfaced.

Proceed to implement.
