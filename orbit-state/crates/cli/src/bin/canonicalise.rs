//! `orbit-canonicalise` — one-shot tool that rewrites every canonical YAML file
//! under `.orbit/` through the canonical writer, fixing byte-drift in place.
//!
//! Use case (ac-23): legacy cards authored before the v0.1 schema froze carry
//! whitespace/ordering drift relative to the canonical writer's output. Once
//! they parse cleanly (schema-fix pass already done), this binary normalises
//! their bytes so the round-trip gate (ac-16) reports clean.
//!
//! Long-term role: a hygiene utility for human-authored entities. Whenever a
//! card or choice is hand-edited, byte-drift may creep in; `orbit-canonicalise`
//! repairs the file without changing its semantic content.
//!
//! Files NOT touched: state.db, schema-version (substrate-managed and almost
//! always already canonical), task `.tasks.jsonl` streams (append-only, not
//! round-trippable as YAML).

use orbit_state_core::canonical::{parse_yaml, serialise_yaml};
use orbit_state_core::layout::OrbitLayout;
use orbit_state_core::schema::{Card, Choice, Memory, Spec};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let mut dry_run = false;
    let mut root = std::env::current_dir().expect("cwd");
    while let Some(a) = args.next() {
        match a.as_str() {
            "--dry-run" => dry_run = true,
            "--root" => {
                root = args
                    .next()
                    .map(PathBuf::from)
                    .expect("--root requires a path");
            }
            "-h" | "--help" => {
                println!(
                    "orbit-canonicalise — rewrite every canonical YAML through the canonical writer.\n\
                     \n\
                     USAGE: orbit-canonicalise [--root PATH] [--dry-run]"
                );
                return ExitCode::SUCCESS;
            }
            other => {
                eprintln!("unknown argument: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let layout = OrbitLayout::at(&root);
    let mut report = Report::default();

    for path in list(layout.list_spec_files()) {
        canonicalise::<Spec>(&path, dry_run, &mut report);
    }
    for path in list(layout.list_card_files()) {
        canonicalise::<Card>(&path, dry_run, &mut report);
    }
    for path in list(layout.list_choice_files()) {
        canonicalise::<Choice>(&path, dry_run, &mut report);
    }
    for path in list(layout.list_memory_files()) {
        canonicalise::<Memory>(&path, dry_run, &mut report);
    }

    println!(
        "{} {} file(s) rewritten, {} unchanged, {} parse-failed",
        if dry_run { "would rewrite" } else { "rewrote" },
        report.rewrote,
        report.unchanged,
        report.failed.len()
    );
    for f in &report.failed {
        eprintln!("  parse failed: {} — {}", f.0.display(), f.1);
    }

    if report.failed.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[derive(Default)]
struct Report {
    rewrote: usize,
    unchanged: usize,
    failed: Vec<(PathBuf, String)>,
}

fn list(r: std::io::Result<Vec<PathBuf>>) -> Vec<PathBuf> {
    r.unwrap_or_default()
}

fn canonicalise<T>(path: &Path, dry_run: bool, report: &mut Report)
where
    T: DeserializeOwned + Serialize,
{
    let original = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            report.failed.push((path.to_path_buf(), format!("read: {e}")));
            return;
        }
    };
    let parsed: T = match parse_yaml(&original) {
        Ok(v) => v,
        Err(e) => {
            report.failed.push((path.to_path_buf(), e.to_string()));
            return;
        }
    };
    let reserialised = match serialise_yaml(&parsed) {
        Ok(s) => s,
        Err(e) => {
            report.failed.push((path.to_path_buf(), e.to_string()));
            return;
        }
    };
    if reserialised == original {
        report.unchanged += 1;
        return;
    }
    if !dry_run {
        if let Err(e) = std::fs::write(path, &reserialised) {
            report.failed.push((path.to_path_buf(), format!("write: {e}")));
            return;
        }
    }
    report.rewrote += 1;
}
