---
status: accepted
date-created: 2026-04-20
date-modified: 2026-04-20
---
# 0005. Drive's Review Artefact Contract: File-on-Disk Authoritative

## Context and Problem Statement

When drive launches a review as a forked Agent, the fork returns both a chat response and (typically) a saved review file on disk. Drive must decide which is the authoritative source for the verdict. A naive design would consult both and try to reconcile; a robust design picks one and is explicit.

The question also covers failure taxonomy: the fork can fail in multiple ways — agent errors before writing the file, agent errors after writing it, file exists but contains no parseable verdict, chat claims APPROVE while file disagrees. A flat rule is simpler than per-mode handling.

## Considered Options

- **Option A: File-on-disk authoritative; chat never read.** Drive parses only the review file at the cycle-specific path. Agent-level success/failure status is irrelevant. Chat response is informational at most.
- **Option B: Chat authoritative with file as audit trail.** Drive reads the verdict from the structured chat return; the file exists for humans to read but drive doesn't parse it.
- **Option C: Both, with reconciliation.** Drive reads both; if they agree, proceed; if they disagree, escalate or retry.
- **Option D: Agent success status + file.** If the forked Agent reports success, read the file; if it reports failure, escalate without parsing anything.

## Decision Outcome

Chosen option: "Option A — File-on-disk authoritative; chat never read", because it treats reviews as a stateless produce-a-file function with a single contract surface. The failure taxonomy collapses to one rule: *is there a parseable verdict in the file?* Every failure mode — agent crash, incomplete write, mismatched chat/file, ignored brief — produces the same observable: no parseable verdict. Drive's response is uniform (retry once, then escalate).

This also aligns with the architectural stance from MetaGPT and similar agent frameworks: artefact-path contracts are durable across session boundaries; session state is not. By refusing to consult chat, drive becomes resilient to any fork-internal quirks and gains a clean fork-boundary.

### Consequences

- Good, because the failure taxonomy is minimal — one rule, one response
- Good, because drive's logic doesn't depend on the Agent tool's return-value shape (which could change across Claude Code versions)
- Good, because chat-vs-file disagreement is impossible by construction — drive never reads chat
- Good, because forks that produce the file successfully but crash during summarisation still succeed (the file is the product; chat is spare)
- Bad, because a review where the fork wrote the file but the file is malformed looks identical to a review where the fork silently ignored the brief — both trigger retry. Accepted: the retry is cheap, and the escalation message is actionable either way.
- Bad, because drive cannot surface the forked Agent's summary in its own chat output to the user — mitigated by drive referencing the saved review path in its status messages, letting the user read the full review directly.

## Evidence

- Constraint #3 in `orbit/specs/2026-04-20-drive-forked-reviews/spec.yaml`: "File-on-disk is the only authoritative source for the verdict. Drive does not consult the forked agent's chat response under any circumstance."
- Interview Q2 documented the collapse of failure-mode complexity: "The agent's chat response is not consulted. Agent-level success/failure status is irrelevant — only the artefact matters."
- Implementation in `plugins/orb/skills/drive/SKILL.md` §5.3: "Drive does not parse the chat response for the verdict — the file on disk is the only authoritative source."
