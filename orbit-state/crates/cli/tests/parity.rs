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
    let stream_path = dir.path().join(".orbit/specs/0001/notes.jsonl");
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

    let stream = std::fs::read_to_string(dir.path().join(".orbit/specs/0001/notes.jsonl")).unwrap();
    let lines: Vec<_> = stream.lines().collect();
    assert_eq!(lines.len(), 2);
    assert!(lines[0].contains(r#""body":"first""#));
    assert!(lines[1].contains(r#""body":"second""#));
}

// ---------------------------------------------------------------------------
// End-to-end lifecycle — create → note → update → close
// ---------------------------------------------------------------------------

#[test]
fn cli_full_spec_lifecycle() {
    let dir = tempfile::tempdir().unwrap();

    // Pre-stage a card so spec.close has something to update.
    let cards_dir = dir.path().join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0020-orbit-state.yaml"),
        "feature: orbit-state\ngoal: substrate\nmaturity: planned\n",
    )
    .unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let root = dir.path().to_str().unwrap();

    // 1. Create
    let out = Command::new(cli_bin)
        .args([
            "--root", root, "spec", "create", "0001", "the goal",
            "--card", "0020-orbit-state",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success(), "create failed: {}", String::from_utf8_lossy(&out.stderr));

    // 2. Note
    let out = Command::new(cli_bin)
        .args([
            "--root", root, "spec", "note", "0001", "kicked off",
            "--timestamp", "2026-05-07T12:00:00Z",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success(), "note failed: {}", String::from_utf8_lossy(&out.stderr));

    // 3. Update goal
    let out = Command::new(cli_bin)
        .args([
            "--root", root, "spec", "update", "0001",
            "--goal", "the revised goal",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success(), "update failed: {}", String::from_utf8_lossy(&out.stderr));

    // 4. Close — triggers transactional card update
    let out = Command::new(cli_bin)
        .args(["--root", root, "spec", "close", "0001"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success(), "close failed: {}", String::from_utf8_lossy(&out.stderr));

    // 5. Verify final state
    //    spec is closed with revised goal
    let spec_text = std::fs::read_to_string(dir.path().join(".orbit/specs/0001/spec.yaml")).unwrap();
    assert!(spec_text.contains("status: closed"), "spec not closed: {spec_text}");
    assert!(spec_text.contains("the revised goal"), "goal not updated: {spec_text}");

    //    note stream has one entry
    let notes = std::fs::read_to_string(dir.path().join(".orbit/specs/0001/notes.jsonl")).unwrap();
    assert_eq!(notes.lines().count(), 1);
    assert!(notes.contains(r#""body":"kicked off""#));

    //    linked card's specs array now contains the spec ref
    let card_text = std::fs::read_to_string(cards_dir.join("0020-orbit-state.yaml")).unwrap();
    assert!(
        card_text.contains(".orbit/specs/0001/spec.yaml"),
        "card not updated: {card_text}"
    );
}

// ---------------------------------------------------------------------------
// AC-check flag — `spec update --ac-check / --ac-uncheck` round-trip
// ---------------------------------------------------------------------------

#[test]
fn cli_spec_update_ac_check_flips_named_ac() {
    let dir = tempfile::tempdir().unwrap();
    let spec_dir = dir.path().join(".orbit/specs/test");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: test\n\
         goal: smoke\n\
         cards: []\n\
         status: open\n\
         labels: []\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: First\n  gate: true\n  checked: false\n\
         - id: ac-02\n  description: Second\n  gate: false\n  checked: false\n",
    )
    .unwrap();

    let cli = env!("CARGO_BIN_EXE_orbit");
    let root = dir.path().to_str().unwrap();

    // Check ac-01.
    let out = Command::new(cli)
        .args(["--root", root, "spec", "update", "test", "--ac-check", "ac-01"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "ac-check failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let yaml = std::fs::read_to_string(spec_dir.join("spec.yaml")).unwrap();
    assert!(yaml.contains("- id: ac-01\n  description: First\n  gate: true\n  checked: true\n"));
    assert!(yaml.contains("- id: ac-02\n  description: Second\n  gate: false\n  checked: false\n"));

    // Re-checking emits a conflict envelope.
    let out = Command::new(cli)
        .args(["--root", root, "--json", "spec", "update", "test", "--ac-check", "ac-01"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(r#""category":"conflict""#), "got: {stdout}");
    assert!(stdout.contains("ac-01 is already checked"), "got: {stdout}");

    // Uncheck flips it back.
    let out = Command::new(cli)
        .args(["--root", root, "spec", "update", "test", "--ac-uncheck", "ac-01"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(out.status.success());

    let yaml = std::fs::read_to_string(spec_dir.join("spec.yaml")).unwrap();
    assert!(yaml.contains("- id: ac-01\n  description: First\n  gate: true\n  checked: false\n"));
}

#[test]
fn cli_spec_update_ac_check_missing_ac_emits_not_found() {
    let dir = tempfile::tempdir().unwrap();
    let spec_dir = dir.path().join(".orbit/specs/test");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: test\n\
         goal: smoke\n\
         cards: []\n\
         status: open\n\
         labels: []\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: First\n  gate: false\n  checked: false\n",
    )
    .unwrap();

    let cli = env!("CARGO_BIN_EXE_orbit");
    let root = dir.path().to_str().unwrap();

    let out = Command::new(cli)
        .args(["--root", root, "--json", "spec", "update", "test", "--ac-check", "ac-99"])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(r#""category":"not-found""#), "got: {stdout}");
}

#[test]
fn cli_spec_update_both_ac_flags_is_malformed() {
    let dir = tempfile::tempdir().unwrap();
    let spec_dir = dir.path().join(".orbit/specs/test");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: test\n\
         goal: smoke\n\
         cards: []\n\
         status: open\n\
         labels: []\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: First\n  gate: false\n  checked: false\n",
    )
    .unwrap();

    let cli = env!("CARGO_BIN_EXE_orbit");
    let root = dir.path().to_str().unwrap();

    let out = Command::new(cli)
        .args([
            "--root", root, "--json",
            "spec", "update", "test",
            "--ac-check", "ac-01",
            "--ac-uncheck", "ac-01",
        ])
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(!out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(r#""category":"malformed""#), "got: {stdout}");
    assert!(stdout.contains("mutually exclusive"), "got: {stdout}");
}

#[test]
fn card_tree_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_related_cards(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "card", "tree", "0001-alpha", "--depth", "1",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_card_tree_alpha_depth1();
    assert_eq!(actual, expected, "CLI envelope diverged from canonical");
}

#[test]
fn card_specs_cli_unknown_id_emits_canonical_err_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_related_cards(dir.path());
    let cards_dir = dir.path().join(".orbit/cards");

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "card", "specs", "9999",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let actual = stdout.trim_end_matches('\n');
    let expected = common::expected_envelope_for_card_specs_unknown(&cards_dir);
    assert_eq!(actual, expected);
}

#[test]
fn graph_cli_unknown_card_emits_canonical_err_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_related_cards(dir.path());
    let cards_dir = dir.path().join(".orbit/cards");

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "graph", "--card", "9999",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let actual = stdout.trim_end_matches('\n');
    let expected = common::expected_envelope_for_graph_unknown(&cards_dir);
    assert_eq!(actual, expected);
}

#[test]
fn card_tree_cli_unknown_id_emits_canonical_err_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_related_cards(dir.path());
    let cards_dir = dir.path().join(".orbit/cards");

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "card", "tree", "9999",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    assert!(!output.status.success(), "CLI should exit non-zero on unknown id");
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let actual = stdout.trim_end_matches('\n');
    let expected = common::expected_envelope_for_card_tree_unknown(&cards_dir);
    assert_eq!(actual, expected, "error envelope diverged from canonical");
}

#[test]
fn audit_conformance_cli_clean_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_conformance_clean_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "audit", "conformance",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_audit_conformance_clean();
    assert_eq!(actual, expected, "CLI envelope diverged from canonical");
}

#[test]
fn audit_conformance_cli_park_signal_envelope() {
    // Spec 2026-05-20-conformance-park-signal ac-02: CLI conformance on a
    // fixture with one parked card and one non-park planned-empty card
    // produces an envelope with exactly one finding (the non-park card).
    let dir = tempfile::tempdir().unwrap();
    common::populate_conformance_park_signal_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "audit", "conformance",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_audit_conformance_park_signal_fixture(dir.path());
    assert_eq!(actual, expected, "CLI park-signal envelope diverged from library reference");
}

#[test]
fn audit_drift_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_card_with_drift(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "audit", "drift",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_audit_drift_one_unknown();
    assert_eq!(actual, expected, "CLI envelope diverged from canonical");
}

#[test]
fn graph_cli_mermaid_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_related_cards(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "graph",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_graph_mermaid_two_related_cards();
    assert_eq!(actual, expected, "CLI envelope diverged from canonical");
}

#[test]
fn overview_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_related_cards(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "overview",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_overview_two_related_cards();
    assert_eq!(actual, expected, "CLI envelope diverged from canonical");
}

#[test]
fn card_specs_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_card_with_linked_spec(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "card", "specs", "0001-alpha",
        ])
        .stdin(Stdio::null())
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
    let expected = common::expected_envelope_for_card_specs_alpha();
    assert_eq!(actual, expected, "CLI envelope diverged from canonical");
}

// ---------------------------------------------------------------------------
// spec.close AC pre-flight (spec 2026-05-13-spec-close-ac-preflight, ac-05)
// ---------------------------------------------------------------------------

#[test]
fn spec_close_cli_unchecked_acs_emits_conflict_envelope() {
    // ac-05 / ac-02: CLI `spec close` against a spec with one unchecked
    // non-time-gated AC emits the canonical conflict envelope; no
    // state mutation occurs.
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_close_preflight_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "close", "0001"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(!output.status.success(), "expected non-zero exit on conflict");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_spec_close_unchecked_blocking());

    // State parity: spec is still open, card is unmutated.
    let spec_text = std::fs::read_to_string(dir.path().join(".orbit/specs/0001/spec.yaml")).unwrap();
    assert!(spec_text.contains("status: open"), "spec mutated: {spec_text}");
    let card_text = std::fs::read_to_string(dir.path().join(".orbit/cards/0020-orbit-state.yaml")).unwrap();
    assert!(!card_text.contains("specs:"), "card specs array touched: {card_text}");
}

#[test]
fn spec_close_cli_force_proceeds_with_envelope() {
    // ac-05 / ac-03: CLI `spec close --force` bypasses the unchecked-AC
    // guard and emits the canonical ok envelope with `forced_unchecked`
    // and `deferrable_open` populated.
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_close_preflight_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "close", "0001", "--force"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "force should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    // `closed_at` is dynamic (current UTC time, set by spec.close per spec
    // 2026-05-26-scope-discipline-front-loaded ac-10). Strip it before the
    // byte-equal compare so the rest of the envelope remains a hard parity
    // contract.
    let actual_stripped = strip_closed_at(actual);
    assert_eq!(actual_stripped, common::expected_envelope_for_spec_close_force());

    // State parity: spec is closed on disk, card's specs array gained the ref.
    let spec_text = std::fs::read_to_string(dir.path().join(".orbit/specs/0001/spec.yaml")).unwrap();
    assert!(spec_text.contains("status: closed"), "spec not closed: {spec_text}");
    let card_text = std::fs::read_to_string(dir.path().join(".orbit/cards/0020-orbit-state.yaml")).unwrap();
    assert!(
        card_text.contains(".orbit/specs/0001/spec.yaml"),
        "card not updated: {card_text}"
    );
}

#[test]
fn spec_close_cli_deferrable_only_proceeds_without_force() {
    // spec 2026-05-16-ac-taxonomy ac-02 (generalising ac-05 / ac-04 of
    // the precursor): CLI `spec close` against a spec whose sole unchecked
    // AC is deferrable-kind (Observation) succeeds without `--force`;
    // envelope carries `deferrable_open` and empty `forced_unchecked`.
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_close_only_deferrable_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "close", "0001"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "close should succeed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    // `closed_at` is dynamic (see ac-10 of
    // 2026-05-26-scope-discipline-front-loaded) — strip before compare.
    let actual_stripped = strip_closed_at(actual);
    assert_eq!(actual_stripped, common::expected_envelope_for_spec_close_only_deferrable());

    // State parity: spec is closed.
    let spec_text = std::fs::read_to_string(dir.path().join(".orbit/specs/0001/spec.yaml")).unwrap();
    assert!(spec_text.contains("status: closed"), "spec not closed: {spec_text}");
}

/// Strip the `closed_at` field from a spec-close envelope JSON string so
/// parity tests can byte-compare against an envelope whose `closed_at`
/// is `None`. The field's value is a dynamic UTC timestamp set by
/// `spec.close` per spec 2026-05-26-scope-discipline-front-loaded ac-10.
fn strip_closed_at(envelope: &str) -> String {
    // Matches `,"closed_at":"...non-quote..."` — the field is always
    // emitted with a leading comma since `id` comes alphabetically first.
    let pattern = r#","closed_at":""#;
    if let Some(start) = envelope.find(pattern) {
        let after_open_quote = start + pattern.len();
        if let Some(rel_end) = envelope[after_open_quote..].find('"') {
            let end = after_open_quote + rel_end + 1; // include closing quote
            let mut out = String::with_capacity(envelope.len());
            out.push_str(&envelope[..start]);
            out.push_str(&envelope[end..]);
            return out;
        }
    }
    envelope.to_string()
}

// ---------------------------------------------------------------------------
// Spec 2026-05-15-agent-learning-loop parity tests
// ---------------------------------------------------------------------------

#[test]
fn session_start_cli_envelope_matches_canonical() {
    let dir = tempfile::tempdir().unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "session",
            "start",
            "--id",
            common::PARITY_SESSION_ID,
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_session_start(dir.path()));

    let on_disk = std::fs::read_to_string(dir.path().join(".orbit/.session-id")).unwrap();
    assert_eq!(on_disk.trim(), common::PARITY_SESSION_ID);
}

#[test]
fn skill_record_invocation_cli_envelope_matches_canonical() {
    let dir = tempfile::tempdir().unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "skill",
            "record-invocation",
            "card",
            "--outcome",
            "worked",
            "--session-id",
            common::PARITY_SESSION_ID,
            "--timestamp",
            common::PARITY_TIMESTAMP,
        ])
        .env_remove("ORBIT_SESSION_ID")
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_skill_record_invocation());

    // State parity: one JSONL row on disk.
    let path = dir.path().join(".orbit/skills/card.invocations.jsonl");
    let body = std::fs::read_to_string(&path).unwrap();
    assert_eq!(body.lines().count(), 1);
}

#[test]
fn skill_recurrence_cli_envelope_empty_matches_canonical() {
    let dir = tempfile::tempdir().unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "skill",
            "recurrence",
            "design",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');
    assert_eq!(actual, common::expected_envelope_for_skill_recurrence_empty());
}

#[test]
fn session_distill_cli_envelope_matches_canonical() {
    use orbit_state_core::schema::Session;
    let dir = tempfile::tempdir().unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");

    // Write the distillate via --from to avoid stdin plumbing.
    let from = dir.path().join("distillate.txt");
    std::fs::write(&from, "parity-distillate").unwrap();

    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "session",
            "distill",
            "--session-id",
            common::PARITY_SESSION_ID,
            "--from",
            from.to_str().unwrap(),
        ])
        .env_remove("ORBIT_SESSION_ID")
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success(), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let actual = stdout.trim_end_matches('\n');

    // Read substrate-stamped timestamps from disk.
    let session_path = dir
        .path()
        .join(".orbit/sessions")
        .join(format!("{}.yaml", common::PARITY_SESSION_ID));
    let text = std::fs::read_to_string(&session_path).unwrap();
    let session: Session = serde_yaml::from_str(&text).unwrap();
    let expected = common::expected_envelope_for_session_distill(
        "parity-distillate",
        &session.started_at,
        session.ended_at.as_deref().unwrap_or(""),
    );
    assert_eq!(actual, expected);
}

// ----- audit topology CLI parity (spec 2026-05-18-documentation-topology ac-06) -----

#[test]
fn audit_topology_cli_not_configured_envelope() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "audit", "topology"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "exit 0 for not-configured");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    assert_eq!(envelope["ok"], true);
    let result = &envelope["data"]["result"];
    assert_eq!(result["configured"], false);
    assert_eq!(result["topology_drift"].as_array().unwrap().len(), 0);
}

