---
name: tabletop
description: Front-loaded thinking before specs are written — one session walks values, trade-offs, halt conditions, escalation triggers, and kill conditions across one card or a cluster, producing one or more aligned specs each carrying a pre-flight contract
---

# /orb:tabletop

Tabletop is the canonical pre-spec session for substantive R&D. One session, one card or a cluster in scope, one or more specs out. The output is the spec's **contract** — values, trade-offs, halt conditions, escalation triggers, kill conditions, hot-wash — *never* the spec's solution.

Choice 0017 (`tabletop-output-is-contract`) pins the load-bearing rule: tabletop captures *what to optimise for and what would stop us*; the spec captures the AC contract; the drive captures the implementation. Conflating these is the failure mode this skill exists to prevent.

Agent prose follows the discipline in `.orbit/STYLE.md` (see card 0026 — `.orbit/cards/0026-agent-prose-discipline.yaml`).

@.orbit/STYLE.md

## Usage

```
/orb:tabletop [card-id | card-id-1 card-id-2 ... | "goal string"]
```

## When to use

- **Substantive R&D** where alignment cost would compound across the work
- **Cross-cutting work** touching two or more cards in one cluster
- **Goal-scoped work** where the cluster of cards isn't pre-determined ("ship X" with no card list)
- **Lightweight pre-flight** where the approach is already decided — use closed mode (§ Closed mode below)

## Recall pre-flight

Before classifying the design space or walking the questions, surface
prior substrate that touches this tabletop's card cluster so the
session doesn't re-litigate decisions already encoded in memories,
cards, choices, prior specs, or memos. This is a **structural step**
at skill entry, not advice. Per spec
2026-05-25-recall-verb-and-skill-step ac-05 and card 0044
(substrate-recall) — the pull-mode counterpart to the substrate-push
hook surface. Mirrors the structural-investigation pattern at Q8.

**Scope derivation.** Iterate the cluster cards (the `Cards in scope:`
set named at session open). For each card, run a recall on the card's
slug; if the card's `references[]` field carries non-pointer entries
that hint at additional topic strings (subsystem names, decisions, prior
session keys), recall on those too.

**Invocation.**

```bash
orbit recall "<card-slug-or-topic>" --json | jq -r '.data.result.matches[] | "\(.score) \(.type) \(.id)\t\(.path)\n  \(.snippet)"' | head -20
```

Quote a 3-5 line summary of the top hits per card inline. The `path`
field on each match resolves to a file you can read in full when a
snippet earns a deeper look. Zero matches is a valid outcome — log
"recall: no prior substrate on `<topic>`" and proceed.

## Trivial-skip advisory

When the work appears trivial — single-line change, typo fix, single-AC scope — the skill surfaces a prose nudge at session open:

> *"This looks trivial; a direct `/orb:spec` is recommended. Tabletop is reserved for substantive R&D where alignment cost would otherwise compound. Proceed anyway? (y/N)"*

The advisory is **prose, not AskUserQuestion** — the operator chooses by typing or by continuing. The skill does NOT refuse. Per card 0019 scenario 10.

## Input modes

### Mode A — Card-list input

`/orb:tabletop 0019` or `/orb:tabletop 0019 0026 0017` opens a tabletop scoped to those cards. The cluster is locked at invocation; alignment work begins after design-space classification (below).

### Mode B — Goal-string input

`/orb:tabletop "ship orbit-state v0.1"` opens a tabletop with no card list. The skill walks the card index (`orbit card list`), infers which cards the goal touches (best-effort — substring match on `feature` / `goal` fields, semantic match, or operator-keyword scan are all acceptable), and presents the inferred cluster in prose with one-line rationales per card:

> *"Inferred cluster from goal `ship orbit-state v0.1`:*
> *- 0020-orbit-state — the substrate the goal directly names*
> *- 0021-tasks — orbit-state's task model; v0.1 ships parity*
> *- 0008-orbit-folder — the directory layout the substrate writes into*
> *- 0017-setup-bead-aware — the bootstrap that creates the layout*
> *Approve, modify, or extend?"*

Then **AskUserQuestion** fires with `approve`, `modify`, or `extend`. The AUQ confirmation is the load-bearing safety valve — the inference need not be deterministic because the author edits before alignment work begins. Per card 0019 scenario 4.

