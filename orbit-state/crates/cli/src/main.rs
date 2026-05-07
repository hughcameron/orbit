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
    envelope_err_string, envelope_ok_string, execute, SpecCloseArgs, SpecCloseResult,
    SpecCreateArgs, SpecCreateResult, SpecListArgs, SpecListResult, SpecNoteArgs, SpecNoteResult,
    SpecShowArgs, SpecShowResult, SpecUpdateArgs, SpecUpdateResult, TaskClaimArgs, TaskDoneArgs,
    TaskEventResult, TaskListArgs, TaskListResult, TaskOpenArgs, TaskOpenResult, TaskReadyArgs,
    TaskShowArgs, TaskShowResult, TaskUpdateArgs, VerbRequest, VerbResponse,
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
    /// Task verbs (open, list, show, ready, claim, update, done).
    Task {
        #[command(subcommand)]
        action: TaskAction,
    },
}

#[derive(Debug, Subcommand)]
enum TaskAction {
    /// Open a new task under a spec.
    Open {
        spec_id: String,
        body: String,
        #[arg(long = "label")]
        labels: Vec<String>,
        #[arg(long)]
        task_id: Option<String>,
        #[arg(long)]
        timestamp: Option<String>,
    },
    /// List tasks (current state per task_id).
    List {
        #[arg(long)]
        spec_id: Option<String>,
        #[arg(long)]
        state: Option<String>,
    },
    /// Show one task with its full event history.
    Show {
        spec_id: String,
        task_id: String,
    },
    /// List claimable (open, no claim) tasks.
    Ready {
        #[arg(long)]
        spec_id: Option<String>,
    },
    /// Claim an open task.
    Claim {
        spec_id: String,
        task_id: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long = "label")]
        labels: Vec<String>,
        #[arg(long)]
        timestamp: Option<String>,
    },
    /// Append an update note to a task.
    Update {
        spec_id: String,
        task_id: String,
        body: String,
        #[arg(long = "label")]
        labels: Vec<String>,
        #[arg(long)]
        timestamp: Option<String>,
    },
    /// Mark a task done.
    Done {
        spec_id: String,
        task_id: String,
        #[arg(long)]
        body: Option<String>,
        #[arg(long = "label")]
        labels: Vec<String>,
        #[arg(long)]
        timestamp: Option<String>,
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
    /// Show a single spec by id.
    Show {
        /// Spec identifier (e.g. `2026-05-07-orbit-state-v0.1` or `0001`).
        id: String,
    },
    /// Append a timestamped note to a spec.
    Note {
        /// Spec identifier.
        id: String,
        /// Note body. Use `-` to read from stdin (not yet implemented).
        body: String,
        /// Free-text labels (repeatable).
        #[arg(long = "label")]
        labels: Vec<String>,
        /// Override the substrate timestamp. Primarily for migration tools
        /// porting historical timestamps; production callers omit this.
        #[arg(long)]
        timestamp: Option<String>,
    },
    /// Create a new spec at `.orbit/specs/<id>.yaml`.
    Create {
        /// Spec identifier (slug-shaped; no path separators).
        id: String,
        /// One-sentence statement of what shipping this spec achieves.
        goal: String,
        /// Cards this spec advances (repeatable).
        #[arg(long = "card")]
        cards: Vec<String>,
        /// Free-text labels (repeatable).
        #[arg(long = "label")]
        labels: Vec<String>,
    },
    /// Update fields on an existing spec (status changes go via `close`).
    Update {
        id: String,
        /// New goal sentence (omit to keep current).
        #[arg(long)]
        goal: Option<String>,
        /// Replace card list. Pass with no values to clear.
        #[arg(long = "cards", num_args = 0..)]
        cards: Option<Vec<String>>,
        /// Replace label list. Pass with no values to clear.
        #[arg(long = "labels", num_args = 0..)]
        labels: Option<Vec<String>>,
    },
    /// Close a spec; transactionally appends to linked cards' `specs` arrays.
    Close {
        id: String,
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
            SpecAction::Show { id } => VerbRequest::SpecShow(SpecShowArgs { id: id.clone() }),
            SpecAction::Note {
                id,
                body,
                labels,
                timestamp,
            } => VerbRequest::SpecNote(SpecNoteArgs {
                id: id.clone(),
                body: body.clone(),
                labels: labels.clone(),
                timestamp: timestamp.clone(),
            }),
            SpecAction::Create {
                id,
                goal,
                cards,
                labels,
            } => VerbRequest::SpecCreate(SpecCreateArgs {
                id: id.clone(),
                goal: goal.clone(),
                cards: cards.clone(),
                labels: labels.clone(),
                acceptance_criteria: vec![],
            }),
            SpecAction::Update {
                id,
                goal,
                cards,
                labels,
            } => VerbRequest::SpecUpdate(SpecUpdateArgs {
                id: id.clone(),
                goal: goal.clone(),
                cards: cards.clone(),
                labels: labels.clone(),
                acceptance_criteria: None,
            }),
            SpecAction::Close { id } => VerbRequest::SpecClose(SpecCloseArgs { id: id.clone() }),
        },
        Command::Task { action } => match action {
            TaskAction::Open {
                spec_id,
                body,
                labels,
                task_id,
                timestamp,
            } => VerbRequest::TaskOpen(TaskOpenArgs {
                spec_id: spec_id.clone(),
                body: body.clone(),
                labels: labels.clone(),
                task_id: task_id.clone(),
                timestamp: timestamp.clone(),
            }),
            TaskAction::List { spec_id, state } => VerbRequest::TaskList(TaskListArgs {
                spec_id: spec_id.clone(),
                state: state.clone(),
            }),
            TaskAction::Show { spec_id, task_id } => VerbRequest::TaskShow(TaskShowArgs {
                spec_id: spec_id.clone(),
                task_id: task_id.clone(),
            }),
            TaskAction::Ready { spec_id } => VerbRequest::TaskReady(TaskReadyArgs {
                spec_id: spec_id.clone(),
            }),
            TaskAction::Claim {
                spec_id,
                task_id,
                body,
                labels,
                timestamp,
            } => VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: spec_id.clone(),
                task_id: task_id.clone(),
                body: body.clone(),
                labels: labels.clone(),
                timestamp: timestamp.clone(),
            }),
            TaskAction::Update {
                spec_id,
                task_id,
                body,
                labels,
                timestamp,
            } => VerbRequest::TaskUpdate(TaskUpdateArgs {
                spec_id: spec_id.clone(),
                task_id: task_id.clone(),
                body: body.clone(),
                labels: labels.clone(),
                timestamp: timestamp.clone(),
            }),
            TaskAction::Done {
                spec_id,
                task_id,
                body,
                labels,
                timestamp,
            } => VerbRequest::TaskDone(TaskDoneArgs {
                spec_id: spec_id.clone(),
                task_id: task_id.clone(),
                body: body.clone(),
                labels: labels.clone(),
                timestamp: timestamp.clone(),
            }),
        },
    }
}

