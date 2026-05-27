//! Entity schemas — strongly-typed, `deny_unknown_fields` everywhere.
//!
//! Per ac-01 the parser MUST reject (not silently drop) unknown fields. This is
//! the single annotation that prevents the lossy-parse failure mode where
//! deserialise + reserialise stays byte-identical because the dropped field is
//! absent from both sides.
//!
//! Per ac-04 the schema-version file is a substrate-written entity classified
//! under `values.enforcement.substrate_written` — schema drift in the version
//! file silently breaks every migration, so it gets the same strict treatment.
//!
//! Layout on disk:
//! - `.orbit/schema-version`                       — single-line entity, opaque to git
//! - `.orbit/specs/<id>.yaml`                      — Spec (substrate-written)
//! - `.orbit/cards/<slug>.yaml`                    — Card (human-written; CI validated)
//! - `.orbit/choices/<slug>.yaml`                  — Choice (human-written; CI validated)
//! - `.orbit/memories/<slug>.yaml`                 — Memory (substrate-written)
//! - `.orbit/sessions/<session-id>.yaml`           — Session (substrate-written)
//! - `.orbit/specs/<id>.tasks.jsonl`               — Task event stream (append-only)
//! - `.orbit/specs/<id>.notes.jsonl`               — Note event stream (append-only)
//! - `.orbit/skills/<skill_id>.invocations.jsonl`  — Skill invocation stream (append-only)
//!
//! Tasks, notes, and skill invocations are intentionally append-only JSONL —
//! they are not round-trippable as a unit and are excluded from the CI
//! round-trip gate per ac-16.

use serde::{Deserialize, Serialize};

// ============================================================================
// schema-version
// ============================================================================

/// The on-disk schema version. Read first by the migration runner on every
/// invocation (per `values.enforcement` rationale).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaVersion {
    /// Major.minor schema identifier, e.g. `"0.1"`.
    pub version: String,
    /// Human-readable note attached to the version (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

// ============================================================================
// Spec
// ============================================================================

impl Spec {
    /// Canonical top-level field set for the audit-drift verb (see
    /// `orbit audit drift`). Kept in lockstep with the struct via the
    /// `spec_fields_matches_struct` unit test in this module — adding a
    /// new field to `Spec` requires extending both the const and the
    /// test fixture, so the audit's allow-set cannot silently drift from
    /// the canonical schema.
    pub const FIELDS: &'static [&'static str] = &[
        "id",
        "goal",
        "cards",
        "status",
        "labels",
        "acceptance_criteria",
        "memories_considered",
        "closed_at",
    ];
}

impl Card {
    pub const FIELDS: &'static [&'static str] = &[
        "id",
        "feature",
        "as_a",
        "i_want",
        "so_that",
        "goal",
        "maturity",
        "park",
        "scenarios",
        "specs",
        "relations",
        "references",
        "notes",
    ];
}

impl ParkSignal {
    pub const FIELDS: &'static [&'static str] = &["reason", "until"];
}

/// Reject empty strings at parse time. Used by `ParkSignal` so a card carrying
/// `park: {reason: "", until: "..."}` fails the strict parse rather than
/// silently emitting a meaningless hold signal.
fn non_empty_string<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    if s.is_empty() {
        return Err(serde::de::Error::custom("must not be empty"));
    }
    Ok(s)
}

impl Choice {
    pub const FIELDS: &'static [&'static str] = &[
        "id",
        "title",
        "status",
        "date_created",
        "date_modified",
        "body",
        "references",
    ];
}

impl Memory {
    pub const FIELDS: &'static [&'static str] = &["key", "body", "timestamp", "labels", "cites"];
}

impl Cite {
    pub const FIELDS: &'static [&'static str] = &["path"];
}

impl CiteEvidence {
    pub const FIELDS: &'static [&'static str] = &["cite_path", "excerpt", "read_at"];
}

impl SkillInvocation {
    pub const FIELDS: &'static [&'static str] =
        &["skill_id", "session_id", "outcome", "correction", "timestamp"];
}

impl Session {
    pub const FIELDS: &'static [&'static str] = &[
        "id",
        "started_at",
        "ended_at",
        "distillate",
        "card_id",
        "labels",
    ];
}

impl AcceptanceCriterion {
    pub const FIELDS: &'static [&'static str] =
        &["id", "description", "gate", "checked", "verification", "ac_type"];
}

impl Scenario {
    pub const FIELDS: &'static [&'static str] = &["name", "given", "when", "then", "gate"];
}

impl Relation {
    pub const FIELDS: &'static [&'static str] = &["card", "choice", "type", "reason"];

    /// Validate that exactly one of `card` / `choice` is set. Per spec
    /// 2026-05-25-relation-schema-choice-targets ac-02 — Relation supports
    /// both card-target and choice-target edges via separate optional
    /// fields; neither/both is a malformed entry.
    pub fn validate(&self) -> std::result::Result<(), crate::error::Error> {
        use crate::error::Error;
        const VERB: &str = "card.parse";
        match (&self.card, &self.choice) {
            (Some(_), Some(_)) => Err(Error::malformed(
                VERB,
                "relation has both `card:` and `choice:` set — pick one",
            )),
            (None, None) => Err(Error::malformed(
                VERB,
                "relation has neither `card:` nor `choice:` set — pick one",
            )),
            _ => Ok(()),
        }
    }
}

impl Card {
    /// Validate every `Relation` entry post-parse. Called from every site
    /// that parses a Card YAML — keeps derive-driven `deny_unknown_fields`
    /// intact while enforcing the exactly-one-target invariant per spec
    /// 2026-05-25-relation-schema-choice-targets ac-02.
    pub fn validate_relations(&self) -> std::result::Result<(), crate::error::Error> {
        for r in &self.relations {
            r.validate()?;
        }
        Ok(())
    }
}

impl Config {
    pub const FIELDS: &'static [&'static str] = &["docs", "plugin_version", "plugin_repo"];
}

impl DocsConfig {
    pub const FIELDS: &'static [&'static str] = &["topology"];
}

impl TopologyEntry {
    pub const FIELDS: &'static [&'static str] = &[
        "subsystem",
        "canonical_code",
        "decision_record",
        "operational_doc",
        "test_surface",
    ];

    /// Minimum length for the `subsystem` slug. Mirrors the ≥ 5 char filter
    /// applied by `spec_close` topology_warnings word-boundary heuristic so
    /// short common tokens cannot collide with subsystem names.
    pub const MIN_SUBSYSTEM_LEN: usize = 5;

    /// Validate non-serde invariants — slug shape and minimum length on the
    /// subsystem key, non-empty canonical_code list. Returns the first
    /// validation error encountered (or `Ok(())`). Called by `verify_all`'s
    /// topology branch after serde parsing succeeds.
    pub fn validate(&self) -> Result<(), String> {
        if self.subsystem.len() < Self::MIN_SUBSYSTEM_LEN {
            return Err(format!(
                "subsystem slug `{}` is below the minimum length of {} characters",
                self.subsystem,
                Self::MIN_SUBSYSTEM_LEN
            ));
        }
        if !is_slug_shaped(&self.subsystem) {
            return Err(format!(
                "subsystem slug `{}` is not slug-shaped (lower-case letters, digits, and hyphens only; \
                 first char must be a letter; no leading/trailing/double hyphens)",
                self.subsystem
            ));
        }
        if self.canonical_code.is_empty() {
            return Err(format!(
                "topology entry `{}` has no canonical_code pointers — entries without code pointers \
                 are not load-bearing",
                self.subsystem
            ));
        }
        Ok(())
    }
}

/// Slug-shape predicate for topology subsystem keys: lower-case letters,
/// digits, and hyphens; first character must be a letter; no leading,
/// trailing, or double hyphens.
fn is_slug_shaped(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return false;
    }
    if !bytes[0].is_ascii_lowercase() {
        return false;
    }
    if bytes[bytes.len() - 1] == b'-' {
        return false;
    }
    let mut prev_hyphen = false;
    for &b in bytes {
        let ok = b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-';
        if !ok {
            return false;
        }
        if b == b'-' && prev_hyphen {
            return false;
        }
        prev_hyphen = b == b'-';
    }
    true
}

