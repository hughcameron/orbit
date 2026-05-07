//! orbit-mcp — Model Context Protocol server for orbit-state.
//!
//! Stub for ac-05; the actual MCP integration lands when the verb surface is
//! ready. Like the CLI, this binary depends on orbit-state-core so the link
//! step exercises rusqlite for ac-21's cross-compile validation.

fn main() -> anyhow::Result<()> {
    orbit_state_core::link_sanity_check()?;
    eprintln!(
        "orbit-mcp (skeleton) — sqlite {} — MCP server lands in ac-05",
        orbit_state_core::sqlite_version()
    );
    Ok(())
}
