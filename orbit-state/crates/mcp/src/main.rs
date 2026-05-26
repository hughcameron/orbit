//! orbit-mcp — Model Context Protocol server for orbit-state.
//!
//! Hand-rolled JSON-RPC 2.0 stdio loop. Real MCP-SDK integration is a
//! follow-up — for ac-05 we need the parity contract, not full wire
//! compliance. Methods supported:
//!
//! - `initialize`     → returns an empty capabilities object
//! - `tools/list`     → returns the verb surface
//! - `tools/call`     → translates `{name, arguments}` to a [`VerbRequest`],
//!                      calls [`orbit_state_core::execute`], and wraps the
//!                      [envelope][orbit_state_core::envelope_ok] in MCP's
//!                      `content[].text` shape
//!
//! Architectural contract with the CLI: both surfaces construct a
//! `VerbRequest`, dispatch through the same `execute`, and emit the same
//! envelope helpers. The envelope text inside `tools/call`'s
//! `result.content[0].text` is byte-identical to the CLI's `--json` stdout.
//! That's how the parity test (`tests/parity.rs` in this crate) verifies
//! ac-05.
//!
//! Wire transport: newline-delimited JSON. One request per line, one
//! response per line. Unparseable lines produce a JSON-RPC parse-error
//! response with `id: null`.

use orbit_state_core::layout::OrbitLayout;
use orbit_state_core::{envelope_err, envelope_ok, execute, VerbRequest};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() -> anyhow::Result<()> {
    // ac-21 link preservation — same rationale as the CLI.
    orbit_state_core::link_sanity_check()?;

    let root = std::env::current_dir()?;
    let layout = OrbitLayout::at(&root);

    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let response = handle_line(&layout, &line);
        // Skip notifications (no id). For requests we always emit a response.
        if let Some(resp) = response {
            writeln!(out, "{resp}")?;
            out.flush()?;
        }
    }
    Ok(())
}

/// Handle a single line of input. Returns `None` for notifications (which
/// JSON-RPC says have no response), `Some(value)` for requests.
fn handle_line(layout: &OrbitLayout, line: &str) -> Option<Value> {
    let req: Value = match serde_json::from_str(line) {
        Ok(v) => v,
        Err(e) => {
            return Some(json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32700, "message": format!("parse error: {e}") }
            }));
        }
    };

    let id = req.get("id").cloned();
    let method = req.get("method").and_then(Value::as_str).unwrap_or("");
    let params = req.get("params").cloned().unwrap_or(Value::Null);

    // Notifications (no id) get no response per JSON-RPC 2.0.
    let id = match id {
        Some(v) if !v.is_null() => v,
        _ => return None,
    };

    Some(dispatch(layout, &id, method, &params))
}

fn dispatch(layout: &OrbitLayout, id: &Value, method: &str, params: &Value) -> Value {
    match method {
        "initialize" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "orbit-mcp",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }
        }),
        "tools/list" => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": { "tools": tool_descriptors() }
        }),
        "tools/call" => handle_tool_call(layout, id, params),
        other => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("method not found: {other}") }
        }),
    }
}

