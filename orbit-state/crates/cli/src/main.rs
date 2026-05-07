//! orbit CLI entry point.
//!
//! Currently a stub for ac-21 (week-1 cross-compile validation). The CLI
//! depends on `orbit-state-core`, which links rusqlite (bundled SQLite). The
//! binary therefore exercises the C-dependency chain at link time, satisfying
//! ac-21's requirement that the skeleton must link SQLite — not just print a
//! version string from pure-Rust code.
//!
//! The verb dispatch tree (ac-05 onward) is added incrementally as each verb
//! AC ships.

use clap::Parser;

/// orbit — files-canonical agent substrate.
#[derive(Debug, Parser)]
#[command(name = "orbit", version, about)]
struct Cli {
    /// (Verb dispatch will land in ac-06 onward.)
    #[arg(hide = true)]
    _placeholder: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    // ac-21 link sanity: actually call into rusqlite so the linker keeps the
    // C-dependency chain in the final binary. Without this call, dead-code
    // elimination drops SQLite and the nm/otool symbol check would not
    // exercise the cross-compile risk class.
    orbit_state_core::link_sanity_check()?;
    println!(
        "orbit (skeleton) — sqlite version {} — verbs land in ac-06+",
        orbit_state_core::sqlite_version()
    );
    Ok(())
}
