# Decision pack — 2026-05-19-act-when-authorised

Card: `.orbit/cards/0042-act-when-authorised.yaml`
Spec goal: *agents in mid-session autonomy halt only when at least one of {recommendation, evidence, authorisation} is genuinely missing; the three-question test fires at every halt-temptation moment as the inverse of "consult the substrate first".*

Source memo (deleted, retrievable via git): `.orbit/memos/2026-05-18-drive-autonomy-too-ready-to-halt.md` (commit `a38d040`, deleted by `ea77286`). Authorising memory: `mid-session-autonomy-contract-default-to-action-halt` (`.orbit/memories/mid-session-autonomy-contract-default-to-action-halt.yaml`).

Sibling cards in the same rally (`2026-05-19-agent-side-substrate-engagement-rally`):

- `0037-memory-gates-decisions.yaml` — "consult memory at the decision moment"
- `0038-skills-infer-or-prompt-before-halt.yaml` — "infer → prompt → halt" recovery sequence on missing args
- `0042` (this card) — **inverse failure mode**: act when substrate already authorises, do not halt

The line between 0042 and 0037/0038 is itself a load-bearing design question — decision D1 below names it explicitly.

---

## D1 — Where the three-question test lives (skill prose vs hook vs CLI verb)

**Context.** Card 0042 names a candidate structural fix: *"a pre-halt check in `/orb:drive` that runs the three-question test before AskUserQuestion fires"* (notes, last bullet). The memo lists three options and judges the pre-halt check as "the actual structural fix" because it "encodes the discipline where the halt-temptation arises". Memory `mid-session-autonomy-contract-default-to-action-halt` already names the rule but did not prevent the failure — skill-prose enforcement alone is known-insufficient (0037 scenario "Skill-prompt-only enforcement is insufficient" makes the same finding for the inverse failure mode).

**Options.**

1. **Prose-only.** Add a "three-question test" subsection to `plugins/orb/skills/drive/SKILL.md` (and a smaller pointer in `plugins/orb/skills/rally/SKILL.md` and `plugins/orb/skills/implement/SKILL.md`) just before each existing AskUserQuestion site. No code, no hook, no verb.
2. **Pre-halt hook.** A Claude Code `PreToolUse` hook on `AskUserQuestion` that fires when the calling agent is inside drive/rally autonomy. The hook prints the three-question test to stderr and (if `ORBIT_NONINTERACTIVE=1`) exits non-zero to suppress the halt, forcing the agent to either act or escalate via the structural NO-GO path. Lives in `plugins/orb/hooks/` and is wired in `plugins/orb/.claude-plugin/plugin.json`.
3. **CLI verb `orbit autonomy authorised?`.** A read-only verb that returns `{authorised: bool, missing: [recommendation|evidence|authorisation]}` based on inputs the agent provides (the current spec id, the proposed action, the relevant memory keys). Skills call it inline before any AskUserQuestion under `ORBIT_NONINTERACTIVE=1`. Lives alongside the existing `orbit-acceptance.sh` helpers and `orbit spec` verbs.

**Trade-offs.**

| Option | Gains | Loses |
|--------|-------|-------|
| 1 Prose-only | Cheap, no new substrate. Reuses existing skill files. | Repeats the failure mode the memo identified: the rule was already in memory and was ignored. 0037 ac-06 explicitly rejects this shape for the sibling problem. |
| 2 Pre-halt hook | Structural — fires at the actual halt-temptation moment, asymmetric to whether the agent remembers to check. Survives recency bias and sunk-cost momentum. | Requires hook infrastructure that does not yet exist in `plugins/orb/` (no `hooks/` dir currently); adds a new failure surface (hook misfire = false-positive halt suppression). |
| 3 CLI verb | Structural and testable — the verb is exercisable in unit tests, the contract is a function signature not a paragraph. Composes with 0038's resolver discipline (infer → prompt → halt). | Requires the agent to remember to call the verb, which is the same enforcement gap as Option 1 unless paired with skill-level "must-call" wording. |

