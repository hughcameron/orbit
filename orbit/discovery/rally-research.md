# Precedents for orbit's rally (multi-card drive), mapped

The strongest precedent for rally drive combines **three patterns from three different families**: CodeR's explicit task-graph artefact, MetaGPT's structured-document contracts, and LangGraph's interrupt/resume with a typed checkpoint. Nothing in the surveyed field automates the hard qualification question ("which cards are safely parallel?"), so orbit should treat qualification as a human decision augmented by cheap static checks, not an agent judgement. The reviewed failure modes are consistent across tools: **sub-agents improvise output paths when prompts are the only contract, "parallel" work silently becomes serial when a shared type lands, and in-memory orchestrator state dies with the process**. Each of these has a mitigation with working prior art that fits orbit's file-based, skill-driven model.

The analysis below maps precedents onto the five design questions you posed, then closes with prioritised recommendations and a prototype/defer split.

## What qualifies cards for a rally

No surveyed framework does reliable automatic qualification, and several explicitly warn against it. Claude Code issue #7406 documents the main agent *claiming* to parallelise but running serially; Cursor's own best-practices blog says bluntly *"if Agent B needs Agent A's output, you cannot parallelise them"*; and a claudefa.st case study describes three parallel sub-agents each inventing a different shape for the same `preferences` type, costing twenty minutes of debugging to avoid a thirty-second sequential step. **The consistent recommendation is to declare a shared contract first, commit it, then fan out.**

Where qualification *is* automated, it's via declarative primitives rather than inference. CrewAI uses `async_execution=True` on tasks plus a downstream sync task listing them in `context=[...]` as the join barrier. LangGraph's Send API returns `[Send("worker", {...}) for x in state["items"]]` from a conditional edge and auto-joins at any shared successor node via reducers. Airflow 2.3+ uses `expand()`/`partial()` for dynamic task mapping with a default cap of 1024 tasks. Temporal's child-workflow guidance is stricter: recommend fewer than 1,000 children per parent because each child's lifecycle events land in the parent's Event History.

Build systems offer the cleanest heuristic: **parallelisability is derived from explicit declared inputs and outputs, not inferred**. Bazel's BUILD files list `srcs`, `hdrs`, `deps` and the scheduler parallelises any targets whose declared sets are disjoint; sandboxing enforces that undeclared inputs don't silently work. Nx's `affected` command walks a project graph from changed files. Orbit's analogue is straightforward: each card can declare a `paths:` or `touches:` glob set, and rally qualification becomes a deterministic disjointness check plus a shared-prerequisite rule, not a skill.

**Recommendation.** Keep qualification as an explicit human step at the rally gate, supported by a deterministic pre-flight: reject a rally whose cards overlap on declared file globs, and reject a rally where any two cards name the same trait, type or migration target in their prerequisites. Do not add an "is this parallelisable" skill; the Claude Code, Cursor and Devin evidence is that LLM qualification judgements are routinely wrong.

## How parallel designs stay on the artefact path

This is the best-studied failure mode in the literature. **MetaGPT's structured communication interface is the strongest positive precedent**: each role has a fixed deliverable schema with canonical file paths, agents publish to a shared pool, and *"an agent activates its action only after receiving all its prerequisite dependencies"*. The practical effect is that a downstream agent refuses to proceed until the expected artefact validates against its schema at the expected location. ChatDev's looser chat-chain approach, which relies on prompted conventions and a `<SOLUTION>` consensus token, is measurably more error-prone.

The pattern appears in three flavours across the field. CrewAI enforces `output_pydantic` schemas validated on return (roughly 50 to 100 tokens of schema-in-prompt overhead, worth it). OpenAI Agents SDK uses `output_type=MyPydantic` with `input_filter` on handoffs rewriting visible context. CodeR goes further and makes communication *exclusively* via standardised report messages on JSON-defined graph edges, decoupling plan from agent implementation.

Of the three options you floated, the evidence prefers **(A) lead agent includes output paths in sub-agent briefs combined with physical enforcement**, not prompts alone. Claude Code ships three layers that translate directly: the `tools:` allow-list in sub-agent frontmatter (hard restriction), the `skills:` frontmatter field that injects the full text of named skills into the sub-agent's context at startup, and `PreToolUse`/`PostToolUse` hooks that can block writes outside a path. Anthropic's SDK docs are explicit that *"the only channel from parent to sub-agent is the Agent tool's prompt string"*, so paths and conventions must travel in that prompt or be pre-installed via skills.

