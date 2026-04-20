# orbit

This repo is the orbit workflow plugin for Claude Code. Sessions here are about **workflow refinement** — improving how orbit guides the card → design → spec → implement → review pipeline.

## What This Is

orbit is a Claude Code plugin that provides specification-driven workflow skills (`/orb:card`, `/orb:distill`, `/orb:design`, `/orb:spec`, `/orb:implement`, `/orb:review-pr`, etc.). The skills, hooks, and card format are the product.

## Working in This Repo

- **Skills live in** `plugins/orb/skills/<name>/SKILL.md`
- **Cards describe orbit's own capabilities** in `orbit/cards/`
- **Specs for orbit changes** live in `orbit/specs/`
- orbit uses itself — cards, specs, and decisions apply to orbit's own development

## Key Concepts

- **Cards are living documents.** They describe capabilities, not work items. Updated in place; git history is the audit trail.
- **First-principles lens.** Distill asks "what does this product do?" not "what's planned next?"
- **No backlogs.** Work flows through decisions and specs. Cards are the feature taxonomy.

## Deployment

The plugin is installed into projects via the Claude Code plugin marketplace. All development happens in this repo — installed copies in other projects receive updates via the marketplace.