**Ordering**: goal-string AUQ cluster confirmation runs FIRST; design-space classification (below) runs only after the cluster is locked.

## Design-space classification

Once the cluster is locked, classify the design space before walking the 10 questions:

| Mode | Signals | Path |
|------|---------|------|
| **closed** | An associated choice file under `.orbit/choices/` already pins the architectural approach (status `accepted`), AND at least one of the cluster's cards has a shipped spec following that pattern. | `Closed mode` below — produce a `tabletop-note.md`, no 10-question methodology |
| **open** | No associated choice file. Decisions unresolved. Multiple plausible shapes for the cluster. | `Open / partial mode` below — walk the 10 questions, produce a `tabletop.md` sidecar per output spec |
| **partial** | A choice exists but residual trade-offs remain — the choice is `proposed`, or only some cards in the cluster are covered by prior specs. | `Open / partial mode` below — walk the 10 questions, scoped to the residual trade-offs |

State the classification in one line before proceeding:

> *"Cluster {0019, 0026}: partial. Choice 0017 pins the contract-not-solution rule but the skill surface is undecided. Walking the 10-question methodology scoped to residual trade-offs."*

## Closed mode — produce a tabletop-note

When the design space is closed, skip the 10-question methodology entirely and produce a one-screen `tabletop-note.md` per output spec at `.orbit/specs/<spec-id>/tabletop-note.md`:

```markdown
# Tabletop Note: <Topic>

**Date:** YYYY-MM-DD
**Cards:** .orbit/cards/NNNN-slug.yaml [+ siblings]
**Mode:** closed
**Choice:** .orbit/choices/NNNN-slug.yaml — <one-line title>

---

## What good looks like

<User-voice paragraph — drafted by the agent from the card(s)' goal and scenarios, offered to the author for editing. One paragraph, three to six sentences.>

## Pinned approach

<Cite the choice and any prior specs that establish the pattern. One to three bullets — what's already decided and why this cluster is operationally inside that decision.>

## Deferred items

- <Anything the cluster raises that this spec does not address>
- <Open question that belongs to a future spec, not this one>

## Implementation notes

- <Means-level leads from codebase scan — starting context for the implementing agent>
- <Recommended `ac_type` per AC the spec will carry (`code` / `config` / `doc` / `ops` / `observation`)>
```

Closed-mode tabletop preserves the contract-not-solution rule from choice 0017 — the pinned approach is what was decided; implementation work still lives in the drive. The `tabletop-note.md` and `tabletop.md` are mutually exclusive per session — closed mode does NOT also write the open-mode sidecar.

After the note lands, exit cleanly:

> *"Tabletop note written. Closed design space — no 10-question methodology needed. Run `/orb:spec` to crystallise the spec from this note."*

## Open / partial mode — the 10-question methodology

Walk the 10 questions in declared order. Each carries a **role line** (what the question surfaces) and a **stop condition** (when the question is done). The 2026-05-07 dogfood at `.orbit/archive/specs/2026-05-07-orbit-state-v0.1/tabletop.md` is the worked example — 241 lines covering the full 10 plus hot-wash.

The AUQ-prose hybrid (§ AUQ-prose hybrid below) decides which questions use prose and which use AskUserQuestion.

### Q1 — Goal narrowing

**Role:** lock the goal in one sentence the author would say to a colleague.
**Stop:** the author accepts a narrowed goal or confirms the card-level goal is the right scope.
**Mode:** prose.

The card's `goal` is the starting point. Surface it; ask whether it should narrow or extend for this session's cluster.

### Q2 — Values

**Role:** name what the work is optimising for. Often there are several; one is usually load-bearing.
**Stop:** the author picks the load-bearing value AND explains why the others fall out of it (downstream consequences, not parallel goals).
**Mode:** prose.

Surface candidate values from the cluster's cards. Ask the author to rank or name the load-bearing one. Listen for the "X is the SUBSTRATE value; the others land on top" reframe — that's a Q2 closing signal.

### Q3 — Trade-offs (the simplest-way contract cut)

**Role:** enumerate what the work is trading against the chosen values. Each trade-off is acceptable, expensive-but-worth-it, or a halt-trigger.
**Stop:** every named trade-off carries a classification AND the author confirms the cut is the simplest cut that holds the values.
**Mode:** prose.

