# Spec Review

**Date:** 2026-05-16
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-16-ac-taxonomy
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 0 |
| 2 — Assumption & failure | content signals (schema migration, deployment, cross-system reconcile, brownfield boundaries) — confirming cycle-1 fixes landed | 0 |
| 3 — Adversarial | not triggered | — |

Gate-AC description rule (Pass 1 step 5): seven gates (ac-01, ac-02, ac-03, ac-04, ac-05, ac-11, ac-15) all carry non-empty, non-placeholder, multi-hundred-char descriptions. Deterministic check passes.

## Cycle-1 finding closure

Verified each cycle-1 finding against the cycle-2 spec text and the live substrate.

### [HIGH] ac-05 rule-shape gap — RESOLVED

The new ac-05 is the rule-shape extension explicitly. It adds `Disposition::Transform(TransformFn)` with `Replace { value: serde_yaml::Value, sibling_writes: Vec<(&'static str, serde_yaml::Value)> } | Quarantine(String)` returns, a `"transform"` action_str arm, and an optional `transform_detail: Option<String>` on `DispositionRecord` for per-AC routing rationale. Regression coverage names the existing `Map / Drop / Quarantine` callers and the seeded entries at `reconcile.rs:111-130`. ac-06 then references ac-05's variant and replaces the existing `(EntityType::Spec, "acceptance_criteria[].ac_type", Disposition::Drop)` at `reconcile.rs:121-125` with `Disposition::Transform(reconcile_ac_type)`. The 2026-05-12-reconcile-mode ac-02 "until a second project demands" trigger is cited verbatim. Implementer can now extend the enum first, then wire the registry — clean two-AC sequence.

### [MEDIUM] ac-04 three-target scope — RESOLVED

ac-04 now enumerates all three live target files: (1) `2026-05-13-spec-close-ac-preflight/spec.yaml` ac-09 (closed); (2) `2026-05-16-memos-own-folder/spec.yaml` ac-12 (closed); (3) this spec's own ac-13 + ac-14 (open at migration time). Closed-spec rewrite is explicitly in-scope with the 2026-04-20 precedent cited for the "migration commit IS the audit record" framing. The post-migration grep tightens to `^  time_gated:` to match field-level instances only. Verification additionally names per-target `orbit spec show` assertions. Confirmed by `grep -rEn "time_gated" .orbit/specs/ | grep -v archive` — the three files (plus the cycle-1 review citation and notes/jsonl which migration leaves alone) are the only matches.

### [MEDIUM] deleted-memory citations — RESOLVED

`grep -n "orb-release-skill-missing-tag-push\|feedback_brownfield_visibility" .orbit/specs/2026-05-16-ac-taxonomy/spec.yaml` returns nothing. ac-13 (release) carries the tag-push step inline (`git tag v<new_version> && git push origin v<new_version>`) without a memory cross-reference. ac-14 (post-release smoke) inlines the visibility constraint as "Working-with-Hugh visibility discipline in CLAUDE.md" — and CLAUDE.md does carry that discipline verbatim at the top of the user-global file. Both signals are now load-bearing in the spec text rather than dangling cross-refs.

### [MEDIUM] envelope rename consumer — RESOLVED

ac-02's call-site list now explicitly enumerates `plugins/orb/skills/drive/SKILL.md:489-492` alongside the CLI renderer, MCP handler, and parity tests; the description ends with "Any other SKILL.md or doc that names `time_gated_open` is updated in the same commit." The verification line tightens to `grep -rEn "time_gated_open|time_gated" orbit-state/ plugins/` returns zero hits — broader than cycle-1's `time_gated_open`-only assertion. Confirmed by `grep -n "time_gated\|time_gated_open" plugins/orb/skills/drive/SKILL.md` returning exactly the two lines (489, 492) that ac-02 names. ac-09 carries a clean disambiguation note that the rename belongs to ac-02, not ac-09.

### [MEDIUM] brownfield dry-run sequencing — RESOLVED

The new ac-12 is the pre-release local-build dry-run (`cargo build --release` → `target/release/orbit canonicalise --reconcile --dry-run` against arcform and finetype). ac-13 is the release. ac-14 is the post-release brewed-binary smoke against the same two repos. The split is named explicitly: ac-12 "catches registry-coverage gaps BEFORE the brewed binary ships — running ac-13's release with an under-spec'd registry would require another release cycle"; ac-14 is "the post-merge confirmation that the brew tap update flowed through". Verification artefacts differ appropriately (local-build paths not retained in release artefact for ac-12; beelink terminal output captured for ac-14). The sequencing closes the cycle-1 chicken-and-egg cleanly.

## Findings (cycle 2)

No new findings. The two cycle-1 LOWs (ac-07 mode-switch interaction, ac-03/ac-06 migration-vs-reconcile boundary) remain unaddressed per the author's request to scope cycle-2 to HIGH+MEDIUM only. They are noted in the cycle-1 review and remain reachable as small follow-up edits during implementation if they bite.