/// A discrete unit of work with numbered acceptance criteria. Substrate-written.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    /// Slug-style identifier (e.g. `"2026-05-07-orbit-state-v0.1"`).
    pub id: String,
    /// One-sentence statement of what shipping this spec achieves.
    pub goal: String,
    /// Cards this spec advances by closure. Empty list is rare but legal.
    #[serde(default)]
    pub cards: Vec<String>,
    /// Status — `open` until close; close requires all child tasks done.
    pub status: SpecStatus,
    /// Free-text labels (`spec`, `experimental`, etc.) — matches bd's label model.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Acceptance criteria, in declaration order. Gate ACs block subsequent ones.
    #[serde(default)]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Memories the spec author reconciled at design time. Each entry names
    /// the memory key, the adoption disposition, and a short reason. Per
    /// spec 2026-05-19-memory-gates-decisions ac-03 (D3a). Read by
    /// `spec.close` to decide whether matching memories are unreconciled
    /// (D4). `#[serde(default, skip_serializing_if = "Vec::is_empty")]`
    /// keeps existing specs byte-identical on disk — the field only
    /// materialises when memories were actually considered.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub memories_considered: Vec<MemoryReconciliation>,
    /// ISO-8601 / RFC 3339 timestamp recorded by `spec.close` when the
    /// spec transitions to `closed`. `None` on open specs and on closed
    /// specs that predate this field (no backfill — see spec
    /// 2026-05-26-scope-discipline-front-loaded ac-10). Read by the
    /// card-coverage audit family (ac-04 / ac-05) to gate the
    /// audit-window: closed specs with `None` or `closed_at` strictly
    /// before the introduction-date constant are excluded.
    ///
    /// `#[serde(default, skip_serializing_if = "Option::is_none")]`
    /// preserves byte-identical canonical output for the dominant
    /// pre-shipped case — older closed specs deserialise as `None` and
    /// re-serialise without an empty field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpecStatus {
    Open,
    Closed,
}

/// One reconciliation record on `Spec.memories_considered`. Stores which
/// memory was reviewed at design time, the disposition (adopted, partially
/// adopted, or not applicable), and a one-sentence reason. Per spec
/// 2026-05-19-memory-gates-decisions ac-03 (D3a).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryReconciliation {
    /// The memory key being reconciled (e.g. `"drive-autonomy-default-to-action"`).
    pub key: String,
    /// How the memory was applied.
    pub disposition: ReconciliationDisposition,
    /// One-sentence rationale — why the memory was adopted, partially
    /// adopted, or considered not applicable.
    pub reason: String,
    /// Evidence that the spec author read the memory's cited sources. One
    /// entry per cite on the matched memory (cite_path + excerpt drawn
    /// from the cite's file contents + RFC3339 read_at). Empty or absent
    /// on entries for memories without cites. `spec.close`'s second-pass
    /// pre-flight (per spec 2026-05-27-memory-cite-reading ac-04) refuses
    /// closure when any cite_path on a referenced memory lacks a
    /// corresponding evidence entry here.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cite_evidence: Vec<CiteEvidence>,
}

/// Adoption status for one reconciled memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReconciliationDisposition {
    /// The memory's mechanism is followed in this spec.
    Adopted,
    /// Part of the memory applies; the reason names the divergence.
    PartiallyAdopted,
    /// The memory matched but is not applicable to this spec's goal.
    NotApplicable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AcceptanceCriterion {
    pub id: String,
    pub description: String,
    #[serde(default)]
    pub gate: bool,
    #[serde(default)]
    pub checked: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification: Option<String>,
    /// The kind of evidence that closes this AC. Drives spec.close's
    /// two-band behaviour (Code/Config/Doc block close when unchecked;
    /// Ops/Observation legitimately defer) per spec
    /// 2026-05-16-ac-taxonomy.
    ///
    /// `#[serde(default)]` keeps untyped legacy corpora parseable — they
    /// deserialise as `AcType::Code` (matches the implicit assumption
    /// every untyped AC carried before this field shipped).
    /// `skip_serializing_if = "AcType::is_code"` preserves byte-identical
    /// canonical output for the dominant Code case so the migration
    /// touches only ACs that need a non-default value.
    #[serde(default, skip_serializing_if = "AcType::is_code")]
    pub ac_type: AcType,
}

/// The kind of evidence that closes an AC. Drives spec.close's two-band
/// behaviour: `Code`, `Config`, `Doc` block close when unchecked;
/// `Ops`, `Observation` are deferrable (the spec is allowed to close with
/// them open). Per spec 2026-05-16-ac-taxonomy.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AcType {
    /// Closes on a passing test, referenced commit, or functional
    /// artefact. The default — matches the implicit assumption every
    /// untyped AC carried before this field shipped.
    #[default]
    Code,
    /// Closes on a config or external-system-state change verifiable by
    /// grep, file inspection, or external query.
    Config,
    /// Closes on a written artefact (CLAUDE.md edit, card text, memo,
    /// MADR).
    Doc,
    /// Closes on operator action with a captured log line, signoff, or
    /// dashboard check. Legitimately deferred at spec.close.
    Ops,
    /// Closes on a dated window of empirical measurement (post-cutover
    /// soak, eval-run output, training-completes-and-produces-metrics).
    /// Legitimately deferred at spec.close.
    Observation,
}

impl AcType {
    /// True when an unchecked AC of this kind blocks `spec.close`.
    /// `Code`, `Config`, `Doc` close on artefacts that exist at commit
    /// time (a passing test, a file diff, written prose); leaving them
    /// unchecked is premature closure. `Ops`, `Observation` close on
    /// events that happen after the spec's other work lands (an
    /// operator signoff, a dated metric window); the spec is allowed to
    /// close with them open and they appear in the deferrable-open
    /// list returned by `spec.close`.
    pub fn blocks_close(&self) -> bool {
        matches!(self, Self::Code | Self::Config | Self::Doc)
    }

    /// Predicate used by `#[serde(skip_serializing_if = ...)]` on
    /// `AcceptanceCriterion::ac_type` so the dominant Code case stays
    /// byte-identical to today's canonical output.
    pub fn is_code(&self) -> bool {
        matches!(self, Self::Code)
    }
}

// ============================================================================
// Task (append-only event)
// ============================================================================

/// One event in a task's append-only JSONL stream. State is reconstructed by
/// reducing events for a `task_id` and taking the last one (per ac-07).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskEvent {
    /// Logical task identifier (stable across events).
    pub task_id: String,
    /// The spec this task belongs to.
    pub spec_id: String,
    /// What happened at this event.
    pub event: TaskEventKind,
    /// Free-text body for the event (e.g. open description, update note).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Free-text labels (e.g. `skill-author`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// ISO-8601 timestamp, written by the substrate at event-append time.
    pub timestamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskEventKind {
    Open,
    Claim,
    Update,
    Done,
}

// ============================================================================
// Spec note (append-only event)
// ============================================================================

// ============================================================================
// Skill invocation (append-only event)
// ============================================================================

/// One row in a skill's append-only invocation log. Recurrence detection
/// (per spec 2026-05-15-agent-learning-loop ac-04) reduces the file by
/// counting rows per [`InvocationOutcome`].
///
/// Layout: `.orbit/skills/<skill_id>.invocations.jsonl`. Excluded from
/// the CI round-trip gate for the same reason tasks and notes are:
/// append-only streams aren't round-trippable as a unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SkillInvocation {
    /// Skill slug — matches the directory name under `plugins/orb/skills/`.
    pub skill_id: String,
    /// Session this invocation belongs to (sourced via `read_session_id`).
    pub session_id: String,
    /// What happened when the agent invoked the skill.
    pub outcome: InvocationOutcome,
    /// Free-text record of what went wrong (or what was corrected). Drives
    /// the SKILL.md edit decision once the recurrence threshold is met —
    /// the count tells the agent *whether*, the corrections tell it *what*.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<String>,
    /// ISO-8601 / RFC 3339 timestamp written by the substrate at append time.
    pub timestamp: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InvocationOutcome {
    /// Skill ran end-to-end and produced the intended result.
    Worked,
    /// Skill ran but the result needed correction.
    Partial,
    /// Skill was invoked in a context the SKILL.md does not cover.
    DidntApply,
    /// Skill produced a wrong result.
    Incorrect,
}

