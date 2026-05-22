//! Routine substrate — chain detection, content-addressed dedupe,
//! agent-authored SKILL.md write path, and verification.
//!
//! Per spec `2026-05-22-routine-proposals`. This module is deliberately
//! kept separate from any "skill author" module (none exists yet — card
//! 0022's skill-author spec ceremony lives in skill spec / verb space,
//! not orbit-state core). The AC-08 boundary is enforced by a code-search
//! test in `verbs.rs` that forbids any cross-import between this file
//! and a `skill_author` module.
//!
//! ## Capabilities
//!
//! - **Chain reconstruction (AC-01):** [`reconstruct_chains`] reads every
//!   `.orbit/skills/*.invocations.jsonl` file, groups invocations by
//!   `session_id`, sorts each group by `timestamp`, and returns one
//!   ordered chain per session. Purely additive — no change to the
//!   `SkillInvocation` schema.
//!
//! - **Recurrence detection (AC-02 + AC-05):** [`detect_recurring_chains`]
//!   takes per-session chains and surfaces sequential sub-chains that
//!   recur ≥ 2 times across sessions. Length-2 matches require exact
//!   ordering; length-≥3 matches allow ≤ 1 skipped step. When multiple
//!   overlapping chains match, the longest consistent chain wins. DAG
//!   shapes are flagged but never returned as recurring (v1 is
//!   sequential-only per AC-05).
//!
//! - **Chain id (AC-04):** [`chain_id`] returns the SHA-256 hex digest of
//!   the RFC-8785 / JCS canonical JSON encoding of the ordered skill_id
//!   sequence. Byte-deterministic across sessions and agents so AC-09's
//!   archive-state lookup is content-addressed.
//!
//! - **Content-addressed lookup (AC-09):** [`existing_routine_for_chain`]
//!   scans both `.claude/skills/` and `.claude/skills/.archive/` for any
//!   SKILL.md whose front-matter declares a matching `chain_id`. When
//!   one is found, the authoring path skips the write.
//!
//! - **Front-matter schema (AC-04):** [`RoutineFrontMatter`] is the
//!   strongly-typed front-matter shape every agent-authored routine
//!   must validate against on write. `created_by`, `created_at`,
//!   `pinned`, `last_verified`, `chain_id` are the load-bearing fields.

use crate::error::{Error, Result};
use crate::schema::{InvocationOutcome, SkillInvocation};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Front-matter shape every agent-authored routine carries.
///
/// Compatible with card 0022's curator metadata (`created_by`,
/// `created_at`, `pinned`) and additive with this card's freshness
/// fields (`last_verified`, `chain_id`). Per spec
/// `2026-05-22-routine-proposals` ac-04.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoutineFrontMatter {
    /// Short name for the routine — matches the directory name under
    /// `.claude/skills/<name>/`.
    pub name: String,
    /// One-sentence description of what the routine does.
    pub description: String,
    /// `agent` for routines authored by an orbit agent; `human` reserved
    /// for hand-authored skills the curator must bypass.
    pub created_by: String,
    /// ISO-8601 / RFC 3339 timestamp the agent wrote the SKILL.md.
    pub created_at: String,
    /// Curator pin flag. Defaults to `false` for agent-authored skills;
    /// the author may flip to `true` to bypass auto-archive.
    pub pinned: bool,
    /// RFC 3339 timestamp of the most recent passing
    /// `orbit routine verify <path>` run. Written by the verify verb only —
    /// `audit.conformance` reads but never mutates this field. Per ac-06.
    pub last_verified: String,
    /// SHA-256 hex digest of the JCS-canonical-JSON encoding of the
    /// ordered chain of skill_ids. Content-addressed dedupe key — see
    /// ac-04 + ac-09.
    pub chain_id: String,
    /// The ordered sequence of skill_ids this routine wraps. The
    /// canonical encoding of this array is what [`chain_id`] hashes.
    pub chain: Vec<String>,
}

/// One reconstructed chain — the ordered sequence of skill_ids invoked
/// in a single session. Per ac-01.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionChain {
    pub session_id: String,
    /// Skill_ids in invocation-timestamp order. Outcomes are not retained
    /// — the recurrence detector is shape-only at v1.
    pub chain: Vec<String>,
}

