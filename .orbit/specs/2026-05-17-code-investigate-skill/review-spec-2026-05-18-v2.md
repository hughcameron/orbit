# Spec Review

**Date:** 2026-05-18
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-17-code-investigate-skill
**Verdict:** REQUEST_CHANGES

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 1 |
| 2 — Assumption & failure | content signals (PreToolUse hook fires in every session in this and consumer repos; cross-skill prose edits in `/orb:implement`, `/orb:researcher`, `/orb:review-pr`; substrate file under `.orbit/`; gitignore amendment) | 2 |
| 3 — Adversarial | not triggered | — |

## Cycle-1 changes acknowledged

The cycle-1 response (commit `a38d040`) tightened the spec on four of the five v1 findings:

- ac-02 now pins the hook to `plugins/orb/hooks/hooks.json` and adds a graceful-degradation clause (silently skip when `.orbit/` is absent; warn when marker is absent within an orbit repo) and a path filter (`.orbit/`, `.claude/`, `*.json/*.yaml/*.yml/*.toml/*.lock`). The v1 MEDIUMs on degradation and warning-surface are resolved.
- ac-03 now pins the marker lifecycle to session-id comparison (no TTL, no Stop-hook dependency) and removes the stale hedge. v1 MEDIUM resolved.
- ac-07 softens the audit date to "ship-date + 4 weeks" and segments the warning-fire count by file kind. v1 LOW resolved; v1 MEDIUM on audit interpretability resolved.

Remaining concerns sit on the load-bearing seam the cycle-1 response committed to but did not verify, plus one fresh failure-mode the new path filter introduces.

## Findings

### [HIGH] ac-02 commits to a plugin-loader behaviour that prior evidence says does not exist
**Category:** missing-requirement
**Pass:** 1 (gap) → 2 (failure-mode amplifier)
**Description:** ac-02 now states as fact that *"the Claude Code plugin loader reads hook entries from this path on plugin load, so consumer repos get the hook the moment they install orb."* This is the spec's load-bearing claim — the hook only delivers the discipline to consumer repos if the plugin loader actually picks it up. The recorded memory `missing-hooks-json-smoke-test` (2026-05-09) is explicit:

> Verified 2026-05-09 at orbit 0.4.5 release: /reload-plugins after the post-#22 release reports '0 hooks · 0 plugin MCP servers' with no warning about the missing plugins/orb/hooks/hooks.json. … Claude Code's loader does not require hooks.json to exist.

That observation directly contradicts ac-02's premise. Either (a) the Claude Code plugin loader has shipped a behaviour change between 0.4.5 (2026-05-09) and now (2026-05-18) that delivers `plugins/<plugin>/hooks/hooks.json` on plugin load, in which case the spec should cite the verification and supersede the prior memory; or (b) the loader still does not deliver plugin-shipped hooks, in which case ac-02's verification clause ("plugins/orb/hooks/hooks.json exists and parses against the Claude Code plugin hook schema") will pass while the hook silently never fires in any consumer repo. That second failure mode is precisely what the prior memory was filed to prevent.

**Evidence:**
- ac-02 description: *"the Claude Code plugin loader reads hook entries from this path on plugin load, so consumer repos get the hook the moment they install orb"*
- Memory `missing-hooks-json-smoke-test` (dated 2026-05-09)
- `plugins/orb/.claude-plugin/plugin.json` has no `hooks` field today
- `.claude/settings.json` is the only surface where this repo's hooks actually fire (SessionStart, PreCompact, Stop) — empirical evidence that the working hook-registration surface for orbit today is settings.json, not the plugin manifest
- ac-02 verification has no smoke that confirms the hook *fires* end-to-end on a fresh plugin install — only that the file exists and parses

**Recommendation:** Resolve before implementation, one of three ways.
1. **Cite re-verification.** If the loader behaviour has been re-tested since 2026-05-09 (e.g. on a recent Claude Code build) and now does deliver `hooks/hooks.json`, add a one-line note to the spec citing the date, version, and observation, and annul the prior memory (`orbit memory remember missing-hooks-json-smoke-test "..." --label superseded`).
2. **Strengthen the verification clause.** Add to ac-02 verification: *"End-to-end smoke from a clean plugin install — install orb in a scratch repo with no `.claude/settings.json` orb entries, restart Claude Code, run `/hooks` or `/reload-plugins` to confirm the loader reports the orb PreToolUse hook as registered (count ≥ 1, not 0)."* This converts the load-bearing assertion from belief to test.
3. **Dual-register as fallback.** If plugin-loader delivery is uncertain, also register the hook in `.claude/settings.json` (matching the existing PreCompact/SessionStart/Stop pattern). Loses the "ship to every consumer repo automatically" property but guarantees the hook fires in this repo. Document the dual-registration explicitly in ac-02.

