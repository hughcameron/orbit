# Decision Pack — Rally Approval is a Structured Prompt

**Card:** `orbit/cards/0006-rally.yaml` (scenario: *"Rally approval is a structured prompt"*)
**Scope:** The §2b proposal-approval gate in `plugins/orb/skills/rally/SKILL.md`. Tighten the already-shipped pattern; do not redesign the proposal flow, the thin-card guard, or the subsequent decision gate in §5.

## Context

The rally skill (shipped in 0.2.19) already uses `AskUserQuestion` at §2b with three options phrased **Approve as-is / Modify the list / Reject the rally**. The new card-level scenario formalises the option set as **approve-all / select-subset / decline**. The two phrasings do not match, and the skill gives no concrete guidance on:

1. Which option *label set* is canonical.
2. How a "modify" / "select-subset" response is *collected* — AskUserQuestion is multiple-choice; the names of the cards to add or remove are free-form.
3. Where per-card **rationale** lives — inline in option descriptions (verbose, hits the option-description length cap) or in a separate preview block (cleaner but splits the surface).
4. Whether the thin-card guard re-runs on every modify loop or only at the final approval.

A sibling card (`0005-drive.yaml`, scenario at lines 70–74) uses AskUserQuestion for review verdicts with the options `approve / request-changes / block / read-full-review`. Rally's approval surface should be consistent in *shape* with drive's verdict surface (fixed, short, action-verb options) while owning its own labels.

---

## Decision 1 — Canonical option labels

### Context

The card text names **approve-all / select-subset / decline**. The skill §2b names **Approve as-is / Modify the list / Reject the rally**. A reader holding one artefact gets a different mental model than a reader holding the other. One has to win; the other has to be rewritten.

### Options

- **A. Adopt the card's triple: `approve-all` / `select-subset` / `decline`.** Rewrite SKILL.md §2b labels. Card is authoritative; labels are terse, consistent with drive's verb-style verdicts (`approve` / `request-changes` / `block`), and "select-subset" is more honest than "modify" because the most common edit is pruning the agent's overreach.

- **B. Adopt the skill's triple: `approve-as-is` / `modify-list` / `decline`.** Rewrite the card scenario to match. Skill is the shipped product; "modify" admits both additions and removals, which matches §2b's existing loop behaviour ("author names cards to add or remove").

- **C. Hybrid: `approve-all` / `modify-list` / `decline`.** Keep "modify" (it covers both add and remove) but drop "as-is" in favour of "all" for symmetry with other orbit prompts (distill uses approve/edit/reject — also a triple with a verb and an explicit alternative action).

### Trade-offs

```
| Option | Gains                                                       | Loses                                                  |
|--------|-------------------------------------------------------------|--------------------------------------------------------|
| A      | Card wins; terse, matches drive verdict shape; select-subset | "select-subset" reads as remove-only; author may still |
|        | is the honest common case                                    | want to ADD a card — label under-describes the action  |
| B      | Skill wins; "modify" covers add+remove honestly              | Card rewrite needed; "as-is" is a hedge phrase         |
| C      | "all" + "modify" is accurate for both directions; matches    | Diverges from card text as literally written (card     |
|        | distill and drive label patterns                             | says "select-subset")                                  |
```

### Recommendation

**Option C (`approve-all` / `modify-list` / `decline`).** Evidence:

- §2b already says *"author names cards to add or remove"* — the add path is a real, documented branch. "select-subset" in the card elides the add path.
- Drive's card (0005) uses hyphenated action labels (`request-changes`, `read-full-review`). `modify-list` fits that shape; `select-subset` does not.
- The card text is a maturity:planned scenario describing *intent* — updating the card scenario's then-clause from `select-subset` → `modify-list` is cheaper than under-describing the interaction surface on every rally run.

**Secondary action:** update `orbit/cards/0006-rally.yaml` scenario line 60 to match (`approve-all / modify-list / decline`). This keeps card and skill in lock-step.

---

## Decision 2 — How `modify-list` collects the edit

### Context

