---
name: implement
description: Pre-flight check + work loop for a beads-tracked bead — surface ACs, enforce gates, escalate detours as sub-beads, close on completion
---

# /orb:implement

Drive a single bead from claim to close. This skill is the agent's working
contract for the duration of an in-flight bead: pre-flight context, AC
tracking, gate enforcement, detour escalation, and close-out are all
expressed as concrete `bd` and `parse-acceptance.sh` commands.

## Why This Exists

Without a pre-flight check, implementing agents treat the codebase as
ground truth and miss spec-prescribed patterns. This skill forces the
bead's acceptance criteria into working memory before implementation
begins, and keeps the bead acceptance field as the single source of truth
throughout.

> **Reference incident:** an agent treated the codebase as ground truth
> and missed a spec-prescribed entrypoint pattern.

## Migration Note

The previous `/orb:implement` mirrored ACs into a `progress.md` file,
emitted Claude Code tasks via `TaskCreate`, computed a `Spec hash` for
drift detection, and ran a cancel-then-recreate "resume reconcile" on
session resume. All of that is now subsumed by beads:

- AC list and status — bead `acceptance_criteria` field
- Live AC visibility — `bd show <id>` and `bd ready`
- Drift detection — removed; the acceptance field IS the contract
- Resume — `bd show <id>` is the canonical refresh
- Detours — sub-beads via `bd create --parent ... --deps "discovered-from:..."`

The mechanisms motivated by cards 0003 (implement-session-visibility)
and 0009 (mission-resilience) are now provided by beads itself; those
cards are referenced as historical pointers, not as live contracts in
this skill.

## Usage

```
/orb:implement [bead-id]
```

If `bead-id` is omitted, the skill resolves the active bead automatically
(see "Input contract" below).

## Input contract

The skill operates on exactly one in-progress bead per session. Resolution
proceeds in three branches:

1. **Argument provided** — `/orb:implement <bead-id>`. Use it directly.
   The skill calls `bd show <bead-id>` to validate; if the bead is not
   in `in_progress` status, the skill asks the agent to claim it first
   via `bd update <bead-id> --claim`.

2. **No argument** — query for in-progress beads:

   ```bash
   bd list --status in_progress --json
   ```

   - **Single match** → use it.
   - **Zero matches** → halt and instruct the agent: "No bead in progress.
     Claim one with `bd ready --type task` then `bd update <id> --claim`."
   - **Multiple matches** → halt and instruct the agent to pass the bead
     ID explicitly, listing the candidates.

This skill does not attempt to manage concurrent in-progress beads —
multi-claim is out of scope. If the agent is juggling multiple beads, it
must invoke this skill once per bead with the explicit ID.

## Pre-flight

Before any code is written, the agent runs the following sequence:

1. **Read the bead.**

   ```bash
   bd show <bead-id>
   ```

   The agent reads the description verbatim (this carries the goal and
   any constraints written into the bead). Constraints are prose — the
   agent applies them; there is no parser.

2. **Enumerate ACs.**

   ```bash
   plugins/orb/scripts/parse-acceptance.sh acs <bead-id>
   ```

   This emits one tab-separated tuple per AC:
   `<ac-id>\t<status>\t<description>\t<is_gate>`. The agent surfaces this
   list in its response so the human-visible context contains the full
   AC roster.

3. **Identify the next AC.**

   ```bash
   plugins/orb/scripts/parse-acceptance.sh next-ac <bead-id>
   ```

   This emits `<ac-id>\t<is_gate>` — the first unchecked AC that is not
   blocked by an unchecked gate. If output is empty, all ACs are checked
   and the agent jumps to "Completion" below.

4. **Run a keyword scan.** Search the project source for keywords drawn
   from the bead title and AC descriptions before writing code (see
   `/orb:keyword-scan`). This surfaces existing patterns the work should
   build on rather than reinvent.

No checklist file is written. The bead acceptance field is authoritative
and remains so for the rest of the session.

## Implement loop

For each AC, in `next-ac` order:

1. **Confirm the next AC** with `parse-acceptance.sh next-ac <bead-id>`.
   This is the authoritative gate — if a gate AC is unchecked, `next-ac`
   will return that gate; the agent must complete it first.

2. **Implement the AC.** Write code, edit deliverables, run tests as
   needed.

3. **Mark the AC done.**

   ```bash
   plugins/orb/scripts/parse-acceptance.sh check <bead-id> <ac-id>
   ```

   This calls `bd update --acceptance` internally, flipping the marker
   from `[ ]` to `[x]`. The bead's acceptance field is the only place AC
   status lives.

4. **Loop** — re-run `next-ac` and continue.

### Gate enforcement

