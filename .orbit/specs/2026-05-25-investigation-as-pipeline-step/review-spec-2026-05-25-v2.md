# Spec Review

**Date:** 2026-05-25
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-25-investigation-as-pipeline-step
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 3 |
| 2 — Assumption & failure | Pass 1 MEDIUMs + content signals (hook config, cross-skill pipeline integration, observation-band audit) | 2 |
| 3 — Adversarial | not triggered | — |

## Findings

### [MEDIUM] ac-01's bypass-reason log targets substrate that doesn't exist
**Category:** missing-requirement
**Pass:** 1
**Description:** ac-01 names the canonical bypass shape as landing in "the spec folder's progress.md (for spec-bound stages) or the session-card's notes (for session-bound stages)." Neither surface is the current orbit substrate.
- `progress.md` was deprecated. `plugins/orb/skills/implement/SKILL.md:26-30` explicitly states "Earlier `/orb:implement` revisions mirrored ACs into a `progress.md` file… All of that is now subsumed by the orbit-state substrate." `plugins/orb/skills/review-pr/SKILL.md:49` confirms: "the spec's `acceptance_criteria` field replaces the earlier `progress.md` tracker."
- "session-card" returns zero matches across `plugins/orb/skills/`. There is no defined surface by that name.
- The actual canonical durable-state surface for spec-bound notes is `orbit spec note <spec-id> "<message>"`, which writes to `notes.jsonl` (this spec's own cycle-2 reframe entry sits in `.orbit/specs/2026-05-25-investigation-as-pipeline-step/notes.jsonl`).

**Evidence:**
- `grep -rn "session-card\|session.card" plugins/orb/skills/` → no matches.
- `plugins/orb/skills/implement/SKILL.md:357-360` names `orbit spec note <spec-id> "NO-GO: …"` as the canonical bypass-trail surface for spec-bound stages.
- `.orbit/specs/2026-05-25-investigation-as-pipeline-step/notes.jsonl` exists and is the substrate-supported persistent stream.

**Recommendation:** Rewrite ac-01's bypass-shape clause to land bypass reasons via `orbit spec note <spec-id> "investigation bypass: <reason>"` for spec-bound stages (implement, review-pr). For session-bound stages (researcher, tabletop pre-spec), name the actual surface — likely `orbit memory remember --label code-investigate "<bypass reason>"` since memories are the only session-bridging persistence available today. Drop "progress.md" and "session-card" from the spec text; both are dangling references.

### [MEDIUM] ac-07's "tightened path filter" describes a loosening, not a tightening
**Category:** constraint-conflict
**Pass:** 1
**Description:** ac-07 specifies the hook's path filter should be "tightened to skip Edit/Write on files inside `.orbit/specs/<id>/` and `.orbit/cards/<id>/` where the pipeline already orchestrated; the filter is conservative (skip only when a marker session-id-matched entry exists for the parent scope)."
But the current `code-investigate-nudge.sh` filter at line 40 already skips ALL paths under `.orbit/`, `.claude/`, and `*.lock` unconditionally — no marker check needed: `case "$rel_path" in .orbit/*|.claude/*|*.lock) exit 0 ;; esac`. Spec/card folders are already exempt. ac-07 as written would convert the unconditional skip into a marker-conditional skip, which **fires the warning more often** on `.orbit/*` paths (those without matching marker entries), not less. That is a behavioural reversal of "tighten".

**Evidence:** `plugins/orb/hooks/code-investigate-nudge.sh:40` — current filter is the unconditional `.orbit/*|.claude/*|*.lock` case statement.

**Recommendation:** Resolve the intent. Two paths:
1. **If the goal is "the hook should warn more on non-orbit paths because pipeline edits land outside .orbit/"** — rewrite ac-07 to either (a) narrow the current `.orbit/*` skip to only the specific subdirs that pipeline skills actually edit (e.g. `.orbit/cards/*`, `.orbit/specs/*`, `.orbit/memos/*`) and let non-substrate paths inside `.orbit/` get warnings, or (b) leave the filter alone and only change the warning text.
2. **If the goal is "leave the substrate skip alone, only sharpen the warning text"** — drop the path-filter clause from ac-07 entirely and keep only the warning-text sharpening.
The current ac-07 text describes a change that is incoherent against the existing hook behaviour. The verification clause ("Trigger a Write on a spec folder file after marker exists; assert no warning fires") will pass trivially under current behaviour without any code change at all — the assertion already holds because of the unconditional skip — which makes ac-07's "implementation" untestable as a delta.

### [MEDIUM] ac-03's scope-derivation source naming is internally inconsistent
**Category:** test-gap
**Pass:** 1
**Description:** ac-03 instructs tabletop's agent to derive Q8 broad-mode scope from "the cluster cards' references[] (filter to file-path entries) and any code-area names surfaced in Q1-Q5". But tabletop's Q1-Q5 (per `plugins/orb/skills/tabletop/SKILL.md:117-149` and this spec's own `tabletop.md`) cover Goal / Values / Trade-offs / Halt conditions / Laterals — none of these questions enumerate code areas. Code areas surface at Q8 itself (Adjacent code) — which is the question whose broad-mode investigation step ac-03 is wiring. The instruction therefore tells the agent to derive Q8's scope from Q8's own output, which is circular at first entry.

**Evidence:**
- `plugins/orb/skills/tabletop/SKILL.md:117-169` — Q1-Q5 enumerated: Goal, Values, Trade-offs, Halt, Laterals.
- `.orbit/specs/2026-05-25-investigation-as-pipeline-step/tabletop.md:99` — this spec's own Q8 is where adjacent code first appears.
- ac-01's per-stage principle gives tabletop Q8 scope as "Q8's named layers + the cluster cards' references[]" — same internal contradiction (Q8 deriving from Q8).

**Recommendation:** Settle on cluster cards' `references[]` (file-path entries) as the sole derivation source for tabletop Q8 broad mode. Drop the "code-area names surfaced in Q1-Q5" clause. If the intent was "any file paths the author has already named earlier in the session," say that directly and name the substrate (the tabletop.md sidecar being authored at this point in the session, if any). Mirror the fix in ac-01's per-stage enumeration so the choice file matches.

### [MEDIUM] ac-08's per-repo baseline rests on a memory that the search doesn't find by substring
**Category:** assumption
**Pass:** 2
**Description:** ac-08 names the `parallel-subagent-jsonl-analysis` memory as the methodology backbone for both pre-ship and post-ship measurement. The memory does exist (confirmed via `orbit memory list`), but `orbit memory search parallel-subagent-jsonl` returns zero matches — the substring-search verb doesn't surface it because the search appears to match on a different field. An implementer running `orbit memory search <obvious-keyword>` to locate the methodology may conclude the memory isn't there and re-invent the analysis pattern, producing methodology drift between pre-ship and post-ship runs.

**Evidence:**
- `orbit memory list` shows `parallel-subagent-jsonl-analysis` with full body present.
- `orbit memory search parallel-subagent-jsonl` returns "(no memories)".
- The substring of the key is in the key itself; the search verb evidently scans bodies/labels but doesn't match key prefixes.

**Recommendation:** Add a one-line method note to ac-08 instructing the implementer to retrieve the methodology via `orbit memory list | grep parallel-subagent` (or via `orbit memory search <body-keyword>` like "JSONL" or "subagent") rather than substring-on-key. Alternatively, file a follow-on note against orbit-state to extend `memory search` to match keys, but that's out of scope here. The cheap fix is to name the retrieval shape inside ac-08 so methodology drift can't happen because the implementer can't find the memory.

### [MEDIUM] Skill-tool args reliability undermines ac-02..ac-05's orchestration contract
**Category:** failure-mode
**Pass:** 2
**Description:** ac-02..ac-05 specify that pipeline skills invoke `/orb:code-investigate` via the Skill tool with an agent-supplied scope argument. Memory `slash-command-args-vs-skill-tool-args` documents an empirically-observed failure mode: "The Skill tool with args parameter — previously believed to be the reliable path — IS NOT RELIABLE for forked skills." When the args drop, the called skill runs with an empty scope, the marker fires (so ac-02's smoke-marker verification passes), but the investigation covers the wrong files. The 610/0 conversion the spec is trying to fix could re-emerge in a new guise — orchestration fires, but with the wrong scope, and ratio-based ac-08 measurements may not catch it.

**Evidence:**
- `orbit memory list` → `slash-command-args-vs-skill-tool-args` body: "I hit the Skill tool dropping args TWICE in a row on /orb:review-spec… The Skill tool with args parameter… IS NOT RELIABLE for forked skills."
- ac-02 verification: marker presence + scope-supplied-via-AUQ — but the verification doesn't assert the marker's `entry_path` field matches the agent's intended scope (only that an entry exists).

**Recommendation:** Strengthen ac-02..ac-05's smoke-marker verification to assert the marker's entry path matches the agent's declared scope (not just "an entry exists"). One concrete shape: the orchestrating skill writes the agent's intended scope to a known location (e.g. `orbit spec note <id> "investigation scope: <paths>"`) immediately before the Skill call; the verification checks that `notes.jsonl`'s scope line and `.code-investigate-recent`'s file/scope entries match. This makes scope-drop failures testable in-cycle rather than waiting on ac-08's +4w window. Alternatively, weaken the contract: if the args-drop risk is judged high, mandate the Agent-tool fallback pattern from the memory and update ac-02..ac-05 prose accordingly.

---

## Honest Assessment

The cycle-2 reframe correctly addressed the cycle-1 HIGH (`adjacent_files` gone, replaced with agent-typed substrate the agent already reads) and most of the cycle-1 MEDIUMs (smoke-marker verification, sharpened review-pr resolution, retain-as-backstop hook, per-repo pre-ship baselines). The spec is structurally closer to shippable than it was 24 hours ago. What blocks REQUEST_CHANGES → APPROVE is that the reframe introduced four new gaps that are all fixable with text edits in the spec, not rework of the design:

1. **Two dangling substrate references** (ac-01's `progress.md` and `session-card`) — these are paste-throughs from older orbit conventions that have been superseded. Use `orbit spec note` and `orbit memory remember --label code-investigate` respectively.
2. **An incoherent hook-filter "tightening"** — ac-07 describes a change that is actually a loosening of the current behaviour, and the verification passes without code change. Pick one of the two intent paths in finding #2.
3. **A circular scope-derivation reference** — ac-03 (and ac-01's mirror) tells tabletop Q8 to derive scope partly from itself.
4. **An args-reliability risk** that the in-cycle verification doesn't catch — strengthen the smoke-marker check to assert scope content matches intent, or pin the Agent-tool fallback in the prose.

None of these change the load-bearing pick (orchestrate as mechanism, retain hook as backstop, per-repo measurement). All four are 1-3 line edits to the spec text. After those, the spec is ready for implement.
