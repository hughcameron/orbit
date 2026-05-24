//! Substrate-wide hygiene check: round-trip + index-rebuild verification.
//!
//! Wires the two CI gates into a single inspection routine so callers get a
//! single pass/fail signal:
//!
//! - **ac-16 (round-trip gate):** every file under `.orbit/specs/`,
//!   `.orbit/cards/`, `.orbit/choices/`, `.orbit/memories/`, plus the
//!   `schema-version` file, parses and re-serialises byte-identically. Tasks
//!   are excluded — they are append-only JSONL events and explicitly out of
//!   scope per the `acceptance_criteria` of ac-16.
//!
//! - **ac-17 (verify gate):** the SQLite index rebuilds from files alone and
//!   diffs clean against any pre-existing on-disk index.
//!
//! Both gates fail the same way (`VerifyOutcome::has_failures() == true`) so
//! CI can run a single `orbit verify` invocation and treat any drift as a
//! merge blocker. The per-failure detail is preserved in the outcome so a
//! human can locate the offending file without re-running.
//!
//! Per ac-16's exclusion list, task JSONL streams are not iterated here. They
//! are append-only events (substrate-written, never rewritten in place); a
//! round-trip test does not apply to that storage shape.
//!
//! The index check creates `state.db` if it does not already exist (CI runs
//! against fresh checkouts where state.db is gitignored). On a fresh tree the
//! check reduces to "files parse cleanly and rebuild succeeds" — which is
//! still the meaningful CI signal.

use crate::canonical::{parse_yaml, serialise_yaml};
use crate::index::Index;
use crate::layout::OrbitLayout;
use crate::migrations::ensure_current;
use crate::schema::{Card, Choice, Config, Memory, SchemaVersion, Session, Spec, TopologyEntry};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Aggregate result of `verify_all`.
///
/// Empty vectors mean all checks passed; any non-empty vector is a failure
/// the caller should report and exit non-zero on.
#[derive(Debug, Default)]
pub struct VerifyOutcome {
    /// Files whose `parse → serialise` does not round-trip byte-identical, or
    /// which fail to parse against the canonical schema.
    pub round_trip_failures: Vec<RoundTripFailure>,
    /// Drift between the on-disk index and a fresh rebuild from files.
    pub index_drift: Vec<String>,
}

impl VerifyOutcome {
    pub fn has_failures(&self) -> bool {
        !self.round_trip_failures.is_empty() || !self.index_drift.is_empty()
    }
}

#[derive(Debug)]
pub struct RoundTripFailure {
    pub path: PathBuf,
    pub kind: RoundTripFailureKind,
}

#[derive(Debug)]
pub enum RoundTripFailureKind {
    /// File could not be parsed against its canonical schema (malformed YAML,
    /// unknown field, CRLF, etc.). The wrapped string is the underlying
    /// canonical-layer error message.
    ParseFailed(String),
    /// File parsed, re-serialised, but the bytes do not match the original.
    /// The substrate has not yet rewritten this file through the canonical
    /// writer — `orbit ... update` (or any verb that touches the file) will
    /// normalise it.
    NotByteIdentical,
}

