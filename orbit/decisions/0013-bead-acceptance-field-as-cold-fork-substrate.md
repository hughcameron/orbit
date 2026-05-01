---
status: accepted
date-created: 2026-05-01
date-modified: 2026-05-01
---
# 0013. Bead acceptance field as the cold-fork review substrate

## Context and Problem Statement

Decision 0011 (beads execution layer) committed orbit's execution discipline to beads, with `D2` naming the cold-fork review architecture as preserved — "Cold-fork stays. Reads from beads (acceptance field) instead of spec.yaml." The orbit-6da.2 drive rewrite shipped a *bridge*: drive rendered the bead's `bd show <bead-id> --json` output as a static markdown snapshot file (`orbit/reviews/<bead-id>/bead-snapshot-<date>.md`) and passed the snapshot path to the forked reviewer. The reviewers continued running their spec.yaml-shaped Pass-1 rules against the snapshot.

Two structural problems with the bridge:

1. **Gate-AC verification check silently no-ops.** review-spec's Pass 1 deterministic check is calibrated for spec.yaml's `ac_type: gate` + `verification: <text>` shape. The markdown snapshot collapses the AC structure — gate-marker information becomes inline text the snapshot reader can't reliably distinguish from non-gate ACs without re-parsing. The check stops firing for the bead substrate.
2. **AC interpretation drift between implement and review.** `/orb:implement` (orbit-6da.1) reads ACs via `parse-acceptance.sh acs <bead-id>`. The reviewers read a markdown render. Two parsers, two interpretations — the bridge guarantees they can disagree at the boundary.

The card 0016 work removes the bridge: reviewers read the bead directly via `bd show` and parse ACs via the same `parse-acceptance.sh` that implement uses.

## Considered Options

### S1: Where does the bead read happen?
- **A. Skill reads the bead directly.** Drive's brief carries only `<bead-id>` + verdict-output path. The review skill calls `bd show <bead-id> --json` and `parse-acceptance.sh acs <bead-id>` itself. Skill is substrate-aware; brief is minimal.
- **B. Drive pre-renders structured payload.** Drive parses + serialises ACs into the brief. Skill stays substrate-agnostic.

### S2: How does the spec.yaml `ac_type: gate` / `verification: <text>` shape map to bead ACs?
- **A. Extend the bead acceptance convention.** Add a multi-line per-AC format with optional `verification:` continuation line. parse-acceptance.sh gains a verification subcommand.
- **B. Treat the AC description text as the verification statement.** The bead AC line is `- [ ] ac-NN [gate]: <description>`; the description IS the verification. Pass-1 rules (non-empty / not-placeholder / ≥20 chars) fire against the description text for `is_gate=1` rows.
- **C. Drop the gate-AC check.** Accept the regression.

### S3: How does `ac_type: code | doc | gate | config` map to bead ACs?
- **A. Add `ac_type` metadata to bead acceptance.** Extend the convention and parser.
- **B. Treat all ACs as code-typed; reviewer judges per-AC.** Search for tests; honest-assessment paragraph contextualises which uncovered ACs are doc/gate-style vs genuine test gaps.
- **C. Stash `ac_type` in bead `--set-metadata`.** Off-line metadata, parser-readable.

### S4: How does gate semantics propagate from card scenario to bead AC?
- **A. promote.sh extension.** Add optional `gate: true` field to card scenarios; `promote.sh` emits `[gate]` token on the corresponding AC line.
- **B. Hand-edit acceptance fields.** Bead authors annotate gates after promotion via `bd update --acceptance`.
- **C. Drop scenario-level gating.** Gates are always added by hand.

### S5: Backward compatibility with snapshot-bridge drives in flight?
- **A. Hard cutover.** Remove the bridge entirely. In-flight drives finish under their start-time skill version.
- **B. Dual-mode with feature flag.** Skills accept both substrates; flag selects.
- **C. Auto-detect.** Skill checks if the brief argument is a bead-id or a file path.

## Decision Outcome

**S1: A — Skill reads the bead directly.** The whole point of the card is to remove the substrate translation step. Drive pre-rendering would just recreate the snapshot bridge by another name. Future bead schema changes need fresh `bd show` reads, not a frozen drive-side render.

**S2: B — AC description text as verification statement.** The bead acceptance convention is intentionally one-line-per-AC for parser simplicity. Extending it (S2 A) is a substrate change that exceeds the card's scope and would force re-promotion of every in-flight bead. The promote.sh template already concatenates the scenario name with the `then`-clause as the AC description, preserving the verification semantics in the description text.