#[test]
fn audit_topology_cli_clean_envelope() {
    // Substrate-folder shape per choice 0025: .orbit/topology/<subsystem>.yaml.
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(orbit_dir.join("topology")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/myauth")).unwrap();
    std::fs::write(dir.path().join("src/myauth/mod.rs"), "// mod\n").unwrap();
    // One well-formed entry whose canonical_code resolves.
    let entry_yaml = "subsystem: myauth\ncanonical_code:\n- src/myauth/mod.rs\n";
    std::fs::write(orbit_dir.join("topology/myauth.yaml"), entry_yaml).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "audit", "topology"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "exit 0 for clean");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    assert_eq!(envelope["ok"], true);
    let result = &envelope["data"]["result"];
    assert_eq!(result["configured"], true);
    assert_eq!(result["topology_drift"].as_array().unwrap().len(), 0);
}

#[test]
fn audit_topology_cli_drift_envelope() {
    // src/ingest exists in the codebase but has no topology entry →
    // missing_entry drift. One entry (myauth) keeps the substrate
    // "configured" without polluting the missing_entry assertion.
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(orbit_dir.join("topology")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/myauth")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/ingest")).unwrap();
    std::fs::write(dir.path().join("src/myauth/mod.rs"), "// mod\n").unwrap();
    let entry_yaml = "subsystem: myauth\ncanonical_code:\n- src/myauth/mod.rs\n";
    std::fs::write(orbit_dir.join("topology/myauth.yaml"), entry_yaml).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "audit", "topology"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "exit 0 for drift-present");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    assert_eq!(envelope["ok"], true);
    let result = &envelope["data"]["result"];
    assert_eq!(result["configured"], true);
    let drift = result["topology_drift"].as_array().unwrap();
    assert!(
        drift
            .iter()
            .any(|d| d["subsystem"] == "ingest" && d["drift_kind"] == "missing_entry"),
        "expected ingest/missing_entry, got {drift:?}",
    );
}

