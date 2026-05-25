# Tabletop Note: Port orbit-acceptance.sh to native `orbit spec` verbs

**Date:** 2026-05-24
**Cards:** .orbit/cards/0003-implement.yaml, .orbit/cards/0005-drive.yaml, .orbit/cards/0020-orbit-state.yaml
**Mode:** closed
**Choice:** .orbit/choices/0020-shell-scripts-to-rust-verbs.yaml — substrate-shaped shell scripts under `plugins/orb/scripts/` migrate into orbit Rust verbs

---

## What good looks like

Drive's implement loop, the pre-close AC pre-flight check, and every other consumer of acceptance-criterion state calls native `orbit` verbs — not a 182-line python3-in-bash shim. The five operations the shim provides today (`acs`, `next-ac`, `blocking-gate`, `has-unchecked`, `check`) land as native verbs on `orbit spec` with byte-equal CLI/MCP parity to existing verbs. The shim becomes a thin compat wrapper, survives until `rg orbit-acceptance.sh` against every consumer SKILL.md returns zero, then the shim and its dedicated test (`test-gate-ac-verification.sh`) delete in the same commit as the last call-site rewrite. The implement loop stops piping JSON through `python3 -c` on every AC tick.

## Pinned approach

- Choice 0020 names the direction: substrate-shaped scripts under `plugins/orb/scripts/` port to `orbit-state` Rust verbs as the default; per-script migration sequenced opportunistically, each in its own spec citing the choice.
- Rust-verb-extension on `orbit-state` is the established pattern with multiple shipped precedents: `2026-05-12-tree-views`, `2026-05-13-spec-close-ac-preflight`, `2026-05-16-ac-taxonomy`, `2026-05-18-topology-substrate-migration`, `2026-05-21-richer-reconcile-rules`. The verb-addition path is well-worn.
- Decommission discipline per choice 0020's "Test-vs-prod parity" clause and card 0030's audit pattern: the shell shim stays in place as a wrapper over the new verbs until grep on every consumer SKILL.md returns clean; the shim and its tests then delete in one commit with the last call-site rewrite. No partial decommission, no orphaned wrapper.

## Deferred items

- `promote.sh` port — second opportunistic spec per choice 0020's enumeration (6 SKILL.md call sites; denser logic; bundles the `relations:respects → choice 0020` edge writes on cards 0005 and 0006).
- `setup-method.sh` port — third opportunistic spec (1 SKILL.md call site but introduces a new `orbit setup` verb family with interactive drift/legacy prompts; bundles the edge write on card 0017).
- `code-investigate-mark.sh` discrimination — file a `/orb:choice` first; the script post-dates choice 0020 and sits on the substrate-vs-tooling discrimination boundary (session-scoped marker file, not a canonical artefact).
- Migration of `plugins/orb/scripts/tests/test-gate-ac-verification.sh` to the Rust suite — moves with the shim's deletion per choice 0020's test-vs-prod parity rule; no separate test-migration spec.
- Lifting the per-fork python3 dependency in implement's heartbeat / progress loops — out of scope here; touch when the consumer skill prose rewrites land.

## Implementation notes

- **Verb-surface fork — decide in the spec.** Two coherent shapes:
  - **One envelope verb** — `orbit spec acs <id>` returns `{acceptance_criteria: [...], next_unblocked: <ac-id|null>, blocking_gate: <ac-id|null>, has_unchecked: bool}` in a single envelope; skill prose reads the field it needs via inline jq. Fewer MCP tool surfaces; richer single call; smaller diff to `verbs.rs`.
  - **Five thin sub-verbs** — `orbit spec acs / next-ac / blocking-gate / has-unchecked / check`, mirroring the shim's surface 1:1. Byte-for-byte SKILL.md migration (`s/plugins\/orb\/scripts\/orbit-acceptance\.sh/orbit spec/g` works for most call sites). Larger MCP surface; smaller per-call payloads; closer to existing verb-per-action pattern on `orbit spec`.

  Pre-recommendation: **five thin sub-verbs**, on the grounds that the existing `orbit spec` verb family already follows verb-per-action (`show`, `resolve`, `note`, `create`, `update`, `close`, `migrate-layout`) and the byte-for-byte SKILL.md migration is a load-bearing decommission lever. Confirm in the spec.
- **`check` is already covered** by `orbit spec update --ac-check <ac-id>`; the migration there is just rewriting the wrapping bash error/idempotency layer in Rust (verb returns `Error::not_found` / `Error::conflict` instead of shell exit codes). Reuse the existing verb body if possible.
- **Source of truth for verb additions:** `orbit-state/crates/core/src/verbs.rs` — extend the `spec_*` family; the AC-traversal helpers (`next-ac`, `blocking-gate`, `has-unchecked`) become pure functions over the `acceptance_criteria` field, called from both the new verbs and any internal consumer (e.g. `spec_close`'s pre-flight already iterates the same field — opportunity to share the helper).
- **Decommission verification:** the final commit's AC must cite `rg --no-heading 'orbit-acceptance\.sh' plugins/orb/skills/ | wc -l` returning `0`. The shim and `test-gate-ac-verification.sh` delete in the same commit.
- **Recommended `ac_type` distribution** for the spec:
  - `code` — verb implementations, byte-equal CLI/MCP parity tests, envelope-shape tests, internal-helper sharing with `spec_close`'s pre-flight.
  - `code` — SKILL.md call-site rewrites verified by the `rg ... | wc -l == 0` check.
  - `doc` — the shim + test deletion commit (a structural deletion verified by file absence, not test).
  - No `ops` or `observation` ACs expected — the port is internally verifiable end-to-end.
