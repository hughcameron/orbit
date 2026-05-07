//! orbit-state-core — the files-canonical agent substrate.
//!
//! Layering:
//!   - [`error`]   : the single error taxonomy (`<verb>: <category>: <sentence>`).
//!   - [`schema`]  : strongly-typed entity definitions with `deny_unknown_fields`.
//!   - [`canonical`] : LF-only, deterministic-key serialiser + parser entry points.
//!   - [`atomic`]  : temp + rename writes; CRLF-rejecting line policy.
//!
//! Higher layers (verbs, index, locks, MCP) build on these.

pub mod atomic;
pub mod canonical;
pub mod error;
pub mod index;
pub mod layout;
pub mod locks;
pub mod migrations;
pub mod schema;
pub mod sqlite_link;
pub mod verbs;

pub use error::{Category, Error, Result};
pub use sqlite_link::{link_sanity_check, sqlite_version};
pub use verbs::{
    envelope_err, envelope_err_string, envelope_ok, envelope_ok_string, execute, SpecListArgs,
    SpecListResult, SpecNoteArgs, SpecNoteResult, SpecShowArgs, SpecShowResult, SpecSummary,
    VerbRequest, VerbResponse,
};