// ----- session prime topology_drift CLI parity (spec 2026-05-18-topology-substrate-wires ac-02) -----

#[test]
fn session_prime_cli_topology_drift_omitted_when_config_absent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "session", "prime"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    // ac-02: key absent (not empty array) when capability not configured.
    assert!(
        result.get("topology_drift").is_none(),
        "expected key absent when config absent, got {result}",
    );
}

#[test]
fn session_prime_cli_topology_drift_omitted_when_docs_topology_unset() {
    // ac-02 4th state: config file present but docs.topology unset →
    // configured == false → key absent (not just empty array).
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(&orbit_dir).unwrap();
    std::fs::write(orbit_dir.join("config.yaml"), "{}\n").unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "session", "prime"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    assert!(
        result.get("topology_drift").is_none(),
        "expected key absent when docs.topology unset, got {result}",
    );
}

#[test]
fn session_prime_cli_topology_drift_empty_array_when_clean() {
    // Substrate-folder shape: one valid entry, canonical_code resolves.
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(orbit_dir.join("topology")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/myauth")).unwrap();
    std::fs::write(dir.path().join("src/myauth/mod.rs"), "// mod\n").unwrap();
    let entry_yaml = "subsystem: myauth\ncanonical_code:\n- src/myauth/mod.rs\n";
    std::fs::write(orbit_dir.join("topology/myauth.yaml"), entry_yaml).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "session", "prime"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    // ac-02: configured + clean → key present, value empty array.
    let drift = result
        .get("topology_drift")
        .expect("topology_drift key must be present when configured");
    assert_eq!(
        drift.as_array().expect("array").len(),
        0,
        "expected empty array when clean",
    );
}

#[test]
fn session_prime_cli_topology_drift_populated_when_drift_present() {
    // Substrate-folder shape: one valid entry (myauth) keeps substrate
    // "configured"; src/ingest exists in codebase but has no entry →
    // missing_entry drift.
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(orbit_dir.join("topology")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/myauth")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/ingest")).unwrap();
    std::fs::write(dir.path().join("src/myauth/mod.rs"), "// mod\n").unwrap();
    let entry_yaml = "subsystem: myauth\ncanonical_code:\n- src/myauth/mod.rs\n";
    std::fs::write(orbit_dir.join("topology/myauth.yaml"), entry_yaml).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "session", "prime"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    let drift = result["topology_drift"].as_array().expect("array");
    assert!(
        drift
            .iter()
            .any(|d| d["subsystem"] == "ingest" && d["drift_kind"] == "missing_entry"),
        "expected ingest/missing_entry, got {drift:?}",
    );
}