Gate enforcement is delegated entirely to `parse-acceptance.sh next-ac`.
By convention (see `.orbit/conventions/acceptance-field.md`), an unchecked
`[gate]` AC blocks all subsequent ACs by declaration order. The parser
implements this — the agent does not re-check gates inline.

If the agent suspects a gate is blocking, it can confirm with:

```bash
plugins/orb/scripts/parse-acceptance.sh blocking-gate <bead-id>
```

This emits the first unchecked gate's `<ac-id>\t<description>` (or
nothing if no gate is blocking).

### Working rules during implementation

- **Bead over codebase.** If the bead's description or an AC prescribes
  a pattern the codebase doesn't have, implement what the bead says. Do
  not work around missing structure — create what the bead requires.
- **Surface unspecced decisions.** When you encounter a choice not
  covered by the bead with meaningful consequences, **stop and ask** via
  `AskUserQuestion`. Present 2–3 options with trade-offs.
- **Constraints in the bead description are non-negotiable.** If you
  find yourself about to violate one, stop and flag it. Either the
  constraint needs updating (re-scope the bead) or the approach needs
  changing.
- **Assumption reversals require escalation.** When implementation
  evidence (tests, benchmarks, real data) contradicts a stated
  assumption, stop. Document the finding (use `bd remember` for
  durable persistence), name the invalidated assumption, and checkpoint
  before proceeding.
- **Derive from evidence.** When prior research, phase results, or
  benchmarks answer a parameter question, use that data. Only escalate
  when evidence is genuinely silent or contradictory.

## Detour escalation

When unplanned work blocks the current AC — a missing dependency, a bug
in a foreign module, a malformed dataset — the agent escalates the
detour as a **sub-bead** with `discovered-from` provenance:

```bash
bd create "<short title for detour>" \
  -t task -p 1 \
  --parent <current-bead-id> \
  --deps "discovered-from:<current-bead-id>" \
  -d "<context: what was discovered, what blocks the parent AC>"
```

Then claim and work the sub-bead:

```bash
bd update <new-sub-bead-id> --claim
# ... resolve the detour ...
bd close <new-sub-bead-id> --reason "<one-line outcome>"
```

Resume the parent:

```bash
bd show <current-bead-id>
plugins/orb/scripts/parse-acceptance.sh next-ac <current-bead-id>
```

`bd show <parent>` reloads the parent's context; `next-ac` returns the
AC the agent was working on. No state lives in any file — the bead graph
is the audit trail.

## Forward findings — three channels

During implementation the agent will discover work that does not belong
in the current bead. The skill routes findings into one of three
channels:

| Finding kind                                  | Channel                                                                    |
|-----------------------------------------------|----------------------------------------------------------------------------|
| Blocking detour (must resolve before this AC) | sub-bead via `bd create --parent <current> --deps "discovered-from:<current>"` (above) |
| Follow-up work that does NOT block this AC    | top-level bead via `bd create -t task` (no `--parent`)                     |
| Product-direction question (capability-level) | memo at `.orbit/cards/memos/YYYY-MM-DD-<slug>.md` for `/orb:distill` later |

**Never suggest "open a follow-up card."** Cards describe capabilities,
not work items. The agent doesn't know whether a finding warrants a new
capability or feeds an existing card's next spec — that's the author's
call during distill. The memo channel preserves the finding without
forcing a structural decision.

## Test execution

Long-running test invocations — expected duration over **60 seconds**, or
full-suite runs (e.g. `cargo test` without a filter, `pytest` at the
repo root, `npm test`) — **MUST** be launched via the `Monitor` tool
with the command piped through a line-buffered failure-marker filter.
The canonical filter is:

```
grep --line-buffered -E 'FAIL|ERROR|AssertionError|Traceback'
```

Short targeted tests (under 60 seconds, a named subset or single test)
continue to use the `Bash` tool as before. **Unfiltered Monitor on a
test suite is forbidden** — every stdout line becoming a notification
swamps the agent. The `grep --line-buffered` wrapper ensures only
failure markers surface as streamed events while the suite runs to
completion.

### First-failure checkpoint

On the **first streamed** line from `Monitor` that matches the
failure-marker regex `FAIL|ERROR|AssertionError|Traceback`, the agent's
behaviour branches by interactivity.

**Interactive path** — stdin is a TTY AND `ORBIT_NONINTERACTIVE` is unset
or not equal to `1`:

The agent MUST pause mid-run, acknowledge the failure inline, and call
`AskUserQuestion` with exactly two options:

- `Fix the failure now (I will investigate and re-run)`
- `Let the suite finish, then triage`

Subsequent failure lines in the same `Monitor` run are surfaced but do
NOT re-prompt. The "first" semantics is per-Monitor-invocation: a new
test run resets the gate.

**Non-interactive path** — no TTY on stdin OR `ORBIT_NONINTERACTIVE=1`
(this is `/orb:drive`, rally, cron, CI):