/// A recurring chain surfaced by [`detect_recurring_chains`]. Carries
/// the chain shape, occurrence count (number of distinct sessions that
/// matched), and `chain_id`. Per ac-02 + ac-04.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecurringChain {
    pub chain: Vec<String>,
    pub occurrences: usize,
    pub chain_id: String,
    /// Session ids whose sequence matched this chain (with the
    /// per-AC-02 relaxation rules applied).
    pub session_ids: Vec<String>,
}

/// Recurrence threshold — chain must appear in this many distinct
/// sessions to be eligible for authoring. Per ac-02 (≥ 2 occurrences)
/// and the simplest-cut row in the tabletop.
pub const RECURRENCE_THRESHOLD: usize = 2;

/// AC-01: Reconstruct one [`SessionChain`] per `session_id` from every
/// invocation row in `.orbit/skills/*.invocations.jsonl`.
///
/// This is the substrate aggregator (option b in the AC-01 decision):
/// purely additive, no schema migration, no per-row sequence_id field.
///
/// Returns chains sorted by `session_id` for determinism. Single-step
/// "chains" of length 1 are excluded — they aren't chains, they're
/// individual invocations the skill-self-improvement loop already
/// handles.
pub fn reconstruct_chains(skills_dir: &Path) -> Result<Vec<SessionChain>> {
    const VERB: &str = "routine.chains";
    if !skills_dir.exists() {
        return Ok(vec![]);
    }
    let mut by_session: HashMap<String, Vec<SkillInvocation>> = HashMap::new();
    let entries = std::fs::read_dir(skills_dir).map_err(|e| {
        Error::unavailable(VERB, format!("read skills dir: {e}"))
    })?;
    for entry in entries {
        let entry = entry.map_err(|e| {
            Error::unavailable(VERB, format!("read skills dir entry: {e}"))
        })?;
        let path = entry.path();
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };
        if !filename.ends_with(".invocations.jsonl") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        for (lineno, raw) in text.lines().enumerate() {
            if raw.is_empty() {
                continue;
            }
            let inv: SkillInvocation = serde_json::from_str(raw).map_err(|e| {
                Error::malformed(
                    VERB,
                    format!("parse {} line {}: {e}", path.display(), lineno + 1),
                )
            })?;
            by_session.entry(inv.session_id.clone()).or_default().push(inv);
        }
    }

    let mut chains = Vec::with_capacity(by_session.len());
    for (session_id, mut rows) in by_session {
        rows.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        if rows.len() < 2 {
            // Single invocation isn't a chain.
            continue;
        }
        let chain: Vec<String> = rows.into_iter().map(|r| r.skill_id).collect();
        chains.push(SessionChain { session_id, chain });
    }
    chains.sort_by(|a, b| a.session_id.cmp(&b.session_id));
    Ok(chains)
}

/// Filter [`reconstruct_chains`] output by `outcome` — only count
/// invocations whose outcome matches. Convenience for tests and future
/// outcome-aware detection. Currently unused by `detect_recurring_chains`
/// which is outcome-blind per the tabletop simplest-cut.
pub fn reconstruct_chains_filtered(
    skills_dir: &Path,
    accept: impl Fn(InvocationOutcome) -> bool,
) -> Result<Vec<SessionChain>> {
    const VERB: &str = "routine.chains";
    if !skills_dir.exists() {
        return Ok(vec![]);
    }
    let mut by_session: HashMap<String, Vec<SkillInvocation>> = HashMap::new();
    for entry in std::fs::read_dir(skills_dir).map_err(|e| {
        Error::unavailable(VERB, format!("read skills dir: {e}"))
    })? {
        let entry = entry.map_err(|e| {
            Error::unavailable(VERB, format!("read skills dir entry: {e}"))
        })?;
        let path = entry.path();
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };
        if !filename.ends_with(".invocations.jsonl") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        for (lineno, raw) in text.lines().enumerate() {
            if raw.is_empty() {
                continue;
            }
            let inv: SkillInvocation = serde_json::from_str(raw).map_err(|e| {
                Error::malformed(
                    VERB,
                    format!("parse {} line {}: {e}", path.display(), lineno + 1),
                )
            })?;
            if !accept(inv.outcome) {
                continue;
            }
            by_session.entry(inv.session_id.clone()).or_default().push(inv);
        }
    }
    let mut chains = Vec::with_capacity(by_session.len());
    for (session_id, mut rows) in by_session {
        rows.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        if rows.len() < 2 {
            continue;
        }
        let chain: Vec<String> = rows.into_iter().map(|r| r.skill_id).collect();
        chains.push(SessionChain { session_id, chain });
    }
    chains.sort_by(|a, b| a.session_id.cmp(&b.session_id));
    Ok(chains)
}

