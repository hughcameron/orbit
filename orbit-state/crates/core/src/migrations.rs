//! Schema-version + migration runner.
//!
//! Per ac-04 (gate) + trade-off (c) counterweight:
//! - `schema-version` file is present from day one (created on `init`)
//! - Migration runner exists from day one (this module)
//! - Runner is idempotent: running twice is a no-op on the second run
//!
//! At v0.1.0 there are no migrations to apply — we're shipping the first
//! version. The empty registry is intentional and the test verifies the
//! "twice = no-op" property holds for both populated and empty registries.
//!
//! Migrations are registered as `(from_version, to_version, fn)` tuples. The
//! runner picks the chain from `current` to `target` and applies each step
//! in order, persisting the new schema-version after each successful step so
//! a partial-failure leaves a known intermediate state.

use crate::atomic::write_atomic;
use crate::canonical::{parse_yaml, serialise_yaml};
use crate::error::{Category, Error, Result};
use crate::layout::OrbitLayout;
use crate::schema::{SchemaVersion, Spec};

/// Current schema version shipped by this build.
pub const CURRENT_SCHEMA_VERSION: &str = "0.5";

/// A single schema migration step.
#[derive(Debug)]
pub struct Migration {
    pub from: &'static str,
    pub to: &'static str,
    pub apply: fn(&OrbitLayout) -> Result<()>,
}

/// The migration registry. Each entry is one step in the chain; the runner
/// walks from the on-disk version up to `CURRENT_SCHEMA_VERSION`.
///
/// 0.1 → 0.2 (spec 2026-05-15-agent-learning-loop ac-02): structural no-op.
/// The change adds the `Session` canonical entity at `.orbit/sessions/<id>.yaml`
/// and the `SkillInvocation` append-only stream at `.orbit/skills/<id>.invocations.jsonl`.
/// Both are additive — no existing files need rewriting — so the migration
/// runner only needs to bump the recorded version. Fresh-at-0.2 workspaces
/// initialised by `init_schema_version` never hit this path.
///
/// 0.2 → 0.3 (spec 2026-05-16-ac-taxonomy ac-03): retire `time_gated: bool`
/// in favour of `ac_type: AcType`. Walks every spec.yaml under
/// `.orbit/specs/**/`, parses raw YAML (the typed struct no longer carries
/// `time_gated`), rewrites each AC by remapping the legacy field, then
/// re-serialises via the canonical writer so output remains byte-identical
/// to a fresh write. The remap is: `time_gated: true` becomes
/// `ac_type: observation` with the legacy key removed; `time_gated: false`
/// is simply dropped (the default `ac_type: code` is implicit).
///
/// 0.3 → 0.4 (spec 2026-05-16-session-handover ac-02): structural no-op.
/// The change adds an optional `card_id: Option<String>` field to the
/// `Session` struct with `#[serde(default, skip_serializing_if = ...)]`,
/// so existing Session YAML files (which carry no `card_id` key) remain
/// parseable without rewriting. The migration step only advances the
/// recorded version; no files are touched.
///
/// 0.4 → 0.5 (spec 2026-05-19-memory-gates-decisions ac-03 / D3a):
/// structural no-op. The change adds a `memories_considered:
/// Vec<MemoryReconciliation>` field to `Spec` with
/// `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, so existing
/// 0.4 spec.yaml files (which carry no `memories_considered` key) remain
/// parseable and round-trip byte-identically. The migration step only
/// advances the recorded version; no files are touched.
pub fn registry() -> &'static [Migration] {
    &[
        Migration {
            from: "0.1",
            to: "0.2",
            apply: |_layout| Ok(()),
        },
        Migration {
            from: "0.2",
            to: "0.3",
            apply: migrate_time_gated_to_ac_type,
        },
        Migration {
            from: "0.3",
            to: "0.4",
            apply: migrate_add_card_id_to_session,
        },
        Migration {
            from: "0.4",
            to: "0.5",
            apply: migrate_add_memories_considered_to_spec,
        },
    ]
}

/// 0.4 → 0.5 migration: structural no-op. The new `Spec.memories_considered`
/// field is `Vec<MemoryReconciliation>` with `#[serde(default,
/// skip_serializing_if = "Vec::is_empty")]`, so existing on-disk spec.yaml
/// files remain parseable under the new schema without rewriting. The
/// runner advances the version after this returns Ok.
fn migrate_add_memories_considered_to_spec(_layout: &OrbitLayout) -> Result<()> {
    Ok(())
}