**Recommendation. Option 2 (pre-halt hook), with Option 1 prose as the human-readable explanation.** The memo's own analysis named the structural fix as highest-leverage, and 0037's "Skill-prompt-only enforcement is insufficient" scenario provides cluster-coherent evidence that prose alone fails for this class of bug. A hook fires regardless of agent attention. Pair with concise prose at the affected AskUserQuestion sites in `drive/SKILL.md` so a human reading the skill understands the rule the hook is enforcing.

Confidence: medium — assumes Claude Code supports `PreToolUse` hooks for `AskUserQuestion` (worth verifying before lock-in). If not, fall back to Option 3 + mandatory prose at every AskUserQuestion site.

---

## D2 — Three-question contract: exact wording and authorisation source-of-truth

**Context.** The card names three questions but does not pin their phrasing or the source the agent consults to answer "does the contract authorise me?". This phrasing is what the hook prints, what the prose embeds, and what `/orb:review-pr` will check existence of. Vague phrasing here propagates everywhere.

**Options.**

1. **Memo phrasing verbatim.** *"Do I have a recommendation? Do I have the evidence to act on it? Is the spec/contract authorising me to proceed?"*
2. **Substrate-typed phrasing.** *"R: do I have a recommendation? E: do I have evidence (memory key, AC text, prior decision)? A: does the contract authorise me (`drive.yaml.autonomy`, memory `mid-session-autonomy-contract-default-to-action-halt`, spec halt-conditions)?"* — each question names the concrete substrate the agent reads.
3. **Inverted three-question test.** *"Is the recommendation missing? Is the evidence missing? Is authorisation missing?"* — at least one yes → halt; all three no → act. Aligns with the card's goal phrasing ("at least one of {recommendation, evidence, authorisation} is genuinely missing").

**Trade-offs.**

| Option | Gains | Loses |
|--------|-------|-------|
| 1 Memo phrasing | Preserves the lived voice of the original observation. | "Spec/contract" is ambiguous — there are at least three contract sources (drive.yaml, memory, spec halt-conditions). |
| 2 Substrate-typed | Removes ambiguity; the agent reads named files. Composable with 0037 (memory) and 0038 (binding). | Slightly heavier prose; risk of over-specifying if substrate surfaces evolve. |
| 3 Inverted | Matches the card's own goal text exactly. Easier for `/orb:review-pr` to verify ("did the agent identify which of the three was missing?"). | Slightly less intuitive at the halt-temptation moment — the agent has to invert before acting. |

**Recommendation. Option 2 (substrate-typed phrasing).** The card's ac-05 names "memory + contract" as load-bearing authorisation; substrate-typed phrasing makes that explicit and audit-checkable. The named substrate also gives the hook (D1) concrete things to check rather than asking the agent to self-report a yes/no. Phrase the test as a positive three-yes gate (memo voice), but each question names its substrate source.

Confidence: high — the named substrate already exists (`drive.yaml`, the memory, spec `halt-conditions` per memo line 5).

---

## D3 — How severity is taught to be reviewer-language, not autonomy-language (ac-02)

**Context.** The memo's clearest concrete failure was treating a HIGH-severity review-spec finding as escalation-worthy. `drive/SKILL.md` lines 290–295 and lines 423–448 already encode the four-option verdict prompt and the §1.6 budget — REQUEST_CHANGES under guided autonomy IS supposed to be absorbed by the cycle budget. So the bug is not in the contract; it is in the agent's reading of it. Ac-02 wants this conflation prevented going forward.

**Options.**

