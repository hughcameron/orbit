//! Shared fixtures for parity tests.
//!
//! Fixtures here are duplicated in `crates/mcp/tests/common/mod.rs` so each
//! test crate is self-contained (`CARGO_BIN_EXE_*` only resolves binaries
//! declared by the same crate). Both copies MUST agree on the fixture and
//! the expected envelope — the parity claim is "both surfaces produce the
//! same bytes core would produce for the same input", and both tests assert
//! against the same library-computed reference.

use orbit_state_core::{envelope_ok_string, SpecListResult, SpecSummary, VerbResponse};
use std::path::Path;

/// Populate `<root>/.orbit/specs/` with two specs:
/// - `0001.yaml` — open, "first spec"
/// - `0002.yaml` — closed, "second spec"
pub fn populate_two_specs(root: &Path) {
    let orbit_dir = root.join(".orbit");
    let specs_dir = orbit_dir.join("specs");
    std::fs::create_dir_all(&specs_dir).unwrap();

    std::fs::write(
        specs_dir.join("0001.yaml"),
        "id: '0001'\ngoal: first spec\nstatus: open\n",
    )
    .unwrap();
    std::fs::write(
        specs_dir.join("0002.yaml"),
        "id: '0002'\ngoal: second spec\nstatus: closed\n",
    )
    .unwrap();
}

/// The canonical envelope expected from `spec.list` against the two-spec
/// fixture, computed by the same library helper that the surfaces use.
///
/// What this asserts when the surface output equals this string: the CLI's
/// argv-parser (or MCP's JSON-RPC handler) produces a `VerbRequest` that
/// dispatches to the same `VerbResponse` and serialises through the same
/// envelope helper as a direct in-process call. That's the parity contract.
pub fn expected_envelope_for_two_specs() -> String {
    let response = VerbResponse::SpecList(SpecListResult {
        specs: vec![
            SpecSummary {
                id: "0001".into(),
                goal: "first spec".into(),
                status: "open".into(),
                cards: vec![],
                labels: vec![],
            },
            SpecSummary {
                id: "0002".into(),
                goal: "second spec".into(),
                status: "closed".into(),
                cards: vec![],
                labels: vec![],
            },
        ],
    });
    envelope_ok_string(&response).expect("envelope serialisation infallible for fixture")
}

/// The expected envelope for an empty `.orbit/specs/` (or no `.orbit/`).
pub fn expected_envelope_for_empty() -> String {
    let response = VerbResponse::SpecList(SpecListResult { specs: vec![] });
    envelope_ok_string(&response).expect("envelope serialisation infallible")
}

/// The expected error envelope for `--status nope`.
pub fn expected_envelope_for_invalid_status() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::malformed("spec.list", "status must be 'open' or 'closed', got 'nope'");
    envelope_err_string(&err)
}

/// Expected envelope for `spec.show 0001` against the two-spec fixture.
pub fn expected_envelope_for_spec_show_0001() -> String {
    use orbit_state_core::schema::{Spec, SpecStatus};
    use orbit_state_core::{SpecShowResult, VerbResponse};
    let response = VerbResponse::SpecShow(SpecShowResult {
        spec: Spec {
            id: "0001".into(),
            goal: "first spec".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
        },
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected error envelope for `spec.show 0099` (not present).
pub fn expected_envelope_for_spec_show_missing(root: &Path) -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let path = root.join(".orbit/specs/0099.yaml");
    let err = Error::not_found("spec.show", format!("no spec at {}", path.display()));
    envelope_err_string(&err)
}

/// The deterministic note used by spec.note parity tests. Caller-supplied
/// timestamp keeps the on-disk bytes reproducible.
pub fn fixture_note() -> orbit_state_core::schema::NoteEvent {
    use orbit_state_core::schema::NoteEvent;
    NoteEvent {
        spec_id: "0001".into(),
        body: "parity test note".into(),
        labels: vec!["test".into()],
        timestamp: "2026-05-07T12:00:00Z".into(),
    }
}

/// Expected envelope for spec.note returning [`fixture_note`].
pub fn expected_envelope_for_fixture_note() -> String {
    use orbit_state_core::{SpecNoteResult, VerbResponse};
    let response = VerbResponse::SpecNote(SpecNoteResult { note: fixture_note() });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected JSONL line (with trailing newline) for [`fixture_note`].
/// The `serialise_json_line` helper produces struct-declaration-order JSON;
/// both surfaces append the same bytes if both use the same helper.
pub fn expected_notes_jsonl_for_fixture_note() -> String {
    orbit_state_core::canonical::serialise_json_line(&fixture_note())
        .expect("serialise_json_line is infallible for fixture")
}