// ----- spec.close topology_warnings CLI parity (ac-03) -----

#[test]
fn spec_close_cli_topology_warnings_populated_on_word_boundary_match() {
    // Substrate-folder shape: per-subsystem yaml entry. Subsystem slug
    // must be slug-shaped (lowercase, hyphens not underscores) per
    // TopologyEntry::validate.
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(orbit_dir.join("specs/0001")).unwrap();
    std::fs::create_dir_all(orbit_dir.join("cards")).unwrap();
    std::fs::create_dir_all(orbit_dir.join("topology")).unwrap();
    std::fs::create_dir_all(dir.path().join("src/session-prime")).unwrap();
    std::fs::write(
        dir.path().join("src/session-prime/mod.rs"),
        "// mod\n",
    )
    .unwrap();

    let entry_yaml = "subsystem: session-prime\ncanonical_code:\n- src/session-prime/mod.rs\n";
    std::fs::write(orbit_dir.join("topology/session-prime.yaml"), entry_yaml).unwrap();

    // Plant a spec whose goal mentions the subsystem.
    let spec_yaml = "id: \"0001\"\ngoal: Adding a topology_drift field to session-prime envelope.\ncards: []\nstatus: open\nlabels: []\nacceptance_criteria: []\n";
    std::fs::write(orbit_dir.join("specs/0001/spec.yaml"), spec_yaml).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "spec", "close", "0001"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "spec.close should succeed: {output:?}");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    let warnings = result["topology_warnings"].as_array().expect("array");
    assert!(
        warnings.iter().any(|w| w["subsystem"] == "session-prime"),
        "expected session-prime warning, got {warnings:?}",
    );
}

// ----- memory.remember nudge CLI parity (ac-04) -----

#[test]
fn memory_remember_cli_topology_label_envelope_carries_nudge() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "memory",
            "remember",
            "k1",
            "body",
            "--label",
            "topology",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    let nudge = result["nudge"].as_str().expect("nudge present");
    assert!(
        nudge.contains("/orb:topology"),
        "nudge text must reference /orb:topology, got {nudge}",
    );
}

#[test]
fn memory_remember_cli_no_label_envelope_omits_nudge() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "memory",
            "remember",
            "k2",
            "body",
            "--label",
            "unrelated",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    assert!(
        result.get("nudge").is_none(),
        "nudge key must be absent without topology label, got {result}",
    );
}

#[test]
fn memory_remember_cli_no_nudge_flag_suppresses_envelope_nudge() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "memory",
            "remember",
            "k3",
            "body",
            "--label",
            "topology",
            "--no-nudge",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    assert!(
        result.get("nudge").is_none(),
        "nudge must be absent when --no-nudge is set even with topology label, got {result}",
    );
}

#[test]
fn memory_remember_cli_human_mode_renders_nudge_to_stderr() {
    // ac-04: human mode (no --json) renders the nudge to STDERR; stdout
    // must NOT contain the nudge text. Locks the channel split.
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "memory",
            "remember",
            "k4",
            "body",
            "--label",
            "topology",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let stderr = String::from_utf8(output.stderr).expect("utf-8");
    assert!(
        stderr.contains("/orb:topology"),
        "nudge must appear on STDERR, got stderr={stderr:?}",
    );
    assert!(
        !stdout.contains("/orb:topology"),
        "nudge must NOT appear on STDOUT, got stdout={stdout:?}",
    );
}

// ----- memory.remember --cite + memory.match cites CLI parity (ac-06) -----
//
// Per spec 2026-05-27-memory-cite-reading ac-06: the --cite flag and the
// cites field are exposed on both CLI and MCP. These tests exercise the
// CLI surface and the matching MCP parity tests live in
// `crates/mcp/tests/parity.rs`. State-mutation parity (same on-disk YAML
// regardless of surface) is asserted by the in-MCP `match-cited` flow.

#[test]
fn cite_remember_cli_with_two_cite_flags_writes_cites_field() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "memory",
            "remember",
            "cli-cite",
            "body",
            "--cite",
            "docs/a.md",
            "--cite",
            "docs/b.md",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let memory = &envelope["data"]["result"]["memory"];
    let cites = memory["cites"].as_array().expect("cites array");
    assert_eq!(cites.len(), 2);
    assert_eq!(cites[0]["path"].as_str(), Some("docs/a.md"));
    assert_eq!(cites[1]["path"].as_str(), Some("docs/b.md"));
    let yaml = std::fs::read_to_string(dir.path().join(".orbit/memories/cli-cite.yaml")).unwrap();
    assert!(yaml.contains("docs/a.md"));
    assert!(yaml.contains("docs/b.md"));
}

