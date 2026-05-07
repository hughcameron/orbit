# Spec Review

**Date:** 2026-04-20
**Reviewer:** Context-separated agent (fresh session)
**Spec:** .orbit/specs/2026-04-20-mission-resilience/spec.yaml
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (cross-system boundaries: session-context.sh hook, implement skill, review-spec skill; shared schema consumed by card 0003; backwards-compatibility implications for existing progress.md files) | 6 |
| 3 — Adversarial | not triggered — no cascading failures, rollback concerns, or contradictory assumptions surfaced | — |

---

## Findings

### [HIGH] "Checkpoint" on drift is undefined operationally
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-02 says the implement skill "requires a checkpoint before continuing" on hash mismatch, but nowhere is "checkpoint" defined. Is it (a) a user-facing prompt that blocks until human input, (b) an explicit acknowledgement token written to progress.md (e.g. `Drift acknowledged: sha256:<new>`), or (c) a re-run of `/orb:review-spec` gated on APPROVE? Without a concrete contract, two implementers will disagree on what passes ac-02. The test (`mrl_ac02_drift_detected_before_ac`) says "advancement blocks until acknowledged" — but "acknowledged" is also undefined, so the test is circular.
**Evidence:** spec.yaml lines 22-23 (ac-02 verification); constraint on line 8 mentions "Recomputation happens before each AC… and on resume" but no checkpoint semantics; the interview (Q3) says "surfaces a REQUEST_CHANGES-style notice… and checkpoints" — which suggests human-in-the-loop, but doesn't nail the mechanism.
**Recommendation:** Add an AC (or extend ac-02) specifying the exact acknowledgement mechanism. Suggested: on mismatch, the implement skill (i) prints the fixed drift notice, (ii) refuses to start the next AC until either the recorded hash is updated to the new hash in progress.md (via a named operator like `/orb:accept-drift` or a literal field write) or `/orb:review-spec` is re-run on the new spec. State which file/field carries the acknowledgement so the test can assert on it.

### [HIGH] Detour lifecycle (open vs. closed) is not specified
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-04 references "when a detour is closed (set to the Return to: target)" and ac-07 simulates "detour resolution," but the spec never defines what "closing" a detour means — is it a field flip (`Status: open|closed`), a convention (the presence of a resolution line appended beneath the entry), or purely an in-memory state of the implement skill? The template in constraint line 12 does not include an open/closed marker, yet ac-05 says "closed detours retain their Return to: field as the audit trail" — implying a distinction exists. ac-04's test asserts Current AC transitions back to the Return to: target on "detour close" with no definition of what triggers it.
**Evidence:** spec.yaml lines 32-33 (ac-04 description), 37-38 (ac-05), 46-48 (ac-07); ontology `detour_entry` (lines 80-82) lists date/description/return_to but no status/closed field; the Q5 template in interview line 57 shows a single entry form with no open/closed distinction.
**Recommendation:** Either (a) declare detours have no "open" state — each entry is recorded atomically at resolution, in which case ac-04/ac-07's "detour close" means "the act of appending the entry," or (b) extend the template with an explicit resolution marker (e.g. a `Resolved:` line beneath the entry, or a `Status: open`/`closed` field) and add an AC governing the open→closed transition. Pick one and make it testable.

### [HIGH] Backwards compatibility with pre-existing progress.md files is unspecified
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-02 and ac-03 both compare the current sha256 to "the Spec hash recorded in progress.md." If an in-flight progress.md from before this change has no Spec hash field, what does the implement skill do? Options: (i) treat absence as drift and surface the notice, (ii) treat absence as "hash unknown" and silently compute-and-record, (iii) fail noisily. None of these is specified. The exit condition on line 110 says "fresh /orb:implement run" — but the spec says nothing about upgrade paths for sessions started before deployment. Since the plugin is deployed via the marketplace (per CLAUDE.md), there will be in-flight progress files when this lands.
**Evidence:** spec.yaml lines 22-28 (ac-02/ac-03 assume the hash is always present); no migration AC; exit_conditions (lines 107-112) only cover fresh runs.
**Recommendation:** Add an AC covering missing-hash behavior. Minimum: the implement skill backfills Spec hash on first run against a progress.md lacking it, logs the backfill, and does not emit a drift notice on that run. Hook behavior (ac-03) should also be specified for the missing-hash case — most likely silent (no notice), since a hook can't distinguish "never recorded" from "deleted."

### [MEDIUM] AC-06 test does not cover non-gate ACs left unchecked
**Category:** test-gap
**Pass:** 1
**Description:** ac-06 states "refuses to start the current AC if any preceding AC with ac_type: gate is not marked [x]." It is silent about whether a preceding non-gate AC being `[ ]` also blocks. Typical expectation in declaration-order workflows is that ACs complete sequentially, but the spec nowhere says this. If ac-01 (code, unchecked) and ac-02 (code, unchecked), does the skill start ac-02? Implementation will land one way or another, but reviewers and the AC test have no rule to verify against. This ambiguity also affects ac-08 fixture A (ac-01 [x], ac-02 [ ]) vs the implicit case of both unchecked non-gate.
**Evidence:** spec.yaml lines 41-43; ontology_schema gate_ac (lines 83-85) only describes gate blocking; no ordering rule for non-gate ACs.
**Recommendation:** Add one sentence to ac-06 (or add ac-06b) stating whether non-gate preceding ACs also block, and extend the test. Either "Non-gate ACs do not block subsequent ACs — only gates block" or "The skill advances in declaration order and will not start ac-NN until ac-(N-1) is marked [x] unless explicitly skipped." Pick one, then test it.