/// MCP tool descriptors. Mirrors the [`VerbRequest`] surface — adding a verb
/// means adding a descriptor here.
fn tool_descriptors() -> Vec<Value> {
    vec![
        json!({
            "name": "spec.list",
            "description": "List specs in the .orbit/ folder, sorted by id.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["open", "closed"],
                        "description": "Filter by status."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.show",
            "description": "Read a single spec by id and return its full contents.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": {
                        "type": "string",
                        "description": "Spec id (slug-shaped; no path separators)."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.note",
            "description": "Append a timestamped note to a spec's notes JSONL stream.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "body"],
                "properties": {
                    "id":   { "type": "string", "description": "Spec id." },
                    "body": { "type": "string", "description": "Note body (free text)." },
                    "labels": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "Optional free-text labels."
                    },
                    "timestamp": {
                        "type": "string",
                        "description": "Override substrate timestamp (RFC 3339). Primarily for migration tools."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.create",
            "description": "Create a new spec at .orbit/specs/<id>.yaml.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "goal"],
                "properties": {
                    "id":   { "type": "string" },
                    "goal": { "type": "string" },
                    "cards": { "type": "array", "items": { "type": "string" } },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "acceptance_criteria": { "type": "array" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.update",
            "description": "Update fields on an existing spec. Status changes go via spec.close.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id":   { "type": "string" },
                    "goal": { "type": "string" },
                    "cards": { "type": "array", "items": { "type": "string" } },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "acceptance_criteria": { "type": "array" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.close",
            "description": "Close a spec; transactionally appends to linked cards' specs arrays.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": {
                    "id": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.acs",
            "description": "Return a spec's full acceptance_criteria list. Native port of the orbit-acceptance.sh acs subcommand per spec 2026-05-24-port-acceptance-shim.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": { "id": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.next-ac",
            "description": "Return the first unchecked AC that is not blocked by an unchecked gate (gate-axis traversal). Native port of the orbit-acceptance.sh next-ac subcommand.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": { "id": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.blocking-gate",
            "description": "Return the first unchecked gate AC, if any. Native port of the orbit-acceptance.sh blocking-gate subcommand.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": { "id": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.has-unchecked",
            "description": "Return true if any AC is unchecked (raw-axis traversal — distinct from spec.close's taxonomy-axis pre-flight). Native port of the orbit-acceptance.sh has-unchecked subcommand.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": { "id": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.check",
            "description": "Flip an AC's checked flag from false to true. Error::not_found for unknown AC, Error::conflict for already-checked AC. Native port of the orbit-acceptance.sh check subcommand.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "ac_id"],
                "properties": {
                    "id":    { "type": "string" },
                    "ac_id": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.uncheck",
            "description": "Flip an AC's checked flag from true to false. Symmetric to spec.check.",
            "inputSchema": {
                "type": "object",
                "required": ["id", "ac_id"],
                "properties": {
                    "id":    { "type": "string" },
                    "ac_id": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "spec.promote",
            "description": "Turn a card into a spec — derives the spec id from today's date + the card's slug, copies the goal, materialises one AC per scenario (preserving gate, seeding checked: false). Native port of plugins/orb/scripts/promote.sh per spec 2026-05-25-port-promote-sh.",
            "inputSchema": {
                "type": "object",
                "required": ["card_path"],
                "properties": {
                    "card_path": { "type": "string", "description": "Path to the card file (absolute or relative to the layout root)." },
                    "dry_run":   { "type": "boolean", "description": "When true, compute the planned spec but write nothing." },
                    "today":     { "type": "string", "description": "Override today's date (YYYY-MM-DD). Test-only; production callers omit this." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "setup.files",
            "description": "Run /orb:setup §6 — atomic CLAUDE.md legacy-block migration, byte-compare-and-copy of canonical METHOD.md + STYLE.md into <project-root>/.orbit/, idempotent @-import appends. Interactive prompts live at the CLI layer; MCP callers always pass typed Action enums. Native port of plugins/orb/scripts/setup-method.sh per spec 2026-05-25-port-setup-method-sh.",
            "inputSchema": {
                "type": "object",
                "required": ["project_root", "legacy_action", "method_drift_action", "style_drift_action"],
                "properties": {
                    "project_root":        { "type": "string", "description": "Project root containing CLAUDE.md and .orbit/." },
                    "legacy_action":       { "type": "string", "enum": ["migrate", "refuse"], "description": "Action when CLAUDE.md contains legacy workflow blocks." },
                    "method_drift_action": { "type": "string", "enum": ["overwrite", "keep"], "description": "Action when .orbit/METHOD.md exists and drifts from canonical." },
                    "style_drift_action":  { "type": "string", "enum": ["overwrite", "keep"], "description": "Action when .orbit/STYLE.md exists and drifts." },
                    "canonical_method_path": { "type": "string", "description": "Override canonical METHOD.md source." },
                    "canonical_style_path":  { "type": "string", "description": "Override canonical STYLE.md source." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.open",
            "description": "Open a new task under a spec.",
            "inputSchema": {
                "type": "object",
                "required": ["spec_id", "body"],
                "properties": {
                    "spec_id": { "type": "string" },
                    "body":    { "type": "string" },
                    "labels":  { "type": "array", "items": { "type": "string" } },
                    "task_id": { "type": "string" },
                    "timestamp": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.list",
            "description": "List tasks (current state per task_id).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "spec_id": { "type": "string" },
                    "state": { "type": "string", "enum": ["open", "claim", "update", "done"] }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.show",
            "description": "Show one task with its full event history.",
            "inputSchema": {
                "type": "object",
                "required": ["spec_id", "task_id"],
                "properties": {
                    "spec_id": { "type": "string" },
                    "task_id": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.ready",
            "description": "List claimable (open, no claim) tasks.",
            "inputSchema": {
                "type": "object",
                "properties": { "spec_id": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.claim",
            "description": "Claim an open task.",
            "inputSchema": {
                "type": "object",
                "required": ["spec_id", "task_id"],
                "properties": {
                    "spec_id": { "type": "string" },
                    "task_id": { "type": "string" },
                    "body": { "type": "string" },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "timestamp": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.update",
            "description": "Append an update note to a task.",
            "inputSchema": {
                "type": "object",
                "required": ["spec_id", "task_id", "body"],
                "properties": {
                    "spec_id": { "type": "string" },
                    "task_id": { "type": "string" },
                    "body": { "type": "string" },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "timestamp": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "task.done",
            "description": "Mark a task done.",
            "inputSchema": {
                "type": "object",
                "required": ["spec_id", "task_id"],
                "properties": {
                    "spec_id": { "type": "string" },
                    "task_id": { "type": "string" },
                    "body": { "type": "string" },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "timestamp": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "memory.remember",
            "description": "Upsert a memory entry. Persists across sessions/machines via git.",
            "inputSchema": {
                "type": "object",
                "required": ["key", "body"],
                "properties": {
                    "key":       { "type": "string" },
                    "body":      { "type": "string" },
                    "labels":    { "type": "array", "items": { "type": "string" } },
                    "timestamp": { "type": "string" },
                    "no_nudge":  { "type": "boolean" },
                    "no_warn":   { "type": "boolean" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "memory.list",
            "description": "List all memories.",
            "inputSchema": { "type": "object", "additionalProperties": false }
        }),
        json!({
            "name": "memory.search",
            "description": "Substring (case-insensitive) search over body + labels.",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": { "query": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "memory.match",
            "description": "Surface memories relevant to a decision-moment topic. Returns ranked matches (token + label overlap); distinct from the operator-keyword memory.search.",
            "inputSchema": {
                "type": "object",
                "required": ["topic"],
                "properties": {
                    "topic":  { "type": "string" },
                    "labels": { "type": "array", "items": { "type": "string" } },
                    "limit":  { "type": "integer", "minimum": 1 }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "card.show",
            "description": "Show a card by slug.",
            "inputSchema": {
                "type": "object",
                "required": ["slug"],
                "properties": { "slug": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "card.list",
            "description": "List cards. Optional filter by maturity.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "maturity": { "type": "string", "enum": ["planned", "emerging", "established"] }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "card.search",
            "description": "Substring (case-insensitive) search over slug + feature + goal.",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": { "query": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "card.tree",
            "description": "Render the local subgraph from a card (outgoing + incoming `relations:` edges). Default depth is 2; cycle-safe.",
            "inputSchema": {
                "type": "object",
                "required": ["slug"],
                "properties": {
                    "slug": { "type": "string" },
                    "depth": { "type": "integer", "minimum": 0 }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "card.specs",
            "description": "List specs advancing a card, with bidirectional link health. Surfaces drift where card.specs[] and spec.cards[] disagree.",
            "inputSchema": {
                "type": "object",
                "required": ["slug"],
                "properties": { "slug": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "overview",
            "description": "Single-screen project synthesis — open specs, cards-by-maturity counts, recent memories, most-connected card, orphan cards. Bounded output regardless of project age.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memory_cap": { "type": "integer", "minimum": 0 }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "graph",
            "description": "Render the cards-specs graph as mermaid (default) or graphviz — pasteable into markdown or a renderer. Optional --card scopes to one card's neighbourhood at the given depth.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "card": { "type": "string" },
                    "depth": { "type": "integer", "minimum": 0 },
                    "format": { "type": "string", "enum": ["mermaid", "graphviz"] }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "audit.drift",
            "description": "Permissive YAML scan that surfaces top-level fields absent from the canonical schema. Read-only; no rewrites.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": "audit.topology",
            "description": "Walk .orbit/topology/<subsystem>.yaml entries (per choice 0025) and report drift across stale_pointer / missing_entry / invalid_field / parse_failed. Exits 0 in all states; consumers discriminate via the envelope's configured flag and topology_drift array.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": "audit.conformance",
            "description": "Workflow-conformance audit (per spec 2026-05-19-workflow-conformance). Aggregates audit.drift + audit.topology results under `aggregated.{drift,topology}` and surfaces new finding families: card-state (planned + empty specs), memo staleness (>7d), plugin-canonical-file drift, plugin-version pin state. Each finding carries `remediation.verb` — the agent acts without translation. Read-only; returns zero findings on a clean repo.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": "topology.setup",
            "description": "Scaffold .orbit/topology/ with self-describing seed entries (one per .orbit/ entity) and opportunistically strip legacy docs.topology from .orbit/config.yaml. Idempotent on re-runs. Per spec 2026-05-18-topology-substrate-migration ac-05.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "answer_wire": {
                        "type": "string",
                        "description": "Script the wire-or-decline prompt for non-interactive runs ('y' to proceed, 'n' to decline)."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "substrate.recall",
            "description": "Substrate-wide ranked search across memory, card, choice, spec, and memo artefacts. Returns tuples (id, type, score, snippet, path) sorted by score desc, type rank (memory > choice > card > spec > memo), then id asc. Per spec 2026-05-25-recall-verb-and-skill-step and card 0044.",
            "inputSchema": {
                "type": "object",
                "required": ["topic"],
                "properties": {
                    "topic": { "type": "string", "description": "Free-text recall topic." },
                    "types": {
                        "type": "array",
                        "items": {
                            "type": "string",
                            "enum": ["memory", "card", "choice", "spec", "memo"]
                        },
                        "description": "Optional fan-out filter. Empty (default) = all five."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "choice.show",
            "description": "Show a choice by id.",
            "inputSchema": {
                "type": "object",
                "required": ["id"],
                "properties": { "id": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "choice.list",
            "description": "List choices. Optional filter by status.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["proposed", "accepted", "rejected", "deprecated", "superseded"]
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "choice.search",
            "description": "Substring (case-insensitive) search over title + body.",
            "inputSchema": {
                "type": "object",
                "required": ["query"],
                "properties": { "query": { "type": "string" } },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "session.prime",
            "description": "Agent session priming context — bounded output (open specs + up to K memories).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "memory_cap": {
                        "type": "integer",
                        "minimum": 0,
                        "description": "Override the default K=10 memory cap."
                    }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "session.start",
            "description": "Generate a session id (UUIDv4) and write it to .orbit/.session-id. Pass `id` to use a verbatim value (test/replay).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "description": "Use this id verbatim instead of generating a UUIDv4." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "session.distill",
            "description": "Write or update .orbit/sessions/<id>.yaml with the agent's end-of-session reflection. Idempotent on session_id.",
            "inputSchema": {
                "type": "object",
                "required": ["distillate"],
                "properties": {
                    "session_id": { "type": "string", "description": "Override the session id source (env var > .session-id file)." },
                    "distillate": { "type": "string", "description": "Free-text end-of-session reflection." },
                    "card_id": { "type": "string", "description": "Optional card slug scoping this session. Falls back to .orbit/.session-card when omitted." },
                    "labels": { "type": "array", "items": { "type": "string" } }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "session.set-card",
            "description": "Validate a card id and write the canonical slug to .orbit/.session-card. The next session.distill (typically the Stop hook) scopes the session to that card.",
            "inputSchema": {
                "type": "object",
                "required": ["card_id"],
                "properties": {
                    "card_id": { "type": "string", "description": "Card id (full slug, padded NNNN, or bare unpadded number)." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "session.handover",
            "description": "Return the most-recent matching Session (or handover: null when none). Filter by --card and/or --since (RFC 3339 lower bound).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "card_id": { "type": "string", "description": "Card id (full slug, padded NNNN, or bare unpadded number)." },
                    "since": { "type": "string", "description": "RFC 3339 cutoff — only consider sessions with started_at >= since." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "skill.record-invocation",
            "description": "Append one row to .orbit/skills/<skill_id>.invocations.jsonl recording how a skill ran.",
            "inputSchema": {
                "type": "object",
                "required": ["skill_id", "outcome"],
                "properties": {
                    "skill_id": { "type": "string" },
                    "outcome": { "type": "string", "enum": ["worked", "partial", "didnt-apply", "incorrect"] },
                    "correction": { "type": "string", "description": "Free-text record of what was corrected (optional)." },
                    "session_id": { "type": "string", "description": "Override the session id source." },
                    "timestamp": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "skill.recurrence",
            "description": "Read the per-skill invocation stream and bucket rows by outcome. Returns the empty shape (all zeros) when the file is absent.",
            "inputSchema": {
                "type": "object",
                "required": ["skill_id"],
                "properties": {
                    "skill_id": { "type": "string" },
                    "since": { "type": "string", "description": "RFC 3339 cutoff — only count rows with timestamp >= since." }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "routine.chains",
            "description": "Reconstruct per-session chains from .orbit/skills/*.invocations.jsonl rows (per spec 2026-05-22-routine-proposals ac-01). Purely additive aggregator — no SkillInvocation schema change.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": "routine.detect",
            "description": "Surface recurring sequential chains at or above the ≥2-occurrence threshold (ac-02). v1 is sequential-only — DAG-shaped patterns are not returned (ac-05).",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
        json!({
            "name": "routine.author",
            "description": "Write a routine SKILL.md at .claude/skills/<name>/SKILL.md with validated front-matter (created_by, created_at, pinned, last_verified, chain_id, chain). Idempotent — if a routine with the same chain_id already exists under .claude/skills/ or its .archive/ subtree, returns the existing path and skips the write (ac-09).",
            "inputSchema": {
                "type": "object",
                "required": ["chain"],
                "properties": {
                    "chain": { "type": "array", "items": { "type": "string" }, "minItems": 2 },
                    "name": { "type": "string" },
                    "description": { "type": "string" },
                    "body": { "type": "string" },
                    "timestamp": { "type": "string" },
                    "occurrences": { "type": "integer", "minimum": 0 }
                },
                "additionalProperties": false
            }
        }),
        json!({
            "name": "routine.verify",
            "description": "Re-validate every /orb:<verb> reference in the routine's SKILL.md body and on pass write the run timestamp to last_verified (ac-06). The verb is the only writer of last_verified; audit.conformance is read-only on routines.",
            "inputSchema": {
                "type": "object",
                "required": ["path"],
                "properties": {
                    "path": { "type": "string" },
                    "timestamp": { "type": "string" }
                },
                "additionalProperties": false
            }
        }),
    ]
}

fn handle_tool_call(layout: &OrbitLayout, id: &Value, params: &Value) -> Value {
    let name = match params.get("name").and_then(Value::as_str) {
        Some(n) => n,
        None => {
            return json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": { "code": -32602, "message": "tools/call: missing 'name'" }
            });
        }
    };
    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| Value::Object(Default::default()));

    // Reconstruct VerbRequest from MCP's {name, arguments} shape. The verbs
    // module's tagged-enum representation is `{"verb": <name>, "args": ...}`
    // — translate by reshaping into that.
    let request_value = json!({ "verb": name, "args": arguments });
    let request: VerbRequest = match serde_json::from_value(request_value) {
        Ok(r) => r,
        Err(e) => {
            // Invalid args surface as a tool-level error envelope inside a
            // successful JSON-RPC response — that's the MCP convention for
            // tool failures (clients should look at `isError`, not the
            // JSON-RPC error channel).
            let err = orbit_state_core::Error::malformed(
                name,
                format!("invalid arguments: {e}"),
            );
            return tool_error_response(id, &err);
        }
    };

    match execute(layout, &request) {
        Ok(response) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "content": [
                    { "type": "text", "text": envelope_ok(&response).to_string() }
                ]
            }
        }),
        Err(err) => tool_error_response(id, &err),
    }
}

fn tool_error_response(id: &Value, err: &orbit_state_core::Error) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": {
            "content": [
                { "type": "text", "text": envelope_err(err).to_string() }
            ],
            "isError": true,
        }
    })
}