`AskUserQuestion` returns exactly one of the presented options (or an "Other" free-text response, depending on the tool's shape). "Modify" is inherently open-ended: the author needs to name cards by path or number. The skill currently says *"author names cards to add or remove (free-form response)"* but does not specify the mechanism. Three mechanisms are available.

### Options

- **A. Single `AskUserQuestion` with "Other" free-text as the modify channel.** The author types the modification verbatim in the AskUserQuestion "Other" field. The lead parses "remove 2, add orbit/cards/0019-foo.yaml" or similar.

- **B. Two prompts: AskUserQuestion for the verdict, then a second AskUserQuestion (free-form) collecting the modification when the verdict is `modify-list`.** Structured verdict, free-form follow-up.

- **C. Conversational fallback: on `modify-list`, drop out of AskUserQuestion and accept the author's next normal message as the modification.** No second prompt — the lead just waits for the next turn.

### Trade-offs

```
| Option | Gains                                              | Loses                                                    |
|--------|----------------------------------------------------|----------------------------------------------------------|
| A      | One gate, one turn — lowest friction if the        | AskUserQuestion "Other" is typically unlabelled / hidden |
|        | author already knows the edit                      | below options; authors miss it. Parsing is LLM-fuzzy.    |
| B      | Explicit, labelled second prompt; author always    | Two gates = two human touchpoints, which cuts against    |
|        | sees a clear "what do you want to change" field    | the rally's "fewer touchpoints" value                    |
| C      | Matches how drive's review surfaces "read the      | Mixing structured and unstructured turns is hard to      |
|        | full review" — conversational fall-through         | resume; session-context hook loses the gate state        |
```

### Recommendation

**Option B (two prompts).** Evidence:

- The card-level goal ("fewer human touchpoints per card") is about *per-card* touchpoints, not per-decision. A proposal gate with a bounded second step is still one rally-level gate.
- AskUserQuestion "Other" (Option A) is documented as gather/clarify — not as the primary input channel. Relying on it for the modify content trades machine-parseable verdicts for LLM-parseable free text on the verdict turn.
- Distill's spec review (`orbit/specs/2026-04-04-distill/review-spec-2026-04-04.md` lines 42–46) raised the same pattern explicitly: approve/edit/reject using AskUserQuestion for the verdict, then a follow-up turn for the edit payload. The pattern is already precedent in orbit.
- Keeps the **§2b loop** trivially re-enterable: verdict → modify instructions → re-present revised list → verdict → ... Each loop iteration is exactly two prompts.

**Concrete second-prompt shape (non-binding but illustrative):** a single AskUserQuestion with no pre-populated options and a prompt text of *"Name cards to add (by path) or remove (by number). Empty response cancels the modification."* The response is interpreted as modification instructions — same interpretation pattern distill uses.

---

## Decision 3 — Per-card rationale placement

### Context

The proposal today (SKILL.md §2b) shows per-card rationale in a **markdown block above** the AskUserQuestion call:

```
## Rally Proposal — <goal string>
Candidate cards:
  1. orbit/cards/<id>-<slug>.yaml — <feature line>
     Rationale: <why this card fits the goal>
  ...
```

Then the three options are presented. AskUserQuestion also accepts a per-option `description` — rationale could live there too. Three placements are possible.

### Options

- **A. Rationale in the preview block only (status quo).** AskUserQuestion shows short, verb-shape options (`approve-all` / `modify-list` / `decline`) with one-line descriptions about the *action*, not the cards.

- **B. Rationale in AskUserQuestion option descriptions only.** The preview block goes away; each option's description summarises which cards and why. Fits well for short rallies (2–3 cards); overflows for 6 cards.

- **C. Both: preview block for per-card rationale + AskUserQuestion with terse option descriptions about actions.** Current approach, kept explicit.

### Trade-offs

```
| Option | Gains                                                       | Loses                                              |
|--------|-------------------------------------------------------------|----------------------------------------------------|
| A      | Clean separation: markdown handles cards, AskUserQuestion   | Two surfaces; slightly more scroll for the author  |
|        | handles action. Works for any N cards.                      |                                                    |
| B      | One surface — author sees everything in the question card   | AskUserQuestion option descriptions have practical |
|        |                                                             | length limits; 6-card rallies break the format     |
| C      | Explicit contract — preview block is the single source of   | Slightly redundant if rationale is short           |
|        | card rationale; options stay pure about action              |                                                    |
```

### Recommendation

**Option C (both, with strict roles).** Evidence:

- Rally's design principle is *"maximum clarity based on the best available evidence"* at each gate. The preview block is where the rationale earns its weight; the AskUserQuestion is the *decision surface*. Collapsing them (B) subordinates evidence to UI, which inverts the value.
- A 6-card rally is plausible (card 0006 scenarios 7 and 9 name parked-card + disjointness paths that assume rallies up to ~5–6 cards). Option B breaks visibly at N=4+.
- The "strict roles" clause is the refinement: preview block = card rationale; AskUserQuestion option `description` field = one-line action summary (e.g. *"Proceed with all N candidates"*, *"Add or remove cards before proceeding"*, *"Abort the rally; offer individual drive as alternative"*). No rationale in option descriptions.

---

## Decision 4 — Thin-card guard re-run on modify loop

### Context

The thin-card guard (§2a) refuses any candidate with <3 scenarios *before the proposal is shown*. When the author picks `modify-list`, the lead:

1. Accepts the modification (add/remove cards).
2. Re-presents the revised candidate list.
3. Awaits another AskUserQuestion verdict.

Question: does the thin-card guard fire **before every re-prompt** or **only at the final approval**? The skill §2b says *"If the author adds a card not in the scan's top-N, include it — then re-run the thin-card guard against the new candidate"* — but is ambiguous about whether the guard is per-addition or per-revised-list, and whether it runs on removals (trivially no, but worth naming).

### Options

- **A. Re-run the guard on every revised candidate list (before every re-prompt).** Every modify loop iteration goes: modify-instructions → apply edits → thin-card guard → re-present with AskUserQuestion. A thin card introduced by the author blocks the loop until removed or thickened.

- **B. Run the guard only on the final approval.** The modify loop proceeds freely; the guard fires once when the author picks `approve-all`. If any card in the final list is thin, the rally refuses and the author is kicked back to modify.

- **C. Run the guard only on *additions* during the modify step.** Removals skip the guard (no new thin candidates possible). Additions pass through the guard before the revised list is re-presented.

### Trade-offs

```
| Option | Gains                                            | Loses                                                    |
|--------|--------------------------------------------------|----------------------------------------------------------|
| A      | Simple invariant: "every candidate list shown to | Re-runs the guard on removals, which is wasted work      |
|        | the author has passed the guard"                 | (but cheap)                                              |
| B      | Minimal guard invocations                        | A thin card can linger through multiple modify loops;    |
|        |                                                  | the author discovers the problem only at final gate,     |
|        |                                                  | which wastes their decision on a list that can't fly     |
| C      | Skips the guard on trivial removes; matches      | Two code paths (add-path / remove-path); harder to       |
|        | skill §2b's current wording                      | reason about when the author does both in one modify    |
```

### Recommendation

**Option A (guard fires before every re-prompt).** Evidence:

- Constraint #1 from `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` (rewritten as ac-01 in v1.3): *"Rally refuses at proposal any rally containing a card with fewer than 3 scenarios, regardless of eventual serial-or-parallel outcome."* The word *any* and the "unconditional" framing support the broader invariant in Option A.
- Option B breaks rally's principle *"maximum clarity based on the best available evidence"*: the author picks `approve-all` and is then told the list is invalid. The gate should present a list that is already pre-qualified.
- Option C's savings are negligible (the guard is a cheap scenario-count scan) and the two-path complexity outweighs the saving.
- The guard is a *pre-qualification gate*, not a *decision*. Rally already treats re-briefs on sub-agent path violations as "pre-qualification retries that don't count against the strike budget" (ac-04, v1.3) — the same framing applies here: re-running the guard is not a strike and does not count against any loop count.

**Bounded-loop note (non-binding):** if the loop is worth capping (author and agent ping-pong on thin-card additions), a soft cap of ~5 modify iterations before the lead offers to escalate to `/orb:card` for the offending cards would match the rally skill's other single-strike patterns. This is out of scope for the card-level scenario and belongs in the implementation spec.

---

## Summary of Recommendations

```
| Decision                                  | Recommendation                                                                                    |
|-------------------------------------------|---------------------------------------------------------------------------------------------------|
| 1. Canonical option labels                | `approve-all` / `modify-list` / `decline` (update card scenario to match)                         |
| 2. How modify-list collects the edit      | Two-prompt: verdict AskUserQuestion → free-form AskUserQuestion for the modification              |
| 3. Per-card rationale placement           | Both: preview block for card rationale, AskUserQuestion option descriptions for the action summary|
| 4. Thin-card guard on modify loop         | Re-run before every re-prompt (pre-qualification invariant); not a strike                         |
```

## Evidence Trail

- Card text — `orbit/cards/0006-rally.yaml` lines 57–62 (scenario), line 73 (goal framing).
- Skill current pattern — `plugins/orb/skills/rally/SKILL.md` §2b (lines 94–116), §5 (line 238), §6c (line 276).
- Drive verdict consistency — `orbit/cards/0005-drive.yaml` lines 70–74.
- Distill precedent for structured verdict + free-text follow-up — `orbit/specs/2026-04-04-distill/review-spec-2026-04-04.md` lines 42–46; `orbit/specs/2026-04-04-distill/spec.yaml` lines 15, 40.
- Thin-card guard framing — `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` ac-01 (v1.3), constraint #3.
- Pre-qualification retry framing (same concept applied to path-discipline re-briefs) — `orbit/specs/2026-04-19-rally-subagent-model/spec.yaml` ac-04 (v1.3) "pre-qualification retry, NOT a rally-level strike".
- Rally value statement — `plugins/orb/skills/rally/SKILL.md` Principle block (post-§3): *"maximum clarity based on the best available evidence"*.