Option (B), running `/design` recursively as a sub-skill, fights Claude Code's architecture. Sub-agents cannot spawn sub-agents (issue #4182), and `claude -p` workarounds lose context. Option (C), where sub-agents return content and the lead writes files, is the safest schema-wise but costs context budget on every return. The SWE-agent Agent-Computer Interface principle argues for a middle path: small, structured actions with guardrails and a validator on write.

**Anti-pattern to flag.** One practitioner write-up describes parallel agents creating `*-enhanced` files alongside originals, producing what they called a "parallel universe" in the codebase. Sub-agents default to *adding* new files when uncertain, rather than editing existing ones. The mitigation is a post-write validator hook that rejects paths outside the declared card directory.

**Recommendation.** Combine option (A) with a PreToolUse hook: each design sub-agent receives a brief that names its canonical output path (`orbit/specs/<card-id>/interview.md`), skills are injected via a `skills:` frontmatter equivalent, and a hook blocks writes outside `orbit/specs/<card-id>/`. This mirrors the Claude Code sub-agent model exactly and fits orbit's existing skill system without breaking the "no sub-skills" rule.

## What artefact records the rally plan

The cleanest prior art is **CodeR's JSON task graph, which the paper describes as decoupling "agent design with the task decomposition"**. Plans can be added, deleted and tuned without changing a line of agent code. Magentic-One's two-layer design is similar: a high-level **Task Ledger** (facts, guesses, step-by-step plan) plus a per-step **Progress Ledger** with loop-detection fields (`is_request_satisfied`, `is_in_loop`, `next_speaker`). The Anthropic multi-agent research blog reports the lead agent *"writes its plan to a Memory file to persist the context"* because the 200k context window truncates otherwise; durable state beats in-context state.

Workflow engines suggest concrete manifest fields worth stealing. Airflow DAGs capture task IDs, dependencies (`>>`, `<<`), retry policy, `execution_timeout` and `on_failure_callback`. Temporal workflows persist Event History and support Signals (async input), Queries (state read), and Updates (sync input with return value). Prefect's `suspend_flow_run` pairs nicely with `wait_for_input=SomeModel` to capture typed human input via auto-generated UI forms, and the docs explicitly recommend task caching because resume re-executes from the beginning.

**The architectural warning is consistent**: do not trust an LLM to maintain the plan file. Magentic-One's ledger parsing is documented as fragile on weaker models (issue #6599: markdown fences around JSON break parsing). AutoGen's own docs warn that `save_state()` mid-run is inconsistent. The pattern that works is: **code writes the plan, sub-agents update fields via explicit tool calls, the plan is append-only or has clear transitions, and the plan is read back on resume rather than reconstructed from conversation**.

Shelley (the successor to sketch.dev) has a public retrospective worth quoting for orbit's file-based philosophy: *"We assumed your container would live exactly as long as your conversation with the agent. This was a mistake. Imagine having IT wipe your laptop clean every day?"* They rebuilt around a persistent per-project sqlite DB. The file-based orbit equivalent already exists in `drive.yaml`; extend it rather than invent a new mechanism.

**Recommendation.** Add `rally.yaml` as a higher-level sibling of `drive.yaml` with these fields: rally id, started timestamp, autonomy level, ordered list of card refs with per-card spec directory and worktree/branch name, declared file-path globs per card, dependencies (explicit edges), current phase (design / review / implement / pr-review), and a `blocked_by` field populated when parallel-to-serial conversion happens. Persist after every phase transition. Make the code the writer; sub-agents update only via named tool calls.

## When parallel becomes sequential

Dependency detection mid-flight is **the single weakest area across every system reviewed**. MetaGPT sidesteps the problem by having the Architect partition files up front, so two Engineers never overlap by construction. ChatDev is serial. OpenHands and AgentScope give you delegation and fanout primitives with no conflict detection. Cursor's docs admit local and cloud agent contexts are "summarised and reduced" before sharing, which commentators correctly note is vague. The consensus practical workaround in the Claude Code, Cursor and Devin communities is to **commit shared contracts first, then fan out**; no tool does this automatically.

The one framework that offers anything close to mid-flight replanning is Magentic-One, whose outer-loop Orchestrator replans after `max_stall_count` no-progress steps, with a `max_reset_count` circuit breaker for the whole run. LangGraph's `Command` return value lets a node dynamically choose the next node and update state, which is a cleaner primitive but still requires the supervising node to *know* a dependency has emerged.

Devin's "Managed Devins" (March 2026) has the most complete coordinator role in production: *"the main Devin session acts as a coordinator: it scopes the work, assigns each piece to a managed Devin, monitors progress, resolves any conflicts, and compiles the results."* It can message children mid-task, put them to sleep, or terminate them. Note this is model-driven rather than rule-driven, which is both its strength and its limit.

