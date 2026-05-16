# Spec Review

**Date:** 2026-05-16
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-16-ac-taxonomy
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 |
| 2 — Assumption & failure | content signals (deployment, brownfield/cross-system boundaries, schema migration); Pass 1 found stale-evidence and scope-claim issues | 5 |
| 3 — Adversarial | not triggered | — |

Gate-AC description rule (Pass 1 step 5): all six gates (ac-01, ac-02, ac-03, ac-04, ac-10, ac-13) carry non-empty, non-placeholder, ≥20-char descriptions. Deterministic check passes.

## Findings

### [HIGH] ac-05 prescribes registry rules that the current `Disposition` enum cannot express

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-05 says `canonicalise --reconcile` will map `ac_type: docs` → `ac_type: doc` (a value-level rewrite), and split `ac_type: gate` into `ac_type: code|observation` + `gate: true` based on a case-insensitive regex over the AC description text. The closed precursor spec `2026-05-12-reconcile-mode` ac-02 explicitly scoped the registry to field-name renames (`Map(new_name)`), field drops (`Drop`), and `Quarantine` — *"Value transforms (enum-value renaming, shape changes that move content between fields) are out of scope for v1; legacy fields that need value-level handling are classified `quarantine` per ac-04 and a follow-up spec teaches the registry the richer rule shape once a second project demands it."* The current `Disposition` enum at `orbit-state/crates/core/src/reconcile.rs` (`Map(rename) / Drop / Quarantine`) has no value-transform variant and no predicate machinery. ac-05 is the second-project trigger named in the precursor — but this spec has no AC that extends the enum or names the new rule shape. Implementer is left to either (a) silently expand reconcile-mode scope mid-implementation or (b) hit a wall and discover the gap during coding.
**Evidence:** `orbit-state/crates/core/src/reconcile.rs:46-49` declares the disposition variants. The lib module-docs at `reconcile.rs:1-40` confirm field-rename-only semantics. The 2026-05-12-reconcile-mode spec is closed and the v1 contract is fixed. ac-05 description names rewrite actions (value renames, conditional value+shape splits keyed on regex matches over an adjacent field's text) that fall outside `Map(new_name)`.
**Recommendation:** Add an explicit AC (preceding ac-05) that extends the reconcile registry with a richer rule shape — at minimum a `MapValue { from: &str, to: &str }` variant and a predicate-bearing `SplitOnDescriptionRegex { regex: &str, on_match: Disposition, on_miss: Disposition }` variant (or equivalent design choice). Cite the choice (likely a new MADR in `.orbit/choices/`) so the rule-shape decision is recorded. Alternatively, declare that brownfield `ac_type: gate` ACs are quarantined wholesale in v1 (matches the v1 contract) and follow-up specs add the split heuristic — this changes ac-11's expected planned-changes counts from "60 split" to "60 quarantined" against finetype.

### [MEDIUM] ac-04 understates the migration scope — three live `time_gated: true` ACs, not one

**Category:** test-gap
**Pass:** 1
**Description:** ac-04 claims "the single known live target is `.orbit/specs/2026-05-16-memos-own-folder/spec.yaml` ac-12 (the only `time_gated: true` AC in the corpus, per memory `session-close-2026-05-16-memos-own-folder-shipped`)" and asserts the post-migration check `grep -rln "time_gated" .orbit/specs/ | grep -v archive` returns nothing. In reality, `grep` against the live corpus surfaces three specs with `time_gated: true`:
  1. `.orbit/specs/2026-05-13-spec-close-ac-preflight/spec.yaml:54` (closed status; ac-09)
  2. `.orbit/specs/2026-05-16-memos-own-folder/spec.yaml:70` (ac-12, the named target)
  3. **This spec itself** — `.orbit/specs/2026-05-16-ac-taxonomy/spec.yaml:67, 73` (ac-11 and ac-12)
The migration in ac-03 walks every spec.yaml so all three get rewritten — functionally correct, but ac-04's test fixture and the "single target" framing risk steering the implementer toward a too-narrow assertion or commit-message scope. There is also the closed-spec edge: rewriting `2026-05-13-spec-close-ac-preflight` (status: closed) post-shipment touches a historical record. The spec doesn't declare whether closed specs are in-scope for the rewrite.
**Evidence:** `grep -rln "time_gated: true" .orbit/specs/ | grep -v archive` returns four file matches with `: true` literally present in spec.yaml files (one in the closed precursor, one in memos-own-folder, two in this spec).
**Recommendation:** Reword ac-04 to enumerate the three known live targets (closed spec included) and state explicitly that closed-spec content is in-scope for the rewrite (the migration commit becomes the audit trail for that, per ac-04's own framing). Update the post-migration grep assertion to match the actual cleanup set. If closed specs should NOT be rewritten, add an exclusion to the migration walker in ac-03 — but that contradicts the "every spec.yaml" claim and would leave stale `time_gated` references the schema no longer supports.

### [MEDIUM] ac-11 and ac-12 cite a memory the substrate deleted earlier today

**Category:** test-gap
**Pass:** 1
**Description:** ac-12 cites "memory `orb-release-skill-missing-tag-push`" as the rationale for the `git tag v<new_version> && git push origin v<new_version>` step. ac-11 cites "memory `feedback_brownfield_visibility.md`" for naming-discipline guidance. Earlier today (commit `eccc16a`, 2026-05-16) the substrate deleted `.orbit/memories/orb-release-skill-missing-tag-push.yaml` as "stale" — the commit message says the `/orb:release` SKILL.md has carried the tag-push step "for some time" and the memory was misleading. Citing a deleted-as-misleading memory in a new spec re-introduces the bad signal. `feedback_brownfield_visibility.md` does not exist in `.orbit/memories/` at all.
**Evidence:** `ls .orbit/memories/ | grep -i "release-skill-missing\|brownfield_visibility"` returns nothing. `git show eccc16a` documents the deletion as deliberate hygiene.
**Recommendation:** Drop the `orb-release-skill-missing-tag-push` citation in ac-12 — the `/orb:release` skill already carries the tag-push step, no memory cross-reference needed. Either locate the correct `feedback_brownfield_visibility` artefact (it may be in `.orbit/memos/` rather than `.orbit/memories/`, or it may be content the implementer was supposed to capture before this spec) or drop the citation and inline the constraint ("name only the two public brownfield repos; refer to private repos generically").

### [MEDIUM] ac-02 renames an envelope field; consumers in `plugins/orb/skills/drive/SKILL.md` will silently break

**Category:** failure-mode
**Pass:** 2
**Description:** ac-02 renames the response envelope field `time_gated_open` to `deferrable_open` across the codebase, naming "CLI renderer, MCP handler, parity tests" as the call sites to update. The SKILL.md for `/orb:drive` at `plugins/orb/skills/drive/SKILL.md:489-492` explicitly instructs the agent to read `time_gated_open` from the response envelope. ac-08 modifies the same SKILL.md for routing rules but doesn't call out removing or updating the existing `time_gated_open` reference. Without the SKILL.md edit, drive's deferred-checkpoint logic reads a field name that no longer exists; the agent gets an empty list and assumes nothing was deferred — silent.
**Evidence:** `grep -n "time_gated_open" plugins/orb/skills/drive/SKILL.md` returns line 492. ac-02's "call sites" list omits SKILL.md surfaces. ac-08's scope is "add routing rules" — it doesn't mention removing the legacy field-name reference.
**Recommendation:** Extend ac-02's call-site list to include `plugins/orb/skills/drive/SKILL.md` and any other SKILL.md that names the envelope field. Or fold the SKILL.md edit into ac-08 explicitly. Either route, the grep `grep -rn "time_gated_open" plugins/` must return nothing post-implementation, and a verification line should assert that.

### [MEDIUM] ac-11 (brownfield dry-run) runs after ac-12 (release) — failure-discovery comes too late

**Category:** failure-mode
**Pass:** 2
**Description:** ac-12 releases the new orbit version via `/orb:release`. ac-11 dry-runs `canonicalise --reconcile` against the public brownfield corpora at meridian-online/arcform and meridian-online/finetype using the released binary on the beelink. If the dry-run uncovers a pattern the reconcile registry doesn't cover (a fourth `ac_type` value, a `gate` description that matches neither regex but isn't quarantine-appropriate, or a registry rule that misfires), the binary is already published and the fix requires another release cycle. This is structurally analogous to the chicken-and-egg the precursor spec hit on its own dogfood AC. The card 0035 evidence base already names the patterns observed (60 finetype `ac_type: gate` ACs split by training/build heuristic), so the surprise surface is small — but small isn't zero.
**Evidence:** ac-11 description says "requires the released orbit binary on PATH against external repo trees" — by construction post-release. ac-12 sequences "wait for the release.yml workflow to publish the brew tap update". The two together cannot run pre-release.
**Recommendation:** Either (a) accept the post-release dry-run as observation-shaped follow-up and add a confidence note that the registry covers the evidence base; or (b) add an AC that runs the brownfield dry-run against a **local build** of the binary (cargo-built, not brew-installed) before release, and keep ac-11 as the post-release re-run smoke against the brewed binary. Option (b) lets the implementer catch a registry gap pre-release without changing the verification artefact. Either route, name the choice explicitly.

### [LOW] `ac_type` prompt in `/orb:design` §6 may collide with the implementation-question filter

**Category:** assumption
**Pass:** 2
**Description:** ac-06 adds an instruction in `/orb:design` §6 to prompt the author for `ac_type` per AC. §6's "implementation-question filter" rejects any candidate question that requires "codebase context, schema knowledge, metric vocabulary, or evaluation tooling to answer". Asking the author "is this AC `code / config / doc / ops / observation`?" arguably requires schema knowledge — the author needs to know what the canonical enum values mean. ac-06 anticipates this: "the implementation-question filter is not relaxed by this addition — `ac_type` is an intent declaration, not an implementation question." That framing holds if the prompt presents each enum value with its semantic gloss inline (which ac-06 also requires). Still, the design SKILL.md has a "**Mode-switch trigger after repeated rejection**" rule that fires on phrases like "I'd need codebase context" — if the author rejects the `ac_type` prompt as schema-shaped, the agent might tip into closed-mode unexpectedly.
**Evidence:** `plugins/orb/skills/design/SKILL.md:162-178` (the implementation-question filter and mode-switch trigger). ac-06's framing acknowledges the tension but resolves it by claim, not by mechanism.
**Recommendation:** In ac-06, add a sentence that explicitly exempts the `ac_type` prompt from the mode-switch-trigger count — author rejections of "this AC's type" are not signals to switch modes; they're routed to a sensible default (`code`) without escalation.

### [LOW] Two adjacent code paths now handle adjacent transforms over the same files

**Category:** missing-requirement
**Pass:** 2
**Description:** ac-03 adds a migration step at `migrations.rs` that walks `.orbit/specs/**/spec.yaml` and rewrites `time_gated: true` → `ac_type: observation`. ac-05 extends the reconcile registry at `reconcile.rs` to handle brownfield `ac_type` values. For orbit's own corpus the migration is what runs (auto on version mismatch); for brownfield corpora the reconcile is what runs (explicit `--reconcile`). The boundary is clean in practice, but the spec doesn't say what happens if a brownfield repo upgrades from a 0.2 schema with `time_gated: true` ACs of its own — does the auto-migration run or does the user need `--reconcile`?
**Evidence:** ac-03 is auto-invoked per `migrate_time_gated_to_ac_type` registry entry. ac-05 is `--reconcile`-gated. No AC names which code path handles a brownfield 0.2 → 0.3 upgrade where the brownfield corpus itself has `time_gated: true` ACs (it wouldn't — `time_gated` only existed in orbit's own 0.2 schema — but the spec doesn't say so).
**Recommendation:** Add one line to ac-03 or ac-05 confirming that the migration runner is the path for `time_gated → ac_type` (regardless of corpus origin), and reconcile-mode is the path for brownfield `ac_type` value normalisation. State the band, then move on.

---

## Honest Assessment

The plan is largely implementation-ready and the AC structure is exceptionally well-tied to file paths and line numbers — the implementer will not have to triangulate where things go. The two-band close-time rule via `AcType::blocks_close()` is a clean single-source-of-truth design, the schema-version bump path is well-formed, and the canonical migration approach (raw-YAML rewrite under a version-gate) is the right tool. The wiring across review-pr / drive / design SKILL.md is plausible and ac-13's meta-gate is the right kind of check.

The biggest risk is ac-05's reconcile-registry extension presuming richer rule shapes than the precursor v1 contract delivers — the implementer either does scope-creep silently to make the regex split work, or hits the v1 ceiling and has to file a follow-up. The 2026-05-12-reconcile-mode spec was explicit that value-transforms wait for a second-project trigger, and this is that trigger; the spec should name the rule-shape extension as its own AC rather than burying it inside ac-05's description.

Secondary risk is the field-rename in ac-02 missing its SKILL.md consumer, the citation of a deleted memo, and the brownfield dry-run sequencing — all fixable with small wording changes. The "single live target" mis-claim in ac-04 is a fact bug that should be corrected even though the migration handles all three targets correctly.

REQUEST_CHANGES on the strength of the ac-05 rule-shape gap (HIGH) and the four MEDIUM findings. The MEDIUMs are individually small edits but cumulatively shift the implementer's mental model — addressing them now beats discovering them mid-cycle.
