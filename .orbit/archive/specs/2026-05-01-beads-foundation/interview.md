# Design interview: Beads foundation

**Date:** 2026-05-01
**Participants:** Hugh, Carson
**Cards:** 0010-objective-functions (partial), 0005-drive (partial), 0006-rally (partial), 0009-mission-resilience (partial)

## Context

Orbit's execution layer (drive.yaml, rally.yaml, progress.md, decisions.md, session-context.sh) has grown into multiple overlapping state mechanisms producing 6-9 files per card. A trial replay of the UX uplift rally as beads (.orbit/discovery/beads-trial-findings.md) confirmed that beads' dependency graph, auto-ready query, and memory system replace the orchestration layer cleanly. Four design decisions were made (.orbit/choices/0011-beads-execution-layer.md) covering AC structure, cold-fork review, context injection, and ready-queue filtering.

This is spec 1 of 4 in a bottom-up migration (Cut A). It establishes the bead encoding conventions that the subsequent implement, drive, and rally specs depend on.

## Q&A

### Q1: Decomposition — one rally or sequenced specs?

**Cut A (bottom-up by dependency):** Ship in layers, each independently testable:
1. Foundation — gate convention, PRIME.md, promote flow
2. Implement — rewrite to read bead acceptance field
3. Drive — rewrite to use bead lifecycle
4. Rally — collapse into bead dependency graph

**Cut B (single rally):** One coordinated body of work.

**Decision:** Cut A. Foundation ships first, validates the gate convention before three skills are rewritten around it.

### Q2: Promote flow — what triggers it?

**Option A:** `/promote` is a skill Hugh runs (human checkpoint).
**Option B:** Agent promotes autonomously during drive (no separate step).

**Decision:** Option B. Agent promotes autonomously. Full autonomy is the default — all skills should be designed for autonomous operation first, with guided/supervised as the exception.

### Q3: Gate convention syntax

**Option A:** Inline `[gate]` markers, no numbering.
**Option B:** Numbered IDs with typed prefix: `ac-NN [gate]: description`.

**Decision:** Option B. Numbered IDs give stable references for "Current AC" tracking. Format is `ac-NN [gate]: description` for gates, `ac-NN: description` for non-gates. Gate ACs block subsequent ACs by declaration order.

## Summary

**Goal:** Establish the bead encoding conventions (acceptance field format, gate syntax, PRIME.md template, promote flow) that the subsequent implement, drive, and rally specs depend on.

**Constraints:**
- beads is the execution substrate; orbit encodes discipline in bead fields
- session-context.sh deprecated in favour of thin bd prime + progressive bd show
- Cold-fork review stays, reads acceptance field instead of spec.yaml
- Autonomy-first: skills assume full autonomy as default
- Cards and memos unchanged — direction layer stays as-is

**Decisions surfaced:**
- D1: Acceptance field with `ac-NN [gate]:` convention (decision 0011 D1)
- D2: Cold-fork stays (decision 0011 D2)
- D3: Thin bd prime via custom PRIME.md (decision 0011 D3)
- D4: Mode-appropriate ready query (decision 0011 D4)
- D5: Agent promotes autonomously during drive (this interview Q2)
- D6: Bottom-up decomposition, four sequenced specs (this interview Q1)
- D7: Full autonomy is the default operating mode (this interview Q2 context)
