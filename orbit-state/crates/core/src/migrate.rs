//! One-shot migrations between substrate layouts.
//!
//! Currently hosts `migrate_spec_layout` per choice 0021: the flat-sidecar
//! spec layout (`.orbit/specs/<id>.yaml + <id>.<sidecar>`) reverts to per-spec
//! folders (`.orbit/specs/<id>/spec.yaml + <id>/<sidecar>`).
//!
//! These functions operate on raw filesystem paths intentionally — they do
//! NOT go through `OrbitLayout`'s spec_file/task_stream/etc helpers. The
//! migration must work BEFORE the helpers are updated to the new shape and
//! AFTER they have been updated, so it cannot depend on them.

use crate::layout::OrbitLayout;
use std::ffi::OsStr;
use std::path::PathBuf;

/// One planned move from a flat sidecar path to a folder path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMove {
    pub from: PathBuf,
    pub to: PathBuf,
}

#[derive(Debug, Default)]
pub struct MigrateReport {
    /// Specs migrated (folder created + files moved). One entry per spec id
    /// that was actually moved.
    pub migrated: Vec<String>,
    /// Specs that were already in folder shape — skipped (idempotency).
    pub already_folder: Vec<String>,
    /// Per-spec planned moves (populated regardless of dry_run).
    pub moves: Vec<PlannedMove>,
    /// Errors encountered during the migration.
    pub errors: Vec<(PathBuf, String)>,
}

/// Sidecar suffix patterns recognised by the migration. Anything matching
/// `<id>.<one-of-these>` is considered a sidecar belonging to `<id>` and
/// gets folded into `<id>/<one-of-these>` (with the leading dot stripped).
///
/// The list is explicit (not "anything after `<id>.`") to avoid the
/// prefix-collision risk choice 0021 names: spec ids that prefix each
/// other (`2026-05-09-foo` vs `2026-05-09-foo-bar`).
const SIDECAR_SUFFIXES: &[&str] = &[
    "drive.yaml",
    "rally.yaml",
    "notes.jsonl",
    "tasks.jsonl",
    "interview.md",
    "decisions.md",
];

/// Returns true if `name` matches a sidecar suffix exactly OR a review-cycle
/// pattern (`review-spec-<date>.md`, `review-spec-<date>-v2.md`,
/// `review-spec-<date>-v3.md`, same for `review-pr-`).
fn is_sidecar_suffix(name: &str) -> bool {
    if SIDECAR_SUFFIXES.iter().any(|s| *s == name) {
        return true;
    }
    is_review_suffix(name)
}

fn is_review_suffix(name: &str) -> bool {
    // review-spec-<date>.md, review-spec-<date>-v2.md, review-spec-<date>-v3.md,
    // review-pr-<date>.md, review-pr-<date>-v2.md, review-pr-<date>-v3.md.
    let stripped = match name.strip_suffix(".md") {
        Some(s) => s,
        None => return false,
    };
    for prefix in ["review-spec-", "review-pr-"] {
        if let Some(rest) = stripped.strip_prefix(prefix) {
            // rest is `<date>` or `<date>-v2` or `<date>-v3`.
            return is_review_date_with_optional_cycle(rest);
        }
    }
    false
}

fn is_review_date_with_optional_cycle(s: &str) -> bool {
    // `<date>` is YYYY-MM-DD (10 chars: 4-2-2 with hyphens). Optional
    // cycle suffix is `-v2` or `-v3`.
    let (date, cycle) = match s.rsplit_once("-v") {
        Some((d, c)) if c == "2" || c == "3" => (d, true),
        _ => (s, false),
    };
    let _ = cycle;
    let bytes = date.as_bytes();
    if bytes.len() != 10 {
        return false;
    }
    bytes.iter().enumerate().all(|(i, b)| match i {
        4 | 7 => *b == b'-',
        _ => b.is_ascii_digit(),
    })
}

