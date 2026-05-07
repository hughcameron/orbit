//! Shared fixtures for parity tests.
//!
//! Mirror of `crates/cli/tests/common/mod.rs` — both copies MUST agree on
//! the fixture and the expected envelope. The parity claim: both surfaces
//! produce the same bytes core would produce for the same input. Both
//! surface tests assert against the same library-computed reference.

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

/// The expected error envelope for `--status nope`.
pub fn expected_envelope_for_invalid_status() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::malformed("spec.list", "status must be 'open' or 'closed', got 'nope'");
    envelope_err_string(&err)
}
