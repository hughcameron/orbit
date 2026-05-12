# Spec Review

**Date:** 2026-05-11
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-12-tree-views
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (cross-system boundaries: CLI + MCP + session-prime + multiple SKILL.md files; schema-as-const-via-Rust-struct) | 2 |
| 3 — Adversarial | not triggered — Pass 2 issues are local, no cross-AC cascades | — |

## Findings

### [LOW] AC-03 "most-connected card" when every card has degree zero
**Category:** test-gap
**Pass:** 2
**Description:** AC-03 names the edge set (outgoing `relations:` + incoming `relations:`; `specs:` entries excluded) and the tie-break (lowest numeric id). What it doesn't say is what `overview` prints when every card has degree zero — fresh project, brownfield import before relations are authored, or any state where no card has been wired yet. Tie-break-by-lowest-id resolves it (everyone tied at zero, so card 0001 wins), but the AC reads as if "most-connected" implies at least one edge somewhere. Two correct implementations could diverge here — one prints the lowest-id card, another prints "none" or omits the line. Cheap to pin; cheaper than discovering it at review-pr time.
**Evidence:** AC-03 description (spec.yaml:18-21). The tie-break clause "ties resolved by lowest numeric card id" implicitly resolves a zero-degree-everywhere graph the same way as any other tie, but the AC doesn't name the case.
**Recommendation:** Add one clause to AC-03's most-connected definition: "if every card has degree zero, the lowest-numeric-id card wins (consistent with the general tie-break)". Or, alternatively: "if every card has degree zero, the line is omitted". Either is fine; pick one. Not blocking — the implementor can resolve this at implement time and the parity test will pin it — but it's the kind of edge case `overview` will hit on a freshly-seeded project and is cheaper to nail in the AC.

### [LOW] AC-08 "where applicable" hedge on parity coverage
**Category:** test-gap
**Pass:** 2
**Description:** AC-08 requires "One parity-tested case per path per verb where applicable". For verbs that take an id (`card tree`, `card specs`), the three paths (unknown id, broken referenced file, malformed YAML) all apply. For verbs that don't take an id (`overview`, `graph` without `--card`, `audit drift`), "unknown id lookup" doesn't apply, and the AC's "where applicable" silently exempts those. That's the right call, but the AC doesn't enumerate which verb-path combinations are exempted, so the implementor's interpretation and the reviewer's interpretation could differ. The hedge is honest but admits two reasonable readings of "parity coverage complete".
**Evidence:** AC-08 description (spec.yaml:38-41). AC-01 and AC-02 take an `<id>` argument; AC-03 and AC-05 do not; AC-04's `--card` is optional. The "unknown id" error path is meaningful for the first set and meaningless for the second.
**Recommendation:** Tighten AC-08 to enumerate the exemptions, e.g. "the three error paths apply to all id-taking verbs (`card tree`, `card specs`, and `graph --card`); `overview` and `audit drift` cover paths (b) and (c) only". This is implement-time-resolvable, so REQUEST_CHANGES is not warranted, but the implement skill should resolve it in the design notes rather than leave it to review-pr judgement.

---

## Gate-AC description check

Five gate ACs (ac-01, ac-03, ac-06, ac-07, ac-08) — all pass the deterministic structural rules: non-empty, no placeholder tokens (TBD/TODO/FIXME/PLACEHOLDER/XXX/???), all well above the 20-character minimum (shortest is ac-01 at 184 chars). No deterministic findings from this rule.

---

## Honest Assessment

v3 closes the three v2 findings cleanly. AC-03's bounded-synthesis cap is now pinned (`K=10` spec ids with `+N more` suffix), AC-04 explicitly accepts that the unscoped graph render is the share-or-paste path and trades single-screen for it, and AC-05 names the source-of-truth mechanism (a hand-maintained `const FIELDS: &[&str]` per struct, with a drift-detection unit test asserting the constant tracks serde's view of the struct). The schema.rs struct paths in AC-05 are real; the parity test paths in AC-06 are real; card 0032's "map / drop / quarantine" vocabulary in AC-05 is correctly cited. What remains is two LOWs — both edge-case sharpenings on verbs whose happy paths are well-specified, both implement-time-resolvable without rework. Neither admits a wrong implementation that would silently pass the AC and explode later; both are about which-of-two-correct-readings the implementor picks. The spec is implement-ready. APPROVE.
