//! Error taxonomy — every error flattens to `<verb>: <category>: <sentence>` per
//! ac-05's contracted format. Categories are the closed set named in the spec.

use std::fmt;
use thiserror::Error;

/// The closed set of error categories per ac-05.
///
/// The contract: every error message has the shape
/// `<verb>: <category>: <human-sentence>` so agents can parse them in <2 attempts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Entity does not exist on disk or in the index.
    NotFound,
    /// Operation conflicts with current state (e.g. `spec.close` with open tasks).
    Conflict,
    /// Lock acquisition failed within the configured timeout.
    Locked,
    /// File on disk failed schema conformance (parse error, unknown field, CRLF in canonical text).
    Malformed,
    /// Caller is not authorised to perform the operation.
    Unauthorised,
    /// Resource exists but is temporarily unavailable (I/O failure, disk full, etc.).
    Unavailable,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::NotFound => "not-found",
            Category::Conflict => "conflict",
            Category::Locked => "locked",
            Category::Malformed => "malformed",
            Category::Unauthorised => "unauthorised",
            Category::Unavailable => "unavailable",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The orbit-state error type.
///
/// Construct via the `verb_*` helpers so the verb name is captured once at the
/// call site and propagated through `Display` consistently.
#[derive(Debug, Error)]
pub struct Error {
    /// The verb that failed (e.g. `"spec.close"`, `"task.claim"`).
    pub verb: String,
    /// The category of failure.
    pub category: Category,
    /// Human-readable single-sentence message.
    pub message: String,
    /// Optional underlying cause for chained debugging.
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}: {}", self.verb, self.category, self.message)
    }
}

impl Error {
    /// Construct a new error from verb + category + message.
    pub fn new(verb: impl Into<String>, category: Category, message: impl Into<String>) -> Self {
        Error {
            verb: verb.into(),
            category,
            message: message.into(),
            source: None,
        }
    }

    /// Attach a source error for debugging context.
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    pub fn not_found(verb: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(verb, Category::NotFound, message)
    }

    pub fn conflict(verb: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(verb, Category::Conflict, message)
    }

    pub fn locked(verb: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(verb, Category::Locked, message)
    }

    pub fn malformed(verb: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(verb, Category::Malformed, message)
    }

    pub fn unauthorised(verb: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(verb, Category::Unauthorised, message)
    }

    pub fn unavailable(verb: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(verb, Category::Unavailable, message)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_format_matches_ac_05_contract() {
        // ac-05 verification: error format `<verb>: <category>: <human-sentence>`.
        let err = Error::not_found("spec.show", "no spec at .orbit/specs/0001.yaml");
        assert_eq!(
            err.to_string(),
            "spec.show: not-found: no spec at .orbit/specs/0001.yaml"
        );
    }

    #[test]
    fn every_category_has_canonical_string() {
        // Categories must match the closed set named in ac-05.
        let expected = [
            (Category::NotFound, "not-found"),
            (Category::Conflict, "conflict"),
            (Category::Locked, "locked"),
            (Category::Malformed, "malformed"),
            (Category::Unauthorised, "unauthorised"),
            (Category::Unavailable, "unavailable"),
        ];
        for (cat, s) in expected {
            assert_eq!(cat.as_str(), s);
            assert_eq!(format!("{cat}"), s);
        }
    }

    #[test]
    fn source_is_chainable() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let err = Error::unavailable("memory.remember", "cannot write memory")
            .with_source(io_err);
        assert!(err.source.is_some());
        // Display still flattens to the canonical format.
        assert_eq!(
            err.to_string(),
            "memory.remember: unavailable: cannot write memory"
        );
    }
}
