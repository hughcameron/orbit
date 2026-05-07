# Spec Review

**Date:** 2026-05-07
**Reviewer:** Context-separated agent (fresh session)
**Bead:** n/a — pre-substrate spec, reviewed from `orbit/specs/2026-05-07-orbit-state-v0.1/spec.yaml` directly. Prior review (`review-spec-2026-05-07.md`) issued REQUEST_CHANGES; this is the v2 pass against the amended spec.
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|--------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (deployment, data migrations, cross-system boundaries, schema changes) | 7 |
| 3 — Adversarial | not triggered (no contradicted load-bearing assumptions; no cascading failure modes) | — |

## Pre-flight: prior review reconciliation

The previous review issued 4 HIGH, 8 MEDIUM, and 5 LOW findings. The amended spec materially incorporates substantially all of them:

```
Prior finding                                    Severity   Status in v2 spec
-----------------------------------------------  --------   ------------------
Self-referential bootstrapping                   HIGH       FIXED — pre_dogfood + during_dogfood policies pinned
Choices write path violates format integrity     HIGH       FIXED — ac-16 explicitly scopes CI round-trip to choices
Round-trip ≠ schema conformance                  HIGH       FIXED — ac-01 now requires deny-unknown-fields + per-entity test
No post-ship rollback path                       HIGH       FIXED — K7 (post-ship critical defect) pinned with 14-day window
Skill rewrite work unbudgeted at granularity     MEDIUM     FIXED — budget.indicative_split decomposes to 17 days
Dogfood window restart has no upper bound        MEDIUM     PARTIAL — E5 covers 2-restart escalation; no 3-restart kill
"22 verbs" parity gaps                           MEDIUM     FIXED — goal explicitly excludes 4 named primitives
Concurrency under-specified                      MEDIUM     FIXED — ac-03 pins timeout, stale-lock recovery, read-during-write
ac-13 sample size undefined                      MEDIUM     FIXED — full-coverage hash-set comparison replaces spot-check
Multi-machine binary not de-risked early         MEDIUM     FIXED — ac-21 pins week-1 cross-compile gate
ac-11 prime output bound undefined               MEDIUM     FIXED — bounded formula `f(N,M) ≤ 40 + 2*open + min(M,10)`
ac-05 parity test compares output only           MEDIUM     FIXED — verification covers canonical files + state.db
Deliverables not in any AC                       LOW        FIXED — ac-20 added for .gitignore + install
"git tag pushed" + internal-only ambiguity       LOW        FIXED — ac-19 clarifies no Release object, no announce
chezmoi distribution timing                      LOW        FIXED — constraints.chezmoi_distribution pinned
Error-message taxonomy unspecified               LOW        FIXED — ac-05 pins format `<verb>: <category>: <sentence>`
Migration A grep wording ambiguous               LOW        FIXED — verbatim `git ls-files | xargs grep` command
Partial-update recovery on multi-card close      LOW        FIXED — ac-06 pins transactional + partial-failure test
```

The amended spec is materially stronger. The remaining findings below are either (a) introduced by the amendments themselves, or (b) gaps that survived the prior pass.

## Findings

### [HIGH] CI round-trip gate (ac-16) omits the schema-version file
**Category:** test-gap
**Pass:** 2
**Description:** ac-01 names six entity types: specs, tasks, cards, choices, memories, **schema-version**. ac-16 (the CI gate that wires the format-integrity halt-trigger into the build) iterates "every file under .orbit/specs/, .orbit/cards/, .orbit/choices/, and the memories file." schema-version is conspicuously absent from the CI scope. If schema-version drifts (a hand-edit, a malformed migration runner output, a serde bug specific to the schema-version entity), CI passes and the format-integrity claim breaks for the one file the migration runner reads first on every invocation. Tasks are also absent from the CI list — defensible if the substrate is the only writer (per values.enforcement.substrate_written) but worth pinning explicitly so the omission is intentional, not accidental.
**Evidence:** ac-01 description (lists schema-version as a typed entity) vs. ac-16 verification (omits it). values.enforcement.substrate_written lists `[specs, tasks, memories]`; values.enforcement.human_written lists `[cards, choices]`. schema-version is in neither category — its enforcement mechanism is undefined.
**Recommendation:** (a) Add schema-version to ac-16's CI iteration list, AND (b) classify schema-version explicitly in values.enforcement (likely substrate_written, since the migration runner owns it). One-sentence amendment to ac-16: "and the schema-version file." One-line addition to values.enforcement.substrate_written.entities: `[specs, tasks, memories, schema-version]`. While there, state explicitly that tasks are excluded from CI round-trip because they're append-only JSONL written exclusively by the substrate.