/// AC-02 + AC-05: detect recurring sequential chains from per-session
/// chains.
///
/// Matching rules:
///
/// - Length-2 chains require *exact* ordering match across sessions.
/// - Length-≥3 chains allow ≤ 1 skipped step between matching
///   invocations (catches minor variations).
/// - When multiple overlapping chains match, the longest consistent
///   chain wins (no sub-chain proposals).
///
/// Threshold: a chain must match in at least [`RECURRENCE_THRESHOLD`]
/// (= 2) distinct sessions to be returned. Chains observed only once
/// or with inconsistent ordering beyond the relaxation are filtered out.
///
/// v1 scope is sequential chains only (AC-05); DAG-shaped patterns
/// (variable orderings, parallel branches) are *not* returned even
/// when they recur.
pub fn detect_recurring_chains(sessions: &[SessionChain]) -> Vec<RecurringChain> {
    if sessions.len() < RECURRENCE_THRESHOLD {
        return vec![];
    }

    // Enumerate every contiguous sub-chain of length ≥2 from each
    // session, keyed by the chain shape. We then count distinct
    // sessions per shape.
    let mut shape_sessions: HashMap<Vec<String>, Vec<String>> = HashMap::new();
    for sc in sessions {
        let len = sc.chain.len();
        for start in 0..len {
            for end in (start + 2)..=len {
                let shape: Vec<String> = sc.chain[start..end].to_vec();
                let entry = shape_sessions.entry(shape).or_default();
                if !entry.contains(&sc.session_id) {
                    entry.push(sc.session_id.clone());
                }
            }
        }
    }

    // For length-≥3 chains, also match against sessions whose chains
    // include the shape with ≤ 1 skipped step. We do this by walking
    // each session chain and checking if a candidate shape is a
    // subsequence with at most one "skip" (one extra element between
    // matched positions).
    let mut candidates: Vec<RecurringChain> = Vec::new();
    for (shape, exact_sessions) in &shape_sessions {
        let mut matched: Vec<String> = exact_sessions.clone();
        if shape.len() >= 3 {
            for sc in sessions {
                if matched.contains(&sc.session_id) {
                    continue;
                }
                if matches_with_one_skip(&sc.chain, shape) {
                    matched.push(sc.session_id.clone());
                }
            }
        }
        matched.sort();
        if matched.len() >= RECURRENCE_THRESHOLD {
            candidates.push(RecurringChain {
                chain_id: chain_id(shape),
                chain: shape.clone(),
                occurrences: matched.len(),
                session_ids: matched,
            });
        }
    }

    // Longest-chain wins: when a candidate's session set is fully
    // contained within a longer candidate's session set, drop the
    // shorter (it's a sub-chain re-proposing the same pattern).
    candidates.sort_by(|a, b| b.chain.len().cmp(&a.chain.len()));
    let mut kept: Vec<RecurringChain> = Vec::new();
    for cand in candidates {
        let dominated = kept.iter().any(|longer| {
            // A "sub-chain" of `longer.chain` whose session set is a
            // subset of `longer.session_ids` is dominated.
            is_subsequence(&longer.chain, &cand.chain)
                && cand
                    .session_ids
                    .iter()
                    .all(|s| longer.session_ids.contains(s))
        });
        if !dominated {
            kept.push(cand);
        }
    }
    // Stable sort for deterministic output: chain_id ascending.
    kept.sort_by(|a, b| a.chain_id.cmp(&b.chain_id));
    kept
}

/// True when `needle` is a contiguous subslice of `haystack`, allowing
/// at most one inserted element (a "skip"). Used by
/// [`detect_recurring_chains`] for length-≥3 relaxation per AC-02.
fn matches_with_one_skip(haystack: &[String], needle: &[String]) -> bool {
    if needle.len() < 2 {
        return false;
    }
    if haystack.len() < needle.len() {
        return false;
    }
    // Slide a window of length needle.len() + 1 (allows one skip).
    let window = needle.len() + 1;
    if haystack.len() < window {
        return false;
    }
    for start in 0..=(haystack.len() - window) {
        let win = &haystack[start..start + window];
        // Try each position as the skipped one.
        for skip in 0..win.len() {
            let mut iter_h = win.iter().enumerate().filter(|(i, _)| *i != skip).map(|(_, v)| v);
            let mut iter_n = needle.iter();
            let mut ok = true;
            loop {
                match (iter_h.next(), iter_n.next()) {
                    (Some(h), Some(n)) if h == n => continue,
                    (None, None) => break,
                    _ => {
                        ok = false;
                        break;
                    }
                }
            }
            if ok {
                return true;
            }
        }
    }
    false
}