The "simplest way" framing matters — over-engineering shows up here. The Q3-extension pattern from the dogfood (verb-surface re-cuts, scope deflation) is normal — multiple iterations until the cut is genuinely simplest.

### Q4 — Failure modes (what could go wrong)

**Role:** enumerate the ways the work fails. Each failure mode is `halt-worthy` (auto-stop) or `engineering hygiene` (pin as AC).
**Stop:** every failure mode carries a classification AND the halt-worthy ones have a measurable trigger named.
**Mode:** prose.

Prefer **real scenarios over imagined** — see § Real scenarios below.

### Q5 — Lateral approaches

**Role:** name the alternatives that *aren't* being picked, with reasons. Lateral options held in reserve become fallback paths.
**Stop:** at least three laterals are named (alternatives, contained-scope versions, defer-entirely) AND each carries a one-line "rejected because" or "held in reserve because".
**Mode:** prose.

Choice 0017 binds: laterals are *named*, not picked. The decision goes to the spec or to a `.orbit/choices/` file. Tabletop surfaces the option set; it does not select.

### Q6 — Success criteria

**Role:** pin binary, measurable criteria that trace to a value or trade-off.
**Stop:** at least four criteria, each binary, each linked back to a Q2 value or Q3 trade-off.
**Mode:** AskUserQuestion at the closing pick. Use AUQ to surface candidate criteria and let the author confirm or modify.

### Q7 — Escalation triggers

**Role:** name when the agent should halt and surface to the author mid-flight, with the proposed action.
**Stop:** each trigger names **condition + state snapshot to surface + proposed action**. "Ask Hugh if confused" / "I'm stuck" are non-actionable and rejected.
**Mode:** AskUserQuestion to confirm the trigger set.

### Q8 — Adjacent code

**Role:** enumerate which layers of the codebase the work touches.
**Stop:** every touched module/file is named AND the migration shape is explicit (rewrite vs path-only-update vs no-op).
**Mode:** prose.

This question is intent-shaped at the layer level (which layers are in scope) but implementation-shaped at the file level. Surface the layer set; route file-level questions to **Implementation Notes** in the sidecar.

**Orchestrate `/orb:code-investigate` (broad mode) at Q8 entry, BEFORE walking the layer enumeration.** Per choice 0029 (pipeline-orchestrates-investigation), tabletop's Q8 is a pipeline-stage moment where investigation must fire structurally, not as advice. The orchestrated invocation feeds Q8's layer enumeration with empirical grounding rather than agent inference.

**Scope is agent-typed from the cluster cards' references[].** Iterate the cluster's cards (the `Cards in scope:` set locked at Q1), read each card's `references[]` field, and collect entries that resolve to file paths in the repo (drop URLs, freeform descriptions, and pointer-only references). That set is the broad-mode scope. Earlier Q1-Q5 do not enumerate code areas — Q8 is the question that does — so derivation cannot depend on Q8's own output; cards' references[] is the only source.

**Write scope to memory BEFORE the Skill call** (args-drop guard per memory `slash-command-args-vs-skill-tool-args` — Skill tool args can drop on forked invocations, leaving the called skill with empty scope). Tabletop is session-bound (no spec exists yet), so the scope lands as a labelled memory:

```bash
orbit memory remember tabletop-investigation-scope-<date>-<slug> "<paths>" --label code-investigate
```

Then invoke `/orb:code-investigate` (broad mode) via the Skill tool with that scope. The called skill writes a marker entry at `.orbit/.code-investigate-recent` and emits prose. **Quote a 5-10 line summary of the return inline** into your working context before walking the Q8 layer prose — marker-write alone is insufficient; re-quoting the prose is what makes the investigation load-bearing for Q8's enumeration.

**Bypass shape.** If the cluster has no cards with file-path references[] (e.g. greenfield work on a non-existent module), call AskUserQuestion with:
- (a) Proceed with broad-mode investigation on the repo root (budget-capped)
- (b) Skip with logged reason

If (b), log via `orbit memory remember tabletop-investigation-bypass-<date>-<slug> "<reason>" --label code-investigate` and proceed to the layer enumeration prose unaided.

