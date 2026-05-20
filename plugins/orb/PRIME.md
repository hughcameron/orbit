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
orbit memory remember <key> "<body>"   # Persists across sessions (key is a short stable id)
orbit memory search <keyword>          # Operator-keyword substring search
orbit memory match <topic> --label <slug>  # Decision-moment ranked matching — surface memories relevant to the current work
```

`memory match` is the decision-moment surface. Lead memory bodies with mechanism ("use X for Y") not state ("Y is hard") — the warning fires at `memory remember` time when the body opens with a state observation. Memories matching active work (above threshold 0.3) must be reconciled in a spec's `memories_considered` field; `spec.close` refuses closure on unreconciled matches unless `--force` is passed.

## Session Close

Before finishing: `orbit task done <spec-id> <task-id>` for every completed task,
then `orbit spec update <spec-id> --ac-check ac-NN` for any ACs the work satisfied.
