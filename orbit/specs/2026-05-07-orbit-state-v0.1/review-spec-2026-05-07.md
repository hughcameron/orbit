# Spec Review

**Date:** 2026-05-07
**Reviewer:** Context-separated agent (fresh session)
**Bead:** n/a — pre-substrate spec, reviewed from `orbit/specs/2026-05-07-orbit-state-v0.1/spec.yaml` directly per decision 0015 (cold-fork reviews read from spec YAML directly)
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 6 |
| 2 — Assumption & failure | Pass-1 medium findings + content signals (deployment, cross-system boundaries, data migrations) | 7 |
| 3 — Adversarial | Pass-2 structural concerns (cold-bootstrap circularity, partial-update recovery, choices-format-integrity contradiction) | 5 |

## Findings

### [HIGH] Self-reference: skill rewrites cannot be driven by the skills being rewritten
**Category:** missing-requirement
**Pass:** 3
**Description:** ac-14 names six substantive skill rewrites: `drive/`, `implement/`, `rally/`, `review-spec/`, `review-pr/`, `audit/`. Implementing this spec uses those exact skills. Until they're rewritten and tested, the implementation must run on the bd-version skills. ac-18 (dogfood window) bans `bd` invocations strictly, but the *pre-dogfood implementation phase* fallback policy is not explicitly contracted. Without that policy, an agent reading the spec mid-build cannot tell whether running `bd ready` on day 5 of week 2 (well before dogfood) is a halt-trigger or routine.
**Evidence:** ac-14 description + ac-18 verification + constraint `fallback_policy: Strict — no bd X invocation in the orbit repo during the dogfood window`. The window-scoped wording is correct, but the implication "outside the window, bd is allowed" is left implicit.
**Recommendation:** Add an explicit pre-dogfood fallback clause to constraints: "Outside the dogfood window, bd-version skills and bd verbs MAY be used until each skill's orbit-version replacement passes its parity test (ac-14 verification). The dogfood window starts only after every skill in ac-14 has shipped." Also pin: ac-14 verification must be runnable against post-migration content **before** the dogfood window opens.

### [HIGH] Choices write path contradicts the load-bearing format-integrity claim
**Category:** constraint-conflict
**Pass:** 2
**Description:** Trade-off (a) and the load-bearing value both rest on "agents never touch raw YAML; format integrity is parser-enforced via serde, every write round-trips." But ac-10 declares: "Choices are indexed read-only (writes happen as MD-to-YAML during Migration A and as direct file writes thereafter — substrate does not own choice writes in v0.1)." If `/orb:design` and human edits write choices outside the substrate, format integrity is no longer parser-enforced for that entity type. The "every write round-trips" property is violated by construction for choices.
**Evidence:** `values.load_bearing` (lines 21–27 of spec.yaml) vs. ac-10 description (lines 256–266). Trade-off (t-a) counterweight is "Round-trip test suite as ship-blocker; atomic write (temp + rename); golden files." None of those apply if non-substrate writers produce the file.
**Recommendation:** Either (a) add a write verb for choices in v0.1 (`choice.write` or similar) routing through serde — preferred, restores the invariant — or (b) pin a CI check that runs the choice round-trip on every commit (ac-16 currently scopes to "round-trip test on every commit" without naming entity coverage). Make the entity scope explicit. If (b), state that choices written outside the substrate must still pass round-trip in CI, and define what "fail" means (block merge).

### [HIGH] Round-trip property does not catch lossy-parse format-integrity bugs
**Category:** test-gap
**Pass:** 2
**Description:** ac-01's verification is "every fixture parses + reserialises to byte-identical output." This catches symmetric serialise/deserialise bugs but is silent on **schema-conformance failures** where the parser silently drops unknown fields. Deserialise → reserialise is byte-identical because the dropped field is absent from both the parsed model and the reserialised output. The spec drifts and round-trip stays green. This is the classic absorbing-state failure for schema migrations.
**Evidence:** ac-01 description + verification together. No mention of "deny unknown fields" or schema-conformance check.
**Recommendation:** Add to ac-01 verification: "Parser is configured to reject (not silently drop) unknown fields. A test fixture with an extra unknown field fails parse with a clear error." This is a one-line serde annotation (`#[serde(deny_unknown_fields)]`) with one test, but the contract must pin it or it's an easy miss.

