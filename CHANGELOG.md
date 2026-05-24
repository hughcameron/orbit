# Changelog

All notable changes to orbit are documented here. Format follows [Keep a Changelog](https://keepachangelog.com/).

## [0.4.33] - 2026-05-24

Ships the brownfield-migration-hardening rally end-to-end — three drives covering the layout-classifier substrate, the conformance audit's new `undotted_substrate` finding family, and the reconcile-mode auto-mapping for legacy `Card.maturity` values. Closes the failure-modes the prior session-prime envelope surfaced: conformance findings that recommended actions which made things worse, topology seeds pointing at orbit-plugin paths in downstream projects, and reconcile-mode aborts on common brownfield maturity values.

### Added

- **`SubstrateLayoutState` 6-variant enum + `classify_substrate_layout` helper** in `orbit-state/crates/core/src/verbs.rs`. Single source of truth for `/orb:setup`'s state machine and `audit.conformance`'s wrapped-undotted suppression — both call the same predicate. Variants: `greenfield`, `idempotent`, `brownfield-bare`, `wrapped-undotted`, `mixed-bare`, `mixed-undotted`. Per spec 2026-05-24-setup-is-orbit-state-aware ac-11.
- **`substrate classify` CLI + MCP verb** exposing the classifier across both surfaces with byte-equal parity tests for the `idempotent` / `wrapped-undotted` / `brownfield-bare` shapes. Per spec 2026-05-24-setup-is-orbit-state-aware ac-18.
- **`plugin_repo` config field** on `OrbitConfig`. Optional boolean; gates `topology.setup`'s substrate-typed seed branch — `plugin_repo: true` writes the five `cards`/`choices`/`memories`/`specs-substrate`/`topology` seeds (after validating each `canonical_code` path exists in the working tree); absent or false writes a one-line `.orbit/topology/README.md` pointing at `/orb:topology`. Fixes the prior failure where topology seeds in downstream projects produced 20+ stale-pointer drift entries on the immediate next audit pass. Per spec 2026-05-24-setup-is-orbit-state-aware ac-12 / ac-13.
- **`undotted_substrate` conformance finding family** at `audit_conformance_at`. HIGH severity, subject `orbit/`, state `undotted_substrate`, subsystem `setup`, remediation `orbit setup`. Predicate: any of `orbit/{cards,choices,specs,memos}/` exists AND `.orbit/cards/` does not. Evidence carries per-subdir counts so consumers can surface migration scale. When fired, every `.orbit/`-dependent finding family (canonical-files-missing / card-state / memo-staleness / pin-state / decisions-md-unmigrated) is suppressed — the single emitted finding is the prerequisite remediation. Per spec 2026-05-24-workflow-conformance.
- **`decisions_md_unmigrated` conformance finding family**. Walks `.orbit/decisions/*.md`; for each unconverted MADR markdown file lacking a matching `.orbit/choices/<slug>.yaml`, emits a finding pointing the operator at the manual MD→YAML conversion task. Closes the `decisions/` vs `choices/` naming gap from brownfield migrations. Per spec 2026-05-24-setup-is-orbit-state-aware ac-15.
- **`reconcile_card_maturity` Transform handler** at `orbit-state/crates/core/src/reconcile.rs`. Maps `active → established` and `in_design → emerging` on `Card.maturity` (the two brownfield values observed in older orbit corpora), passes canonical values through with `canonical pass-through: <value>` detail, quarantines anything else. Mirrors `reconcile_ac_type`'s shape exactly. Per spec 2026-05-24-brownfield-spec-migration ac-08 / ac-09.
- **`apply_top_level_transform` walker extension** in reconcile. The pre-existing walker only looked up registry rules for *unknown* top-level keys; the `recurse_inner` ac-06 pattern (Transform rules fire on canonical inner fields) is now mirrored at the top level. Without this, a Transform rule on a canonical top-level scalar like `Card.maturity` never fires. Per spec 2026-05-24-brownfield-spec-migration ac-10.

### Changed

- **`/orb:setup` SKILL.md** — adds §0 preconditions (orbit-state binary present + version ≥ 0.4.33), §3.W new section for the `wrapped-undotted` single-rename arm (`git mv orbit .orbit`), §4 mixed-state refusal naming both substrate paths and the `git status --porcelain` no-mutation invariant, §3 decisions/ migration target changed from `.orbit/choices/` to `.orbit/decisions/` (folder rename only — no MADR→YAML auto-conversion per rally lockdown), §3f.1 inline MADR conversion warning, §5 idempotent-branch preserves `plugin_repo: true`, §6d plugin_repo gating + canonical_code validation prose, §7 explicit "setup does not auto-run a drive" note.
- **`/orb:drive` SKILL.md** — input contract gains a substrate-initialised pre-check; halts with "run `/orb:setup` first" when `.orbit/state.db` and `.orbit/cards/` are both absent (instead of failing downstream with opaque `orbit` errors). Per spec 2026-05-24-setup-is-orbit-state-aware ac-08.
- **`audit_conformance_at` suppression** — single `layout_dominates` branch gates every `.orbit/`-dependent finding-builder (replaces the previous wrapped-undotted-only `layout_non_canonical` partial check). Pin-dominance now gated on `!layout_dominates` — pin is upstream of file drift, but layout is upstream of pin. Routine + aggregated drift/topology continue regardless (they don't read `.orbit/` substrate).
- **This repo's `.orbit/config.yaml`** created with `plugin_repo: true` so its own `orbit topology setup` continues to seed the substrate-typed entries.
- **Card 0017** maturity `planned → emerging`; goal sharpened to name the wrapped-undotted shape explicitly. Cards 0032 / 0039 specs[] appended via `spec.close`.

### Notes

- Tests: 446 → 509 across the three drives (+63 across classifier units, decisions-md-unmigrated builder, topology-gating, substrate.classify parity, undotted_substrate fixtures, reconcile maturity routing, walker top-level Transform regression).
- Three squashed PRs: [#29](https://github.com/meridian-online/orbit/pull/29) (Drive A, 21 ACs), [#30](https://github.com/meridian-online/orbit/pull/30) (Drive C, 12 ACs), [#31](https://github.com/meridian-online/orbit/pull/31) (Drive B, 10 ACs). Each driven from a parked rally state in a fresh top-level session — the rally's original parking reason (Agent tool unavailable inside Agent-spawned sub-agents) was harness-shaped, not design-shaped.
- Rally [`2026-05-24-brownfield-migration-hardening-rally`](.orbit/specs/2026-05-24-brownfield-migration-hardening-rally/rally.yaml) marked complete; all three children `card_phase: complete`; `unparked_2026-05-24` note added.
- Cross-drive scope decision: Drive A shipped the wrapped-undotted canonical-files-missing suppression mechanism using its classifier (a one-line check); Drive C extended the suppression to the broader `layout_dominates` pattern covering every `.orbit/`-dependent family. This split worked because the suppression piece is small enough to absorb without violating the rally's "A and C parallel" choreography.

## [0.4.32] - 2026-05-24

Adds an automatic merge step to `/orb:drive` §Completion scoped to **full autonomy**. APPROVE from the forked review-pr now triggers `gh pr merge --auto` without operator intervention; non-APPROVE verdicts and non-full autonomy modes behave exactly as before. Removes the last operator-bottleneck in full-autonomy drive pipelines.

### Added

- **`/orb:drive` auto-merge step** at §Completion step 5, scoped to `autonomy == full`. APPROVE invokes `gh pr merge --auto` (queue-merge — defers to required checks / branch protection automatically). REQUEST_CHANGES and BLOCK route through §NO-GO unchanged; `guided` and `supervised` keep today's four-option prompt. Per spec 2026-05-22-default-merge-after-review (card 0014).
- **Graceful degradation on merge failure** — when `gh pr merge --auto` returns non-zero (auto-merge disabled, branch protection, draft PR, auth/network error), drive logs the exit code, writes a `merge deferred — <reason>` spec note with a canonical token (`auto-merge-disabled` / `branch-protection` / `draft-pr` / `auth-failure` / `network-error` / `unknown`), and closes the spec without halting. Author handles merge manually on next look; the close-comment carries PR url + verdict + merge state for visibility.
- **Idempotent PR-create on resume** — §Completion step 4 inspects `gh pr view --json number,autoMergeRequest,state` before invoking `gh pr create`; if a PR already exists for the branch (prior drive crashed between create and merge), the existing PR is carried forward to step 5. No new `drive.yaml` stage value required; §Resumption table unchanged.

### Changed

- **`/orb:drive` §Completion** reordered to six numbered steps (commit-impl → commit-cards → push → gh pr create → gh pr merge --auto → spec close). Push surfaced as its own explicit step (was implicit inside "Create the PR" today); runs at all autonomy levels because `gh pr create` requires it. Heartbeat cleanup moves to a post-close admin block.
- **Autonomy table `full` row** description updated — "Pauses only for PR merge" replaced with "APPROVE auto-merges via `gh pr merge --auto`". `guided` and `supervised` rows unchanged.
- **Card 0014** maturity `planned → emerging` on first spec close.

### Notes

- ac-09 (observation window — K1 forked-review-trust kill condition) deferred by design. First five auto-merged drives capture a one-line entry indicating whether the author would have intervened; if ≥2 of 5 surface author-catchable misses, K1's pivot path fires (revert to manual merge, tighten cold-fork reviewer prompt, or invest in pre-merge hold window).
- Follow-up memo `.orbit/memos/2026-05-22-drive-autonomy-mode-names.md` carries forward to card 0005 — the `guided` vs `supervised` naming axis question surfaced mid-tabletop; not in this release's scope.

## [0.4.31] - 2026-05-22

Ships v1 of the routine-authoring substrate (card 0013, spec 2026-05-22-routine-proposals). Agents observing recurring chains of skill invocations now author named routine SKILL.md files directly to `.claude/skills/<name>/`, with audit-driven freshness via a dedicated `orbit routine verify` verb. Pillar-2 (agent self-learning) sibling to card 0022 (single-skill authoring) — sequential chains in v1 (DAGs deferred), content-addressed via `chain_id` (SHA-256 of the canonical-JSON-encoded skill sequence) so author renames don't break archive-state lookups.

### Added

- **`orbit routine` verb family** — four new substrate verbs covering the routine lifecycle: `routine detect` (aggregator over SkillInvocation JSONL — reconstructs chains from existing rows via session_id + timestamp ordering, no schema migration), `routine author` (writes `.claude/skills/<name>/SKILL.md` with front-matter per card 0022 + this card's additive `chain_id` and `last_verified` fields), `routine verify` (atomic-write of `last_verified` on a passing resolve-check of every `/orb:<verb>` reference), and `routine list` (status overview of authored routines). CLI subcommands and MCP tools registered for all four. Per spec 2026-05-22-routine-proposals ac-01..ac-09.
- **Audit conformance `routines` finding family** — `orbit audit conformance` emits findings with subsystem slug `routines` and two state slugs: `stale` (`last_verified` >30 days) and `broken_refs` (one or more `/orb:<verb>` references no longer resolves to a live skill). Severity medium; remediation verbs `orbit routine verify <path>` and `archive via curator`. Audit stays read-only — the verify verb is the only writer of `last_verified`.
- **`RoutineFrontMatter` schema** with `created_by`, `created_at`, `pinned` (card 0022 compatibility) plus `chain_id` and `last_verified` (additive to 0022's metadata convention — no 0022 amendment). `chain_id` is SHA-256 hex digest of RFC-8785-canonical-JSON-encoded ordered skill_id sequence (e.g. SHA-256 of `["/orb:tabletop","/orb:spec","/orb:implement"]`) — content-addressed, deterministic across sessions and agents.

### Changed

- **Card 0013 reframed** — playbook-fast-path → skill-orchestration-proposals → routine-proposals across the session. Original framing solved a problem choice 0012 (`design-intent-not-means`) already handled; accepted reframe targets the empty wedge — orchestration of existing skills into named chains — with `/orb:drive`, `/orb:rally`, and `/orb:release` as shipped proof. Maturity bumped planned → emerging on v1 ship.
- **Card 0022 → 0013 back-reference** added to `relations` (closes the AC-08 cross-reference loop; was one-sided).
- **`skill-self-improvement.md`** worked example now uses `/orb:tabletop` (was `/orb:design`); allowlist drops `design`, adds `tabletop`.

### Fixed

- **Stale `/orb:design` references** in canonical SKILL.md files (`tabletop/SKILL.md`'s load-bearing-rule prose and "When to use" bullets) and in card 0039's `ready_for_design` scenarios (corrected to `ready_for_tabletop` + `/orb:tabletop <id>` to match the conformance audit's actual emitted verb).

### Notes

- Substrate gap surfaced + captured at `.orbit/memos/2026-05-22-same-day-nogo-promote-collision.md` — drive's §NO-GO path hits a name collision when `promote.sh` derives the same `<YYYY-MM-DD>-<slug>` spec-id within one session. Cycle-3 inline override avoided the lossy mechanical path; future workflow-refinement spec to fix at the substrate level (memo proposes 3 fix shapes; APPROVE_WITH_CONDITIONS verdict is the leanest).
- ac-10 (`ac_type: observation`) deferred per spec — post-ship drift soak; closure pending a real drift event within 30 days of the first agent-authored routine landing in any orbit-using repo.

## [0.4.30] - 2026-05-22

Sharpens `/orb:prioritise` from a top-5 menu to a single-move recommendation. The default output is one imperative verb plus one sentence on why-now; the full top-5 list moves to an on-request follow-up. Aligns with STYLE.md's "pick one action" rule and makes the layering visibly distinct from `orbit session prime` (raw envelope vs synthesis).

### Changed

- **`/orb:prioritise` output contract** — default output is now one verb + one why-now sentence + a deferred count (≤5 lines), not a top-5 list. The full list (`N. what / why / effort / next`) remains available when the author asks for it on a follow-up turn. The ranking algorithm still applies (severity HIGH > MEDIUM > LOW → memo staleness desc → open-spec age desc → id asc) but now picks one move rather than enumerating five.
- **Why-now framing** — the supporting sentence justifies *timing* (the pattern that promotes this to now), not the item's intrinsic value. Restatement of the item's own stake is called out as the anti-pattern.
- **Card 0043 scenarios** updated to match the new contract (one move + verb verbatim + full list on request + differentiated from `orbit session prime` and `orbit overview`).

## [0.4.29] - 2026-05-21

Ships `/orb:tabletop` as the canonical pre-spec session and retires `/orb:design`. Tabletop replaces design with a stronger contract-not-solution rule enforced at the SKILL.md level: the 10-question methodology (goal narrowing, values, trade-offs, failure modes, lateral approaches, success criteria, escalation triggers, adjacent code, budget, kill conditions) plus a closing hot-wash, with output landing in a per-spec `tabletop.md` sidecar that carries values + trade-offs + halt conditions + escalation triggers + kill conditions + hot-wash. One session can produce one or more specs (multi-card fan-out); closed-mode tabletop ports `/orb:design`'s closed-space design-note path to a `tabletop-note.md` artefact. Retirement was gated on an ambiguity-floor probe (per spec 2026-05-21-tabletop ac-10) — the probe returned GO under the project-bar criterion (tabletop 0.200 vs baseline 0.165; gap was domain-driven in `criteria_clarity`, `constraint_clarity` beat baseline).

### Added

- **`/orb:tabletop` skill** at `plugins/orb/skills/tabletop/SKILL.md` — front-loaded thinking before specs are written. Goal-scoped (one or more cards, or a goal string with agent-inferred cluster + AUQ author confirmation), one-to-many in spec output, multi-card in scope. The 10-question methodology runs in open/partial mode with each question carrying a role line + stop condition; closed mode (when an accepted choice file pins the approach AND prior specs build on the pattern) skips the methodology and produces a one-screen `tabletop-note.md`. Per spec 2026-05-21-tabletop ac-01..ac-09.
- **Per-spec `tabletop.md` sidecar** carrying six sections in declared order (values, trade-offs, halt conditions, escalation triggers, kill conditions, hot-wash). Mirrors the 2026-05-07 dogfood pattern. Closed-mode tabletop is exempt — produces `tabletop-note.md` instead (mutually exclusive per session). No `Spec` schema change; the sidecar lives outside `spec.yaml`. Per choice 0027 (`tabletop-contract-sidecar`).
- **AUQ-prose hybrid pattern** pinned per question — prose opens reframable questions (Q1 goal, Q2 values, Q3 trade-offs, Q4 failure modes, Q5 laterals, Q8 adjacent code); AUQ closes picks (Q6 success criteria, Q7 escalation triggers, Q9 budget with inflation-guard recut applied, Q10 kill conditions). AUQ-refusal fallback: a custom-response reframe returns to the prose phase.
- **Actionable-shape enforcement** for halt conditions (measurable trigger + revert path), escalation triggers (condition + state snapshot + proposed action), and kill conditions (load-bearing claim + named pivot path). Rejected anti-patterns (`"things go wrong"`, `"halt and reassess"`, `"ask Hugh if confused"`, `"I'm stuck"`) listed inline with canonical shapes.
- **MADR choice 0026** (`tabletop-replaces-design`, status `accepted`) — names the v1 design failure mode, alternatives considered (heavyweight tier, multi-card variant only), the chosen full replacement, and the ac-10 probe as the gating mechanism.
- **MADR choice 0027** (`tabletop-contract-sidecar`, status `accepted`) — names the schema-extension and AC-fold alternatives, the chosen sidecar shape, the 2026-05-07 dogfood pattern as precedent, and the per-spec sidecar location convention.

### Changed

- **Conformance audit** now emits `ready_for_tabletop` state slug and `/orb:tabletop <numeric-id>` remediation verb (was `ready_for_design` + `/orb:design <numeric-id>`). The `card has scenarios but no tabletop pass` rationale text updates accordingly. Downstream consumer `/orb:prioritise` updated to surface the new verb.
- **METHOD.md pipeline** updates from `memo → distill → card → design → spec → ...` to `memo → distill → card → tabletop → spec → ...` across all three byte-identical copies (canonical at `orbit-state/crates/core/canonical/METHOD.md`, vendored at `plugins/orb/skills/setup/METHOD.md`, project at `.orbit/METHOD.md`). Narrative `/design` references at the drive description and verb-routing line also updated.
- **`/orb:spec`** now accepts `tabletop-note.md` as the closed-space input artefact (was `design-note.md`). The `verbs.rs` sidecar scan-set in `spec.close`'s topology-warning path updated to match.
- **Plugin skill cascade** — every live `/orb:design` reference in tracked SKILL.md files (`distill`, `discovery`, `prioritise`, `setup`, `review-spec`, `drive`, `keyword-scan`, `card`, `spec`) becomes `/orb:tabletop` or is rewritten for the new shape.
- **Project `CLAUDE.md`** skill list + pipeline narrative updated.
- **README** mermaid flowchart node, skills table, and narrative prose updated to name `/orb:tabletop`.
- **Topology pointer** for the choices subsystem rewritten from `plugins/orb/skills/design/SKILL.md` to `plugins/orb/skills/tabletop/SKILL.md` in `.orbit/topology/choices.yaml`.

### Removed

- **`/orb:design` skill** — `plugins/orb/skills/design/` directory removed. Tabletop is the canonical pre-spec session. Historical artefacts (cards, choices, memos, prior spec records) keep their `/orb:design` references as dated context.

## [0.4.28] - 2026-05-21

Ships richer brownfield reconcile rules so projects with pre-orbit-state spec corpora can run `orbit canonicalise --reconcile` once and reach `orbit verify` clean. Validation against a brownfield corpus migrated all 54 parse-failed specs to canonical shape in a single invocation. Both `canonicalise` envelopes (with and without `--reconcile`) now emit a "run `orbit audit conformance --json`" breadcrumb when residual parse failures remain, routing agents to the structured-findings verb instead of leaving them at raw yaml errors.

### Added

- **Reconcile registry-shape extensions** — two structural extensions on top of the existing `(EntityType, structural_path) → Disposition` shape: a pre-walk synthesise phase that fires before `walk_and_classify`'s keys-iteration loop (so rules can insert missing required keys from filesystem context), and a list-element scope keyed by structural-paths of form `<field>[]` (so scalar list entries can be wrapped before the per-element walk). Two new `Disposition` variants — `Synthesise(label, fn)` and `WrapListElement(label, fn)` — carry their action label inline so the run summary records rule-specific names. Per spec 2026-05-21-richer-reconcile-rules ac-07.
- **Four new reconcile rules.** `synthesise-id-from-filename` derives `Spec.id` from the parent folder's stem when absent. `synthesise-status-default-open` defaults `Spec.status` to `open` when absent (operators correct closed specs after migration). `wrap-scalar-ac` converts bare string `acceptance_criteria[N]` entries into structured ACs with positional id, original string as description, and explicit `gate: false` + `checked: false`. `criterion → description` Map renames the legacy inner field name to the canonical one. Per spec 2026-05-21-richer-reconcile-rules ac-01, ac-02, ac-08, ac-09.
- **Failure-surface breadcrumb on both canonicalise envelopes.** When any file remains in `parse_failed` after the registry's full rule set has applied, both human-readable output and JSON envelope route the agent to the conformance verb. The two hand-rolled JSON envelopes gain an additive optional `next_step` field carrying the breadcrumb string when `parse_failed > 0` and `null` otherwise; existing keys unchanged. Identical text across `run_canonicalise` and `run_reconcile` paths. Per spec 2026-05-21-richer-reconcile-rules ac-04.

### Changed

- **Choice 0023 (`reconcile-as-canonicalise-mode`) extended** with the second-project-trigger consequence: the registry-shape extensions, the four new rule names, the breadcrumb shape. References section now points at `2026-05-21-richer-reconcile-rules/spec.yaml` as the follow-on substrate.

## [0.4.27] - 2026-05-21

Ships `/orb:prioritise` — a session-start priority synthesis skill that re-derives a ranked Decision Brief live from workflow conformance, the session-prime envelope, and recent memories. Read-only by prose contract; deterministic 4-tier ranking (severity → memo staleness → open-spec age → id) gives byte-identical ordering on identical substrate. The agent pays the compression cost so the author doesn't have to. Sharpens `/orb:code-investigate` and `/orb:keyword-scan` to acknowledge the `rtk hook claude` PreToolUse intercept that silently routes `rg` through `grep`, and adds two new discipline bullets to `/orb:code-investigate` for diagnosing tool interception and citing substrate rules with freshness probes.

### Added

- **`/orb:prioritise` skill** — read-only synthesis over `orbit audit conformance --json`, `orbit session prime`, and recent memories. Returns the top 5 priorities, each with what + why + effort (S/M/L) + next-action verb. Deterministic 4-tier ranking (severity `HIGH` > `MEDIUM` > `LOW` → memo staleness days desc → open-spec age desc → id asc). Empty-substrate fallback surfaces planned-empty cards or declares "no priorities" explicitly. Card 0043, spec 2026-05-21-session-start-priority-synthesis.
- **`/orb:code-investigate` gains two new discipline bullets** — *Diagnose before bypassing* (run `type`, `command -V`, `alias`, AND grep `~/.claude/settings.json` for PreToolUse hooks before declaring a tool "shimmed"; hook interception and shell-alias shims feel the same but are different mechanisms) and *Cite the substrate, don't paraphrase* (when applying a project rule, include `file:line` and the exact quoted text; if the rule may have shifted, run `orbit audit conformance --json` first as the freshness probe).

### Changed

- **`/orb:keyword-scan` search-command prose** — drops the "prefer ripgrep for speed" framing. Some environments hook-route `rg` to a grep proxy (e.g. `rtk grep`), so the skill now recommends queries that work in either tool (POSIX ERE alternation, no PCRE2) and points to absolute-path invocation (`/home/linuxbrew/.linuxbrew/bin/rg`) when ripgrep-specific features (PCRE2, `--json`, regex extensions) are required.
- **`/orb:code-investigate` tool taxonomy** — consolidates the previously-separate `ripgrep (rg)` and `rtk-wrapped variants` rows into a single text-search entry; the hook-route caveat is named inline.

## [0.4.26] - 2026-05-20

Adds the `park:` block to the Card schema so cards deliberately held — waiting on a second use-case, a cluster synthesis, or an upstream decision — can opt out of the conformance audit's `ready_for_design` finding without losing the rationale. `audit.conformance` now silently skips parked cards in the card-state family while continuing to fire on genuinely-undesigned ones. Six cards in this repo dogfooded the mechanism in the same release, cutting the local card-state finding count from 9 to 3.

### Added

- **`park:` field on the Card schema** — new optional block with `reason:` (free-form prose, non-empty at parse time) and `until:` (free-form prose, non-empty). When present, conformance's card-state finding family skips the card silently; other finding families (memo staleness, plugin-canonical drift, aggregated drift / topology) continue to fire normally. `Card::FIELDS` extended; `orbit verify` recognises the new field. Per spec 2026-05-20-conformance-park-signal.
- **`/orb:card` SKILL.md gains a "Parking a card" section** — when to use the field, shape of `reason:` / `until:`, scope of the carve-out, how to unpark (remove the block), and the v1 limitation (free-form `until:` only — no automated unpark on date or spec-id resolution).
- **`/orb:setup` SKILL.md §6e gains the parked-card carve-out paragraph** so agents reading conformance findings discover the exclusion from the audit prose itself, cross-linked to `/orb:card`'s authoring side.

### Changed

- **Six cards parked in-repo using the new field**: 0010 / 0011 / 0012 / 0015 (external-execution cluster — awaiting upstream investment decision), 0029-fan-out (awaiting third use-case forcing), 0041-reference-integrity (N=1 hold awaiting second project). Local conformance card-state findings shrink 9 → 3 — only the genuinely-undesigned 0013 / 0014 / 0019 remain in the `ready_for_design` queue.

## [0.4.25] - 2026-05-20

Clarifies the METHOD.md substrate-rules bullet that previously read as an outright ban on `TaskCreate`. New shape separates `orbit task` (cross-session persistence) from `TaskCreate` (in-session working structure), and drops the superseded `TodoWrite` from the prohibition list.

### Changed

- **METHOD.md substrate-rules bullet** — rewritten across all three plugin-canonical copies (plugin source, Rust-vendored canonical, project dogfood). `TaskCreate` is now explicitly permitted for in-session task structure; `orbit task` named as the cross-session persistence boundary (must outlast the session, sync via git, or attach to a spec). `TodoWrite` removed entirely — superseded by `TaskCreate`, not a substantive failure mode. Resolves agent friction observed in recent sessions where the harness's `TaskCreate` nudges were refused per the literal rule reading.

## [0.4.24] - 2026-05-20

Ships STYLE.md as a plugin-canonical file with the same lifecycle as METHOD.md (plugin source → Rust-vendored canonical → `include_str!` → `/orb:setup` writes seed → conformance audit byte-compares). The reworked prose discipline reaches every consumer project on next `/orb:setup`. Drops the BLUF / Decision Brief framing from METHOD.md and the plugin SKILL.md cascade — STYLE.md is now the single canonical source for agent prose discipline. Renames card 0026 (`executive-communication` → `agent-prose-discipline`) and METHOD.md pillar #1 (`Executive-level interaction` → `Author-level interaction`) to retire the "executive" framing.

### Added

- **STYLE.md plugin-canonical pipeline** — `plugins/orb/skills/setup/STYLE.md` (canonical source) vendored to `orbit-state/crates/core/canonical/STYLE.md` (cross-compile docker mount); embedded via `include_str!` in `verbs.rs`'s `CANONICAL_FILES` const alongside METHOD.md. Conformance audit auto-extends (single loop drives both byte-compares); `pin_behind` / `pin_ahead` suppress per-file findings for both. Three new tests: `conformance_byte_drift_fires_when_style_md_differs`, `conformance_missing_fires_when_style_md_absent`, `conformance_vendored_style_md_matches_plugin`.
- **`/orb:setup` §6 covers STYLE.md** — §6b copies both METHOD.md and STYLE.md with per-file byte-compare-and-prompt-on-drift; §6c appends both `@.orbit/METHOD.md` and `@.orbit/STYLE.md` to CLAUDE.md (idempotent). `setup-method.sh` refactored to function-based per-file copy/import; new `--canonical-style` and `--answer-style-drift` flags (with `--canonical` and `--answer-drift` preserved as aliases). Test script extended: scenario 1 gains t4/t5/t6 (STYLE.md fresh-project), scenario 2 gains t2 (STYLE.md drift), scenario 3a verifies STYLE.md migration; 18/18 assertions pass.
- **`/orb:release` syncs STYLE.md too** — §5 pre-flight sync step now `cp`s both canonical files to their vendored copies. Sync-check unit tests catch divergence at build time.

### Changed

- **STYLE.md reworked** — from BLUF / Decision Brief skeleton (5-section template with TL;DR / Recommendation / Why / Detail / Confidence labels) to plain prose discipline. Substance rules retained (lead with the answer, imperative voice, no menus, the seven anti-patterns); format prescription dropped. Project-specific stance (persona) now lives in each project's CLAUDE.md as a Persona section.
- **METHOD.md prose contract section dropped** — "## Prose contract — BLUF / Decision Brief" (BLUF skeleton, anti-patterns, tone) replaced with a one-line pointer to `.orbit/STYLE.md` across all three copies (plugin canonical, Rust-vendored, local). STYLE.md is the single source for prose discipline; METHOD.md carries the workflow overview only.
- **METHOD.md pillar #1 renamed** — `Executive-level interaction` → `Author-level interaction`. Same body, retired "executive" framing.
- **Card 0026 renamed** — `executive-communication` → `agent-prose-discipline`. Filename, internal `id`, reference notes all updated; scenarios reworked to match the new prose contract.
- **Plugin SKILL.md cascade** — `design`, `review-spec`, `review-pr`, `setup` citations of *"BLUF / Decision Brief — see card 0026 (`.orbit/cards/0026-executive-communication.yaml`)"* replaced with *"Agent prose follows the discipline in `.orbit/STYLE.md` (see also card 0026 — `.orbit/cards/0026-agent-prose-discipline.yaml`)"*. `drive`'s halt-temptation guard prose updated likewise (drops the "Decision Brief frame" and "anti-pattern #4" specific references while preserving the semantic).

## [0.4.23] - 2026-05-20

Ships the agent-side substrate-engagement rally end-to-end — three cards delivered together through `/orb:rally`'s proposal → queued design → consolidated decision gate → consolidated design review → stacked PR shape. Each card targets a distinct lifecycle moment where the agent must engage with persistent substrate: design-time memory matching (0037), skill-entry resolution (0038), and halt-temptation discipline mid-autonomy (0042).

### Added

- **`memory.match` verb** — ranks memories against a topic + optional labels using a token-overlap (body) + label-overlap signal, weighted 2× toward labels and normalised to `[0, 1]`. Returns `{memory, score, reason}` per match. Powers two new gates: `/orb:design` §2 calls `orbit memory match <card-slug>` at evidence-load time, and `spec.close` now blocks when matching memories aren't reconciled (threshold 0.3, `--force` bypass mirroring the existing AC pre-flight).
- **`memories_considered` on `Spec`** — new `Vec<MemoryReconciliation>` field with `Adopted | PartiallyAdopted | NotApplicable` disposition + reason. Skip-on-default keeps existing specs byte-identical.
- **`memory.remember` shape warning** — soft warning fired when a body leads with state-shape language ("X is hard") instead of mechanism ("use X when Y"). Never blocks; `--no-warn` flag suppresses. Mirrors the topology-label nudge precedent.
- **`spec.resolve` verb** — unifies the infer → prompt → halt recovery for skills that take a spec-id argument. Returns `{outcome: resolved, id}` when bound-card has a single open spec, `{outcome: prompt, candidates: [...]}` for the multi-spec case, or `Error::unavailable` with two byte-identical halt templates (terminal vs recoverable) when both fallbacks fail.
- **PreToolUse hook on `AskUserQuestion`** — `plugins/orb/hooks/three-question-test.sh` scope-gated to `ORBIT_NONINTERACTIVE=1` AND `drive.yaml` presence. Prints the three substrate-typed questions (recommendation? evidence? authorisation?) to stderr and exits non-zero to suppress the halt under autonomy. The agent does the stage-match; the hook prompts.

### Changed

- **Five skills (`/orb:implement`, `/orb:review-pr`, `/orb:review-spec`, `/orb:audit`, `/orb:drive`) now call `orbit spec resolve --skill <name>`** instead of carrying divergent input-contract branches. Single canonical recovery; uniform halt messages.
- **`drive/SKILL.md` adds a §"Halt-temptation guard"** documenting the three-question test, the inverse failure mode (acting on substrate authorisation rather than halting), and the stage-scope rule for pre-commit halts. §1.6 prefaces the cycle-budget rule with the severity-is-reviewer-language clarification.
- **`STYLE.md` adds a §"Closing recommendations vs in-flight decisions"** — the Decision Brief shape closes recommendations to the operator; the imperative single-action form is correct mid-autonomy. Card 0026 carries a matching scenario.
- **`rally/SKILL.md` and `implement/SKILL.md` carry pointers** to the halt-temptation guard so rally sub-agents and implementing skills share the discipline.

### Fixed

- Three cards (0037 memory-gates-decisions, 0038 skills-infer-or-prompt-before-halt, 0042 act-when-authorised) bumped from maturity `planned` → `emerging` to reflect shipped specs.

## [0.4.22] - 2026-05-19

Fixes the 0.4.21 cross-compile failure. The `audit.conformance` verb's `include_str!` of `plugins/orb/skills/setup/METHOD.md` reached outside `cross`'s docker mount during aarch64-linux builds. Vendored the canonical bytes into `orbit-state/crates/core/canonical/METHOD.md` and added a /orb:release pre-flight step to keep the vendored copy in sync with the plugin source. No behavioural changes vs the (un-released) 0.4.21 — same 8 ACs of spec `2026-05-19-workflow-conformance`.

### Changed

- `audit.conformance` reads canonical METHOD.md bytes from a vendored copy at `orbit-state/crates/core/canonical/METHOD.md` rather than the plugin source path. New unit test `conformance_vendored_method_md_matches_plugin` is the local drift detector.
- /orb:release §1.5 adds the vendored-METHOD.md sync as a pre-flight step when substrate changed in the release window.

## [0.4.21] - 2026-05-19

`orbit audit conformance` lands as the agent-on-demand workflow conformance audit. The verb aggregates the existing `audit.drift` + `audit.topology` results under `aggregated.{drift,topology}` and surfaces three new finding families — plugin-canonical-file drift (`.orbit/METHOD.md` byte-compare against compile-time canonical bytes), card-state (cards at `maturity:planned` with empty specs, ready for design), memo staleness (filename-date > 7 days) — alongside a plugin-version pin state derived from `.orbit/config.yaml`. Each finding carries an explicit `remediation.verb` the agent runs without translation (`orbit setup`, `/orb:design <id>`, `/orb:distill <path>`). Audience is agent-first: zero-finding case is silent; operator sees output only on agent escalation. Ships spec `2026-05-19-workflow-conformance` end-to-end (8/8 ACs, card 0039 maturity planned → emerging).

### Added

- `ConformanceFinding` / `Remediation` / `AggregatedAudits` / `PinState` types in `orbit-state` (deny-unknown-fields, severity / state slug strings for forward-compat).
- `audit_conformance` Rust verb (CLI + MCP) — `orbit audit conformance [--json]`. Read-only, idempotent. Composes existing audits rather than subsuming them — `audit drift` and `audit topology` retain their independent surfaces.
- `Config.plugin_version: Option<String>` field — per-repo pin storage in `.orbit/config.yaml`. `Config::FIELDS` extended so the canonicaliser recognises the key. `None` defaults to current (orbit-state binary's `CARGO_PKG_VERSION` under lockstep release).
- Pin-state model: `unpinned` / `matches` / `pin_behind` / `pin_ahead`. `pin_behind` (installed > pinned) and `pin_ahead` (installed < pinned) each fire ONE dominant finding and suppress per-file findings (single-finding dominance).
- `Layout::list_memo_files()` + `Layout::repo_root()` helpers in `orbit-state-core`.
- `CANONICAL_FILES` v1 inventory: `.orbit/METHOD.md` is the single plugin-canonical file in v1; the inventory expands as the plugin adds more (no spec change required).
- 21 unit tests covering schema round-trip, deny-unknown-fields, card-state matrix, memo-staleness boundary cases, byte-drift matrix, pin-state derivation, pin-state suppression, aggregation byte-equality, and dispatch.
- CLI + MCP parity tests on the clean-fixture envelope.
- /orb:setup SKILL.md §6e + METHOD.md substrate-rules line documenting the verb for agent / operator discovery.

### Changed

- `time` workspace feature set extended with `parsing` + `local-offset` (memo filename-date parsing + `OffsetDateTime::now_local`).
- Card 0039 slug renamed `setup-conformance-check` → `workflow-conformance`; scope reframed from artefact byte-compare to substrate-vs-plugin-contract evidence.

## [0.4.20] - 2026-05-18

Topology moves out of the documentation tree and into the agent substrate. Per choice 0025 (`topology-substrate-folder`), the canonical shape is per-subsystem yaml at `.orbit/topology/<subsystem>.yaml` parsed against the new `TopologyEntry` schema — pointer-only, agent-queryable, prunable with `rm`. The substrate-engagement envelopes shipped in 0.4.19 (`session.prime` `topology_drift`, `spec.close` `topology_warnings`, `memory.remember` `--label topology` nudge) are preserved against the new parser, so consumer-facing envelope shapes don't break. Ships spec `2026-05-18-topology-substrate-migration` end-to-end (5/5 ACs, card 0040 maturity planned → emerging).

### Added

- `TopologyEntry` schema in `orbit-state` (deny-unknown-fields, slug-shape and ≥ 5-char validation, non-empty `canonical_code` required); list-valued pointer fields for `canonical_code` / `decision_record` / `operational_doc` / `test_surface`.
- `Layout::topology_dir()` / `topology_file(slug)` / `list_topology_files()` mirroring the existing card/choice/memory scanner shape.
- `verify_all` extends with a topology-entry branch — round-trip + non-serde validate per file.
- `orbit topology setup` Rust verb (CLI + MCP): scaffolds `.orbit/topology/` with a self-describing seed (one entry per `.orbit/` entity type — `cards`, `choices`, `specs-substrate`, `memories`, `topology` itself), opportunistically strips legacy `docs.topology` from `.orbit/config.yaml`, idempotent on re-runs. Replaces `plugins/orb/scripts/setup-topology.sh` per choice 0020.
- Choice 0025 (`topology-substrate-folder`) — MADR for the architectural shift.

### Changed

- `audit_topology` parser swap — markdown header/list scanning → per-file yaml parse via `schema::TopologyEntry`. Drift codes: `stale_pointer` / `missing_entry` preserved for envelope continuity; `invalid_field` / `parse_failed` added for structural failures.
- `audit_topology(...).configured` canonical predicate: `.orbit/topology/` exists AND contains ≥ 1 entry (populated == configured per the design pass's UX call).
- `compute_topology_warnings` rewired to load subsystem names directly from `.orbit/topology/` entries.
- `/orb:topology` skill rewritten for per-file editing (write/read/audit modes operate on `.orbit/topology/<subsystem>.yaml`).
- `/orb:setup` §6d invokes `orbit topology setup` (Rust verb, not bash script).
- `/orb:distill` and `/orb:release` topology gates re-anchored to ".orbit/topology/ exists and is populated".

### Fixed

- `promote.sh` was checking the obsolete flat spec path (`.orbit/specs/<id>.yaml`) and silently bailing before writing acceptance_criteria; now uses the folder-sidecar path (`.orbit/specs/<id>/spec.yaml`).

### Deprecated

- `DocsConfig::topology` field — retained as parse-only so brownfield consumer repos that wired topology under 0.4.19 do not hard-fail `Config::from_str` on session prime. The canonical writer preserves the field; `orbit topology setup` strips it from on-disk configs. A follow-on spec deletes the field entirely after consumer-repo soak.

### Removed

- Legacy markdown parser in `audit_topology` (`parse_topology_doc` and `load_topology_subsystem_names` helpers).
- `plugins/orb/scripts/setup-topology.sh` and its bash test (replaced by `orbit topology setup` Rust verb + CLI/MCP parity tests).

## [0.4.19] - 2026-05-18

Topology capability becomes self-maintaining. The substrate shipped in 0.4.18's parent spec (`/orb:topology` skill + `Config` schema + `orbit audit topology` verb) now reaches into the moments where it earns its keep — `/orb:setup` scaffolds the config + stub on first project boot, `orbit session prime` surfaces topology drift in its envelope at session start, `orbit spec close` flags warnings when the closing spec text touches a documented subsystem, and `orbit memory remember --label topology` nudges toward `/orb:topology` so the index gets updated at the learning moment rather than at the next gate. Closes 4 of 5 ACs in spec `2026-05-18-topology-substrate-wires` (card 0040); ac-05 is the 2026-06-15 4-week observation audit, calendar-deferred per the ac-taxonomy observation band.

### Added

- `orbit session prime` envelope gains `topology_drift: Option<Vec<TopologyDriftEntry>>`. `Some` whenever `audit_topology(...).configured == true` (`Some([])` on configured-clean), `None` (key omitted) when not configured. Skip-on-default contract: the key is absent both when `.orbit/config.yaml` is missing and when the file exists but `docs.topology` is unset.
- `orbit spec close` envelope gains `topology_warnings: Vec<TopologyDriftEntry>` (`skip_serializing_if = Vec::is_empty`). Scans the closing spec's `spec.yaml + interview.md + design-note.md` (each when present); case-insensitive `\b<regex::escape(subsystem)>\b` match against topology entries with names ≥ 5 characters. Non-blocking — closure proceeds with exit 0; warnings are advisory. `design-note.md` is in the scan set because it routinely names subsystems by canonical handle.
- `orbit memory remember --no-nudge` flag + `MemoryRememberResult.nudge: Option<String>` envelope field. When the labels include `topology` and the flag is absent, the nudge fires on the structured envelope (MCP) and on stderr (CLI human mode). Stdout is reserved for the primary verb output.
- `plugins/orb/scripts/setup-topology.sh` — `/orb:setup` §6d scaffolding. Greenfield path scaffolds `.orbit/config.yaml` + a stub `docs/topology.md` (heading + explainer + empty entry list). Brownfield path prompts before writing; brownfield-accept creates the parent directory tree when the target path nests in non-existent dirs; never overwrites an existing target file. Test affordance `--answer-wire y|n` scripts the prompt.
- `plugins/orb/scripts/tests/test-setup-topology.sh` — exercises the six fixture states for the setup script (greenfield-accept, brownfield-decline, brownfield-accept with/without existing target, nested target path, idempotent re-run).

### Changed

- `plugins/orb/skills/setup/SKILL.md` gains §6d documenting the topology scaffolding step in the byte-compare-and-prompt voice of §6b. Topology scaffolding runs after §6a–§6c but is independent of them — it neither reads nor writes CLAUDE.md / METHOD.md.
- `SessionPrimeResult`, `SpecCloseResult`, and `MemoryRememberResult` extended with the new envelope fields. All additions use `skip_serializing_if` so happy-path responses remain byte-identical to the pre-change shape for callers that don't use the new features. `MemoryRememberArgs` gains `no_nudge: bool` (defaulted false; mirrors `--no-edit` / `--no-verify` naming convention).
- `TopologyDriftEntry` is the shared cross-verb entry shape. `session_prime` and `spec_close` import from `audit_topology`'s definition — no redeclaration.

## [0.4.18] - 2026-05-18

Codebase mastery becomes operable. `/orb:code-investigate` ships as the agent-equipment surface delivering card 0025 — narrow mode for specific queries (where is X, what calls Y, how many Z), broad mode for neighbourhood awareness, with a tool taxonomy that routes structural queries to ast-grep and tree-sitter, text searches to ripgrep, and command output through rtk-wrapped variants by default. A PreToolUse hook shipped at `plugins/orb/hooks/hooks.json` softly warns when Edit/Write hits an uninvestigated path; the warning is non-blocking and grep-stable so the 4-week audit (ac-07, observation band) can count fires across sessions. Call-points embedded in `/orb:implement`, `/orb:researcher`, and `/orb:review-pr` with imperative voice make the discipline a default reach rather than an optional one. Closes 6 of 7 ACs in spec `2026-05-17-code-investigate-skill` (card 0025); ac-07 defers per the ac-taxonomy observation band.

### Added

- `plugins/orb/skills/code-investigate/SKILL.md` — narrow + broad modes, tool taxonomy (ast-grep / tree-sitter / ripgrep / rtk / Read), discipline rules, heuristic closing instruction for memories labelled `code-investigate`. Front-matter follows card 0022's curator convention (`name`, `description`, `created_by`, `created_at`, `pinned`).
- `plugins/orb/hooks/hooks.json` — declares a PreToolUse hook on the `Edit|Write` matcher, pointing at `${CLAUDE_PLUGIN_ROOT}/hooks/code-investigate-nudge.sh`. The Claude Code plugin loader picks this up on plugin enable; no consumer-side `.claude/settings.json` edit required.
- `plugins/orb/hooks/code-investigate-nudge.sh` — non-blocking soft nudge. Path filter skips `.orbit/`, `.claude/`, and `*.lock`; graceful degradation skips silently when `.orbit/` is absent (non-orbit repos that happen to load the plugin) and warns when the marker is missing or session-stale.
- `plugins/orb/scripts/code-investigate-mark.sh` — atomic-write marker at `.orbit/.code-investigate-recent` with a session-id header line plus tab-delimited `(timestamp, kind, path)` entries. Preserves prior entries when the session-id matches; treats stale-session markers as empty.
- Memory label convention `code-investigate` documented in `.orbit/METHOD.md` — the canonical label that `/orb:code-investigate` pivots on for the learning loop.

### Changed

- `/orb:implement`, `/orb:researcher`, `/orb:review-pr` gain imperative-voice call-points naming `/orb:code-investigate` (broad mode for implement/researcher, narrow mode for review-pr) with a one-line rationale tied to the agent-equipment framing.
- Card 0025 (codebase-mastery) reframed — `i_want` and `goal` name `/orb:code-investigate` as the delivery vehicle; scenario 6 names the skill as the principle-router; references list adds the spec path. Maturity stays `planned` until the 4-week observation window confirms the capability is being used in practice.
- `.orbit/.gitignore` adds `.code-investigate-recent` alongside the existing session-state files.

## [0.4.17] - 2026-05-16

Per-card session handover. The `Session` entity gains an optional `card_id`; new verbs `orbit session set-card` and `orbit session handover` let agents bind a session to a card and retrieve the latest matching session; `orbit session prime` now surfaces the latest handover at session start. `orbit session distill` learns to extract `last_assistant_message` from the Claude Code Stop-hook JSON envelope — the carrier was previously dumping raw JSON into `Session.distillate`. Closes 10 of 11 ACs in spec `2026-05-16-session-handover` (card 0036). Schema bumps 0.3 → 0.4 (additive no-op, chains after the 0.2 → 0.3 ac-taxonomy migration).

### Added

- `card_id: Option<String>` field on `Session` (orbit-state/crates/core/src/schema.rs). `#[serde(default, skip_serializing_if = "Option::is_none")]` keeps canonical output byte-identical for sessions without a card binding.
- `orbit session set-card <id>` CLI + MCP verb — writes `.orbit/.session-card` atomically; validates `<id>` via the existing card-lookup helper. `orbit session distill --card <id>` and `orbit session handover --card <id>` both fall back to this file when the flag is absent.
- `orbit session handover [--card <id>] [--since <iso>]` CLI + MCP verb — returns the most-recent matching `Session` across the substrate.
- `handover` field on the `orbit session prime` envelope — carries the most-recent `Session` across all cards. Per-card lookup remains the explicit `handover` verb; prime surfaces only the global latest.
- `0.3 → 0.4` schema migration step — additive no-op (adds the optional `card_id` field); chains after the 0.2 → 0.3 ac-taxonomy migration on the same `ensure_current` walk.
- `.orbit/choices/0024-handover-register-is-discursive.yaml` MADR — records the deliberate divergence from STYLE.md's BLUF discipline for handover prose. Audience and purpose differ: the handover is reading-for-orientation prose written for the next agent, not a decision brief written for Hugh.
- `.orbit/conventions/session-handover.md` — agent-facing discipline (call `set-card` early, write a discursive reflection covering what was tried + what worked + what didn't + where to pick up).

### Changed

- `orbit session distill` reads stdin and detects a Claude Code Stop-hook JSON envelope; when present, extracts `last_assistant_message` and stores that as `Session.distillate`. Plain-text stdin is preserved verbatim — the JSON-envelope path is opt-in by structural detection, not by flag. Fixes the load-bearing carrier bug where the entire envelope was being stored as distillate prose.
- `.claude/settings.json` Stop hook updates so both `.session-id` and `.session-card` are deleted after `orbit session distill` completes. The `(... 2>/dev/null && rm -f ...) || true` invariant is preserved — distill failures still fall through harmlessly to keep Stop unblocked.
- `CURRENT_SCHEMA_VERSION` bumps `0.3` → `0.4`. The schema-version file step is idempotent on already-migrated trees.

### Binary state (substrate-binary parity gate)

`orbit-state/` changed in this window (`Session.card_id` field, `0.3 → 0.4` migration, `session_set_card` / `session_handover` verbs, distill JSON-envelope extraction, prime envelope `handover` wiring). Released via the gate's path (a) — rebuild and reinstall the orbit binary at 0.4.17 before tagging so substrate and skills ship in lockstep.

### Deferred to follow-up

- ac-11 of spec `2026-05-16-session-handover` (end-to-end ops-band smoke against the released brew binary on the Beelink). Defers `spec.close` per the `ac_type: ops` band — to be executed after this release's brew formula updates.

## [0.4.16] - 2026-05-16

Typed acceptance criteria: `AcceptanceCriterion` now carries a five-value `ac_type` enum (`code` / `config` / `doc` / `ops` / `observation`) declaring what kind of evidence closes the AC. `spec.close` honours the type via a two-band rule — `code` / `config` / `doc` block close when unchecked; `ops` / `observation` legitimately defer. Closes most of spec `2026-05-16-ac-taxonomy` (card 0035-ac-taxonomy, plus card 0034-spec-close-ac-preflight superseded, card 0030-canonical-schema-and-glossary extended, card 0032-brownfield-spec-migration extended). Forward-incompatible: a 0.4.15 binary scanning a 0.4.16 spec.yaml that carries `ac_type` rejects the unknown field via `deny_unknown_fields`.

### Added

- `AcType` enum on `AcceptanceCriterion` (`Code` default, `Config`, `Doc`, `Ops`, `Observation`) with `blocks_close()` as the single source of truth for the two-band split. `ac_type` field on `AcceptanceCriterion` with `#[serde(default)]` so untyped legacy corpora deserialise as `Code`; `skip_serializing_if = "AcType::is_code"` keeps existing canonical output byte-identical for the dominant case.
- `Disposition::Transform(TransformFn)` variant on the reconcile registry, with `TransformResult::{ Replace { value, sibling_writes, detail }, Quarantine(reason) }`. Lets reconcile rules rewrite a field's VALUE (not just its name) and atomically set sibling fields. Used by the typed-AC handler to split brownfield `ac_type: gate` into orthogonal `ac_type: code|observation` + `gate: true`. `DispositionRecord` gains an optional `transform_detail: Option<String>` surfacing the per-AC routing rationale in the run summary and JSON envelope.
- `migrations::ensure_current(layout)` — initialises the schema-version file if missing AND advances it through any pending migrations to `CURRENT_SCHEMA_VERSION`. `verify_all` calls it so substrate migrations auto-apply on the next orbit verb against an older tree.
- `0.2 → 0.3` schema migration step `migrate_time_gated_to_ac_type` — walks every spec.yaml, rewrites `time_gated: true` → `ac_type: observation` (legacy key removed), drops `time_gated: false` (default `code` is implicit). Idempotent on already-migrated trees.

### Changed

- `spec.close`'s unchecked-blocking computation switches from `!ac.checked && !ac.time_gated` to `!ac.checked && ac.ac_type.blocks_close()`. Response envelope field `time_gated_open` renamed to `deferrable_open` across CLI / MCP / parity tests / drive SKILL.md. Error wording adjusts from "unchecked AC(s) in spec" to "unchecked blocking AC(s) in spec" so the deferrable distinction surfaces in the error itself. `--force` path unchanged.
- `canonicalise --reconcile` registry — the seeded `Disposition::Drop` entry for `acceptance_criteria[].ac_type` is REPLACED with `Disposition::Transform(reconcile_ac_type)` routing brownfield values: `docs` → `doc` (typo); canonical values pass through (no-op recorded); `ac_type: gate` splits via description regex into `code + gate=true` (build/cargo/cmake/make/compile keywords) or `observation + gate=true` (eval/score/accuracy/training/metric keywords); unknown values quarantine with a reason.
- `CURRENT_SCHEMA_VERSION` bumps `0.2` → `0.3`.
- `/orb:design` SKILL.md §6 + §4: agent prompts the author for `ac_type` per AC. `/orb:review-pr` SKILL.md: per-AC evidence-expectation table BEFORE the AC-walk, with explicit "ACs of `ac_type: ops` or `observation` MUST NOT be flagged as missing test evidence" rule. `/orb:drive` SKILL.md: AC routing by `ac_type` in Stage 2 (Implement) — `ops` escalates to operator-handoff with memo path; `observation` registers as deferred-checkpoint via `deferrable_open`. `.orbit/METHOD.md` (and the byte-equal mirror at `plugins/orb/skills/setup/METHOD.md`) gain an Acceptance-criterion `ac_type` sub-section under Vocabulary.

### Removed

- `time_gated: bool` field on `AcceptanceCriterion`. Superseded by `ac_type: observation` (the kind that captures the deferrable-at-close semantics the bool encoded). The `0.2 → 0.3` migration rewrites every existing `time_gated: true` AC to `ac_type: observation`.
- `Disposition::Drop` entry for `acceptance_criteria[].ac_type` in the reconcile registry — replaced with `Disposition::Transform`.

### Binary state (substrate-binary parity gate)

`orbit-state/` changed in this window (`AcType` enum, `Disposition::Transform` variant, `0.2 → 0.3` schema migration, `ensure_current` wire). Released via the gate's path (a) — rebuild and reinstall the orbit binary at 0.4.16 before tagging so substrate and skills ship in lockstep. The skill's parity-gate refusal mode was bypassed manually because §1.4 has a chicken-and-egg for first-substrate-bump releases (PATH binary by definition predates the new version when both are bumped together).

### Deferred to follow-up

- ac-12 of spec `2026-05-16-ac-taxonomy` (pre-release brownfield dry-run against the public corpora) is force-closed at spec.close. The Transform handler's correctness is proven by 11 unit tests; end-to-end validation against richer brownfield drift (missing top-level `id`, scalar AC entries, `orbit/` vs `.orbit/` substrate layout) needs reconcile-mode v3 rules that are out of scope. See `.orbit/memos/2026-05-16-richer-reconcile-rules.md` for distillation.

## [0.4.15] - 2026-05-16

Memos relocate from `.orbit/cards/memos/` to a sibling `.orbit/memos/` directory. Closes spec `2026-05-16-memos-own-folder` (cards 0001-memos, 0008-consolidated-orbit-artefact-folder). Substrate ontology now reads correctly — memos are inputs *to* cards, not part of cards. Forward-incompatible: a 0.4.14 binary scanning a 0.4.15 layout sees zero memos at the legacy path.

### Changed

- `OrbitLayout::memos_dir()` returns `<root>/memos` (was `<root>/cards/memos`). The `list_yaml_files_shallow` wrapper at `orbit-state/crates/core/src/layout.rs` is removed as dead code; `list_card_files` calls `list_yaml_files` directly.
- All skills (`/orb:memo`, `/orb:distill`, `/orb:design`, `/orb:rally`, `/orb:implement`) and the `setup/METHOD.md` vocabulary table reference `.orbit/memos/`.
- 23 cards, 5 live spec files, choice 0021-spec-folders, and 2 memories had their `.orbit/cards/memos/` references rewritten to `.orbit/memos/` for substrate consistency. Migration commit is the audit record per the 2026-04-20 precedent.

### Binary state (substrate-binary parity gate)

`orbit-state/` changed in this window (the `memos_dir()` path + dead-code wrapper removal). Released via the gate's path (a) — rebuild and reinstall the orbit binary at 0.4.15 before tagging so substrate and skills ship in lockstep.

## [0.4.14] - 2026-05-16

Agent learning loop v1 ships — skills can record per-invocation outcomes and read recurrence; sessions get a canonical `Session` entity and an idempotent distill verb. Closes spec `2026-05-15-agent-learning-loop` (cards 0022-skill-curator, 0023-memory-loop). Card 0023 bumps `planned → emerging` because `orbit session prime` now scores memories by label-overlap with open-spec labels before recency — the "recent and relevant" gate is real. Card 0022 stays `planned`: the convention is an explicit stopgap until the curator-metadata system ships.

### Binary state (substrate-binary parity gate)

`orbit-state/` changed in this window (3 commits add four CLI/MCP verbs). Released via the gate's path (c) — forward-compatible:

- The new verbs (`session.start`, `session.distill`, `skill.record-invocation`, `skill.recurrence`) are purely additive — no existing verb signature changed, no entity-schema break.
- The Stop hook in `.claude/settings.json` is wrapped `(orbit session distill 2>/dev/null && rm -f .orbit/.session-id) || true`, so a pre-0.4.14 binary on PATH degrades silently rather than erroring.
- `orbit verify` against the new substrate is unchanged — it doesn't reach for the new verbs.

Users on the brewed `orbit` will see the new skill prose (and the Stop hook will no-op on session end) until they rebuild/reinstall the binary at 0.4.14+. Until then, the new verbs are reachable via the in-repo release binary (`orbit-state/target/release/orbit`).

### Added

- `orbit skill record-invocation <skill_id> --outcome <enum> [--correction <str>]` — appends one row to `.orbit/skills/<skill_id>.invocations.jsonl`. The `outcome` arg accepts `worked`, `partial`, `didnt-apply`, `incorrect`; any other value returns a malformed error naming the accepted set. `correction` is optional free text. `session_id` is sourced from `ORBIT_SESSION_ID` env, falling back to `.orbit/.session-id`, falling back to an unavailable error naming both sources.
- `orbit skill recurrence <skill_id> [--since <iso-date>]` — reads the per-skill invocation stream and returns per-outcome counts with the recorded `correction` entries. Every outcome key (`worked`, `partial`, `didnt-apply`, `incorrect`) is always present in the response even when count is 0, so agents can index without first checking for missing keys. Returns the empty shape (total=0) when the file is absent.
- `orbit session start [--id <uuid>]` — generates a UUIDv4 (or uses the supplied id) and writes it to `.orbit/.session-id` atomically. Idempotent on re-run: a new UUID overwrites, which is the fresh-session semantics.
- `orbit session distill` — idempotent CLI/MCP verb that writes or updates `.orbit/sessions/<session-id>.yaml`. CLI reads the distillate from stdin (or `--from <path>`); MCP takes a required `distillate` arg. Session_id precedence is `--session-id` > `ORBIT_SESSION_ID` > `.orbit/.session-id`. First call sets `started_at = ended_at = now`; subsequent calls preserve `started_at` and advance `ended_at`.
- `Session` canonical YAML entity at `.orbit/sessions/<session-id>.yaml` — substrate-written, round-trippable, schema-version bumped 0.1 → 0.2. The migration is structurally a no-op (additive) and the runner rejects unknown versions with `malformed` rather than guessing.
- `SkillInvocation` event struct + `InvocationOutcome` kebab-case enum in `schema.rs`, alongside the existing `TaskEvent` / `NoteEvent` family. Append-only JSONL, excluded from the CI round-trip gate (events aren't round-trippable as a unit).
- `.orbit/conventions/skill-self-improvement.md` — codifies the v1 rules for agent-judgment live edits to `SKILL.md`: ≥2 same-skill-same-outcome recurrence threshold, one-off failures route to `orbit memory remember`, named allowlist of editable skills (`card`, `design`, `discovery`, `implement`, `review-spec`, `spec`), and the worked example showing read-recurrence → reason-from-corrections → edit-SKILL.md flow. The convention names card 0022 as the future home of metadata-based enforcement.
- `read_session_id` shared helper (`orbit-state/crates/core/src/session.rs`) — the canonical sourcing precedence consumed by every session-scoped verb. New module so future session-scoped verbs reach for one entry point.
- `.orbit/.session-id` added to `.orbit/.gitignore` (transient per-clone state).
- 40 new unit tests and 8 new CLI+MCP parity tests covering the new verbs end-to-end.

### Changed

- `orbit session prime` ranks memories by label-overlap with open-spec labels (descending) before timestamp DESC. When no open spec has labels, behaviour is unchanged. The cap, `item_bound`, and `next_step` text are unchanged.
- `.claude/settings.json` wires the loop into Claude Code: `SessionStart` chains `orbit session start` + `orbit session prime`; `Stop` runs `orbit session distill` then removes `.orbit/.session-id`; `PreCompact` runs `orbit session prime`. Stale `bd prime` references retired. The Stop hook is wrapped to degrade gracefully when the verbs aren't on PATH yet (see Binary state above).

### Fixed

- `/orb:release` skill no longer pins `model: sonnet` in its frontmatter — the pin forced 1M-context Sonnet, which requires extra-usage enablement. The skill now inherits the invoking session's model.

## [0.4.13] - 2026-05-14

`orbit spec close` gains an AC pre-flight: it refuses to flip a spec to closed while any non-time-gated AC remains `checked: false`, mirroring the existing unfinished-tasks guard. A new `time_gated: bool` field on `AcceptanceCriterion` carves out ACs that are legitimately expected to remain unchecked at close (post-deploy observation windows, operator sign-off awaiting calendar) so they don't have to be force-closed. Closes spec `2026-05-13-spec-close-ac-preflight` (card 0034). Dogfooded on its own delivery: the spec's release-smoke AC (ac-09) is itself `time_gated: true`, so this spec closed via the new path rather than `--force`.

The change is forward-compatible: every existing spec parses unchanged via `#[serde(default)]`, no canonical-output churn (skip-if-false on serialise keeps existing AC YAML byte-identical), and the close-response gains optional fields with `skip_serializing_if = "Vec::is_empty"` so happy-path responses are byte-identical to the previous shape.

### Added

- `AcceptanceCriterion.time_gated: bool` (default `false`) — declares an AC as legitimately deferred at close. `orbit spec close` excludes these from the unchecked-blocking set and reports them in the structured response under `time_gated_open`.
- `orbit spec close --force` — deliberate opt-in that bypasses the AC pre-flight when ACs are genuinely unfinished and the close is intentional (review NO-GO, scoped deferral). Bypassed AC ids surface in the response's `forced_unchecked` field, so the audit trail lives in the substrate, not only in shell history. The flag does not bypass the unfinished-tasks guard or the already-closed guard.
- `SpecCloseResult` gains `forced_unchecked: Vec<String>` and `time_gated_open: Vec<String>`, both `skip_serializing_if = "Vec::is_empty"`. The struct intentionally lacks `deny_unknown_fields`, preserving forward-additive read compatibility for callers that cache an older response shape.
- Card `0034-spec-close-ac-preflight` — capability description, closed by this release.
- Card `0035-ac-taxonomy` — follow-up filed during design: generalises `time_gated: bool` to a categorical AC type (code / operational / observation / …) that informs close semantics, review evidence, and drive strategy. Deferred until the brownfield migration path (card 0032) and the canonical-schema work (card 0030) are ready to receive a richer enum.

### Changed

- `/orb:drive` close step (`plugins/orb/skills/drive/SKILL.md`) names the AC pre-flight as reconcile-first (forgot-to-tick is the common case), names `--force` as the deliberate escape with rationale-capture discipline (`orbit spec note` before `--force`), and documents time-gated ACs as the never-blocks-close category. Documentary wire; enforcement remains in the substrate.
- Card 0028 (`four-pillars`) `i_want` line re-framed from schema-field to relations-graph framing — matches the goal and scenarios already in place.

### Fixed

- `spec.close`'s response is now structurally explicit about deliberate-deferral. Previously a spec could be closed with unchecked ACs and no record of which ACs were left open; the new `forced_unchecked` and `time_gated_open` fields surface both categories in the structured response.

## [0.4.12] - 2026-05-13

Reconcile mode shipped — `orbit canonicalise --reconcile` is the on-ramp from legacy yaml field shapes to the canonical schema. A permissive read lives in a new `reconcile.rs` module gated behind the flag; every schema struct keeps `deny_unknown_fields`, so routine paths (`orbit verify`, `orbit canonicalise` without `--reconcile`, every other verb) stay strict. Closes spec `2026-05-12-reconcile-mode` (card 0032).

The change is forward-compatible for routine work — only invoking `--reconcile` itself requires the 0.4.12 binary. `/orb:setup`'s brownfield path is the only routine consumer (via the new §3g step, gated on `orbit audit drift` reporting drift).

### Added

- `orbit canonicalise --reconcile` — permissive pass that walks the substrate, applies dispositions from a built-in registry (`map` renames a field, `drop` removes it), and quarantines unknown content into a sibling `<name>.legacy.yaml` sidecar so semantic content is never silently destroyed. Combined with `--dry-run` it lists every disposition and exits non-zero when the tree is not clean — useful as a CI gate.
- `dispositions: [{path, kind, field, action}]` array on the canonicalise JSON envelope (only present in reconcile mode). Each entry names the file, entity kind, structural field path (e.g. `acceptance_criteria[2].ac_type`), and action (`map` / `drop` / `quarantine`).
- `AcceptanceCriterion::FIELDS`, `Scenario::FIELDS`, `Relation::FIELDS` — inner-shape field-name constants. Reconcile uses them to classify legacy fields inside lists-of-struct; lockstep unit tests keep each constant in sync with its struct.
- `/orb:setup` brownfield path gains §3g — after the layout migration completes, it runs `orbit audit drift` and offers `orbit canonicalise --reconcile --dry-run` → confirm → apply when drift is non-empty. Greenfield setup, `orbit verify`, and pre-commit hooks never invoke reconcile.
- Choice `0023-reconcile-as-canonicalise-mode` — MADR record of the surface decision (mode on `canonicalise` vs a separate verb).

### Changed

- Card 0030 (canonical-schema-and-glossary) names `orbit canonicalise --reconcile` as the on-ramp from legacy field shapes.
- Card 0032 (brownfield-spec-migration) reworded against the new mode; `specs[]` references this spec.

## [0.4.11] - 2026-05-12

Tree-views shipped — five new read-only navigation and synthesis verbs make the substrate's shape legible from the CLI and MCP without opening a single YAML file. Closes spec `2026-05-12-tree-views` (cards 0033, 0020). Surfacing wires land alongside the verbs so agents discover them at the right pipeline moments.

### Added

- `orbit card tree <id>` — local relations subgraph, depth-bounded, cycle-safe. Renders the cards/choices/specs/memories adjacent to a card so a session-start agent can see context without paging through files.
- `orbit card specs <id>` — bidirectional drift detection on `card.specs[]` against `spec.cards[]`. Surfaces orphaned refs in either direction.
- `orbit overview` — single-screen project synthesis: open specs, cards by maturity, recent memories, most-connected card, orphan cards. Bounded output regardless of project age.
- `orbit graph [--format mermaid|graphviz]` — renders the full cards-specs graph to stdout, pasteable into markdown or a renderer.
- `orbit audit drift` — permissive YAML scan against the canonical `Card` / `Spec` / `Choice` / `Memory` schemas. Surfaces unknown fields, missing required fields, and type mismatches that the canonical writer would silently rewrite.
- `Card::FIELDS`, `Spec::FIELDS`, `Choice::FIELDS`, `Memory::FIELDS` — public field-name constants on each schema type, the load-bearing surface that `orbit audit drift` checks against.
- `session.prime` gains a `next_step` field pointing at `orbit overview` so the very first verb after session start surfaces the substrate's shape.
- `/orb:card` SKILL §4 suggests `orbit card tree` after authoring; `/orb:distill` SKILL §2 directs to `overview` + `card tree` *before* drafting.

### Changed

- Wire envelope error coverage extended — every new verb's failure modes round-trip through the canonical `{ ok: false, error: { code, message } }` envelope shape with CLI ↔ MCP parity.

## [0.4.10] - 2026-05-11

Spec layout reverts to per-spec folders per choice 0021. `.orbit/specs/<id>.yaml + <id>.<sidecar>` becomes `.orbit/specs/<id>/spec.yaml + <id>/<sidecar>` across the substrate, the canonical writer, and every SKILL.md path string. Closes spec `2026-05-10-spec-folders-migration` (cards 0008).

The new `list_spec_files` walks immediate subdirectories of `.orbit/specs/` and returns every `<id>/spec.yaml`. As a side-effect it surfaced 19 bd-era specs that the previous flat scanner was silently skipping; those folders moved to `.orbit/archive/specs/` (no schema migration — the bd-era `constraints` / `values` fields are out of orbit-state v0.1's Spec schema). Card refs to those archived specs were rewritten to `.orbit/archive/specs/<id>/...`.

**Forward-incompatible layout change** — the parity gate fires. The 0.4.10 binary expects folder-shape; the 0.4.9 binary reads zero specs against the new layout.

### Added

- `OrbitLayout::spec_dir(id)` and `ensure_spec_dir(id)` helpers — callers writing per-spec files (spec.yaml, tasks.jsonl, notes.jsonl, sidecars) ensure the folder exists before invoking `write_atomic` / `append_jsonl_line`.
- `.orbit/archive/specs/` — quarantine destination for the 20 pre-orbit-state-v0.1 bd-era folders that don't parse against the current Spec schema.

### Changed

- `OrbitLayout::spec_file(id)` now returns `<root>/specs/<id>/spec.yaml`; `task_stream(id)` and `notes_stream(id)` now return `<id>/tasks.jsonl` and `<id>/notes.jsonl`. `list_spec_files` scans subdirectories.
- `spec.close` writes `.orbit/specs/<id>/spec.yaml` into linked-card `specs` arrays (was `<id>.yaml`). Existing card refs were updated for the post-migration specs and the archived bd-era specs in the same release.
- `.orbit/conventions/spec-layout.md` rewritten — folder shape canonical, flat sidecar layout named as the prior experiment with rationale (visual mess, prefix collision, non-atomic rename).
- `.orbit/METHOD.md` (and the byte-mirror at `plugins/orb/skills/setup/METHOD.md`) — vocabulary table Spec / Interview / Review / Drive state / Rally state rows updated to folder paths.
- SKILL.md sweep across `drive`, `rally`, `review-spec`, `review-pr`, `setup` — every cited sidecar path reverts from `<id>.<sidecar>` to `<id>/<sidecar>`.

## [0.4.9] - 2026-05-10

`Card` gains an explicit `id:` field; `orbit card show` and `orbit choice show` accept bare `NNNN` shorthand. The substrate's id conventions are documented as three families (enumerated for cards/choices, dated for specs, keyed for memories). Choices `0021-spec-folders` (per-spec folders revert) and `0022-entity-id-conventions` (id heterogeneity) are accepted; their migration specs open against cards 0008 and 0030.

### Added

- `Card.id: Option<String>` as the first field in the schema. Parsers accept legacy id-less yaml; the canonical writer fills `id` from the filename on the next canonicalise pass and rejects yaml whose `id` disagrees with its filename. One-shot pass over `.orbit/cards/` populated 31 existing cards.
- `resolve_numeric_slug` in `orbit-state/crates/core/src/verbs.rs` — `orbit card show 8` and `orbit choice show 21` resolve via filename prefix-match. Errors: zero matches → `not-found`; multiple matches → `ambiguous`. Six unit tests cover the resolver.
- `.orbit/conventions/id-conventions.md` — documents the three id-shape families, per-entity yaml field conventions, the type-qualifier prose contract, and CLI lookup forms.
- Choices `0021-spec-folders.yaml` (revert flat-sidecar specs to per-spec folders; supersedes the file-shape decision in the 2026-05-09 sidecar migration) and `0022-entity-id-conventions.yaml` (formalise the three id-shape families).
- Specs `2026-05-10-spec-folders-migration.yaml` (8 ACs, 6 gating) and `2026-05-10-card-id-field-and-conventions.yaml` (7 ACs, 5 gating) — open, ready for drive.
- README gains a `## Repository layout` section signposting the four top-level directories.

### Changed

- `.orbit/METHOD.md` and `plugins/orb/skills/setup/METHOD.md` — vocabulary table gains an Id-shape column; new Memory row; new Reference style section names the type-qualifier contract and bare-NNNN shorthand. Files stay byte-equal.
- `orbit-state` workspace version aligns with plugin (0.4.3 → 0.4.9). Substrate-binary parity gate now passes for terminals running the 0.4.9 binary against the new card schema.

### Fixed

- Spec `2026-05-10-repo-cruft-removal` shipped — `.beads-archive/` and the empty `.claude/worktrees/` removed from the working tree.

### Removed

- `.beads-archive/` (gitignored archived bd state, no longer needed) and `.claude/worktrees/` (empty stale runtime dir).

## [0.4.8] - 2026-05-10

`/orb:release` gains a substrate-binary parity gate. When `orbit-state/` changed in the release window but the on-PATH `orbit` binary predates the change, release refuses with a three-option resolution path (rebuild formula, set `ORBIT` env, or explicit `--accept-binary-lag` for forward-compatible changes). Closes the defect from 0.4.7 — sidecar-aware skill prose shipped against an older binary, which broke `orbit verify` for any terminal still on brew 0.4.3.

### Changed

- `plugins/orb/skills/release/SKILL.md` — pre-flight §1 gains step 4: substrate-binary parity gate. §7 confirm output now restates the binary state explicitly (resolved path, or "not gated" when orbit-state was untouched in this window).

## [0.4.7] - 2026-05-09

The bd-era folder layout for per-spec sidecars (drive.yaml, rally.yaml, review files) migrates to flat sidecar paths (`.orbit/specs/<id>.<file>`) — one substrate convention across drives, rallies, and reviews. The orbit-state scanner gains a dotless-stem filter so `<id>.drive.yaml` and `<id>.rally.yaml` are skipped during spec parsing; `orbit verify` and `orbit spec list` stay clean with sidecars on disk.

### Added

- `.orbit/conventions/spec-layout.md` — canonical sidecar inventory naming every per-spec sidecar shape (`<id>.yaml`, `<id>.tasks.jsonl`, `<id>.notes.jsonl`, `<id>.drive.yaml`, `<id>.rally.yaml`, `<id>.decisions.md`, `<id>.interview.md`, `<id>.review-{spec,pr}-<date>.md` with `-v2`/`-v3` cycle suffixes). The bd-era folder layout is named explicitly as deprecated.
- `plugins/orb/scripts/tests/test-sidecar-layout.sh` — five-step smoke test against a temp `--root`: promote produces flat spec, drive sidecar reachable via `[[ -f *.drive.yaml ]]`, rally sidecar reached via `*.rally.yaml` glob, `orbit verify` clean, `orbit spec list` excludes sidecar ids.
- Two unit tests in `orbit-state-core` pin the scanner-fix contract: `list_spec_files_skips_sidecar_shapes` (layout) and `verify_excludes_sidecar_yaml_shapes` (verify).

### Changed

- `orbit-state/crates/core/src/layout.rs` — `list_yaml_files` filters spec YAML loads to dotless-stem files only. Both `verify_all` and `Index::rebuild_from_files` consume the filtered list, so adding a new sidecar shape requires no scanner changes — the dotless-stem rule excludes it automatically.
- `/orb:drive` SKILL.md — every drive sidecar reference (path, code block, resumption-detection snippet, embedded CronCreate heartbeat prompt body) and every review-file path uses sidecar form. The promote-stage description corrected: `promote.sh` materialises a spec at the flat `.orbit/specs/<spec-id>.yaml` (no folder).
- `/orb:rally` SKILL.md — folder convention collapsed end-to-end. `RALLY_DIR` removed; CLI argument changes from `<rally-folder>` to `<rally-id>`; resumption scan iterates `.orbit/specs/*.rally.yaml`. Per-child decision packs and interviews migrate to sidecars (`<child-spec-id>.decisions.md`, `<child-spec-id>.interview.md`); the path-discipline contract names the two specific sidecars rather than a per-child folder.
- `/orb:review-spec` and `/orb:review-pr` SKILL.md — inline-invocation defaults default to sidecar paths; the `<spec-folder>`-shaped branch and the `.orbit/reviews/` fallback are removed.
- `.orbit/METHOD.md` and `plugins/orb/skills/setup/METHOD.md` — vocabulary table rewritten to sidecar form (Drive state, Rally state, Interview rows). The two files stay byte-equal so greenfield projects bootstrapped via `/orb:setup` get the same canonical statement.

### Fixed

- `orbit verify` and `orbit spec list` no longer break when a `<id>.drive.yaml` or `<id>.rally.yaml` sidecar is present in `.orbit/specs/` — previously the scanner attempted to parse them as `Spec` and surfaced an `unknown field, expected one of id, goal, cards, status, labels, acceptance_criteria` error. The dotless-stem filter excludes sidecar shapes from primary entity loads.

## [0.4.6] - 2026-05-09

`/orb:setup` now primes downstream projects with a canonical orbit method overview that CLAUDE.md `@-imports` — no more inline vocabulary blocks drifting across plugin versions. `/orb:card` and `/orb:distill` gain a card-vs-choice pre-flight so implementation-surface decisions ('should X be in bash or rust?') route to choice files, not aspirational cards.

### Added

- `plugins/orb/skills/setup/METHOD.md` — canonical orbit method overview (single screen, ~72 lines): pipeline, vocabulary, card-vs-choice-vs-spec-vs-memo decision tree, substrate rules, four pillars, BLUF / Decision Brief skeleton inlined directly so projects without `.orbit/STYLE.md` get the prose contract too.
- `plugins/orb/scripts/setup-method.sh` — atomic `/orb:setup` §6 implementation: legacy-CLAUDE.md detection BEFORE any file write (decline → atomic refuse, no orphan METHOD.md), byte-for-byte drift detection on re-run, idempotent `@-import`. Supports `--answer-legacy` / `--answer-drift` for scripted contexts.
- `plugins/orb/scripts/tests/test-setup-method.sh` — four scenarios (fresh / drift-prompt / legacy-accept / legacy-refuse), all green.
- `/orb:card` and `/orb:distill` SKILL.md gain a "Card or Choice?" pre-flight — implementation-surface decisions route out to MADR choice files at `.orbit/choices/`, not new cards.
- Choice `0020-shell-scripts-to-rust-verbs` — policy choice naming the migration path for `promote.sh`, `setup-method.sh`, and `orbit-acceptance.sh` to orbit Rust verbs, sequenced opportunistically per script.

### Changed

- `/orb:setup` SKILL.md §6 rewritten end-to-end. The old inline `## Workflow (orbit)` / `## Orbit vocabulary` / `## Current Sprint` snippet is removed; METHOD.md is the single source of truth. Existing downstream CLAUDE.md files containing the legacy blocks get an atomic migrate-or-refuse prompt — no path to dual-source drift.
- CLAUDE.md decision tree gains a fourth branch covering choices, placed before the card branch so agents discriminate before defaulting to a card. Worked example named: "should `orbit spec promote` live in rust" is a choice, not a card.
- CLAUDE.md vocabulary table's `Decision` row renamed to `Choice` (matches the `.orbit/choices/` directory), path corrected from `.md` to `.yaml`, and the row carries the implementation-surface framing.
- `/orb:distill` SKILL.md §2 Draft adds per-candidate capability-vs-choice classification — choice-shape distillations write MADR files instead of cards.
- Card 0017 amended: greenfield scenario then-clause updated to "writes `.orbit/METHOD.md` and ensures CLAUDE.md @-imports it"; two new scenarios cover drift detection and atomic legacy migration; pillar 2 (agent self-learning) attribution added via `relations:feeds → 0028`.
- orbit-repo CLAUDE.md dogfooded: 119 → 32 lines. Substrate sections (vocabulary, decision tree, pipeline, four pillars, key concepts, orbit-state quick reference) replaced by `@.orbit/METHOD.md`. The standalone "Session Completion / Mandatory Workflow" section is reshaped to a tight 4-line "Push discipline" block; substrate-shaped rules (orbit task verbs, hand-off via memory) deleted from CLAUDE.md, project-specific git discipline kept inline.

## [0.4.5] - 2026-05-09

The bd-era cleanup arc closes — `promote.sh` is ported to orbit-state, every /orb:drive promote stage runs against the substrate directly, no manual workaround. /orb:design also gains three modes (open / closed / partial), an implementation-question filter, and a user-voice prose paragraph promoted to a first-class output that downstream specs cite as the intent contract.

### Added

- /orb:design pre-flight design-space classification — open (no choice file), closed (architectural choice already pinned), partial (residual trade-offs). Closed mode emits a one-screen `design-note.md` instead of a full interview.
- Implementation-question filter at /orb:design — each candidate question must require codebase context, schema knowledge, metric vocabulary, or evaluation tooling to pass. Author-preference questions get routed to implementation-notes for the implementing agent rather than surfaced to the author.
- Top-of-file user-voice "What good looks like" paragraph slot in interview / design-note artefacts, drafted by the agent from the card and offered for editing rather than reconstructed via Q&A.
- /orb:spec and /orb:spec-architect cite the user-voice paragraph as the intent contract — quoted in the spec's `goal` or `notes`, alongside the Q&A.
- Mode-switch trigger at /orb:design — twice-rejected implementation-shaped questions trigger a switch to closed/partial mode rather than another reformulation.

### Changed

- `plugins/orb/scripts/promote.sh` rewritten against orbit-state — derives `<YYYY-MM-DD>-<card-slug>` from the card filename, calls `orbit spec create`, writes `acceptance_criteria` directly into the flat spec YAML, then runs `orbit canonicalise`. Stdout still emits just the spec id; new `--root` passthrough makes the script testable.
- `test-promote-gate-propagation.sh` now exercises the real promote → orbit-spec-create → orbit-spec-show round-trip end-to-end under a temp `--root`, not just the dry-run path.
- /orb:drive SKILL.md trimmed 853 → 688 lines (-19%); /orb:rally SKILL.md trimmed 1016 → 840 lines (-17%). Slim Critical Rules sections restored.
- `.orbit/conventions/acceptance-field.md` rewritten from the bd-era markdown-line format to orbit-state's structured `acceptance_criteria`.
- Project `CLAUDE.md` no longer inlines STYLE.md — the `@.orbit/STYLE.md` import resolves at session start, verified empirically against fresh subagent forks.
- Card 0028 amended to documentation-only pillar wiring; goal refined to reflect emergent pillar outcomes rather than schema fields.

### Removed

- Six bd-era files: `bd-init.sh`, `parse-progress.sh`, `session-context.sh`, `rally-coherence-scan.sh`, `AGENTS.md`, `plugins/orb/hooks/hooks.json`.

## [0.4.4] - 2026-05-08

First live wires under choice 0019 (cards declare framework wires in scenarios; aspirational cards don't pass review). Card 0026's BLUF / Decision Brief contract is now substrate-enforced — distilled into `.orbit/STYLE.md`, imported into project CLAUDE.md, and cited from the three prose-producing orb skills. Closes the canonical aspirational-card example the choice was written about.

### Added

- `.orbit/STYLE.md` — distilled BLUF / Decision Brief contract: TL;DR-led skeleton, recommendation discipline, seven anti-patterns by name, response-variant table, tone contract. Single-screen distillation, not a verbatim card transcription.
- Project `CLAUDE.md` imports STYLE.md via `@.orbit/STYLE.md` (with the contract inlined for cache-resilience) so the contract loads into every orbit-repo session.
- `/orb:design`, `/orb:review-spec`, `/orb:review-pr` SKILL.md files cite card 0026 + STYLE.md using the belt-and-braces pattern (one-line prose marker + `@` import).
- Choice 0019 — cards must declare framework wires in scenarios; aspirational cards (`maturity: planned` + empty `specs:`) don't pass review.
- Cards 0028 (four pillars), 0029 (fan-out), 0030 (canonical schema and glossary), 0031 (design-session user language) distilled from memos. Each carries the "Wired into the framework" gate scenario.

### Changed

- Project CLAUDE.md: four pillars (executive-level interaction, agent self-learning, agent state-persistence, long-running R&D) named explicitly as the load-bearing why-test for any work in this repo.
- Card 0026 (executive-communication) maturity bumped `planned` → `emerging` after the first wires drive shipped.

### Fixed

- `orbit memory remember` invocation syntax in skill prompts and PRIME.md — previously used a stale form that didn't match the current orbit-state CLI.

## [0.4.3] - 2026-05-08

`orbit canonicalise` is now a first-class subcommand of the main `orbit` binary. Hand-edited cards and choices that drift from the canonical writer's output (whitespace, field order, trailing newlines) used to fail `orbit verify` with `not_byte_identical` and no in-toolbox fixer — the brew binary shipped only `verify`, and the standalone `orbit-canonicalise` repair tool wasn't packaged. Surfaced when a downstream session got stuck adding a new MADR with no path forward short of building from source.

### Added

- **`orbit canonicalise [--dry-run] [--json]`** — walks `.orbit/{specs,cards,choices,memories}`, parses each file, reserialises through the canonical writer, and rewrites any drift in place. Mirrors `orbit verify`'s output shape; exits non-zero only on parse failures (drift fixed in place is success). The shared logic now lives in `orbit_state_core::canonicalise`, callable from both the main CLI subcommand and the standalone `orbit-canonicalise` binary.

### Changed

- **`orbit verify` error message** for `NotByteIdentical` now points at `orbit canonicalise` as the fixer, replacing the prior advice to "run a verb that touches the file" — a workflow that didn't exist for `Choice` (read-only verbs only).

## [0.4.2] - 2026-05-08

orbit now lives at `meridian-online/orbit` and shares the meridian release pipeline with `finetype` and `arcform`. End-users install the orbit binary via `brew install meridian-online/tap/orbit` (Homebrew on macOS, Linuxbrew on linux) instead of `cargo install --path orbit-state/crates/cli`. Plugin and binary versions are aligned from this release onward; both move in lockstep. See decision `0018-orbit-distribution-via-meridian` and spec `orbit-distro` for the migration plan; card `0027-brew-installable` is the capability being delivered.

### Migration notes for orb plugin users

Existing installations of `orb@orbit` against `hughcameron/orbit` need to re-add the marketplace from the new home:

```
/plugin marketplace remove orbit
/plugin marketplace add meridian-online/orbit
/plugin install orb@orbit
```

GitHub auto-redirects the old clone URL, so existing `git clone` of the substrate repo continues to work, but the Claude Code plugin marketplace metadata pins the original org/repo and needs to be refreshed manually.

### Added

- **`orbit` binary distribution** — pinned tar.gz archives for x86_64 and aarch64 on macOS and linux, sha256-stamped, published to GitHub Releases on every tag. The release pipeline auto-updates `meridian-online/homebrew-tap`'s `Formula/orbit.rb` so `brew upgrade orbit` is the upgrade path for end-users. Cargo-install remains supported for contributors building from source.

### Changed

- **Plugin and binary versions aligned at 0.4.2.** `plugins/orb/.claude-plugin/plugin.json` and `orbit-state/Cargo.toml` workspace version are now synchronised; releases bump both in lockstep. The orbit-state binary moves from its `0.1.0-dev` development version to the unified release line — this is an alignment jump, not a semver claim about the binary's API.

## [0.4.1] - 2026-05-08

orbit-state v0.1 substrate adoption — the six core skills now read and write the files-canonical orbit-state substrate (`.orbit/cards`, `.orbit/specs`, `.orbit/choices`, `.orbit/memories`) via the `orbit` CLI instead of `bd`. Verdict-line contracts, deterministic gate checks, and the cold-fork architecture are preserved verbatim; the underlying file format and tool surface have changed.

This is a substrate-shaped patch release. The skills assume the host repo has migrated to orbit-state per the playbook at `~/github/hughcameron/ops/playbooks/migration-orbit-state-v0.1.md`. Pre-migration repos should pin to 0.4.0 or migrate before upgrading.

### Added

- **`orbit-acceptance.sh`** — orbit-state-shaped sibling of `parse-acceptance.sh`. Same five subcommands (`acs`, `next-ac`, `blocking-gate`, `has-unchecked`, `check`) and same tab-separated tuple contract, but reads via `orbit spec show <id> --json` and writes via `orbit spec update --ac-check` instead of bd's `--acceptance` field.

### Changed

- **`/orb:implement`** rewritten against orbit-state. Spec-id input (was bead-id). AC list read from the spec's `acceptance_criteria` array (`{id, description, gate, checked}`). AC flips through `orbit-acceptance.sh check` → `orbit spec update --ac-check`. Detours become sub-tasks under the current spec via `orbit task open --spec-id <current>`; the bd `discovered-from` dep edge has no orbit-state v0.1 equivalent and is captured in the task body text. NO-GO close uses `orbit spec note` + `orbit spec close` (no `--reason` flag in orbit-state).
- **`/orb:drive`** rewritten against orbit-state. Drive state migrates from bd metadata fields (`drive_stage`, `drive_iteration`, `drive_review_*_cycle`) to `.orbit/specs/<spec>/drive.yaml` — the named slot in the orbit vocabulary. Iteration chains move from the bd dep tree to a `drive.yaml.iteration_history` array. Review output paths move from `orbit/reviews/<bead-id>/` to `.orbit/specs/<spec-id>/`. Verdict-line regex contract preserved verbatim.
- **`/orb:rally`** rewritten against orbit-state. Epic bead + child bead graph + dep edges all collapse into `.orbit/specs/<rally-folder>/rally.yaml`. The claimable-set rule (open + all `dep_predecessors` closed/parked) replaces `bd ready --type task --parent <epic>`. Six-token reason_label vocabulary preserved.
- **`/orb:review-spec`** rewritten against orbit-state. Spec-id input; reads via `orbit spec show <id> --json` + `orbit-acceptance.sh acs <id>`. Verdict-line contract preserved verbatim. Output paths support both flat (`.orbit/specs/<id>.yaml`) and folder-shaped (`.orbit/specs/<folder>/spec.yaml`) specs.
- **`/orb:review-pr`** rewritten against orbit-state. Same parser + verb shift as review-spec; AC coverage check now reads from the spec's `acceptance_criteria` array.
- **`/orb:audit`** rewritten against orbit-state. Locates specs via `orbit spec list` (was filesystem glob). Drops the deprecated `ac_type` field — orbit-state's strict schema stores ACs uniformly with `{id, description, gate, checked}`. Non-code classification is now made from description text plus gate flag at audit time.
- **Path-only updates** across the remaining skills (`card`, `design`, `discovery`, `distill`, `keyword-scan`, `memo`, `setup`, `spec`, `spec-architect`) and the gate-AC verification regression test — all `bd` references swapped for `orbit` verbs; `orbit/` → `.orbit/` paths.

### Removed

- **`parse-acceptance.sh`** — bd-era markdown AC parser. Its only live consumer (the gate-AC verification regression test) was ported to `orbit-acceptance.sh`'s JSON-array stdin shape.

### Notes

- Skills assume host-repo migration via the orbit-state v0.1 playbook. Mixing this plugin version with a bd-era host repo produces parse errors.
- The `orbit-state` Rust binary is a separate distribution (not bundled with this plugin). See the migration playbook for build instructions.

## [0.4.0] - 2026-05-01

Bead-native execution layer — orbit's four-card overhaul (orbit-6da.1–6da.4) makes beads the canonical substrate for AC tracking, drive orchestration, and rally state. The snapshot bridge between drive and the cold-fork reviewers is removed; reviewers read beads directly. `drive.yaml`, `progress.md`, and `rally.yaml` are gone. The bead graph IS the workflow.

### Added

- **Bead-native cold-fork reviews** (card 0016, orbit-6da.4). `/orb:review-spec` and `/orb:review-pr` read the bead directly via `bd show <bead-id> --json` and `parse-acceptance.sh acs <bead-id>` — the same parser `/orb:implement` uses, so AC interpretation cannot drift between implement and review. The snapshot bridge (`bead-snapshot-<date>.md`) is removed pipeline-wide. Verdict files land at `orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md` for both forked and inline invocations.
- **End-to-end gate semantics.** Card scenario `gate: true` propagates through `promote.sh` to bead AC `[gate]` marker. `parse-acceptance.sh acs` exposes `is_gate=1` as a parsed column. `/orb:review-spec` Pass-1 deterministic check (non-empty / not-placeholder / ≥20 chars) fires against gate-AC description text — was silently no-op under the snapshot bridge.
- **Test fixtures for the bead-native review substrate.** `plugins/orb/scripts/tests/test-gate-ac-verification.sh` (parser + 3 deterministic rules) and `test-promote-gate-propagation.sh` (card scenario → promote.sh → bead AC `[gate]` marker).
- **MADR 0013** — `.orbit/choices/0013-bead-acceptance-field-as-cold-fork-substrate.md`. Documents five design decisions (skill-reads-bead vs drive-prerender; AC-shape mapping; ac_type mapping; gate propagation via promote.sh; hard cutover), the substrate-mapping table, and full consequences including accepted losses (ac_type exemption fidelity; AC commit-provenance; cycle-history `[x]` leak).
- **Card 0017** — `/orb:setup` is bead-aware (planned). Folds bd precondition check, orbit plugin version sanity, and `bd-init.sh` invocation into `/orb:setup` so the orbit/ layout and `.beads/` initialise atomically. Until this ships, bead-init runs as a manual operator step.
- **Beads foundation** (orbit-6da.0). Beads issue tracker initialised in orbit itself. Acceptance-field convention (`.orbit/conventions/acceptance-field.md`). Core scripts: `parse-acceptance.sh` (five subcommands for AC enumeration and check-off), `promote.sh` (card → bead with AC generation), `bd-init.sh` (project initialisation), `PRIME.md` (session-start context).
- **`/orb:implement` rewritten against beads** (orbit-6da.1). Bead acceptance field replaces `progress.md` as the AC source of truth. `TaskCreate`, drift detection (sha256), and resume reconcile removed. Detours escalate as sub-beads via `bd create --parent ... --deps "discovered-from:..."`. Gate enforcement delegated entirely to `parse-acceptance.sh next-ac`.
- **`/orb:drive` rewritten against beads** (orbit-6da.2). Design + Spec stages collapse into `promote.sh card→bead`. Drive state machine lives in bead metadata (`drive_stage`, `drive_iteration`, `drive_review_*_cycle`). Iteration history tracked via `discovered-from` dependency edges between iteration beads. NO-GO closes current bead and promotes a new iteration bead carrying constraint history in the description.
- **`/orb:rally` collapses onto the bead dependency graph** (orbit-6da.3). `rally.yaml` removed. Epic bead + child beads IS the rally. `bd ready --type task --parent <epic>` replaces TaskList for in-session card visibility. Rally phase tracking lives in epic bead metadata. Mid-flight parallel→serial conversion is a single `bd dep add` invocation.

### Changed

- **Drive cold-fork brief** — Stage 1 (review-spec) and Stage 3 (review-pr) briefs carry only `<bead-id>`, absolute verdict output path, and verdict-line contract. Snapshot paths gone.
- **Drive Completion** — commit-1 description and PR-body no longer reference bead snapshots (they no longer exist). Commit 1: `All code changes and the review files`.
- **Inline-mode verdict paths** in both review skills moved to `orbit/reviews/<bead-id>/review-{spec,pr}-<date>.md` (was `.orbit/specs/YYYY-MM-DD-<topic>/...`).
- **Drive SKILL.md section renumbering** — Stage 1: §1.1 is now "Compute the cycle-specific verdict path" (was §1.2; §1.1 "Write the bead snapshot" is gone). Stages 1 and 3 section numbers updated throughout; Resumption table cross-references corrected.
- **`/orb:review-spec` Step 1** renamed to "Gather the Bead"; takes a bead-id argument; reads `bd show <bead-id> --json` + `parse-acceptance.sh acs <bead-id>`. Spec.yaml lookup, interview_ref lookup removed.
- **`/orb:review-pr` Phase 1/2** reads bead via `bd show` + `parse-acceptance.sh`; `progress.md` cross-reference removed; `ac_type` / `test_prefix` field references removed; AC coverage check uses bare `ac<NN>` test-name pattern; reviewer contextualises exemptions in the honest-assessment paragraph.
- **Decision 0002 (`ac-test-prefix`)** status updated to `superseded by 0013 (review-pr scope only)` — `test_prefix` remains live in `/orb:spec`, `/orb:spec-architect`, `/orb:audit`, `/orb:implement`.
- **Decision numbering collision resolved** — `0011-design-intent-not-means.md` renamed to `0012-design-intent-not-means.md`. New substrate MADR is `0013`.
- **Drive heartbeat self-termination** — full-autonomy heartbeat calls `CronDelete` on itself when the bead transitions to `closed`, as a backstop alongside primary cleanup in §Completion and §Escalation.
- **Cold-fork review gate hardened** against nested Agent unavailability — drive escalates immediately rather than falling back to inline review, preserving the cold-fork separation contract.

### Removed

- `drive.yaml` per-iteration orchestration state — replaced by bead metadata fields.
- `progress.md` AC tracker — replaced by bead `acceptance_criteria` field via `parse-acceptance.sh`.
- `rally.yaml` rally state — replaced by epic bead + child bead graph.
- Bead snapshot bridge (`bead-snapshot-<date>.md`, `bead-snapshot-<date>-pr.md`) from drive's review pipeline.

## [0.3.3] - 2026-04-22

### Added
- `/orb:implement` §6a — out-of-scope findings during implementation are forwarded as memos (`.orbit/cards/memos/`) with data and provenance. Agents no longer suggest "open a follow-up card" — cards describe capabilities, not work items. Distill handles the structural decision later.

### Changed
- `/orb:review-pr` — explicit rule: never suggest follow-up cards in findings.

## [0.3.2] - 2026-04-21

### Changed
- **Design interviews capture intent, not means.** `/orb:design` reframed from "works out the how" to "captures what good looks like." Questions target outcomes, priorities, risk appetite, and scope — not implementation approach. Means-level observations (which function, what algorithm, test structure) are recorded as implementation notes for the implementing agent instead of being asked as interview questions.
- Interviewer persona gains a decision-level gate before the evidence hierarchy: "Would the author need codebase context to answer this?" If yes, it's a means question — record as an implementation note, don't ask.
- `/orb:discovery` aligned with the same intent-level questioning principle.

### Added
- `implementation_notes` field in spec YAML format — means-level leads from the design session. Not constraints; starting context the implementing agent can use or override with evidence. Consumed by `/orb:implement`.
- `.orbit/choices/0012-design-intent-not-means.md` (originally numbered 0011 at 0.3.2 release; renumbered in 0.4.0 to resolve a numbering collision with `0011-beads-execution-layer.md`)

## [0.3.1] - 2026-04-21

### Changed
- **Rally state moves into a spec-shaped folder.** `rally.yaml` now lives at `.orbit/specs/<date>-<slug>-rally/rally.yaml` instead of a flat `.orbit/specs/rally.yaml`. Completed rallies stay where they are — the folder itself is the history record. No sibling `archive/` directory, no archival prompt when the next rally begins.
- `/orb:rally` §1 scans `.orbit/specs/*/rally.yaml` for an active rally (phase != complete); §3 Initialise creates the rally folder before writing `rally.yaml` inside it; §10 Completion and §11 Resumption drop the "awaiting archival" language and the archive prompt. Two or more rallies with `phase != complete` is a state error per §12.
- `session-context.sh` scans `.orbit/specs/*/rally.yaml` instead of checking a fixed path, and the `latest_spec` find excludes `*-rally` folders so the workflow surface never mistakes a rally folder for a spec folder.
- CLAUDE.md vocabulary row for Rally state updated to the new folder-per-rally path.

### Added
- **Vocabulary glossary in `/orb:setup`.** The `## Workflow (orbit)` snippet appended to a project's `CLAUDE.md` now carries a six-row `## Orbit vocabulary` block (Card / Memo / Interview / Spec / Progress / Decision) and the "cards describe *what*, specs describe *work*" discipline line. Idempotent setup runs detect the pre-vocabulary shape and offer a targeted migration prompt — on `y`, the legacy "Artefacts live in…" line is replaced with the full `## Orbit vocabulary` block while the skills list and Current Sprint are left untouched.

## [0.3.0] - 2026-04-20

UX uplift rally — four coordinated cards shipped together (PRs #12, #14, #11, #10) to make orbit sessions mission-resilient, visible in real time, and sharper at approval gates.

### Added
- **Mission resilience — three-layer spec fidelity through disruptions.** `progress.md` gains `Spec path:`, `Spec hash:`, `Current AC:` header fields and a `## Detours` section for out-of-order work. The `SessionStart` hook surfaces the current AC on resume, detects spec drift (sha256 mismatch with recorded baseline), and blocks advancement past `(gate)`-annotated ACs until the gate closes. `/orb:implement` §5 now declares detour discipline, spec-hash backfill, drift-halt, and gate-enforcement rules. (#12)
- **Session visibility — first-class TaskList integration for `/orb:implement`.** After writing `progress.md`, the skill emits a `TaskCreate` per hard constraint and per AC (flat, scoped by `metadata.spec_path`, subjects verbatim from progress.md). `TaskUpdate` must land in the same tool-call turn as the progress.md checkbox flip — anything else is a protocol violation. Mid-session resumes reconcile the task list against progress.md via a deterministic cancel-then-recreate algorithm using `TaskUpdate status: cancelled` + `TaskCreate`, with a canonical `RESUME_REBUILD_WARNING`. (#14)
- **`plugins/orb/scripts/parse-progress.sh`** — single source of truth for `progress.md` parsing. Six subcommands: `acs`, `constraints`, `spec-path`, `next-unchecked-ac`, `post-gate-ac`, `has-unchecked`. `## Detours` content is ignored by the AC parser — a `- [x] ac-02` inside Detours never flips ac-02's status. Both the mission-resilience next-AC surface and the session-visibility resume reconcile delegate to this helper. (#14)
- **Monitor heuristic for long test runs.** `/orb:implement` §5 declares that tests expected to run >60 seconds or full-suite should be launched via Monitor with the canonical failure-marker filter `grep --line-buffered -E 'FAIL|ERROR|AssertionError|Traceback'`, so failures stream back mid-run rather than on completion. Short tests stay on Bash. (#14)
- **First-failure checkpoint.** On the first test failure of a run, `/orb:implement` pauses and offers two canonical options (investigate-and-re-run vs let-the-suite-finish-then-triage) via `AskUserQuestion` under an interactive TTY; subsequent failures do not re-prompt. Under `/orb:drive` full (non-interactive), the skill emits a canonical `FIRST_FAILURE_NONINTERACTIVE_MARKER` to stderr and halts with exit 2 for upstream triage. (#14)
- **`/orb:drive` live visibility — heartbeat, escalation ping, four-option verdict gate.** Guided-mode PR gate now offers four canonical choices (GO, NO-GO, read-reviews-first, drop-to-supervised) instead of a binary. Long-running stages emit heartbeat surfaces so the author knows the agent is alive; escalations ping the author with context rather than silently parking. (#11)
- **`/orb:rally` §2b approval gate tightened.** Approval uses canonical labels; the modify flow is now a two-prompt loop (collect edits, confirm, re-present) rather than free-form one-shot. Thin-card refusal still runs unconditionally before the gate. (#10)

### Changed
- `/orb:implement` §1–§4c are byte-identical to the post-mission-resilience baseline (sha256 verified, empty diff) — the session-visibility changes land as §4d + four §5 rules, not as rewrites of the shipped pre-flight behaviour.
- `plugins/orb/scripts/session-context.sh` next-AC surfacing and resume-reconcile blocks refactored to delegate to `parse-progress.sh`; zero `awk|sed` hits remain in those regions.

## [0.2.19] - 2026-04-20

### Added
- **`/orb:rally`** — new top-level orchestration skill for multi-card sprints. Proposes a rally, runs design/implementation in parallel via nested forked Agents with recursive context separation, and enforces a consolidated decision gate. Coherence is enforced via `plugins/orb/scripts/rally-coherence-scan.sh`. See `.orbit/choices/0003-rally-skill-boundary.md`, `0008-rally-subagent-path-discipline.md`, `0009-rally-parallel-drive-full.md`, `0010-rally-thin-card-guard.md`.
- `SessionStart` hook now detects an active `.orbit/specs/rally.yaml` and surfaces rally goal, phase, autonomy mode, per-card status, and parked constraints. Individual drive states are subordinated to the rally display when a rally is active.

### Changed
- **Artefact layout consolidated under `orbit/`.** The four top-level directories (`cards/`, `specs/`, `decisions/`, `discovery/`) have moved to `.orbit/cards/`, `.orbit/specs/`, `.orbit/choices/`, and `.orbit/discovery/`. All skill docs, hooks, examples, and references have been rewritten to point at the new paths. The move was done via `git mv` so history is preserved (`git log --follow` traces every artefact back through the rename).
- `/orb:setup` now detects four repo states — **greenfield** (create fresh `orbit/`), **brownfield** (legacy bare dirs present → single all-or-nothing migration prompt), **idempotent** (already migrated, no-op), and **mixed** (refuse with a clear collision report). Brownfield migration runs one `git mv` transaction covering every detected bare dir; untracked residue is reported after the move.
- `SessionStart` hook (`session-context.sh`) now gates on the presence of `orbit/` and emits a one-line nudge (`orbit: legacy layout detected. Run /orb:setup to migrate.`) when bare-layout dirs are found without `orbit/`. Hardened against partial `orbit/` layouts: `find` pipelines inside the drive and latest-spec scans are guarded with `[[ -d ... ]]` checks plus `|| true`, so the hook survives manually-created `orbit/` directories without `cards/` or `specs/` subdirs.
- `CLAUDE.md` snippet appended by `/orb:setup` now references `.orbit/cards/`, `.orbit/specs/`, and `.orbit/choices/`.
- **`/orb:drive` forks its review stages.** `review-spec` and `review-pr` now run in nested forked Agents with `context: fork` at the architectural root, honouring the context-separation contract that the review skills themselves already declared. Verdict is read from the written artefact rather than the return message. See `.orbit/choices/0005-drive-review-artefact-contract.md`, `0006-drive-cold-re-review.md`, `0007-drive-rerequest-budget.md`.

### Notes
- Prior review artefacts (e.g. `review-pr-*.md`, `review-spec-*.md`) that quoted old bare paths were rewritten in place during the artefact-folder migration. This is a deliberate evidence-fidelity trade-off in favour of a clean end-state; the migration commit itself is the audit trail for the path change.

## [0.2.18] - 2026-04-17

### Added
- `test_prefix` metadata field for specs — disambiguates AC-to-test mapping across multi-spec projects. Skills `spec`, `spec-architect`, `audit`, `review-pr`, and `implement` all consume the prefix.
- `decisions/0002-ac-test-prefix.md` — documents the choice of explicit spec-scoped prefixes over globally unique IDs or auto-derived slugs.

### Changed
- AC naming guidance now recommends slug-style prefixes (`remat`, `introspect`) over version-like prefixes (`v03`), since `metadata.version` already carries the version.
- `/orb:audit` warns when multiple specs exist but any lack `test_prefix`.

## [0.2.17] - 2026-04-16

### Changed
- `/orb:release` — moved from user-level skill (`~/.claude/skills/release/`) into the orbit plugin. Invoked as `/orb:release` instead of `/release`, freeing the `/release` namespace for project-specific release skills.

## [0.2.16] - 2026-04-16

### Changed
- `/orb:drive` pipeline expanded to 5 stages: Design → Spec → **Review-Spec** → Implement → Review-PR. Every spec now gets reviewed as part of the drive.
- `/orb:drive` guided mode removes intermediate go/no-go gates. Reviews ARE the quality gates. The only interactive pause is a rich final summary (spec review verdict, AC coverage, honest assessment) before PR creation. "Let me read the reviews first" is an explicit option.
- `/orb:drive` supervised mode gates now include richer context (AC counts, finding summaries) instead of bare "greenlight?" prompts.
- `/orb:review-spec` replaced with progressive 3-pass model (decision 0001). Pass 1 (structural scan) always runs. Pass 2 (assumption & failure analysis) triggered by findings or content signals. Pass 3 (adversarial review) triggered by structural concerns. Depth scales with findings, not upfront classification.
- Removed risk tier classification (HIGH/STANDARD/SKIP) from `/orb:spec`. Every spec gets reviewed — the progressive model makes tier gating unnecessary.

### Added
- `decisions/0001-progressive-spec-review.md` — first orbit decision record. Documents why tier-based review gating was replaced with progressive review.

## [0.2.15] - 2026-04-15

### Added
- `/orb:drive` Disposition section — defines the agent's working stance: find the way through, treat negatives as constraints on the next iteration, push past the first plateau. Ported from prior agent research disposition.
- Semantic escalation triggers (recurring failure mode, contradicted hypothesis, diminishing signal) alongside the mechanical 3-iteration budget. An honest agent may escalate before the budget is spent.
- Escalation summaries now include "What would have to be true" — what assumptions need revisiting for a future attempt to succeed.

## [0.2.14] - 2026-04-15

### Added
- `/orb:drive` — agent-driven card delivery. Takes a card path and autonomy level (full/guided/supervised), then drives the full orbit pipeline (design → spec → implement → review-pr) as a single inline session. Tracks state in `drive.yaml` for session resumption, with a 3-iteration budget before escalation. Thin cards (< 3 scenarios) are refused for full autonomy.
- `session-context.sh` now detects `drive.yaml` and surfaces active drive state (card, autonomy level, iteration, status, next action) at session start. Escalated drives show a distinct message.

## [0.2.13] - 2026-04-13

### Added
- `/orb:card` "What Gets Closed" section — specs are the closure unit; cards are never closed. A NO-GO result updates the card's `goal` and `maturity`, not its existence.
- `/orb:implement` step 7 "When a Spec Produces a NO-GO" — guidance to record evidence in `progress.md`, mark ACs with the result, update the card's goal, and loop back to `/orb:design` with the new evidence.

## [0.2.12] - 2026-04-12

### Added
- `/orb:keyword-scan` — shared technique for keyword-based search across orbit artifacts. Extracts 5–8 distinctive domain terms from a card, spec, or interview; builds a ripgrep alternation pattern; falls back to `grep -rl` in environments without `rg`. Referenced by all workflow skills rather than inlining the pattern.
- `/orb:spec` now appends the new spec path to the card's `specs` array after saving (write-time enforcement). Agents downstream that read the array get a complete work trail without manual upkeep.
- `/orb:design` reconciles the card's `specs` array against a keyword scan of `specs/` before the session starts — surfaces orphaned specs the author can confirm to link.
- `/orb:distill` checks `cards/` for existing capability overlap before drafting new cards.
- `/orb:card` checks `cards/` and `specs/` for overlap before finalising a new card.
- `/orb:discovery` searches `specs/` and `decisions/` for prior art before the interview begins.
- `/orb:implement` searches the project source for existing code and patterns related to the spec's ACs.
- `/orb:review-pr` searches `decisions/` for architectural choices the implementation should respect.

### Changed
- README workflow diagram shows the multi-spec loop: dashed edge from Ship back to Design when the card goal is not yet met.
- End-to-end walkthrough describes iterative goal pursuit across multiple specs.

## [0.2.11] - 2026-04-11

### Changed
- `/orb:design` now reads the card's `specs` array as cumulative progress. Presents what each prior spec contributed and anchors the session on the gap between current state and goal.
- Design sessions no longer assume linear spec progression — specs may enhance a capability from different angles (infrastructure, data quality, tooling, adjacent work). The session surfaces which path the author intends.
- Interview record template includes goal, prior spec summary, and gap context.

### Added
- `CHANGELOG.md` backfilled from v0.2.0 through v0.2.10 (added in this release cycle).

## [0.2.10] - 2026-04-09

### Added
- `goal` field on cards — specific, measurable target at the current maturity. `so_that` is timeless (why); `goal` is current (what success looks like now). Goals evolve as the capability matures; git history tracks the progression.
- Sprint goal structure in CLAUDE.md — `/orb:setup` scaffolds a `Current Sprint` section listing the objective and card goals.
- README documents goals and sprint concepts.

## [0.2.9] - 2026-04-09

### Changed
- Replaced `priority` field (now/next/later) with `maturity` (planned/emerging/established) on cards. Cards describe capability state, not work priority.

### Added
- `specs` array on cards — lists the specs that have addressed each capability, giving a clear trail of work done.

## [0.2.8] - 2026-04-09

### Changed
- Distill now uses a staged **Draft → Review → Write** flow instead of per-card approve/edit/reject.
- All cards are drafted first and presented as a numbered batch.
- The agent surfaces overlaps, gaps, and low-confidence cards during review.
- Batch feedback (merge, split, drop, rename) replaces individual card gates.
- Nothing is written to disk until the author explicitly says "write."

## [0.2.7] - 2026-04-08

### Changed
- Cards are living documents — the lifecycle table (Open/In progress/Delivered/Closed) replaced with "Cards Are Living Documents" section.
- `cards/done/` directory removed from prescribed structure.
- Distill now accepts files, directories, or natural-language scope descriptions (not just a single file path).
- First-principles lens is always applied: "what does this product do?" not "what's planned next?"

### Added
- `CLAUDE.md` for the orbit repo — establishes that sessions here are about workflow refinement.

## [0.2.6] - 2026-04-08

### Changed
- Renamed `/orb:init` to `/orb:setup` to avoid collision with built-in `/init` command.

## [0.2.5] - 2026-04-08

### Changed
- Use "the author" for the human driving the workflow; reserve "the user" for end-users of the software being built.

## [0.2.4] - 2026-04-07

### Added
- `/orb:audit` skill — audit AC-to-test traceability across specs, finding untested code ACs, orphaned test prefixes, and coverage gaps.
- `ac_type` classification (code/doc/gate/config) for acceptance criteria.

## [0.2.3] - 2026-04-06

### Added
- `/orb:memo` skill — quickly jot rough ideas as freeform markdown in `cards/memos/`.
- README rewritten for orbit 0.3 — end-to-end walkthrough, "four ways in" section.

### Changed
- Evidence hierarchy added to interviewer, design, implement, and spec-architect skills.
- Implement skill proceeds after checklist without waiting for confirmation.

## [0.2.2] - 2026-04-05

### Fixed
- Implement skill: clarify that step 4 proceeds to write code after presenting the checklist.

## [0.2.1] - 2026-04-04

### Added
- Specs for cards 0001–0003.
- Implemented 0001-memos: freeform idea capture in `cards/memos/`.
- Implemented 0002-distill: extract cards from unstructured input.
- Implemented 0003-implement: pre-flight spec check with AC checklist.

### Added
- Cards for memos (0001), distill (0002), and implement (0003) features.

## [0.2.0] - 2026-04-03

### Added
- All 18 orbit workflow skills, README, and LICENSE.
- References field on cards; card-aware interview mode.
- Split interview into separate design and discovery skills.
- SessionStart hook for workflow context injection.

### Removed
- Evaluate and evolve skills (superseded by audit and design).
- `disable-model-invocation` from workflow skills.