### Q9 — Budget

**Role:** name the working-day budget at Claude-execution pace, not at conservative-engineering quotes.
**Stop:** the author confirms the budget AND names the Theme-5a halt trigger condition (the "if real burn rate trends toward the inflated estimate inside week 1, halt and reassess scope/architecture").
**Mode:** AskUserQuestion for the budget option pick. **Apply the inflation-guard recut** per the 2026-05-07 dogfood — agent-authored effort estimates default to conservative-engineering quotes and are divided by ~3 before treating as planning input. Surface BOTH the original estimate and the recut.

### Q10 — Kill conditions

**Role:** name the failure of each load-bearing claim, with a named pivot path.
**Stop:** each load-bearing claim from Q2 (values), Q3 (trade-offs), Q4 (failure modes) has a kill condition; each kill condition names the specific claim being killed AND a pivot path.
**Mode:** AskUserQuestion to confirm the kill set.

### Hot-wash debrief

**Role:** capture meta-observations — what kept coming up, what was unclear, what reframes surfaced.
**Stop:** the session is ready to be written up.
**Mode:** prose.

This is the closing prose section — fresh, before formal write-up sanitises the signal. Two to five bullets per category: `recurred`, `surprised`, `friction`, `meta-patterns-for-future-tabletops`.

## AUQ-prose hybrid

The hybrid pattern from the 2026-05-07 dogfood (memory `tabletop-auq-hybrid-pattern`): **prose opens forks; AUQ closes picks**.

| Question | Mode | Why |
|----------|------|-----|
| Q1 goal narrowing | prose | author may reframe the goal entirely |
| Q2 values | prose | the load-bearing-value reframe is the value of the question |
| Q3 trade-offs | prose | multiple iterations to find the simplest cut |
| Q4 failure modes | prose | enumeration with classification — too open for AUQ |
| Q5 laterals | prose | naming options without picking — AUQ would imply picking |
| Q6 success criteria | AUQ at close | finalise the criterion set with author confirmation |
| Q7 escalation triggers | AUQ at close | confirm the trigger set |
| Q8 adjacent code | prose | layer-level enumeration is reframable |
| Q9 budget | AUQ at close | budget-option pick, with inflation-guard recut surfaced |
| Q10 kill conditions | AUQ at close | confirm the kill set |
| Trivial-skip advisory | prose | y/N continuation; not a discrete option set |
| Goal-string cluster confirmation (Mode B) | AUQ at close | approve / modify / extend the inferred cluster |

### AUQ-refusal fallback

If the author rejects all offered AUQ options with a **custom response that reframes the question itself** (rather than picking one of the offered options), the agent treats the response as a return-to-prose signal and re-walks that question's prose phase. The reframe is the value the author is offering; the AUQ frame is what's failing.

## Real scenarios over imagined

For Q4 (failure modes) and Q5 (laterals), walk past run-logs **before** inventing scenarios:

1. Each card in the cluster has a `specs[]` array — read each `progress.md` and `spec.yaml`.
2. `.orbit/archive/specs/` carries shipped specs from prior sessions.
3. Prior `tabletop.md` sidecars carry recorded Q4/Q5 enumerations for cards in this cluster's neighbourhood.

Imagined scenarios that survive (no past run-log evidence) are explicitly flagged in the sidecar with an `[imagined]` marker:

```markdown
## Q4 — Failure modes

- Format-integrity bug locks every agent  [hygiene; ref 2026-05-07 K1]
- Concurrency race on cluster locks  [imagined; halt-worthy]
- Migration locks the orbit repo  [halt; ref 2026-05-07 H-4]
```

Per card 0019 scenario 7.

## Halt / escalation / kill — actionable shape

The skill enforces actionable phrasing on Q4 halts, Q7 escalations, and Q10 kill conditions:

### Halt conditions (Q4)

**Required:** measurable trigger AND revert path.
**Rejected anti-patterns:**
- *"Things go wrong"* — no trigger
- *"Halt and reassess"* — no revert path
- *"If we get stuck"* — no trigger, no revert

**Canonical shape:**
> *"Drop below X within N hours with revert path Y"* — e.g. *"Test suite drops below 90% green within 2h of any cascade step → `git restore .` and re-attempt as a whole."*