### [MEDIUM] ac-21 "skeleton binary" may not exercise the actual cross-compile risk
**Category:** failure-mode
**Pass:** 2
**Description:** ac-21 converts the multi-machine cross-compile cliff into a week-1 known. Good. But the verification ("skeleton binary that performs `orbit --version` or equivalent smallest-meaningful invocation") doesn't pin what the skeleton must link. Rust + SQLite cross-compile pain on mac↔linux is concentrated in C dependencies — rusqlite bundled vs system, openssl, ring, etc. A skeleton binary that only prints a version string doesn't link rusqlite and won't surface the most likely failure mode. The early-warning signal turns out to fire only when the actual blocker is in pure-Rust code, which is the less-likely failure class.
**Evidence:** ac-21 description ("'hello world' orbit binary") + verification ("smallest-meaningful invocation"). No requirement that the skeleton link the C-dependency chain that production binaries will use.
**Recommendation:** Strengthen ac-21: "Skeleton binary must link the same C-dependency chain as the production binary (rusqlite at minimum; bundled or system per the production build configuration). `orbit --version` is acceptable as the runtime invocation, but the link step must exercise SQLite. Verification: `nm` (linux) / `otool -L` (mac) on the skeleton binary shows SQLite symbols present."

### [MEDIUM] Main-branch migration timing is not explicitly pinned
**Category:** missing-requirement
**Pass:** 2
**Description:** ac-12 and ac-13 are explicitly worktree validations ("Migration A runs against a worktree of orbit's main branch"; "Migration B runs against orbit's actual bd state in a worktree"). The actual main-branch migration — the moment `.orbit/` becomes the live state and `.beads/` becomes `.beads-archive/` on main — has no AC. Implied to happen at dogfood-window-open (ac-14 + ac-18 read together imply this), but never stated. The pre_dogfood fallback policy and the during_dogfood strict policy both reference "the dogfood window," whose start moment is the implicit migration cutover — but the cutover itself is unowned.
**Evidence:** ac-12, ac-13 (both worktree-only). ac-14 (parity tests gate dogfood). ac-18 (dogfood window). No AC pins "Migration A and B run against main, atomically, on date X."
**Recommendation:** Add an AC (or extend ac-18): "Pre-dogfood cutover — on the day the dogfood window opens, Migration A and Migration B run against main (not a worktree). The cutover is a single atomic operation: both migrations succeed or both are reverted via `git reset --hard HEAD` before the window starts. Verification: cutover commit on main contains both migrations; pre-cutover and post-cutover commit hashes recorded in operator log; no orbit-version skill invocation occurs against main before cutover."

### [MEDIUM] Skill parity testing for review/audit skills is harder than for CRUD verbs
**Category:** test-gap
**Pass:** 2
**Description:** ac-14 verification: "invoke the bd version against a reference scenario, invoke the orbit version against the same scenario, compare resulting state and outputs. Behaviour-parity passes for all six." For drive/implement/rally this is reasonable — they have observable state effects (specs created, tasks claimed, beads closed). But review-spec, review-pr, and audit produce *judgements* — the output is a verdict + findings list. Parity for these skills is fuzzy: the bd-version reviewer may flag 17 issues; the orbit-version may flag 14. Are 14 of the 17 the same? Is the verdict the same? "Same outputs" is unspecified for judgement-emitting skills. Without a clearer parity bar, ac-14 verification can pass trivially (same verdict) or fail noisily (different finding ordering).
**Evidence:** ac-14 description + verification. Skill list includes review-spec, review-pr, audit — all judgement-emitting.
**Recommendation:** Differentiate the parity bar: "For state-mutating skills (drive, implement, rally), parity is byte-identical resulting state. For judgement-emitting skills (review-spec, review-pr, audit), parity is (a) verdict equality (APPROVE / REQUEST_CHANGES / BLOCK match) AND (b) finding-coverage equality at the HIGH severity level (every HIGH finding from the bd-version maps to a finding of equal-or-higher severity in the orbit-version, and vice versa). Lower-severity finding deltas are recorded but do not fail the parity test."

### [MEDIUM] ac-05 "reset" semantics are not pinned
**Category:** test-gap
**Pass:** 2
**Description:** ac-05 verification: "snapshot the .orbit/ tree before invocation; invoke via CLI; snapshot; **reset**; invoke via MCP; snapshot." "Reset" is undefined. If reset is `git checkout -- .orbit/`, state.db is not in git (per ac-20: state.db is gitignored), so the reset doesn't restore it — the second snapshot starts from a polluted state. If reset rebuilds state.db from files (per ac-02), the rebuild process itself is under test. If reset is `rm -rf .orbit/ && orbit init`, fixture content is lost. The verification cannot run reproducibly until reset is pinned.
**Evidence:** ac-05 verification (line 268) + ac-20 (.gitignore for state.db).
**Recommendation:** Pin reset semantics: "Reset is `git stash` of any working-tree changes plus deletion-and-rebuild of state.db (`rm .orbit/state.db && orbit verify --rebuild`). This restores canonical files to pre-invocation state and rebuilds the index from files. The reset itself is exercised by ac-02; ac-05 inherits its correctness."

