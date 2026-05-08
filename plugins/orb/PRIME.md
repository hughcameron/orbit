# Orbit Execution Context

## Start

```bash
orbit session prime    # Open specs + recent memories (bounded)
orbit task ready       # Claimable work (open, no claim)
```

## Work Loop

```bash
orbit task show <spec-id> <task-id>     # Read task before starting
orbit task claim <spec-id> <task-id>    # Claim (atomic, prevents races)
# ... implement against acceptance criteria on the parent spec ...
orbit spec update <spec-id> --ac-check ac-NN   # Flip an AC to checked
orbit task done <spec-id> <task-id> --body "what shipped"
```

ACs live on the spec, not the task. Use `orbit spec show <spec-id>` to read them.

## Decisions

```bash
orbit memory remember "insight"     # Persists across sessions
orbit memory search <keyword>       # Search prior decisions
```

## Session Close

Before finishing: `orbit task done <spec-id> <task-id>` for every completed task,
then `orbit spec update <spec-id> --ac-check ac-NN` for any ACs the work satisfied.
