# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-port-setup-method-sh
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | content signals (cross-system CLI/MCP parity, shim deletion ripple) + Pass 1 gaps | 2 |
| 3 — Adversarial | not triggered | — |

## Findings

### [MEDIUM] AC-03's rollback claim outruns the named primitive
**Category:** assumption
**Pass:** 1
**Description:** AC-03 says "If the verb errors mid-batch, it rolls back any disk writes from this invocation (atomic write-or-skip per file using `write_atomic`)". `write_atomic` provides per-file atomicity via the temp-file-then-rename pattern — not multi-file transactional rollback. If METHOD.md is written successfully and then STYLE.md write fails (disk full, permission, panic), the prior METHOD.md swap is already on disk and is not undone. The same gap exists in the shim (`cp` is not transactional either), so the Rust port is not strictly worse, but the AC's prose promises rollback the primitive cannot deliver.
**Evidence:** Spec line 18 (AC-03 description); `orbit-state/crates/core/src/atomic.rs` (write_atomic semantics — single file scope).
**Recommendation:** Either (a) downgrade the prose to "atomic per-file write via `write_atomic`; multi-file rollback is best-effort and matches the shim's prior behaviour" — making it clear the verb does NOT promise multi-file transactional rollback — or (b) name a real staging-then-promote strategy (e.g., stage all targets in a `.orbit/.setup-staging/` directory, then atomic-rename in a single second pass) and add a mid-batch-failure test to AC-06 that proves the rollback claim. (a) is the cheap path and aligns with shim parity.

### [MEDIUM] AC-06 fixture coverage skips STYLE.md drift branches
**Category:** test-gap
**Pass:** 2
**Description:** AC-06 enumerates "(d) drift-overwrite on METHOD.md → file replaced, (e) drift-keep on METHOD.md → file unchanged". Both drift cases exercise METHOD.md only. The shim runs `copy_canonical` twice — once per canonical — with independent drift decisions. A regression where STYLE.md's drift path misfires (e.g., wrong arg threaded, copy-paste bug from the METHOD.md branch) would not be caught.
**Evidence:** Spec line 30 (AC-06 listed scenarios); shim lines 237-238 — `copy_canonical` called twice with different operands.
**Recommendation:** Add "(f) drift-overwrite on STYLE.md", "(g) drift-keep on STYLE.md" — or rephrase (d) and (e) as parameterised across both canonicals. The "at least five scenarios" framing accommodates either shape.

### [LOW] Whitespace-normalisation steps in the shim are not pinned by ACs
**Category:** missing-requirement
**Pass:** 1
**Description:** Two shim behaviours affect output bytes but are unmentioned in the spec:
1. The Python legacy-migration path collapses 3+ consecutive blank lines to 2 (shim line 211 — `re.sub(r'\n{3,}', '\n\n', text)`). The Rust port may or may not preserve this; AC-03 does not name it.
2. The Python legacy-migration path's @-import append guarantees a blank line before each import (`text.endswith('\n\n')`), while the non-legacy `ensure_at_import` bash path produces single-newline separation. AC-05 cites the bash-path semantics (lines 154-159) but the legacy-migrate batch path (AC-03) uses the Python rule. Either the two paths converge in the port or both rules are preserved — the spec is silent.
**Evidence:** Shim lines 154-159, 191-224 (Python heredoc); AC-03, AC-05.
**Recommendation:** Either add a sentence to AC-03 saying "whitespace normalisation preserves the shim's behaviour: collapse 3+ blank lines to 2, and ensure a blank line precedes appended @-imports", or explicitly say "the port may diverge in stacked-blank-line normalisation as long as AC-05's exact-line-presence invariant holds". Pick one. If fixtures in AC-06 include CLAUDE.md inputs with stacked blanks, byte-equal tests will catch divergence either way — but the spec should pin the intent.

### [LOW] Tabletop-deferred check on incidental shim refs in other shell tests is not lifted into an AC
**Category:** missing-requirement
**Pass:** 2
**Description:** Tabletop note line 24 flags "the other three [shell tests] may have incidental refs — check during implement". The spec did not lift this into an AC. AC-08 only names the dedicated test for deletion. If incidental refs exist in other tests that survive the deletion, those tests will break silently on the shim's removal.
**Evidence:** Spec ACs 07-08 cover only SKILL.md grep + dedicated test deletion; tabletop-note.md line 24.
**Recommendation:** Add a one-line clause to AC-07 or AC-08: "Pre-deletion grep `grep -rl setup-method.sh plugins/orb/scripts/tests/` returns only `test-setup-method.sh`; if other tests reference the shim incidentally, they are updated or flagged as separate hygiene work in the same commit." Cheap insurance against silent test breakage.

### [LOW] AC-06 idempotency assertion is underspecified
**Category:** test-gap
**Pass:** 2
**Description:** AC-06 ends with "Plus an idempotency test (run twice, assert same final state)". "Final state" is ambiguous between (a) filesystem byte-equal across run-1 and run-2, and (b) envelope from run-2 reports no-op rather than re-create. Both are useful invariants; the AC names neither explicitly.
**Evidence:** Spec line 30.
**Recommendation:** Tighten to "the second run's envelope reports no disk writes (or the same `Ok` envelope with no `migrated`/`overwritten` flags set) AND the filesystem byte-state matches run-1's post-state". The reviewer notes this is minor — implementer judgement covers it adequately.

---

## Honest Assessment

The spec is mostly ready and inherits a clean precedent shape from the two prior choice-0020 ports (acceptance shim, promote shim). The mechanics — verb + CLI flags + parity tests + same-commit shim deletion + choice-0020 table update + CHANGELOG — are well-rehearsed and concrete. The single substantive issue is AC-03's rollback claim, which over-promises against `write_atomic`'s actual single-file scope; this is best fixed by softening the prose to match shim parity rather than building a staging directory. The STYLE.md drift test-gap is the second material item — easy to add, prevents a real regression class. The remaining three findings are LOW polish items that an attentive implementer would handle inline but worth pinning explicitly. Biggest residual risk: silent test breakage from incidental shim refs in non-dedicated shell tests post-deletion — covered by the AC-08 tweak in finding 4.