Option 2 is the minimum change. Option 1 is the cleanest if the loader has indeed changed. Option 3 is the safe fallback. The spec needs to pick one — the current text reads as option 1 with no supporting evidence.

### [MEDIUM] ac-02 path filter excludes `*.toml` — Cargo manifests fall outside the warning surface in Rust consumer repos
**Category:** failure-mode
**Pass:** 2
**Description:** The path filter added in the cycle-1 response excludes `*.json/*.yaml/*.yml/*.toml/*.lock`. The intent reads correctly for top-level config noise (e.g. agents editing `package.json`, `settings.json`, lockfiles) — but `Cargo.toml` is the canonical dependency-management surface in Rust repos and is exactly the kind of file where "investigate before you change" applies (a new dep, a feature-flag toggle, a workspace-member edit). orbit's own orbit-state binary is a Rust crate; its primary `Cargo.toml` would be excluded by this filter. Same for `pyproject.toml` in Python repos. Same for `tsconfig.json` (extension already excluded).

The pattern's failure mode is silent: the agent edits `Cargo.toml` without prior `/orb:code-investigate`, no warning fires, and the 4-week audit can't even count this as a miss because the filter skipped the path before the marker check.

**Evidence:** ac-02 description: *"Path filter — hook skips edits to paths under .orbit/ or .claude/ and to config files (*.json, *.yaml, *.yml, *.toml, *.lock)"*. orbit-state lives in `orbit-state/Cargo.toml` (root-level), not under `.orbit/` or `.claude/`.
**Recommendation:** Tighten the filter to one of:
- Skip only paths under `.orbit/`, `.claude/`, and `*.lock` files (the strict "machine-managed" set). Drop the extension list otherwise — agents editing settings/config files arguably *should* be nudged.
- Or skip the extension list **only when at the repo root** (e.g. `^(package|tsconfig|settings|deno)\.json$`, `^Cargo\.lock$`) — but explicitly do not skip `Cargo.toml` / `pyproject.toml` / nested `.json/.yaml` files where the agent is editing source-adjacent config.

One-line tightening in ac-02. The audit interpretation in ac-07 stays valid either way; if anything, a narrower filter gives the audit more signal.

---

## Honest Assessment

The cycle-1 response did the work it needed to do on four of the five v1 findings. ac-03's marker lifecycle is now coherent; ac-07's date hedge is gone; ac-02's degradation and warning-surface clauses tighten what was previously hand-waved. The interview record (initial + design pass 2) remains the strongest part of this spec.

The unresolved item is the same load-bearing seam that drove the v1 HIGH — and it has narrowed rather than disappeared. v1 said "registration target unspecified". The cycle-1 response specified a target but asserted its behaviour as fact without addressing the prior memory that says the target doesn't work. The risk is concrete: ac-02 verification passes (file exists, parses), the spec closes, the hook ships to consumer repos, and never fires once. The 4-week audit (ac-07) returns warning-fire count = 0 and the spec's authors will need to discover that the hook never registered at all rather than that agents are reaching for `/orb:code-investigate` perfectly.

Picking option 2 (strengthen the verification clause to end-to-end load-time confirmation) is the minimum viable response and turns this from a faith-based claim into a tested one. Option 3 (dual-register in `.claude/settings.json` as fallback) is the conservative path and matches how every other orbit hook ships today.

The `*.toml` filter finding is fresh — introduced by the cycle-1 response itself — and is a one-line tightening. Worth catching now rather than after the first Rust consumer repo's `Cargo.toml` edit slips past unannouced.

Verdict is REQUEST_CHANGES rather than APPROVE because the HIGH is a substrate-shape decision the spec should commit on with evidence, not a tactical detail. Once ac-02 picks one of the three resolution paths and the toml filter is tightened, the spec is ready.
