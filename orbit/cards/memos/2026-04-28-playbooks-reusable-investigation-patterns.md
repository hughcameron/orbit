# Playbooks: reusable investigation patterns

**Date:** 2026-04-28
**Source:** Nightingale's amount-variant-generators post-mortem + Carson/Hugh discussion

## Observation

Discovery sessions assume the author knows more than the agent and uses Socratic Q&A to extract that knowledge. This assumption inverts as agents accumulate domain expertise. Nightingale's subtype-collapse diagnostic arc — corpus counts → value-shape Jaccard → confident-wrong check → raw softmax top-k — has been run three times with consistent results. By the third run, the agent proposed skipping discovery entirely and going straight to measurement (Option C). Hugh agreed: "the questions zoom in on implementation and definition of success. Nightingale now has a better read on these than I do."

## The general concept

A **playbook** is a named, proven investigation sequence where:

- The steps are proven (run ≥2 times with consistent results)
- The agent can execute them mechanically
- The author's role shifts from "answer questions" to "approve the approach and review the findings"
- The output is evidence that shapes a spec (e.g., "hint-layer problem, not model problem")

A playbook replaces discovery, not spec. The finding still needs ACs, review, and implementation — the playbook just gets you to the finding faster than a Socratic interview.

## Where playbooks live

Playbooks are **project-scoped, not orbit-scoped.** FineType's subtype-collapse arc is meaningless outside FineType. The arcs belong in the project (agent definition or a project-level file). What orbit provides is the convention — a recognised pattern where an agent declares "I have a known playbook for this," presents the steps for approval, and proceeds to measurement.

## Design caution

Implementing playbooks as a new artefact type risks feature/artefact bloat. The orbit vocabulary already has cards, memos, interviews, specs, progress, reviews, decisions, rally state, and drive state. Adding another first-class type should be weighed against consolidation — the right treatment might streamline the existing flow rather than extend it.

Candidate approaches (not yet evaluated):

- **New artefact type** with its own file format and storage location. Maximum structure, maximum vocabulary expansion.
- **Card metadata** — playbooks as a property of the card they diagnose. Keeps them tied to capabilities but overloads the card format.
- **Design-stage fast path** — no stored artefact; the design skill recognises when an agent declares a known playbook and offers approval-then-execute instead of Q&A. Lightest touch; relies on agent memory rather than stored state.
- **Consolidation play** — revisit the interview/discovery/design boundary and fold playbooks into a unified "evidence gathering" stage that supports both Socratic and mechanical modes. Most ambitious; highest payoff if it simplifies.

## Status

Memo. Not scoped to a card or spec yet. Held here pending a design session — likely to surface when orbit's artefact vocabulary is next reviewed for consolidation.
