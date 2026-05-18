# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-17-code-investigate-skill
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (cross-session PreToolUse hook, plugin-loader behaviour, substrate marker file, cross-skill prose edits, 4-week observation audit) | 1 |
| 3 — Adversarial | not triggered | — |

## Cycle-2 changes acknowledged

The v2 review (REQUEST_CHANGES) raised one HIGH and one MEDIUM. Both are resolved in the spec as it stands today:

- **v2 HIGH (ac-02 load-bearing plugin-loader claim).** The cycle-1 author's commit message asserts the claim; the cycle-2 substrate update went further and filed memory `plugin-shipped-hooks-supported` (2026-05-18) documenting that the docs check resolved the question — plugin-shipped hooks at `plugins/<name>/hooks/hooks.json` are registered automatically. ac-02's verification clause now reads as v2 Option 2 (strengthen verification with a load-time smoke counting registered hooks) plus v2 Option 3 (explicit dual-registration fallback in `.claude/settings.json` if the loader reports 0). Belief converted to test; contingency named.
- **v2 MEDIUM (`*.toml` filter excludes Cargo.toml).** ac-02's path filter is now the strict "machine-managed" set v2 recommended — only `.orbit/`, `.claude/`, and `*.lock`. The spec adds an explicit clause naming source-adjacent config (Cargo.toml, pyproject.toml, package.json, nested yaml/json) as paths that *do* trigger the warning when uninvestigated. Tighter than v2 asked for, and the rationale is on the AC.

The remaining fresh findings are minor — one structural gap that the implementing agent can resolve cleanly, and one downstream-audit data-source ambiguity that does not block close.

## Findings

### [LOW] ac-03 doesn't pin which entry-kind each mode writes
**Category:** test-gap
**Pass:** 1
**Description:** ac-03 specifies the marker file's entry shape as `<unix-timestamp>\t<kind>\t<path>` with `kind ∈ {file, scope}`, and the hook in ac-02 considers an Edit investigated if the file is a literal `file` entry or sits under a `scope` prefix. What the spec does not pin is which kind each invocation mode writes. The natural mapping — narrow mode writes `file` entries from the matched paths, broad mode writes a `scope` entry from the directory/module passed — is inferable but not stated. An implementing agent could reasonably ship narrow mode writing the query string as a `scope` entry (which would never match an Edit path), or broad mode writing nothing at all when no scope is supplied (which would leave the marker file empty and the hook firing on every subsequent Edit).
**Evidence:** ac-03 description names the two kinds but does not link them to ac-01's two modes; the skill prose described in ac-01 is the only other surface where the mapping could land, and ac-01 doesn't mention the marker-write contract.
**Recommendation:** Add one sentence to ac-03 description: *"Narrow mode writes one `file` entry per matched path (resolved to repo-root-relative); broad mode writes one `scope` entry for the directory/module argument (or for the repo root when no scope is supplied)."* Or, equivalently, pin the mapping in ac-01's skill-prose contract. One-line tightening; doesn't change scope.

### [LOW] ac-07 doesn't pin the warning-fire data source
**Category:** test-gap
**Pass:** 2
**Description:** ac-07's audit memo includes "warning-fire count from the ac-02 hook (how often agents edited without prior investigation)" segmented by file kind. ac-02 specifies the warning text as grep-stable ("consider /orb:code-investigate before editing") but doesn't pin where the fires accrue durably enough for an audit 4 weeks later to count them. Claude Code hook stdout goes to tool output, which is captured in the session transcript but is not directly grep-able 4 weeks later across multiple sessions in multiple repos without additional plumbing. The audit may discover at week 4 that the count is `≥ 0` (i.e. "we know the warning fired sometimes but can't tabulate") rather than a clean integer.
**Evidence:** ac-02 description names the warning text but no log sink; ac-07 verification requires a count without pinning the source.
**Recommendation:** Either (a) ac-02 adds: *"On fire, the hook also appends a one-line record to `.orbit/.code-investigate-warnings` (timestamp, session-id, file path, file-kind) — same gitignore and atomic-write conventions as the marker"* — making the audit tractable via a single grep; or (b) ac-07 explicitly acknowledges the audit's warning-fire metric is approximate (transcript-derived) and lists what counts as a fire (e.g. "session transcripts grepped for the warning string"). Option (a) costs one more substrate file but makes the 4-week audit a true tabulation rather than a manual sweep. Observation-band, so this does not block close — but the implementing agent should not arrive at the audit point with no log to read.

---

## Honest Assessment

This is a ready spec. The intent contract was strong from v1; the cycle-1 response did the load-bearing work on ac-02 (registration target pinned to plugin-shipped, contingency named, prior memory's misread corrected via a fresh docs check) and the toml filter narrowing. The substrate seams (marker file, hook, session-id lifecycle, label convention, observation window) all hang together and reuse existing patterns (atomic-write convention, session-id rollover, memory labels). The two-mode skill is clearly described and the call-points in ac-04 sit at the right structural moments.

The remaining LOWs are tighten-up work the implementing agent can resolve in the natural course of writing the skill prose (ac-03 mode→kind mapping) and the hook script (ac-07 warning log sink). Neither is a substrate-shape decision; both are one-line additions to existing ACs. They do not justify another REQUEST_CHANGES cycle — surfacing them in the review record gives the implementing agent the same flag the reviewer noticed, without gating another response cycle.

Verdict APPROVE — "I couldn't find problems" in the spec-shape sense the contract names. The biggest residual risk is downstream of close (the 4-week audit's data quality if the warning sink isn't pinned), and it is observation-band by ac-taxonomy.