/// Plan and (unless `dry_run`) execute the flat-sidecar → per-spec-folder
/// migration. Idempotent: specs already in folder shape are skipped.
pub fn migrate_spec_layout(layout: &OrbitLayout, dry_run: bool) -> MigrateReport {
    let mut report = MigrateReport::default();
    let specs_dir = layout.specs_dir();
    if !specs_dir.exists() {
        return report;
    }

    // 1. Enumerate flat-spec ids: any `<id>.yaml` directly under specs_dir
    //    whose stem contains no dot (so `<id>.drive.yaml` is excluded — its
    //    stem `<id>.drive` has a dot).
    let entries = match std::fs::read_dir(&specs_dir) {
        Ok(it) => it,
        Err(e) => {
            report.errors.push((specs_dir, e.to_string()));
            return report;
        }
    };
    let mut spec_ids: Vec<String> = Vec::new();
    let mut all_files: Vec<(String, PathBuf)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Pre-existing folder (bd-era or already migrated). Skip.
            continue;
        }
        let name = match path.file_name().and_then(OsStr::to_str) {
            Some(s) => s.to_string(),
            None => continue,
        };
        all_files.push((name.clone(), path.clone()));
        if name.ends_with(".yaml") {
            let stem = &name[..name.len() - ".yaml".len()];
            if !stem.contains('.') {
                spec_ids.push(stem.to_string());
            }
        }
    }
    spec_ids.sort();

    // 2. For each spec id, plan the moves.
    for spec_id in &spec_ids {
        let spec_folder = specs_dir.join(spec_id);
        // Three folder states:
        //   - folder absent → fresh migration
        //   - folder present, spec.yaml inside → already migrated, skip
        //   - folder present, spec.yaml absent → hybrid state from a
        //     prior partial migration (e.g. bd-era folder kept its
        //     sidecars while the spec body moved to the flat layout).
        //     Fold the flat spec.yaml plus any flat sidecars into the
        //     existing folder; leave folder contents alone.
        if spec_folder.join("spec.yaml").exists() {
            report.already_folder.push(spec_id.clone());
            continue;
        }

        let mut planned: Vec<PlannedMove> = Vec::new();
        // Spec yaml itself.
        planned.push(PlannedMove {
            from: specs_dir.join(format!("{spec_id}.yaml")),
            to: spec_folder.join("spec.yaml"),
        });
        // Sidecars: anything starting `<spec_id>.` with the rest matching
        // a known sidecar suffix.
        let prefix = format!("{spec_id}.");
        for (name, full_path) in &all_files {
            if !name.starts_with(&prefix) {
                continue;
            }
            let suffix = &name[prefix.len()..];
            // Skip the spec.yaml itself (handled above) — its `name` is
            // `<spec_id>.yaml` which matches the prefix; the suffix is
            // `yaml` which isn't a sidecar suffix, so it would be
            // skipped by the next check anyway — but explicit is clearer.
            if suffix == "yaml" {
                continue;
            }
            if !is_sidecar_suffix(suffix) {
                continue;
            }
            planned.push(PlannedMove {
                from: full_path.clone(),
                to: spec_folder.join(suffix),
            });
        }

        // 3. Execute the moves (unless dry_run).
        if !dry_run {
            if let Err(e) = std::fs::create_dir_all(&spec_folder) {
                report.errors.push((spec_folder.clone(), e.to_string()));
                continue;
            }
            let mut had_error = false;
            for mv in &planned {
                if let Err(e) = std::fs::rename(&mv.from, &mv.to) {
                    report.errors.push((mv.from.clone(), e.to_string()));
                    had_error = true;
                    break;
                }
            }
            if had_error {
                continue;
            }
        }

        report.moves.extend(planned);
        report.migrated.push(spec_id.clone());
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn fresh() -> (tempfile::TempDir, OrbitLayout) {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        (dir, layout)
    }

    #[test]
    fn migrate_moves_flat_spec_into_folder() {
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "id: 2026-05-10-foo\n").unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert_eq!(report.migrated, vec!["2026-05-10-foo"]);
        assert!(report.errors.is_empty());

        assert!(specs.join("2026-05-10-foo").is_dir());
        assert!(specs.join("2026-05-10-foo/spec.yaml").exists());
        assert!(!specs.join("2026-05-10-foo.yaml").exists());
    }

    #[test]
    fn migrate_folds_all_sidecar_shapes() {
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "id: 2026-05-10-foo\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.drive.yaml"), "drive\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.rally.yaml"), "rally\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.notes.jsonl"), "{}\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.tasks.jsonl"), "{}\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.interview.md"), "# i\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.decisions.md"), "# d\n").unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo.review-spec-2026-05-10.md"),
            "# r\n",
        )
        .unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo.review-spec-2026-05-10-v2.md"),
            "# r\n",
        )
        .unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo.review-pr-2026-05-10.md"),
            "# r\n",
        )
        .unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);

        let folder = specs.join("2026-05-10-foo");
        for name in [
            "spec.yaml",
            "drive.yaml",
            "rally.yaml",
            "notes.jsonl",
            "tasks.jsonl",
            "interview.md",
            "decisions.md",
            "review-spec-2026-05-10.md",
            "review-spec-2026-05-10-v2.md",
            "review-pr-2026-05-10.md",
        ] {
            assert!(folder.join(name).exists(), "missing {name}");
        }
    }

    #[test]
    fn migrate_is_idempotent() {
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "id: 2026-05-10-foo\n").unwrap();

        let _ = migrate_spec_layout(&layout, false);
        let report2 = migrate_spec_layout(&layout, false);
        // Second run finds no flat specs — fully migrated state. No moves,
        // no errors. (`already_folder` remains empty because nothing in
        // the flat-spec scan matched, so there's no list to populate.)
        assert!(report2.migrated.is_empty());
        assert!(report2.moves.is_empty());
        assert!(report2.errors.is_empty());

        // The post-migration folder content remains intact.
        assert!(specs.join("2026-05-10-foo/spec.yaml").exists());
    }

    #[test]
    fn migrate_partial_state_skips_when_folder_has_spec_yaml() {
        // Flat `<id>.yaml` AND folder `<id>/spec.yaml` both present (a
        // hand-edited or already-fully-migrated state). The migration
        // must skip rather than overwrite the folder's spec.yaml.
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "stray\n").unwrap();
        std::fs::create_dir_all(specs.join("2026-05-10-foo")).unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo/spec.yaml"),
            "id: 2026-05-10-foo\n",
        )
        .unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert!(report.migrated.is_empty());
        assert_eq!(report.already_folder, vec!["2026-05-10-foo"]);
        // Folder content not clobbered.
        let body = std::fs::read_to_string(specs.join("2026-05-10-foo/spec.yaml")).unwrap();
        assert!(body.contains("id: 2026-05-10-foo"));
        // Stray flat file left in place — operator can decide what to do.
        assert!(specs.join("2026-05-10-foo.yaml").exists());
    }

    #[test]
    fn migrate_hybrid_state_folds_into_existing_folder() {
        // Flat `<id>.yaml` exists; folder `<id>/` exists with sidecars
        // but NO `spec.yaml` (e.g. bd-era folder kept its sidecars while
        // the spec body lived in the flat layout). Migration must fold
        // the flat spec.yaml into the folder without disturbing existing
        // folder content.
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(
            specs.join("2026-05-10-foo.yaml"),
            "id: 2026-05-10-foo\nfresh\n",
        )
        .unwrap();
        std::fs::create_dir_all(specs.join("2026-05-10-foo")).unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo/interview.md"),
            "# old interview\n",
        )
        .unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo/review-spec-2026-05-10.md"),
            "# old review\n",
        )
        .unwrap();
        // A flat sidecar exists too — should fold into the folder.
        std::fs::write(specs.join("2026-05-10-foo.notes.jsonl"), "{}\n").unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert_eq!(report.migrated, vec!["2026-05-10-foo"]);
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);

        // Spec yaml landed inside the folder.
        let body = std::fs::read_to_string(specs.join("2026-05-10-foo/spec.yaml")).unwrap();
        assert!(body.contains("fresh"));
        // Pre-existing sidecars untouched.
        assert!(specs.join("2026-05-10-foo/interview.md").exists());
        assert!(specs.join("2026-05-10-foo/review-spec-2026-05-10.md").exists());
        // Flat sidecar folded in.
        assert!(specs.join("2026-05-10-foo/notes.jsonl").exists());
        assert!(!specs.join("2026-05-10-foo.notes.jsonl").exists());
    }

    #[test]
    fn migrate_does_not_collide_with_prefix_shared_ids() {
        // 2026-05-10-foo and 2026-05-10-foo-bar are both flat specs.
        // The prefix-collision risk choice 0021 names: when migrating
        // `<foo>`, the planner must not pick up `<foo-bar>.yaml` as a
        // sidecar of `<foo>`.
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "id: 2026-05-10-foo\n").unwrap();
        std::fs::write(
            specs.join("2026-05-10-foo-bar.yaml"),
            "id: 2026-05-10-foo-bar\n",
        )
        .unwrap();
        std::fs::write(specs.join("2026-05-10-foo.drive.yaml"), "drive\n").unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert!(report.errors.is_empty(), "errors: {:?}", report.errors);

        // Both got their own folders.
        assert!(specs.join("2026-05-10-foo").is_dir());
        assert!(specs.join("2026-05-10-foo-bar").is_dir());
        // foo's drive sidecar landed in foo's folder, not foo-bar's.
        assert!(specs.join("2026-05-10-foo/drive.yaml").exists());
        assert!(!specs.join("2026-05-10-foo-bar/drive.yaml").exists());
    }

    #[test]
    fn migrate_dry_run_does_not_write() {
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "id: 2026-05-10-foo\n").unwrap();

        let report = migrate_spec_layout(&layout, true);
        assert_eq!(report.migrated, vec!["2026-05-10-foo"]);
        // dry_run leaves the source flat file in place.
        assert!(specs.join("2026-05-10-foo.yaml").exists());
        assert!(!specs.join("2026-05-10-foo").is_dir());
    }

    #[test]
    fn migrate_skips_existing_folders() {
        // bd-era folder pre-exists. Migration must not touch it.
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::create_dir_all(specs.join("2026-04-19-rally")).unwrap();
        std::fs::write(
            specs.join("2026-04-19-rally/spec.yaml"),
            "id: 2026-04-19-rally\n",
        )
        .unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert!(report.migrated.is_empty());
        assert!(report.errors.is_empty());
        // bd-era folder content untouched.
        assert!(specs.join("2026-04-19-rally/spec.yaml").exists());
    }

    #[test]
    fn migrate_ignores_unknown_sidecar_suffixes() {
        // A file like `<id>.random.txt` should NOT be treated as a sidecar.
        let (_dir, layout) = fresh();
        let specs = layout.specs_dir();
        std::fs::write(specs.join("2026-05-10-foo.yaml"), "id: 2026-05-10-foo\n").unwrap();
        std::fs::write(specs.join("2026-05-10-foo.random.txt"), "junk\n").unwrap();

        let report = migrate_spec_layout(&layout, false);
        assert!(report.errors.is_empty());
        // The unknown-suffix file stays at the top level.
        assert!(specs.join("2026-05-10-foo.random.txt").exists());
        assert!(!specs.join("2026-05-10-foo/random.txt").exists());
    }

    #[test]
    fn is_review_suffix_recognises_cycles() {
        assert!(is_review_suffix("review-spec-2026-05-10.md"));
        assert!(is_review_suffix("review-spec-2026-05-10-v2.md"));
        assert!(is_review_suffix("review-spec-2026-05-10-v3.md"));
        assert!(is_review_suffix("review-pr-2026-05-10.md"));
        assert!(is_review_suffix("review-pr-2026-05-10-v2.md"));
        assert!(!is_review_suffix("review-spec-2026-05-10-v4.md"));
        assert!(!is_review_suffix("review-other-2026-05-10.md"));
        assert!(!is_review_suffix("notreview-spec-2026-05-10.md"));
    }
}
