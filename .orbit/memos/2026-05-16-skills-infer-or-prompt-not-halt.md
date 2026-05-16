`/orb:*` skills that take a spec-id (review-pr, implement, design, drive, review-spec) silently halt when `ARGUMENTS` is empty. From the user end this looks like the skill failed; really it's a guard. Hit it tonight on `/orb:review-pr` right after shipping card 0036.

The substrate to fix it just landed. `set-card` / `.orbit/.session-card` (ac-04 of 2026-05-16-session-handover) gives every spec-taking skill a free fallback target. Mirror what `session distill` now does: when no arg is supplied, read `.session-card`; if still empty, list open specs and confirm with one `AskUserQuestion`; halt only when both fail.

Scope is a cross-skill convention, not a one-skill patch — same guard exists in every `/orb:*` skill with a required arg. Right shape is probably a new convention doc at `.orbit/conventions/skill-no-arg-fallback.md` plus one PR touching each affected SKILL.md.

Worth ranking against the three other open memos before distill (search-and-code-mastery, setup-conformance, style-md-softening).
