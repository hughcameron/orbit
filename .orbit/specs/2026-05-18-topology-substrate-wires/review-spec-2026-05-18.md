# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-18-topology-substrate-wires
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 4 (1 MEDIUM, 3 LOW) |
| 2 — Assumption & failure | content signals (cross-system boundaries, additive envelope shape, config files) + Pass 1 findings | 3 (1 MEDIUM rolled into Pass 1, 2 additional LOW) |
| 3 — Adversarial | not triggered | — |

## Findings

### [MEDIUM] spec.close heuristic does not read design-note.md

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-03's "subsystems touched" heuristic concatenates `spec.yaml` and `interview.md` and runs the word-boundary regex against that text. Since spec 2026-05-12 the design-note has become a canonical sidecar (it's the load-bearing artefact this very spec uses to pin the closed-design approach), and design-notes routinely name subsystems by their canonical name when they capture the pinned approach. Excluding it from the scan blinds the heuristic to its most discriminating evidence source.
**Evidence:** ac-03 description: *"read the spec's spec.yaml and (when present) interview.md, then for each topology entry whose subsystem name…"*. The spec's own `design-note.md` mentions `session prime`, `spec.close`, `memory.remember`, `verbs.rs`, `/orb:setup` — all candidate subsystem names — and would not be scanned. Compare: METHOD.md's artefact table now lists design-note alongside interview as canonical sidecars.
**Recommendation:** Extend the concatenation to `spec.yaml + interview.md + design-note.md`, each `when: present`. Update the verification line accordingly (a fixture spec with a topology-touching subsystem name appearing only in `design-note.md` returns a populated `topology_warnings`).

### [LOW] ac-02 "not configured" predicate is underspecified

**Category:** test-gap
**Pass:** 2
**Description:** ac-02 says the `topology_drift` key is omitted "when `.orbit/config.yaml` is absent **or** the topology capability is not configured." It does not define "not configured" precisely. `AuditTopologyResult.configured` (verbs.rs:865) is the canonical truth source ("True when `.orbit/config.yaml` exists AND `docs.topology` is set") — the AC should defer to that field rather than re-derive the predicate.
**Evidence:** verbs.rs:863-865 documents the canonical `configured` boolean. ac-02 verification line tests three states (configured-clean, configured-drift, no-config-file) but does not cover the fourth: config-file-present-but-docs.topology-absent. Implementation could legitimately omit the key, emit empty array, or emit a synthetic missing-pointer drift — ambiguity is bug-prone.
**Recommendation:** Tighten ac-02 to: *"Omit the topology_drift key iff `audit_topology(...).configured == false`."* Add a fourth verification case: a fixture with `.orbit/config.yaml` present but `docs.topology` unset returns the envelope without the `topology_drift` key.

### [LOW] ac-01 brownfield-accept stub creation does not address parent-dir absence

**Category:** failure-mode
**Pass:** 2
**Description:** ac-01 says brownfield-accept creates the stub file "at that path" when `docs.topology`'s target does not exist on disk. Default is `docs/topology.md` (top-level `docs/` directory likely exists in most repos), but the operator can configure any path (`docs/architecture/topology.md`, `.orbit/topology.md`). The AC does not say whether `/orb:setup` creates missing parent directories.
**Evidence:** ac-01 description: *"if the target path named by docs.topology does not already exist on disk, /orb:setup ALSO creates the stub file at that path"*. No clause about parent directories.
**Recommendation:** Add one sentence to ac-01: *"If the target path's parent directory does not exist, /orb:setup creates the parent directory tree before writing the stub."* Add a verification case: brownfield-accept on a fixture where `docs.topology=docs/architecture/topology.md` and `docs/architecture/` does not exist; assert both the directory and the stub are created.

### [LOW] ac-04 flag name and render target locked at implementation, not at review

**Category:** test-gap
**Pass:** 1
**Description:** ac-04 leaves two implementation choices open: the suppress-flag name ("`--no-nudge` or equivalent — final flag name decided in implementation") and the CLI render destination ("stdout in human mode (or stderr — implementation choice, locked at implementation)"). For a feature whose surface is "the nudge fires in the envelope", these are minor — the envelope shape is what matters. But the verification line refers to *both* possibilities, which weakens the testability of those facets.
**Evidence:** ac-04 description: *"suppressible via `--no-nudge` flag or equivalent — final flag name decided in implementation"*; verification: *"With `--no-nudge` (or chosen equivalent flag name), no nudge fires…"*.
**Recommendation:** Pick the flag name now (`--no-nudge` is the obvious choice, mirrors `--no-edit`, `--no-verify` etc.) and lock it in the AC. Pick the render target now (stderr is the conventional channel for advisory output; stdout is reserved for the primary verb output in human mode). Replace the two parenthetical caveats with the chosen values. Implementation should not be choosing UX surface names that ACs assert against — that's the spec's job.

### [INFO] Self-reference in word-boundary heuristic on this spec

**Category:** test-gap
**Pass:** 2
**Description:** This spec's own `spec.yaml` mentions `spec.close`, `session prime`, `memory.remember`, `/orb:setup` — all candidate subsystem names. When this spec is itself closed, ac-03's heuristic will flag any drift in those subsystems (which is precisely the design intent). Worth noting that the spec does not pre-emptively try to deduplicate self-references; this is the right call (drift in subsystems the spec touches is exactly what we want flagged), but a brief sentence in `ac-03` confirming the design intent would prevent a future reviewer from filing it as a bug.
**Evidence:** ac-03 description does not discuss self-reference. design-note.md does not either.
**Recommendation:** Optional. Add one sentence to ac-03: *"A spec that modifies a subsystem will typically mention that subsystem's name and trigger a warning at its own close — this is intended; the warning is non-blocking."* No verification change needed.

---

## Honest Assessment

This is a tight follow-on spec. The substrate it consumes is shipped (verified — `TopologyDriftEntry` lives at verbs.rs:873; `audit_topology` at verbs.rs:2866; `DocsConfig` + FIELDS at schema.rs:487 with the lockstep test at schema.rs:1038-1040; `SessionPrimeResult` / `SpecCloseResult` / `MemoryRememberResult` all exist and are stable). Five ACs, four work-bands of similar shape (verbs.rs envelope add + parity test) plus one observation. Implementation sequencing is well-thought-through; the DRY-against-`TopologyDriftEntry` and `regex::escape` carry-forwards from the parent spec's cycle-2 review are explicit.

The biggest risk is the ac-03 design-note omission — the heuristic's most discriminating evidence source is the artefact most likely to name subsystems by their canonical handles. That's the MEDIUM finding above. Without it, spec.close will silently under-flag drift on this very spec's drives — false-negatives are quieter than false-positives, easy to miss in practice.

The three LOW findings are tightening tasks, not rework. ac-02's "not configured" predicate, ac-01's parent-dir handling, and ac-04's flag-name lock are each a one-line clarification.

Recommend addressing the MEDIUM finding (extend ac-03 to read `design-note.md`) plus the three LOW findings in a single cycle, then proceed to implement. No design rework needed.
