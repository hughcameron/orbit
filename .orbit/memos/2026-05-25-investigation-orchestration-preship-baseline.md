# Pre-ship per-repo baseline — investigation orchestration

**Date:** 2026-05-25
**Spec:** `.orbit/specs/2026-05-25-investigation-as-pipeline-step/spec.yaml` (ac-08)
**Purpose:** Locks the pre-ship per-repo baseline against which ac-08's +4w post-ship audit will compare. Captured BEFORE the first SKILL.md edit commit lands (verifiable from git log: this commit precedes any `plugins/orb/skills/*/SKILL.md` change for ac-02..ac-05).

## Method

Re-run of the 5-parallel-subagent JSONL analysis pattern. Methodology key: `parallel-subagent-jsonl-analysis` (retrieve via `orbit memory list | grep parallel-subagent` — `orbit memory search` does not match on key substrings).

Data source: today's empirical-audit run (≤12 hours old, no behavioural intervention in between). Documented in `.orbit/memos/2026-05-25-code-investigate-empirical-audit.md`. This memo extracts the per-repo lines into a stable per-repo baseline for the +4w comparison.

## Per-repo baseline (T-zero)

| Repo | Sessions sampled | Window | Inv-before-edit ratio | `/orb:code-investigate` invocations | Hook fires |
|---|---|---|---|---|---|
| **arcform** | 3 | 2026-04-18 → 2026-05-24 | 42.9% (126/294) | 0 | 6 (1 session, post-ship) |
| **orbit (dogfood)** | 10 | 2026-05-21 → 2026-05-25 | 42.3% (210/496) | 0 | 224 |
| **brightfield** | 2 | 2026-04-28 → 2026-04-29 | 47.1% (64/136) | 0 | 0 (pre-ship sample) |
| **repo P (private)** | 10 | 2026-05-23 → 2026-05-25 | 40.2% (178/443) | 0 | 247 |
| **finetype** | 10 | 2026-05-04 → 2026-05-25 | 41.3% (147/356) | 0 | 133 |
| **Aggregate** | **35** | | **~42.0%** (725/1725) | **0** | **610** |

## Comparator rules for the +4w audit

Per ac-08:

1. **Kill-judgement-bearing repo:** `orbit` (the dogfood). K1 (kill condition for behaviour-changing claim) fires if orbit's post-ship ratio hasn't moved measurably above 42.3%. Other repos report as supporting evidence.

2. **Lift target:** *sustained lift above each repo's own pre-ship baseline*. Same-repo comparison, not cross-repo. Mixing aggregate post-ship with a single-repo baseline (as the prior draft did) is mixed-population and rejected per cycle-2 review.

3. **Hook→action conversion target:** above-zero across all repos in pipeline-skill-entry sessions. Orchestrated `/orb:code-investigate` calls appear as `tool_use` with `name: "Skill"` and `input.skill = "orb:code-investigate"` — easy to count.

4. **Memory-write target:** at least one `code-investigate`-labelled memory captured *from* an investigation per active development week, aggregate across repos. (Today's count: 3 labelled memories, all *about* the skill; 0 *from* it.)

## Caveats locked in

- **arcform's "6 hook fires in 1 session" is post-ship**, not pre-ship — the session ran 2026-05-22, after the 2026-05-17 ship. Arcform's pre-ship behaviour is captured by its other two sessions (2026-04-18, 2026-04-29) which predate the hook. For ac-08's comparator, arcform's pre-ship hook-fire baseline is effectively 0.
- **orbit, repo P, finetype** all carry mixed pre/post-ship hook-fire numbers in their windows. The "baseline" we're locking is "current operational state before the new orchestration mechanism lands" — which includes the existing post-hoc hook firing for ~8 days. That's the correct comparator for a mechanism change.
- **Sample size per repo varies** (2 to 10 sessions). The +4w audit should sample at the same per-repo cadence (last-10-by-mtime, fewer if dir has fewer) for like-for-like comparison.

## What "moved measurably" means

ac-08 names "sustained lift" but doesn't fix a threshold. Lock for the +4w audit:

- **Measurable lift:** ≥5 percentage points above pre-ship baseline, sustained over the audit window's most-recent 5 sessions per repo.
- **Sustained:** the most-recent 5 sessions per repo show the same direction (all above baseline, or all below).
- If orbit shows ≥10pp lift and 4 of 5 other-repo comparisons also lift: clear pass on K1.
- If orbit shows <5pp lift OR negative: K1 fires; tabletop's pivot path (try lateral B/C or revisit cut) kicks in.

## Source files referenced

- `.orbit/memos/2026-05-25-code-investigate-empirical-audit.md` — the parent audit memo with aggregate numbers
- `.orbit/memories/parallel-subagent-jsonl-analysis.yaml` — the canonical methodology memory
- Source dirs (for re-running):
  - `/home/hugh/.claude/projects/-home-hugh-github-meridian-online-arcform/`
  - `/home/hugh/.claude/projects/-home-hugh-github-meridian-online-orbit/`
  - `/home/hugh/.claude/projects/-Users-hugh-github-meridian-online-brightfield/` (Mac-path; Linux equivalent `/home/hugh/...`)
  - `/home/hugh/.claude/projects/-home-hugh-github-hughcameron-hydrofoil/` (private repo P)
  - `/home/hugh/.claude/projects/-Users-hugh-github-meridian-online-finetype/`
