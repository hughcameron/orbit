//! Canonical serialiser + parser entry points.
//!
//! Per `values.load_bearing` (the substrate value): every file conforms to schema,
//! parser-validated. The canonical writer is the half of that property that
//! prevents whitespace/ordering drift between writes; the strict parser
//! (`deny_unknown_fields` on every type, plus CRLF rejection here) is the half
//! that prevents lossy parses.
//!
//! Per ac-01 fixture (iv): canonical files are LF-only by contract. CRLF input
//! is rejected at parse time with a clear error.

use crate::error::{Category, Error, Result};
use serde::{de::DeserializeOwned, Serialize};

/// Verb identifier used in canonical-layer errors. Higher layers can shadow
/// this with the actual verb (`spec.show`, etc.); the canonical layer uses
/// this generic name when it surfaces parse / serialise errors directly.
const CANONICAL_VERB: &str = "canonical";

/// Parse a canonical YAML string into `T`.
///
/// Rejects:
/// - CRLF line endings (`\r\n`) — canonical files are LF-only
/// - Lone CR (`\r`) — rejected for the same reason
/// - Unknown fields — by virtue of `T`'s `deny_unknown_fields` annotation
///
/// Returns a [`Category::Malformed`] error on any parse failure.
pub fn parse_yaml<T: DeserializeOwned>(text: &str) -> Result<T> {
    reject_cr(text, "yaml")?;
    serde_yaml::from_str(text).map_err(|e| {
        Error::malformed(CANONICAL_VERB, format!("yaml parse failed: {e}"))
            .with_source(e)
    })
}

/// Parse a single JSON line (used for task event streams).
pub fn parse_json_line<T: DeserializeOwned>(line: &str) -> Result<T> {
    reject_cr(line, "jsonl")?;
    serde_json::from_str(line).map_err(|e| {
        Error::malformed(CANONICAL_VERB, format!("json parse failed: {e}"))
            .with_source(e)
    })
}

