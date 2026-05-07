# orbit

This repo is the orbit workflow plugin for Claude Code. Sessions here are about **workflow refinement** — improving how orbit guides the card → design → spec → implement → review pipeline.

## What This Is

orbit is a Claude Code plugin that provides specification-driven workflow skills (`/orb:card`, `/orb:distill`, `/orb:design`, `/orb:spec`, `/orb:implement`, `/orb:review-pr`, etc.). The skills, hooks, and card format are the product.

## Working in This Repo

- **Skills live in** `plugins/orb/skills/<name>/SKILL.md`
- **Cards describe orbit's own capabilities** in `.orbit/cards/`
- **Specs for orbit changes** live in `.orbit/specs/`
- orbit uses itself — cards, specs, and decisions apply to orbit's own development

## Key Concepts

- **Cards are living documents.** They describe capabilities, not work items. Updated in place; git history is the audit trail.
- **First-principles lens.** Distill asks "what does this product do?" not "what's planned next?"
- **No backlogs.** Work flows through decisions and specs. Cards are the feature taxonomy.

## Orbit vocabulary

Each artefact has one job. Don't invent new names — if something doesn't fit, it probably needs a different existing artefact, not a new one.

| Artefact    | Where                                                       | What it is                                                                                                                   |
|-------------|-------------------------------------------------------------|------------------------------------------------------------------------------------------------------------------------------|
| Card        | `.orbit/cards/NNNN-<slug>.yaml`                              | A capability the product provides. Written in user language. Never closed — updated in place as the capability evolves.       |
| Memo        | `.orbit/cards/memos/<date>-<slug>.md`                        | Raw idea awaiting distillation. Freeform markdown. Turned into cards via `/distill`. Deleted after promotion.                |
| Interview   | `.orbit/specs/<date>-<slug>/interview.md`                    | Q&A record from a `/design` or `/discovery` session. Feeds the spec.                                                          |
| Spec        | `.orbit/specs/<date>-<slug>/spec.yaml`                       | A discrete unit of work with numbered acceptance criteria. One card may spawn many specs over time.                          |
| Progress    | `.orbit/specs/<date>-<slug>/progress.md`                     | AC tracker maintained during implementation. The implementation diary.                                                       |
| Review      | `.orbit/specs/<date>-<slug>/review-{spec,pr}-<date>.md`      | Verdict artefact from `/review-spec` or `/review-pr`.                                                                         |
| Decision    | `.orbit/choices/NNNN-<slug>.md`                            | MADR record of an architectural choice. Referenced by specs that respect it.                                                 |
| Rally state | `.orbit/specs/<date>-<slug>-rally/rally.yaml`                | Durable state for a multi-card rally. Owned by the rally lead. Rally folders live alongside card spec folders — no separate archive.|
| Drive state | `.orbit/specs/<date>-<slug>/drive.yaml`                      | Durable state for a single-card drive. Owned by the drive agent.                                                             |

**Cards describe *what*, specs describe *work*.** When someone asks to "make a card for X":

- Is X a capability the product provides? → card in `.orbit/cards/`.
- Is X a discrete piece of work with acceptance criteria? → spec via `/design` + `/spec`.
- Is X a rough idea you don't want to lose? → memo via `/memo`.
- Is X a retrospective, options memo, or investigation plan? → none of the above. Retrospectives update the card they're about; options memos become `/discovery` sessions; investigation plans become specs.

**Cards never close.** A card may reach `maturity: established` and stop acquiring specs, but it isn't archived or deleted. The card is the product's self-description; the specs underneath it are the work.

**"Follow-up cards" is usually wrong.** If a session surfaces follow-up work, it's almost always new specs against existing cards — not new cards. New cards are for new capabilities, not for splitting work.

## Deployment

The plugin is installed into projects via the Claude Code plugin marketplace. All development happens in this repo — installed copies in other projects receive updates via the marketplace.


<!-- BEGIN BEADS INTEGRATION v:1 profile:minimal hash:ca08a54f -->
## Beads Issue Tracker

This project uses **bd (beads)** for issue tracking. Run `bd prime` to see full workflow context and commands.

### Quick Reference

```bash
bd ready              # Find available work
bd show <id>          # View issue details
bd update <id> --claim  # Claim work
bd close <id>         # Complete work
```

### Rules

- Use `bd` for ALL task tracking — do NOT use TodoWrite, TaskCreate, or markdown TODO lists
- Run `bd prime` for detailed command reference and session close protocol
- Use `bd remember` for persistent knowledge — do NOT use MEMORY.md files

## Session Completion

**When ending a work session**, you MUST complete ALL steps below. Work is NOT complete until `git push` succeeds.

**MANDATORY WORKFLOW:**

1. **File issues for remaining work** - Create issues for anything that needs follow-up
2. **Run quality gates** (if code changed) - Tests, linters, builds
3. **Update issue status** - Close finished work, update in-progress items
4. **PUSH TO REMOTE** - This is MANDATORY:
   ```bash
   git pull --rebase
   bd dolt push
   git push
   git status  # MUST show "up to date with origin"
   ```
5. **Clean up** - Clear stashes, prune remote branches
6. **Verify** - All changes committed AND pushed
7. **Hand off** - Provide context for next session

**CRITICAL RULES:**
- Work is NOT complete until `git push` succeeds
- NEVER stop before pushing - that leaves work stranded locally
- NEVER say "ready to push when you are" - YOU must push
- If push fails, resolve and retry until it succeeds
<!-- END BEADS INTEGRATION -->
