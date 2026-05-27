---
name: spec
description: Generate a structured YAML specification with numbered ACs from interview results
argument-hint: "[interview_file]"
allowed-tools: Bash Read Edit Write
---

# /orb:spec

Generate a validated specification from interview results or conversation context.

## Usage

```
/orb:spec [interview_file]
```

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

### 4. Carry the tabletop verification classification

Every AC's `verification` clause must end with the per-scenario classification verbatim from the tabletop sidecar's **Verification posture** section (see `/orb:tabletop`) — either `verifies: capability` or `verifies: stand-in (real thing is X), accepted because Y`. The classification is copied as-is, not paraphrased.

**Halt rule.** When an AC's source scenario has no classification in the tabletop sidecar (or closed-mode `tabletop-note.md`, which carries the same convention), /orb:spec halts and routes to **AskUserQuestion** with three picks:

- **rescope inline** — rewrite the AC so it verifies the capability directly; no classification needed.
- **re-walk tabletop** — short detour back to /orb:tabletop for this one scenario; resume /orb:spec after.
- **accept-with-rationale** — accept the AC as-is and capture rationale as a spec note using the canonical `deferred-scenario:` prefix so the conformance audit can parse it. Format:

  ```
  deferred-scenario: <card-id>:<scenario-name> -- <rationale>
  ```

  `<card-id>` is the full slug (e.g. `0045-scope-discipline`); `<scenario-name>` matches a name from the card's `scenarios[].name`. Persisted via `orbit spec note <spec-id> "deferred-scenario: ..."`. The audit family `card_coverage_gap` (per spec 2026-05-26-scope-discipline-front-loaded) fires when 2+ such deferrals accumulate on a card without a follow-up spec.

### 5. Record memories considered

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

### 5a. Read cited sources and record `cite_evidence`

For each memory in `memories_considered` whose record carries `cites:`, the spec author MUST read each cited source and record `cite_evidence` on that memory's reconciliation entry. The shape is a list of `{ cite_path, excerpt, read_at }`:

```yaml
memories_considered:
  - key: load-bearing-cited-memory
    disposition: adopted
    reason: "evidence drawn from the cited docs, not the body summary"
    cite_evidence:
      - cite_path: docs/the-load-bearing-doc.md
        excerpt: |
          the 1-3 line passage that carries the mechanical detail
          the memory body summarised
        read_at: 2026-05-27T12:34:56Z
```

Four directives, all gated by the `spec.close` pre-flight (per spec 2026-05-27-memory-cite-reading ac-04 — the second-pass cite-evidence gate refuses closure when any cite on a referenced memory lacks evidence):

- One `cite_evidence` entry per cite on the matched memory — each entry MUST carry `cite_path`, `excerpt`, AND `read_at` (RFC3339 timestamp). Empty or absent on memories without `cites:`.
- The `excerpt` MUST be drawn verbatim from the file contents at `cite_path` — not paraphrased from the memory body, not synthesised, not summarised. The whole point of the cite-read step is to defeat memory-compression artefacts; an excerpt that doesn't come from the cited file is evidence that nobody opened the file.
- `read_at` is the RFC3339 timestamp at the moment of read — it pins the evidence in time so a later reviewer can audit which version of the cited file was the source.
- `spec.close` blocks closure when any `cites[].path` on a referenced memory lacks a matching `cite_evidence.cite_path` entry. The refusal message names the memory key, the missing cite paths, and prompts the reader to read the file and record an excerpt.

### 6. Save the Spec

Save to: `.orbit/specs/YYYY-MM-DD-<topic-slug>/spec.yaml`

If the interview file exists in a spec directory, save alongside it.

### 7. Update the Card's Specs Array

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
