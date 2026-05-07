//! Verb dispatch surface — single entry point shared by CLI and MCP.
//!
//! Per ac-05: "MCP server and CLI both call same Rust core — state-mutation
//! parity (canonical files + state.db byte-identical), error format
//! `<verb>: <category>: <sentence>`."
//!
//! This module defines:
//! - [`VerbRequest`]   — typed input taxonomy (one variant per verb).
//! - [`VerbResponse`]  — typed output taxonomy (one variant per verb).
//! - [`execute`]       — the single dispatch fn both surfaces call.
//! - [`envelope_ok`] / [`envelope_err`] — wire envelope helpers.
//!
//! Adding a verb is a closed-form change: extend the two enums with matching
//! variants and add a private impl fn dispatched from [`execute`]. Both
//! surfaces (CLI argv parser, MCP JSON-RPC handler) construct `VerbRequest`
//! independently, then call [`execute`] — that's where the parity contract
//! lives. The wire envelope is shared so byte-equal payloads fall out for
//! free as long as both surfaces serialise the same `VerbResponse` with the
//! same helper.
//!
//! v0.1 surface: `spec.list` only. Subsequent ACs (ac-06..11) add the rest.

use crate::canonical::parse_yaml;
use crate::error::{Error, Result};
use crate::layout::OrbitLayout;
use crate::schema::{Spec, SpecStatus};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

// ============================================================================
// Request / Response taxonomy
// ============================================================================

/// Typed verb request. Tagged on the wire as `{"verb": "<name>", "args": {...}}`
/// so the MCP `tools/call` translation is trivial:
///
/// ```text
/// MCP {name: "spec.list", arguments: {...}} → {"verb": "spec.list", "args": {...}} → VerbRequest
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "verb", content = "args")]
pub enum VerbRequest {
    #[serde(rename = "spec.list")]
    SpecList(SpecListArgs),
}

/// Args for `spec.list`. Optional `status` filter; further filters land later.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecListArgs {
    /// Restrict to specs in this status. Must be `"open"` or `"closed"` if
    /// provided. Empty string and other values are rejected as malformed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Typed verb response. One variant per verb, mirroring [`VerbRequest`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "verb", content = "result")]
pub enum VerbResponse {
    #[serde(rename = "spec.list")]
    SpecList(SpecListResult),
}

/// Result for `spec.list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecListResult {
    pub specs: Vec<SpecSummary>,
}

/// Projection of a spec for list views — id, goal, status, plus the cards it
/// advances and any labels. Excludes ACs and other heavy fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecSummary {
    pub id: String,
    pub goal: String,
    pub status: String,
    #[serde(default)]
    pub cards: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

// ============================================================================
// Dispatch
// ============================================================================

/// Dispatch a verb against the layout. The single entry point both CLI and
/// MCP call — the architectural guarantee from ac-05 lives here.
pub fn execute(layout: &OrbitLayout, request: &VerbRequest) -> Result<VerbResponse> {
    match request {
        VerbRequest::SpecList(args) => spec_list(layout, args).map(VerbResponse::SpecList),
    }
}

// ============================================================================
// Verb implementations
// ============================================================================

/// `spec.list` — enumerate spec files under `.orbit/specs/`, sorted by id.
///
/// Reads files directly (not the index). Reading from files is correct and
/// deterministic; once the index proves out for write paths, read verbs can
/// switch to index-backed for performance. ac-05 does not require index reads.
fn spec_list(layout: &OrbitLayout, args: &SpecListArgs) -> Result<SpecListResult> {
    const VERB: &str = "spec.list";

    if let Some(s) = args.status.as_deref() {
        if !matches!(s, "open" | "closed") {
            return Err(Error::malformed(
                VERB,
                format!("status must be 'open' or 'closed', got '{s}'"),
            ));
        }
    }

    let files = layout
        .list_spec_files()
        .map_err(|e| Error::unavailable(VERB, format!("list specs dir: {e}")))?;

    let mut specs = Vec::with_capacity(files.len());
    for path in files {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        let spec: Spec = parse_yaml(&text).map_err(|mut e| {
            // The canonical layer tags errors with verb="canonical"; re-tag to
            // the calling verb so the on-wire error format is correct.
            e.verb = VERB.into();
            e
        })?;
        let status = match spec.status {
            SpecStatus::Open => "open",
            SpecStatus::Closed => "closed",
        };
        if let Some(filter) = args.status.as_deref() {
            if status != filter {
                continue;
            }
        }
        specs.push(SpecSummary {
            id: spec.id,
            goal: spec.goal,
            status: status.into(),
            cards: spec.cards,
            labels: spec.labels,
        });
    }

    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(SpecListResult { specs })
}