1. **Edit `drive/SKILL.md` §1.5 / §1.6** to add a one-line clarification: *"Severity (LOW/MEDIUM/HIGH) is reviewer-language. Under guided or full autonomy, severity does not change the routing — REQUEST_CHANGES is absorbed by the cycle budget regardless of severity. Severity informs priority of fixes within a cycle, not whether to surface to the operator."*
2. **Move the clarification into the hook (D1).** When the agent is about to AskUserQuestion mid-cycle under guided/full autonomy with a non-APPROVE verdict, the hook fires with the severity-as-reviewer-language reminder.
3. **Both.** Skill prose carries the rule; hook reinforces at the trigger moment.

**Trade-offs.**

| Option | Gains | Loses |
|--------|-------|-------|
| 1 Skill prose | Cheapest. Lives where the routing is defined; reader sees the rule next to the mechanism. | Same enforcement gap as D1 Option 1 — prose lost to recency bias. |
| 2 Hook only | Fires at trigger; preserves drive.md's existing brevity. | The clarifying *why* is no longer co-located with the cycle budget rules; a human reading drive.md misses the connection. |
| 3 Both | Coherence: prose explains, hook enforces. | Two places to update if the rule changes (manageable — the rule is unlikely to change). |

**Recommendation. Option 3 (both).** The cost of duplication is trivial here (one sentence × two locations). Skill prose at `drive/SKILL.md` §1.6 is the human-readable definition; the hook from D1 cites the same rule when it fires.

Confidence: high — `drive/SKILL.md` §1.6 already contains the cycle-budget rule; we are adding one clarifying paragraph, not changing routing.

---

## D4 — Pre-commit halt scope: stage vs surface (ac-04)

**Context.** The memo's contributing-factor #1 was "I extended that halt to the review-spec stage where the same surface was being decided in spec text". `drive/SKILL.md` has no explicit `pre-commit halt` mechanism — the memo's usage refers to halt-conditions named in the spec or in `mid-session-autonomy-contract-default-to-action-halt`. The card's ac-04 wants the scope rule encoded: a halt registered against ac-02 hook registration during `/orb:implement` does not widen to cover the same surface during `/orb:review-spec`.

**Options.**

1. **Spec-level halt-conditions field with explicit `stage:`.** Extend the spec schema so any halt-condition entry carries a required `stage: design | review-spec | implement | review-pr` field. Drive and the implementing skills read `stage` and only honour halts whose stage matches the current pipeline phase. Lives in `orbit spec` verb and `plugins/orb/skills/spec/SKILL.md`.
2. **Implicit stage scope, no schema change.** Document in `drive/SKILL.md` that pre-commit halts named in spec text apply only to the stage in which they were registered (a halt arising from `/orb:implement` does not apply during `/orb:review-spec`). Drive's hook (D1) treats stage-cross widening as a violation.
3. **Explicit `surface:` field with default `surface: this-stage-only`.** Author opts in to surface-wide halts via `surface: any-stage-touching-<surface-id>`. Default is stage-local.

**Trade-offs.**

| Option | Gains | Loses |
|--------|-------|-------|
| 1 Schema with `stage:` | Audit-trail: every halt has a named stage. Hook can read it directly. Composes with `/orb:review-spec` ac coverage checks. | Schema change touches `orbit spec` verbs and the spec yaml shape — broader surface, possible coupling with 0037's `memories_considered` work. |
| 2 Prose-only | No substrate change. Cheapest. | Same memo failure mode: agent re-conservatively widens because prose interpretation differs from author intent. |
| 3 `surface:` opt-in | Sensible default (stage-local), explicit opt-in to widen. | A second field on every halt; risk of confusion between `stage:` and `surface:`. |

