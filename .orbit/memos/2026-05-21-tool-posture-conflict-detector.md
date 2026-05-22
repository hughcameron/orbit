# Tool-posture conflict detector — /orb:setup + conformance extension

**Date:** 2026-05-21
**Status:** Memo — distil + tabletop when sequenced

## What

`/orb:setup` (especially in brownfield mode) gains a step that catalogues every project tool the local rules surface takes an explicit posture on. The catalogue lands at a stable substrate location (candidate: `.orbit/conventions/tool-posture.yaml`). Conformance reads it and emits a new finding family — `tool_posture_conflict` — when (a) two canonical rules-files contradict each other on the same tool, OR (b) the harness keeps nudging a tool the project rules are silent on. Mid-flight skills cite the manifest rather than deriving their own interpretation of the rules.

## Why this exists

The recurring failure mode: an agent reading a project's canonical-rules surface (CLAUDE.md + `.orbit/METHOD.md` + `.claude/settings.json`) hits a *silence*, not a contradiction. The agent then **extrapolates** — "rule X bans tool A; tool B looks like A; therefore B is banned too" — and narrates the extrapolation as if it were a project override. The agent's reasoning is self-consistent but wrong; subsequent harness nudges re-trigger the cycle.

Concrete instance (sanitised): a downstream consumer repo's CLAUDE.md says "do not use tool A" and is silent on tool B. The plugin's METHOD.md explicitly says tool B is allowed in-session. The harness fires a periodic `task_reminder` attachment because no tasks are open. The agent reads the silence on B as a transitive ban (extrapolating from A), narrates the override, and continues. Four corrective edits to the rules-files haven't stopped it — each edit clarifies B-specific posture, but the next session's agent re-extrapolates from a different gap. The problem is *agent inference under silence*, not rule wording.

The harness `task_reminder` itself is a system-level attachment in Claude Code (verified in transcript JSONL — `"attachment":{"type":"task_reminder","content":[]}` injected periodically). It is not a hook output; no `settings.json` hook event intercepts it.

## What's load-bearing

**Part 1 — Setup-side detector.** /orb:setup walks the canonical rules surface (CLAUDE.md, METHOD.md, STYLE.md, project `.claude/settings.json` env + permissions + hooks, plus any `.orbit/conventions/`) and extracts every directive-shaped statement keyed by tool name. Output is a manifest: for each tool of interest, posture (`allowed` / `forbidden` / `recommended` / `discouraged` / `silent`) plus the source file + line. Detection runs by tool-name match (TaskCreate, TodoWrite, AskUserQuestion, `/reload-plugins`, /orb:* skill names, common Bash patterns) plus directive-shaped lemmas ("do not", "use", "is fine", "is forbidden", "prefer", "ban", "allow").

**Part 2 — Conformance finding family.** Two finding shapes:
- `tool_posture_conflict` — two files name contradictory postures on the same tool. Severity: HIGH. Remediation: re-run `/orb:setup` with the manifest as authoritative; the operator picks which posture wins.
- `tool_posture_silent_with_nudge` — the harness or a global instruction is known to push for a tool the project rules are silent on. Severity: MEDIUM. Remediation: `/orb:setup` re-runs and asks the operator to take an explicit position (allowed / forbidden / project-specific).

**Part 3 — Mid-flight escalation.** When a skill encounters a posture-shaped decision in the live work (e.g. "should I call TaskCreate here?"), it reads the manifest deterministically. If the manifest says `silent`, the skill prompts `/orb:setup` re-run rather than extrapolating. **This part only works if the manifest exists** — without Part 1, Part 3 collapses back into agent vibes (the original failure mode).

## Why it does not conflict with existing conformance

Today's conformance is mechanical: byte-compare of plugin-canonical files, cards-by-maturity, memo age, version pin, topology pointers. It does not check *rule-content consistency across rules-files*. The tool-posture detector is **additive** — same envelope shape, new finding family, same remediation pattern (`remediation.verb` points at `/orb:setup`). The plugin-canonical-file byte-compare keeps catching drift in the substrate prose; the tool-posture detector catches drift in the *interpretation surface*.

## Risks and constraints

- **False positives on directive-extraction.** "Do not use X" matched by substring can fire on prose like "We do not use X because..." (explanation, not directive). Mitigation: extraction needs to be conservative — flag only when the lemma is in an imperative position (top of a list, bold heading, dedicated paragraph). Tabletop-time decision on the heuristic.
- **Part 3 must be deterministic.** If the mid-flight skill is asked to "look for conflicts" without reading the manifest, it will reproduce the original failure mode (agent extrapolation). The escalation MUST read the structured manifest file; if the manifest is absent, the skill silently proceeds rather than guessing.
- **Tool-name list is not closed.** New tools, new MCP servers, new slash-commands all join the surface over time. The manifest's tool-name set needs to be regenerated periodically — most naturally as part of `/orb:setup` re-runs, or as a separate audit verb.
- **Brownfield ergonomics.** First-time brownfield runs will surface many `silent` findings (the project hasn't taken a position on most tools). The operator needs a workflow that's "accept defaults" friendly — probably AUQ at setup time with the global defaults pre-selected, so the operator only intervenes on tools they want to override.

## Card mapping

Likely fits card **0032-brownfield-spec-migration** as a new spec — the substrate migration story already covers "make a brownfield repo land on the canonical orbit shape", and this is the same shape one layer up (canonical *interpretation surface*, not just canonical files). Worth checking against card 0039-workflow-conformance too — the finding-family lives there structurally.

Open: does this warrant its own card? Probably not — METHOD.md's "follow-up cards is usually wrong" rule says map to an existing card unless the capability is genuinely new. The capability ("make rules-surface drift mechanically visible") is squarely inside conformance's brief.

## Suggested next

1. `/orb:distill .orbit/memos/2026-05-21-tool-posture-conflict-detector.md` when prioritised — verdict: card-update against 0032 or 0039, not a new card.
2. `/orb:tabletop` against the card to settle Part 1's extraction heuristic, the manifest's on-disk shape, and the two finding-family signatures.

## Related substrate

- Card 0039 — workflow-conformance (the audit verb this finding family extends)
- Card 0032 — brownfield-spec-migration (the setup-side migration story this extends)
- Memo 2026-05-21-tabletop-nogo.md — names a related precedent of "amend an AC inline after probe evidence" — the manifest's `silent` finding plays a similar role for rules-surface evidence
- The recurring harness `task_reminder` attachment surfaced in this same window — system-level, not gated by project settings, motivates the `_silent_with_nudge` finding shape