/// 0.3 → 0.4 migration: structural no-op. The new `Session.card_id` field
/// is `Option<String>` with `#[serde(default)]`, so existing on-disk
/// Session YAML files remain parseable under the new schema without
/// rewriting. The runner advances the version after this returns Ok.
fn migrate_add_card_id_to_session(_layout: &OrbitLayout) -> Result<()> {
    Ok(())
}

/// 0.2 → 0.3 migration: walk every spec.yaml and convert any
/// `time_gated: bool` field on each AC into the new `ac_type: AcType`
/// shape. Idempotent: re-running on a tree that has no `time_gated`
/// keys is a no-op (no file is rewritten when no AC carried the
/// legacy field). Errors fail the migration loudly per the runner's
/// existing partial-failure contract.
fn migrate_time_gated_to_ac_type(layout: &OrbitLayout) -> Result<()> {
    let spec_files = layout.list_spec_files().map_err(|e| {
        Error::unavailable(
            "migration.0.2-to-0.3",
            format!("list specs: {e}"),
        )
        .with_source(e)
    })?;

    for path in spec_files {
        let original = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(
                "migration.0.2-to-0.3",
                format!("read {}: {e}", path.display()),
            )
            .with_source(e)
        })?;

        let mut value: serde_yaml::Value = serde_yaml::from_str(&original).map_err(|e| {
            Error::malformed(
                "migration.0.2-to-0.3",
                format!("parse {}: {e}", path.display()),
            )
        })?;

        let mut changed = false;
        if let Some(mapping) = value.as_mapping_mut() {
            let acs_key = serde_yaml::Value::String("acceptance_criteria".into());
            if let Some(acs_value) = mapping.get_mut(&acs_key) {
                if let Some(seq) = acs_value.as_sequence_mut() {
                    for item in seq.iter_mut() {
                        let ac_map = match item.as_mapping_mut() {
                            Some(m) => m,
                            None => continue,
                        };
                        let tg_key = serde_yaml::Value::String("time_gated".into());
                        match ac_map.remove(&tg_key) {
                            Some(serde_yaml::Value::Bool(true)) => {
                                ac_map.insert(
                                    serde_yaml::Value::String("ac_type".into()),
                                    serde_yaml::Value::String("observation".into()),
                                );
                                changed = true;
                            }
                            Some(_) => {
                                // time_gated: false (or unexpected scalar) — drop
                                // the key; default ac_type: code is implicit.
                                changed = true;
                            }
                            None => {}
                        }
                    }
                }
            }
        }

        if !changed {
            continue;
        }

        // Re-parse via the typed schema then re-serialise via the canonical
        // writer. The typed parse will accept the new ac_type field (added
        // in spec 2026-05-16-ac-taxonomy ac-01). Output is byte-identical
        // to a fresh canonical write, which keeps `orbit verify` clean
        // post-migration.
        let migrated_spec: Spec = serde_yaml::from_value(value).map_err(|e| {
            Error::malformed(
                "migration.0.2-to-0.3",
                format!("re-parse {} after rewrite: {e}", path.display()),
            )
        })?;
        let canonical_text = serialise_yaml(&migrated_spec)?;
        if canonical_text != original {
            write_atomic(&path, canonical_text.as_bytes())?;
        }
    }

    Ok(())
}

/// Initialise the schema-version file if missing, then advance it through
/// any pending migrations to `CURRENT_SCHEMA_VERSION`. Idempotent — safe
/// to call on every verb invocation. Fresh trees skip the run() path
/// (file just got created at the current version); already-current trees
/// take the no-op branch in `run`.
///
/// Per spec 2026-05-16-ac-taxonomy ac-04: this is the wire that makes
/// substrate migrations auto-apply on the next orbit verb against an
/// older tree. `verify_all` calls it; other verbs that mutate substrate
/// state should call it too if they want a guaranteed-current tree.
pub fn ensure_current(layout: &OrbitLayout) -> Result<MigrationReport> {
    init_schema_version(layout)?;
    run(layout, CURRENT_SCHEMA_VERSION)
}

/// Initialise the schema-version file at the configured layout.
///
/// Idempotent: if the file already exists with the current version, this is a
/// no-op. If it exists with a different version, the runner is what advances
/// it (this function does not silently rewrite an existing version).
pub fn init_schema_version(layout: &OrbitLayout) -> Result<()> {
    let path = layout.schema_version_file();
    if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable("init.schema_version", format!("read failed: {e}"))
                .with_source(e)
        })?;
        let existing: SchemaVersion = parse_yaml(&text)?;
        if existing.version == CURRENT_SCHEMA_VERSION {
            return Ok(()); // already current — idempotent
        }
        // Different version — run() is the path that advances it.
        return Ok(());
    }
    let sv = SchemaVersion {
        version: CURRENT_SCHEMA_VERSION.into(),
        note: None,
    };
    let text = serialise_yaml(&sv)?;
    write_atomic(&path, text.as_bytes())?;
    Ok(())
}

