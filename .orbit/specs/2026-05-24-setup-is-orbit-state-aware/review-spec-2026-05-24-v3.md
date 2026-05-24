# Spec Review

**Date:** 2026-05-24
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-24-setup-is-orbit-state-aware
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (schema change on `OrbitConfig`, cross-drive boundary with `2026-05-24-workflow-conformance`, history-preserving migration via `git mv`, shared library code in orbit-state) | 1 (LOW) |
| 3 — Adversarial | not triggered | — |

The v3 spec absorbs every remaining finding from the v2 review. AC count grew from 18 → 21:

- v2's MEDIUM #1 (no AC for `mixed-bare` / `mixed-undotted` runtime refusal) → closed by **ac-19** (refusal contract, `git status --porcelain` no-mutation check, exit non-zero, gate).
- v2's MEDIUM #2 (cross-drive suppression assumption only wired through ac-11) → closed by **ac-20** (post-condition AC pinning `canonical-files-missing` non-firing on a `wrapped-undotted` repo, with an integration-test fixture verification, gate).
- v2's MEDIUM #3 (no AC pinning `brownfield-bare` preservation after rename) → closed by **ac-21** (preserves the §3 block at SKILL.md:48-98 structurally, doc AC).
- v2's LOW (idempotency on the `plugin_repo` field) → folded into **ac-07** ("if `.orbit/config.yaml` already contains the `plugin_repo` true setting, setup does not modify the file").

Deterministic gate-AC description check: all 9 gates (ac-01, ac-06, ac-10, ac-11, ac-12, ac-14, ac-15, ac-19, ac-20) have non-empty, non-placeholder, ≥20-character descriptions. Pass.

## Findings

### [LOW] ac-04 byte-for-byte METHOD.md compare is sensitive to line-ending normalisation

**Category:** failure-mode
**Pass:** 2
**Description:** ac-04 requires "byte-for-byte" comparison between the canonical METHOD.md shipped by the plugin and the project's copy. If either copy gets touched by tooling that normalises line endings (a Windows checkout, a stray editor save with CRLF, a tool that strips trailing whitespace), the byte-compare reports drift on a semantically-identical file and prompts the operator to overwrite their own unchanged copy. This is low risk on the canonical Linux-only target environment but worth a sentinel.

**Evidence:** spec.yaml ac-04 names "compares byte-for-byte (including the top-of-file 'How to update' line)"; no normalisation clause. The plugin's source METHOD.md presumably ships with LF endings; any project-side mutation that introduces CRLF will trip the compare.

**Recommendation:** Optional polish only. Either (a) name the normalisation explicitly ("byte-for-byte after LF normalisation"), or (b) leave as-is and accept that operators on non-Linux environments may see false-positive prompts. Not blocking — the prompt-before-overwrite gate means a false-positive is recoverable.

---

## Honest Assessment

The spec is implementation-ready. The three MEDIUM findings carried forward from v2 are all closed cleanly: ac-19 (mixed-state refusal), ac-20 (cross-drive suppression post-condition), ac-21 (brownfield-bare preservation) — each with a named verification mechanism (integration test fixtures, `git status --porcelain` equality, diff inspection on the §3 block). The v2 LOW on config-file idempotency is absorbed directly into ac-07.

The cross-drive dependency on `2026-05-24-workflow-conformance` is now substrate, not tribal knowledge — ac-11 names the shared classifier helper at `orbit-state/crates/core/src/verbs.rs` with both call sites, and ac-20 names the sister drive's `undotted_substrate` suppression at `audit_conformance_at`. The dependency surface is bidirectional and explicit; the only residual coupling is drive-order (this drive's ac-20 integration test fails until the sister drive lands its suppression). That's a sequencing concern for the rally lead, not a spec-quality concern.

The biggest residual risk is the schema change to `OrbitConfig`: adding the `plugin_repo` field needs to land cleanly across existing configs that don't have the field. ac-12 names it as "optional ... defaulting to false", which is adequate AC wording for the backward-compat contract — implementers familiar with serde defaults will reach for `#[serde(default)]` reflexively. If this turns out to be missed at implement-time, the symptom is loud (existing `.orbit/config.yaml` files fail to parse), so the failure mode is detectable.

The one LOW finding above is optional polish. Approve.