The agent MUST NOT call `AskUserQuestion`. On the first matching failure
line, emit the canonical non-interactive marker string below to stderr,
stop consuming further `Monitor` output, and halt with **exit status 2**.
The upstream orchestrator (drive) uses the exit-2 convention to route to
a checkpoint distinct from a clean test-suite failure (exit 1).

**Canonical non-interactive first-failure marker (single source of truth):**

> **FIRST_FAILURE_NONINTERACTIVE_MARKER** = `orbit: first-failure checkpoint skipped (non-interactive); halting for upstream triage`

This exact string is emitted verbatim. Test fixtures grep this file for
the constant and assert the emitted marker matches byte-for-byte.

## Completion

When all ACs are checked the bead is ready to close.

```bash
plugins/orb/scripts/parse-acceptance.sh has-unchecked <bead-id>
```

Exit status 1 means no ACs remain unchecked — the bead is done. The
agent then runs:

```bash
bd close <bead-id> --reason "<one-line summary of what shipped>"
```

After close, the agent suggests the next step:

> Run `/orb:review-pr` to verify the implementation.

If `bd close --suggest-next` is used, beads will surface any newly
unblocked beads in the same call.

## NO-GO outcome

Not every bead ships code. Some produce evidence that an approach
doesn't work — that's a valid outcome, not a failure. When results
invalidate the bead's hypothesis:

1. **Persist the finding.** Use `bd remember` so the insight survives
   the bead close:

   ```bash
   bd remember "<bead-id>: <evidence-backed insight>"
   ```

2. **Close with a NO-GO reason.**

   ```bash
   bd close <bead-id> --reason "NO-GO: <one-line evidence summary>"
   ```

3. **Card direction-layer updates remain the author's call.** Cards
   describe capabilities and are never closed; if the NO-GO requires the
   card's `goal` to narrow or shift, that's a `/orb:distill` task for
   the author, not an automatic skill action.

## Worked example — orbit-6da.1

This is a literal command trace using the bead this very skill was
written against. Each step is a copy-pasteable command.

```bash
# 1. Find the bead and claim it
bd ready --type task                            # see what's claimable
bd show orbit-6da.1                             # read description + ACs
bd update orbit-6da.1 --claim                   # atomic claim

# 2. Pre-flight
bd show orbit-6da.1
plugins/orb/scripts/parse-acceptance.sh acs orbit-6da.1
plugins/orb/scripts/parse-acceptance.sh next-ac orbit-6da.1
# → ac-01    1   (gate, unchecked, must close first)

# 3. Implement loop — close each AC
# (work the AC, then mark it done)
plugins/orb/scripts/parse-acceptance.sh check orbit-6da.1 ac-01
plugins/orb/scripts/parse-acceptance.sh next-ac orbit-6da.1
# → ac-02    0   (now startable — ac-01 gate cleared)

# ... repeat for ac-02, ac-03, ... ac-09 ...

# 4. (If a detour is discovered mid-AC)
bd create "Fix flaky parse-acceptance.sh test under bash 3.2" \
  -t task -p 1 \
  --parent orbit-6da.1 \
  --deps "discovered-from:orbit-6da.1" \
  -d "Surfaced while implementing ac-04; test passes on bash 5 but \
parse-acceptance.sh check fails silently under bash 3.2 (macOS default)."
bd update <new-id> --claim
# ... fix it ...
bd close <new-id> --reason "Quoted ac_id in sed expression for bash 3.2 compat"
bd show orbit-6da.1
plugins/orb/scripts/parse-acceptance.sh next-ac orbit-6da.1

# 5. Completion
plugins/orb/scripts/parse-acceptance.sh has-unchecked orbit-6da.1
# (exit 1 → all done)
bd close orbit-6da.1 --reason "Rewrote /orb:implement to read ACs from \
the bead acceptance field; progress.md / TaskCreate / drift detection / \
resume reconcile removed; detours escalate as sub-beads with \
discovered-from provenance."
```

## Integration with other skills

- **`/orb:review-pr`** reads the bead's acceptance field (via
  `parse-acceptance.sh acs`) to cross-reference AC coverage against the
  implementation diff. The cold-fork review architecture (decision 0011
  D2) is preserved — the reviewer reads the bead, not a spec file.
- **`/orb:review-spec`** runs against `spec.yaml` artefacts at design
  time; once a spec promotes to a bead via `promote.sh`, this skill
  takes over.
- **`/orb:drive`** invokes `/orb:implement` as one stage of an
  end-to-end pipeline; in non-interactive contexts the
  `FIRST_FAILURE_NONINTERACTIVE_MARKER` exit-2 convention is the
  handoff.

---

**Next step:** after `bd close`, run `/orb:review-pr` to verify the
implementation against the bead's acceptance criteria.