#[test]
fn cite_remember_cli_without_cite_flag_omits_cites_field_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "memory",
            "remember",
            "cli-no-cite",
            "body",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let memory = &envelope["data"]["result"]["memory"];
    assert!(
        memory.get("cites").is_none(),
        "cite-less memory remember response must omit cites: {memory}",
    );
    let yaml = std::fs::read_to_string(dir.path().join(".orbit/memories/cli-no-cite.yaml")).unwrap();
    assert!(!yaml.contains("cites:"), "yaml must omit cites: {yaml}");
}

#[test]
fn cite_match_cli_exposes_cites_on_every_result() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    // Plant cited memory.
    let _ = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "memory",
            "remember",
            "cli-match-cited",
            "decision-moment mechanism for cli parity",
            "--label",
            "card-cli-parity",
            "--cite",
            "docs/cited.md",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    // Plant uncited memory.
    let _ = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "memory",
            "remember",
            "cli-match-uncited",
            "decision-moment mechanism for cli parity",
            "--label",
            "card-cli-parity",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "memory",
            "match",
            "decision mechanism parity",
            "--label",
            "card-cli-parity",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let matches = envelope["data"]["result"]["matches"]
        .as_array()
        .expect("matches array");
    assert_eq!(matches.len(), 2);
    for m in matches {
        let memory = &m["memory"];
        let cites = memory
            .get("cites")
            .and_then(serde_json::Value::as_array)
            .expect("every match.memory must carry cites field");
        let key = memory["key"].as_str().unwrap();
        if key == "cli-match-cited" {
            assert_eq!(cites.len(), 1);
            assert_eq!(cites[0]["path"].as_str(), Some("docs/cited.md"));
        } else {
            assert!(cites.is_empty(), "uncited match must have empty cites");
        }
    }
}

// ----- topology setup CLI parity (spec 2026-05-18-topology-substrate-migration ac-05) -----

/// Set up a fixture that exercises the plugin-repo branch of
/// topology.setup: writes `plugin_repo: true` in config and stubs the
/// substrate-typed seeds' canonical_code path. Used by the parity tests
/// that pre-date plugin_repo gating; the README-only branch has its own
/// targeted parity tests.
fn populate_plugin_repo_topology_fixture(root: &std::path::Path) {
    let orbit_dir = root.join(".orbit");
    std::fs::create_dir_all(&orbit_dir).unwrap();
    std::fs::write(orbit_dir.join("config.yaml"), "plugin_repo: true\n").unwrap();
    let stub = root.join("orbit-state/crates/core/src");
    std::fs::create_dir_all(&stub).unwrap();
    std::fs::write(stub.join("schema.rs"), "// stub\n").unwrap();
}

#[test]
fn topology_setup_cli_greenfield_envelope() {
    let dir = tempfile::tempdir().unwrap();
    populate_plugin_repo_topology_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "topology", "setup"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success(), "exit 0 for greenfield setup");
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    assert_eq!(envelope["ok"], true);
    let result = &envelope["data"]["result"];
    assert_eq!(result["dir_created"], true);
    assert_eq!(result["config_cleaned"], false);
    assert_eq!(result["declined"], false);
    assert_eq!(result["readme_created"], false);
    let seeds = result["seeds_created"].as_array().unwrap();
    assert_eq!(seeds.len(), 5, "five orbit-substrate seeds");
    // Each seed file landed on disk.
    for slug in &["cards", "choices", "memories", "specs-substrate", "topology"] {
        let path = dir.path().join(".orbit/topology").join(format!("{slug}.yaml"));
        assert!(path.exists(), "missing seed file: {slug}");
    }
}

#[test]
fn topology_setup_cli_idempotent() {
    // Two-stage idempotency per spec ac-05.
    let dir = tempfile::tempdir().unwrap();
    populate_plugin_repo_topology_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    // First invocation mutates.
    Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "topology", "setup"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    // Second invocation is a no-op on every surface.
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "topology", "setup"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    assert_eq!(result["dir_created"], false, "dir already exists");
    assert_eq!(result["seeds_created"].as_array().unwrap().len(), 0);
    assert_eq!(result["seeds_skipped"].as_array().unwrap().len(), 5);
}

#[test]
fn topology_setup_cli_brownfield_strips_legacy_config() {
    let dir = tempfile::tempdir().unwrap();
    let orbit_dir = dir.path().join(".orbit");
    std::fs::create_dir_all(&orbit_dir).unwrap();
    std::fs::write(
        orbit_dir.join("config.yaml"),
        "docs:\n  topology: docs/topology.md\n",
    )
    .unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "topology", "setup"])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let result = &envelope["data"]["result"];
    assert_eq!(result["config_cleaned"], true, "legacy docs.topology must be stripped");
    // Confirm on-disk: docs.topology absent.
    let config_text = std::fs::read_to_string(orbit_dir.join("config.yaml")).unwrap();
    assert!(
        !config_text.contains("topology: docs/topology.md"),
        "post-cleanup config must not carry docs.topology: {config_text}"
    );
}

// ============================================================================
// Spec 2026-05-21-richer-reconcile-rules ac-04 — canonicalise / reconcile
// emit a "run orbit audit conformance --json" breadcrumb on parse failure.
// The breadcrumb fires identically on both envelopes (canonicalise +
// reconcile) and on both surfaces (JSON envelope `next_step` field +
// human-readable stderr line).
// ============================================================================

#[test]
fn ac_04_canonicalise_envelope_next_step_set_on_parse_failures() {
    // A spec.yaml with an unknown top-level field — strict parse rejects
    // (deny_unknown_fields per ac-01 of the orbit-state v0.1 spec), so
    // canonicalise reports parse_failed = 1. The additive `next_step`
    // field carries the breadcrumb.
    let dir = tempfile::tempdir().unwrap();
    let specs_dir = dir.path().join(".orbit/specs/0001");
    std::fs::create_dir_all(&specs_dir).unwrap();
    std::fs::write(
        specs_dir.join("spec.yaml"),
        "id: '0001'\ngoal: g\nstatus: open\nbogus_unknown_field: x\n",
    )
    .unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "canonicalise"])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let next_step = envelope["next_step"].as_str().expect("next_step is a string");
    assert!(
        next_step.contains("1 file(s) failed parse"),
        "next_step missing count: {next_step}"
    );
    assert!(
        next_step.contains("run \"orbit audit conformance --json\""),
        "next_step missing verb pointer: {next_step}"
    );
}

