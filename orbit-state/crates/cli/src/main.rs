//! orbit CLI — files-canonical agent substrate.
//!
//! Architectural shape (per ac-05): the CLI parses argv into a typed
//! [`VerbRequest`], hands it to [`orbit_state_core::execute`], and renders
//! the response. The MCP server uses the same dispatch fn — the parity
//! guarantee falls out of "both surfaces serialise the same `VerbResponse`
//! through the same envelope helper."
//!
//! Output modes:
//! - default: human-readable text (TSV-like)
//! - `--json`: the wire envelope (`{"data":...,"ok":true}` / `{"error":...,"ok":false}`)
//!
//! The `--json` output is byte-identical to the envelope MCP wraps in its
//! `tools/call` response — that's the parity contract.
//!
//! ac-21 link preservation: `link_sanity_check` is called once at startup so
//! the linker keeps the rusqlite dependency even when the invoked verb path
//! doesn't touch SQLite. Once write verbs land (ac-06+) and exercise the
//! index, this call becomes redundant and can be removed.

use clap::{Parser, Subcommand};
use orbit_state_core::layout::OrbitLayout;
use orbit_state_core::{
    envelope_err_string, envelope_ok_string, execute, SpecListArgs, SpecListResult, VerbRequest,
    VerbResponse,
};
use std::path::PathBuf;
use std::process::ExitCode;

/// orbit — files-canonical agent substrate.
#[derive(Debug, Parser)]
#[command(name = "orbit", version, about)]
struct Cli {
    /// Path to the repo root (defaults to the current directory). The
    /// `.orbit/` folder is resolved relative to this.
    #[arg(long, global = true)]
    root: Option<PathBuf>,

    /// Emit the wire envelope as JSON instead of human-readable output.
    /// In `--json` mode the bytes are byte-identical to MCP's `tools/call`
    /// response payload — this is what the parity test compares.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Spec verbs (list, show, create, ...).
    Spec {
        #[command(subcommand)]
        action: SpecAction,
    },
}

#[derive(Debug, Subcommand)]
enum SpecAction {
    /// List specs in `.orbit/specs/`, sorted by id.
    List {
        /// Filter by status (`open` or `closed`).
        #[arg(long)]
        status: Option<String>,
    },
}

fn main() -> ExitCode {
    // ac-21 link preservation: ensure the linker keeps rusqlite/SQLite even
    // if the invoked verb path doesn't touch the index.
    if let Err(e) = orbit_state_core::link_sanity_check() {
        eprintln!("orbit: unavailable: link sanity check failed: {e}");
        return ExitCode::FAILURE;
    }

    let cli = Cli::parse();
    let root = match cli.root.clone() {
        Some(p) => p,
        None => match std::env::current_dir() {
            Ok(p) => p,
            Err(e) => {
                eprintln!("orbit: unavailable: cannot resolve cwd: {e}");
                return ExitCode::FAILURE;
            }
        },
    };
    let layout = OrbitLayout::at(&root);

    let request = build_request(&cli.command);

    match execute(&layout, &request) {
        Ok(response) => {
            if cli.json {
                match envelope_ok_string(&response) {
                    Ok(s) => println!("{s}"),
                    Err(e) => {
                        eprintln!("{e}");
                        return ExitCode::FAILURE;
                    }
                }
            } else {
                render_human(&response);
            }
            ExitCode::SUCCESS
        }
        Err(err) => {
            if cli.json {
                println!("{}", envelope_err_string(&err));
            } else {
                eprintln!("{err}");
            }
            ExitCode::FAILURE
        }
    }
}

/// Translate the parsed argv into a [`VerbRequest`]. Pure function — no I/O,
/// no dispatch — so it stays unit-testable and the parity layer's "two
/// independent parsers, same dispatch" property is easy to reason about.
fn build_request(command: &Command) -> VerbRequest {
    match command {
        Command::Spec { action } => match action {
            SpecAction::List { status } => VerbRequest::SpecList(SpecListArgs {
                status: status.clone(),
            }),
        },
    }
}

/// Human-readable rendering. Best-effort, not stable for parsing — agents
/// should use `--json`.
fn render_human(response: &VerbResponse) {
    match response {
        VerbResponse::SpecList(result) => render_spec_list(result),
    }
}

fn render_spec_list(result: &SpecListResult) {
    if result.specs.is_empty() {
        println!("(no specs)");
        return;
    }
    // Tab-separated for cheap eyeballing. id, status, goal.
    for s in &result.specs {
        println!("{}\t{}\t{}", s.id, s.status, s.goal);
    }
}