The observed orbit failure (cards 0015 to 0017 sharing trait changes) is the textbook case. The fix that shows up repeatedly in practitioner write-ups is static, not dynamic: **scan the N designs for a shared set of signals before starting implementation**, specifically references to the same trait, type, interface, migration, protobuf schema, or public API path. If the set is non-empty, the lead agent proposes a serial order and the human confirms at the existing consolidated design review gate.

A Temporal design lesson is worth adopting here: Signals and Queries separate async input (user interrupts) from state reads. For orbit, the equivalent is that `rally.yaml` records the proposed ordering and a `serialised_reason` field describing *why* parallel became serial, so the decision is inspectable and reversible.

**Recommendation.** Do not attempt runtime deadlock detection. After the consolidated design review, run a deterministic scan over the approved designs: extract every symbol name that appears in a design's "changes" section, compute the intersection set across cards, and if non-empty, the lead agent updates `rally.yaml` with a serial order and a rationale. Implementation then walks the serial order. This matches the observed behaviour you already want and mirrors Bazel's "disjoint declared outputs means safe to parallelise" principle at the semantic level rather than the file level.

## How assurance scales with rally size

Three patterns appear, and evidence suggests they are complementary rather than alternatives.

**Consolidated review at rally boundaries** (one human touch per phase) is the Anthropic research-system pattern, where a separate `CitationAgent` stage aggregates sub-agent output before surfacing to the user. LangGraph's parallel-interrupts mechanism is the closest framework-level support: when multiple parallel nodes each call `interrupt(payload)` in the same superstep, the caller receives a list of `Interrupt` objects with unique IDs and resumes with a `Command(resume={id1: v1, id2: v2, ...})` dict. This is direct prior art for orbit's "review N designs at one gate, resume N implements" flow. Watch the known bug (issues #6624, #6626) where parallel tool calls *inside one ToolNode* can collide on IDs; true parallel nodes each with their own interrupt work correctly, which is what orbit needs.

**Stacked PR review** is mature tooling and strongly relevant to orbit's final gate. Graphite, git-spice, Sapling, Aviator and GitHub's own stacked-PRs preview (April 2026, private) all implement the same ergonomic: a reviewer walks the stack bottom-up, each PR is small, dependencies are explicit. Jackson Gabbard's much-quoted line is that engineers who've used stacked diffs *"seek it wherever they next go"*; discount for marketing bias, but the underlying claim that small stacked reviews cut cycle time from days to hours is broadly supported. Critically, Aviator's merge-queue rallying (`rally_size`, parallel mode) handles the case where the stack passes CI together rather than one PR at a time, which matters when N cards share infrastructure. GitHub's merge queue itself does not rally efficiently; community discussions flag it spending 1,000 CI-minutes where Bors or Mergify would rally to ten.

**Individual PR review** is the safe fallback and the default in Cursor (per-agent PRs) and Conductor (per-workspace PRs). It defeats the throughput goal but is the only option when cards have diverged enough that review-as-a-stack would mislead.

The decision framework in the stacked-PR literature translates directly: review as a stack when changes build on each other and share rationale; review as a rally (single diff against main) when changes are genuinely independent but you want a single sweep; review individually only when a card's implementation diverged materially from the approved design. **Orbit's two-gate structure (ideation, assurance) maps cleanly: the assurance gate should default to stacked review for rallyes that hit serial implementation, and rallyed diff for rallyes that stayed parallel.**

**Anti-pattern to avoid.** Warp's own design guidance is worth citing: *"In the future, Warp may add optional agent-on-agent review, but our foundation is built around human oversight."* Do not let the rallyed-review workload drift into agent-reviews-agent at the assurance gate. That's the ideation gate's job, and even there only as a prompt for the human reviewer.

**Recommendation.** Default the final gate to **stacked review** when implementation was serial, and **rallyed diff review** when implementation was parallel and cards did not share files. Fall back to individual PR review only when `rally.yaml` records a divergence from the approved design. Do not implement cross-PR agent review at this gate.

## Cross-cutting failure modes worth designing against

Five failure modes show up across multiple frameworks and map directly onto orbit's observed issues.

The **improvised-folder failure** (your own observation, the `designs/` folder) is Claude Code issue #37549 in a different guise (silent no-op on `isolation: worktree` when combined with `team_name`) and the "parallel universe" Medium write-up's `*-enhanced` file pattern. Mitigation: physical enforcement beats prompted discipline, always. Combine `tools:` allow-list, `skills:` frontmatter injection of conventions, and a `PreToolUse` hook that rejects writes outside the card directory. Verify after each sub-agent returns that the expected artefact exists at the expected path before continuing.

The **parallel-turns-serial revelation** (cards 0015 to 0017) is the shared-types case in the claudefa.st study and the liranbaba.dev Cursor review. Mitigation: a deterministic scan of designs before implement, updating `rally.yaml` with a serial order and a rationale.

The **session-death mid-rally** failure is Shelley's explicit retrospective on ephemeral containers, Claude Code Agent Teams' documented limitation (*"/resume and /rewind do not restore in-process teammates"*), and AutoGen's `save_state()` mid-run inconsistency warning. Mitigation: `rally.yaml` written to disk at every phase transition; nothing important lives only in the orchestrating process's memory.

The **orchestrator mis-claims parallelism** failure is Claude Code issue #7406. Mitigation: the code orchestrator, not the LLM, decides whether to fan out, based on the declarative plan in `rally.yaml`.

The **fragile LLM-maintained plan** failure is Magentic-One's issue #6599 (markdown fences around JSON). Mitigation: the plan is code-written YAML; sub-agents only touch it via named tool calls with schema validation.

## Prioritised recommendations for rally drive

The following sequence is ordered by risk reduction per unit of implementation effort, informed by which patterns have the strongest prior-art support:

1. **Define `rally.yaml` before writing any orchestration code.** Fields: rally id, started timestamp, autonomy, cards (id, spec dir, worktree, declared path globs, prerequisites), phase, dependencies, serialisation rationale. Make it the single source of truth for resume. This is the CodeR plan-artefact pattern plus Shelley's durable-state lesson.
2. **Use Claude Code sub-agent frontmatter for on-path enforcement.** Brief each parallel design sub-agent with its canonical output path, inject the design skill's conventions via `skills:`, restrict its tools, and add a `PreToolUse` hook blocking writes outside the card directory. Do not adopt option (B) (running `/design` as a sub-skill); Claude Code's sub-agents-cannot-spawn-sub-agents limit rules it out.
3. **Run a deterministic shared-symbol scan after the consolidated design review.** Extract every trait/type/interface/migration/API path from each approved design; intersect across cards; if non-empty, the lead proposes a serial order and updates `rally.yaml` with a `serialised_reason`. This handles the observed 0015-to-0017 case without mid-flight replanning.
4. **Default the assurance gate to stacked review for serial rallyes, rallyed diff for parallel ones.** Adopt the Aviator/Graphite model rather than individual-PR-per-card review; fall back only on material divergence recorded in `rally.yaml`.
5. **Write `rally.yaml` after every phase transition, not mid-phase.** This is LangGraph's atomic-superstep model adapted to files: state is snapshot at gate boundaries, not in-flight. Sub-agents' in-progress state is treated as transient.

## What to prototype first, what to defer

**Prototype first** (highest value for effort):

- `rally.yaml` schema plus read/write helpers, and the single-card-at-a-time rally of size one as a smoke test for the manifest round-trip across session death
- Parallel design fan-out with explicit per-sub-agent output-path briefs and the PreToolUse hook, tested on a two-card rally with disjoint paths
- Deterministic shared-symbol scan and serial ordering, tested against the 0015-to-0017 case as a regression fixture

**Defer** (high complexity, unproven benefit for orbit's constraints):

- Automatic rally qualification as a skill; keep it human
- Runtime dependency detection during implementation; rely on the post-design scan
- Cross-PR agent review at the assurance gate; keep human oversight canonical
- Worktree-per-card isolation; the file-path hook plus declared globs is sufficient for orbit's current scale, and worktrees carry the documented disk-cost and runtime-isolation pitfalls (Cursor users reporting 9.8GB in a 20-minute session, shared ports and DBs across worktrees)
- Agentless-style N-candidate design sampling; conceptually attractive but the selector problem (41% oracle vs 32% selected in the paper) means the human reviewer becomes the bottleneck rather than the beneficiary

## What changed in the understanding

Going in, rally drive looked like a coordination problem: how does one agent manage N parallel children well. The evidence says that's the easy part. The hard parts are **durable plan state, physical enforcement of output contracts, and explicit handling of the parallel-to-serial transition**, and all three have strong prior art in adjacent domains. Orbit's existing design principles (file-based state, single-session skills, inline invocation, British English) are actually *advantages* here: they match Shelley's hard-won durable-state lesson, MetaGPT's artefact-path discipline, and Aider's "the git history is the audit trail" philosophy more closely than any of the agent-native frameworks. The rally.yaml proposal, combined with Claude Code's existing sub-agent primitives and a deterministic dependency scan, covers every documented failure mode in the surveyed field without introducing new orchestration machinery.

The one genuine research gap is automatic qualification. The surveyed field has no working answer, and orbit should not try to solve it. A disjointness check on declared file globs plus a human gate is state-of-the-art, and it fits the two-gate model already in place.