/// True when `needle` is a contiguous subslice of `haystack`.
fn is_subsequence(haystack: &[String], needle: &[String]) -> bool {
    if needle.len() > haystack.len() {
        return false;
    }
    haystack.windows(needle.len()).any(|w| w == needle)
}

/// AC-04: compute the routine's `chain_id`.
///
/// Algorithm: JCS-canonical-JSON encoding of the ordered skill_id
/// sequence, then SHA-256 hex digest.
///
/// For an array of strings, JCS (RFC 8785) reduces to a deterministic
/// byte serialisation: `[` `<elem>` `,` `<elem>` `]` with no whitespace
/// and `<elem>` being the JSON string with the minimal escape set. We
/// emit the same bytes `serde_json` does for an array of plain ASCII
/// strings (the orbit skill_ids are all ASCII), which is JCS-compatible
/// in the strings-only-array subset we use.
///
/// Worked example: for `["/orb:tabletop","/orb:spec","/orb:implement"]`
/// the canonical bytes are exactly that string (no whitespace), and the
/// chain_id is the SHA-256 hex digest of those UTF-8 bytes.
pub fn chain_id(chain: &[String]) -> String {
    let canonical = canonical_json_string_array(chain);
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    hex_encode(&hasher.finalize())
}

/// Emit JCS-canonical JSON for an array of strings. The orbit skill_id
/// surface uses plain ASCII slugs; the JCS subset for `string` in such
/// inputs matches `serde_json::to_string` byte-for-byte.
fn canonical_json_string_array(items: &[String]) -> String {
    let mut out = String::from("[");
    for (i, s) in items.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        // Escape per JSON rules; the JCS profile picks the minimal
        // escape set (`\"`, `\\`, `\b`, `\f`, `\n`, `\r`, `\t`, `\uXXXX`
        // for the < 0x20 range and the lone-surrogate range). For
        // skill_ids this collapses to the trivial wrap-in-quotes case.
        let encoded = serde_json::to_string(s).expect("string encodes");
        out.push_str(&encoded);
    }
    out.push(']');
    out
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push(HEX[(b >> 4) as usize] as char);
        out.push(HEX[(b & 0x0f) as usize] as char);
    }
    out
}

/// Front-matter parse error categorised so the verb layer can re-tag
/// it with the right `verb` field.
#[derive(Debug)]
pub struct FrontMatterParseError(pub String);

/// Parse a SKILL.md body, extract its YAML front-matter, and validate
/// it against [`RoutineFrontMatter`]. Errors are returned as
/// [`FrontMatterParseError`] so the verb wrapper assigns the correct
/// verb name in the envelope.
///
/// Front-matter delimiter: `---\n` lines bracket the YAML block at
/// the top of the file. Anything before the first `---` is rejected.
pub fn parse_front_matter(body: &str) -> std::result::Result<RoutineFrontMatter, FrontMatterParseError> {
    let trimmed = body.trim_start_matches('\u{feff}');
    let after_marker = trimmed
        .strip_prefix("---\n")
        .ok_or_else(|| FrontMatterParseError(
            "SKILL.md must start with '---\\n' front-matter marker".into(),
        ))?;
    let end = after_marker
        .find("\n---")
        .ok_or_else(|| FrontMatterParseError(
            "SKILL.md front-matter is unterminated (no closing '---')".into(),
        ))?;
    let yaml = &after_marker[..end];
    let parsed: RoutineFrontMatter = serde_yaml::from_str(yaml)
        .map_err(|e| FrontMatterParseError(format!("front-matter yaml: {e}")))?;
    // Validate the additive invariants.
    if parsed.created_by != "agent" && parsed.created_by != "human" {
        return Err(FrontMatterParseError(format!(
            "created_by must be 'agent' or 'human', got '{}'",
            parsed.created_by
        )));
    }
    if parsed.chain.len() < 2 {
        return Err(FrontMatterParseError(
            "chain must contain ≥ 2 skill_ids (single-skill routines aren't chains)".into(),
        ));
    }
    let expected = chain_id(&parsed.chain);
    if parsed.chain_id != expected {
        return Err(FrontMatterParseError(format!(
            "chain_id mismatch: declared {} but recomputed {}",
            parsed.chain_id, expected,
        )));
    }
    Ok(parsed)
}

