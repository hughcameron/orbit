//! `orbit-canonicalise` — standalone shim around `orbit_state_core::canonicalise_all`.
//!
//! Rewrites every canonical YAML under `.orbit/` through the canonical writer
//! to fix byte-drift in place. The same logic ships as a subcommand of the
//! main `orbit` binary (`orbit canonicalise`); this standalone exists for
//! muscle-memory continuity with the original ac-23 migration tool. New users
//! should prefer `orbit canonicalise`.
//!
//! Files NOT touched: state.db, schema-version (substrate-managed and almost
//! always already canonical), task `.tasks.jsonl` streams (append-only, not
//! round-trippable as YAML).

use orbit_state_core::canonicalise_all;
use orbit_state_core::layout::OrbitLayout;
use std::path::PathBuf;
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
                     USAGE: orbit-canonicalise [--root PATH] [--dry-run]\n\
                     \n\
                     The same operation is available as `orbit canonicalise` in the\n\
                     main CLI; new users should prefer that surface."
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
    let report = canonicalise_all(&layout, dry_run);

    println!(
        "{} {} file(s) rewritten, {} unchanged, {} parse-failed",
        if dry_run { "would rewrite" } else { "rewrote" },
        report.rewrote,
        report.unchanged,
        report.parse_failed.len()
    );
    for (path, msg) in &report.parse_failed {
        eprintln!("  parse failed: {} — {msg}", path.display());
    }

    if report.has_failures() {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
