//! ac-05 parity harness — CLI surface.
//!
//! Strategy: the CLI's `--json` stdout MUST equal the canonical envelope
//! produced by [`orbit_state_core::envelope_ok`] over the same response.
//! The MCP test (`crates/mcp/tests/parity.rs`) checks the same expected
//! envelope from its surface — when both pass, the two surfaces agree.
//!
//! Cross-binary comparison is unnecessary: both surfaces match the same
//! reference, so by transitivity they match each other. This sidesteps the
//! `CARGO_BIN_EXE_*` cross-crate visibility problem.

use std::path::Path;
use std::process::{Command, Stdio};

mod common;

#[test]
fn spec_list_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "list"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run orbit cli");

    assert!(
        output.status.success(),
        "CLI exited non-zero: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let actual = stdout.trim_end_matches('\n');

    let expected = common::expected_envelope_for_two_specs();
    assert_eq!(
        actual, expected,
        "CLI envelope diverged from canonical envelope"
    );
}

#[test]
fn spec_list_cli_default_output_is_human_readable() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "spec", "list"])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run orbit cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    // Two specs, tab-separated, sorted by id.
    assert!(stdout.contains("0001\topen\tfirst spec"), "got: {stdout}");
    assert!(stdout.contains("0002\tclosed\tsecond spec"), "got: {stdout}");
    let pos1 = stdout.find("0001").unwrap();
    let pos2 = stdout.find("0002").unwrap();
    assert!(pos1 < pos2, "specs not sorted by id: {stdout}");
}

#[test]
fn spec_list_cli_empty_dir_emits_ok_envelope() {
    let dir = tempfile::tempdir().unwrap();
    // Don't populate — directory has no .orbit/ at all. spec_list returns Ok([]).

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "list"])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    assert!(output.status.success(), "CLI exited non-zero on empty dir");
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_empty());
}

#[test]
fn spec_list_cli_invalid_status_emits_err_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "spec", "list", "--status", "nope",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    assert!(!output.status.success(), "CLI must exit non-zero on err");
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_invalid_status());
}

#[test]
fn spec_show_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "show", "0001"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_spec_show_0001());
}

#[test]
fn spec_show_cli_missing_id_emits_not_found() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "show", "0099"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(
        actual,
        common::expected_envelope_for_spec_show_missing(dir.path())
    );
}

// ---------------------------------------------------------------------------
// State-mutation parity (ac-05 core gate) — spec.note
// ---------------------------------------------------------------------------

#[test]
fn spec_note_cli_writes_byte_identical_jsonl_and_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let note = common::fixture_note();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json",
            "spec", "note",
            &note.spec_id,
            &note.body,
            "--label", &note.labels[0],
            "--timestamp", &note.timestamp,
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(
        output.status.success(),
        "spec.note failed: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Envelope parity: CLI stdout matches the canonical envelope.
    let stdout = String::from_utf8(output.stdout).unwrap();
    let envelope = stdout.trim_end_matches('\n');
    assert_eq!(envelope, common::expected_envelope_for_fixture_note());

    // State parity: the JSONL stream on disk matches what the canonical
    // serialiser would produce. This is the "byte-identical state" half of
    // ac-05's parity contract — both surfaces, given the same input, produce
    // the same on-disk bytes.
    let stream_path = dir.path().join(".orbit/specs/0001.notes.jsonl");
    let actual = std::fs::read_to_string(&stream_path).unwrap();
    assert_eq!(actual, common::expected_notes_jsonl_for_fixture_note());
}

#[test]
fn spec_note_cli_appends_in_order_for_two_calls() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    for (i, body) in ["first", "second"].iter().enumerate() {
        let ts = format!("2026-05-07T12:00:0{i}Z");
        let status = Command::new(cli_bin)
            .args([
                "--root", dir.path().to_str().unwrap(),
                "spec", "note", "0001", body, "--timestamp", &ts,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("run cli");
        assert!(status.success());
    }

    let stream = std::fs::read_to_string(dir.path().join(".orbit/specs/0001.notes.jsonl")).unwrap();
    let lines: Vec<_> = stream.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains(r#""body":"first""#));
    assert!(lines[1].contains(r#""body":"second""#));
}

// Helper visible to ensure the test binary depends on the CLI binary.
#[allow(dead_code)]
fn _binary_dep_anchor(_p: &Path) {}