### [HIGH] No post-ship rollback path
**Category:** missing-requirement
**Pass:** 3
**Description:** ac-19 ships v0.1.0. Kill conditions K1–K6 cover pre-ship pivots. Halts h-1, h-4, h-5a, h-5b cover in-build reverts. There is no path for "v0.1.0 tag is pushed; dogfood-week-2 reveals a deep flaw; what now?" The strict no-fallback policy means agents cannot revert to bd. Migration B preserves `.beads/` only "during dogfood" (ac-13); the cluster's premise is dogfood lasts ≥1 week (sc-4), and the spec implies `.beads/` is removed before tag push (ac-19: ".beads/ removed from the orbit repo"). Once `.beads/` is removed and the tag is pushed, there is no return path other than re-running Migration B in reverse — which is undefined.
**Evidence:** ac-19 verification pins `.beads/ absent from main`. h-* and k-* conditions are pre-ship. No "post-ship rollback" or "revert tag" path.
**Recommendation:** Either (a) keep `.beads/` archived (e.g. `.beads-archive/` in a non-tracked branch or a separate stash branch) for N weeks post-ship — give the dogfood-criterion window a real revert path; or (b) add a kill condition K7 "post-ship critical defect within N days of tag push" with a named pivot (revert tag, restore `.beads/` from git history, re-enter from decision 0015). State explicitly which.

### [MEDIUM] Skill rewrite work in ac-14 is unbudgeted at per-skill granularity
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-14 lists six substantive skill rewrites. drive/, rally/, review-pr/ are non-trivial. The recut budget is 13–17 working days **for the whole spec**. There is no per-deliverable budget for the skill rewrites, and the recut estimate's derivation isn't shown in the spec. If skill rewrites alone consume 8 days, the substrate work has 5–9 days. If they consume 12 days, the substrate is starved.
**Evidence:** budget section + ac-14 description. recut_estimate is named but not decomposed.
**Recommendation:** Add an internal allocation hint (not a hard constraint): "Indicative split — substrate (Rust + serde + SQLite + MCP): N days. Migrations A+B: M days. Skill rewrites (six skills): K days. CI gates: J days. Multi-machine builds: I days. Dogfood: 5 days. Sums to recut estimate." The number doesn't have to be perfect, but a decomposition forces the estimator to commit to per-deliverable shape and gives Theme 5a halt-trigger an early-warning signal at the subtotal level, not just the total.

### [MEDIUM] Dogfood window restart has no upper bound
**Category:** failure-mode
**Pass:** 2
**Description:** ac-18: "Window is restartable if a verb breaks mid-window (broken verb is fixed, window starts over)." A broken verb on day 6 of dogfood means a 5-day window restart. Two such breaks consume 10 working days. The spec does not name a maximum number of restarts before escalation. With the 4-week ceiling and 13–17-day recut, two restarts would push past week 4 invisibly.
**Evidence:** ac-18 + budget + sc-4 (≥1 week without fallback).
**Recommendation:** Add: "Two consecutive restarts (i.e. dogfood breaks twice) triggers escalation E5 (awkward-middle gap), with the question 'restructure dogfood scope or extend budget.' Three restarts is a kill condition (drop into K3-bd-primitive-inexpressible or a new K7-dogfood-not-converging)."

### [MEDIUM] "22 verbs" parity claim has unscored gaps relative to bd
**Category:** missing-requirement
**Pass:** 1
**Description:** Goal claims "bd verb parity (single-repo only)." Scope explicitly defers cross-repo `--global`, the memory loop, the gate verb, and any bd primitive (issue dependency graph, prime cross-repo) flagged via K3. The "parity" rhetoric obscures these gaps. A reader reaching "ac-05 — 22 verbs across both surfaces" may believe parity is full, when it is "single-repo, no graph dependencies, no gate, no auto-injected memory."
**Evidence:** goal (lines 2–8) vs. scope.deferred_to_v0_2_or_later (lines 115–123) vs. K3 description.
**Recommendation:** Reword goal: "bd verb parity for single-repo workflow — explicitly excluding cross-repo queries, the memory loop (auto-injection at prime), gate semantics, and bd's issue-dependency-graph primitive. These are v0.2+." Also: pin a deviation table — for each bd verb in current `bd --help`, mark "covered in v0.1 / deferred / deliberately dropped." Without this, K3 trigger ("a bd primitive the v0.1 schema cannot express") fires on a moving target.

### [MEDIUM] Concurrency & locking under-specified for the agent-concurrency case
**Category:** failure-mode
**Pass:** 2
**Description:** ac-03 names file-level locking with configurable timeout. Concurrent-write races between two short writes are covered. The harder case — an LLM tool call that holds a lock for a long time (mid-`spec.update`, agent thinks for 90 seconds, second agent calls `task.claim`) — is not explicitly addressed. K4 names the kill condition but ac-03 doesn't pin the *expected* behaviour: read-during-write, lock TTL with auto-release, stale-lock recovery, lock owner identification.
**Evidence:** ac-03 description + verification + K4 trigger ("fails to prevent races at acceptable performance OR deadlocks under realistic agent concurrency"). "Realistic agent concurrency" is undefined.
**Recommendation:** Add to ac-03: (i) default lock timeout value (recommend 30s — matches typical LLM tool-call latency); (ii) stale-lock recovery: "lock files contain pid + start_timestamp; locks older than N×timeout are considered stale and reclaimable"; (iii) read consistency: "reads do not require lock acquisition; readers may see a slightly stale view, never a partial view (atomic temp+rename guarantees this)."