### [MEDIUM] AC-11 verification is single-point and leaves "placeholder" / "describes no observable completion criterion" undefined
**Category:** test-gap
**Pass:** 2
**Description:** ac-11 says review-spec Pass 1 flags any gate whose verification is "empty, placeholder, or describes no observable completion criterion." The test fixture uses literal `"TBD"`. "Empty" and "TBD" are programmatic; "describes no observable completion criterion" is an LLM-judgment call that will vary between runs. Per engineering principle #3 in the user's CLAUDE.md ("LLMs for parsing, programmatic checks for validation"), this gate should have a deterministic rule. As written, ac-11 can pass while leaving most vague gates undetected.
**Evidence:** spec.yaml lines 66-68; compare with evaluation_principles weighting (line 100) treating gate determinism as 0.15.
**Recommendation:** Narrow ac-11 to a deterministic check (e.g. "verification field is non-empty, is not in the set {TBD, FIXME, TODO, placeholder, ...}, and is ≥ N characters"). If richer semantic detection is genuinely wanted, split it into (a) a deterministic structural check now and (b) a separate future card for LLM-assisted review.

### [MEDIUM] sha256 input normalization is not specified
**Category:** assumption
**Pass:** 2
**Description:** ac-01 says "sha256 of the fixture spec bytes," but the spec file passes through editors, git checkout line-ending conversion (core.autocrlf), and potentially YAML canonicalization. Two environments with identical logical specs can produce different sha256 values (CRLF vs LF, trailing newline, BOM). This will manifest as false drift notices on every session resume after a checkout on a different platform.
**Evidence:** spec.yaml line 18 (ac-01 verification — "sha256 of the fixture spec bytes"); no constraint or AC pinning byte-level normalization.
**Recommendation:** Either (a) declare the hash is computed over raw bytes as read from disk with no normalization (accept occasional false drift as cheap) and document the trade-off, or (b) specify a normalization step (strip trailing newline, normalize line endings to `\n`, UTF-8) and apply it in both the implement skill and session-context.sh. Add an AC or constraint that pins the choice so the two layers agree.

### [LOW] Drift-notice string is duplicated across ACs without a canonicalization source
**Category:** constraint-conflict
**Pass:** 2
**Description:** The exact string `"spec modified since implementation started, re-review recommended"` appears in ac-02, ac-03, and ontology_schema.drift_notice. If the wording changes, three edits are needed and tests will drift silently (ac-03 uses `grep` on stdout; ac-02 asserts on message emission). Low risk but cheap to fix.
**Evidence:** spec.yaml lines 22, 28, 87-88.
**Recommendation:** Either factor the string into a single place (e.g. a named constant in the implement skill and imported by the hook) or explicitly list the drift-notice string as a project constant referenced by both ACs. A shared fixture used by mrl_ac02 and mrl_ac03 is sufficient.

### [LOW] ac-08 fixture B wording is slightly tangled
**Category:** test-gap
**Pass:** 2
**Description:** Fixture B has ac-01 (gate) `[ ]` and ac-02 `[ ]`. ac-01 IS the next unchecked AC — it happens to also be a gate. The test asserts "stdout names ac-01 as blocking and ac-02 as next-after-gate," which adds a "next-after-gate" concept not described in ac-08's prose ("prints the next unchecked AC — including any blocking gate AC"). The two phrasings are compatible but the spec prose does not commit to printing the post-gate next AC.
**Evidence:** spec.yaml lines 51-53; ontology next_unchecked_ac (lines 89-91) says "When a gate AC is the next unchecked item, the notifier names it explicitly as blocking" — does not say "and also names the next-after-gate AC."
**Recommendation:** Either add "and the AC that would follow it once the gate closes" to ac-08's description, or relax fixture B to only assert ac-01 is named as blocking. Pick whichever matches the intended UX.

---

## Honest Assessment

The plan is structurally sound. Three-layer coverage is cleanly decomposed, ACs are almost entirely testable, constraints are consistent, and the ontology matches the criteria. The spec reads as the product of a thorough interview. What's missing is specification precision around two runtime contracts: (1) what "checkpoint" and "acknowledgement" mean on drift, and (2) what "close" means for a detour entry. Both are load-bearing for AC verification but are handled as if self-evident. Backwards compatibility with existing in-flight progress.md files is the third real gap — it's the one most likely to break on day one of deployment. None of these are hard blockers; they are concrete clarifications an implementer would otherwise invent silently, producing behaviour that drifts between test and production. The biggest risk is not complexity but ambiguity-absorbed-as-implementation: the implementer will pick a checkpoint shape and a detour close convention, and that becomes the de-facto schema for card 0003 and every downstream consumer — without ever having been decided. Fix the three HIGH findings, tighten ac-11 to deterministic checks, and this spec is ready.