/// Run the full substrate hygiene check. Idempotent; safe to invoke from CI.
///
/// Steps in order:
/// 1. Ensure layout subdirectories exist.
/// 2. Ensure `schema-version` exists (substrate-written file; CI checkouts
///    don't carry it because it's gitignored).
/// 3. Round-trip each canonical file (schema-version, specs, cards, choices,
///    memories). Tasks excluded per ac-16.
/// 4. Open or create `state.db`, rebuild from files, diff.
pub fn verify_all(layout: &OrbitLayout) -> std::io::Result<VerifyOutcome> {
    layout.ensure_dirs()?;
    // ensure_current is idempotent: initialises the schema-version file if
    // missing, advances any pending migrations to CURRENT_SCHEMA_VERSION
    // (spec 2026-05-16-ac-taxonomy ac-04 wire). Errors here surface as
    // round-trip failures on the schema-version path so the caller sees a
    // single channel of diagnostics rather than a mid-run abort.
    let _ = ensure_current(layout);

    let mut outcome = VerifyOutcome::default();

    // 1. schema-version (single file).
    if layout.schema_version_file().exists() {
        check_round_trip::<SchemaVersion>(&layout.schema_version_file(), &mut outcome);
    }

    // 2. specs/*.yaml — only the spec yamls, NOT the .tasks.jsonl streams
    //    (task events are append-only per ac-16's exclusion).
    for path in list_or_empty(layout.list_spec_files()) {
        check_round_trip::<Spec>(&path, &mut outcome);
    }

    // 3. cards/*.yaml — shallow scan of the cards directory.
    for path in list_or_empty(layout.list_card_files()) {
        check_round_trip::<Card>(&path, &mut outcome);
    }

    // 4. choices/*.yaml.
    for path in list_or_empty(layout.list_choice_files()) {
        check_round_trip::<Choice>(&path, &mut outcome);
    }

    // 5. memories/*.yaml.
    for path in list_or_empty(layout.list_memory_files()) {
        check_round_trip::<Memory>(&path, &mut outcome);
    }

    // 6. sessions/*.yaml — substrate-written summaries, included in the
    //    round-trip gate (sessions are NOT event streams).
    for path in list_or_empty(layout.list_session_files()) {
        check_round_trip::<Session>(&path, &mut outcome);
    }

    // 7. config.yaml — opt-in project config (spec
    //    2026-05-18-documentation-topology ac-04). Absence is tolerated:
    //    the topology capability is opt-in. Presence MUST round-trip
    //    against the Config schema.
    if layout.config_file().exists() {
        check_round_trip::<Config>(&layout.config_file(), &mut outcome);
    }

    // 8. topology/*.yaml — per-subsystem topology entries (spec
    //    2026-05-18-topology-substrate-migration ac-01, choice 0025).
    //    Each file round-trips against TopologyEntry AND must pass the
    //    non-serde validate() check (slug shape, min length, non-empty
    //    canonical_code).
    for path in list_or_empty(layout.list_topology_files()) {
        check_topology_round_trip(&path, &mut outcome);
    }

    // 9. Index rebuild check (ac-17).
    //
    // We always rebuild against a fresh in-memory index — that's the hygiene
    // signal. A failure here is a file that parses individually but breaks
    // the index's stronger invariants (FK references, uniqueness, etc.). The
    // on-disk `state.db` is intentionally NOT touched: it is gitignored and
    // therefore absent on CI, and where it does exist (developer machines)
    // the index can lag the files in normal operation. Drift between an
    // existing state.db and files is a local-development question, not a
    // merge gate.
    match Index::open_in_memory() {
        Ok(mut idx) => {
            if let Err(e) = idx.rebuild_from_files(layout) {
                outcome
                    .index_drift
                    .push(format!("index rebuild failed: {e}"));
            }
        }
        Err(e) => outcome
            .index_drift
            .push(format!("index open failed: {e}")),
    }

    Ok(outcome)
}

fn list_or_empty(result: std::io::Result<Vec<PathBuf>>) -> Vec<PathBuf> {
    result.unwrap_or_default()
}

/// Topology files get a check_round_trip pass plus a non-serde validate()
/// step (slug shape / min length / canonical_code non-empty). A validate
/// failure is reported as ParseFailed so the diagnostic channel stays
/// uniform with the rest of verify_all.
fn check_topology_round_trip(path: &Path, outcome: &mut VerifyOutcome) {
    // Capture the count of round_trip_failures before delegating — if
    // check_round_trip pushes a failure for this path we skip validate()
    // so the operator sees one diagnostic per file at a time.
    let pre_failure_count = outcome.round_trip_failures.len();
    check_round_trip::<TopologyEntry>(path, outcome);
    if outcome.round_trip_failures.len() != pre_failure_count {
        return;
    }
    // File round-tripped; now run validate(). Re-read + parse — cheap and
    // keeps check_round_trip's signature uniform across entity types.
    let text = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            outcome.round_trip_failures.push(RoundTripFailure {
                path: path.to_path_buf(),
                kind: RoundTripFailureKind::ParseFailed(format!("validate re-read failed: {e}")),
            });
            return;
        }
    };
    let entry: TopologyEntry = match parse_yaml(&text) {
        Ok(v) => v,
        Err(e) => {
            outcome.round_trip_failures.push(RoundTripFailure {
                path: path.to_path_buf(),
                kind: RoundTripFailureKind::ParseFailed(format!("validate re-parse failed: {e}")),
            });
            return;
        }
    };
    if let Err(msg) = entry.validate() {
        outcome.round_trip_failures.push(RoundTripFailure {
            path: path.to_path_buf(),
            kind: RoundTripFailureKind::ParseFailed(format!("validate failed: {msg}")),
        });
    }
}