### [MEDIUM] ac-13 sample size for Migration B fidelity is undefined
**Category:** test-gap
**Pass:** 2
**Description:** ac-13 verification: "Sample of N bd issues are spot-checked against the resulting orbit files for content fidelity." N is unspecified. Spot-checking is non-deterministic — cannot be enforced in CI, cannot be reproduced.
**Evidence:** ac-13 verification (lines 305–309).
**Recommendation:** Either (a) replace spot-check with full-coverage verification: "every bd issue with label=spec produces exactly one .orbit/specs/<id>.yaml; every bd note produces exactly one note event; lossless conversion proven by total-count equality and hash-set comparison of (issue_id, note_id) tuples." Or (b) name N (e.g., "N = max(10, 10% of total)") and pin selection method (random with seed for reproducibility). Recommend (a).

### [MEDIUM] Multi-machine binary build is not de-risked early
**Category:** failure-mode
**Pass:** 2
**Description:** ac-19 places multi-machine binary builds (mac + beelink) at ship time. E6 is the escalation if they diverge. But Rust + SQLite (with C dependencies) cross-compile mac↔linux is not always smooth, especially statically-linked. Discovering a cross-compile blocker in week 4 has no recovery time. Q6-iv records "cross-compile mature" as the assumption — that's the assumption most likely to be wrong here.
**Evidence:** ac-19 (verification at ship time) + E6 (post-discovery escalation) + Q6-iv (asserted, not validated).
**Recommendation:** Add an early-validation AC: "By end of week 1, a 'hello world' orbit binary builds and runs on both mac and beelink. If not, E6 fires at week-1 review, not at week 4." This converts the cross-compile risk from a week-4 cliff into a week-1 known.

### [MEDIUM] ac-11 prime output bound is undefined for non-trivial states
**Category:** test-gap
**Pass:** 2
**Description:** ac-11: "Output is thin (target under 40 lines for a project with 5 open specs and 3 memories)." This bounds the small case only. With 50 specs / 200 memories — easily reached after a few months of orbit use — what's the bound? Output silently grows or the test silently passes only in small projects.
**Evidence:** ac-11 description + verification.
**Recommendation:** Restate as: "Output for a project with N specs and M memories is bounded by `f(N, M)` lines, where `f(5, 3) ≤ 40`. The full ready queue is shown; recent memories are capped at K most-recent (recommend K=10); specs beyond the open set are summarised, not enumerated. The cap K is configurable."