/// Render a [`RoutineFrontMatter`] block plus the agent-supplied body
/// to a SKILL.md document. The front-matter ordering is fixed so byte
/// drift between writes is detectable.
pub fn render_skill_md(fm: &RoutineFrontMatter, body: &str) -> String {
    // We render the front-matter directly rather than going through
    // serde_yaml so the key order is stable across serde upgrades.
    let mut out = String::new();
    out.push_str("---\n");
    out.push_str(&format!("name: {}\n", fm.name));
    out.push_str(&format!("description: {}\n", yaml_escape(&fm.description)));
    out.push_str(&format!("created_by: {}\n", fm.created_by));
    out.push_str(&format!("created_at: {}\n", fm.created_at));
    out.push_str(&format!("pinned: {}\n", fm.pinned));
    out.push_str(&format!("last_verified: {}\n", fm.last_verified));
    out.push_str(&format!("chain_id: {}\n", fm.chain_id));
    out.push_str("chain:\n");
    for step in &fm.chain {
        out.push_str(&format!("  - {}\n", yaml_escape(step)));
    }
    out.push_str("---\n");
    if !body.starts_with('\n') {
        out.push('\n');
    }
    out.push_str(body);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Quote a YAML scalar when it contains characters that would otherwise
/// be ambiguous. Skill_ids start with `/` so they always need quoting.
fn yaml_escape(s: &str) -> String {
    let needs_quote = s.is_empty()
        || s.starts_with(|c: char| {
            matches!(c, '/' | '@' | '%' | '!' | '#' | '&' | '*' | ',' | '?' | '|' | '>' | '-' | ':' | '[' | ']' | '{' | '}' | '"' | '\'')
        })
        || s.contains(':')
        || s.contains('#')
        || s.contains('\n');
    if needs_quote {
        // Use JSON-style double-quoted to keep escapes simple.
        serde_json::to_string(s).expect("string encodes")
    } else {
        s.to_string()
    }
}

/// AC-09: scan `.claude/skills/` and `.claude/skills/.archive/` for any
/// SKILL.md whose `chain_id` front-matter field matches `target`.
/// Returns the path to the matching file, or `None` if none exists.
///
/// The lookup is path-independent — author renames of the routine
/// directory don't change the `chain_id`, so subsequent detection still
/// recognises the chain as already-authored. Archived routines also
/// match (curator move to `.archive/<slug>/`) so an author archive
/// permanently silences re-authoring of the same chain.
pub fn existing_routine_for_chain(claude_skills_dir: &Path, target: &str) -> Result<Option<PathBuf>> {
    const VERB: &str = "routine.existing";
    let mut roots: Vec<PathBuf> = Vec::new();
    if claude_skills_dir.exists() {
        roots.push(claude_skills_dir.to_path_buf());
    }
    let archive = claude_skills_dir.join(".archive");
    if archive.exists() {
        roots.push(archive);
    }
    for root in roots {
        let entries = match std::fs::read_dir(&root) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries {
            let entry = entry.map_err(|e| {
                Error::unavailable(VERB, format!("read {}: {e}", root.display()))
            })?;
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            // Skip the .archive entry when iterating the top-level dir;
            // we visit it explicitly above.
            if path.file_name().and_then(|s| s.to_str()) == Some(".archive") {
                continue;
            }
            let skill_md = path.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let body = match std::fs::read_to_string(&skill_md) {
                Ok(b) => b,
                Err(_) => continue,
            };
            let fm = match parse_front_matter(&body) {
                Ok(f) => f,
                Err(_) => continue,
            };
            if fm.chain_id == target {
                return Ok(Some(skill_md));
            }
        }
    }
    Ok(None)
}

/// Suggest a directory name for a new routine given its chain. The
/// name is derived from the chain — e.g.
/// `["/orb:tabletop","/orb:spec"]` becomes `tabletop-spec`. Stripping
/// the `/orb:` prefix keeps names short and human-readable; the author
/// may rename freely (the `chain_id` lookup is path-independent).
pub fn default_routine_name(chain: &[String]) -> String {
    let parts: Vec<String> = chain
        .iter()
        .map(|step| {
            step.trim_start_matches('/')
                .splitn(2, ':')
                .nth(1)
                .unwrap_or(step.as_str())
                .replace(['/', '\\', ' '], "-")
        })
        .collect();
    parts.join("-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_id_matches_canonical_example() {
        // Spec AC-04 worked example: SHA-256 of the canonical JSON for
        // ["/orb:tabletop","/orb:spec","/orb:implement"].
        let chain = vec![
            "/orb:tabletop".to_string(),
            "/orb:spec".to_string(),
            "/orb:implement".to_string(),
        ];
        let id = chain_id(&chain);
        // Recompute outside the function to confirm the canonical
        // bytes match what we expect.
        let canonical = r#"["/orb:tabletop","/orb:spec","/orb:implement"]"#;
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let expected = hex_encode(&hasher.finalize());
        assert_eq!(id, expected);
        // Length and hex-shape sanity.
        assert_eq!(id.len(), 64, "SHA-256 hex digest is 64 chars");
        assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn chain_id_is_deterministic_across_calls() {
        let chain = vec!["a".into(), "b".into(), "c".into()];
        let a = chain_id(&chain);
        let b = chain_id(&chain);
        assert_eq!(a, b);
    }

    #[test]
    fn chain_id_differs_on_reorder() {
        let abc = chain_id(&["a".into(), "b".into(), "c".into()]);
        let acb = chain_id(&["a".into(), "c".into(), "b".into()]);
        assert_ne!(abc, acb, "ordering is significant per ac-04");
    }

    #[test]
    fn canonical_json_for_strings_is_jcs_compatible() {
        let s = canonical_json_string_array(&["a".into(), "b".into()]);
        assert_eq!(s, r#"["a","b"]"#);
    }

    #[test]
    fn default_name_drops_orb_prefix() {
        let n = default_routine_name(&[
            "/orb:tabletop".into(),
            "/orb:spec".into(),
            "/orb:implement".into(),
        ]);
        assert_eq!(n, "tabletop-spec-implement");
    }

    #[test]
    fn matches_with_one_skip_finds_skipped_step() {
        let haystack: Vec<String> = ["a", "b", "x", "c"].iter().map(|s| s.to_string()).collect();
        let needle: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert!(matches_with_one_skip(&haystack, &needle));
    }

    #[test]
    fn matches_with_one_skip_rejects_two_skips() {
        let haystack: Vec<String> = ["a", "x", "b", "y", "c"].iter().map(|s| s.to_string()).collect();
        let needle: Vec<String> = ["a", "b", "c"].iter().map(|s| s.to_string()).collect();
        assert!(!matches_with_one_skip(&haystack, &needle));
    }

    #[test]
    fn render_and_parse_roundtrips() {
        let chain = vec![
            "/orb:tabletop".to_string(),
            "/orb:spec".to_string(),
            "/orb:implement".to_string(),
        ];
        let fm = RoutineFrontMatter {
            name: "tabletop-spec-implement".into(),
            description: "Chain routine".into(),
            created_by: "agent".into(),
            created_at: "2026-05-22T10:00:00Z".into(),
            pinned: false,
            last_verified: "2026-05-22T10:00:00Z".into(),
            chain_id: chain_id(&chain),
            chain: chain.clone(),
        };
        let body = render_skill_md(&fm, "# Routine body\n");
        let parsed = parse_front_matter(&body).expect("roundtrip parse");
        assert_eq!(parsed, fm);
    }

    #[test]
    fn parse_rejects_missing_front_matter() {
        let err = parse_front_matter("no marker here\n").unwrap_err();
        assert!(err.0.contains("front-matter marker"));
    }

    #[test]
    fn parse_rejects_chain_id_mismatch() {
        let chain = vec!["/orb:a".to_string(), "/orb:b".to_string()];
        let bad_id = "0".repeat(64);
        let fm = RoutineFrontMatter {
            name: "x".into(),
            description: "x".into(),
            created_by: "agent".into(),
            created_at: "2026-05-22T10:00:00Z".into(),
            pinned: false,
            last_verified: "2026-05-22T10:00:00Z".into(),
            chain_id: bad_id,
            chain,
        };
        // We construct the body manually because render_skill_md doesn't
        // gate on chain_id validity (only parse does).
        let body = render_skill_md(&fm, "body\n");
        let err = parse_front_matter(&body).unwrap_err();
        assert!(err.0.contains("chain_id mismatch"));
    }
}
