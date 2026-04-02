# orbit

An opinionated specification-driven workflow for [Claude Code](https://claude.ai/claude-code).

**Card → Interview → Spec → Implement → Review → Ship.**

Every feature starts as a card. Every card becomes a spec. Every spec has acceptance criteria. Every AC maps to a test. The chain is auditable from either end.

## Install

```
/plugin marketplace add hughcameron/orbit
/plugin install orb@orbit
```

Then in any project:

```
/orb:init
```

This creates the directory structure (`cards/`, `specs/`, `decisions/`), adds a workflow snippet to your `CLAUDE.md`, and walks you through writing your first feature card.

## Workflow

```
Card ──→ Interview ──→ Spec ──→ Implement ──→ Review ──→ Ship
  │                      │                        │         │
  │ who, why, scenarios  │ goal, ACs, constraints │ tests   │ card updated
  │                      │                        │ AC IDs  │ with releases
  └──────────────────────┴────────────────────────┴─────────┘
                   artifacts are the handoff
```

### Workflow skills

| Skill | Purpose |
|-------|---------|
| `/orb:init` | Set up a project — directories, CLAUDE.md, first card |
| `/orb:card` | Write a feature card with scenarios |
| `/orb:interview` | Socratic Q&A to clarify requirements |
| `/orb:spec` | Generate a structured spec from an interview |
| `/orb:review-spec` | Stress-test a spec before implementation |
| `/orb:review-pr` | Verify a PR against the spec + AC coverage |
| `/orb:evaluate` | 3-stage verification (mechanical → semantic → consensus) |
| `/orb:evolve` | Iterate a spec based on evaluation results |

### Persona skills

These are loaded by workflow skills — you don't invoke them directly.

| Persona | Role |
|---------|------|
| `interviewer` | Socratic questioner |
| `spec-architect` | Spec extraction with numbered ACs |
| `advocate` | Case FOR a solution |
| `contrarian` | Challenge assumptions |
| `judge` | Render final verdict |
| `evaluator` | Run the 3-stage pipeline |
| `ontologist` | Identify essential nature |
| `simplifier` | Cut complexity |
| `hacker` | Unconventional workarounds |
| `researcher` | Systematic investigation |

## Concepts

### Feature cards

A card captures **who** needs something, **why** it matters, and **what they'd expect to see**. Scenarios are written in user language, not engineering language.

```yaml
# cards/0001-step-progress.yaml
feature: See pipeline step progress
as_a: analyst
i_want: to see progress of long-running steps as they execute
so_that: I know the job is still running and roughly how long is left

scenarios:
  - name: Step name appears before execution
    given: a pipeline with a long-running step
    when: the step starts
    then: the step name is visible immediately

  - name: Failure is obvious
    given: a step that fails
    when: the error occurs
    then: I can see which step failed and why
```

### Acceptance criteria → test naming

Every spec AC gets an `ac-NN` ID. Tests are prefixed with that ID, creating a machine-checkable link:

```
Spec:   ac-03: "Steps execute in declared order"
Test:   fn ac03_steps_execute_in_declared_order() { ... }
```

The `/orb:review-pr` skill checks this automatically — it parses the spec for AC IDs and greps test files for matching prefixes.

### Decisions

Decisions use the [MADR](https://adr.github.io/madr/) format and live in `decisions/`. They surface during interviews and are recorded immediately — not after implementation.

### Context separation

Review skills (`/orb:review-spec`, `/orb:review-pr`) run in a forked context — a fresh agent session with no shared conversation history. A reviewer who watched you build something has confirmation bias. A fresh agent reads the spec and diff cold.

## Directory structure

orbit prescribes this structure at your project root:

```
cards/                              # Feature cards
├── 0001-step-progress.yaml
├── 0002-search-without-sql.yaml
└── done/                           # Completed (optional)

specs/                              # Specifications & knowledge
├── 2026-04-02-step-progress/
│   ├── interview.md
│   ├── spec.yaml
│   ├── review-spec-2026-04-02.md
│   ├── review-pr-2026-04-02.md
│   └── evaluation-2026-04-02.md
└── ...

decisions/                          # MADR decision register
├── 0001-short-title.md
└── ...
```

## Design context

orbit builds on well-established ideas from the agile and software engineering community:

| Concept | Origin | Reference |
|---------|--------|-----------|
| Card, Conversation, Confirmation | Ron Jeffries, 2001 | "Essential XP: Card, Conversation, Confirmation" |
| User stories as planning tools | Mike Cohn, 2004 | *User Stories Applied* (Addison-Wesley) |
| INVEST quality criteria | Bill Wake, 2003 | Independent, Negotiable, Valuable, Estimable, Small, Testable |
| Gherkin scenario format | Cucumber project | [cucumber.io/docs/gherkin](https://cucumber.io/docs/gherkin/reference/) |
| Decisions as code (MADR) | ADR community | [adr.github.io/madr](https://adr.github.io/madr/) |
| Context-separated review | Claude Code research | Fresh-context review avoids confirmation bias |

## License

MIT
