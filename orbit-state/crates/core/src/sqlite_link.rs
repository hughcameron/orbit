//! Minimal SQLite touch-point for ac-21 (week-1 cross-compile validation).
//!
//! ac-21 requires the skeleton binary to link the C-dependency chain that
//! production binaries will use — specifically `rusqlite` (bundled SQLite).
//! Without an actual rusqlite call site, the linker drops the dependency and
//! the cross-compile validation degrades to "pure-Rust binary builds on both
//! platforms," which doesn't surface the most likely failure mode (C-dep
//! cross-compile pain).
//!
//! This module is a deliberate touch-point. The function it exports returns
//! the SQLite library version, which exercises both the link step and a
//! trivial runtime call. ac-02 will replace this with the real index layer.

use rusqlite::Connection;

/// Returns the SQLite library version reported by the linked C library.
///
/// On the cross-compile path this proves:
/// - rusqlite (and its bundled SQLite C source) compiled and linked
/// - the resulting binary can call into the C code at runtime
///
/// Used by [`crate::link_sanity_check`] (re-exported from the crate root).
pub fn sqlite_version() -> String {
    rusqlite::version().to_string()
}

/// Open an in-memory SQLite database. Returns `Ok(())` if successful.
///
/// This is a stronger touch-point than `sqlite_version()` — it actually
/// constructs a connection, which exercises the C library's runtime
/// initialisation path. ac-21's nm/otool check is satisfied by this; ac-02
/// builds on it.
pub fn link_sanity_check() -> Result<(), rusqlite::Error> {
    let _conn = Connection::open_in_memory()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sqlite_version_returns_nonempty_string() {
        let v = sqlite_version();
        assert!(!v.is_empty(), "rusqlite reported empty version");
        // Sanity: version strings are typically 3.x.y.
        assert!(v.starts_with('3'), "unexpected SQLite major version: {v}");
    }

    #[test]
    fn link_sanity_check_opens_in_memory_db() {
        link_sanity_check().expect("rusqlite Connection::open_in_memory must work");
    }
}