**S3: B — All ACs treated as code-typed; reviewer judges per-AC.** The bead acceptance convention has no `ac_type`. Inventing one extends the convention scope past what this card is doing. A reviewer with the bead's description in hand can read each AC and judge whether it's a code AC; the deterministic check becomes "search for tests; report what you find," not "search for tests; fail if any required AC has no test."

**S4: A — promote.sh extension with optional `gate: true` on card scenarios.** Without this, gate semantics survive only on hand-edited bead acceptance fields. The card scenarios → promoted bead pipeline is the canonical authoring path; gate semantics must propagate end-to-end or the substrate-parity claim is structurally false (the cycle-1 review caught this).

**S5: A — Hard cutover.** Decision 0011 D2 commits to the cold-fork reading from beads, not from a translated artefact. Dual-mode preserves the bridge as a maintained code path indefinitely. The orbit-6da.2 bridge was explicitly a stop-gap. Pre-cutover drives finish under their start-time skill version (same migration discipline as the forked-reviews migration shipped earlier).

### Substrate mapping (single source of truth)

| spec.yaml field        | Bead substrate equivalent                                                       |
|------------------------|---------------------------------------------------------------------------------|
| `ac.id`                | `ac-NN` token in the AC line (column 1 of `parse-acceptance.sh acs`)            |
| `ac.ac_type == "gate"` | `parse-acceptance.sh acs` row where `is_gate == 1` (column 4)                   |
| `ac.verification`      | The AC description text (column 3 of `parse-acceptance.sh acs`)                 |
| `ac.ac_type == "code"` | Reviewer-judged from description text; no field mapping                         |
| `metadata.test_prefix` | Removed from review-pr — bare `ac<NN>` test names. (Decision 0002 superseded for the review-pr surface only; `test_prefix` stays live in spec/spec-architect/audit/implement.) |
| Card `scenario.gate`   | New optional field on card scenarios; promote.sh emits `[gate]` on the resulting AC line |

### Consequences

- **Good** — Cold-fork review preserved end-to-end. Gate-AC deterministic check fires for `is_gate=1` rows under bead substrate (was silently no-op under the bridge).
- **Good** — AC interpretation cannot drift between `/orb:implement` and either review skill. All three call `parse-acceptance.sh acs <bead-id>`.
- **Good** — Card scenario `gate: true` propagates through `promote.sh` to bead AC `[gate]` marker — gate semantics no longer require hand-editing acceptance fields after promotion.
- **Good** — Brief is shorter than the snapshot-bridge version, not longer. No intermediate artefact to maintain or commit.
- **Bad** — Lost fidelity on `ac_type: doc | config` exemption: review-pr no longer auto-exempts non-code ACs from the test-coverage check. Reviewer must judge per-AC from description text. Mitigation: most AC types are obvious from description ("Document the X" vs "Implement Y"); the honest-assessment paragraph contextualises uncovered ACs.
- **Bad** — Lost AC-to-commit provenance signal that `progress.md` provided. Under bead substrate `parse-acceptance.sh acs` reports `[x]` status but no commit/file pointer per AC. Reviewers rely on the diff plus AC text to verify implementation. Mitigation: a fresh-context reviewer reading the diff can usually identify which hunk maps to which AC; explicit provenance was a nice-to-have, not load-bearing.
- **Bad** — Cycle-history `[x]` leak under bead substrate. On a cycle-2 review-spec re-fork after a cycle-1 REQUEST_CHANGES + implement-edit pass, the fork sees ACs already marked `[x]` by the prior implement work. Under spec.yaml substrate the snapshot was static; under bead substrate the live `bd show` query exposes implementer state. Bounded by drive ordering (review-spec runs before implement on the canonical path) and accepted as the substrate cost. The brief's "no conversation context" prohibition continues to hold byte-for-byte; this consequence clarifies that "cold fork" means "no conversation history" and not "no AC status state."
- **Neutral** — Decision 0002 (`ac-test-prefix`) is partially superseded: the `test_prefix` convention remains live in `/orb:spec`, `/orb:spec-architect`, `/orb:audit`, `/orb:implement` for spec.yaml-flavoured projects. Under bead substrate review-pr no longer reads `test_prefix` (the bead acceptance field has no metadata field for it). Decision 0002's status header is updated to `superseded by 0013 (review-pr scope only)`.