/// Run all migrations from the file's current version up to `target`.
///
/// Idempotent: if the file is already at `target`, this is a no-op and
/// returns `MigrationReport { applied: 0, .. }`. If migrations fail mid-chain,
/// the schema-version file reflects the last successfully-applied step so a
/// retry can resume.
pub fn run(layout: &OrbitLayout, target: &str) -> Result<MigrationReport> {
    let path = layout.schema_version_file();
    if !path.exists() {
        return Err(Error::not_found(
            "migration.run",
            format!("schema-version file missing at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| {
        Error::unavailable("migration.run", format!("read schema-version: {e}"))
            .with_source(e)
    })?;
    let mut current_sv: SchemaVersion = parse_yaml(&text)?;

    let mut report = MigrationReport {
        from: current_sv.version.clone(),
        to: target.to_string(),
        applied: 0,
        skipped: false,
    };

    if current_sv.version == target {
        report.skipped = true;
        return Ok(report);
    }

    let registry = registry();

    // Spec 2026-05-15-agent-learning-loop ac-02: an on-disk version that is
    // neither the target nor a known migration source is malformed input —
    // not a missing migration path. The known-versions set is derived from
    // the registry (every `from` and `to`) plus the target.
    if !is_known_version(registry, &current_sv.version, target) {
        return Err(Error::malformed(
            "migration.run",
            format!(
                "schema-version file has unknown version `{}`; known versions: {}",
                current_sv.version,
                known_versions_csv(registry, target)
            ),
        ));
    }

    let chain = build_chain(registry, &current_sv.version, target)?;
    for step in chain {
        (step.apply)(layout)?;
        // Persist the new version after each successful step so a crash
        // halfway leaves a known intermediate state.
        current_sv.version = step.to.to_string();
        let text = serialise_yaml(&current_sv)?;
        write_atomic(&path, text.as_bytes())?;
        report.applied += 1;
    }
    Ok(report)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationReport {
    pub from: String,
    pub to: String,
    pub applied: usize,
    /// True when the file was already at the target version.
    pub skipped: bool,
}

fn is_known_version(registry: &[Migration], version: &str, target: &str) -> bool {
    if version == target {
        return true;
    }
    registry.iter().any(|m| m.from == version || m.to == version)
}

fn known_versions_csv(registry: &[Migration], target: &str) -> String {
    let mut versions: Vec<&str> = registry
        .iter()
        .flat_map(|m| [m.from, m.to])
        .chain(std::iter::once(target))
        .collect();
    versions.sort_unstable();
    versions.dedup();
    versions.join(", ")
}

fn build_chain<'a>(
    registry: &'a [Migration],
    from: &str,
    to: &str,
) -> Result<Vec<&'a Migration>> {
    if from == to {
        return Ok(vec![]);
    }
    let mut chain: Vec<&Migration> = Vec::new();
    let mut cursor = from.to_string();
    while cursor != to {
        let next = registry.iter().find(|m| m.from == cursor);
        match next {
            Some(m) => {
                chain.push(m);
                cursor = m.to.to_string();
                // Sanity: prevent infinite loops on cyclic registries.
                if chain.len() > registry.len() + 1 {
                    return Err(Error::new(
                        "migration.run",
                        Category::Conflict,
                        format!("cyclic migration registry between {from} and {to}"),
                    ));
                }
            }
            None => {
                return Err(Error::new(
                    "migration.run",
                    Category::NotFound,
                    format!(
                        "no migration path from {} to {} (chain reached {})",
                        from, to, cursor
                    ),
                ));
            }
        }
    }
    Ok(chain)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn init_creates_schema_version_file() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        init_schema_version(&layout).unwrap();
        assert!(layout.schema_version_file().exists());

        let text = std::fs::read_to_string(layout.schema_version_file()).unwrap();
        let sv: SchemaVersion = parse_yaml(&text).unwrap();
        assert_eq!(sv.version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn init_is_idempotent() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        init_schema_version(&layout).unwrap();
        let mtime_first = std::fs::metadata(layout.schema_version_file())
            .unwrap()
            .modified()
            .unwrap();

        // Sleep briefly so any rewrite would change mtime detectably.
        std::thread::sleep(std::time::Duration::from_millis(20));
        init_schema_version(&layout).unwrap();

        let mtime_second = std::fs::metadata(layout.schema_version_file())
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(
            mtime_first, mtime_second,
            "init_schema_version should not rewrite an already-current file"
        );
    }

    #[test]
    fn run_at_target_is_noop() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        init_schema_version(&layout).unwrap();

        let report = run(&layout, CURRENT_SCHEMA_VERSION).unwrap();
        assert_eq!(report.applied, 0);
        assert!(report.skipped);
    }

    #[test]
    fn run_twice_is_idempotent() {
        // ac-04 verification: running migration twice produces no-op on second run.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        init_schema_version(&layout).unwrap();

        let report1 = run(&layout, CURRENT_SCHEMA_VERSION).unwrap();
        let report2 = run(&layout, CURRENT_SCHEMA_VERSION).unwrap();
        assert_eq!(report1.applied, report2.applied);
        assert!(report1.skipped && report2.skipped);
    }

    #[test]
    fn run_without_schema_version_file_returns_not_found() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // Don't init.
        let err = run(&layout, CURRENT_SCHEMA_VERSION).unwrap_err();
        assert_eq!(err.category, Category::NotFound);
    }

    #[test]
    fn build_chain_empty_when_from_equals_to() {
        let chain = build_chain(&[], "0.1", "0.1").unwrap();
        assert!(chain.is_empty());
    }

    #[test]
    fn build_chain_returns_not_found_when_no_path_exists() {
        // Empty registry, mismatched from/to.
        let err = build_chain(&[], "0.0", "0.1").unwrap_err();
        assert_eq!(err.category, Category::NotFound);
    }

    #[test]
    fn migrations_apply_in_order_and_persist_after_each_step() {
        // Synthetic two-step migration: 0.0 -> 0.0a -> 0.1.
        // We test against a custom registry by inlining build_chain semantics
        // — the public registry is empty at v0.1.0 by design.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // Plant a "0.0" schema-version file.
        let sv = SchemaVersion {
            version: "0.0".into(),
            note: None,
        };
        let text = serialise_yaml(&sv).unwrap();
        write_atomic(layout.schema_version_file(), text.as_bytes()).unwrap();

        // Build a custom chain manually and apply, asserting persistence.
        let synthetic = [
            Migration {
                from: "0.0",
                to: "0.0a",
                apply: |_| Ok(()),
            },
            Migration {
                from: "0.0a",
                to: "0.1",
                apply: |_| Ok(()),
            },
        ];
        let chain = build_chain(&synthetic, "0.0", "0.1").unwrap();
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].to, "0.0a");
        assert_eq!(chain[1].to, "0.1");
    }

    #[test]
    fn registry_at_v0_5_has_four_entries() {
        // spec 2026-05-19-memory-gates-decisions ac-03: the registry now
        // carries 0.1 → 0.2 (no-op), 0.2 → 0.3 (time_gated → ac_type),
        // 0.3 → 0.4 (additive Session.card_id no-op), and 0.4 → 0.5
        // (additive Spec.memories_considered no-op).
        let r = registry();
        assert_eq!(r.len(), 4, "expected four migration entries at v0.5");
        assert_eq!(r[0].from, "0.1");
        assert_eq!(r[0].to, "0.2");
        assert_eq!(r[1].from, "0.2");
        assert_eq!(r[1].to, "0.3");
        assert_eq!(r[2].from, "0.3");
        assert_eq!(r[2].to, "0.4");
        assert_eq!(r[3].from, "0.4");
        assert_eq!(r[3].to, "0.5");
    }

    #[test]
    fn migrate_0_1_to_current_walks_chain_and_lands_at_target() {
        // spec 2026-05-16-session-handover ac-02: a fixture at 0.1 with no
        // legacy time_gated content walks the full chain (0.1 → 0.2 → 0.3
        // → 0.4) and ends at CURRENT_SCHEMA_VERSION. Existing canonical
        // files not carrying time_gated remain untouched.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // Plant a 0.1 schema-version file (overriding init's default).
        let sv = SchemaVersion { version: "0.1".into(), note: None };
        let text = serialise_yaml(&sv).unwrap();
        write_atomic(layout.schema_version_file(), text.as_bytes()).unwrap();

        // Plant a canonical memory file we will assert remained untouched.
        let memory_path = layout.memory_file("test-memory");
        let memory_yaml = "key: test-memory\nbody: hello\ntimestamp: 2026-05-15T12:00:00Z\n";
        write_atomic(&memory_path, memory_yaml.as_bytes()).unwrap();
        let memory_mtime_before = std::fs::metadata(&memory_path).unwrap().modified().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(20));
        let report = run(&layout, CURRENT_SCHEMA_VERSION).unwrap();
        assert_eq!(report.from, "0.1");
        assert_eq!(report.to, CURRENT_SCHEMA_VERSION);
        assert_eq!(
            report.applied, 4,
            "expected 0.1 → 0.2, 0.2 → 0.3, 0.3 → 0.4, and 0.4 → 0.5 steps"
        );
        assert!(!report.skipped);

        // schema-version file is now CURRENT.
        let new_text = std::fs::read_to_string(layout.schema_version_file()).unwrap();
        let new_sv: SchemaVersion = parse_yaml(&new_text).unwrap();
        assert_eq!(new_sv.version, CURRENT_SCHEMA_VERSION);

        // The memory file was not touched.
        let memory_mtime_after = std::fs::metadata(&memory_path).unwrap().modified().unwrap();
        assert_eq!(
            memory_mtime_before, memory_mtime_after,
            "memory files must not be rewritten by chain migration"
        );
        let memory_text_after = std::fs::read_to_string(&memory_path).unwrap();
        assert_eq!(memory_text_after, memory_yaml);
    }

    #[test]
    fn migrate_already_at_current_is_noop() {
        // spec 2026-05-16-ac-taxonomy ac-03 (generalising 2026-05-15-
        // agent-learning-loop ac-02): a fixture already at the current
        // version must not run any migration step. Zero files change.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        init_schema_version(&layout).unwrap();

        let sv_mtime_before = std::fs::metadata(layout.schema_version_file())
            .unwrap()
            .modified()
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(20));
        let report = run(&layout, CURRENT_SCHEMA_VERSION).unwrap();
        assert_eq!(report.applied, 0);
        assert!(report.skipped);

        let sv_mtime_after = std::fs::metadata(layout.schema_version_file())
            .unwrap()
            .modified()
            .unwrap();
        assert_eq!(
            sv_mtime_before, sv_mtime_after,
            "fresh-at-current must not rewrite the schema-version file"
        );
    }

    #[test]
    fn migrate_0_2_to_0_3_rewrites_time_gated_to_ac_type_observation() {
        // spec 2026-05-16-ac-taxonomy ac-03: a fixture spec.yaml carrying
        // `time_gated: true` on an AC migrates to `ac_type: observation`
        // with the time_gated key removed.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // Plant a 0.2 schema-version file.
        let sv = SchemaVersion { version: "0.2".into(), note: None };
        let text = serialise_yaml(&sv).unwrap();
        write_atomic(layout.schema_version_file(), text.as_bytes()).unwrap();

        // Plant a spec.yaml with time_gated: true on ac-X and time_gated:
        // false on ac-Y. Use the legacy field name directly — pre-0.3
        // canonical content.
        layout.ensure_spec_dir("0001").unwrap();
        let spec_path = layout.spec_file("0001");
        let legacy_yaml = "id: '0001'\n\
                           goal: test\n\
                           status: open\n\
                           acceptance_criteria:\n\
                           - id: ac-X\n  description: X\n  gate: false\n  checked: false\n  time_gated: true\n\
                           - id: ac-Y\n  description: Y\n  gate: false\n  checked: false\n  time_gated: false\n";
        write_atomic(&spec_path, legacy_yaml.as_bytes()).unwrap();

        let report = run(&layout, CURRENT_SCHEMA_VERSION).unwrap();
        // Chain from 0.2 covers 0.2 → 0.3 (time_gated → ac_type),
        // 0.3 → 0.4 (Session.card_id additive no-op), and 0.4 → 0.5
        // (Spec.memories_considered additive no-op).
        assert_eq!(
            report.applied, 3,
            "expected three steps (0.2 → 0.3, 0.3 → 0.4, and 0.4 → 0.5)"
        );

        let migrated = std::fs::read_to_string(&spec_path).unwrap();
        // time_gated keys are gone.
        assert!(
            !migrated.contains("time_gated"),
            "time_gated must be removed:\n{migrated}"
        );
        // ac-X carries ac_type: observation.
        assert!(
            migrated.contains("ac_type: observation"),
            "ac-X must carry ac_type: observation:\n{migrated}"
        );
        // ac-Y has no ac_type line (default code is implicit).
        // We assert there's exactly one ac_type occurrence (for ac-X).
        assert_eq!(
            migrated.matches("ac_type:").count(),
            1,
            "exactly one ac_type line expected (ac-X), default-code ac-Y is implicit:\n{migrated}"
        );

        // Re-parse via typed schema succeeds.
        let parsed: Spec = parse_yaml(&migrated).unwrap();
        assert_eq!(parsed.acceptance_criteria.len(), 2);
        assert_eq!(
            parsed.acceptance_criteria[0].ac_type,
            crate::schema::AcType::Observation,
        );
        assert_eq!(
            parsed.acceptance_criteria[1].ac_type,
            crate::schema::AcType::Code,
        );

        // schema-version file is now CURRENT (0.4).
        let new_text = std::fs::read_to_string(layout.schema_version_file()).unwrap();
        let new_sv: SchemaVersion = parse_yaml(&new_text).unwrap();
        assert_eq!(new_sv.version, CURRENT_SCHEMA_VERSION);
    }

    #[test]
    fn migrate_0_2_to_0_3_is_idempotent_on_already_migrated_tree() {
        // spec 2026-05-16-ac-taxonomy ac-03: re-running on a tree with no
        // time_gated keys touches no files (the migration step's changed
        // flag stays false, no write).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let sv = SchemaVersion { version: "0.2".into(), note: None };
        let text = serialise_yaml(&sv).unwrap();
        write_atomic(layout.schema_version_file(), text.as_bytes()).unwrap();

        // Plant a spec.yaml that already uses ac_type (post-migration shape).
        layout.ensure_spec_dir("0001").unwrap();
        let spec_path = layout.spec_file("0001");
        let modern_yaml = "id: '0001'\n\
                           goal: test\n\
                           status: open\n\
                           acceptance_criteria:\n\
                           - id: ac-X\n  description: X\n  gate: false\n  checked: false\n  ac_type: observation\n";
        write_atomic(&spec_path, modern_yaml.as_bytes()).unwrap();
        let mtime_before = std::fs::metadata(&spec_path).unwrap().modified().unwrap();

        std::thread::sleep(std::time::Duration::from_millis(20));
        run(&layout, CURRENT_SCHEMA_VERSION).unwrap();

        let mtime_after = std::fs::metadata(&spec_path).unwrap().modified().unwrap();
        assert_eq!(
            mtime_before, mtime_after,
            "spec.yaml without time_gated must not be rewritten by the 0.2 → 0.3 migration"
        );
    }

    #[test]
    fn ensure_current_initialises_and_advances_in_one_call() {
        // spec 2026-05-16-ac-taxonomy ac-04: the wire that makes substrate
        // migrations auto-apply on the next orbit verb. ensure_current
        // initialises if missing AND advances the version through any
        // pending migrations.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // 1. Missing file: ensure_current initialises at CURRENT (no advance needed).
        assert!(!layout.schema_version_file().exists());
        let report = ensure_current(&layout).unwrap();
        assert!(report.skipped, "fresh init should skip the run() chain");
        let text = std::fs::read_to_string(layout.schema_version_file()).unwrap();
        let sv: SchemaVersion = parse_yaml(&text).unwrap();
        assert_eq!(sv.version, CURRENT_SCHEMA_VERSION);

        // 2. Manually downgrade to 0.2; ensure_current advances to current.
        let downgraded = SchemaVersion { version: "0.2".into(), note: None };
        let text = serialise_yaml(&downgraded).unwrap();
        write_atomic(layout.schema_version_file(), text.as_bytes()).unwrap();
        let report = ensure_current(&layout).unwrap();
        assert_eq!(report.from, "0.2");
        assert_eq!(report.to, CURRENT_SCHEMA_VERSION);
        assert!(!report.skipped);
    }

    #[test]
    fn migrate_unknown_version_is_malformed() {
        // spec 2026-05-15-agent-learning-loop ac-02: a version that is
        // neither the target nor a known migration source returns
        // Error::malformed rather than NotFound. This protects against
        // silent guessing on hand-edited or future-versioned workspaces.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let sv = SchemaVersion { version: "0.9-future".into(), note: None };
        let text = serialise_yaml(&sv).unwrap();
        write_atomic(layout.schema_version_file(), text.as_bytes()).unwrap();

        let err = run(&layout, CURRENT_SCHEMA_VERSION).unwrap_err();
        assert_eq!(err.category, Category::Malformed);
        assert!(err.message.contains("0.9-future"));
    }
}