**Recommendation. Option 2 (prose-only with hook reinforcement).** Spec halt-conditions are currently free-form prose carried in memory and in spec notes; introducing a schema field for stage scope is heavier than the failure mode warrants. The hook from D1 can carry the stage-scope rule as part of its three-question check (the "is authorisation missing?" question reads the current pipeline stage from `drive.yaml` and matches it against the halt-condition's stage). Revisit Option 1 only if `/orb:review-pr` audit traceability work (a different card) needs structured halt-conditions for other reasons.

Confidence: medium — assumes the agent reading `drive.yaml.stage` is reliable. Low confidence in Option 1 being unnecessary; if a second instance of stage-cross widening appears in the wild, escalate to Option 1.

---

## D5 — Decision Brief framing rule: how `/orb:review-pr` audits ac-03

**Context.** Ac-03 says the BLUF / Decision Brief frame from card 0026 is for *closing recommendations to the operator*, not for *in-flight decisions* mid-autonomy. The memo named "three options with a recommendation" as the seductive failure shape. This rule has to be checkable by `/orb:review-pr` (the spec acceptance criterion needs evidence).

**Options.**

1. **Verbal-only rule in `STYLE.md` / `0026-executive-communication.yaml`.** Add a paragraph: *"The Decision Brief shape closes recommendations to the operator. Mid-autonomy in-flight decisions take the imperative single-action form (one line: `Run X on Y`); they do not present a menu of options."* No mechanical check.
2. **Pattern-match check in `/orb:review-pr`.** Reviewer looks at every AskUserQuestion site invoked mid-autonomy during the spec's drive (visible in `drive.yaml` and the spec note stream) and flags any whose body matches the menu-of-options shape (regex on "Option 1 / Option 2" or three-suggested-answers + "recommendation" in body). Findings are LOW severity by default.
3. **Hook-side check at AskUserQuestion (extends D1).** The pre-halt hook reads the proposed AskUserQuestion body and, if it matches the menu shape, prints a stronger warning ("menu-presenting mid-autonomy: surface a single imperative instead").

**Trade-offs.**

| Option | Gains | Loses |
|--------|-------|-------|
| 1 Prose-only | Cheap; lives next to 0026 where the rule belongs. | No closing-the-loop mechanism; the rule is recorded but not verified. |
| 2 Audit in review-pr | The rule is checkable by an agent reading drive state after the fact. Composes with 0037's audit-style enforcement (memories_considered). | The check happens after the fact, not at the halt-temptation moment. False positives possible on legitimate operator-closing questions. |
| 3 Hook-side check | Fires at the right moment. Composes naturally with D1's hook. | Pattern-matching natural-language question bodies is fragile; false positives risk over-suppression. |

**Recommendation. Option 1 (prose-only update to `STYLE.md` and `0026-executive-communication.yaml`).** The failure-mode is interpretive — agent misapplies a closing-frame to an in-flight moment — and the structural fix is the three-question test from D1, not a separate menu-detection mechanism. The three-question test already prevents the in-flight AskUserQuestion at all; if it fires, the body shape is moot. Add the prose anchor so a future reader of 0026 understands the boundary, and reference it from the three-question test prose so the connection is explicit.

Confidence: high — D1's hook subsumes the structural enforcement; ac-03 needs the rule articulated, not a separate check.

---

## D6 — Where 0042 ends and 0037 / 0038 begin (boundary check for the rally)

**Context.** The rally lead computes disjointness across all three cards at Stage 4. Naming the boundary precisely now reduces the chance of the consolidated design review forcing serialised execution. The substantive overlap is the AskUserQuestion call site: 0037 wants memory consulted before the question is asked; 0038 wants infer-then-prompt-then-halt for missing-arg cases; 0042 wants the three-question test before any halt fires.

**Options.**

1. **Stack the three rules sequentially in the same hook.** Single `PreToolUse` hook on `AskUserQuestion`: (a) memory-match check (0037), (b) infer-from-binding check (0038), (c) three-question test (0042). One hook, three independent checks.
2. **Separate hooks, ordered.** Three `PreToolUse` hooks registered with explicit ordering. Each card owns its hook.
3. **Different surfaces.** 0037 fires at `/orb:design` open and at `spec.close`; 0038 fires at skill entry; 0042 fires at AskUserQuestion mid-autonomy. The three cards then touch different files and different lifecycle moments — no shared surface.

**Trade-offs.**

| Option | Gains | Loses |
|--------|-------|-------|
| 1 Stacked hook | Single hook file → easy disjointness check (one owner). Three checks compose naturally. | Three cards share one file → consolidated review forces serialised execution. |
| 2 Separate hooks | Each card owns its surface; clean disjointness. | Three hook files all listening to the same trigger; order coupling becomes a fragile contract. |
| 3 Different surfaces | Genuinely disjoint — 0037 and 0038 touch design / skill entry, 0042 touches the halt-temptation point. Each card stays in its own files. | Loses the symmetry of "three checks at the same hook"; 0042's mid-autonomy halt-temptation is genuinely a different moment from 0037's "memory at design time" and 0038's "spec-id at skill entry". |

**Recommendation. Option 3 (different surfaces).** 0037 (`/orb:design` open + `spec.close` block) and 0038 (skill entry resolver) already name surfaces distinct from 0042's halt-temptation moment inside drive/rally. Naming this explicitly in the spec lets the rally lead's disjointness check fire on disjoint files: 0042 touches `plugins/orb/skills/drive/SKILL.md`, `plugins/orb/skills/rally/SKILL.md`, `plugins/orb/skills/implement/SKILL.md`, a new `plugins/orb/hooks/three-question-test.sh` (if D1 lands Option 2), `.orbit/cards/0026-executive-communication.yaml`, `.orbit/STYLE.md`. 0037 touches `plugins/orb/skills/design/SKILL.md`, `plugins/orb/skills/spec/SKILL.md`, the spec schema's `memories_considered` field. 0038 touches `plugins/orb/skills/*/SKILL.md` front-matter + a shared resolver. Overlap is small (drive/SKILL.md may be touched by 0042 and rally/SKILL.md only; 0038's resolver does not touch drive).

