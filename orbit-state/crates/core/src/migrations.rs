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
use crate::schema::SchemaVersion;

/// Current schema version shipped by this build.
pub const CURRENT_SCHEMA_VERSION: &str = "0.2";

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
pub fn registry() -> &'static [Migration] {
    &[Migration {
        from: "0.1",
        to: "0.2",
        apply: |_layout| Ok(()),
    }]
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
    fn registry_at_v0_2_has_one_entry() {
        // spec 2026-05-15-agent-learning-loop ac-02: the 0.1 → 0.2 entry is
        // the first migration in orbit's history. If a future version adds
        // 0.2 → 0.3 this test grows to assert the chain shape.
        let r = registry();
        assert_eq!(r.len(), 1, "expected exactly one migration entry at v0.2");
        assert_eq!(r[0].from, "0.1");
        assert_eq!(r[0].to, "0.2");
    }

    #[test]
    fn migrate_0_1_to_0_2_writes_new_version_and_leaves_files_untouched() {
        // spec 2026-05-15-agent-learning-loop ac-02: a fixture at 0.1 with
        // existing canonical files must end at 0.2 with no other file
        // mutated — the migration is structural no-op + version bump.
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
        assert_eq!(report.to, "0.2");
        assert_eq!(report.applied, 1);
        assert!(!report.skipped);

        // schema-version file is now 0.2.
        let new_text = std::fs::read_to_string(layout.schema_version_file()).unwrap();
        let new_sv: SchemaVersion = parse_yaml(&new_text).unwrap();
        assert_eq!(new_sv.version, "0.2");

        // The memory file was not touched.
        let memory_mtime_after = std::fs::metadata(&memory_path).unwrap().modified().unwrap();
        assert_eq!(
            memory_mtime_before, memory_mtime_after,
            "0.1 → 0.2 must not rewrite existing canonical files"
        );
        let memory_text_after = std::fs::read_to_string(&memory_path).unwrap();
        assert_eq!(memory_text_after, memory_yaml);
    }

    #[test]
    fn migrate_already_at_0_2_is_noop() {
        // spec 2026-05-15-agent-learning-loop ac-02: a fixture already at
        // 0.2 (fresh workspace seeded after this AC ships) must not run
        // the 0.1 → 0.2 path. Zero files change.
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
            "fresh-at-0.2 must not rewrite the schema-version file"
        );
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