/// Serialise `value` to the canonical YAML form.
///
/// Properties:
/// - Output is LF-only (never CRLF)
/// - Output ends with exactly one newline
/// - Field ordering is determined by `T`'s `Serialize` derive (which respects
///   declaration order on structs), giving deterministic output across runs
///
/// Round-trip property: for any `T` where `T: Serialize + DeserializeOwned`
/// with a stable schema, `parse_yaml(serialise_yaml(x)) == x` AND
/// `serialise_yaml(parse_yaml(serialise_yaml(x))) == serialise_yaml(x)`
/// byte-identical. The second property is what ac-01 verifies.
pub fn serialise_yaml<T: Serialize>(value: &T) -> Result<String> {
    let mut out = serde_yaml::to_string(value).map_err(|e| {
        Error::malformed(CANONICAL_VERB, format!("yaml serialise failed: {e}"))
            .with_source(e)
    })?;
    // serde_yaml emits LF on all platforms (it doesn't honour platform line
    // separator), but we defensively normalise in case a future implementation
    // changes this.
    if out.contains('\r') {
        out = out.replace("\r\n", "\n").replace('\r', "\n");
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

/// Serialise a single task event to a JSONL line (newline-terminated).
pub fn serialise_json_line<T: Serialize>(value: &T) -> Result<String> {
    let mut out = serde_json::to_string(value).map_err(|e| {
        Error::malformed(CANONICAL_VERB, format!("json serialise failed: {e}"))
            .with_source(e)
    })?;
    out.push('\n');
    Ok(out)
}

/// Reject any input containing CR characters.
///
/// Canonical files are LF-only by contract (ac-01 fixture (iv)). A CRLF body
/// MUST fail parse with a clear error; this function provides that check
/// uniformly across YAML and JSONL entry points.
fn reject_cr(text: &str, format: &str) -> Result<()> {
    if let Some(pos) = text.find('\r') {
        // Find the byte offset's line number for a more useful diagnostic.
        let line = text[..pos].matches('\n').count() + 1;
        return Err(Error {
            verb: CANONICAL_VERB.into(),
            category: Category::Malformed,
            message: format!(
                "{format} input contains CR at line {line} — canonical files are LF-only"
            ),
            source: None,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{Choice, ChoiceStatus, SchemaVersion};

    #[test]
    fn round_trip_byte_identical_after_one_normalisation() {
        // ac-01: parse -> serialise -> parse -> serialise produces byte-identical
        // output. The first serialise normalises, so the *second* round-trip
        // is what we verify byte-identical.
        let original = SchemaVersion {
            version: "0.1".into(),
            note: Some("bootstrap".into()),
        };
        let yaml1 = serialise_yaml(&original).unwrap();
        let parsed: SchemaVersion = parse_yaml(&yaml1).unwrap();
        let yaml2 = serialise_yaml(&parsed).unwrap();
        assert_eq!(yaml1, yaml2, "second round-trip must be byte-identical");
    }

    #[test]
    fn output_is_lf_only() {
        let v = SchemaVersion { version: "0.1".into(), note: None };
        let out = serialise_yaml(&v).unwrap();
        assert!(!out.contains('\r'), "canonical output must not contain CR");
        assert!(out.ends_with('\n'), "canonical output must end with newline");
    }

    #[test]
    fn parse_rejects_crlf_with_clear_error() {
        // ac-01 fixture (iv): a CRLF body MUST fail parse with a clear error.
        let crlf = "version: '0.1'\r\nnote: hello\r\n";
        let err = parse_yaml::<SchemaVersion>(crlf).unwrap_err();
        assert_eq!(err.category, Category::Malformed);
        assert!(
            err.message.contains("CR") && err.message.contains("LF-only"),
            "error must clearly explain the CR rejection: got {err}"
        );
    }

    #[test]
    fn parse_rejects_lone_cr() {
        let weird = "version: '0.1'\rnote: hello\n";
        let err = parse_yaml::<SchemaVersion>(weird).unwrap_err();
        assert_eq!(err.category, Category::Malformed);
    }

    #[test]
    fn parse_rejects_unknown_field_via_deny_unknown_fields() {
        // Strict schema conformance — the lossy-parse failure mode prevention.
        let yaml = "version: '0.1'\nunknown_field: oops\n";
        let err = parse_yaml::<SchemaVersion>(yaml).unwrap_err();
        assert_eq!(err.category, Category::Malformed);
        assert!(err.message.contains("unknown"));
    }

    // ------------------------------------------------------------------------
    // Choice fixture suite — ac-01 (i)-(iv) edge cases for multiline bodies.
    // ------------------------------------------------------------------------

    fn choice_with_body(body: &str) -> Choice {
        Choice {
            id: "0001".into(),
            title: "test".into(),
            status: ChoiceStatus::Accepted,
            date_created: "2026-05-07".into(),
            date_modified: None,
            body: body.into(),
            references: vec![],
        }
    }

    fn assert_round_trip_stable(choice: &Choice) {
        let yaml1 = serialise_yaml(choice).unwrap();
        let parsed: Choice = parse_yaml(&yaml1).unwrap();
        let yaml2 = serialise_yaml(&parsed).unwrap();
        assert_eq!(
            yaml1, yaml2,
            "choice round-trip not byte-identical for body fixture"
        );
        assert_eq!(parsed.body, choice.body, "choice body lost in round-trip");
    }

    #[test]
    fn choice_fixture_i_code_fence_with_yaml() {
        // (i) a body with a triple-backtick code fence containing YAML.
        let body = "Context: see the example below.\n\n```yaml\nfoo: bar\nlist:\n  - a\n  - b\n```\n\nDecision: ship it.";
        assert_round_trip_stable(&choice_with_body(body));
    }

    #[test]
    fn choice_fixture_ii_trailing_blank_line() {
        // (ii) a body ending in a trailing blank line.
        // This is a known YAML round-trip hazard: trailing whitespace stripped
        // by some emitters. We assert the parsed body equals the original AND
        // the second-round serialisation is byte-identical.
        let body = "Decision: ship it.\n\n";
        assert_round_trip_stable(&choice_with_body(body));
    }

    #[test]
    fn choice_fixture_iii_hard_tab_indentation() {
        // (iii) a body with hard-tab indentation. YAML disallows tabs in
        // indentation, but tabs WITHIN a quoted string body are fine.
        let body = "Algorithm:\n\tstep 1\n\tstep 2\n";
        assert_round_trip_stable(&choice_with_body(body));
    }

    #[test]
    fn choice_fixture_iv_crlf_body_fails_parse() {
        // (iv) a body with CRLF line endings — MUST fail parse with a clear
        // error per ac-01 verification.
        let yaml = "id: '0001'\ntitle: t\nstatus: accepted\ndate_created: '2026-05-07'\nbody: \"line1\\r\\nline2\"\nreferences: []\n";
        // Note: this YAML doesn't contain literal CR — the body field has the
        // escape sequence. The check that matters is when YAML *itself*
        // contains CR, which is what reject_cr enforces. For a body whose
        // RUNTIME value contains CR (after parsing), the canonical writer
        // would re-emit it; we test that path separately.
        let _: Choice = parse_yaml(yaml).expect("escape-sequence body parses fine");

        // The CRLF rejection that matters: literal CR in the YAML source.
        let crlf_yaml = "id: '0001'\r\ntitle: t\r\nstatus: accepted\r\n";
        let err = parse_yaml::<Choice>(crlf_yaml).unwrap_err();
        assert_eq!(err.category, Category::Malformed);
        assert!(err.message.contains("LF-only"));
    }
}
