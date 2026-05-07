Spec path: .orbit/specs/2026-04-20-mission-resilience/spec.yaml
Spec hash: sha256:289938527266aa32a0ebb31b4741a741b91370a764b0ad609341f2fae54d8b4b
Started: 2026-04-20
Current AC: none

# Implementation Progress

## Hard Constraints
- [x] Card 0009 owns the progress.md schema — schema lands in implement/SKILL.md §4 and this progress.md; card 0003 will consume.
- [x] Three-layer implementation — progress.md (state), session-context.sh (surface), implement/SKILL.md (behaviour) all updated; each owns a distinct phase.
- [x] Reuse ac_type: gate — no new gate: boolean or blocks: list; annotation `(gate)` is human-readable only; runtime enforcement stays in implement/SKILL.md §4b(3).
- [x] Detours live inside progress.md — ## Detours section defined in implement/SKILL.md template, positioned between ## Hard Constraints and ## Acceptance Criteria.
- [x] Detour entries atomic and terminal — SKILL.md §4c commits "the append IS the resolution"; no open/closed state, no Status field.
- [x] sha256-based drift — both session-context.sh (compute + compare) and implement/SKILL.md §4b(2) (pre-AC check) wired; Monitor-tool deferred.
- [x] Raw-byte sha256 — session-context.sh uses sha256sum (binary mode); implement/SKILL.md §4a documents raw-bytes contract.
- [x] Canonical DRIFT_NOTICE — single source in implement/SKILL.md §4a; session-context.sh declares `DRIFT_NOTICE=...` matching verbatim.
- [x] Rally-level AC progress out of scope — no edits to rally.yaml schema or rally skill in this spec.
- [x] Implement skill remains runtime enforcer — review-spec Pass 1 added structural check only (§Pass 1 rule 5); does not replace SKILL.md §4b.
- [x] Parser ignores ## Detours — session-context.sh awk parser scans inside ## Acceptance Criteria block only (verified with fixture: [x] in Detours did not mark ac-02 done).
- [x] Authoritative progress.md template verbatim per interview Q5 — implement/SKILL.md §4 matches.
- [x] Non-gate ACs do not block — SKILL.md §4b(3) "Non-gate preceding ACs do not block"; review-spec check only runs on ac_type: gate.
- [x] Backwards compatibility — SKILL.md §4b(1) backfill documented; session-context.sh grep returns empty when field absent, drift block does not fire.
- [x] Pre-AC sequence fixed: (1) backfill → (2) drift → (3) gate — SKILL.md §4b structured in that order.
- [x] ac-02 non-interactive branch — SKILL.md §4b(2) documents TTY + ORBIT_NONINTERACTIVE=1 branch with exit 1.

## Detours

## Acceptance Criteria
- [x] ac-01 (code): progress.md includes `Spec hash: sha256:<hex>` — this file demonstrates (sha matches raw bytes of spec.yaml); template in implement/SKILL.md §4.
- [x] ac-02 (code): pre-AC drift check with interactive/non-interactive branch — implement/SKILL.md §4b(2) specifies exact AskUserQuestion contract (interactive) and exit 1 (non-interactive); missing-hash defer to §4b(1).
- [x] ac-03 (code): session-context.sh recomputes sha256 and emits DRIFT_NOTICE on mismatch; silently skips when `Spec hash:` line absent. Fixture verified (matching=no notice, mismatch=notice, absent=no notice).
- [x] ac-04 (code): Current AC field defined in template (SKILL.md §4); update-on-advance and update-on-detour-append documented in §4c.
- [x] ac-05 (code): ## Detours section positioned between ## Hard Constraints and ## Acceptance Criteria; atomic/terminal entry format (`YYYY-MM-DD: <desc>` + `Return to: <ac-id>`) documented in §4 + §4c.
- [x] ac-06 (code): gate enforcement sequence in SKILL.md §4b(3) — walk declaration order, refuse on unchecked preceding gate, name blocking id; non-gate ACs explicitly do not block.
- [x] ac-07 (code): re-anchor after detour documented in SKILL.md §4c — (i) set Current AC to Return to target, (ii) re-read progress.md and AC list, (iii) select from first unchecked AC in ## Acceptance Criteria.
- [x] ac-08 (code): session-context.sh prints next unchecked AC; when it's a gate, names blocking + post-gate. Fixture verified: "Next AC — ac-01 is a blocking gate. ac-02 becomes startable once ac-01 closes."
- [x] ac-09 (code): parser discipline — session-context.sh awk scans only inside ## Acceptance Criteria section; fixture with `[x] ac-02` inside ## Detours verified to still report ac-01 as next unchecked (detour content ignored).
- [x] ac-10 (doc): implement/SKILL.md documents template verbatim (grep confirms `Spec hash:`, `Current AC:`, `## Detours`, `- [ ] ac-NN (gate):` all present); DRIFT_NOTICE declared once in SKILL.md §4a and referenced literally in session-context.sh (1 occurrence in each file).
- [x] ac-11 (code): review-spec/SKILL.md Pass 1 rule 5 adds deterministic gate-verification check (empty / placeholder-token set {TBD, TODO, FIXME, PLACEHOLDER, XXX, ???} case-insensitive / <20 chars); flag MEDIUM naming gate id and rule; no LLM judgement.
- [x] ac-12 (code): backwards-compat backfill in implement/SKILL.md §4b(1) — compute hash, insert line, log 'Backfilled Spec hash for existing progress.md', do NOT emit drift; sequenced BEFORE gate enforcement so log ordering is deterministic.

---

## Notes

All 12 ACs and all 16 constraints addressed. Three-layer implementation:
1. **State (progress.md)**: template with Spec hash, Current AC, ## Detours, gate annotation
2. **Surface (session-context.sh)**: drift check + next-AC notifier with gate semantics
3. **Behaviour (implement/SKILL.md)**: pre-AC sequence (backfill → drift → gate), detour lifecycle, re-anchor

Fixture verification performed at /tmp/mrl-fixture:
- Matching hash → no drift notice ✓
- Modified spec → drift notice emitted ✓
- Missing Spec hash → silent skip ✓
- Gate at ac-01 unchecked → "ac-01 blocking gate + ac-02 startable after" ✓
- All ACs checked → "No unchecked ACs remain" ✓
- Detour with `[x] ac-02` → parser ignores; ac-01 still reported as next ✓

Verification deliverables not shipped as executable tests in this iteration — the ACs' `mrl_ac*` fixture assertions describe the intended test shape; they land as executable suites when the orbit plugin grows a test harness. Integration smoke tests above demonstrate the behavior holds.
