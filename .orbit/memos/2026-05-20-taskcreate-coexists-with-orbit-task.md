# METHOD.md task rule reads as anti-TaskCreate; intent was persistence, not prohibition

**Date:** 2026-05-20
**Source:** observed agent behaviour in a recent session (downstream project)

## Observation

METHOD.md substrate rules currently say:

> Use `orbit` verbs for ALL task and spec tracking — do NOT use `TodoWrite`, `TaskCreate`, or markdown TODO lists.

An agent in a recent session read this literally and announced *"Per project rules, task tracking lives in orbit-state, not TaskCreate — ignoring that reminder"* when the Claude Code harness suggested `TaskCreate`. The agent obeyed the rule by refusing the tool.

A later session (2026-05-20, orbit-internal) repeated the pattern in a more pronounced form:

> *"Ignoring the TaskCreate reminder per project rules — using orbit substrate. Let me execute: first check orbit verbs, then apply edits, then persist."*

Same shape: agent reads the rule literally, announces the refusal, then narrates the alternative plan. The harness keeps nudging; the rule keeps shutting it down. Two instances now — same friction from both sides.

That isn't the intent. Hugh's framing: *"Task creation is really valuable and we don't want to compete with it — we just want to keep track of them."*

## The actual distinction

The two tools serve different windows:

- **`TaskCreate` / `TaskUpdate` / `TaskList`** — ephemeral in-session structure. Lives in the harness's task list. Survives nothing past the session.
- **`orbit task open / claim / update / done`** — substrate-persistent. Lives in `.orbit/tasks/` (or equivalent). Survives session resets, syncs across machines via git, integrates with specs.

These are not competing surfaces. They are complementary: `TaskCreate` is the agent's working memory for the current session; `orbit task` is durable state that needs to outlast the session.

## Why the current wording mis-fires

The rule as written ("do NOT use TaskCreate") sits in a hard list with `TodoWrite` and `markdown TODO lists` — both of which ARE failure modes (markdown TODO lists are unreadable substrate; `TodoWrite` is a now-deprecated harness verb). `TaskCreate` doesn't belong in that grouping. The intent was "no ephemeral-only tracking for things that matter across sessions," but the wording reads as "TaskCreate is banned."

The agent in the recent session was honouring the substrate. The rule needs to honour the agent's working tools back.

## Suggested refinement (for the eventual rewrite)

Replace the current bullet with two:

- Use `orbit task open / claim / update / done` for **cross-session task tracking** — anything that needs to survive session resets, sync across machines, or integrate with specs.
- `TaskCreate` is fine for **in-session working structure** — multi-step tasks in the current session, especially when the harness's reminders are nudging you to use it. Mirror to `orbit task` only when the task is substantive enough to need persistence.

The boundary is "does this need to outlast the session?" — not which tool the agent reaches for first.

## Adjacent substrate noise

The repeated `TaskCreate` reminders that fired during *this* session (the substrate-engagement rally) are evidence of the same friction from the other direction — the harness keeps nudging, the agent keeps declining per METHOD.md, neither side updates. Updating the rule would quiet both surfaces.

## Status

Memo only. The rule change touches METHOD.md (plugin-shipped), which means a small spec to (a) re-word the rule, (b) optionally ship the canonical METHOD.md update via the existing release flow. Could pair with the STYLE.md plugin-shipping work already queued (`.orbit/memos/2026-05-16-style-md-less-executive-more-practical.md`) — same surface, similar shape.