// ============================================================================
// Wire envelope
// ============================================================================
//
// Both CLI (`--json` mode) and MCP (`tools/call` response payload) emit the
// same envelope shape so byte-equal output falls out for free:
//
//   ok  : {"data":<verb-response>,"ok":true}
//   err : {"error":{"category":"<cat>","message":"<msg>","verb":"<verb>"},"ok":false}
//
// serde_json sorts object keys alphabetically by default, so the exact byte
// layout is deterministic across both surfaces. Inner struct fields preserve
// declaration order via the Serialize derive.

/// Build the OK envelope as a JSON [`Value`]. Callers stringify via
/// [`serde_json::to_string`] when they want bytes.
pub fn envelope_ok<T: Serialize>(data: &T) -> Value {
    json!({ "ok": true, "data": data })
}

/// Build the error envelope as a JSON [`Value`].
pub fn envelope_err(err: &Error) -> Value {
    json!({
        "ok": false,
        "error": {
            "verb": err.verb,
            "category": err.category.as_str(),
            "message": err.message,
        }
    })
}

/// Convenience: stringify the OK envelope. Returns the canonical wire bytes
/// as a UTF-8 string. Infallible for any `T: Serialize` whose serialise is
/// itself infallible (the envelope wrapper introduces no new failure modes).
pub fn envelope_ok_string<T: Serialize>(data: &T) -> Result<String> {
    serde_json::to_string(&envelope_ok(data)).map_err(|e| {
        Error::malformed("envelope", format!("serialise ok envelope: {e}")).with_source(e)
    })
}

