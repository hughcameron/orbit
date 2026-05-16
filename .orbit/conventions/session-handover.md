# Session handover — v1 convention

This convention codifies the agent-facing discipline that turns the
per-card `Session` substrate (spec
`.orbit/specs/2026-05-16-session-handover/spec.yaml`, card
`.orbit/cards/0036-session-handover.yaml`) into a handover the next
session can read on first prime.

## The substrate

- `orbit session set-card <id>` writes a single-line `.orbit/.session-card`
  with the resolved canonical slug. The id accepts the same prefix-match
  semantics as `card.show` (full slug, padded `NNNN`, or bare unpadded
  number) per `.orbit/conventions/id-conventions.md`.
- `orbit session distill` reads stdin. When stdin is a Claude Code
  Stop-hook JSON envelope (the `hook_event_name == "Stop"` shape), the
  verb extracts `last_assistant_message` and writes it into
  `Session.distillate`. When stdin is plain text, the bytes land in
  `Session.distillate` verbatim (lossy UTF-8 conversion — the verb never
  panics on invalid input).
- `orbit session prime`'s envelope carries a `handover` field. When a
  most-recent Session exists, the field is populated and `next_step` is
  prefixed with `"Read the handover above before any other action. "`
  so the next agent reads the handover before any other action.
- `orbit session handover [--card <id>] [--since <iso>]` is the explicit
  per-card lookup verb. Use it when the session you want isn't the
  global latest — for example, returning to a card you haven't touched
  for a week.

## The agent's discipline

### Call `set-card` early in the session

As soon as the active card is known — typically once `/orb:implement`
or `/orb:drive` resolves the spec to its parent card — run:

```bash
orbit session set-card <id>
```

This is what scopes the handover. Without it, the Session lands without
a `card_id` and the per-card lookup in the next session won't find it.
The global-latest fallback still works, but it's coarser than the
per-card lookup the verb is designed to provide.

### The final assistant message before Stop IS the handover

The Stop hook (`.claude/settings.json` → Stop) pipes the JSON envelope
into `orbit session distill`. The verb extracts `last_assistant_message`
— so whatever the agent says last, that's what lands in the substrate
verbatim.

Write the final assistant message as a multi-paragraph reflection. The
register is discursive (per `.orbit/choices/0024-handover-register-is-discursive.yaml`),
not BLUF / Decision Brief — the audience is the next agent orienting,
not Hugh deciding. The format is freeform markdown; no required
sections, no template.

### Four orientation elements

The prose should cover these four elements somewhere in the body
(order is the agent's call):

1. **Current state of the work** — where the spec / card / pipeline sits
   right now, on disk and in flight.
2. **What was tried this session** — the moves the agent attempted,
   including dead-ends and partial wins. The texture matters; "tried X,
   didn't work because Y" beats "didn't ship X".
3. **Blockers or open threads** — anything the next session has to
   resolve before progress resumes (a half-done refactor, a pending
   review, an unanswered question).
4. **Next concrete step** — the single most useful thing the next
   session should do first.

### Mid-session re-set-card is legal and overwrites

If the active card shifts mid-session (rare, but legitimate — a
detour spawns a sub-task under a different card and now drives the
remainder of the session), calling `orbit session set-card <id>` a
second time replaces the slug in `.orbit/.session-card`. The Stop hook
then writes the most-recent card_id into the Session yaml — matches the
rest of orbit's "latest write wins" discipline.

### Direct verb-call leak case

If `orbit session distill` is invoked manually (a script, a replay
fixture, a test harness) without the Stop hook firing afterwards,
`.orbit/.session-card` is NOT auto-deleted. The file persists until the
next Stop hook runs or the operator removes it manually. This is a
deliberate scope choice: `distill` is read-only on `.session-card`; the
Stop hook (`.claude/settings.json` → Stop) owns deletion. Operators
running `distill` manually for fresh card-id contexts should clear the
file themselves.

## Failure modes this prevents

- **Re-discovery cost** — "I had to grep and ask before doing anything"
  is the visible failure when the next session has no handover. The
  agent burns time re-establishing context that the previous session
  already had.
- **Wrong direction** — "I picked up on a different angle and wasted
  the session" is the visible failure when the handover exists but is
  too terse to convey the previous agent's judgement. The discursive
  register exists specifically to carry the "I'd push here next"
  texture that BLUF compression discards.

## Enforcement surface

This convention is v1's enforcement surface — there is no front-matter
on `SKILL.md` files modified by this AC, matching the precedent from
spec `2026-05-15-agent-learning-loop` ac-08 for skill self-improvement.
The discipline is captured in prose; agents follow it because the
convention names it explicitly.

## References

- `.orbit/specs/2026-05-16-session-handover/spec.yaml` — the spec this
  convention serves; ACs 3, 4, 5, 6, 7, 8 are the substrate plumbing
  this discipline rides on top of.
- `.orbit/cards/0036-session-handover.yaml` — the capability card.
- `.orbit/choices/0024-handover-register-is-discursive.yaml` — the MADR
  recording the discursive-over-BLUF register decision.
- `.orbit/STYLE.md` — the BLUF / Decision Brief contract that governs
  every other agent-to-Hugh prose surface; choice 0024 carves out an
  explicit exception for the handover artefact.