// ============================================================================
// Spec note (append-only event)
// ============================================================================

/// One note appended to a spec via `spec.note`. Lives in the same
/// append-only family as [`TaskEvent`] — JSONL stream, ordered by
/// position-in-file, never rewritten in place.
///
/// Layout: `.orbit/specs/<spec_id>.notes.jsonl`. Excluded from the CI
/// round-trip gate (ac-16) for the same reason tasks are: append-only
/// streams aren't round-trippable as a unit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NoteEvent {
    /// Spec this note attaches to.
    pub spec_id: String,
    /// Free-text body. Multi-line strings are escaped per JSON rules.
    pub body: String,
    /// Free-text labels (e.g. `migrated-from-bd`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// ISO-8601 / RFC 3339 timestamp written by the substrate at append time.
    /// Migration tools may pre-supply this when porting historical notes.
    pub timestamp: String,
}

// ============================================================================
// Card (human-written; CI-validated)
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Card {
    /// Full slug (e.g. `0008-consolidated-orbit-artefact-folder`) — must equal
    /// the filename minus `.yaml`. Optional for backwards compatibility with
    /// pre-choice-0022 cards; the canonical writer fills it from the filename
    /// on the next canonicalise pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub feature: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub as_a: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub i_want: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub so_that: Option<String>,
    pub goal: String,
    pub maturity: CardMaturity,
    /// Deliberate-hold signal. When present, conformance audit's card-state
    /// finding family skips this card (per spec 2026-05-20-conformance-park-signal).
    /// Removing the block returns the card to the audit's view. `reason:` and
    /// `until:` are both free-form prose; v1 has no automated unpark on date
    /// or spec-id resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub park: Option<ParkSignal>,
    #[serde(default)]
    pub scenarios: Vec<Scenario>,
    /// Spec paths advanced by this card. Substrate appends here on `spec.close`.
    #[serde(default)]
    pub specs: Vec<String>,
    #[serde(default)]
    pub relations: Vec<Relation>,
    #[serde(default)]
    pub references: Vec<String>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardMaturity {
    Planned,
    Emerging,
    Established,
}

/// A deliberate-hold signal on a Card. Both fields are free-form prose,
/// non-empty at parse time. See `Card.park` for full semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ParkSignal {
    /// Free-form prose explaining why the card is held.
    #[serde(deserialize_with = "non_empty_string")]
    pub reason: String,
    /// Free-form prose naming the unhold condition.
    #[serde(deserialize_with = "non_empty_string")]
    pub until: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scenario {
    pub name: String,
    pub given: String,
    pub when: String,
    pub then: String,
    #[serde(default)]
    pub gate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Relation {
    /// Card-target edge: full slug (e.g. `0019-tabletop`). Mutually
    /// exclusive with `choice` — exactly one of the two is set per entry.
    /// `Relation::validate()` enforces this post-parse (called from every
    /// Card-parse site). `skip_serializing_if` preserves byte-equal
    /// serialisation of existing card-target relations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<String>,
    /// Choice-target edge: bare numeric id (e.g. `'0020'`, matching
    /// `Choice.id` on disk per `.orbit/conventions/id-conventions.md`).
    /// Mutually exclusive with `card`. Per spec
    /// 2026-05-25-relation-schema-choice-targets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub choice: Option<String>,
    #[serde(rename = "type")]
    pub kind: RelationKind,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RelationKind {
    DependsOn,
    Feeds,
    Supersedes,
    SupersededBy,
    /// A card honours a choice's policy. Pairs with a `choice:` target;
    /// kind-vs-target coherence is not validated in v1 per spec
    /// 2026-05-25-relation-schema-choice-targets ac-03.
    Respects,
}

// ============================================================================
// Choice (human-written; CI-validated)
// ============================================================================

/// A choice (architectural decision in MADR shape). Human-written; the CI
/// round-trip gate (ac-16) is the format-integrity enforcement mechanism.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub title: String,
    pub status: ChoiceStatus,
    pub date_created: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date_modified: Option<String>,
    /// MADR body — multi-line prose. The choice fixture suite (ac-01) covers
    /// the round-trip edge cases for this field.
    pub body: String,
    #[serde(default)]
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChoiceStatus {
    Proposed,
    Accepted,
    Rejected,
    Deprecated,
    Superseded,
}

// ============================================================================
// Memory
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Memory {
    pub key: String,
    pub body: String,
    pub timestamp: String,
    #[serde(default)]
    pub labels: Vec<String>,
    /// Authoritative local files this memory cites as load-bearing for the
    /// fix it describes. Opt-in additive: cite-less memories store no
    /// entries here and operate exactly as before this field shipped (per
    /// spec 2026-05-27-memory-cite-reading ac-01). `#[serde(default)]` lets
    /// existing on-disk YAMLs deserialise as `cites: vec![]` without a
    /// migration; `skip_serializing_if = "Vec::is_empty"` keeps those same
    /// files byte-identical on round-trip — `cites: []` never materialises
    /// on disk.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cites: Vec<Cite>,
}

/// One cite entry on a [`Memory`]. Names a local file path that carries the
/// mechanical detail the memory body summarised. Read by `/orb:spec`'s
/// cite-read step and enforced by `spec.close`'s second-pass pre-flight
/// (per spec 2026-05-27-memory-cite-reading ac-04).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cite {
    /// Repo-relative path to the authoritative source the memory cites.
    /// V1 carries the string verbatim — no resolution at parse time; the
    /// caller (spec.close pre-flight, /orb:spec cite-read step) reads the
    /// file when it needs the contents.
    pub path: String,
}

/// One cite-read evidence entry on a [`MemoryReconciliation`]. The spec
/// author records what the cite said (1-3 line `excerpt` drawn from the
/// file at `cite_path`) and when they read it (`read_at`, RFC3339).
/// Per spec 2026-05-27-memory-cite-reading ac-03.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CiteEvidence {
    /// The cite path this evidence covers — matches one of the memory's
    /// `cites[].path` entries.
    pub cite_path: String,
    /// Verbatim 1-3 line excerpt drawn from the file at `cite_path`. The
    /// content-derived value closes tabletop halt-condition #2 (cite-read
    /// evidence must carry a content-derived value, not a boolean).
    pub excerpt: String,
    /// RFC3339 timestamp recording when the cite was read.
    pub read_at: String,
}

// ============================================================================
// Config
// ============================================================================

/// Project-level orbit configuration at `.orbit/config.yaml`. Opt-in:
/// absence of the file is tolerated and the rest of orbit-state functions
/// unchanged. When present, `orbit verify` validates the file against this
/// schema. Per spec 2026-05-18-documentation-topology ac-02.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Documentation-surface configuration. Optional — when absent the
    /// topology capability is unconfigured (the audit verb returns
    /// "topology capability not configured").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<DocsConfig>,
    /// Plugin-version pin — semver-compatible string identifying the
    /// plugin release this repo's substrate conforms to. Per spec
    /// 2026-05-19-workflow-conformance ac-05. When `None`, conformance
    /// audits the repo against the currently-installed plugin (the
    /// orbit-state binary's `CARGO_PKG_VERSION` under the lockstep
    /// release contract).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_version: Option<String>,
    /// Marks this repo as the orbit-plugin source repo itself. When `Some(true)`,
    /// `/orb:setup`'s topology-scaffolding step seeds the 5 substrate-typed entries
    /// (`cards`, `choices`, `memories`, `specs-substrate`, `topology`) that describe
    /// orbit's own substrate types. When `None` or `Some(false)`, setup scaffolds an
    /// empty `.orbit/topology/` directory with a README pointing at `/orb:topology` —
    /// the substrate-typed seeds are a category error for any project that isn't the
    /// orbit-plugin source. Per spec 2026-05-24-setup-is-orbit-state-aware ac-12.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plugin_repo: Option<bool>,
}