/// Convenience: stringify the error envelope. Cannot fail in practice —
/// errors are simple owned strings + an enum.
pub fn envelope_err_string(err: &Error) -> String {
    // unwrap-justified: envelope_err produces only owned strings + a fixed
    // shape; serde_json::to_string on a Value cannot fail for these inputs.
    serde_json::to_string(&envelope_err(err)).expect("error envelope serialisation is infallible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::serialise_yaml;
    use crate::schema::Spec;
    use tempfile::tempdir;

    fn write_spec(layout: &OrbitLayout, id: &str, goal: &str, status: SpecStatus) {
        let spec = Spec {
            id: id.into(),
            goal: goal.into(),
            cards: vec![],
            status,
            labels: vec![],
            acceptance_criteria: vec![],
        };
        std::fs::write(layout.spec_file(id), serialise_yaml(&spec).unwrap()).unwrap();
    }

    #[test]
    fn spec_list_returns_empty_when_no_specs() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let resp = execute(&layout, &VerbRequest::SpecList(SpecListArgs::default())).unwrap();
        match resp {
            VerbResponse::SpecList(r) => assert!(r.specs.is_empty()),
        }
    }

    #[test]
    fn spec_list_returns_specs_sorted_by_id() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0002", "second", SpecStatus::Open);
        write_spec(&layout, "0001", "first", SpecStatus::Open);

        let resp = execute(&layout, &VerbRequest::SpecList(SpecListArgs::default())).unwrap();
        let VerbResponse::SpecList(r) = resp;
        let ids: Vec<_> = r.specs.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["0001", "0002"]);
    }

    #[test]
    fn spec_list_status_filter_open_excludes_closed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "first", SpecStatus::Open);
        write_spec(&layout, "0002", "second", SpecStatus::Closed);

        let args = SpecListArgs { status: Some("open".into()) };
        let resp = execute(&layout, &VerbRequest::SpecList(args)).unwrap();
        let VerbResponse::SpecList(r) = resp;
        assert_eq!(r.specs.len(), 1);
        assert_eq!(r.specs[0].id, "0001");
    }

    #[test]
    fn spec_list_invalid_status_filter_is_malformed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let args = SpecListArgs { status: Some("nope".into()) };
        let err = execute(&layout, &VerbRequest::SpecList(args)).unwrap_err();
        assert_eq!(err.to_string(), "spec.list: malformed: status must be 'open' or 'closed', got 'nope'");
    }

    #[test]
    fn spec_list_malformed_file_surfaces_with_correct_verb() {
        // ac-05 verification: error format `<verb>: <category>: <sentence>`,
        // and the verb is the one the caller invoked (not the canonical
        // layer's generic tag).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        std::fs::write(layout.spec_file("bad"), "id: '0001'\nunknown_field: oops\n").unwrap();

        let err = execute(&layout, &VerbRequest::SpecList(SpecListArgs::default())).unwrap_err();
        assert!(
            err.to_string().starts_with("spec.list: malformed: "),
            "expected spec.list-tagged malformed error, got {err}"
        );
    }

    #[test]
    fn verb_request_round_trips_through_json() {
        // The MCP surface translates `tools/call` into VerbRequest by
        // constructing `{"verb": name, "args": arguments}` and deserialising.
        // This test pins that contract.
        let json = serde_json::json!({
            "verb": "spec.list",
            "args": { "status": "open" }
        });
        let req: VerbRequest = serde_json::from_value(json).unwrap();
        match req {
            VerbRequest::SpecList(args) => assert_eq!(args.status.as_deref(), Some("open")),
        }
    }

    #[test]
    fn verb_request_rejects_unknown_args_field() {
        // deny_unknown_fields on args means typo'd MCP arguments fail loudly
        // rather than being silently ignored.
        let json = serde_json::json!({
            "verb": "spec.list",
            "args": { "stutus": "open" }
        });
        let err = serde_json::from_value::<VerbRequest>(json).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn envelope_ok_shape_is_stable() {
        let resp = VerbResponse::SpecList(SpecListResult {
            specs: vec![SpecSummary {
                id: "0001".into(),
                goal: "g".into(),
                status: "open".into(),
                cards: vec![],
                labels: vec![],
            }],
        });
        let s = envelope_ok_string(&resp).unwrap();
        // Object keys are alphabetically ordered by default in serde_json,
        // so "data" comes before "ok". Inner struct fields follow declaration
        // order via the derive: id, goal, status, cards, labels.
        assert!(s.starts_with(r#"{"data":"#), "got {s}");
        assert!(s.contains(r#""ok":true"#), "got {s}");
    }

    #[test]
    fn envelope_err_shape_matches_error_format() {
        let err = Error::not_found("spec.list", "no specs dir");
        let s = envelope_err_string(&err);
        // Outer keys alphabetical: error, ok. Inner keys alphabetical:
        // category, message, verb.
        assert_eq!(
            s,
            r#"{"error":{"category":"not-found","message":"no specs dir","verb":"spec.list"},"ok":false}"#
        );
    }

    #[test]
    fn envelope_round_trip_deterministic() {
        // Two independent serialisations of the same response must produce
        // byte-identical envelopes — this is the parity guarantee for ac-05
        // expressed at the envelope layer.
        let resp = VerbResponse::SpecList(SpecListResult {
            specs: vec![
                SpecSummary {
                    id: "0001".into(),
                    goal: "first".into(),
                    status: "open".into(),
                    cards: vec!["0020-orbit-state".into()],
                    labels: vec!["spec".into()],
                },
                SpecSummary {
                    id: "0002".into(),
                    goal: "second".into(),
                    status: "closed".into(),
                    cards: vec![],
                    labels: vec![],
                },
            ],
        });
        let a = envelope_ok_string(&resp).unwrap();
        let b = envelope_ok_string(&resp).unwrap();
        assert_eq!(a, b);
    }
}