/// Human-readable rendering. Best-effort, not stable for parsing — agents
/// should use `--json`.
fn render_human(response: &VerbResponse) {
    match response {
        VerbResponse::SpecList(result) => render_spec_list(result),
        VerbResponse::SpecShow(result) => render_spec_show(result),
        VerbResponse::SpecNote(result) => render_spec_note(result),
        VerbResponse::SpecCreate(result) => render_spec_create(result),
        VerbResponse::SpecUpdate(result) => render_spec_update(result),
        VerbResponse::SpecClose(result) => render_spec_close(result),
        VerbResponse::TaskOpen(result) => render_task_open(result),
        VerbResponse::TaskList(result) | VerbResponse::TaskReady(result) => {
            render_task_list(result)
        }
        VerbResponse::TaskShow(result) => render_task_show(result),
        VerbResponse::TaskClaim(result)
        | VerbResponse::TaskUpdate(result)
        | VerbResponse::TaskDone(result) => render_task_event(result),
    }
}

fn render_task_open(result: &TaskOpenResult) {
    println!(
        "opened task {} on {}: {}",
        result.task_id,
        result.event.spec_id,
        result.event.body.as_deref().unwrap_or("")
    );
}

fn render_task_list(result: &TaskListResult) {
    if result.tasks.is_empty() {
        println!("(no tasks)");
        return;
    }
    for t in &result.tasks {
        println!("{}\t{}\t{}\t{}", t.spec_id, t.task_id, t.state, t.body.as_deref().unwrap_or(""));
    }
}

fn render_task_show(result: &TaskShowResult) {
    println!("task:    {}", result.state.task_id);
    println!("spec:    {}", result.state.spec_id);
    println!("state:   {}", result.state.state);
    println!("events:  {}", result.state.event_count);
    for ev in &result.events {
        println!(
            "  {} [{}] {}",
            ev.timestamp,
            event_kind_label(&ev.event),
            ev.body.as_deref().unwrap_or("")
        );
    }
}

fn render_task_event(result: &TaskEventResult) {
    println!(
        "{} task {} ({})",
        event_kind_label(&result.event.event),
        result.event.task_id,
        result.event.timestamp
    );
}

fn event_kind_label(kind: &orbit_state_core::schema::TaskEventKind) -> &'static str {
    use orbit_state_core::schema::TaskEventKind::*;
    match kind {
        Open => "open",
        Claim => "claim",
        Update => "update",
        Done => "done",
    }
}

fn render_spec_note(result: &SpecNoteResult) {
    println!(
        "noted on {}: {} ({})",
        result.note.spec_id, result.note.body, result.note.timestamp
    );
}

fn render_spec_create(result: &SpecCreateResult) {
    println!("created spec {}: {}", result.spec.id, result.spec.goal);
}

fn render_spec_update(result: &SpecUpdateResult) {
    println!("updated spec {}: {}", result.spec.id, result.spec.goal);
}

fn render_spec_close(result: &SpecCloseResult) {
    println!("closed spec {}", result.spec.id);
    if !result.cards_updated.is_empty() {
        println!("cards updated: {}", result.cards_updated.join(", "));
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

fn render_spec_show(result: &SpecShowResult) {
    let s = &result.spec;
    let status = match s.status {
        orbit_state_core::schema::SpecStatus::Open => "open",
        orbit_state_core::schema::SpecStatus::Closed => "closed",
    };
    println!("id:     {}", s.id);
    println!("status: {status}");
    println!("goal:   {}", s.goal);
    if !s.cards.is_empty() {
        println!("cards:  {}", s.cards.join(", "));
    }
    if !s.labels.is_empty() {
        println!("labels: {}", s.labels.join(", "));
    }
    if !s.acceptance_criteria.is_empty() {
        println!("acceptance:");
        for ac in &s.acceptance_criteria {
            let check = if ac.checked { "x" } else { " " };
            let gate = if ac.gate { " [gate]" } else { "" };
            println!("  [{check}] {}{gate}: {}", ac.id, ac.description);
        }
    }
}