/// Documentation-surface inner config. Per spec
/// 2026-05-18-documentation-topology ac-03.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct DocsConfig {
    /// DEPRECATED. The `docs.topology` config key is retained as a parse-only
    /// field so brownfield consumer repos that wired topology under orbit
    /// 0.4.19 (spec `2026-05-18-topology-substrate-wires` ac-01) do not
    /// hard-fail `Config::from_str` on session-prime after the substrate
    /// migration. The field is unused — no code path reads it; topology lives
    /// at `.orbit/topology/<subsystem>.yaml` per choice 0025
    /// (`topology-substrate-folder`). Canonical write preserves the field so
    /// `verify_all` sees no drift. A follow-on spec to spec
    /// `2026-05-18-topology-substrate-migration` deletes this field entirely
    /// once consumer-repo soak completes.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topology: Option<String>,
}

// ============================================================================
// Topology
// ============================================================================

/// A topology entry at `.orbit/topology/<subsystem>.yaml`. Per choice 0025
/// (`topology-substrate-folder`) and spec
/// `2026-05-18-topology-substrate-migration` ac-01: per-subsystem yaml,
/// pointer-only, agent-queryable substrate. Fields store opaque strings
/// verbatim — resolution (filesystem existence check, choice-id-to-path
/// translation) is the drift-detector's responsibility, not the parser's.
///
/// The `subsystem` slug is the file stem and the entry's key — it must be
/// slug-shaped and at least 5 characters (mirrors the `≥ 5 char` filter on
/// the spec.close topology_warnings word-boundary heuristic in
/// `verbs.rs::spec_close`, so subsystem names that would be filtered out of
/// that heuristic are also rejected at schema-validation time).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TopologyEntry {
    /// Subsystem slug — kebab-case, lower-case, digits allowed, must match
    /// the file stem at `.orbit/topology/<subsystem>.yaml`. Minimum length
    /// 5 characters.
    pub subsystem: String,
    /// Canonical code paths for the subsystem (typically file or directory
    /// paths). Required and non-empty — a topology entry without a code
    /// pointer is not load-bearing.
    pub canonical_code: Vec<String>,
    /// Decision-record references. Typically choice ids resolved via the
    /// `resolve_numeric_slug(VERB, &layout.choices_dir(), id)` then
    /// `layout.choice_file(&resolved)` two-step pattern; may also be a
    /// direct path. Drift detection tries id-resolution first and falls
    /// through to direct path check.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_record: Vec<String>,
    /// Operational documentation paths — typically the writing SKILL.md
    /// or a substrate convention doc.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub operational_doc: Vec<String>,
    /// Test surface — paths or test-target identifiers covering the
    /// subsystem.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub test_surface: Vec<String>,
}

// ============================================================================
// Session
// ============================================================================

