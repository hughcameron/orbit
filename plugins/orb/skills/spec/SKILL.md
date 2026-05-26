---
name: spec
description: Generate a structured YAML specification with numbered ACs from interview results
---

# /orb:spec

Generate a validated specification from interview results or conversation context.

## Usage

```
/orb:spec [interview_file]
```

## Recall pre-flight

Before crystallising the interview or tabletop note into a spec, surface
prior substrate touching the topic so the spec doesn't duplicate prior
decisions or reinvent earlier ACs. This is a **structural step** at
skill entry, not advice. Per spec
2026-05-25-recall-verb-and-skill-step ac-05 and card 0044
(substrate-recall) — the pull-mode counterpart to the substrate-push
hook surface.

**Scope derivation.** Pick the topic from the input artefact's goal or
title — the interview.md `Card:` line, the tabletop.md `Cards in
scope:` set, or the conversation-context summary the agent is acting
on.

**Invocation.**

```bash
orbit recall "<topic>" --json | jq -r '.data.result.matches[] | "\(.score) \(.type) \(.id)\t\(.path)\n  \(.snippet)"' | head -20
```

Quote a 3-5 line summary inline. Cite any prior spec the recall surfaces
in the new spec's goal or notes so the lineage is explicit. Zero matches
is a valid outcome — log "recall: no prior substrate on `<topic>`" and
proceed.

## Instructions

### 1. Gather Interview Context

The input artefact may be a full **interview** (`interview.md`) from an open or partial design space, or a short **tabletop note** (`tabletop-note.md`) from a closed design space. Both are valid inputs — `/orb:spec` does not require a Q&A record. The closed-space path produces a tabletop note instead of an interview, and that tabletop note is a sufficient handoff (see `/orb:tabletop` §3–§4).

- If an interview or tabletop-note file path is provided: Read it
- If no file path: Check conversation history for a recent `/orb:tabletop` or `/orb:discovery` session — and look in `.orbit/specs/YYYY-MM-DD-<topic-slug>/` for either `interview.md` or `tabletop-note.md`
- If neither: Ask the author what to crystallise

**Cite the user-voice paragraph as the intent contract.** Both the interview template and the tabletop-note template carry a top-of-file **What good looks like** paragraph — written from the user's seat, in the author's idiom. When this paragraph is present in the input artefact, the generated spec quotes or directly references it as the intent contract — not only the structured Q&A or the deferred-items list. Concretely: the paragraph appears in the spec's `goal` (if it compresses to one sentence) or in `notes` / a leading note (if it doesn't), so the implementing agent reads prose-level user intent, not just answers to questions.

### 2. Assess Ambiguity

Score clarity before generating:

- **Goal Clarity** (40% weight): Is the goal specific?
- **Constraint Clarity** (30% weight): Are constraints specified?
- **Success Criteria Clarity** (30% weight): Are criteria measurable?

**Formula:** `ambiguity = 1 - (goal * 0.40 + constraints * 0.30 + criteria * 0.30)`

**Threshold:** Ambiguity must be ≤ 0.2. If higher, suggest returning to `/orb:tabletop` or `/orb:discovery`.

```
Ambiguity Assessment:
  Goal Clarity:             X% (weight: 40%)
  Constraint Clarity:       X% (weight: 30%)
  Success Criteria Clarity: X% (weight: 30%)
  Overall Ambiguity:        X.XX (threshold: ≤ 0.2)
  Ready for Spec:           Yes/No
```

### 3. Generate the Spec

Adopt the spec-architect role (see `/orb:spec-architect` for extraction guidelines).

Every acceptance criterion gets a sequential `ac-NN` ID. These IDs are used by implementers to prefix test function names. The `test_prefix` in metadata disambiguates ACs when a project has multiple specs:

```yaml
goal: "Clear primary objective"

constraints:
  - "Hard limitation 1"

acceptance_criteria:
  - id: ac-01
    ac_type: code        # code | doc | gate | config
    description: "Measurable criterion"
    verification: "How to verify"

implementation_notes:          # means-level leads from the tabletop session — not constraints
  - "Starting context for the implementing agent"

ontology_schema:
  name: "DomainModel"
  description: "What this models"
  fields:
    - name: "field_name"
      type: "string"
      description: "What this field represents"

evaluation_principles:
  - principle: "Quality dimension"
    weight: 0.3

exit_conditions:
  - "When to stop iterating"

metadata:
  version: "1.0"
  test_prefix: "remat"  # short label for this spec — disambiguates ACs across specs
  timestamp: "YYYY-MM-DDTHH:MM:SSZ"
  ambiguity_score: 0.15
  interview_ref: ".orbit/specs/YYYY-MM-DD-<topic>/interview.md"  # or tabletop-note.md for closed-space inputs
```

### 4. Record memories considered

Lift the tabletop-time memory reconciliations into the spec's `memories_considered` field. The tabletop session ran `orbit memory match <card-slug>` and captured a disposition for each matching memory under **Implementation Notes**. Each entry becomes:

```yaml
memories_considered:
  - key: drive-autonomy-default-to-action
    disposition: adopted                # adopted | partially-adopted | not-applicable
    reason: "wired the close-time gate as the memory recommends"
  - key: prime-relevance-overlap-heuristic
    disposition: partially-adopted
    reason: "reused the label-overlap idea; rebuilt the ranker for token + label weighting"
  - key: state-shape-vs-mechanism
    disposition: not-applicable
    reason: "this spec touches enforcement, not memory authorship"
```

Per spec 2026-05-19-memory-gates-decisions ac-03 (D3a): `memories_considered` is a top-level `Spec` field — uniform across the spec, not per-AC. `spec.close` reads this field at close time and refuses closure for any matching memory whose key is absent. If the tabletop session found no matching memories, omit the field; it is `skip_serializing_if = "Vec::is_empty"` so absent specs stay byte-identical on disk.

### 5. Save the Spec

Save to: `.orbit/specs/YYYY-MM-DD-<topic-slug>/spec.yaml`

If the interview file exists in a spec directory, save alongside it.

### 6. Update the Card's Specs Array

The spec references a card (from the interview's or tabletop note's `Card:` line). After saving `spec.yaml`, append its path to the card's `specs` array so the work trail stays complete.

1. Parse the card path from the input artefact (the `**Card:**` line in either `interview.md` or `tabletop-note.md`) or from conversation context
2. If no card is identified, skip this step — not all specs originate from a card
3. Read the card YAML
4. Append the new spec path (e.g. `.orbit/specs/2026-04-12-topic/spec.yaml`) to the `specs` array
5. If the `specs` array doesn't exist yet, create it
6. Write the updated card back to disk

**This is non-negotiable.** Every spec that addresses a card must appear in the card's `specs` array. Agents downstream (`/orb:tabletop`, `/orb:implement`) rely on this array to understand cumulative progress. An incomplete array causes agents to lose the thread and repeat or contradict prior work.

---

**Next step:** Run `/orb:review-spec` to stress-test the plan, then `/orb:implement`.
