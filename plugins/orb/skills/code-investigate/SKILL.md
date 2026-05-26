---
name: code-investigate
description: Investigate code before changing it — narrow mode for specific queries, broad mode for neighbourhood awareness. Routes to ast-grep, tree-sitter, ripgrep, and rtk-wrapped variants by default.
when_to_use: "When investigating code structure, finding symbols, locating definitions, or understanding a codebase neighbourhood before making changes."
allowed-tools: Bash Read Grep Glob
user-invocable: true
created_by: human
created_at: 2026-05-18
pinned: true
---

# /orb:code-investigate

Investigate before you change. The agent owns the code; this skill makes that ownership cheap to exercise.

Two modes:

- **Narrow mode** — a specific query: *where is X implemented*, *what calls Y*, *how many Z exist*. The skill picks the right tool per query shape and returns the answer.
- **Broad mode** — no specific query, or a directory/module scope. The skill gathers a synthesised neighbourhood picture: directory shape, hot files, where complexity clusters, what sits adjacent to the area being changed.

## Usage

```
/orb:code-investigate [query or scope]
```

- A specific question → narrow mode.
- A directory, module, or no argument → broad mode.

## Mark the invocation

Before returning results, record the investigation so the PreToolUse nudge knows this scope or file has been covered for the session:

```bash
plugins/orb/scripts/code-investigate-mark.sh <kind> <path>
```

`<kind>` is `file` for a narrow-mode file target, `scope` for a broad-mode directory or module. The script handles the atomic write and session-id header per the marker contract.

For narrow queries that resolve to a specific file, mark that file. For broad-mode scopes, mark the directory (the hook resolves prefix-matched files as investigated). For narrow queries that don't resolve to a single file (e.g. cross-module structural questions), mark the broadest enclosing scope.

## Tool taxonomy

The agent reaches for these by default:

| Tool | Use for |
|------|---------|
| **ast-grep** | Structural patterns regex cannot express — *find unwrap calls in match arms*, *find async functions returning Result*, language-aware AST matching. |
| **tree-sitter** | AST queries when you need a parse-tree, not just a match — *count async functions per file*, *list trait impls*, structural counts. |
| **Text search** (`rg`, `grep -rE`, `rtk grep`) | Use `-l` for file-list only, `-C N` for context lines, `-c` for per-file counts. Some environments hook-route `rg` invocations through a grep proxy — write queries that work in either (POSIX ERE alternation, no PCRE2). For true ripgrep features (PCRE2, `--json`, regex extensions), invoke the binary by absolute path. |
| **rtk-wrapped variants** | Token-frugal wrappers for verbose commands — `rtk ls`, `rtk tree`, `rtk diff`, `rtk find`, `rtk read`, `rtk log`. Reach for these when raw output would burn tokens. |
| **Read / Glob** | When you need a full file or a path pattern that isn't search-shaped. Read is the fallback once you know exactly what to open. |

## Narrow mode

Pick the tool per query shape:

- **"Where is X?"** → `rg -l "X"` for file locations, then `-C 3` for context once you've narrowed. Use ast-grep if X is a structural pattern regex cannot express.
- **"What calls Y?"** → `rg "Y\\("` for the function-call shape, or ast-grep for AST-precise call-site matching (handles method calls, chained invocations, generics).
- **"How many Z?"** → ast-grep or tree-sitter for accurate structural counts. Don't grep-and-eyeball; use the parsing tool and quote the number.
- **"What's the type/shape of W?"** → Read the file; the type lives there.

Return the answer with a citation — file path, line number, the matched stat. *Quote* the number; don't approximate.

## Broad mode

Gather a neighbourhood picture in this order:

1. **Directory shape** — `rtk tree <scope>` or `rg --files <scope> | head` for file enumeration.
2. **Hot files** — `git log --pretty=format: --name-only -- <scope> | sort | uniq -c | sort -rn | head` for change-frequency. Token-cheap; identifies what's actively maintained.
3. **Complexity clusters** — `wc -l <scope>/**/*.<ext> | sort -rn | head` for line-count distribution, or `rg -c "<term>" <scope> | sort -rn -t: -k2 | head` for keyword-density-by-file.
4. **Adjacent surface** — for any file in scope you intend to edit, find its imports/dependents: `rg -l "<module-name>"` or ast-grep for AST-precise dependency edges.

Return a synthesised picture, not a dump. The calling agent wants the *shape* of the area, not every file.

## Discipline

- **Quote stats accurately.** If you say "47 async functions", run the count via ast-grep or tree-sitter — don't approximate from a regex.
- **Default to the token-frugal variant.** rtk-wrapped commands exist for a reason; the verbose form is the exception.
- **Read the cited source.** A grep hit is a candidate, not a conclusion — open the file at the matched line before drawing inferences.
- **Diagnose before bypassing.** When a CLI returns surprising output (wrong binary, unsupported flag, unexpected stderr), the workaround is justified only after you've named the mechanism. Run `type <cmd>`, `command -V <cmd>`, `alias <cmd>`, AND grep `~/.claude/settings.json` for `PreToolUse` Bash hooks. Hooks (e.g. `rtk hook claude`) intercept unqualified invocations and can rewrite them — "shimmed" and "hook-intercepted" feel the same but are different mechanisms; naming the actual one keeps memory writes accurate.
- **Cite the substrate, don't paraphrase.** When applying a project rule (`CLAUDE.md`, `.orbit/METHOD.md`, `.orbit/STYLE.md`, choices, memories), include `<file>:<line>` and the exact quoted text. Adjacent files overlap intentionally (CLAUDE.md is per-project, METHOD.md is the orbit canonical, STYLE.md is the prose canonical — easy to conflate). And: project copies of canonical files can drift behind the plugin. If a citation depends on a rule that might have shifted, run `orbit audit conformance --json` first; a non-empty `plugin_canonical_file_drift` finding means the local copy is stale.
- **Investigate before you change.** This is the discipline the skill exists to make cheap.

## After using this skill

If something non-obvious surfaced — a tool that worked where another failed, a query shape worth reaching for again, or a structural insight worth keeping — write a short memory:

```bash
orbit memory remember <key> "<body>" --label code-investigate
```

Capture (a) the query or scope, (b) the tools reached for, (c) what was non-obvious. Quality-gated reach rather than every-invocation write; the learning loop wants signal, not log volume. The label `code-investigate` is the substrate seam the learning loop pivots on — periodic distillation lifts recurring patterns from these memories back into this skill's prose.