### [MEDIUM] Migration A YAML body round-trip not exercised by a fixture
**Category:** failure-mode
**Pass:** 2
**Description:** ac-12 converts MADR markdown decisions to YAML, "preserving frontmatter as YAML keys." MADR bodies are unstructured prose — they presumably land in a `body: |` (or `body: >`) multiline string. YAML multiline string round-tripping has well-known edge cases: trailing whitespace stripped, line-ending normalisation (CRLF→LF), block-scalar style choice (literal vs folded), indentation indicator drift. ac-01's fixture set ("≥1 fixture per entity type, ≥3 covering edge cases like unicode and quoting") doesn't explicitly require a multiline-body fixture, and choice files are the entity most exposed to this failure mode. A choice with a code-fence in its body, a trailing blank line, or hard-tab indentation is likely to break round-trip on the first migration.
**Evidence:** ac-12 (MD→YAML conversion), ac-01 (fixture set), values.load_bearing (format integrity, parser-validated).
**Recommendation:** Add to ac-01 fixture set: "Choice fixture suite includes (i) a body with a triple-backtick code fence containing YAML, (ii) a body ending in a trailing blank line, (iii) a body with hard-tab indentation. Each must round-trip byte-identical." Also add to ac-12 verification: "Post-migration, every choice file round-trips byte-identical (per ac-16 CI gate). At least one converted choice in the fixture set has a multiline body with embedded special characters."

### [MEDIUM] Dogfood window restart count has escalation but no kill
**Category:** missing-requirement
**Pass:** 2
**Description:** Prior review recommended "Two restarts → escalation E5; three restarts → kill condition." E5 now contains the 2-restart escalation (good). But no kill condition fires at 3 restarts. The escalation outcome at 2 restarts is "(a) defer to v0.2, (b) extend budget, (c) restructure dogfood scope" — all path-forward options. At 3 restarts, the implication is K3 (bd-primitive-inexpressible) MIGHT fire, but only if the restart cause is verb-surface. A non-verb cause (lock corruption, MCP latency spike, migration drift) at 3 restarts has no kill path — the spec allows infinite restarts.
**Evidence:** E5 (2-restart escalation), kill_conditions K1-K7 (none cover dogfood-not-converging).
**Recommendation:** Either (a) add a new kill K8: "Three consecutive dogfood restarts within v0.1 budget — invalidates dogfood-in-orbit-self claim; pivot to dogfood on a fresh greenfield repo per K5 pivot path." Or (b) explicitly accept the policy: "Restart count is bounded by the budget ceiling (4 weeks). Beyond that, escalation E5 has fired and a Hugh decision is required; no automatic kill." Recommend (a) — keeps the kill-condition discipline tight.

### [LOW] ac-13 hash-set verification source-side snapshot is implicit
**Category:** test-gap
**Pass:** 2
**Description:** ac-13 verification: "Hash-set comparison of (issue_id, note_id, memory_id) tuples between source and destination produces zero deltas." After Migration B runs, `.beads/` has been renamed to `.beads-archive/`. The "source" side of the hash-set comparison must be snapshotted *before* the rename, or the comparison reads from the archive. Implied but not stated.
**Evidence:** ac-13 verification.
**Recommendation:** One-line clarification: "Source-side hash set is captured before migration runs and persisted to a test fixture; destination-side hash set is computed post-migration. Comparison runs against the persisted source snapshot, not against `.beads-archive/`."

---

## Honest Assessment

The amendments materially incorporate substantially all the prior review's findings. The contract is significantly stronger than v1: the choices write path is policed by CI, post-ship has a real return path, the cross-compile cliff has a week-1 early-warning, and concurrency edge cases have explicit semantics.

The single remaining HIGH (CI scope omits schema-version) is a one-line fix but materially completes the format-integrity claim — the migration runner reads schema-version on every invocation, and an undetected drift there breaks every subsequent migration.

The MEDIUMs cluster around two themes:
1. **Test-fidelity gaps** — "reset" semantics, multi-line YAML round-trip fixtures, judgement-skill parity. Each is a concrete test-suite addition, not an architectural change.
2. **Operational ordering** — main-branch cutover timing, skeleton-binary link scope. The architecture is sound; the runbook isn't fully written.

None of these findings invalidate the architecture or the cluster premise. None require returning to design or discovery. They sharpen the contract against runtime ambiguity that would otherwise consume implementation cycles.

The 4-week budget at 13–17 working days remains the load-bearing planning claim; whether it holds is outside the spec's reviewability. The structural discipline that would let it hold is in place.

Recommend REQUEST_CHANGES with the one HIGH as the ship-blocker, the six MEDIUMs as before-implementation amendments, and the one LOW as a polish item. After these land, the spec is ready for /orb:implement.