### Escalation triggers (Q7)

**Required:** condition + state snapshot to surface to author + proposed action.
**Rejected anti-patterns:**
- *"Ask Hugh if confused"* — no condition, no snapshot, no action
- *"I'm stuck"* — same
- *"Escalate on failure"* — no specificity

**Canonical shape:**
> *"`cargo test -p orbit-state` fails on more than 5 tests in a single cascade step. Surface: failing test names + first-failure log lines + diff of cascade step. Action: AUQ author with two picks — (a) fix forward inline, (b) `git restore .` and re-attempt the cascade as a whole."*

### Kill conditions (Q10)

**Required:** the specific load-bearing claim being killed AND a named pivot path.
**Rejected anti-patterns:**
- *"If this fails"* — no claim
- *"Kill the project"* — no pivot

**Canonical shape:**
> *"K1: format-integrity claim. If `orbit verify` returns drift on a freshly-canonicalised tree, format-integrity is dead. Pivot: revert substrate work; ship as files-only without canonicalisation."*

Per card 0019 scenarios 8 and 9.

## Output

### Open / partial mode — `tabletop.md` sidecar per output spec

Every open-mode or partial-mode tabletop session writes one `tabletop.md` sidecar per output spec at `.orbit/specs/<spec-id>/tabletop.md`. Shape mirrors the 2026-05-07 dogfood:

```markdown
# Tabletop — <Topic>

**Date:** YYYY-MM-DD
**Facilitator + domain expert:** <author name>
**Scribe + driver:** <agent name>
**Cards in scope:** <cluster — full list>
**Methodology:** Card 0019 — 10-question methodology; choice 0017 — output is contract, not solution
**Output spec:** .orbit/specs/<spec-id>/spec.yaml

[When N > 1 specs from one session: link to sibling sidecars]
**Sibling sidecars:** .orbit/specs/<spec-id-2>/tabletop.md, .orbit/specs/<spec-id-3>/tabletop.md

---

## Values
<from Q2 — load-bearing value + reframe>

## Trade-offs
<from Q3 — each classified acceptable / expensive-but-worth-it / halt-trigger>

## Halt conditions
<from Q4 halt-worthy entries — measurable trigger + revert path each>

## Escalation triggers
<from Q7 — condition + state snapshot + proposed action each>

## Kill conditions
<from Q10 — load-bearing claim + pivot path each>

## Hot-wash
<from closing — recurred / surprised / friction / meta-patterns>
```

The six sections are in declared order. Imagined scenarios in any section carry an `[imagined]` marker.

### Closed mode — `tabletop-note.md`

See § Closed mode above. The note replaces the sidecar entirely; the two are mutually exclusive per session.

### One-to-many fan-out

One tabletop session produces N specs (N ≥ 1). Each output spec is its own folder `.orbit/specs/<date>-<slug>/` carrying `spec.yaml` and `tabletop.md`. Each spec's `cards:` array carries one or more entries — single-card specs are legal; cross-cutting specs list multiple.

When N > 1, each sidecar's header names the cluster of cards in scope and links to sibling sidecars (`**Sibling sidecars:** .orbit/specs/<id-2>/tabletop.md, ...`). The author can navigate the fan-out without losing visibility.

After the session writes the N sidecars, the agent surfaces a one-screen summary in prose:

> *"Tabletop produced 3 specs:*
> *- 2026-05-21-foo (cards 0019, 0026) — sidecar at .orbit/specs/2026-05-21-foo/tabletop.md*
> *- 2026-05-21-bar (card 0017) — sidecar at .orbit/specs/2026-05-21-bar/tabletop.md*
> *- 2026-05-21-baz (cards 0019, 0017) — sidecar at .orbit/specs/2026-05-21-baz/tabletop.md*
> *Next: `/orb:spec` against each to crystallise the AC contract."*

Per card 0019 scenarios 1, 3, and 6.

---

**Next step:** `/orb:spec` against each output spec folder to crystallise the AC contract from the tabletop's values, trade-offs, halt conditions, escalation triggers, kill conditions, and acceptance criteria. The `/orb:spec` skill accepts either `interview.md` (legacy `/orb:design` artefact) or `tabletop.md` / `tabletop-note.md` as the design handoff.
