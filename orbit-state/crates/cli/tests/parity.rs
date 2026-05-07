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

// Helper visible to ensure the test binary depends on the CLI binary.
#[allow(dead_code)]
fn _binary_dep_anchor(_p: &Path) {}
