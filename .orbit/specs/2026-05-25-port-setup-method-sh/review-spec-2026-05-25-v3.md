# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-port-setup-method-sh
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | not triggered | — |
| 3 — Adversarial | not triggered | — |

## v2-cycle disposition (informational)

Both v2 findings are addressed in the current spec text:

- **AC-03 regex prescription (MED, addressed):** AC-03 now reads "Implementation is behaviour-driven (NOT pattern-prescribed): the workspace ships `regex = "1.10"` which is RE2-derived and rejects lookahead, so the marker-to-next-heading scan uses a line-walking loop OR a fixed-pattern split, implementer's choice. AC-06's legacy-migrate fixture (b) is the byte-equivalence backstop." This is the clean reshape — the AC pins the **behaviour** (legacy markers stripped to next top-level heading or EOF) and leaves the crate-compatible implementation choice to the implementer, with the byte-equal fixture in AC-06 as the regression backstop. The lookahead-incompatibility blocker is gone.
- **AC-03/AC-05 @-import path convergence (LOW, accepted-as-is):** v2 noted that the shim's bash `ensure_at_import` (lines 148-161) and the Python heredoc's append (lines 213-220) produce identical bytes for typical inputs but the spec doesn't pin convergence. The current spec text still doesn't add a one-line convergence note; this reviewer agrees with v2's own assessment that "an attentive implementer handles it inline and the AC-06 fixtures backstop it" — the LOW finding is a polish nit, not a structural defect. AC-06's seven-fixture matrix (with idempotency + filesystem byte-equality) is sufficient to catch any divergence. No action required.

Gate-AC deterministic check (ac-01, ac-06, ac-07): all three descriptions are present, non-placeholder, and well above the 20-character minimum (smallest is ac-07 at ~280 chars; ac-01 and ac-06 are several hundred chars longer). PASS.

Pre-flight assertion check: `rg --no-heading 'setup-method\.sh' plugins/orb/skills/` returns one line (`plugins/orb/skills/setup/SKILL.md:246`). AC-07's "pre-flight count: 1 SKILL.md call site (setup)" matches reality.

Content-signal scan: spec touches cross-system CLI/MCP boundaries (parity tests), shim deletion ripple (call-site grep), and choice-substrate edges. All three signals are explicitly addressed by ACs (06 for parity, 07-08 for deletion, 09-10 for substrate edges). No untouched signal classes — Pass 2 not triggered.

## Findings

None.

---

## Honest Assessment

The spec is ready. v1's five findings were addressed by the v2 cycle; v2's single MEDIUM (the regex-lookahead blocker) is addressed cleanly in this cycle by reshaping AC-03 from pattern-prescriptive to behaviour-driven, with AC-06's legacy-migrate fixture (b) as the load-bearing regression backstop. v2's LOW convergence note is correctly accepted as a polish nit that the AC-06 fixtures cover. The spec inherits the well-rehearsed shape of the two prior choice-0020 ports (acceptance shim PR #32 / 0.4.34, promote shim PR #33 / 0.4.35) and bundles card 0017's deferred relations-edge cleanly. Biggest residual risk: the implementer choosing the line-walking variant must produce byte-equivalent output to the shim's `re.DOTALL`-based stripper across the AC-06 legacy-migrate fixture — but that's exactly what the fixture is for, and a one-shot test failure during implement surfaces it without spec rework. Ship it.