#[test]
fn ac_04_canonicalise_envelope_next_step_null_on_clean_tree() {
    // A clean tree → parse_failed empty → next_step is JSON null.
    // The field is always present (additive) so JSON consumers branch
    // on null-vs-string without checking for the field's existence.
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "--json", "canonicalise"])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    assert!(
        envelope["next_step"].is_null(),
        "next_step should be null on clean tree; got: {}",
        envelope["next_step"]
    );
}

#[test]
fn ac_04_canonicalise_human_emits_breadcrumb_on_parse_failures() {
    let dir = tempfile::tempdir().unwrap();
    let specs_dir = dir.path().join(".orbit/specs/0001");
    std::fs::create_dir_all(&specs_dir).unwrap();
    std::fs::write(
        specs_dir.join("spec.yaml"),
        "id: '0001'\ngoal: g\nstatus: open\nbogus_unknown_field: x\n",
    )
    .unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args(["--root", dir.path().to_str().unwrap(), "canonicalise"])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(
        stderr.contains("run \"orbit audit conformance --json\" for structured findings"),
        "stderr breadcrumb missing: {stderr}"
    );
}

#[test]
fn ac_04_reconcile_envelope_next_step_set_on_post_reconcile_parse_failures() {
    // A card.yaml lacking the required `feature` field — permissive
    // parse succeeds, walk finds no unknown fields to apply, typed
    // re-parse fails because feature is required. Reconcile's
    // `parse_failed` is non-empty; the additive `next_step` carries
    // the breadcrumb identically to canonicalise.
    let dir = tempfile::tempdir().unwrap();
    let cards_dir = dir.path().join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0099-test.yaml"),
        "as_a: a\ni_want: w\nso_that: t\nmaturity: planned\n",
    )
    .unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "canonicalise",
            "--reconcile",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    let next_step = envelope["next_step"].as_str().expect("next_step is a string");
    assert!(
        next_step.contains("1 file(s) failed parse"),
        "reconcile next_step missing count: {next_step}"
    );
    assert!(
        next_step.contains("run \"orbit audit conformance --json\""),
        "reconcile next_step missing verb pointer: {next_step}"
    );
}

#[test]
fn ac_04_reconcile_envelope_next_step_null_on_clean_tree() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_two_specs(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "--json",
            "canonicalise",
            "--reconcile",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    let envelope: serde_json::Value =
        serde_json::from_str(stdout.trim_end_matches('\n')).expect("json");
    assert!(
        envelope["next_step"].is_null(),
        "reconcile next_step should be null on clean tree; got: {}",
        envelope["next_step"]
    );
}

#[test]
fn ac_04_reconcile_human_emits_breadcrumb_on_parse_failures() {
    let dir = tempfile::tempdir().unwrap();
    let cards_dir = dir.path().join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0099-test.yaml"),
        "as_a: a\ni_want: w\nso_that: t\nmaturity: planned\n",
    )
    .unwrap();

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "canonicalise",
            "--reconcile",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");

    let stderr = String::from_utf8(output.stderr).expect("stderr is utf-8");
    assert!(
        stderr.contains("run \"orbit audit conformance --json\" for structured findings"),
        "reconcile stderr breadcrumb missing: {stderr}"
    );
}

// ----- substrate.classify CLI parity (spec 2026-05-24-setup-is-orbit-state-aware ac-18) -----

fn substrate_classify_cli_envelope(root: &Path) -> String {
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            root.to_str().unwrap(),
            "--json",
            "substrate",
            "classify",
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run orbit cli");
    assert!(
        output.status.success(),
        "CLI exited non-zero: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).expect("stdout is utf-8");
    stdout.trim_end_matches('\n').to_string()
}

#[test]
fn substrate_classify_cli_idempotent_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_substrate_idempotent_fixture(dir.path());
    let actual = substrate_classify_cli_envelope(dir.path());
    let expected = common::expected_envelope_for_substrate_classify(
        orbit_state_core::SubstrateLayoutState::Idempotent,
    );
    assert_eq!(
        actual, expected,
        "CLI substrate.classify envelope diverged for idempotent shape"
    );
}

#[test]
fn substrate_classify_cli_wrapped_undotted_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_substrate_wrapped_undotted_fixture(dir.path());
    let actual = substrate_classify_cli_envelope(dir.path());
    let expected = common::expected_envelope_for_substrate_classify(
        orbit_state_core::SubstrateLayoutState::WrappedUndotted,
    );
    assert_eq!(
        actual, expected,
        "CLI substrate.classify envelope diverged for wrapped-undotted shape"
    );
}

#[test]
fn substrate_classify_cli_brownfield_bare_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_substrate_brownfield_bare_fixture(dir.path());
    let actual = substrate_classify_cli_envelope(dir.path());
    let expected = common::expected_envelope_for_substrate_classify(
        orbit_state_core::SubstrateLayoutState::BrownfieldBare,
    );
    assert_eq!(
        actual, expected,
        "CLI substrate.classify envelope diverged for brownfield-bare shape"
    );
}

// ----- undotted_substrate finding CLI parity (spec 2026-05-24-workflow-conformance) -----

#[test]
fn audit_conformance_cli_wrapped_undotted_envelope() {
    // Wrapped-undotted shape — orbit/cards/ exists, .orbit/cards/ absent.
    // The conformance envelope MUST contain exactly one finding
    // (undotted_substrate, HIGH, subject orbit/, evidence carrying the
    // four per-subdir counts). All .orbit/-dependent finding families
    // suppressed.
    let dir = tempfile::tempdir().unwrap();
    common::populate_substrate_wrapped_undotted_fixture(dir.path());

    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root", dir.path().to_str().unwrap(),
            "--json", "audit", "conformance",
        ])
        .stdin(Stdio::null())
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
    let expected =
        common::expected_envelope_for_audit_conformance_wrapped_undotted(dir.path());
    assert_eq!(
        actual, expected,
        "CLI undotted_substrate envelope diverged from library reference"
    );
}