## Pass 1 structural scan (cycle 2)

- **AC testability**: every AC names specific verification (cargo test, grep, exact file paths and line numbers, exact `orbit` invocations). No vague criteria. Pass.
- **Constraint conflicts**: none. The `time_gated` field removal (ac-01) + auto-migration (ac-03) + corpus rewrite (ac-04) form a coherent migration unit. The Transform variant addition (ac-05) is purely additive — existing `Map/Drop/Quarantine` callers continue to work. Pass.
- **Scope vs goal**: goal names seven wires (schema, spec.close, migration, corpus migrate, design, review-pr, drive, reconcile registry, time_gated removal). 15 ACs map cleanly: ac-01 schema, ac-02 spec.close, ac-03 migration runner, ac-04 corpus migrate, ac-05 Transform variant, ac-06 reconcile registry, ac-07 design, ac-08 review-pr, ac-09 drive, ac-10 docs, ac-11 test gate, ac-12 pre-release smoke, ac-13 release, ac-14 post-release smoke, ac-15 meta-gate. No over-spec or under-spec. Pass.
- **Obvious gaps**: error handling named for partial-failure migration (per-step persistence); rollback implicit via the schema-version file gating; monitoring captured via `orbit audit drift` and `orbit verify` in ac-11. Pass.
- **Gate-AC description rule**: 7 gates, all non-empty, all non-placeholder, all >>20 chars. Pass.
- **Content signal scan**: schema migration, deployment (brew tap), cross-system reconcile, brownfield boundaries — content signals present and addressed by the spec text. Pass 2 triggered for confirmation only.

## Pass 2 structural confirmation (cycle 2)

The cycle-2 spec preserves the assumption surface from cycle-1 with the additions analysed under cycle-1 finding closure above. Specifically:

- **Field-rename completeness**: ac-02's broader grep assertion (`time_gated_open|time_gated` across `orbit-state/` and `plugins/`) catches any forgotten SKILL.md / doc reference. The 2026-05-13 closed spec's `notes.jsonl` and cycle-1 review markdown reference `time_gated` but are not in-scope for rewrite — `.orbit/specs/**/spec.yaml` (the migration walker's path) does not include `notes.jsonl` or review files.
- **Transform-variant fixture coverage**: ac-05 verification covers (a)/(b)/(c)/(d)/(e) — the five disposition shapes the new variant supports. ac-06 verification covers each of the six routing branches with synthetic AC fixtures. Together they pin the Transform-variant behaviour at both the enum-shape level (ac-05) and the registry-rule level (ac-06).
- **Migration ordering**: ac-01 (schema change), ac-03 (migration runner), and ac-04 (corpus migrate) ship in the same commit per ac-04's framing — this is the right sequencing because the runner cannot exist without the new field, and the corpus cannot migrate without the runner. Same-commit shipping avoids any intermediate-state landing on main.
- **Card 0032 ac-03 reference**: ac-06's "per card 0032 ac-03" citation resolves — card 0032 scenario 3 (mapping-known-fields-migrate-cleanly) is the right anchor for the run-summary-records-disposition contract.

No structural concerns. Pass 3 not triggered.

---

## Honest Assessment

The cycle-1 verdict's five substantive concerns are all resolved with clean, named edits. The biggest change — splitting cycle-1's single ac-05 into ac-05 (rule shape) + ac-06 (registry mapping) — is the right shape: the rule-shape decision is now first-class rather than buried inside a registry description, and the registry mapping can reference the variant explicitly. ac-12's pre-release dry-run against locally-built binary is a tidy fix for the cycle-1 sequencing concern and adds a registry-coverage smoke before the brewed binary ships. The ac-04 enumeration of three target files (with closed-spec rewrite scoped explicitly) makes the migration test fixture concrete.

The renumber from 13 ACs to 15 ACs propagates correctly: ac-15's meta-gate cites ac-01, ac-02, ac-05, ac-06, ac-07, ac-08, ac-09, ac-10 — all under the new numbering. Cross-references within other ACs (ac-02 → ac-09 note, ac-03 → ac-06, ac-06 → ac-05, ac-09 → ac-02 note, ac-12 → ac-13, ac-14 → ac-12, ac-13 → ac-13) all resolve. Gate count matches the brief (7) and time_gated count matches the brief (2).

The two cycle-1 LOWs remain as the only outstanding concerns and are explicitly out of scope for this cycle per the author's brief. Neither is implementation-blocking — the ac-07 mode-switch interaction is mitigated by ac-07's existing language ("implementation-question filter is not relaxed by this addition"), and the ac-03/ac-06 boundary question is now implicit-clear (auto-migration handles `time_gated → ac_type`, reconcile handles brownfield `ac_type` value normalisation, and ac-03's description does state this distinction at "brownfield `ac_type` VALUE normalisation is a separate path through `canonicalise --reconcile` (ac-06)").

APPROVE — the plan is implementation-ready. Drive can take this through implement with no further design loop.