Confidence: medium — assumes the rally lead's disjointness check is path-level, not symbol-level. If it is symbol-level, the three cards may share `AskUserQuestion`-adjacent symbols and serialise anyway. Worth flagging to the rally lead before Stage 4.

---

## Files this spec is likely to touch (named for the rally's disjointness check)

- `plugins/orb/skills/drive/SKILL.md` — §1.6 prose clarifying severity-as-reviewer-language (D3); three-question test introduction (D1 Option 1 prose half).
- `plugins/orb/skills/rally/SKILL.md` — pointer to the three-question test for rally sub-agents.
- `plugins/orb/skills/implement/SKILL.md` — pointer at the existing "stop and ask" line (lines 176–178) reminding the agent the three-question test fires first.
- `plugins/orb/hooks/three-question-test.sh` — NEW. The PreToolUse hook on AskUserQuestion (D1 Option 2). Only if Claude Code supports the hook surface.
- `plugins/orb/.claude-plugin/plugin.json` — register the hook (if D1 Option 2 lands).
- `.orbit/STYLE.md` — paragraph clarifying that the Decision Brief frame is for closing recommendations, not in-flight halts (D5).
- `.orbit/cards/0026-executive-communication.yaml` — mirror the same boundary in the card text (D5).

No spec file outside `.orbit/specs/2026-05-19-act-when-authorised/` is touched.

---

## Summary

| Decision | Choice | Confidence |
|----------|--------|------------|
| D1 Where the three-question test lives | Pre-halt hook + prose pointer | Medium |
| D2 Three-question wording | Substrate-typed phrasing | High |
| D3 Severity-as-reviewer-language | Skill prose + hook reinforcement | High |
| D4 Pre-commit halt scope | Prose + hook reinforcement | Medium |
| D5 Decision Brief framing rule | Prose-only update to STYLE / 0026 | High |
| D6 Card boundary (0037 / 0038 / 0042) | Different surfaces | Medium |