// Helper visible to ensure the test binary depends on the CLI binary.
#[allow(dead_code)]
fn _binary_dep_anchor(_p: &Path) {}

// ---------------------------------------------------------------------------
// spec.acs / next-ac / blocking-gate / has-unchecked / check parity
// (per spec 2026-05-24-port-acceptance-shim ac-07).
// ---------------------------------------------------------------------------

fn run_cli_json(root: &Path, args: &[&str]) -> std::process::Output {
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let mut full_args: Vec<&str> = vec!["--root", root.to_str().unwrap(), "--json"];
    full_args.extend(args);
    Command::new(cli_bin)
        .args(&full_args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run cli")
}

#[test]
fn spec_acs_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "acs", "0010"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_acs_mixed()
    );
}

#[test]
fn spec_next_ac_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "next-ac", "0010"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_next_ac_mixed()
    );
}

#[test]
fn spec_blocking_gate_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "blocking-gate", "0010"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_blocking_gate_mixed()
    );
}

#[test]
fn spec_has_unchecked_cli_true_emits_envelope_and_exits_zero() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "has-unchecked", "0010"]);
    assert!(output.status.success(), "exit 0 = unchecked exists (shim parity)");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_has_unchecked_true()
    );
}

#[test]
fn spec_has_unchecked_cli_false_emits_envelope_and_exits_one() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_all_checked_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "has-unchecked", "0011"]);
    // Shim contract: exit 1 = no unchecked. Envelope is still ok-shaped.
    assert_eq!(output.status.code(), Some(1));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_has_unchecked_false()
    );
}

#[test]
fn spec_check_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "check", "0010", "ac-02"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_check_ac02()
    );
}

#[test]
fn spec_check_cli_missing_ac_returns_not_found() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "check", "0010", "ac-99"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_check_missing()
    );
}

#[test]
fn spec_check_cli_already_checked_returns_conflict() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_spec_acs_mixed_fixture(dir.path());
    let output = run_cli_json(dir.path(), &["spec", "check", "0010", "ac-03"]);
    assert!(!output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_check_already_checked()
    );
}

// ---------------------------------------------------------------------------
// spec.promote parity (per spec 2026-05-25-port-promote-sh ac-06).
// ---------------------------------------------------------------------------

#[test]
fn spec_promote_cli_json_matches_canonical_envelope() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_promote_card_fixture(dir.path());
    let output = run_cli_json(
        dir.path(),
        &[
            "spec",
            "promote",
            ".orbit/cards/0050-promote-fixture.yaml",
            "--today",
            common::PROMOTE_FIXTURE_TODAY,
        ],
    );
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_promote_fixture()
    );
    // Side-effect verification: the spec file was actually written.
    let spec_file = dir
        .path()
        .join(format!(
            ".orbit/specs/{}-promote-fixture/spec.yaml",
            common::PROMOTE_FIXTURE_TODAY
        ));
    assert!(spec_file.exists(), "non-dry-run must write the spec file");
}

#[test]
fn spec_promote_cli_dry_run_writes_nothing_and_envelope_matches() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_promote_card_fixture(dir.path());
    // Snapshot layout before the dry-run call.
    let layout_before = snapshot_layout_paths(dir.path());
    let output = run_cli_json(
        dir.path(),
        &[
            "spec",
            "promote",
            ".orbit/cards/0050-promote-fixture.yaml",
            "--dry-run",
            "--today",
            common::PROMOTE_FIXTURE_TODAY,
        ],
    );
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_promote_fixture_dry_run()
    );
    // No spec file written.
    let spec_file = dir
        .path()
        .join(format!(
            ".orbit/specs/{}-promote-fixture/spec.yaml",
            common::PROMOTE_FIXTURE_TODAY
        ));
    assert!(!spec_file.exists(), "dry-run must NOT write the spec file");
    // Full layout snapshot equality — defence-in-depth.
    let layout_after = snapshot_layout_paths(dir.path());
    assert_eq!(
        layout_before, layout_after,
        "dry-run must produce a byte-identical filesystem"
    );
}

#[test]
fn spec_promote_cli_dry_run_succeeds_when_target_exists() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_promote_card_fixture(dir.path());
    // First call: actually create the spec.
    let _ = run_cli_json(
        dir.path(),
        &[
            "spec",
            "promote",
            ".orbit/cards/0050-promote-fixture.yaml",
            "--today",
            common::PROMOTE_FIXTURE_TODAY,
        ],
    );
    // Second call: --dry-run against the now-existing target — must succeed
    // (per spec ac-04: dry-run path stays read-only and reports what WOULD
    // be written, not what is).
    let output = run_cli_json(
        dir.path(),
        &[
            "spec",
            "promote",
            ".orbit/cards/0050-promote-fixture.yaml",
            "--dry-run",
            "--today",
            common::PROMOTE_FIXTURE_TODAY,
        ],
    );
    assert!(
        output.status.success(),
        "dry-run must succeed even when target exists; stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_spec_promote_fixture_dry_run()
    );
}

#[test]
fn spec_promote_cli_default_stdout_is_bare_spec_id() {
    // Per spec ac-02: default-mode stdout is the bare spec id alone so
    // drive's `SPEC_ID=$(orbit spec promote <card-path>)` shell-capture
    // shape preserves.
    let dir = tempfile::tempdir().unwrap();
    common::populate_promote_card_fixture(dir.path());
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let output = Command::new(cli_bin)
        .args([
            "--root",
            dir.path().to_str().unwrap(),
            "spec",
            "promote",
            ".orbit/cards/0050-promote-fixture.yaml",
            "--today",
            common::PROMOTE_FIXTURE_TODAY,
        ])
        .stdin(Stdio::null())
        .output()
        .expect("run cli");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout,
        format!("{}-promote-fixture\n", common::PROMOTE_FIXTURE_TODAY),
        "stdout must be just the spec id + newline"
    );
}

/// Walk `.orbit/` and return a sorted list of (relative path, file size).
/// Used by the dry-run no-side-effect assertion to detect any FS mutation.
fn snapshot_layout_paths(root: &Path) -> Vec<(String, u64)> {
    let orbit_dir = root.join(".orbit");
    if !orbit_dir.exists() {
        return vec![];
    }
    let mut entries = vec![];
    walk_dir(&orbit_dir, &orbit_dir, &mut entries);
    entries.sort();
    entries
}