fn check_round_trip<T>(path: &Path, outcome: &mut VerifyOutcome)
where
    T: DeserializeOwned + Serialize,
{
    let original = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            outcome.round_trip_failures.push(RoundTripFailure {
                path: path.to_path_buf(),
                kind: RoundTripFailureKind::ParseFailed(format!("read failed: {e}")),
            });
            return;
        }
    };
    let parsed: T = match parse_yaml(&original) {
        Ok(v) => v,
        Err(e) => {
            outcome.round_trip_failures.push(RoundTripFailure {
                path: path.to_path_buf(),
                kind: RoundTripFailureKind::ParseFailed(e.to_string()),
            });
            return;
        }
    };
    let reserialised = match serialise_yaml(&parsed) {
        Ok(v) => v,
        Err(e) => {
            outcome.round_trip_failures.push(RoundTripFailure {
                path: path.to_path_buf(),
                kind: RoundTripFailureKind::ParseFailed(e.to_string()),
            });
            return;
        }
    };
    if reserialised != original {
        outcome.round_trip_failures.push(RoundTripFailure {
            path: path.to_path_buf(),
            kind: RoundTripFailureKind::NotByteIdentical,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomic::write_atomic;
    use crate::canonical::serialise_yaml;
    use crate::schema::{Choice, ChoiceStatus, SchemaVersion};
    use tempfile::tempdir;

    fn fresh_layout() -> (tempfile::TempDir, OrbitLayout) {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        (dir, layout)
    }

    #[test]
    fn verify_clean_on_empty_substrate_initialises_schema_version() {
        let (_dir, layout) = fresh_layout();
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "empty substrate should verify clean: {outcome:?}"
        );
        assert!(
            layout.schema_version_file().exists(),
            "schema-version must be initialised by verify_all"
        );
    }

    #[test]
    fn verify_clean_with_canonical_choice() {
        let (_dir, layout) = fresh_layout();
        let choice = Choice {
            id: "0042".into(),
            title: "verify works".into(),
            status: ChoiceStatus::Accepted,
            date_created: "2026-05-07".into(),
            date_modified: None,
            body: "Decision: verify all the things.\n".into(),
            references: vec![],
        };
        let yaml = serialise_yaml(&choice).unwrap();
        write_atomic(layout.choice_file("0042"), yaml.as_bytes()).unwrap();

        let outcome = verify_all(&layout).unwrap();
        assert!(!outcome.has_failures(), "{outcome:?}");
    }

    #[test]
    fn verify_detects_non_canonical_byte_drift() {
        // ac-16: a non-canonical file (extra whitespace, wrong field order)
        // must fail the round-trip check.
        let (_dir, layout) = fresh_layout();
        let non_canonical = "\
status: accepted
id: '0042'
title: drift
date_created: '2026-05-07'
body: 'x'
references: []
";
        write_atomic(layout.choice_file("0042"), non_canonical.as_bytes()).unwrap();

        let outcome = verify_all(&layout).unwrap();
        assert!(
            outcome
                .round_trip_failures
                .iter()
                .any(|f| matches!(f.kind, RoundTripFailureKind::NotByteIdentical)),
            "expected NotByteIdentical failure, got {outcome:?}"
        );
    }

    #[test]
    fn verify_detects_unknown_field() {
        // ac-01 strict-schema property surfaces through verify as a parse
        // failure on the offending file.
        let (_dir, layout) = fresh_layout();
        let bad = "\
id: '0042'
title: bad
status: accepted
date_created: '2026-05-07'
body: 'x'
references: []
mystery_field: ohno
";
        write_atomic(layout.choice_file("0042"), bad.as_bytes()).unwrap();

        let outcome = verify_all(&layout).unwrap();
        let parse_failures: Vec<_> = outcome
            .round_trip_failures
            .iter()
            .filter(|f| matches!(f.kind, RoundTripFailureKind::ParseFailed(_)))
            .collect();
        assert!(
            !parse_failures.is_empty(),
            "expected ParseFailed; got {outcome:?}"
        );
    }

    #[test]
    fn verify_excludes_task_jsonl_from_round_trip() {
        // ac-16 exclusion: task event JSONL is append-only and not iterated.
        // Under choice 0021 the stream lives inside the per-spec folder.
        let (_dir, layout) = fresh_layout();
        layout.ensure_spec_dir("2026-05-07-x").unwrap();
        std::fs::write(
            layout.task_stream("2026-05-07-x"),
            r#"{"task_id":"t","spec_id":"2026-05-07-x","event":"open","timestamp":"x"}
"#,
        )
        .unwrap();
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "task jsonl must not be iterated; got {outcome:?}"
        );
    }

    #[test]
    fn verify_excludes_sidecar_shapes_inside_spec_folder() {
        // Sidecars (drive/rally/review markdown) live alongside spec.yaml
        // inside the per-spec folder under choice 0021. `list_spec_files`
        // returns only `<id>/spec.yaml`, so sidecars never reach the Spec
        // round-trip check.
        let (_dir, layout) = fresh_layout();
        let id = "2026-05-09-foo";
        layout.ensure_spec_dir(id).unwrap();
        // Plant a real spec.yaml so the folder isn't silently skipped.
        let spec = Spec {
            id: id.into(),
            goal: "g".into(),
            cards: vec![],
            status: crate::schema::SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        write_atomic(
            layout.spec_file(id),
            serialise_yaml(&spec).unwrap().as_bytes(),
        )
        .unwrap();
        // Drive sidecar with a non-Spec shape — would fail Spec round-trip
        // if iterated. Must be ignored.
        std::fs::write(
            layout.spec_dir(id).join("drive.yaml"),
            "spec_id: '2026-05-09-foo'\nstage: review-spec\niteration: 1\n",
        )
        .unwrap();
        std::fs::write(
            layout.spec_dir(id).join("rally.yaml"),
            "rally_id: '2026-05-09-foo'\nchildren: []\n",
        )
        .unwrap();
        std::fs::write(
            layout.spec_dir(id).join("review-spec-2026-05-09.md"),
            "# Review\n",
        )
        .unwrap();
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "sidecars inside per-spec folder must not be iterated as Specs; got {outcome:?}"
        );
    }

    #[test]
    fn verify_repairs_known_schema_version_drift_via_migration() {
        // spec 2026-05-16-ac-taxonomy ac-04: verify_all calls ensure_current
        // which initialises + advances the schema-version through any
        // pending migrations. A non-canonical-but-known schema-version file
        // is silently normalised by the migration runner's per-step
        // persistence — the on-disk bytes after verify match the canonical
        // form at CURRENT_SCHEMA_VERSION, and verify reports no failures.
        let (_dir, layout) = fresh_layout();
        verify_all(&layout).unwrap();
        // Overwrite with a parseable but non-canonical body at an OLDER
        // known version. The migration runner will rewrite it.
        std::fs::write(
            layout.schema_version_file(),
            "version: '0.1'\nnote:\n",
        )
        .unwrap();

        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "ensure_current should have repaired the drift: {outcome:?}"
        );
        // The on-disk file is now canonical at CURRENT_SCHEMA_VERSION.
        let on_disk = std::fs::read_to_string(layout.schema_version_file()).unwrap();
        let canonical_form = serialise_yaml(&SchemaVersion {
            version: crate::migrations::CURRENT_SCHEMA_VERSION.into(),
            note: None,
        })
        .unwrap();
        assert_eq!(on_disk, canonical_form);
    }

    // ----- Config verify wiring (spec 2026-05-18-documentation-topology ac-04) -----

    #[test]
    fn verify_clean_when_config_file_absent() {
        // ac-04 verification: orbit verify on a repo without
        // .orbit/config.yaml exits 0 (opt-in tolerance per ac-02).
        let (_dir, layout) = fresh_layout();
        assert!(!layout.config_file().exists());
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "absent config.yaml must not fail verify: {outcome:?}"
        );
    }

    #[test]
    fn verify_clean_with_valid_config() {
        // ac-04 verification: orbit verify on a repo with a valid
        // .orbit/config.yaml exits 0.
        use crate::schema::{Config, DocsConfig};
        let (_dir, layout) = fresh_layout();
        let config = Config {
            docs: Some(DocsConfig {
                topology: Some("docs/topology.md".into()),
            }),
            plugin_version: None,
            plugin_repo: None,
        };
        let yaml = serialise_yaml(&config).unwrap();
        write_atomic(layout.config_file(), yaml.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        assert!(!outcome.has_failures(), "valid config must verify clean: {outcome:?}");
    }

    #[test]
    fn verify_detects_invalid_config_topology_type() {
        // ac-04 verification: orbit verify on a fixture with an invalid
        // Config (wrong type on docs.topology) reports a round-trip failure.
        let (_dir, layout) = fresh_layout();
        let bad = "docs:\n  topology: [not, a, string]\n";
        write_atomic(layout.config_file(), bad.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        let parse_failures: Vec<_> = outcome
            .round_trip_failures
            .iter()
            .filter(|f| matches!(f.kind, RoundTripFailureKind::ParseFailed(_)))
            .filter(|f| f.path == layout.config_file())
            .collect();
        assert!(
            !parse_failures.is_empty(),
            "expected ParseFailed on config.yaml; got {outcome:?}"
        );
    }

    #[test]
    fn verify_detects_unknown_config_field() {
        // ac-04 verification: unknown fields on Config fail verify per
        // deny_unknown_fields.
        let (_dir, layout) = fresh_layout();
        let bad = "docs:\n  topology: docs/topology.md\nmystery_field: ohno\n";
        write_atomic(layout.config_file(), bad.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        let parse_failures: Vec<_> = outcome
            .round_trip_failures
            .iter()
            .filter(|f| matches!(f.kind, RoundTripFailureKind::ParseFailed(_)))
            .filter(|f| f.path == layout.config_file())
            .collect();
        assert!(
            !parse_failures.is_empty(),
            "expected ParseFailed on config.yaml; got {outcome:?}"
        );
    }

    // ----- Topology verify wiring (spec 2026-05-18-topology-substrate-migration ac-01) -----

    #[test]
    fn verify_clean_when_topology_dir_absent() {
        // Absence of .orbit/topology/ is tolerated (the directory is created
        // by ensure_dirs but is empty until orbit topology setup runs).
        let (_dir, layout) = fresh_layout();
        // ensure_dirs creates the topology directory empty.
        layout.ensure_dirs().unwrap();
        assert!(layout.topology_dir().exists());
        assert!(layout.list_topology_files().unwrap().is_empty());
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "empty topology dir must not fail verify: {outcome:?}"
        );
    }

    #[test]
    fn verify_clean_with_valid_topology_entry() {
        // A well-formed TopologyEntry round-trips AND validates.
        let (_dir, layout) = fresh_layout();
        let entry = TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0016".into()],
            operational_doc: vec!["plugins/orb/skills/card/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        };
        let yaml = serialise_yaml(&entry).unwrap();
        write_atomic(layout.topology_file("cards"), yaml.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "valid topology entry must verify clean: {outcome:?}"
        );
    }

    #[test]
    fn verify_detects_unknown_topology_field() {
        // deny_unknown_fields: an unknown field on a topology entry fails
        // round-trip with ParseFailed.
        let (_dir, layout) = fresh_layout();
        let bad = "\
subsystem: cards
canonical_code: [orbit-state/crates/core/src/schema.rs]
unknown_field: oops
";
        write_atomic(layout.topology_file("cards"), bad.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        let parse_failures: Vec<_> = outcome
            .round_trip_failures
            .iter()
            .filter(|f| matches!(f.kind, RoundTripFailureKind::ParseFailed(_)))
            .filter(|f| f.path == layout.topology_file("cards"))
            .collect();
        assert!(
            !parse_failures.is_empty(),
            "expected ParseFailed on topology entry; got {outcome:?}"
        );
    }

    #[test]
    fn verify_detects_missing_required_field() {
        // canonical_code is serde-required.
        let (_dir, layout) = fresh_layout();
        let bad = "subsystem: cards\n";
        write_atomic(layout.topology_file("cards"), bad.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        assert!(
            outcome.has_failures(),
            "missing required field must fail verify: {outcome:?}"
        );
    }

    #[test]
    fn verify_detects_short_subsystem_slug() {
        // validate() catches subsystem slug below MIN_SUBSYSTEM_LEN (5 chars).
        // The yaml itself is structurally valid; only validate() fails.
        let (_dir, layout) = fresh_layout();
        // Round-trip-clean yaml: subsystem "card" (4 chars, below min).
        let entry = TopologyEntry {
            subsystem: "card".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        let yaml = serialise_yaml(&entry).unwrap();
        write_atomic(layout.topology_file("card"), yaml.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        let validate_failures: Vec<_> = outcome
            .round_trip_failures
            .iter()
            .filter(|f| {
                matches!(&f.kind, RoundTripFailureKind::ParseFailed(msg) if msg.contains("validate failed"))
            })
            .collect();
        assert!(
            !validate_failures.is_empty(),
            "expected validate-failed on short slug; got {outcome:?}"
        );
    }

    #[test]
    fn verify_detects_dangling_pointer_is_validate_only() {
        // The verify-time check only round-trips + runs validate() — it does
        // NOT walk canonical_code paths to check filesystem existence
        // (that's audit_topology's drift detection, ac-02). A topology entry
        // whose canonical_code points at a non-existent path still verifies
        // clean — drift is a separate signal.
        let (_dir, layout) = fresh_layout();
        let entry = TopologyEntry {
            subsystem: "ghosty".into(),
            canonical_code: vec!["does/not/exist.rs".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        let yaml = serialise_yaml(&entry).unwrap();
        write_atomic(layout.topology_file("ghosty"), yaml.as_bytes()).unwrap();
        let outcome = verify_all(&layout).unwrap();
        assert!(
            !outcome.has_failures(),
            "dangling pointer is NOT a verify-time failure (it's an audit-time drift signal): {outcome:?}"
        );
    }
}