### [MEDIUM] ac-05 parity test compares output structures only, not state mutations
**Category:** test-gap
**Pass:** 2
**Description:** ac-05 verification: "comparing output structures." Both CLI and MCP also write to state.db and to canonical files. Output structures might match while state mutations diverge (e.g. CLI writes a denormalised cache that MCP doesn't, or MCP triggers an index rebuild that CLI skips).
**Evidence:** ac-05 description + verification.
**Recommendation:** Strengthen verification: "After invocation via each surface, the resulting on-disk state (canonical files + state.db) is byte-identical between CLI and MCP for every verb. Snapshot before, invoke via CLI, snapshot, reset, invoke via MCP, snapshot — diff."

### [LOW] Deliverables include items not pinned in any AC
**Category:** missing-requirement
**Pass:** 1
**Description:** Deliverables list includes `.orbit/.gitignore` (selective gitignore) and "install instructions" in README. Neither has a corresponding AC. ac-15 requires the early-release notice in README but does not require install instructions.
**Evidence:** deliverables (lines 569–587) vs. ac-15 (lines 326–337).
**Recommendation:** Add a short AC: "ac-20 (code): `.orbit/.gitignore` ships with the contracted ignore set (state.db, schema-version, locks/ ignored; cards/, choices/, specs/, memories/ tracked). README includes install instructions covering both mac and beelink. Verification: grep `.orbit/.gitignore` for each contracted entry; README install section runs end-to-end on a clean machine and produces a working `orbit prime`."

### [LOW] "git tag pushed" + "internal-only release (no GitHub release)" potential ambiguity
**Category:** constraint-conflict
**Pass:** 1
**Description:** ac-19 names both "v0.1.0 git tag pushed" and "Internal-only release (no GitHub release artifacts, no crates.io publish)." A pushed tag IS visible on GitHub. The two are not strictly incompatible (a tag without an associated GitHub Release is still internal in spirit), but a reader may parse "no GitHub release artifacts" as "do not push the tag publicly."
**Evidence:** ac-19 (lines 371–381).
**Recommendation:** Clarify: "git tag v0.1.0 created and pushed to origin. No GitHub Release object is created. No artifacts are uploaded. No crates.io publish. The tag exists in the public git history but is not packaged or announced."

### [LOW] Other-machine chezmoi-distribution timing not pinned
**Category:** failure-mode
**Pass:** 3
**Description:** Q8 notes "chezmoi distributes new MCP config + skill updates." If a development machine pushes a chezmoi update mid-build, the second machine pulls partial state. Cross-machine consistency during the dogfood window is implicit.
**Evidence:** Q8 + sc-4 + ac-18.
**Recommendation:** Add to constraints: "chezmoi distribution of v0.1 MCP config and updated skills happens only after ac-19. During the build, the second machine continues to use pre-orbit-state config." One sentence.

### [LOW] No AC pins error-message taxonomy
**Category:** missing-requirement
**Pass:** 1
**Description:** Multiple ACs reference error paths (ac-03 "loud failure," ac-06 "rejects with a clear error," E4 "error messages requiring >2 agent attempts to parse"). E4 implies measurable agent-readability, but no AC pins what "clear" means. Without a contract, error messages drift toward developer-jargon over time.
**Evidence:** ac-03, ac-06, E4.
**Recommendation:** Add to ac-05 (or a new AC): "Error messages follow the orbit error format: `<verb>: <category>: <human-sentence>`. Categories: not-found, conflict, locked, malformed, unauthorised, unavailable. Test fixtures cover one error per category per verb where applicable."

### [LOW] Migration A grep coverage scoped to tracked files only
**Category:** test-gap
**Pass:** 2
**Description:** ac-12 verification: "grep for old paths in any tracked file returns 0 results." Correctly scoped to tracked files (untracked test fixtures or scratch dirs would generate false positives). But the wording "any tracked file" is ambiguous — does it include `.gitignore`d files that are nonetheless tracked? The intent is likely "git ls-files | xargs grep." Worth pinning the exact command.
**Evidence:** ac-12 (lines 280–294).
**Recommendation:** State the command verbatim: "Verification: `git ls-files | xargs grep -l 'orbit/cards\|orbit/decisions\|orbit/specs'` returns no matches."

### [LOW] No AC for partial-update recovery on multi-card spec.close
**Category:** failure-mode
**Pass:** 2
**Description:** ac-06 says spec.close updates linked cards' specs_array as a serde side-effect. ac-09 says "exactly once" for both cards in a 2-card test. But what if the write to card #1 succeeds and the write to card #2 fails (disk full, lock contention, parser error)? Partial-update state is undefined. Either both cards update or neither — but the spec doesn't pin which.
**Evidence:** ac-06, ac-09.
**Recommendation:** Pin the semantics: "spec.close performs all linked-card updates in a single transaction. If any update fails, all are rolled back; the spec remains open. Verification: simulate a failure on the second card update; assert the first card is unchanged and the spec is still open."

---

## Honest Assessment

This is a thorough, well-structured spec — the tabletop methodology shows. The values, trade-offs, halts, escalations, kills, and ACs are coherent and traceable. The recut estimate (13–17 days) inside a 4-week budget is the load-bearing planning claim; whether it holds is unprovable from the spec alone but the structural discipline is in place.

The biggest risks are **structural**, not tactical:

1. **Self-referential bootstrapping (HIGH).** The skills under rewrite are the skills used to operate the rewrite. The pre-dogfood fallback policy must be explicit, and skill-by-skill ship order needs to land before the dogfood window opens. This is the single most likely place for the build to wedge.
2. **Choices write path (HIGH).** Format integrity is the load-bearing value, but ac-10 lets choices be written outside the substrate. Either close that hole or weaken the load-bearing claim. Don't ship both.
3. **Round-trip ≠ schema-conformance (HIGH).** A serde annotation away. Easy to fix, easy to miss.
4. **No post-ship rollback (HIGH).** Once `.beads/` is gone and the tag is pushed, there's no return. Pin a return path or accept the no-return.
5. **Multi-machine cross-compile (MEDIUM).** Validate at week 1, not week 4. The single change with the highest expected-value-of-information.

Recommend REQUEST_CHANGES with the four HIGH findings as ship-blockers and the eight MEDIUM findings as before-implementation-starts items. The five LOW findings are polish — acknowledge them in the response and incorporate at the author's discretion.

The contract is *close* to ready. None of the findings invalidate the architecture or the cluster — they sharpen the contract so implementation has fewer blind spots.