fn walk_dir(root: &Path, dir: &Path, out: &mut Vec<(String, u64)>) {
    for entry in std::fs::read_dir(dir).unwrap().flatten() {
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap().display().to_string();
        let meta = entry.metadata().unwrap();
        if meta.is_dir() {
            walk_dir(root, &path, out);
        } else {
            out.push((rel, meta.len()));
        }
    }
}

// ---------------------------------------------------------------------------
// card.show with mixed-target relations (per spec
// 2026-05-25-relation-schema-choice-targets ac-06).
// ---------------------------------------------------------------------------

#[test]
fn card_show_cli_json_matches_canonical_envelope_with_mixed_relations() {
    let dir = tempfile::tempdir().unwrap();
    common::populate_card_with_mixed_relations(dir.path());
    let output = run_cli_json(dir.path(), &["card", "show", "0050-mixed"]);
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_card_show_mixed_relations()
    );
}

// ---------------------------------------------------------------------------
// setup.files parity (per spec 2026-05-25-port-setup-method-sh ac-06).
// ---------------------------------------------------------------------------

fn run_setup_files_cli(
    project_root: &Path,
    plugin_root: &Path,
    extra_args: &[&str],
) -> std::process::Output {
    let cli_bin = env!("CARGO_BIN_EXE_orbit");
    let (method, style) = common::write_setup_files_canonicals(plugin_root);
    let project_str = project_root.to_str().unwrap();
    let method_str = method.to_str().unwrap();
    let style_str = style.to_str().unwrap();
    let mut args: Vec<&str> = vec![
        "--root",
        project_str,
        "--json",
        "setup",
        "files",
        "--project-root",
        project_str,
        "--canonical-method",
        method_str,
        "--canonical-style",
        style_str,
    ];
    args.extend(extra_args);
    Command::new(cli_bin)
        .args(&args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run cli")
}

#[test]
fn setup_files_cli_greenfield_envelope_matches() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_greenfield(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &[]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_greenfield()
    );
    // Files actually created.
    assert!(project.path().join("CLAUDE.md").exists());
    assert!(project.path().join(".orbit/METHOD.md").exists());
    assert!(project.path().join(".orbit/STYLE.md").exists());
}

#[test]
fn setup_files_cli_legacy_migrate_envelope_matches() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_legacy(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &["--answer-legacy", "y"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_legacy_migrate()
    );
    // Legacy markers stripped; canonicals copied; @-imports present.
    let claude_md = std::fs::read_to_string(project.path().join("CLAUDE.md")).unwrap();
    assert!(!claude_md.contains("## Workflow (orbit)"), "legacy marker not stripped: {claude_md}");
    assert!(!claude_md.contains("## Orbit vocabulary"));
    assert!(!claude_md.contains("## Current Sprint"));
    assert!(claude_md.contains("## Keep this section"));
    assert!(claude_md.contains("@.orbit/METHOD.md"));
    assert!(claude_md.contains("@.orbit/STYLE.md"));
}

#[test]
fn setup_files_cli_legacy_refuse_errors_and_writes_nothing() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_legacy(project.path());
    let before = snapshot_layout_paths(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &["--answer-legacy", "n"]);
    assert!(!output.status.success(), "refuse must exit non-zero");
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_legacy_refuse()
    );
    // Filesystem-snapshot equality — no disk writes from the verb.
    let after = snapshot_layout_paths(project.path());
    assert_eq!(before, after, "refuse must leave the filesystem unchanged");
}

#[test]
fn setup_files_cli_method_drift_overwrite_envelope_matches() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_method_drift(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &["--answer-method-drift", "y"]);
    assert!(output.status.success(), "stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_method_overwrite()
    );
    // METHOD.md replaced with canonical.
    let method = std::fs::read_to_string(project.path().join(".orbit/METHOD.md")).unwrap();
    assert_eq!(method, common::FIXTURE_CANONICAL_METHOD);
}

#[test]
fn setup_files_cli_method_drift_keep_envelope_matches() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_method_drift(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &["--answer-method-drift", "n"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_method_keep()
    );
    // METHOD.md unchanged.
    let method = std::fs::read_to_string(project.path().join(".orbit/METHOD.md")).unwrap();
    assert_eq!(method, "# Local edited METHOD\n");
}

#[test]
fn setup_files_cli_style_drift_overwrite_envelope_matches() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_style_drift(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &["--answer-style-drift", "y"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_style_overwrite()
    );
    let style = std::fs::read_to_string(project.path().join(".orbit/STYLE.md")).unwrap();
    assert_eq!(style, common::FIXTURE_CANONICAL_STYLE);
}

#[test]
fn setup_files_cli_style_drift_keep_envelope_matches() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_style_drift(project.path());
    let output = run_setup_files_cli(project.path(), plugin.path(), &["--answer-style-drift", "n"]);
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim_end_matches('\n'),
        common::expected_envelope_for_setup_files_style_keep()
    );
    let style = std::fs::read_to_string(project.path().join(".orbit/STYLE.md")).unwrap();
    assert_eq!(style, "# Local edited STYLE\n");
}

#[test]
fn setup_files_cli_idempotent_filesystem_byte_equality() {
    let project = tempfile::tempdir().unwrap();
    let plugin = tempfile::tempdir().unwrap();
    common::populate_setup_files_greenfield(project.path());
    let _ = run_setup_files_cli(project.path(), plugin.path(), &[]);
    let after_first = snapshot_layout_paths(project.path());
    let claude_first = std::fs::read(project.path().join("CLAUDE.md")).unwrap();
    let method_first = std::fs::read(project.path().join(".orbit/METHOD.md")).unwrap();
    let style_first = std::fs::read(project.path().join(".orbit/STYLE.md")).unwrap();

    let _ = run_setup_files_cli(project.path(), plugin.path(), &[]);
    let after_second = snapshot_layout_paths(project.path());
    let claude_second = std::fs::read(project.path().join("CLAUDE.md")).unwrap();
    let method_second = std::fs::read(project.path().join(".orbit/METHOD.md")).unwrap();
    let style_second = std::fs::read(project.path().join(".orbit/STYLE.md")).unwrap();

    assert_eq!(after_first, after_second, "filesystem snapshot diverged across idempotent runs");
    assert_eq!(claude_first, claude_second, "CLAUDE.md bytes diverged across runs");
    assert_eq!(method_first, method_second, "METHOD.md bytes diverged");
    assert_eq!(style_first, style_second, "STYLE.md bytes diverged");
}
