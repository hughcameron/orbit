# Acceptance field format convention

Orbit uses the beads `acceptance` field to store structured acceptance criteria for work beads. This convention defines the format that orbit skills parse.

## Format

One line per AC. Lines that don't match the format are skipped with a warning.

```
- ac-NN [gate]: Description of the acceptance criterion
- ac-NN: Description of the acceptance criterion
```

- **`ac-NN`** — sequential identifier starting at `ac-01`. Zero-padded to two digits.
- **`[gate]`** — optional marker. Gate ACs block all subsequent ACs by declaration order.
- **`[x]`** — checked prefix replaces `- ` when the AC is complete: `- [x] ac-NN: ...`

### Parsing regex

```
^- \[( |x)\] (ac-\d{2,3})(\s+\[gate\])?:\s+(.+)$
```

Capture groups:
1. Check status: space = unchecked, `x` = checked
2. AC identifier: `ac-01`, `ac-02`, etc.
3. Gate marker: ` [gate]` or absent
4. Description text

### Gate enforcement rules

- A gate AC blocks all subsequent ACs by declaration order, regardless of whether those subsequent ACs are themselves gates.
- Non-gate ACs do not block each other.
- An unchecked gate means: the agent must not start any AC declared after it.
- A checked gate releases all subsequent ACs until the next unchecked gate.
- Multiple consecutive gates are valid — each must be checked in order.

### Checked vs unchecked

Unchecked:
```
- [ ] ac-01 [gate]: Decide hash algorithm
```

Checked:
```
- [x] ac-01 [gate]: Decide hash algorithm
```

The implement skill updates the acceptance field (via `bd update --acceptance`) to check off ACs as they complete.

## Worked examples

### Example 1: Spec with gates

```
- [ ] ac-01 [gate]: Decide hash algorithm before implementing drift detection
- [ ] ac-02: Implement sha256 drift check in pre-AC sequence
- [ ] ac-03: Write session-context.sh drift notice on resume
- [ ] ac-04 [gate]: Confirm schema ownership with dependent cards before extending progress.md
- [ ] ac-05: Write progress.md parser
- [ ] ac-06: Add gate enforcement to implement skill
```

**Parse result:**

| AC    | Gate | Status    | Blocked by |
|-------|------|-----------|------------|
| ac-01 | yes  | unchecked | —          |
| ac-02 | no   | unchecked | ac-01      |
| ac-03 | no   | unchecked | ac-01      |
| ac-04 | yes  | unchecked | ac-01      |
| ac-05 | no   | unchecked | ac-01      |
| ac-06 | no   | unchecked | ac-01      |

**Next AC:** ac-01 (first unchecked; it's a gate so nothing after it can start)

After checking ac-01:

| AC    | Gate | Status    | Blocked by |
|-------|------|-----------|------------|
| ac-01 | yes  | checked   | —          |
| ac-02 | no   | unchecked | —          |
| ac-03 | no   | unchecked | —          |
| ac-04 | yes  | unchecked | —          |
| ac-05 | no   | unchecked | ac-04      |
| ac-06 | no   | unchecked | ac-04      |

**Next AC:** ac-02 (first unchecked, not blocked — ac-04 is also available but ac-02 comes first)

### Example 2: Spec without gates

```
- [ ] ac-01: Add heartbeat CronCreate at drive start
- [ ] ac-02: Define heartbeat format string
- [x] ac-03: Document escalation ping one-shot
- [ ] ac-04: Add CronDelete at drive completion
```

**Parse result:**

| AC    | Gate | Status    | Blocked by |
|-------|------|-----------|------------|
| ac-01 | no   | unchecked | —          |
| ac-02 | no   | unchecked | —          |
| ac-03 | no   | checked   | —          |
| ac-04 | no   | unchecked | —          |

**Next AC:** ac-01 (first unchecked; no gates so nothing is blocked)

## Constraints carried from current spec conventions

- The acceptance field is the single source of truth for AC status within a bead.
- AC numbering is stable — IDs are never renumbered after creation.
- The implement skill computes "next AC" as the first unchecked item that is not blocked by an unchecked gate.
- Malformed lines (not matching the regex) are skipped with a warning, not treated as errors.