/// A summary record for one agent session. Substrate-written by the
/// `session.distill` verb at session end. Idempotent on `id`: re-running
/// distill on the same session overwrites the same file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    /// Session identifier — typically a UUIDv4 generated by `session.start`
    /// and persisted to `.orbit/.session-id` for the duration of the session.
    pub id: String,
    /// ISO-8601 / RFC 3339 timestamp of the first `session.distill` call.
    /// Preserved across subsequent calls for the same `id`.
    pub started_at: String,
    /// ISO-8601 / RFC 3339 timestamp of the most recent `session.distill`
    /// call. None until distill is first invoked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    /// The agent's end-of-session reflection — free-text markdown.
    pub distillate: String,
    /// Optional card slug scoping this session — populated by
    /// `orbit session set-card <id>` (writes `.orbit/.session-card`) and
    /// then resolved by `orbit session distill` at session end. Absent
    /// from on-disk YAML when None so existing pre-card_id sessions stay
    /// byte-identical. See spec 2026-05-16-session-handover ac-01.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    /// Free-text labels for prime-relevance and search.
    #[serde(default)]
    pub labels: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_rejects_unknown_field() {
        // ac-01 verification: extra unknown field MUST fail parse.
        let yaml = "version: '0.1'\nnote: bootstrap\nunknown_field: oops\n";
        let err = serde_yaml::from_str::<SchemaVersion>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn spec_rejects_unknown_field() {
        let yaml = r#"
id: '0001'
goal: build it
status: open
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<Spec>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn card_rejects_unknown_field() {
        let yaml = r#"
feature: x
goal: y
maturity: planned
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<Card>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn choice_rejects_unknown_field() {
        let yaml = r#"
id: '0001'
title: t
status: accepted
date_created: '2026-05-07'
body: hello
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<Choice>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn memory_rejects_unknown_field() {
        let yaml = r#"
key: k
body: b
timestamp: '2026-05-07T00:00:00Z'
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<Memory>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn task_event_rejects_unknown_field() {
        let json = r#"{"task_id":"t1","spec_id":"s1","event":"open","timestamp":"2026-05-07T00:00:00Z","unknown_field":"oops"}"#;
        let err = serde_json::from_str::<TaskEvent>(json).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    /// Helper: extract sorted top-level keys from a serde_yaml::Value mapping.
    fn top_level_keys(value: &serde_yaml::Value) -> Vec<String> {
        let mut out: Vec<String> = value
            .as_mapping()
            .expect("expected mapping")
            .iter()
            .filter_map(|(k, _)| k.as_str().map(String::from))
            .collect();
        out.sort();
        out
    }

    #[test]
    fn spec_fields_matches_struct() {
        // ac-05 verification: Spec::FIELDS must equal the struct's serde
        // top-level field set. The test populates a fully-populated Spec
        // (every Option=Some, every Vec non-empty) so skip_serializing_if
        // doesn't drop fields, serialises to YAML, and compares the key
        // set against the constant. Adding a new field to Spec requires
        // extending this fixture (the struct literal won't compile
        // otherwise) AND the FIELDS const — drift between the two trips
        // this assertion.
        let spec = Spec {
            id: "id".into(),
            goal: "goal".into(),
            cards: vec!["c".into()],
            status: SpecStatus::Open,
            labels: vec!["l".into()],
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "ac-01".into(),
                description: "d".into(),
                gate: false,
                checked: false,
                verification: Some("v".into()),
                ac_type: AcType::Observation,
            }],
            memories_considered: vec![MemoryReconciliation {
                key: "k".into(),
                disposition: ReconciliationDisposition::Adopted,
                reason: "r".into(),
                cite_evidence: vec![CiteEvidence {
                    cite_path: "docs/foo.md".into(),
                    excerpt: "load-bearing line".into(),
                    read_at: "2026-05-27T00:00:00Z".into(),
                }],
            }],
            closed_at: Some("2026-05-26T00:00:00Z".into()),
        };
        let value = serde_yaml::to_value(&spec).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = Spec::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Spec::FIELDS drifted from struct");
    }

    #[test]
    fn card_fields_matches_struct() {
        let card = Card {
            id: Some("0001-x".into()),
            feature: "f".into(),
            as_a: Some("a".into()),
            i_want: Some("i".into()),
            so_that: Some("s".into()),
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: Some(ParkSignal {
                reason: "r".into(),
                until: "u".into(),
            }),
            scenarios: vec![Scenario {
                name: "n".into(),
                given: "g".into(),
                when: "w".into(),
                then: "t".into(),
                gate: false,
            }],
            specs: vec!["sp".into()],
            relations: vec![Relation {
                card: Some("c".into()),
                choice: None,
                kind: RelationKind::Feeds,
                reason: "r".into(),
            }],
            references: vec!["r".into()],
            notes: vec!["n".into()],
        };
        let value = serde_yaml::to_value(&card).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = Card::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Card::FIELDS drifted from struct");
    }

    #[test]
    fn park_signal_fields_matches_struct() {
        // Mirrors the per-struct FIELDS test pattern. A fully-populated
        // ParkSignal must serialise to exactly the keys named in
        // ParkSignal::FIELDS.
        let signal = ParkSignal {
            reason: "r".into(),
            until: "u".into(),
        };
        let value = serde_yaml::to_value(&signal).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = ParkSignal::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "ParkSignal::FIELDS drifted from struct");
    }

    #[test]
    fn card_round_trips_without_park() {
        // ac-01 (a): card with no `park:` field parses and round-trips
        // byte-identical through the canonical writer. Existing cards on
        // disk must not change shape after this spec lands.
        let yaml = "feature: x\ngoal: y\nmaturity: planned\n";
        let card: Card = serde_yaml::from_str(yaml).unwrap();
        assert!(card.park.is_none());
        let reserialised = crate::canonical::serialise_yaml(&card).unwrap();
        let reparsed: Card = serde_yaml::from_str(&reserialised).unwrap();
        let re_reserialised = crate::canonical::serialise_yaml(&reparsed).unwrap();
        assert_eq!(reserialised, re_reserialised, "round-trip not byte-identical");
    }

    #[test]
    fn card_round_trips_with_park() {
        // ac-01 (b): card with `park: {reason, until}` parses and round-trips
        // byte-identical.
        let yaml = r#"feature: x
goal: y
maturity: planned
park:
  reason: awaiting third use-case
  until: N=2 evidence
"#;
        let card: Card = serde_yaml::from_str(yaml).unwrap();
        let park = card.park.as_ref().expect("park: block should parse");
        assert_eq!(park.reason, "awaiting third use-case");
        assert_eq!(park.until, "N=2 evidence");
        let reserialised = crate::canonical::serialise_yaml(&card).unwrap();
        let reparsed: Card = serde_yaml::from_str(&reserialised).unwrap();
        let re_reserialised = crate::canonical::serialise_yaml(&reparsed).unwrap();
        assert_eq!(reserialised, re_reserialised, "round-trip not byte-identical");
    }

    #[test]
    fn card_rejects_park_with_empty_reason() {
        // ac-01 (c): empty reason → parse error.
        let yaml = r#"feature: x
goal: y
maturity: planned
park:
  reason: ''
  until: y
"#;
        let err = serde_yaml::from_str::<Card>(yaml).unwrap_err();
        assert!(
            err.to_string().contains("must not be empty"),
            "expected empty-string error, got: {err}",
        );
    }

    #[test]
    fn card_rejects_park_with_empty_until() {
        // ac-01 (c-symmetric): empty until → parse error (symmetric coverage
        // per the review-spec MINOR finding).
        let yaml = r#"feature: x
goal: y
maturity: planned
park:
  reason: x
  until: ''
"#;
        let err = serde_yaml::from_str::<Card>(yaml).unwrap_err();
        assert!(
            err.to_string().contains("must not be empty"),
            "expected empty-string error, got: {err}",
        );
    }

    #[test]
    fn card_rejects_park_missing_until() {
        // ac-01 (d): missing until → parse error.
        let yaml = r#"feature: x
goal: y
maturity: planned
park:
  reason: x
"#;
        let err = serde_yaml::from_str::<Card>(yaml).unwrap_err();
        assert!(
            err.to_string().contains("until") || err.to_string().contains("missing"),
            "expected missing-field error, got: {err}",
        );
    }

    #[test]
    fn card_rejects_park_with_unknown_subfield() {
        // ac-01 (e): unknown subfield → parse error (deny_unknown_fields on
        // ParkSignal).
        let yaml = r#"feature: x
goal: y
maturity: planned
park:
  reason: x
  until: y
  extra: z
"#;
        let err = serde_yaml::from_str::<Card>(yaml).unwrap_err();
        assert!(
            err.to_string().contains("unknown"),
            "expected unknown-field error, got: {err}",
        );
    }

    #[test]
    fn choice_fields_matches_struct() {
        let choice = Choice {
            id: "0001".into(),
            title: "t".into(),
            status: ChoiceStatus::Accepted,
            date_created: "2026-05-12".into(),
            date_modified: Some("2026-05-12".into()),
            body: "b".into(),
            references: vec!["r".into()],
        };
        let value = serde_yaml::to_value(&choice).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = Choice::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Choice::FIELDS drifted from struct");
    }

    #[test]
    fn memory_fields_matches_struct() {
        let memory = Memory {
            key: "k".into(),
            body: "b".into(),
            timestamp: "2026-05-12T00:00:00Z".into(),
            labels: vec!["l".into()],
            cites: vec![Cite {
                path: "docs/foo.md".into(),
            }],
        };
        let value = serde_yaml::to_value(&memory).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = Memory::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Memory::FIELDS drifted from struct");
    }

    #[test]
    fn session_fields_matches_struct() {
        // spec 2026-05-15-agent-learning-loop ac-02: Session::FIELDS must
        // equal the struct's serde top-level field set. Fully-populated
        // fixture so skip_serializing_if doesn't drop fields.
        let session = Session {
            id: "5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f".into(),
            started_at: "2026-05-15T12:00:00Z".into(),
            ended_at: Some("2026-05-15T13:00:00Z".into()),
            distillate: "got the loop running".into(),
            card_id: Some("0036-session-handover".into()),
            labels: vec!["loop".into()],
        };
        let value = serde_yaml::to_value(&session).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> =
            Session::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Session::FIELDS drifted from struct");
    }

    #[test]
    fn session_rejects_unknown_field() {
        // spec 2026-05-15-agent-learning-loop ac-02: parser MUST reject
        // unknown fields rather than silently dropping them.
        let yaml = r#"
id: 5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f
started_at: 2026-05-15T12:00:00Z
distillate: hello
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<Session>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn session_round_trips_byte_identical() {
        // spec 2026-05-15-agent-learning-loop ac-02: round-trip lossless on
        // a populated Session (both started_at and ended_at present).
        let session = Session {
            id: "5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f".into(),
            started_at: "2026-05-15T12:00:00Z".into(),
            ended_at: Some("2026-05-15T13:00:00Z".into()),
            distillate: "got the loop running\n".into(),
            card_id: None,
            labels: vec![],
        };
        let yaml = serde_yaml::to_string(&session).unwrap();
        let parsed: Session = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(session, parsed);
    }

    #[test]
    fn session_optional_ended_at_skipped_when_none() {
        // spec 2026-05-15-agent-learning-loop ac-02: ended_at = None must
        // NOT appear as `ended_at: null` on disk — the skip_serializing_if
        // discipline keeps freshly-started session files lean.
        let session = Session {
            id: "5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f".into(),
            started_at: "2026-05-15T12:00:00Z".into(),
            ended_at: None,
            distillate: "".into(),
            card_id: None,
            labels: vec![],
        };
        let yaml = serde_yaml::to_string(&session).unwrap();
        assert!(
            !yaml.contains("ended_at"),
            "expected no `ended_at` key when None; got: {yaml}"
        );
    }

    #[test]
    fn skill_invocation_fields_matches_struct() {
        // spec 2026-05-15-agent-learning-loop ac-01: SkillInvocation::FIELDS
        // must equal the struct's serde top-level field set. Fully-populated
        // fixture so skip_serializing_if doesn't drop fields.
        let inv = SkillInvocation {
            skill_id: "card".into(),
            session_id: "5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f".into(),
            outcome: InvocationOutcome::Worked,
            correction: Some("nudged the wording".into()),
            timestamp: "2026-05-15T12:00:00Z".into(),
        };
        let value = serde_yaml::to_value(&inv).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> =
            SkillInvocation::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "SkillInvocation::FIELDS drifted from struct");
    }

    #[test]
    fn skill_invocation_rejects_unknown_field() {
        // spec 2026-05-15-agent-learning-loop ac-01: parser MUST reject
        // unknown fields rather than silently dropping them.
        let yaml = r#"
skill_id: card
session_id: 5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f
outcome: worked
timestamp: 2026-05-15T12:00:00Z
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<SkillInvocation>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn invocation_outcome_kebab_case_round_trip() {
        // spec 2026-05-15-agent-learning-loop ac-01: kebab-case rename_all
        // must apply to every variant — `didnt-apply` is the only one whose
        // serialised form differs from snake_case, so test it explicitly.
        let parsed: InvocationOutcome = serde_yaml::from_str("didnt-apply").unwrap();
        assert_eq!(parsed, InvocationOutcome::DidntApply);
        let serialised = serde_yaml::to_string(&parsed).unwrap();
        assert!(
            serialised.trim() == "didnt-apply",
            "expected `didnt-apply`, got `{}`",
            serialised.trim()
        );
        // Sanity-check the other variants round-trip.
        for (variant, expected) in [
            (InvocationOutcome::Worked, "worked"),
            (InvocationOutcome::Partial, "partial"),
            (InvocationOutcome::DidntApply, "didnt-apply"),
            (InvocationOutcome::Incorrect, "incorrect"),
        ] {
            let s = serde_yaml::to_string(&variant).unwrap();
            assert_eq!(s.trim(), expected, "variant {variant:?} did not round-trip");
            let back: InvocationOutcome = serde_yaml::from_str(expected).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn skill_invocation_omits_null_correction_on_serialize() {
        // spec 2026-05-15-agent-learning-loop ac-03 / ac-04: a row with no
        // correction must serialise without a `correction:` key at all so
        // the on-disk JSONL line matches the round-trip discipline.
        let inv = SkillInvocation {
            skill_id: "card".into(),
            session_id: "5f6b1a7e-7a32-4f6e-9d31-1a2b3c4d5e6f".into(),
            outcome: InvocationOutcome::Worked,
            correction: None,
            timestamp: "2026-05-15T12:00:00Z".into(),
        };
        let serialised = serde_yaml::to_string(&inv).unwrap();
        assert!(
            !serialised.contains("correction"),
            "expected no `correction` key when None; got: {serialised}"
        );
    }

    #[test]
    fn acceptance_criterion_fields_matches_struct() {
        // ac-04 verification: AcceptanceCriterion::FIELDS must equal the
        // struct's serde top-level field set. Mirrors spec_fields_matches_struct
        // — fully-populated fixture so skip_serializing_if doesn't drop fields.
        let ac = AcceptanceCriterion {
            id: "ac-01".into(),
            description: "d".into(),
            gate: false,
            checked: false,
            verification: Some("v".into()),
            ac_type: AcType::Observation,
        };
        let value = serde_yaml::to_value(&ac).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> =
            AcceptanceCriterion::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(
            got, expected,
            "AcceptanceCriterion::FIELDS drifted from struct"
        );
    }

    #[test]
    fn scenario_fields_matches_struct() {
        let scenario = Scenario {
            name: "n".into(),
            given: "g".into(),
            when: "w".into(),
            then: "t".into(),
            gate: false,
        };
        let value = serde_yaml::to_value(&scenario).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = Scenario::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Scenario::FIELDS drifted from struct");
    }

    #[test]
    fn relation_fields_matches_struct() {
        // Construct a Relation with BOTH target fields populated so the
        // top-level-keys assertion compares apples-to-apples against
        // Relation::FIELDS (which lists every legal field, including the
        // mutually-exclusive `card`/`choice` pair per spec
        // 2026-05-25-relation-schema-choice-targets ac-01).
        let relation = Relation {
            card: Some("c".into()),
            choice: Some("0020".into()),
            kind: RelationKind::Feeds,
            reason: "r".into(),
        };
        let value = serde_yaml::to_value(&relation).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> = Relation::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Relation::FIELDS drifted from struct");
    }

    // -----------------------------------------------------------------------
    // spec 2026-05-25-relation-schema-choice-targets ac-05 — Relation parse +
    // validate coverage across both target shapes.
    // -----------------------------------------------------------------------

    #[test]
    fn relation_parse_card_target_succeeds() {
        let yaml = "card: 0019-tabletop\ntype: depends-on\nreason: r\n";
        let r: Relation = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(r.card.as_deref(), Some("0019-tabletop"));
        assert_eq!(r.choice, None);
        assert!(r.validate().is_ok());
    }

    #[test]
    fn relation_parse_choice_target_succeeds() {
        let yaml = "choice: '0020'\ntype: respects\nreason: r\n";
        let r: Relation = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(r.card, None);
        assert_eq!(r.choice.as_deref(), Some("0020"));
        assert!(r.validate().is_ok());
    }

    #[test]
    fn relation_validate_rejects_both_set() {
        let r = Relation {
            card: Some("c".into()),
            choice: Some("0020".into()),
            kind: RelationKind::Respects,
            reason: "r".into(),
        };
        let err = r.validate().expect_err("both-set must reject");
        assert!(err.to_string().contains("both"), "got: {err}");
    }

    #[test]
    fn relation_validate_rejects_neither_set() {
        let r = Relation {
            card: None,
            choice: None,
            kind: RelationKind::Feeds,
            reason: "r".into(),
        };
        let err = r.validate().expect_err("neither-set must reject");
        assert!(err.to_string().contains("neither"), "got: {err}");
    }

    #[test]
    fn relation_parse_unknown_field_still_rejected() {
        // deny_unknown_fields stays in force across the schema change —
        // adding `choice:` as a legal field doesn't open the door to arbitrary
        // sibling fields.
        let yaml = "card: c\ntype: feeds\nreason: r\nrogue: x\n";
        let result: Result<Relation, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err(), "deny_unknown_fields should reject `rogue:`");
    }

    #[test]
    fn relation_round_trip_omits_unset_target_field() {
        // Backward-compat guarantee per spec ac-05(f): card-target relations
        // emit YAML with NO `choice:` key, choice-target relations emit YAML
        // with NO `card:` key. Option::is_none + skip_serializing_if.
        let card_rel = Relation {
            card: Some("c".into()),
            choice: None,
            kind: RelationKind::Feeds,
            reason: "r".into(),
        };
        let yaml = serde_yaml::to_string(&card_rel).unwrap();
        assert!(!yaml.contains("choice:"), "card-target leaked choice key: {yaml}");

        let choice_rel = Relation {
            card: None,
            choice: Some("0020".into()),
            kind: RelationKind::Respects,
            reason: "r".into(),
        };
        let yaml = serde_yaml::to_string(&choice_rel).unwrap();
        assert!(!yaml.contains("card:"), "choice-target leaked card key: {yaml}");
    }

    #[test]
    fn card_validate_relations_propagates_first_error() {
        let card = Card {
            id: Some("0001-x".into()),
            feature: "f".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![Relation {
                card: None,
                choice: None,
                kind: RelationKind::Feeds,
                reason: "r".into(),
            }],
            references: vec![],
            notes: vec![],
        };
        let err = card.validate_relations().expect_err("invalid relation must propagate");
        assert!(err.to_string().contains("neither"), "got: {err}");
    }

    #[test]
    fn spec_round_trip_is_lossless() {
        let spec = Spec {
            id: "0001".into(),
            goal: "do the thing".into(),
            cards: vec!["0020-orbit-state".into()],
            status: SpecStatus::Open,
            labels: vec!["spec".into()],
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "ac-01".into(),
                description: "first".into(),
                gate: true,
                checked: false,
                verification: Some("v1".into()),
                ac_type: AcType::Code,
            }],
            memories_considered: vec![],
            closed_at: None,
        };
        let yaml = serde_yaml::to_string(&spec).unwrap();
        let parsed: Spec = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(spec, parsed);
    }

    #[test]
    fn acceptance_criterion_round_trips_ac_type_observation() {
        // spec 2026-05-16-ac-taxonomy ac-01 verification: round-trip an AC
        // with ac_type: observation, serialise to canonical YAML,
        // deserialise, assert byte-identical equality on the struct.
        let ac = AcceptanceCriterion {
            id: "ac-18".into(),
            description: "Post-cutover monitoring — 7-day live behaviour window".into(),
            gate: false,
            checked: false,
            verification: Some("operator dashboard review for 7 calendar days".into()),
            ac_type: AcType::Observation,
        };
        let yaml = serde_yaml::to_string(&ac).unwrap();
        let parsed: AcceptanceCriterion = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(ac, parsed);
        // Sanity: the field surfaces in the serialised YAML.
        assert!(yaml.contains("ac_type: observation"), "ac_type must serialise: {yaml}");
    }

    #[test]
    fn acceptance_criterion_parses_legacy_yaml_without_ac_type() {
        // spec 2026-05-16-ac-taxonomy ac-01 verification: existing spec.yaml
        // content without the ac_type field parses cleanly and deserialises
        // with ac_type: Code (the default).
        let legacy_yaml = "id: ac-01\ndescription: legacy\ngate: false\nchecked: false\n";
        let parsed: AcceptanceCriterion = serde_yaml::from_str(legacy_yaml).unwrap();
        assert_eq!(parsed.ac_type, AcType::Code);
    }

    #[test]
    fn ac_type_round_trips_snake_case_for_every_variant() {
        // spec 2026-05-16-ac-taxonomy ac-01 verification: every AcType
        // variant serialises to its snake_case form and back.
        for (variant, expected) in [
            (AcType::Code, "code"),
            (AcType::Config, "config"),
            (AcType::Doc, "doc"),
            (AcType::Ops, "ops"),
            (AcType::Observation, "observation"),
        ] {
            let s = serde_yaml::to_string(&variant).unwrap();
            assert_eq!(
                s.trim(),
                expected,
                "variant {variant:?} did not serialise to expected snake_case",
            );
            let back: AcType = serde_yaml::from_str(expected).unwrap();
            assert_eq!(back, variant);
        }
    }

    #[test]
    fn ac_type_blocks_close_two_band_split() {
        // spec 2026-05-16-ac-taxonomy ac-01 verification: blocks_close()
        // returns true for Code/Config/Doc and false for Ops/Observation.
        assert!(AcType::Code.blocks_close());
        assert!(AcType::Config.blocks_close());
        assert!(AcType::Doc.blocks_close());
        assert!(!AcType::Ops.blocks_close());
        assert!(!AcType::Observation.blocks_close());
    }

    #[test]
    fn ac_type_default_is_code() {
        // spec 2026-05-16-ac-taxonomy ac-01 verification: the Default
        // impl returns Code (matches the implicit assumption every
        // untyped AC carried before this field shipped).
        assert_eq!(AcType::default(), AcType::Code);
        assert!(AcType::default().is_code());
    }

    // ----- Config schema-drift coverage (spec 2026-05-18-documentation-topology) -----

    #[test]
    fn config_fields_matches_struct() {
        // ac-02 verification: Config::FIELDS must equal the struct's
        // serde top-level field set. Fully-populated fixture so
        // skip_serializing_if doesn't drop fields.
        let config = Config {
            docs: Some(DocsConfig {
                topology: Some("docs/topology.md".into()),
            }),
            plugin_version: Some("0.4.20".into()),
            plugin_repo: Some(true),
        };
        let value = serde_yaml::to_value(&config).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> =
            Config::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "Config::FIELDS drifted from struct");
    }

    #[test]
    fn docs_config_fields_matches_struct() {
        // ac-03 verification: DocsConfig::FIELDS must equal its serde
        // top-level field set.
        let docs = DocsConfig {
            topology: Some("docs/topology.md".into()),
        };
        let value = serde_yaml::to_value(&docs).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> =
            DocsConfig::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "DocsConfig::FIELDS drifted from struct");
    }

    #[test]
    fn config_rejects_unknown_field() {
        // ac-02 verification: parser MUST reject unknown fields rather than
        // silently dropping them (matches the deny_unknown_fields contract
        // shared with Spec/Card/Choice/Memory/Session).
        let yaml = r#"
docs:
  topology: docs/topology.md
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<Config>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn docs_config_rejects_unknown_field() {
        // ac-03 verification: inner DocsConfig also rejects unknown fields.
        let yaml = r#"
topology: docs/topology.md
unknown_inner: nope
"#;
        let err = serde_yaml::from_str::<DocsConfig>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn config_empty_is_valid() {
        // ac-03 verification: a fixture without docs.topology parses with
        // Config { docs: None } (opt-in tolerance).
        let yaml = "{}";
        let parsed: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(parsed.docs.is_none());
    }

    #[test]
    fn deprecated_docs_topology_field_round_trips_intact() {
        // Spec 2026-05-18-topology-substrate-migration ac-04 — Option A:
        // DocsConfig::topology is retained as a parse-only deprecated
        // field so brownfield consumer repos (those that wired topology
        // under 0.4.19 via spec 2026-05-18-topology-substrate-wires
        // ac-01) do not hard-fail Config::from_str on session-prime. The
        // canonical writer preserves the field so verify_all sees no
        // round-trip drift. A follow-on spec deletes the field after a
        // consumer-repo soak window.
        let yaml = "docs:\n  topology: docs/topology.md\n";
        let parsed: Config = serde_yaml::from_str(yaml).unwrap();
        // Confirm the field round-trips through the canonical writer.
        let reserialised = serde_yaml::to_string(&parsed).unwrap();
        let reparsed: Config = serde_yaml::from_str(&reserialised).unwrap();
        assert_eq!(parsed, reparsed);
        // Confirm the field value is preserved on write.
        assert!(
            reserialised.contains("topology: docs/topology.md"),
            "deprecated docs.topology field must be preserved on canonical write: {reserialised}",
        );
    }

    #[test]
    fn config_round_trips_byte_identical() {
        // ac-03 verification: a populated Config round-trips through
        // serde_yaml without loss.
        let config = Config {
            docs: Some(DocsConfig {
                topology: Some("docs/topology.md".into()),
            }),
            plugin_version: Some("0.4.20".into()),
            plugin_repo: Some(false),
        };
        let yaml = serde_yaml::to_string(&config).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(config, parsed);
    }

    // ========================================================================
    // TopologyEntry — per choice 0025, spec
    // 2026-05-18-topology-substrate-migration ac-01
    // ========================================================================

    #[test]
    fn topology_entry_rejects_unknown_field() {
        // ac-01 verification: deny-unknown-fields contract holds.
        let yaml = r#"
subsystem: cards
canonical_code: [orbit-state/crates/core/src/schema.rs]
unknown_field: oops
"#;
        let err = serde_yaml::from_str::<TopologyEntry>(yaml).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn topology_entry_missing_required_subsystem() {
        // ac-01 verification: serde-required field `subsystem` cannot be omitted.
        let yaml = r#"
canonical_code: [orbit-state/crates/core/src/schema.rs]
"#;
        let err = serde_yaml::from_str::<TopologyEntry>(yaml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("subsystem") || msg.contains("missing"),
            "expected a missing-required-field error, got: {msg}"
        );
    }

    #[test]
    fn topology_entry_missing_required_canonical_code() {
        // ac-01 verification: serde-required field `canonical_code` cannot be
        // omitted (the empty-vec case is enforced by validate(), see below).
        let yaml = r#"
subsystem: cards
"#;
        let err = serde_yaml::from_str::<TopologyEntry>(yaml).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("canonical_code") || msg.contains("missing"),
            "expected a missing-required-field error, got: {msg}"
        );
    }

    #[test]
    fn topology_entry_round_trips_byte_identical() {
        // ac-01 verification: serde round-trip is lossless for a fully-populated
        // entry. Every Vec non-empty so skip_serializing_if doesn't drop fields.
        let entry = TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0016".into()],
            operational_doc: vec!["plugins/orb/skills/card/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        };
        let yaml = serde_yaml::to_string(&entry).unwrap();
        let parsed: TopologyEntry = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(entry, parsed);
    }

    #[test]
    fn topology_entry_optional_lists_omitted_when_empty() {
        // ac-01 verification: skip_serializing_if = Vec::is_empty drops the
        // optional list fields from the canonical output when empty.
        let entry = TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        let yaml = serde_yaml::to_string(&entry).unwrap();
        assert!(!yaml.contains("decision_record"));
        assert!(!yaml.contains("operational_doc"));
        assert!(!yaml.contains("test_surface"));
        // Round-trip recovers an entry with empty vecs from a yaml that omits
        // the keys (serde defaults).
        let parsed: TopologyEntry = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.decision_record, Vec::<String>::new());
        assert_eq!(parsed.operational_doc, Vec::<String>::new());
        assert_eq!(parsed.test_surface, Vec::<String>::new());
    }

    #[test]
    fn topology_entry_fields_matches_struct() {
        // ac-01 verification: TopologyEntry::FIELDS equals the serde top-level
        // key set. Drift-trap parallel to the existing card_fields_matches_struct
        // / docs_config_fields_matches_struct pattern.
        let entry = TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec!["a".into()],
            decision_record: vec!["b".into()],
            operational_doc: vec!["c".into()],
            test_surface: vec!["d".into()],
        };
        let yaml = serde_yaml::to_string(&entry).unwrap();
        let value: serde_yaml::Value = serde_yaml::from_str(&yaml).unwrap();
        let got = top_level_keys(&value);
        let mut expected: Vec<String> =
            TopologyEntry::FIELDS.iter().map(|s| s.to_string()).collect();
        expected.sort();
        assert_eq!(got, expected, "TopologyEntry::FIELDS drifted from struct");
    }

    #[test]
    fn topology_entry_validate_accepts_well_formed() {
        let entry = TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        assert!(entry.validate().is_ok());
    }

    #[test]
    fn topology_entry_validate_rejects_short_subsystem() {
        // ac-01 verification: subsystem slug shorter than MIN_SUBSYSTEM_LEN
        // (5 chars) is rejected.
        let entry = TopologyEntry {
            subsystem: "card".into(), // 4 chars
            canonical_code: vec!["x".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        let err = entry.validate().unwrap_err();
        assert!(err.contains("minimum length"));
        assert!(err.contains("card"));
    }

    #[test]
    fn topology_entry_validate_rejects_non_slug_subsystem() {
        // ac-01 verification: subsystem with upper-case / underscores / leading
        // digit / leading hyphen / trailing hyphen / double-hyphen fails.
        for bad in &[
            "Cards",     // upper-case
            "card_db",   // underscore
            "1cards",    // leading digit
            "-cards",    // leading hyphen
            "cards-",    // trailing hyphen
            "card--db",  // double hyphen
            "cards/sub", // slash
            "cards.db",  // dot
        ] {
            let entry = TopologyEntry {
                subsystem: (*bad).into(),
                canonical_code: vec!["x".into()],
                decision_record: vec![],
                operational_doc: vec![],
                test_surface: vec![],
            };
            let err = entry.validate().unwrap_err();
            assert!(
                err.contains("slug-shaped") || err.contains("minimum length"),
                "expected slug-shape rejection for `{bad}`, got: {err}"
            );
        }
    }

    #[test]
    fn topology_entry_validate_rejects_empty_canonical_code() {
        let entry = TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec![],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        let err = entry.validate().unwrap_err();
        assert!(err.contains("canonical_code"));
    }

    #[test]
    fn topology_entry_validate_accepts_known_orbit_substrate_slugs() {
        // ac-01 verification: the self-describing seed shipped by ac-05 uses
        // these slugs; verify validate() accepts each.
        for slug in &["cards", "choices", "specs", "memories", "topology"] {
            let entry = TopologyEntry {
                subsystem: (*slug).into(),
                canonical_code: vec!["x".into()],
                decision_record: vec![],
                operational_doc: vec![],
                test_surface: vec![],
            };
            assert!(
                entry.validate().is_ok(),
                "expected `{slug}` to validate as a topology slug"
            );
        }
    }

    #[test]
    fn config_topology_wrong_type_rejected() {
        // ac-03 verification: docs.topology of the wrong type (list,
        // not string) fails to parse.
        let yaml = r#"
docs:
  topology: [not, a, string]
"#;
        let err = serde_yaml::from_str::<Config>(yaml).unwrap_err();
        // serde_yaml error wording varies — just assert it fails.
        let msg = err.to_string();
        assert!(
            msg.contains("string") || msg.contains("sequence") || msg.contains("expected"),
            "expected a type-mismatch error, got: {msg}"
        );
    }

    // ========================================================================
    // memory cite_evidence shape tests — spec 2026-05-27-memory-cite-reading
    // ac-03: MemoryReconciliation gains an optional `cite_evidence` field.
    // ========================================================================

    /// AC-03: a spec.yaml carrying `cite_evidence` on a memories_considered
    /// entry parses cleanly and round-trips byte-identically. The shape:
    /// `[{ cite_path, excerpt, read_at }]`.
    #[test]
    fn cite_spec_with_cite_evidence_parses_and_round_trips() {
        let yaml = r#"id: 2026-05-27-test
goal: g
cards: []
status: open
memories_considered:
- key: m
  disposition: adopted
  reason: r
  cite_evidence:
  - cite_path: docs/a.md
    excerpt: a load-bearing line
    read_at: 2026-05-27T00:00:00Z
"#;
        let spec: Spec = crate::canonical::parse_yaml(yaml).unwrap();
        assert_eq!(spec.memories_considered.len(), 1);
        let entry = &spec.memories_considered[0];
        assert_eq!(entry.cite_evidence.len(), 1);
        assert_eq!(entry.cite_evidence[0].cite_path, "docs/a.md");
        assert_eq!(entry.cite_evidence[0].excerpt, "a load-bearing line");
        assert_eq!(entry.cite_evidence[0].read_at, "2026-05-27T00:00:00Z");
        let reserialised = crate::canonical::serialise_yaml(&spec).unwrap();
        let reparsed: Spec = crate::canonical::parse_yaml(&reserialised).unwrap();
        assert_eq!(spec, reparsed);
    }

    /// AC-03: missing `read_at` on a cite_evidence entry fails parse.
    #[test]
    fn cite_evidence_missing_read_at_rejected() {
        let yaml = r#"id: x
goal: g
cards: []
status: open
memories_considered:
- key: m
  disposition: adopted
  reason: r
  cite_evidence:
  - cite_path: docs/a.md
    excerpt: ex
"#;
        let err = crate::canonical::parse_yaml::<Spec>(yaml).unwrap_err();
        assert!(
            err.message.contains("read_at") || err.message.contains("missing"),
            "expected missing-field error on read_at, got: {err}",
        );
    }

    /// AC-03: non-string excerpt on a cite_evidence entry fails parse with
    /// a type-mismatch error.
    #[test]
    fn cite_evidence_non_string_excerpt_rejected() {
        let yaml = r#"id: x
goal: g
cards: []
status: open
memories_considered:
- key: m
  disposition: adopted
  reason: r
  cite_evidence:
  - cite_path: docs/a.md
    excerpt: [not, a, string]
    read_at: 2026-05-27T00:00:00Z
"#;
        let err = crate::canonical::parse_yaml::<Spec>(yaml).unwrap_err();
        let msg = err.message.clone();
        assert!(
            msg.contains("string") || msg.contains("sequence") || msg.contains("expected"),
            "expected type-mismatch on excerpt, got: {msg}",
        );
    }

    /// AC-03: a spec.yaml without `cite_evidence` (the dominant case for
    /// memories without cites) parses unchanged and the field defaults to
    /// an empty vec. Backward-compat: existing specs in the corpus that
    /// have memories_considered entries without cite_evidence load cleanly.
    #[test]
    fn cite_legacy_memories_considered_entry_loads_with_empty_cite_evidence() {
        let yaml = r#"id: legacy
goal: g
cards: []
status: open
memories_considered:
- key: m
  disposition: adopted
  reason: r
"#;
        let spec: Spec = crate::canonical::parse_yaml(yaml).unwrap();
        assert_eq!(spec.memories_considered.len(), 1);
        assert!(spec.memories_considered[0].cite_evidence.is_empty());
        // Reserialise: cite_evidence must not appear on disk (empty Vec
        // skipped via skip_serializing_if).
        let reserialised = crate::canonical::serialise_yaml(&spec).unwrap();
        assert!(
            !reserialised.contains("cite_evidence"),
            "empty cite_evidence must not materialise on disk: {reserialised}",
        );
    }

    /// AC-03 / unknown field guard: an extra key on a cite_evidence entry
    /// fails parse via `deny_unknown_fields`. Closes the lossy-parse
    /// failure mode for the new shape.
    #[test]
    fn cite_evidence_rejects_unknown_field() {
        let yaml = r#"id: x
goal: g
cards: []
status: open
memories_considered:
- key: m
  disposition: adopted
  reason: r
  cite_evidence:
  - cite_path: docs/a.md
    excerpt: ex
    read_at: 2026-05-27T00:00:00Z
    bogus: field
"#;
        let err = crate::canonical::parse_yaml::<Spec>(yaml).unwrap_err();
        assert!(
            err.message.contains("unknown"),
            "expected unknown-field rejection, got: {err}",
        );
    }
}
