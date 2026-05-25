# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-port-setup-method-sh
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (regex semantics, cross-system CLI/MCP parity, shim deletion ripple) | 1 |
| 3 — Adversarial | not triggered | — |

## v1-cycle disposition (informational)

All five v1 findings are addressed in the current spec text:

- **AC-03 rollback over-claim (MED, addressed):** AC-03 now reads "Each file write uses `write_atomic` (per-file atomic; multi-file rollback is NOT promised, matching shim parity — partial-failure recovery is the operator's job, same as the shim today)". Prose matches the primitive.
- **AC-06 STYLE.md drift coverage (MED, addressed):** AC-06 now enumerates "(f) drift-overwrite on STYLE.md" and "(g) drift-keep on STYLE.md" — seven scenarios total, symmetric across both canonicals.
- **AC-03 whitespace normalisation (LOW, addressed):** AC-03 now names "collapses 3-plus consecutive blank lines back to 2 (shim line 211)" as part of the migrate sequence.
- **AC-07/08 incidental shim refs (LOW, resolved by reality):** `grep -rn setup-method plugins/orb/scripts/tests/` returns lines only in `test-setup-method.sh`. No incidental refs exist in the other three test files. The finding was a precaution against a class of breakage that does not exist for this shim. AC-08 stands as written.
- **AC-06 idempotency assertion (LOW, addressed):** AC-06 now ends "actual on-disk byte-equality is the load-bearing assertion" — disambiguated.

Gate-AC deterministic check (ac-01, ac-06, ac-07): all three descriptions are non-empty, not placeholder tokens, and well above the 20-character minimum. PASS.

## Findings

### [MEDIUM] AC-03 prescribes a regex pattern that the named crate cannot compile
**Category:** assumption
**Pass:** 1
**Description:** AC-03 says the verb strips legacy markers "using `regex` crate, pattern `(^|\n)<marker>\s*\n.*?(?=\n##\s|\n#\s|\z)` per shim lines 199-209". The workspace's regex dependency is `regex = "1.10"` (verified at `orbit-state/Cargo.toml:46`), which is Rust's standard RE2-derived `regex` crate. That crate does NOT support lookaround assertions of any kind, including the `(?=...)` positive lookahead the AC prescribes. The pattern as written will fail at `Regex::new` with `error: look-around, including look-ahead and look-behind, is not supported`. The Python shim uses `re` which does support lookahead, so the pattern is valid in the shim's source — it is not portable to the Rust crate the AC names.
**Evidence:** `orbit-state/Cargo.toml:46` declares `regex = "1.10"`; the `regex` crate's public docs (`https://docs.rs/regex/1.10/regex/`) explicitly list lookaround as unsupported. Shim lines 199-205 use Python `re` with `re.DOTALL`, where `(?=...)` is supported.
**Recommendation:** Pick one of three resolutions and pin it in AC-03:
- **(a) Reshape the pattern to no-lookahead** — match `(^|\n)<marker>\s*\n` to locate the start, then iterate forward line-by-line to find the next `\n##\s` / `\n#\s` / EOF and slice. Two-pass but no extra dep. **Pre-recommendation.**
- **(b) Add `fancy-regex` as a workspace dep** — supports lookaround at a ~2x compile/parse cost. Smallest source-shape change to the heredoc port.
- **(c) Use `regex` with a capturing pattern** — `(?ms)(^|\n)<marker>\s*\n(.*?)(\n##\s|\n#\s|\z)` (captures the next-heading marker as group 3), then re-emit the captured trailing marker after the strip. Same crate, no lookaround, slightly more boilerplate per marker.
Either way, the AC's "per shim lines 199-209" framing should make clear the **behaviour** is preserved, not the literal pattern syntax. Under (a) or (c) the byte-output equivalence is still testable via the AC-06 legacy-migrate fixture — that fixture will catch regressions regardless of which crate-compatible variant is chosen.

### [LOW] AC-03 and AC-05 @-import append paths use different shim heuristics, but the spec doesn't pin convergence
**Category:** missing-requirement
**Pass:** 2
**Description:** Two shim paths append @-imports: the bash `ensure_at_import` (lines 148-161, used by the non-legacy branch) and the Python heredoc (lines 213-220, used by the legacy-migrate branch). They differ in shape:
- bash: ensures trailing `\n`, then `printf '\n%s\n' "$import_line"` → guaranteed `\n\n<import>\n` separation.
- python: if not `text.endswith('\n')` add `\n`; if not `text.endswith('\n\n')` add another `\n`; then `text += import_line + '\n'` → also guaranteed `\n\n<import>\n` separation.
The two paths converge for typical inputs — same final-byte shape. But AC-05 cites only the bash-path rule (shim lines 154-159), and AC-03 says "adds @-imports — same sequence as the shim" without naming which sub-path. A reasonable implementer could collapse to one helper or implement two; the spec is silent. Risk is low (both heuristics produce identical bytes on typical inputs); the AC-06 legacy-migrate fixture catches divergence if it occurs.
**Evidence:** Shim lines 148-161 (bash `ensure_at_import`), lines 213-220 (Python legacy-migrate). AC-03, AC-05 in spec.yaml.
**Recommendation:** Add one sentence to AC-03 or AC-05 stating "the legacy-migrate path's @-import append and the non-legacy path's @-import append converge on identical output for identical CLAUDE.md inputs — a shared helper is acceptable". This is a polish nit; an attentive implementer handles it inline and the AC-06 fixtures backstop it.

---

## Honest Assessment

The spec is nearly ready. The v1 cycle's substantive findings are all properly addressed — AC-03's rollback claim now matches the primitive, AC-06 covers both canonicals symmetrically with on-disk byte-equality, AC-03's whitespace step is named, and AC-06's idempotency assertion is unambiguous. The single MEDIUM finding here is mechanical: AC-03 names a regex pattern the workspace's `regex = "1.10"` crate cannot compile because it uses positive lookahead. This is a real blocker that would surface as a compile-time `Regex::new` failure during implement — better caught now than five minutes into the implementation pass. The fix is small: reshape the pattern to a no-lookahead variant (path (a) above) and let AC-06's legacy-migrate fixture continue to backstop byte-equivalence with the shim's output. The LOW finding is a convergence note that the implementer will probably handle inline. Once AC-03's regex prescription is reshaped, this spec is APPROVE-ready.
