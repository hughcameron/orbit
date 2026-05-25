//! Verb dispatch surface — single entry point shared by CLI and MCP.
//!
//! Per ac-05: "MCP server and CLI both call same Rust core — state-mutation
//! parity (canonical files + state.db byte-identical), error format
//! `<verb>: <category>: <sentence>`."
//!
//! This module defines:
//! - [`VerbRequest`]   — typed input taxonomy (one variant per verb).
//! - [`VerbResponse`]  — typed output taxonomy (one variant per verb).
//! - [`execute`]       — the single dispatch fn both surfaces call.
//! - [`envelope_ok`] / [`envelope_err`] — wire envelope helpers.
//!
//! Adding a verb is a closed-form change: extend the two enums with matching
//! variants and add a private impl fn dispatched from [`execute`]. Both
//! surfaces (CLI argv parser, MCP JSON-RPC handler) construct `VerbRequest`
//! independently, then call [`execute`] — that's where the parity contract
//! lives. The wire envelope is shared so byte-equal payloads fall out for
//! free as long as both surfaces serialise the same `VerbResponse` with the
//! same helper.
//!
//! v0.1 surface: `spec.list` only. Subsequent ACs (ac-06..11) add the rest.

use crate::atomic::{append_jsonl_line, write_atomic};
use crate::canonical::{parse_json_line, parse_yaml, serialise_json_line, serialise_yaml};
use crate::error::{Error, Result};
use crate::layout::OrbitLayout;
use crate::locks;
use crate::schema::{
    AcType, AcceptanceCriterion, Card, Choice, InvocationOutcome, Memory, NoteEvent, Session,
    SkillInvocation, Spec, SpecStatus, TaskEvent, TaskEventKind,
};
use crate::session::{read_session_card, read_session_id};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

// ============================================================================
// Decision log — captured here so it travels with the code.
// ============================================================================
//
// Wire shape vs on-disk shape: the response wraps `schema::Spec` directly
// for now (`SpecShowResult { spec: Spec }`). On-disk and wire are isomorphic
// at v0.1. If they diverge later (e.g. wire wants resolved derived fields
// like aggregated note count), the wrapper struct gives us the seam to
// project without breaking the wire contract.

// ============================================================================
// Request / Response taxonomy
// ============================================================================

/// Typed verb request. Tagged on the wire as `{"verb": "<name>", "args": {...}}`
/// so the MCP `tools/call` translation is trivial:
///
/// ```text
/// MCP {name: "spec.list", arguments: {...}} → {"verb": "spec.list", "args": {...}} → VerbRequest
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "verb", content = "args")]
pub enum VerbRequest {
    #[serde(rename = "spec.list")]
    SpecList(SpecListArgs),
    #[serde(rename = "spec.show")]
    SpecShow(SpecShowArgs),
    #[serde(rename = "spec.resolve")]
    SpecResolve(SpecResolveArgs),
    #[serde(rename = "spec.note")]
    SpecNote(SpecNoteArgs),
    #[serde(rename = "spec.create")]
    SpecCreate(SpecCreateArgs),
    #[serde(rename = "spec.update")]
    SpecUpdate(SpecUpdateArgs),
    #[serde(rename = "spec.close")]
    SpecClose(SpecCloseArgs),
    #[serde(rename = "spec.acs")]
    SpecAcs(SpecAcsArgs),
    #[serde(rename = "spec.next-ac")]
    SpecNextAc(SpecNextAcArgs),
    #[serde(rename = "spec.blocking-gate")]
    SpecBlockingGate(SpecBlockingGateArgs),
    #[serde(rename = "spec.has-unchecked")]
    SpecHasUnchecked(SpecHasUncheckedArgs),
    #[serde(rename = "spec.check")]
    SpecCheck(SpecCheckArgs),
    #[serde(rename = "spec.uncheck")]
    SpecUncheck(SpecUncheckArgs),
    #[serde(rename = "spec.promote")]
    SpecPromote(SpecPromoteArgs),
    #[serde(rename = "task.open")]
    TaskOpen(TaskOpenArgs),
    #[serde(rename = "task.list")]
    TaskList(TaskListArgs),
    #[serde(rename = "task.show")]
    TaskShow(TaskShowArgs),
    #[serde(rename = "task.ready")]
    TaskReady(TaskReadyArgs),
    #[serde(rename = "task.claim")]
    TaskClaim(TaskClaimArgs),
    #[serde(rename = "task.update")]
    TaskUpdate(TaskUpdateArgs),
    #[serde(rename = "task.done")]
    TaskDone(TaskDoneArgs),
    #[serde(rename = "memory.remember")]
    MemoryRemember(MemoryRememberArgs),
    #[serde(rename = "memory.list")]
    MemoryList(MemoryListArgs),
    #[serde(rename = "memory.search")]
    MemorySearch(MemorySearchArgs),
    #[serde(rename = "memory.match")]
    MemoryMatch(MemoryMatchArgs),
    #[serde(rename = "card.show")]
    CardShow(CardShowArgs),
    #[serde(rename = "card.list")]
    CardList(CardListArgs),
    #[serde(rename = "card.search")]
    CardSearch(CardSearchArgs),
    #[serde(rename = "card.tree")]
    CardTree(CardTreeArgs),
    #[serde(rename = "card.specs")]
    CardSpecs(CardSpecsArgs),
    #[serde(rename = "overview")]
    Overview(OverviewArgs),
    #[serde(rename = "graph")]
    Graph(GraphArgs),
    #[serde(rename = "audit.drift")]
    AuditDrift(AuditDriftArgs),
    #[serde(rename = "audit.topology")]
    AuditTopology(AuditTopologyArgs),
    #[serde(rename = "audit.conformance")]
    AuditConformance(AuditConformanceArgs),
    #[serde(rename = "topology.setup")]
    TopologySetup(TopologySetupArgs),
    #[serde(rename = "substrate.classify")]
    SubstrateClassify(SubstrateClassifyArgs),
    #[serde(rename = "choice.show")]
    ChoiceShow(ChoiceShowArgs),
    #[serde(rename = "choice.list")]
    ChoiceList(ChoiceListArgs),
    #[serde(rename = "choice.search")]
    ChoiceSearch(ChoiceSearchArgs),
    #[serde(rename = "session.prime")]
    SessionPrime(SessionPrimeArgs),
    #[serde(rename = "session.start")]
    SessionStart(SessionStartArgs),
    #[serde(rename = "session.distill")]
    SessionDistill(SessionDistillArgs),
    #[serde(rename = "session.set-card")]
    SessionSetCard(SessionSetCardArgs),
    #[serde(rename = "session.handover")]
    SessionHandover(SessionHandoverArgs),
    #[serde(rename = "skill.record-invocation")]
    SkillRecordInvocation(SkillRecordInvocationArgs),
    #[serde(rename = "skill.recurrence")]
    SkillRecurrence(SkillRecurrenceArgs),
    #[serde(rename = "routine.chains")]
    RoutineChains(RoutineChainsArgs),
    #[serde(rename = "routine.detect")]
    RoutineDetect(RoutineDetectArgs),
    #[serde(rename = "routine.author")]
    RoutineAuthor(RoutineAuthorArgs),
    #[serde(rename = "routine.verify")]
    RoutineVerify(RoutineVerifyArgs),
}

/// Args for `spec.list`. Optional `status` filter; further filters land later.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecListArgs {
    /// Restrict to specs in this status. Must be `"open"` or `"closed"` if
    /// provided. Empty string and other values are rejected as malformed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// Args for `spec.show` — locate the spec by id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecShowArgs {
    pub id: String,
}

/// Args for `spec.resolve` — return the spec id a skill should act on when
/// no explicit id was passed at invocation. Per spec
/// 2026-05-19-skills-infer-or-prompt-before-halt and the rally
/// "agent-side-substrate-engagement" decision pack: the resolver implements
/// the three-step recovery (infer → prompt → halt) once in the substrate so
/// every spec-id consumer skill (`/orb:implement`, `/orb:review-pr`,
/// `/orb:review-spec`, `/orb:audit`, `/orb:drive`) emits the same behaviour.
///
/// Resolution order (per decision D2):
///
/// 1. **Infer from the bound card.** If `.orbit/.session-card` is bound to
///    a card slug and that card has exactly one open spec, return
///    `Resolved { id }`. (The optional `card` arg overrides
///    `.session-card` — a skill can scope the resolution to a specific
///    card without flipping the session binding.)
/// 2. **Prompt with a menu.** If no card is bound, or the bound card has
///    multiple open specs, return `Prompt { candidates }` where each
///    candidate carries the spec id and a one-line goal label
///    (`goal_first_line`) so the skill can present a self-describing
///    AskUserQuestion.
/// 3. **Halt.** If both fallbacks fail (no bound card AND no open specs,
///    or the bound card exists but has no open specs), return
///    `Error::unavailable` carrying one of the two canonical halt-message
///    templates from decision D5.
///
/// Skills never expand the prose; they call this verb and either use the
/// resolved id, present the prompt menu, or surface the `unavailable`
/// error message verbatim.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecResolveArgs {
    /// Calling skill name, surfaced in halt-message templates so the
    /// user sees which skill emitted the halt (e.g. `implement`,
    /// `review-pr`). Optional; omitted when an inline caller doesn't
    /// know its own name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill: Option<String>,
    /// Override the card binding. When set, the resolver scopes
    /// inference to this card instead of reading `.session-card`. Slug
    /// shape; resolved via the same `resolve_numeric_slug` path as
    /// `card.show`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<String>,
}

/// Args for `spec.create` — write a new spec file.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecCreateArgs {
    pub id: String,
    pub goal: String,
    /// Cards this spec advances. Empty list is legal but unusual.
    #[serde(default)]
    pub cards: Vec<String>,
    /// Free-text labels (e.g. `spec`, `experimental`).
    #[serde(default)]
    pub labels: Vec<String>,
    /// Initial acceptance criteria — usually empty at creation; populated
    /// via spec.update once the spec is designed.
    #[serde(default)]
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
}

/// Args for `spec.update` — modify fields on an existing spec. Only the
/// fields included in the args are applied; omitted fields keep prior
/// values. Status changes go through `spec.close` (which has transactional
/// card-linkage logic), not here.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecUpdateArgs {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cards: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub labels: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<Vec<AcceptanceCriterion>>,
}

/// Args for `spec.close` — transition status to `closed` and append the
/// spec's path to every linked card's `specs` array atomically.
///
/// `force` bypasses the unchecked-AC pre-flight added by spec
/// 2026-05-13-spec-close-ac-preflight (ac-02 / ac-03). It does not bypass
/// the unfinished-tasks guard or the already-closed guard.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecCloseArgs {
    pub id: String,
    /// When true, close even if non-time-gated ACs remain unchecked.
    /// The bypassed AC ids surface in `SpecCloseResult.forced_unchecked`
    /// so the audit trail is preserved in the structured response.
    #[serde(default)]
    pub force: bool,
}

// ----------------------------------------------------------------------------
// Spec acceptance-criterion verbs (per spec 2026-05-24-port-acceptance-shim).
// Port of plugins/orb/scripts/orbit-acceptance.sh into native verbs on the
// `spec` family — `spec.acs`, `spec.next-ac`, `spec.blocking-gate`,
// `spec.has-unchecked` are read-only; `spec.check` / `spec.uncheck` mutate
// an AC's `checked` flag with idempotency.
// ----------------------------------------------------------------------------

/// Args for `spec.acs` — return the full acceptance_criteria list for a spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecAcsArgs {
    pub id: String,
}

/// Args for `spec.next-ac` — return the first unchecked AC that is not
/// blocked by an unchecked gate. Gate-axis traversal (per ac-06 of spec
/// 2026-05-24-port-acceptance-shim).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecNextAcArgs {
    pub id: String,
}

/// Args for `spec.blocking-gate` — return the first unchecked gate AC, if any.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecBlockingGateArgs {
    pub id: String,
}

/// Args for `spec.has-unchecked` — true if any AC is unchecked. Raw-axis
/// (`!checked`) traversal, distinct from `spec.close`'s taxonomy-axis
/// pre-flight per ac-06 of spec 2026-05-24-port-acceptance-shim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecHasUncheckedArgs {
    pub id: String,
}

/// Args for `spec.check` — flip an AC's `checked` flag from false to true.
/// Errors on missing AC (`Error::not_found`) or already-checked AC
/// (`Error::conflict`). The CLI `--ac-check` flag is preserved as sugar.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecCheckArgs {
    pub id: String,
    pub ac_id: String,
}

/// Args for `spec.uncheck` — flip an AC's `checked` flag from true to false.
/// Symmetric to `spec.check`; same error contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecUncheckArgs {
    pub id: String,
    pub ac_id: String,
}

/// Args for `spec.promote` — turn a card into a spec. Per spec
/// 2026-05-25-port-promote-sh (second opportunistic migration under
/// choice 0020). Reads the card at `card_path`, derives the spec id as
/// `<today-iso>-<slug-without-NNNN>`, creates the spec with the card''s
/// `goal` and `cards: [<card.id>]`, and populates `acceptance_criteria`
/// one entry per `scenario` (preserving `gate: bool`, seeding
/// `checked: false`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecPromoteArgs {
    /// Path to the card file (absolute or relative to the layout root).
    pub card_path: String,
    /// When true, compute the planned spec but write nothing. Round-trips
    /// the same envelope shape as the non-dry-run path for parity tests.
    #[serde(default)]
    pub dry_run: bool,
    /// Override the date used in the derived spec id (`YYYY-MM-DD`).
    /// Production callers omit this — the substrate reads `now_utc()`.
    /// Primarily for parity tests so the expected envelope is byte-
    /// deterministic regardless of when the test runs. Mirrors
    /// `SpecNoteArgs.timestamp`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub today: Option<String>,
}

// ----------------------------------------------------------------------------
// Task verb args (ac-07)
// ----------------------------------------------------------------------------

/// Args for `task.open` — append an Open event creating a new task under
/// `<spec_id>.tasks.jsonl`. Substrate generates `task_id` if not supplied;
/// callers supply one for migrations or tests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskOpenArgs {
    pub spec_id: String,
    pub body: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Args for `task.list` — list tasks (current state per task_id) for one
/// spec, or all specs if `spec_id` is None.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskListArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
    /// Filter by current state (`open`, `claim`, `update`, `done`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
}

/// Args for `task.show` — show one task with its full event history.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskShowArgs {
    pub spec_id: String,
    pub task_id: String,
}

/// Args for `task.ready` — list tasks whose last event is Open (claimable).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskReadyArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spec_id: Option<String>,
}

/// Args for `task.claim` — append a Claim event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskClaimArgs {
    pub spec_id: String,
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Args for `task.update` — append an Update event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskUpdateArgs {
    pub spec_id: String,
    pub task_id: String,
    pub body: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Args for `task.done` — append a Done event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskDoneArgs {
    pub spec_id: String,
    pub task_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

// ----------------------------------------------------------------------------
// Memory / card / choice verb args (ac-08, ac-09, ac-10)
// ----------------------------------------------------------------------------

/// Args for `memory.remember` — upsert a memory entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MemoryRememberArgs {
    pub key: String,
    pub body: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Suppress the topology-label nudge even when the labels list
    /// includes `topology`. Per spec 2026-05-18-topology-substrate-wires
    /// ac-04. Defaults to false; mirrors the `--no-edit` / `--no-verify`
    /// naming convention.
    #[serde(default)]
    pub no_nudge: bool,
    /// Suppress the state-shape warning emitted when the body's first
    /// sentence reads as a state observation rather than a mechanism
    /// clause. Per spec 2026-05-19-memory-gates-decisions ac-05 (D5b).
    /// Defaults to false; mirrors `--no-nudge`.
    #[serde(default)]
    pub no_warn: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MemoryListArgs {}

/// Args for `memory.search` — substring (case-insensitive) over body + labels.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MemorySearchArgs {
    pub query: String,
}

/// Args for `memory.match` — surface memories relevant to a decision moment.
/// Ranked output, distinct semantic from operator-keyword `memory.search`.
/// Per spec 2026-05-19-memory-gates-decisions ac-01/ac-02 (D1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MemoryMatchArgs {
    /// Free text describing the decision context — typically a card slug,
    /// a spec goal, or a short snippet of the proposed approach.
    pub topic: String,
    /// Optional label-overlap hint — typically the card slugs the decision
    /// belongs to or skill/topic labels. Weighted higher than body overlap
    /// in the ranker.
    #[serde(default)]
    pub labels: Vec<String>,
    /// Cap on returned matches. Defaults to 10.
    #[serde(default = "default_match_limit")]
    pub limit: usize,
}

fn default_match_limit() -> usize {
    10
}

/// Args for `card.show`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CardShowArgs {
    pub slug: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CardListArgs {
    /// Filter by maturity (`planned`, `emerging`, `established`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maturity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CardSearchArgs {
    pub query: String,
}

/// Args for `card.tree` — render the local subgraph from a card.
///
/// `depth` defaults to 2 (one hop in each direction expanded) and may be 0
/// (returns just the root with no edges). The graph is cycle-safe: a slug
/// already seen on the current expansion path is rendered as a truncated
/// node so the structure doesn't recurse indefinitely.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CardTreeArgs {
    pub slug: String,
    #[serde(default = "default_card_tree_depth")]
    pub depth: u32,
}

fn default_card_tree_depth() -> u32 {
    2
}

/// Args for `card.specs` — list specs that advance a card, with bidirectional
/// link health.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CardSpecsArgs {
    pub slug: String,
}

/// Args for `overview` — single-screen project synthesis.
///
/// All output is bounded. The optional `memory_cap` mirrors `session.prime`
/// (default K=10) and applies uniformly to memories, the recent-open-spec
/// list, and the orphan list so the verb stays single-screen as the project
/// ages.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OverviewArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_cap: Option<usize>,
}

/// Args for `graph` — render the cards/specs graph to mermaid or graphviz.
///
/// The unscoped default render is intentionally permitted to exceed
/// single-screen — it serves the share-or-paste use case, not the synthesis
/// use case (the bounded contract applies to `overview`, not here).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GraphArgs {
    /// Scope the render to one card and its neighbourhood. When set, the
    /// graph is the union of nodes within `depth` hops of this card.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card: Option<String>,
    /// Depth in hops from `card`. Default 2; only meaningful with `card`.
    #[serde(default = "default_graph_depth")]
    pub depth: u32,
    /// Output format. Default mermaid.
    #[serde(default)]
    pub format: GraphFormat,
}

fn default_graph_depth() -> u32 {
    2
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GraphFormat {
    #[default]
    Mermaid,
    Graphviz,
}

/// Args for `audit.drift` — permissive YAML scan that surfaces top-level
/// fields absent from the canonical schema. No flags at v0.1; the verb
/// walks the full substrate.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditDriftArgs {}

/// Args for `audit.topology` — walks `.orbit/topology/<subsystem>.yaml`
/// per choice 0025 (`topology-substrate-folder`) and reports drift
/// (stale_pointer, missing_entry, invalid_field, parse_failed). No
/// flags at v0.1. Per spec 2026-05-18-documentation-topology ac-06 and
/// 2026-05-18-topology-substrate-migration ac-02.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditTopologyArgs {}

/// Args for `audit.conformance` — aggregate workflow-conformance audit
/// per spec 2026-05-19-workflow-conformance ac-01. No public fields in
/// v1; test injection of a controllable `today` lands on the sibling
/// private helper `audit_conformance_at(layout, today)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditConformanceArgs {}

/// Args for `topology.setup` — scaffolds the `.orbit/topology/`
/// substrate folder and writes the self-describing seed entries, with
/// opportunistic brownfield cleanup of any legacy `docs.topology`
/// config key. Per spec 2026-05-18-topology-substrate-migration ac-05
/// and choice 0020 (Rust verb migration of setup-topology.sh).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TopologySetupArgs {
    /// Script the wire-or-decline prompt for non-interactive runs.
    /// Some("y") proceeds, Some("n") declines, None defers to caller-
    /// driven interaction. CLI surfaces this as `--answer-wire`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_wire: Option<String>,
}

/// Args for `substrate.classify` — pure-read classifier that inspects
/// the working tree at `layout.repo_root()` and returns one of the six
/// [`SubstrateLayoutState`] variants. No public fields in v1. Per spec
/// 2026-05-24-setup-is-orbit-state-aware ac-11.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubstrateClassifyArgs {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChoiceShowArgs {
    pub id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChoiceListArgs {
    /// Filter by status (`proposed`, `accepted`, `rejected`, `deprecated`, `superseded`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ChoiceSearchArgs {
    pub query: String,
}

/// Args for `session.prime` — agent session priming context.
///
/// Per ac-11: bounded output formula `f(N specs, M memories) ≤ 40 +
/// 2*open_specs + min(M,10)`. The K=10 memory cap is enforced here;
/// the per-open-spec bound is structural (each spec contributes one
/// summary to the output, regardless of how heavy it is).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionPrimeArgs {
    /// Override the default memory cap (K=10). Tests use this to verify
    /// the bound is enforced; production callers omit it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory_cap: Option<usize>,
}

// ----------------------------------------------------------------------------
// Session / skill verb args (spec 2026-05-15-agent-learning-loop)
// ----------------------------------------------------------------------------

/// Args for `session.start` — write a session id to `.orbit/.session-id`.
///
/// When `id` is supplied (typically by test fixtures or replay scenarios) it
/// is used verbatim. Otherwise a UUIDv4 is generated. Re-running with no `id`
/// overwrites with a new UUID — the intended "fresh session" semantics.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionStartArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

/// Args for `session.distill` — write or update `.orbit/sessions/<id>.yaml`.
///
/// `session_id` precedence: arg > `ORBIT_SESSION_ID` env > `.orbit/.session-id`.
/// `distillate` is the agent's end-of-session reflection (free text).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionDistillArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    pub distillate: String,
    /// Optional card slug scoping the distilled session. Resolution
    /// precedence (per spec 2026-05-16-session-handover ac-03): explicit
    /// arg first, else `.orbit/.session-card` fallback, else None. The
    /// id is NOT validated at distill time — validation lives at
    /// `session.set-card` time so the hot path stays cheap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

/// Args for `session.set-card` — validate a card id and write the canonical
/// slug to `.orbit/.session-card` so the next `session.distill` (typically
/// the Stop hook) scopes the session to that card.
///
/// See spec 2026-05-16-session-handover ac-04.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionSetCardArgs {
    pub card_id: String,
}

/// Args for `session.handover` — return the most-recent matching Session.
/// Both fields optional: no `card_id` means "latest across all cards";
/// no `since` means "no lower bound". When the sessions directory is
/// absent or no Session matches, the result envelope carries
/// `handover: null` (NOT an error — same shape as `skill.recurrence`).
///
/// See spec 2026-05-16-session-handover ac-06.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SessionHandoverArgs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

/// Args for `skill.record-invocation` — append one row to the skill's stream.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillRecordInvocationArgs {
    pub skill_id: String,
    pub outcome: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Args for `skill.recurrence` — read per-outcome counts for one skill.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SkillRecurrenceArgs {
    pub skill_id: String,
    /// RFC 3339 cutoff — only rows with `timestamp >= since` are counted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<String>,
}

// ----------------------------------------------------------------------------
// Routine verb args (spec 2026-05-22-routine-proposals)
// ----------------------------------------------------------------------------

/// Args for `routine.chains` — reconstruct one chain per `session_id`
/// from every `.orbit/skills/*.invocations.jsonl` row. Per ac-01.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineChainsArgs {}

/// Args for `routine.detect` — run [`crate::routine::detect_recurring_chains`]
/// against the reconstructed session chains and return recurring chains
/// at or above the threshold. Per ac-02 + ac-05.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineDetectArgs {}

/// Args for `routine.author` — write a routine SKILL.md at
/// `.claude/skills/<name>/SKILL.md` carrying validated front-matter.
/// Per ac-03 + ac-04.
///
/// Idempotency: if a routine for the same `chain_id` already exists
/// anywhere under `.claude/skills/` or its `.archive/` subtree, the
/// verb returns `RoutineAuthorResult { written: false, path }` rather
/// than overwriting. The author-archive (ac-09) is the load-bearing
/// case: once an author archives a routine, the agent never re-authors
/// the same chain.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineAuthorArgs {
    /// Ordered skill_id sequence the routine wraps. Length ≥ 2 is
    /// enforced — single-skill "chains" don't go through this path.
    pub chain: Vec<String>,
    /// Directory name under `.claude/skills/`. When omitted the
    /// substrate derives one from the chain via
    /// [`crate::routine::default_routine_name`]. Author renames later
    /// don't break the content-addressed lookup (per ac-09).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// One-sentence description for the front-matter. Optional —
    /// defaults to a derived sentence naming the chain.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// SKILL.md body that follows the front-matter block. Optional —
    /// the substrate emits a default body listing the chain steps when
    /// the caller omits one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Override the substrate timestamp used for both `created_at`
    /// and the initial `last_verified`. For migration and test use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// How many times the chain was observed — surfaces in the result
    /// for the agent's commit message ("N occurrences"). Optional;
    /// defaults to `None` (caller may pass for traceability).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub occurrences: Option<usize>,
}

/// Args for `routine.verify` — re-validate every `/orb:<verb>`
/// reference in the routine's SKILL.md body and on pass write the
/// run timestamp to the routine's `last_verified` front-matter field.
/// Per ac-06.
///
/// The verb is the *only* writer of `last_verified` —
/// `audit.conformance` is read-only on routines (ac-06 split).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineVerifyArgs {
    /// Path to the routine's SKILL.md, relative to the repo root.
    pub path: String,
    /// Override the substrate timestamp written into `last_verified`.
    /// For migration / test use.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Args for `spec.note` — append a timestamped note to a spec.
///
/// The `timestamp` arg is the documented test/migration seam. Production
/// callers omit it and the substrate stamps RFC 3339 UTC at append time.
/// Migration tools (Migration B in the spec — "bd notes → spec.note events")
/// pre-supply the original bd-recorded timestamp so historical ordering
/// survives the cutover.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SpecNoteArgs {
    pub id: String,
    pub body: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// Typed verb response. One variant per verb, mirroring [`VerbRequest`].
///
/// Note: `Eq` is intentionally omitted because `MemoryMatchResult`
/// carries `f32` scores (PartialEq only). `PartialEq` is sufficient for
/// every existing call site (test assertions, envelope round-trip).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "verb", content = "result")]
pub enum VerbResponse {
    #[serde(rename = "spec.list")]
    SpecList(SpecListResult),
    #[serde(rename = "spec.show")]
    SpecShow(SpecShowResult),
    #[serde(rename = "spec.resolve")]
    SpecResolve(SpecResolveResult),
    #[serde(rename = "spec.note")]
    SpecNote(SpecNoteResult),
    #[serde(rename = "spec.create")]
    SpecCreate(SpecCreateResult),
    #[serde(rename = "spec.update")]
    SpecUpdate(SpecUpdateResult),
    #[serde(rename = "spec.close")]
    SpecClose(SpecCloseResult),
    #[serde(rename = "spec.acs")]
    SpecAcs(SpecAcsResult),
    #[serde(rename = "spec.next-ac")]
    SpecNextAc(SpecNextAcResult),
    #[serde(rename = "spec.blocking-gate")]
    SpecBlockingGate(SpecBlockingGateResult),
    #[serde(rename = "spec.has-unchecked")]
    SpecHasUnchecked(SpecHasUncheckedResult),
    #[serde(rename = "spec.check")]
    SpecCheck(SpecCheckResult),
    #[serde(rename = "spec.uncheck")]
    SpecUncheck(SpecUncheckResult),
    #[serde(rename = "spec.promote")]
    SpecPromote(SpecPromoteResult),
    #[serde(rename = "task.open")]
    TaskOpen(TaskOpenResult),
    #[serde(rename = "task.list")]
    TaskList(TaskListResult),
    #[serde(rename = "task.show")]
    TaskShow(TaskShowResult),
    #[serde(rename = "task.ready")]
    TaskReady(TaskListResult),
    #[serde(rename = "task.claim")]
    TaskClaim(TaskEventResult),
    #[serde(rename = "task.update")]
    TaskUpdate(TaskEventResult),
    #[serde(rename = "task.done")]
    TaskDone(TaskEventResult),
    #[serde(rename = "memory.remember")]
    MemoryRemember(MemoryRememberResult),
    #[serde(rename = "memory.list")]
    MemoryList(MemoryListResult),
    #[serde(rename = "memory.search")]
    MemorySearch(MemoryListResult),
    #[serde(rename = "memory.match")]
    MemoryMatch(MemoryMatchResult),
    #[serde(rename = "card.show")]
    CardShow(CardShowResult),
    #[serde(rename = "card.list")]
    CardList(CardListResult),
    #[serde(rename = "card.search")]
    CardSearch(CardListResult),
    #[serde(rename = "card.tree")]
    CardTree(CardTreeResult),
    #[serde(rename = "card.specs")]
    CardSpecs(CardSpecsResult),
    #[serde(rename = "overview")]
    Overview(OverviewResult),
    #[serde(rename = "graph")]
    Graph(GraphResult),
    #[serde(rename = "audit.drift")]
    AuditDrift(AuditDriftResult),
    #[serde(rename = "audit.topology")]
    AuditTopology(AuditTopologyResult),
    #[serde(rename = "audit.conformance")]
    AuditConformance(AuditConformanceResult),
    #[serde(rename = "topology.setup")]
    TopologySetup(TopologySetupResult),
    #[serde(rename = "substrate.classify")]
    SubstrateClassify(SubstrateClassifyResult),
    #[serde(rename = "choice.show")]
    ChoiceShow(ChoiceShowResult),
    #[serde(rename = "choice.list")]
    ChoiceList(ChoiceListResult),
    #[serde(rename = "choice.search")]
    ChoiceSearch(ChoiceListResult),
    #[serde(rename = "session.prime")]
    SessionPrime(SessionPrimeResult),
    #[serde(rename = "session.start")]
    SessionStart(SessionStartResult),
    #[serde(rename = "session.distill")]
    SessionDistill(SessionDistillResult),
    #[serde(rename = "session.set-card")]
    SessionSetCard(SessionSetCardResult),
    #[serde(rename = "session.handover")]
    SessionHandover(SessionHandoverResult),
    #[serde(rename = "skill.record-invocation")]
    SkillRecordInvocation(SkillRecordInvocationResult),
    #[serde(rename = "skill.recurrence")]
    SkillRecurrence(SkillRecurrenceResult),
    #[serde(rename = "routine.chains")]
    RoutineChains(RoutineChainsResult),
    #[serde(rename = "routine.detect")]
    RoutineDetect(RoutineDetectResult),
    #[serde(rename = "routine.author")]
    RoutineAuthor(RoutineAuthorResult),
    #[serde(rename = "routine.verify")]
    RoutineVerify(RoutineVerifyResult),
}

/// Result for `spec.list`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecListResult {
    pub specs: Vec<SpecSummary>,
}

/// Result for `spec.show`. Wraps the on-disk Spec; future fields (resolved
/// note count, derived task counts) extend the wrapper without breaking the
/// envelope contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecShowResult {
    pub spec: Spec,
}

/// Result for `spec.resolve`. The two success branches map to D1's
/// `{resolved: "<id>"}` and `{prompt_with: [...]}`; the halt branch is an
/// `Error::unavailable` carrying the canonical D5 halt message and so does
/// not appear here.
///
/// Wire shape is tagged on `outcome`: `{"outcome": "resolved", "id": "..."}`
/// or `{"outcome": "prompt", "candidates": [...]}`. Skills branch on
/// `outcome` and never expand the prose.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "outcome", rename_all = "snake_case")]
pub enum SpecResolveResult {
    /// Single open spec found via inference (the bound card's only open
    /// spec, or the only open spec project-wide). Skills use this id
    /// without prompting.
    Resolved {
        /// The resolved spec id (full slug, e.g.
        /// `2026-05-19-skills-infer-or-prompt-before-halt`).
        id: String,
        /// Why this id was chosen — `bound_card` (single open spec
        /// under `.orbit/.session-card`), `card_arg` (single open spec
        /// under the explicit `--card` override), or `single_open`
        /// (project-wide fallback). Skills surface this in their
        /// "before doing other work" preamble so the user sees the
        /// inference chain.
        source: String,
    },
    /// Multiple open specs match; the skill must AskUserQuestion. Each
    /// candidate carries the spec id and a one-line goal label so the
    /// menu is self-describing without a second `spec.show` round-trip.
    /// Per decision D4.
    Prompt {
        /// Open specs to present in the menu, sorted by id.
        candidates: Vec<SpecResolveCandidate>,
        /// Why the prompt fires — `unbound` (no `.session-card` and
        /// multiple open specs project-wide), `bound_card_multi`
        /// (`.session-card` bound to a card with multiple open specs),
        /// or `card_arg_multi` (explicit `--card` with multiple opens).
        /// Surfaces in the prompt preamble.
        source: String,
    },
}

/// A spec entry the resolver returns in the prompt branch. Per decision
/// D4: spec id plus a one-line goal label is the high-leverage middle
/// ground — bare ids are uninterpretable in a many-spec project, full
/// goals are noisy in a menu.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecResolveCandidate {
    /// Full spec id (e.g. `2026-05-19-skills-infer-or-prompt-before-halt`).
    pub id: String,
    /// First newline-bounded line of the spec's `goal` field. Matches
    /// the CLI's `first_line` helper. Empty goal yields an empty
    /// string; the caller renders it verbatim into the menu label.
    pub goal_first_line: String,
}

/// Result for `spec.note` — echoes the appended event so callers can confirm
/// the substrate-stamped timestamp without re-reading the stream.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecNoteResult {
    pub note: NoteEvent,
}

/// Result for `spec.create` — echoes the on-disk spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecCreateResult {
    pub spec: Spec,
}

/// Result for `spec.update` — returns the post-update spec.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecUpdateResult {
    pub spec: Spec,
}

/// Result for `spec.close` — returns the closed spec plus a list of cards
/// whose `specs` array was extended.
///
/// `forced_unchecked` lists ACs that were bypassed via the `force` flag
/// (per spec 2026-05-13-spec-close-ac-preflight ac-03); empty when no
/// bypass occurred. `deferrable_open` lists ACs of deferrable kind
/// (`Ops`/`Observation` per `AcType::blocks_close()`) that remained
/// unchecked at close (spec 2026-05-16-ac-taxonomy ac-02); empty when
/// no deferrable ACs remained open. Both fields use
/// `skip_serializing_if = "Vec::is_empty"` so happy-path responses
/// remain byte-identical to the pre-change shape.
///
/// Note: this struct intentionally does NOT carry `deny_unknown_fields`,
/// preserving forward-additive read compatibility for callers that
/// cache an older response shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecCloseResult {
    pub spec: Spec,
    pub cards_updated: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forced_unchecked: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub deferrable_open: Vec<String>,
    /// Memory keys whose reconciliation was bypassed via `--force` at the
    /// close-time memory-match gate. Empty (and `skip_serializing_if`-
    /// omitted) when no bypass occurred. Per spec
    /// 2026-05-19-memory-gates-decisions ac-04 (D4).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forced_unreconciled: Vec<String>,
    /// Topology drift entries for subsystems the closing spec text touched.
    /// Word-boundary match (regex `\b<regex::escape(subsystem)>\b`,
    /// case-insensitive) of subsystem names ≥ 5 characters against the
    /// concatenation of `spec.yaml + interview.md + tabletop-note.md`
    /// (each sidecar included when present). Non-blocking — closure
    /// proceeds with exit 0; this field is informational. Empty (and
    /// `skip_serializing_if`-omitted) when not configured or when no
    /// matches exist. Per spec 2026-05-18-topology-substrate-wires ac-03.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topology_warnings: Vec<TopologyDriftEntry>,
}

// ----------------------------------------------------------------------------
// Spec acceptance-criterion verb results (per spec 2026-05-24-port-acceptance-shim).
// ----------------------------------------------------------------------------

/// Result for `spec.acs` — the full acceptance_criteria list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecAcsResult {
    pub acs: Vec<AcceptanceCriterion>,
}

/// Pointer to the first unchecked AC not blocked by an unchecked gate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NextAc {
    pub id: String,
    pub gate: bool,
}

/// Result for `spec.next-ac` — `None` when all checked or when the first
/// unchecked is preceded by an unchecked gate of a different id.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecNextAcResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<NextAc>,
}

/// Pointer to the first unchecked gate AC.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockingGate {
    pub id: String,
    pub description: String,
}

/// Result for `spec.blocking-gate` — `None` when no unchecked gate exists.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecBlockingGateResult {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking: Option<BlockingGate>,
}

/// Result for `spec.has-unchecked` — true if any AC is unchecked.
/// Raw-axis (`!checked`) traversal per ac-06 of spec 2026-05-24-port-acceptance-shim.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecHasUncheckedResult {
    pub has_unchecked: bool,
}

/// Result for `spec.check` — the spec post-flip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecCheckResult {
    pub spec: Spec,
}

/// Result for `spec.uncheck` — the spec post-flip.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecUncheckResult {
    pub spec: Spec,
}

/// Result for `spec.promote` — the freshly-created spec, or the planned
/// spec when `dry_run: true`. `dry_run` echoes back so consumers can
/// branch on whether the write happened.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecPromoteResult {
    pub spec: Spec,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryRememberResult {
    pub memory: Memory,
    /// Advisory nudge populated when the stored memory carried the
    /// canonical `topology` label and the caller did not pass
    /// `--no-nudge`. Non-blocking — the memory still stored. Per spec
    /// 2026-05-18-topology-substrate-wires ac-04.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nudge: Option<String>,
    /// Advisory warning populated when the stored memory's body leads
    /// with a state observation rather than a mechanism clause and the
    /// caller did not pass `--no-warn`. Non-blocking — the memory is
    /// stored as written. Per spec 2026-05-19-memory-gates-decisions
    /// ac-05 (D5b).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shape_warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryListResult {
    pub memories: Vec<Memory>,
}

/// Result for `memory.match` — ranked matches above a relevance threshold.
/// Per spec 2026-05-19-memory-gates-decisions ac-01/ac-02 (D1).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryMatchResult {
    pub matches: Vec<MemoryMatch>,
}

/// One ranked match returned by `memory.match`. The score is normalised
/// (0.0..=1.0) and `reason` is a short phrase explaining the overlap
/// (e.g. `"label overlap on 'drive'"`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryMatch {
    pub memory: Memory,
    pub score: f32,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardShowResult {
    pub slug: String,
    pub card: Card,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardListResult {
    pub cards: Vec<CardSummary>,
}

/// Projection of a card for list/search views.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardSummary {
    pub slug: String,
    pub feature: String,
    pub goal: String,
    pub maturity: String,
}

/// Result for `card.tree` — local subgraph from a root card.
///
/// The `tree` node is the resolved root; its `outgoing` and `incoming`
/// vectors carry the immediate edges (one hop). Each edge's `target` is
/// itself a `CardTreeNode`, recursing up to the configured depth. At the
/// depth boundary or on a revisited slug, `target.truncated = true` and
/// its `outgoing` / `incoming` vectors are empty.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardTreeResult {
    pub root: String,
    pub depth: u32,
    pub tree: CardTreeNode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardTreeNode {
    pub slug: String,
    pub feature: String,
    pub outgoing: Vec<CardTreeEdge>,
    pub incoming: Vec<CardTreeEdge>,
    /// True when this node was reached at the depth boundary or on a
    /// cycle revisit — its edges are intentionally elided.
    #[serde(default, skip_serializing_if = "is_false")]
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardTreeEdge {
    pub kind: String,
    pub reason: String,
    pub target: CardTreeNode,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Result for `card.specs` — every spec that's linked to a card by either
/// direction (card → spec via `card.specs[]`, or spec → card via
/// `spec.cards[]`). Each entry names whether both directions agree; one-way
/// references surface as drift.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardSpecsResult {
    pub root: String,
    pub specs: Vec<CardSpecsEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardSpecsEntry {
    pub spec_id: String,
    pub spec_path: String,
    /// True if the card's `specs:` array lists this spec.
    pub listed_on_card: bool,
    /// True if the spec's `cards:` array back-references this card.
    pub back_referenced_by_spec: bool,
    /// `open`, `closed`, `missing`, or `parse-failed` — gives the caller
    /// enough context to triage drift without re-reading the spec.
    pub status: String,
}

/// Result for `overview` — single-screen project synthesis. All vectors are
/// bounded by `memory_cap` (default K=10); overflow counters expose how
/// much was elided so the caller can scroll the substrate manually if it
/// matters.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OverviewResult {
    pub open_spec_count: usize,
    /// Up to K=10 most-recent open spec ids (by id, which is date-prefixed).
    pub recent_open_spec_ids: Vec<String>,
    /// Number of open specs not surfaced because they fell past the cap.
    pub spec_overflow: usize,
    pub cards_by_maturity: CardMaturityCounts,
    pub memories: Vec<Memory>,
    /// Card with the highest degree (outgoing + incoming `relations:` count;
    /// `specs:` entries do NOT contribute). Ties broken by lowest numeric id.
    /// `None` when no card has any relations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub most_connected_card: Option<MostConnectedCard>,
    /// Cards with `specs: []` AND zero incoming `relations:`. Capped at
    /// K=10; `orphan_overflow` counts the rest.
    pub orphans: Vec<String>,
    pub orphan_overflow: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CardMaturityCounts {
    pub planned: usize,
    pub emerging: usize,
    pub established: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MostConnectedCard {
    pub slug: String,
    pub feature: String,
    pub degree: usize,
}

/// Result for `graph` — the rendered text plus the format it's in. The
/// caller pastes `text` into a markdown block or graphviz tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphResult {
    pub format: String,
    pub text: String,
}

/// Result for `audit.drift` — one entry per unknown top-level field across
/// all walked files. Empty `drift` means the substrate is clean against the
/// canonical schema.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditDriftResult {
    pub drift: Vec<DriftEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DriftEntry {
    pub path: String,
    pub kind: String,
    pub field: String,
    pub disposition: String,
}

/// Result for `audit.topology` — three states are possible: (a) topology
/// capability not configured (`configured: false`, empty drift), (b)
/// configured and clean (`configured: true`, empty drift), (c) configured
/// with drift (`configured: true`, non-empty drift). Exit code is 0 for
/// all three; consumers discriminate via the envelope, never via `$?`.
/// Per spec 2026-05-18-documentation-topology ac-06.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditTopologyResult {
    /// True when `.orbit/config.yaml` exists AND `docs.topology` is set.
    /// False when either is missing — the topology capability is opt-in.
    pub configured: bool,
    /// Drift entries, one per detected issue. Empty when configured + clean
    /// AND when not configured.
    pub topology_drift: Vec<TopologyDriftEntry>,
}

/// Result for `audit.conformance` — workflow-conformance audit.
/// Aggregates `audit.drift` + `audit.topology` results verbatim under
/// `aggregated.{drift,topology}` and surfaces new finding families
/// (plugin-canonical-file drift, card-state, memo staleness, pin
/// state) under `findings`. Per spec 2026-05-19-workflow-conformance
/// ac-01.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AuditConformanceResult {
    /// New finding families produced by this verb (NOT audit.drift /
    /// audit.topology results — those live under `aggregated`).
    pub findings: Vec<ConformanceFinding>,
    /// Existing sub-verb results carried verbatim. Byte-equal contract:
    /// the JSON for `aggregated.drift` is byte-identical to the
    /// standalone `audit.drift --json` `data.result` payload; same for
    /// `aggregated.topology`.
    pub aggregated: AggregatedAudits,
    /// Plugin-version pin state. Per ac-05.
    pub pin: PinState,
}

/// Aggregated sub-verb audit results. Per ac-06.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AggregatedAudits {
    pub drift: AuditDriftResult,
    pub topology: AuditTopologyResult,
}

/// A single workflow-conformance finding. Severity / state are
/// slug-shaped strings (not enums) for forward-compatibility — future
/// finding families add new state values without a schema break.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ConformanceFinding {
    /// Severity band: "high" | "medium" | "low".
    pub severity: String,
    /// Subsystem the finding belongs to: "cards" | "memos" | "setup".
    pub subsystem: String,
    /// Opaque subject identifier — typically a card id or a file path.
    pub subject: String,
    /// State slug describing the gap: "ready_for_tabletop" | "stale" |
    /// "byte_drift" | "missing" | "pin_behind" | "pin_ahead".
    pub state: String,
    /// Optional finding-family-specific structured context.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_yaml::Value>,
    /// Next-action handle the agent can run without translation.
    pub remediation: Remediation,
}

/// Remediation handle attached to every ConformanceFinding.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Remediation {
    /// The agent-runnable verb: "orbit setup", "/orb:tabletop 39",
    /// "/orb:distill .orbit/memos/...", etc.
    pub verb: String,
    /// Short rationale — why this remediation matches the finding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rationale: Option<String>,
}

/// Plugin-version pin state. The "installed" version is the
/// orbit-state binary's `CARGO_PKG_VERSION` at compile time
/// (lockstep release with the plugin manifest version). The
/// "pinned" version is read from `.orbit/config.yaml`'s
/// `plugin_version` field. Per ac-05.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PinState {
    /// The `plugin_version` value from `.orbit/config.yaml`, or `None`
    /// if the field is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pinned: Option<String>,
    /// The installed plugin version (orbit-state binary's
    /// `CARGO_PKG_VERSION`).
    pub current: String,
    /// Derived state: "unpinned" | "matches" | "pin_behind" |
    /// "pin_ahead".
    pub status: String,
}

/// Result for `topology.setup`. Per spec
/// 2026-05-18-topology-substrate-migration ac-05.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopologySetupResult {
    /// True if the brownfield-cleanup arm ran (legacy `docs.topology`
    /// key found and stripped from `.orbit/config.yaml`).
    pub config_cleaned: bool,
    /// True if `.orbit/topology/` was newly created in this invocation.
    /// False on idempotent re-runs.
    pub dir_created: bool,
    /// Subsystem slugs whose seed entries were newly written. Empty on
    /// idempotent re-runs AND on non-plugin-repo projects (the
    /// substrate-typed seeds describe orbit's own substrate types and
    /// are a category error in any other project — see `readme_created`
    /// for the gating side). Per spec
    /// 2026-05-24-setup-is-orbit-state-aware ac-12.
    pub seeds_created: Vec<String>,
    /// Subsystem slugs whose seed entries already existed and were
    /// skipped (operator edits preserved — no overwrite).
    pub seeds_skipped: Vec<String>,
    /// True if the prompt fired and was declined (operator chose not to
    /// scaffold). Mutually exclusive with the other create/skip fields.
    pub declined: bool,
    /// True if a `.orbit/topology/README.md` was newly written. The
    /// README is the non-plugin-repo seed: a one-line pointer at
    /// `/orb:topology` that primes the operator on how to author the
    /// first topology entry, replacing the substrate-typed seeds that
    /// only make sense inside the orbit-plugin source repo. Idempotent
    /// (false on re-run when the file already exists). Per spec
    /// 2026-05-24-setup-is-orbit-state-aware ac-12.
    #[serde(default)]
    pub readme_created: bool,
}

/// Result for `substrate.classify` — names the current layout state of
/// the working tree at `layout.repo_root()`. Per spec
/// 2026-05-24-setup-is-orbit-state-aware ac-11.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SubstrateClassifyResult {
    /// One of the six mutually-exclusive layout states.
    pub state: SubstrateLayoutState,
}

/// A single drift entry from `audit.topology`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TopologyDriftEntry {
    /// The subsystem name (for stale_pointer / shape_drift) or the
    /// detected codebase directory (for missing_entry).
    pub subsystem: String,
    /// One of: `stale_pointer`, `missing_entry`, `shape_drift`.
    pub drift_kind: String,
    /// Optional detail — the offending path for stale_pointer, the
    /// missing anchor for shape_drift, or empty for missing_entry.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChoiceShowResult {
    pub choice: Choice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChoiceListResult {
    pub choices: Vec<ChoiceSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChoiceSummary {
    pub id: String,
    pub title: String,
    pub status: String,
    pub date_created: String,
}

/// Result for `session.prime` — agent priming context. Per ac-11:
/// `f(N specs, M memories) ≤ 40 + 2*open_specs + min(M,10)`.
///
/// The bound is "items in the response", not bytes/tokens — agents can size
/// their context separately. Items here are open spec summaries + memory
/// references.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionPrimeResult {
    pub open_specs: Vec<SpecSummary>,
    pub memories: Vec<Memory>,
    /// Most-recent Session across all cards (no card filter at prime —
    /// per-card lookup is via `orbit session handover --card <id>`). The
    /// agent reads this before any other action when it's Some. See
    /// spec 2026-05-16-session-handover ac-07.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub handover: Option<HandoverSummary>,
    /// Hard upper bound on items: 40 + 2*open_specs + min(memory_cap, 10),
    /// plus +1 when `handover` is Some — so clients know the field is in
    /// the bound (otherwise it's invisible to them).
    pub item_bound: usize,
    /// Next-step suggestion. Per tree-views ac-07 this references `orbit
    /// overview` so a fresh session reaches the synthesis layer in one
    /// step — the load-bearing wire from card 0033's surfacing scenario.
    /// When `handover` is Some the prefix sentinel from ac-07 of spec
    /// 2026-05-16-session-handover (`"Read the handover above before any
    /// other action. "`) is joined onto the front so the next agent reads
    /// the handover before the overview.
    pub next_step: String,
    /// Topology drift entries surfaced at session start. `Some` whenever
    /// the topology capability is configured (`audit_topology(...).configured == true`,
    /// i.e. `.orbit/config.yaml` exists AND `docs.topology` is set) —
    /// `Some(vec![])` for the configured + clean case, `Some(non-empty)`
    /// when drift is present. `None` (key omitted via
    /// `skip_serializing_if`) when the topology capability is not
    /// configured. Per spec 2026-05-18-topology-substrate-wires ac-02.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub topology_drift: Option<Vec<TopologyDriftEntry>>,
}

/// Result for `session.start` — echoes the session id written to disk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionStartResult {
    pub session_id: String,
    pub path: String,
}

/// Result for `session.distill` — echoes the post-write Session entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionDistillResult {
    pub session: Session,
}

/// Result for `session.set-card` — echoes the canonical resolved slug
/// and the path the substrate wrote. See spec 2026-05-16-session-handover
/// ac-04 for the validation + atomic-write contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionSetCardResult {
    pub card_id: String,
    pub path: String,
}

/// Per-card session summary surfaced by `session.handover` and embedded
/// in the `session.prime` envelope (ac-07). Subset of `Session` carrying
/// just the orientation-relevant fields — the full entity is on disk for
/// callers who want it.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HandoverSummary {
    pub session_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_id: Option<String>,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ended_at: Option<String>,
    pub distillate: String,
}

/// Result for `session.handover` — the most-recent matching Session, or
/// `None` when no Session matches. See spec 2026-05-16-session-handover
/// ac-06 for the no-match-is-not-an-error contract.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionHandoverResult {
    pub handover: Option<HandoverSummary>,
}

/// Result for `skill.record-invocation` — echoes the appended row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRecordInvocationResult {
    pub invocation: SkillInvocation,
}

/// One invocation entry returned by `skill.recurrence`. `correction` is
/// omitted from the wire when the original record had none.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecurrenceInvocation {
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correction: Option<String>,
}

/// One outcome bucket — count + the entries that contributed to it.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecurrenceBucket {
    pub count: usize,
    pub invocations: Vec<RecurrenceInvocation>,
}

/// Per-outcome breakdown for `skill.recurrence`. Every variant key is always
/// present (even with count 0) so agents can index without first checking
/// for missing keys.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecurrenceByOutcome {
    pub worked: RecurrenceBucket,
    pub partial: RecurrenceBucket,
    #[serde(rename = "didnt-apply")]
    pub didnt_apply: RecurrenceBucket,
    pub incorrect: RecurrenceBucket,
}

/// Result for `skill.recurrence` — per-outcome counts + invocation entries.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkillRecurrenceResult {
    pub skill_id: String,
    pub by_outcome: RecurrenceByOutcome,
    pub total: usize,
}

/// Result for `routine.chains` — per-session reconstructed chains
/// (ordered skill_id sequences, one per `session_id`). Per ac-01.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineChainsResult {
    pub chains: Vec<crate::routine::SessionChain>,
}

/// Result for `routine.detect` — recurring chains that pass the
/// threshold (per ac-02 + ac-05). DAG-shaped patterns are excluded
/// from the output even when they recur (v1 is sequential-only per
/// ac-05).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineDetectResult {
    pub recurring: Vec<crate::routine::RecurringChain>,
}

/// Result for `routine.author`. Per ac-03 + ac-09.
///
/// `written` is `true` when this call created the SKILL.md, `false`
/// when an existing routine with the same `chain_id` was found anywhere
/// under `.claude/skills/` or its `.archive/` subtree. The `path` field
/// is always populated — to the newly-written file when `written` is
/// true, or to the existing matching file when `written` is false.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineAuthorResult {
    pub written: bool,
    pub path: String,
    pub chain_id: String,
    pub name: String,
}

/// Result for `routine.verify`. Per ac-06 + ac-07.
///
/// `resolved` is the list of `/orb:<verb>` references that resolved to
/// a live skill; `broken_refs` is the list that did not. `last_verified`
/// is the timestamp written into the SKILL.md when every reference
/// resolved; absent when the verification failed and the timestamp was
/// NOT advanced.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoutineVerifyResult {
    pub path: String,
    pub resolved: Vec<String>,
    pub broken_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_verified: Option<String>,
}

/// Reduced view of a task — its current state derived from the last event
/// for its task_id, plus the event history count.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskState {
    pub task_id: String,
    pub spec_id: String,
    /// Current state — `open`, `claim`, `update`, or `done`.
    pub state: String,
    /// Body from the last event that carried one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<String>,
    /// Labels carried on the last event (not aggregated).
    #[serde(default)]
    pub labels: Vec<String>,
    /// Timestamp of the last event.
    pub timestamp: String,
    /// Number of events in this task's history.
    pub event_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskOpenResult {
    pub event: TaskEvent,
    pub task_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskListResult {
    pub tasks: Vec<TaskState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskShowResult {
    pub state: TaskState,
    pub events: Vec<TaskEvent>,
}

/// Result for the three Claim/Update/Done verbs — each appends one event
/// and echoes it for confirmation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaskEventResult {
    pub event: TaskEvent,
}

/// Projection of a spec for list views — id, goal, status, plus the cards it
/// advances and any labels. Excludes ACs and other heavy fields.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpecSummary {
    pub id: String,
    pub goal: String,
    pub status: String,
    #[serde(default)]
    pub cards: Vec<String>,
    #[serde(default)]
    pub labels: Vec<String>,
}

// ============================================================================
// Dispatch
// ============================================================================

/// Dispatch a verb against the layout. The single entry point both CLI and
/// MCP call — the architectural guarantee from ac-05 lives here.
pub fn execute(layout: &OrbitLayout, request: &VerbRequest) -> Result<VerbResponse> {
    match request {
        VerbRequest::SpecList(args) => spec_list(layout, args).map(VerbResponse::SpecList),
        VerbRequest::SpecShow(args) => spec_show(layout, args).map(VerbResponse::SpecShow),
        VerbRequest::SpecResolve(args) => spec_resolve(layout, args).map(VerbResponse::SpecResolve),
        VerbRequest::SpecNote(args) => spec_note(layout, args).map(VerbResponse::SpecNote),
        VerbRequest::SpecCreate(args) => spec_create(layout, args).map(VerbResponse::SpecCreate),
        VerbRequest::SpecUpdate(args) => spec_update(layout, args).map(VerbResponse::SpecUpdate),
        VerbRequest::SpecClose(args) => spec_close(layout, args).map(VerbResponse::SpecClose),
        VerbRequest::SpecAcs(args) => spec_acs(layout, args).map(VerbResponse::SpecAcs),
        VerbRequest::SpecNextAc(args) => spec_next_ac(layout, args).map(VerbResponse::SpecNextAc),
        VerbRequest::SpecBlockingGate(args) => {
            spec_blocking_gate(layout, args).map(VerbResponse::SpecBlockingGate)
        }
        VerbRequest::SpecHasUnchecked(args) => {
            spec_has_unchecked(layout, args).map(VerbResponse::SpecHasUnchecked)
        }
        VerbRequest::SpecCheck(args) => spec_check(layout, args).map(VerbResponse::SpecCheck),
        VerbRequest::SpecUncheck(args) => {
            spec_uncheck(layout, args).map(VerbResponse::SpecUncheck)
        }
        VerbRequest::SpecPromote(args) => {
            spec_promote(layout, args).map(VerbResponse::SpecPromote)
        }
        VerbRequest::TaskOpen(args) => task_open(layout, args).map(VerbResponse::TaskOpen),
        VerbRequest::TaskList(args) => task_list(layout, args).map(VerbResponse::TaskList),
        VerbRequest::TaskShow(args) => task_show(layout, args).map(VerbResponse::TaskShow),
        VerbRequest::TaskReady(args) => task_ready(layout, args).map(VerbResponse::TaskReady),
        VerbRequest::TaskClaim(args) => task_claim(layout, args).map(VerbResponse::TaskClaim),
        VerbRequest::TaskUpdate(args) => task_update(layout, args).map(VerbResponse::TaskUpdate),
        VerbRequest::TaskDone(args) => task_done(layout, args).map(VerbResponse::TaskDone),
        VerbRequest::MemoryRemember(args) => {
            memory_remember(layout, args).map(VerbResponse::MemoryRemember)
        }
        VerbRequest::MemoryList(args) => memory_list(layout, args).map(VerbResponse::MemoryList),
        VerbRequest::MemorySearch(args) => {
            memory_search(layout, args).map(VerbResponse::MemorySearch)
        }
        VerbRequest::MemoryMatch(args) => {
            memory_match(layout, args).map(VerbResponse::MemoryMatch)
        }
        VerbRequest::CardShow(args) => card_show(layout, args).map(VerbResponse::CardShow),
        VerbRequest::CardList(args) => card_list(layout, args).map(VerbResponse::CardList),
        VerbRequest::CardSearch(args) => card_search(layout, args).map(VerbResponse::CardSearch),
        VerbRequest::CardTree(args) => card_tree(layout, args).map(VerbResponse::CardTree),
        VerbRequest::CardSpecs(args) => card_specs(layout, args).map(VerbResponse::CardSpecs),
        VerbRequest::Overview(args) => overview(layout, args).map(VerbResponse::Overview),
        VerbRequest::Graph(args) => graph(layout, args).map(VerbResponse::Graph),
        VerbRequest::AuditDrift(args) => audit_drift(layout, args).map(VerbResponse::AuditDrift),
        VerbRequest::TopologySetup(args) => topology_setup(layout, args).map(VerbResponse::TopologySetup),
        VerbRequest::SubstrateClassify(args) => {
            substrate_classify(layout, args).map(VerbResponse::SubstrateClassify)
        }
        VerbRequest::AuditTopology(args) => {
            audit_topology(layout, args).map(VerbResponse::AuditTopology)
        }
        VerbRequest::AuditConformance(args) => {
            audit_conformance(layout, args).map(VerbResponse::AuditConformance)
        }
        VerbRequest::ChoiceShow(args) => choice_show(layout, args).map(VerbResponse::ChoiceShow),
        VerbRequest::ChoiceList(args) => choice_list(layout, args).map(VerbResponse::ChoiceList),
        VerbRequest::ChoiceSearch(args) => {
            choice_search(layout, args).map(VerbResponse::ChoiceSearch)
        }
        VerbRequest::SessionPrime(args) => {
            session_prime(layout, args).map(VerbResponse::SessionPrime)
        }
        VerbRequest::SessionStart(args) => {
            session_start(layout, args).map(VerbResponse::SessionStart)
        }
        VerbRequest::SessionDistill(args) => {
            session_distill(layout, args).map(VerbResponse::SessionDistill)
        }
        VerbRequest::SessionSetCard(args) => {
            session_set_card(layout, args).map(VerbResponse::SessionSetCard)
        }
        VerbRequest::SessionHandover(args) => {
            session_handover(layout, args).map(VerbResponse::SessionHandover)
        }
        VerbRequest::SkillRecordInvocation(args) => {
            skill_record_invocation(layout, args).map(VerbResponse::SkillRecordInvocation)
        }
        VerbRequest::SkillRecurrence(args) => {
            skill_recurrence(layout, args).map(VerbResponse::SkillRecurrence)
        }
        VerbRequest::RoutineChains(args) => {
            routine_chains(layout, args).map(VerbResponse::RoutineChains)
        }
        VerbRequest::RoutineDetect(args) => {
            routine_detect(layout, args).map(VerbResponse::RoutineDetect)
        }
        VerbRequest::RoutineAuthor(args) => {
            routine_author(layout, args).map(VerbResponse::RoutineAuthor)
        }
        VerbRequest::RoutineVerify(args) => {
            routine_verify(layout, args).map(VerbResponse::RoutineVerify)
        }
    }
}

// ============================================================================
// Verb implementations
// ============================================================================

/// `spec.list` — enumerate spec files under `.orbit/specs/`, sorted by id.
///
/// Reads files directly (not the index). Reading from files is correct and
/// deterministic; once the index proves out for write paths, read verbs can
/// switch to index-backed for performance. ac-05 does not require index reads.
fn spec_list(layout: &OrbitLayout, args: &SpecListArgs) -> Result<SpecListResult> {
    const VERB: &str = "spec.list";

    if let Some(s) = args.status.as_deref() {
        if !matches!(s, "open" | "closed") {
            return Err(Error::malformed(
                VERB,
                format!("status must be 'open' or 'closed', got '{s}'"),
            ));
        }
    }

    let files = layout
        .list_spec_files()
        .map_err(|e| Error::unavailable(VERB, format!("list specs dir: {e}")))?;

    let mut specs = Vec::with_capacity(files.len());
    for path in files {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        let spec: Spec = parse_yaml(&text).map_err(|mut e| {
            // The canonical layer tags errors with verb="canonical"; re-tag to
            // the calling verb so the on-wire error format is correct.
            e.verb = VERB.into();
            e
        })?;
        let status = match spec.status {
            SpecStatus::Open => "open",
            SpecStatus::Closed => "closed",
        };
        if let Some(filter) = args.status.as_deref() {
            if status != filter {
                continue;
            }
        }
        specs.push(SpecSummary {
            id: spec.id,
            goal: spec.goal,
            status: status.into(),
            cards: spec.cards,
            labels: spec.labels,
        });
    }

    specs.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(SpecListResult { specs })
}

/// `spec.note` — append a note event to a spec's notes JSONL stream.
///
/// Locking: acquires the spec's lock so concurrent appends serialise. The
/// raw write itself is POSIX-O_APPEND atomic, but the lock guarantees a
/// well-defined append order across multiple writers.
fn spec_note(layout: &OrbitLayout, args: &SpecNoteArgs) -> Result<SpecNoteResult> {
    const VERB: &str = "spec.note";

    if args.id.is_empty() {
        return Err(Error::malformed(VERB, "id must not be empty"));
    }
    if args.id.contains('/') || args.id.contains('\\') || args.id.contains("..") {
        return Err(Error::malformed(
            VERB,
            format!("id must not contain path separators or '..': '{}'", args.id),
        ));
    }
    if args.body.is_empty() {
        return Err(Error::malformed(VERB, "body must not be empty"));
    }

    // Spec must exist before we can attach a note to it.
    let spec_path = layout.spec_file(&args.id);
    if !spec_path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no spec at {}", spec_path.display()),
        ));
    }

    let timestamp = match &args.timestamp {
        Some(t) => t.clone(),
        None => current_rfc3339_utc().map_err(|e| {
            Error::unavailable(VERB, format!("substrate timestamp generation failed: {e}"))
        })?,
    };

    let event = NoteEvent {
        spec_id: args.id.clone(),
        body: args.body.clone(),
        labels: args.labels.clone(),
        timestamp,
    };

    // Acquire the spec lock for the append. Reads of the same stream don't
    // need this — see ac-03's "reads do not require lock acquisition" rule.
    let lock_key = format!("spec-{}", args.id);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    // serialise_json_line guarantees a trailing newline, which append_jsonl_line
    // requires.
    let line = serialise_json_line(&event).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    let stream_path = layout.notes_stream(&args.id);
    append_jsonl_line(&stream_path, &line).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SpecNoteResult { note: event })
}

/// `spec.create` — write a new spec.yaml file.
///
/// Conflict if a spec with that id already exists. Lock is acquired so two
/// concurrent creates can't race.
fn spec_create(layout: &OrbitLayout, args: &SpecCreateArgs) -> Result<SpecCreateResult> {
    const VERB: &str = "spec.create";

    validate_spec_id(VERB, &args.id)?;
    if args.goal.is_empty() {
        return Err(Error::malformed(VERB, "goal must not be empty"));
    }

    let lock_key = format!("spec-{}", args.id);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    let path = layout.spec_file(&args.id);
    if path.exists() {
        return Err(Error::conflict(
            VERB,
            format!("spec already exists at {}", path.display()),
        ));
    }
    layout
        .ensure_dirs()
        .map_err(|e| Error::unavailable(VERB, format!("ensure dirs: {e}")))?;
    layout
        .ensure_spec_dir(&args.id)
        .map_err(|e| Error::unavailable(VERB, format!("ensure spec dir: {e}")))?;

    let spec = Spec {
        id: args.id.clone(),
        goal: args.goal.clone(),
        cards: args.cards.clone(),
        status: SpecStatus::Open,
        labels: args.labels.clone(),
        acceptance_criteria: args.acceptance_criteria.clone(),
        memories_considered: Vec::new(),
    };
    let yaml = serialise_yaml(&spec).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    write_atomic(&path, yaml.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SpecCreateResult { spec })
}

/// `spec.update` — modify fields on an existing spec. Status changes are
/// not allowed here; `spec.close` owns that transition.
fn spec_update(layout: &OrbitLayout, args: &SpecUpdateArgs) -> Result<SpecUpdateResult> {
    const VERB: &str = "spec.update";

    validate_spec_id(VERB, &args.id)?;

    let lock_key = format!("spec-{}", args.id);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    let path = layout.spec_file(&args.id);
    if !path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no spec at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::unavailable(VERB, format!("read {}: {e}", path.display())))?;
    let mut spec: Spec = parse_yaml(&text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    // Apply field-by-field. Empty-goal still rejected (validation, not
    // omission).
    if let Some(goal) = &args.goal {
        if goal.is_empty() {
            return Err(Error::malformed(VERB, "goal must not be empty"));
        }
        spec.goal = goal.clone();
    }
    if let Some(cards) = &args.cards {
        spec.cards = cards.clone();
    }
    if let Some(labels) = &args.labels {
        spec.labels = labels.clone();
    }
    if let Some(acs) = &args.acceptance_criteria {
        spec.acceptance_criteria = acs.clone();
    }

    let yaml = serialise_yaml(&spec).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    write_atomic(&path, yaml.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SpecUpdateResult { spec })
}

/// `spec.close` — flip status to `closed` and transactionally append the
/// spec's path to every linked card's `specs` array.
///
/// Per ac-06: "transactional: either all linked cards update or none do,
/// with the spec remaining open if any update fails." Implementation:
///
/// 1. Acquire spec lock.
/// 2. Read spec; verify status == open.
/// 3. Read each linked card; build the proposed updated card.
///    Validate each (parse round-trip) BEFORE writing anything.
/// 4. Write each updated card atomically. On any failure mid-batch, roll
///    back the cards already written (using the pre-image we cached).
/// 5. If all card writes succeeded, write the closed spec.
/// 6. If the spec write fails after card writes succeeded, roll back cards
///    too — the spec remaining "open" with cards updated is an inconsistent
///    state and we'd rather pay the rollback cost than leave drift.
///
/// `cards_updated` in the result names the cards whose `specs` array now
/// contains this spec's relative path.
fn spec_close(layout: &OrbitLayout, args: &SpecCloseArgs) -> Result<SpecCloseResult> {
    const VERB: &str = "spec.close";

    validate_spec_id(VERB, &args.id)?;

    let lock_key = format!("spec-{}", args.id);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    let spec_path = layout.spec_file(&args.id);
    if !spec_path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no spec at {}", spec_path.display()),
        ));
    }
    let spec_text = std::fs::read_to_string(&spec_path)
        .map_err(|e| Error::unavailable(VERB, format!("read spec: {e}")))?;
    let mut spec: Spec = parse_yaml(&spec_text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    if spec.status == SpecStatus::Closed {
        return Err(Error::conflict(VERB, format!("spec '{}' already closed", spec.id)));
    }

    // AC pre-flight (spec 2026-05-13-spec-close-ac-preflight, ac-02 / ac-04;
    // generalised by spec 2026-05-16-ac-taxonomy ac-02). The spec's
    // acceptance_criteria are already in memory from the parse above, so
    // checking them is essentially free — we do this BEFORE the
    // unfinished-tasks check (which requires task-stream IO) so the cheaper
    // guard fails fast. The unfinished-tasks guard below is unchanged in
    // behaviour (ac-06 of the precursor spec).
    //
    // Blocking set: ACs that are unchecked AND of blocking kind
    // (Code/Config/Doc per AcType::blocks_close()). Unchecked deferrable-
    // kind ACs (Ops/Observation) are reported in the result's
    // `deferrable_open` field but do not block close.
    let unchecked_blocking: Vec<&AcceptanceCriterion> = spec
        .acceptance_criteria
        .iter()
        .filter(|ac| !ac.checked && ac.ac_type.blocks_close())
        .collect();
    let deferrable_open: Vec<String> = spec
        .acceptance_criteria
        .iter()
        .filter(|ac| !ac.checked && !ac.ac_type.blocks_close())
        .map(|ac| ac.id.clone())
        .collect();
    if !unchecked_blocking.is_empty() && !args.force {
        let ids: Vec<&str> = unchecked_blocking.iter().map(|ac| ac.id.as_str()).collect();
        let gate_ids: Vec<&str> = unchecked_blocking
            .iter()
            .filter(|ac| ac.gate)
            .map(|ac| ac.id.as_str())
            .collect();
        let gate_suffix = if gate_ids.is_empty() {
            String::new()
        } else {
            format!(" (gate: {})", gate_ids.join(", "))
        };
        return Err(Error::conflict(
            VERB,
            format!(
                "{} unchecked blocking AC(s) in spec '{}': {}{}",
                ids.len(),
                spec.id,
                ids.join(", "),
                gate_suffix,
            ),
        ));
    }
    let forced_unchecked: Vec<String> = if args.force {
        unchecked_blocking.iter().map(|ac| ac.id.clone()).collect()
    } else {
        Vec::new()
    };

    // Memory-reconciliation gate (spec 2026-05-19-memory-gates-decisions
    // ac-04, D4). Match memories against `spec.goal + spec.cards` via the
    // same primitive as `memory.match`, filter to score >= MEMORY_MATCH_THRESHOLD,
    // and refuse closure when any matching memory key is absent from
    // `spec.memories_considered`. `--force` bypasses; bypassed keys land
    // in `forced_unreconciled` on the response.
    let memory_matches = memory_match(
        layout,
        &MemoryMatchArgs {
            topic: spec.goal.clone(),
            labels: spec.cards.clone(),
            limit: usize::MAX,
        },
    )
    .map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    let reconciled_keys: BTreeSet<&str> = spec
        .memories_considered
        .iter()
        .map(|r| r.key.as_str())
        .collect();
    let unreconciled: Vec<String> = memory_matches
        .matches
        .iter()
        .filter(|m| m.score >= MEMORY_MATCH_THRESHOLD)
        .filter(|m| !reconciled_keys.contains(m.memory.key.as_str()))
        .map(|m| m.memory.key.clone())
        .collect();
    if !unreconciled.is_empty() && !args.force {
        return Err(Error::conflict(
            VERB,
            format!(
                "{} unreconciled memor{} matching spec '{}': {} — reconcile via spec.memories_considered or pass --force",
                unreconciled.len(),
                if unreconciled.len() == 1 { "y" } else { "ies" },
                spec.id,
                unreconciled.join(", "),
            ),
        ));
    }
    let forced_unreconciled: Vec<String> = if args.force && !unreconciled.is_empty() {
        unreconciled
    } else {
        Vec::new()
    };

    // Per ac-06: spec.close requires every child task to be in state `done`.
    // Read the task stream once; reduce per task; reject if any non-done.
    let task_events = read_task_events(layout, &spec.id).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    let mut by_id: BTreeMap<String, Vec<TaskEvent>> = BTreeMap::new();
    for ev in task_events {
        by_id.entry(ev.task_id.clone()).or_default().push(ev);
    }
    let unfinished: Vec<String> = by_id
        .iter()
        .filter_map(|(id, evs)| {
            evs.last().and_then(|last| {
                if matches!(last.event, TaskEventKind::Done) {
                    None
                } else {
                    Some(id.clone())
                }
            })
        })
        .collect();
    if !unfinished.is_empty() {
        return Err(Error::conflict(
            VERB,
            format!(
                "{} unfinished task(s) under spec '{}': {}",
                unfinished.len(),
                spec.id,
                unfinished.join(", ")
            ),
        ));
    }

    // Reference inserted into each linked card's `specs` array. We use the
    // spec id with the `.orbit/specs/` prefix so the reference stays
    // stable regardless of where the workspace is rooted. Folder-shape
    // layout (choice 0021): `.orbit/specs/<id>/spec.yaml`.
    let spec_ref = format!(".orbit/specs/{}/spec.yaml", spec.id);

    // Phase 1: read every linked card and compute the proposed update.
    // We deliberately collect everything into memory before writing
    // ANYTHING, so a malformed card surfaces before any side effects.
    let mut card_updates: Vec<CardUpdate> = Vec::with_capacity(spec.cards.len());
    for card_slug in &spec.cards {
        validate_card_slug(VERB, card_slug)?;
        let card_path = layout.card_file(card_slug);
        if !card_path.exists() {
            return Err(Error::not_found(
                VERB,
                format!("linked card not found: {} ({})", card_slug, card_path.display()),
            ));
        }
        let pre_image = std::fs::read_to_string(&card_path)
            .map_err(|e| Error::unavailable(VERB, format!("read card {card_slug}: {e}")))?;
        let mut card: crate::schema::Card = parse_yaml(&pre_image).map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;
        // Idempotent: if the spec ref is already present, do nothing for
        // this card (helps if a previous spec.close partially completed).
        let needs_write = !card.specs.contains(&spec_ref);
        if needs_write {
            card.specs.push(spec_ref.clone());
        }
        let post_image = serialise_yaml(&card).map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;
        card_updates.push(CardUpdate {
            slug: card_slug.clone(),
            path: card_path,
            pre_image,
            post_image,
            written: false,
            needs_write,
        });
    }

    // Phase 2: write every card. On any failure, roll back the ones we
    // already wrote.
    for upd in card_updates.iter_mut() {
        if !upd.needs_write {
            continue;
        }
        if let Err(e) = write_atomic(&upd.path, upd.post_image.as_bytes()) {
            rollback_cards(&card_updates);
            let mut tagged = e;
            tagged.verb = VERB.into();
            return Err(tagged);
        }
        upd.written = true;
    }

    // Phase 3: write the closed spec. If this fails, roll back cards.
    spec.status = SpecStatus::Closed;
    let new_yaml = match serialise_yaml(&spec) {
        Ok(y) => y,
        Err(mut e) => {
            rollback_cards(&card_updates);
            e.verb = VERB.into();
            return Err(e);
        }
    };
    if let Err(e) = write_atomic(&spec_path, new_yaml.as_bytes()) {
        rollback_cards(&card_updates);
        let mut tagged = e;
        tagged.verb = VERB.into();
        return Err(tagged);
    }

    let cards_updated: Vec<String> = card_updates
        .iter()
        .filter(|u| u.needs_write)
        .map(|u| u.slug.clone())
        .collect();

    // Topology warnings surface (spec 2026-05-18-topology-substrate-wires
    // ac-03). Concatenate the spec's substantive sidecars
    // (spec.yaml + interview.md + tabletop-note.md, each when present),
    // and word-boundary-match each topology-doc subsystem name against
    // the concatenation. Subsystem names < 5 characters are excluded to
    // suppress false-positives on short common tokens. Names are passed
    // through regex::escape before \b...\b interpolation so
    // metacharacters (dots, hyphens, slashes) match literally. Best
    // effort: a malformed config or unreadable sidecar yields no
    // warnings rather than failing the close.
    let topology_warnings = compute_topology_warnings(layout, &spec.id);

    Ok(SpecCloseResult {
        spec,
        cards_updated,
        forced_unchecked,
        deferrable_open,
        forced_unreconciled,
        topology_warnings,
    })
}

/// Per ac-03: subsystem-name word-boundary scan across the spec's
/// substantive sidecars. Returns empty when the topology capability is
/// not configured or when no matches exist. Errors swallowed (this is
/// an advisory surface, not a blocking gate).
fn compute_topology_warnings(layout: &OrbitLayout, spec_id: &str) -> Vec<TopologyDriftEntry> {
    // Substrate-folder shape per choice 0025: load subsystem names directly
    // from `.orbit/topology/<subsystem>.yaml` entries. Files that fail to
    // parse or validate are dropped — this heuristic is advisory, not a
    // gate (audit_topology surfaces structural failures via the drift
    // envelope instead).
    let subsystems: Vec<String> = load_topology_entries(layout)
        .into_iter()
        .filter_map(|(_, result)| result.ok())
        .map(|entry| entry.subsystem)
        .collect();
    if subsystems.is_empty() {
        return Vec::new();
    }

    let spec_dir = layout.spec_dir(spec_id);
    let mut text = String::new();
    for sidecar in &["spec.yaml", "interview.md", "tabletop-note.md"] {
        let path = spec_dir.join(sidecar);
        if let Ok(body) = std::fs::read_to_string(&path) {
            text.push_str(&body);
            text.push('\n');
        }
    }
    if text.is_empty() {
        return Vec::new();
    }

    let mut out: Vec<TopologyDriftEntry> = Vec::new();
    for subsystem in subsystems {
        // Length filter — suppress false-positives on short common tokens
        // (memo, spec, ac, ...).
        if subsystem.chars().count() < 5 {
            continue;
        }
        // regex::escape before \b...\b interpolation — subsystem names
        // may contain metacharacters (dots, hyphens, slashes). Case-
        // insensitive via the (?i) inline flag.
        let pattern = format!(r"(?i)\b{}\b", regex::escape(&subsystem));
        let re = match regex::Regex::new(&pattern) {
            Ok(r) => r,
            Err(_) => continue,
        };
        if re.is_match(&text) {
            out.push(TopologyDriftEntry {
                subsystem,
                drift_kind: "spec_touch".into(),
                detail: String::new(),
            });
        }
    }
    out
}

/// In-memory record of one card's pre/post image during spec.close.
struct CardUpdate {
    slug: String,
    path: std::path::PathBuf,
    pre_image: String,
    post_image: String,
    written: bool,
    needs_write: bool,
}

/// Restore every card we'd already written back to its pre-image. Best-
/// effort — failures here are logged via the surface error but don't change
/// the outer return value.
fn rollback_cards(updates: &[CardUpdate]) {
    for upd in updates {
        if upd.written {
            // Best-effort restore. Failures here are logged via stderr
            // because they imply a partially-corrupted state we couldn't
            // fully clean up; the caller's error already names the
            // original failure.
            if let Err(e) = write_atomic(&upd.path, upd.pre_image.as_bytes()) {
                eprintln!(
                    "spec.close: rollback failed for card {}: {e} — manual recovery required",
                    upd.slug
                );
            }
        }
    }
}

/// Reject empty IDs, path traversal, and separators. Used by every verb
/// that takes a spec id.
fn validate_spec_id(verb: &str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(Error::malformed(verb, "id must not be empty"));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("id must not contain path separators or '..': '{}'", id),
        ));
    }
    Ok(())
}

/// Same protections for card slugs (cards live in `.orbit/cards/<slug>.yaml`).
fn validate_card_slug(verb: &str, slug: &str) -> Result<()> {
    if slug.is_empty() {
        return Err(Error::malformed(verb, "card slug must not be empty"));
    }
    if slug.contains('/') || slug.contains('\\') || slug.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("card slug must not contain path separators or '..': '{slug}'"),
        ));
    }
    Ok(())
}

/// Per choice 0022: cards and choices accept bare-NNNN as a CLI shorthand.
/// `8` and `0008` both resolve to the unique file in `dir` whose filename
/// starts with `0008-`. Returns `Ok(Some(slug))` on unique match, `Ok(None)`
/// when the query isn't bare-numeric (caller falls back to literal lookup),
/// and an error on zero or multiple matches.
fn resolve_numeric_slug(verb: &str, dir: &Path, query: &str) -> Result<Option<String>> {
    if query.is_empty() || !query.chars().all(|c| c.is_ascii_digit()) || query.len() > 4 {
        return Ok(None);
    }
    let n: u32 = query
        .parse()
        .map_err(|e| Error::malformed(verb, format!("parse `{query}`: {e}")))?;
    let padded = format!("{n:04}-");
    let mut matches: Vec<String> = Vec::new();
    let read = match std::fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return Ok(None),
    };
    for entry in read.flatten() {
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(s) => s,
            None => continue,
        };
        if !name.ends_with(".yaml") {
            continue;
        }
        if name.starts_with(&padded) {
            matches.push(name.trim_end_matches(".yaml").to_string());
        }
    }
    match matches.len() {
        0 => Err(Error::not_found(
            verb,
            format!("no entry matching `{padded}*` in {}", dir.display()),
        )),
        1 => Ok(Some(matches.pop().unwrap())),
        _ => {
            matches.sort();
            Err(Error::malformed(
                verb,
                format!(
                    "ambiguous: `{query}` matches {} entries: {}",
                    matches.len(),
                    matches.join(", ")
                ),
            ))
        }
    }
}

// ============================================================================
// Spec acceptance-criterion verbs (per spec 2026-05-24-port-acceptance-shim).
// Ported from the deprecated plugins/orb/scripts/orbit-acceptance.sh shim:
// `spec.acs`, `spec.next-ac`, `spec.blocking-gate`, `spec.has-unchecked` are
// read-only; `spec.check` / `spec.uncheck` flip an AC's `checked` flag with
// idempotency.
//
// Per ac-06: traversal helpers split along their two predicates and do NOT
// force-share with `spec_close`. Gate-axis helpers back `spec.next-ac` and
// `spec.blocking-gate`; the raw-axis helper backs `spec.has-unchecked`
// (drive's implement-loop termination). The taxonomy-axis pre-flight inside
// `spec_close` (over `ac_type.blocks_close()`) is unchanged.
// ============================================================================

/// Gate-axis: first unchecked AC not preceded by an unchecked gate. Returns
/// `None` when all checked or when a gate precedes every unchecked AC of a
/// different id. Per ac-06 of spec 2026-05-24-port-acceptance-shim.
pub(crate) fn next_unblocked(acs: &[AcceptanceCriterion]) -> Option<&AcceptanceCriterion> {
    let mut blocked = false;
    for ac in acs {
        if !ac.checked && !blocked {
            return Some(ac);
        }
        if ac.gate && !ac.checked {
            blocked = true;
        }
    }
    None
}

/// Gate-axis: first unchecked gate AC, if any. Per ac-06 of spec
/// 2026-05-24-port-acceptance-shim.
pub(crate) fn first_blocking_gate(acs: &[AcceptanceCriterion]) -> Option<&AcceptanceCriterion> {
    acs.iter().find(|ac| !ac.checked && ac.gate)
}

/// Raw-axis: true if any AC is unchecked. Distinct from `spec_close`'s
/// taxonomy-axis pre-flight (which filters on `ac_type.blocks_close()`).
/// Per ac-06 of spec 2026-05-24-port-acceptance-shim.
pub(crate) fn any_unchecked(acs: &[AcceptanceCriterion]) -> bool {
    acs.iter().any(|ac| !ac.checked)
}

/// `spec.acs` — return the full acceptance_criteria list for a spec.
fn spec_acs(layout: &OrbitLayout, args: &SpecAcsArgs) -> Result<SpecAcsResult> {
    const VERB: &str = "spec.acs";
    let spec = load_spec_for_ac_verb(layout, VERB, &args.id)?;
    Ok(SpecAcsResult {
        acs: spec.acceptance_criteria,
    })
}

/// `spec.next-ac` — first unchecked AC not blocked by an unchecked gate.
fn spec_next_ac(layout: &OrbitLayout, args: &SpecNextAcArgs) -> Result<SpecNextAcResult> {
    const VERB: &str = "spec.next-ac";
    let spec = load_spec_for_ac_verb(layout, VERB, &args.id)?;
    Ok(SpecNextAcResult {
        next: next_unblocked(&spec.acceptance_criteria).map(|ac| NextAc {
            id: ac.id.clone(),
            gate: ac.gate,
        }),
    })
}

/// `spec.blocking-gate` — first unchecked gate AC.
fn spec_blocking_gate(
    layout: &OrbitLayout,
    args: &SpecBlockingGateArgs,
) -> Result<SpecBlockingGateResult> {
    const VERB: &str = "spec.blocking-gate";
    let spec = load_spec_for_ac_verb(layout, VERB, &args.id)?;
    Ok(SpecBlockingGateResult {
        blocking: first_blocking_gate(&spec.acceptance_criteria).map(|ac| BlockingGate {
            id: ac.id.clone(),
            description: ac.description.clone(),
        }),
    })
}

/// `spec.has-unchecked` — true if any AC is unchecked.
fn spec_has_unchecked(
    layout: &OrbitLayout,
    args: &SpecHasUncheckedArgs,
) -> Result<SpecHasUncheckedResult> {
    const VERB: &str = "spec.has-unchecked";
    let spec = load_spec_for_ac_verb(layout, VERB, &args.id)?;
    Ok(SpecHasUncheckedResult {
        has_unchecked: any_unchecked(&spec.acceptance_criteria),
    })
}

/// `spec.check` — flip an AC's `checked` flag from false to true.
fn spec_check(layout: &OrbitLayout, args: &SpecCheckArgs) -> Result<SpecCheckResult> {
    const VERB: &str = "spec.check";
    let spec = flip_ac_checked(layout, VERB, &args.id, &args.ac_id, true)?;
    Ok(SpecCheckResult { spec })
}

/// `spec.uncheck` — flip an AC's `checked` flag from true to false.
fn spec_uncheck(layout: &OrbitLayout, args: &SpecUncheckArgs) -> Result<SpecUncheckResult> {
    const VERB: &str = "spec.uncheck";
    let spec = flip_ac_checked(layout, VERB, &args.id, &args.ac_id, false)?;
    Ok(SpecUncheckResult { spec })
}

/// Shared loader for the four read-only AC verbs — same id validation +
/// not-found shape as `spec.show`, returns the parsed Spec.
fn load_spec_for_ac_verb(layout: &OrbitLayout, verb: &'static str, id: &str) -> Result<Spec> {
    if id.is_empty() {
        return Err(Error::malformed(verb, "id must not be empty"));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("id must not contain path separators or '..': '{id}'"),
        ));
    }
    let path = layout.spec_file(id);
    if !path.exists() {
        return Err(Error::not_found(
            verb,
            format!("no spec at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::unavailable(verb, format!("read {}: {e}", path.display())))?;
    parse_yaml::<Spec>(&text).map_err(|mut e| {
        e.verb = verb.into();
        e
    })
}

/// Shared mutator for `spec.check` / `spec.uncheck` — acquires the spec
/// lock, reads, validates the AC exists, errors on already-in-target-state,
/// flips, writes atomically. Returns the post-write Spec.
fn flip_ac_checked(
    layout: &OrbitLayout,
    verb: &'static str,
    spec_id: &str,
    ac_id: &str,
    want_checked: bool,
) -> Result<Spec> {
    if spec_id.is_empty() {
        return Err(Error::malformed(verb, "id must not be empty"));
    }
    if spec_id.contains('/') || spec_id.contains('\\') || spec_id.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("id must not contain path separators or '..': '{spec_id}'"),
        ));
    }
    if ac_id.is_empty() {
        return Err(Error::malformed(verb, "ac_id must not be empty"));
    }

    let lock_key = format!("spec-{spec_id}");
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;

    let path = layout.spec_file(spec_id);
    if !path.exists() {
        return Err(Error::not_found(
            verb,
            format!("no spec at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::unavailable(verb, format!("read {}: {e}", path.display())))?;
    let mut spec: Spec = parse_yaml(&text).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;

    let pos = spec
        .acceptance_criteria
        .iter()
        .position(|ac| ac.id == ac_id)
        .ok_or_else(|| {
            Error::not_found(verb, format!("AC {ac_id} not found on spec {spec_id}"))
        })?;
    if spec.acceptance_criteria[pos].checked == want_checked {
        let state = if want_checked { "checked" } else { "unchecked" };
        return Err(Error::conflict(
            verb,
            format!("AC {ac_id} is already {state}"),
        ));
    }
    spec.acceptance_criteria[pos].checked = want_checked;

    let yaml = serialise_yaml(&spec).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;
    write_atomic(&path, yaml.as_bytes()).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;

    Ok(spec)
}

// ============================================================================
// Spec promote verb (per spec 2026-05-25-port-promote-sh).
// Port of plugins/orb/scripts/promote.sh — second opportunistic migration
// under choice 0020.
// ============================================================================

/// `spec.promote` — turn a card into a spec. Reads the card at
/// `args.card_path`, derives the spec id as
/// `<today-iso>-<slug-without-NNNN-prefix>`, creates the spec with the
/// card's `goal` and `cards: [<card.id>]`, populates `acceptance_criteria`
/// one entry per scenario (preserving `gate: bool`, seeding
/// `checked: false`), and writes through the canonical writer.
///
/// `args.dry_run: true` returns the planned spec without writing.
///
/// Error contract:
/// - `Error::not_found` — card path doesn't exist
/// - `Error::malformed` — card has no scenarios, empty goal, or path
///   contains `..` / escapes the layout root (canonicalise + containment)
/// - `Error::conflict` — non-dry-run: spec at derived id already exists
///   (dry-run path succeeds even when target exists; ac-04 contract)
fn spec_promote(layout: &OrbitLayout, args: &SpecPromoteArgs) -> Result<SpecPromoteResult> {
    const VERB: &str = "spec.promote";

    // 1. Resolve card path. Accept absolute or relative-to-project-root.
    //    `layout.root` is the `.orbit/` directory; the project root is its
    //    parent. Use canonicalise + containment so symlinks pointing
    //    outside the project root are also rejected (per spec ac-05).
    let project_root = layout.root.parent().ok_or_else(|| {
        Error::unavailable(
            VERB,
            format!("layout root has no parent: {}", layout.root.display()),
        )
    })?;
    let card_path_raw = std::path::PathBuf::from(&args.card_path);
    let card_path_abs = if card_path_raw.is_absolute() {
        card_path_raw
    } else {
        project_root.join(&card_path_raw)
    };
    if !card_path_abs.exists() {
        return Err(Error::not_found(
            VERB,
            format!("card not found: {}", card_path_abs.display()),
        ));
    }
    let card_path_canon = std::fs::canonicalize(&card_path_abs)
        .map_err(|e| Error::unavailable(VERB, format!("canonicalise {}: {e}", card_path_abs.display())))?;
    let root_canon = std::fs::canonicalize(project_root)
        .map_err(|e| Error::unavailable(VERB, format!("canonicalise project root: {e}")))?;
    if !card_path_canon.starts_with(&root_canon) {
        return Err(Error::malformed(
            VERB,
            format!(
                "card path resolves outside project root: {} (root: {})",
                card_path_canon.display(),
                root_canon.display(),
            ),
        ));
    }

    // 2. Parse the card.
    let card_text = std::fs::read_to_string(&card_path_canon)
        .map_err(|e| Error::unavailable(VERB, format!("read {}: {e}", card_path_canon.display())))?;
    let card: Card = parse_yaml(&card_text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    if card.goal.is_empty() {
        return Err(Error::malformed(
            VERB,
            format!("card has empty goal: {}", card_path_canon.display()),
        ));
    }
    if card.scenarios.is_empty() {
        return Err(Error::malformed(
            VERB,
            format!("card has no scenarios: {}", card_path_canon.display()),
        ));
    }

    // 3. Derive ids. card_id = filename minus .yaml. slug = card_id with
    //    leading NNNN- stripped. spec_id = <today-iso>-<slug>.
    let card_id = card_path_canon
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| Error::malformed(VERB, format!("card filename not utf-8: {}", card_path_canon.display())))?
        .to_string();
    let slug = card_id
        .strip_prefix(|c: char| c.is_ascii_digit())
        .and_then(|s| s.strip_prefix(|c: char| c.is_ascii_digit()))
        .and_then(|s| s.strip_prefix(|c: char| c.is_ascii_digit()))
        .and_then(|s| s.strip_prefix(|c: char| c.is_ascii_digit()))
        .and_then(|s| s.strip_prefix('-'))
        .unwrap_or(&card_id);
    let today = match &args.today {
        Some(t) => t.clone(),
        None => OffsetDateTime::now_utc()
            .date()
            .format(&time::format_description::well_known::Iso8601::DATE)
            .map_err(|e| Error::unavailable(VERB, format!("format today's date: {e}")))?,
    };
    let spec_id = format!("{today}-{slug}");

    // 4. Build the planned spec in memory.
    let acceptance_criteria: Vec<AcceptanceCriterion> = card
        .scenarios
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let name = s.name.trim();
            let then = s.then.trim();
            let description = if then.is_empty() {
                name.to_string()
            } else {
                format!("{name} — {then}")
            };
            AcceptanceCriterion {
                id: format!("ac-{:02}", i + 1),
                description,
                gate: s.gate,
                checked: false,
                verification: None,
                ac_type: AcType::default(),
            }
        })
        .collect();

    let planned = Spec {
        id: spec_id.clone(),
        goal: card.goal.clone(),
        cards: vec![card_id.clone()],
        status: SpecStatus::Open,
        labels: vec![],
        acceptance_criteria,
        memories_considered: vec![],
    };

    // 5. Dry-run short-circuits before any disk write — succeeds even when
    //    the target spec already exists (per ac-04).
    if args.dry_run {
        return Ok(SpecPromoteResult {
            spec: planned,
            dry_run: true,
        });
    }

    // 6. Pre-check: target must not already exist. Error verb-tagged as
    //    spec.promote (not spec.create) so consumers see the right surface
    //    per review-spec cycle 3 finding.
    let spec_path = layout.spec_file(&spec_id);
    if spec_path.exists() {
        return Err(Error::conflict(
            VERB,
            format!("spec '{spec_id}' already exists; promote produces fresh specs"),
        ));
    }

    // 7. Acquire spec lock + write.
    let lock_key = format!("spec-{spec_id}");
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    // Materialise the spec folder.
    let spec_dir = spec_path
        .parent()
        .expect("spec_file path always has a parent");
    std::fs::create_dir_all(spec_dir)
        .map_err(|e| Error::unavailable(VERB, format!("mkdir {}: {e}", spec_dir.display())))?;

    let yaml = serialise_yaml(&planned).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    write_atomic(&spec_path, yaml.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SpecPromoteResult {
        spec: planned,
        dry_run: false,
    })
}

// ============================================================================
// Task verbs (ac-07) — append-only JSONL events with last-event-wins state
// reduction. Per ac-07: "Tasks are append-only JSONL events. State =
// last event for that task_id."
// ============================================================================

/// `task.open` — append an Open event creating a new task. Generates a
/// task_id if the caller doesn't supply one.
fn task_open(layout: &OrbitLayout, args: &TaskOpenArgs) -> Result<TaskOpenResult> {
    const VERB: &str = "task.open";
    validate_spec_id(VERB, &args.spec_id)?;
    if args.body.is_empty() {
        return Err(Error::malformed(VERB, "body must not be empty"));
    }

    let spec_path = layout.spec_file(&args.spec_id);
    if !spec_path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no spec at {}", spec_path.display()),
        ));
    }

    let task_id = match &args.task_id {
        Some(id) => {
            validate_task_id(VERB, id)?;
            id.clone()
        }
        None => generate_task_id().map_err(|e| {
            Error::unavailable(VERB, format!("generate task_id: {e}"))
        })?,
    };
    let timestamp = stamp_or(VERB, &args.timestamp)?;

    // Conflict if a task with this id already has events. Reading events for
    // the spec is cheap; the JSONL file is small in v0.1.
    let existing = read_task_events(layout, &args.spec_id)?;
    if existing.iter().any(|e| e.task_id == task_id) {
        return Err(Error::conflict(
            VERB,
            format!("task '{task_id}' already exists in spec '{}'", args.spec_id),
        ));
    }

    let event = TaskEvent {
        task_id: task_id.clone(),
        spec_id: args.spec_id.clone(),
        event: TaskEventKind::Open,
        body: Some(args.body.clone()),
        labels: args.labels.clone(),
        timestamp,
    };
    append_task_event(VERB, layout, &args.spec_id, &event)?;
    Ok(TaskOpenResult { event, task_id })
}

/// `task.list` — current state per task, optionally filtered by state.
fn task_list(layout: &OrbitLayout, args: &TaskListArgs) -> Result<TaskListResult> {
    const VERB: &str = "task.list";
    if let Some(s) = args.state.as_deref() {
        if !matches!(s, "open" | "claim" | "update" | "done") {
            return Err(Error::malformed(
                VERB,
                format!("state must be one of open|claim|update|done, got '{s}'"),
            ));
        }
    }

    let states = collect_task_states(layout, args.spec_id.as_deref(), VERB)?;
    let filtered: Vec<TaskState> = states
        .into_iter()
        .filter(|s| match args.state.as_deref() {
            Some(want) => s.state == want,
            None => true,
        })
        .collect();
    Ok(TaskListResult { tasks: filtered })
}

/// `task.ready` — equivalent to `task.list --state open`.
fn task_ready(layout: &OrbitLayout, args: &TaskReadyArgs) -> Result<TaskListResult> {
    const VERB: &str = "task.ready";
    let states = collect_task_states(layout, args.spec_id.as_deref(), VERB)?;
    Ok(TaskListResult {
        tasks: states.into_iter().filter(|s| s.state == "open").collect(),
    })
}

/// `task.show` — full event history + reduced state for one task.
fn task_show(layout: &OrbitLayout, args: &TaskShowArgs) -> Result<TaskShowResult> {
    const VERB: &str = "task.show";
    validate_spec_id(VERB, &args.spec_id)?;
    validate_task_id(VERB, &args.task_id)?;

    let events = read_task_events(layout, &args.spec_id).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    let task_events: Vec<TaskEvent> = events
        .into_iter()
        .filter(|e| e.task_id == args.task_id)
        .collect();
    if task_events.is_empty() {
        return Err(Error::not_found(
            VERB,
            format!("no task '{}' in spec '{}'", args.task_id, args.spec_id),
        ));
    }
    let state = reduce_task_events(&task_events).expect("non-empty events have a last");
    Ok(TaskShowResult {
        state,
        events: task_events,
    })
}

fn task_claim(layout: &OrbitLayout, args: &TaskClaimArgs) -> Result<TaskEventResult> {
    append_task_lifecycle_event(
        "task.claim",
        layout,
        &args.spec_id,
        &args.task_id,
        TaskEventKind::Claim,
        args.body.clone(),
        args.labels.clone(),
        args.timestamp.clone(),
        |prev_state| {
            // Claim only legal from Open.
            if prev_state != "open" {
                return Err(Error::conflict(
                    "task.claim",
                    format!("task in state '{prev_state}' cannot be claimed; only 'open' tasks are claimable"),
                ));
            }
            Ok(())
        },
    )
}

fn task_update(layout: &OrbitLayout, args: &TaskUpdateArgs) -> Result<TaskEventResult> {
    if args.body.is_empty() {
        return Err(Error::malformed("task.update", "body must not be empty"));
    }
    append_task_lifecycle_event(
        "task.update",
        layout,
        &args.spec_id,
        &args.task_id,
        TaskEventKind::Update,
        Some(args.body.clone()),
        args.labels.clone(),
        args.timestamp.clone(),
        |prev_state| {
            if prev_state == "done" {
                return Err(Error::conflict(
                    "task.update",
                    "task already done; updates are not appended after done",
                ));
            }
            Ok(())
        },
    )
}

fn task_done(layout: &OrbitLayout, args: &TaskDoneArgs) -> Result<TaskEventResult> {
    append_task_lifecycle_event(
        "task.done",
        layout,
        &args.spec_id,
        &args.task_id,
        TaskEventKind::Done,
        args.body.clone(),
        args.labels.clone(),
        args.timestamp.clone(),
        |prev_state| {
            if prev_state == "done" {
                return Err(Error::conflict(
                    "task.done",
                    "task already done",
                ));
            }
            Ok(())
        },
    )
}

/// Shared lifecycle-event append for claim / update / done. Validates the
/// task exists, the prior state allows the transition (via `validate`), then
/// appends the event under the spec lock.
#[allow(clippy::too_many_arguments)]
fn append_task_lifecycle_event(
    verb: &'static str,
    layout: &OrbitLayout,
    spec_id: &str,
    task_id: &str,
    kind: TaskEventKind,
    body: Option<String>,
    labels: Vec<String>,
    timestamp_arg: Option<String>,
    validate: impl FnOnce(&str) -> Result<()>,
) -> Result<TaskEventResult> {
    validate_spec_id(verb, spec_id)?;
    validate_task_id(verb, task_id)?;

    let lock_key = format!("spec-{spec_id}");
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;

    let events = read_task_events(layout, spec_id).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;
    let task_events: Vec<&TaskEvent> = events.iter().filter(|e| e.task_id == task_id).collect();
    if task_events.is_empty() {
        return Err(Error::not_found(
            verb,
            format!("no task '{task_id}' in spec '{spec_id}'"),
        ));
    }
    let prev_state = task_event_kind_str(task_events.last().unwrap().event);
    validate(prev_state)?;

    let timestamp = stamp_or(verb, &timestamp_arg)?;
    let event = TaskEvent {
        task_id: task_id.into(),
        spec_id: spec_id.into(),
        event: kind,
        body,
        labels,
        timestamp,
    };
    append_task_event(verb, layout, spec_id, &event)?;
    Ok(TaskEventResult { event })
}

// --- Task helpers ----------------------------------------------------------

/// Read every event in `<spec_id>.tasks.jsonl` in order.
fn read_task_events(layout: &OrbitLayout, spec_id: &str) -> Result<Vec<TaskEvent>> {
    let path = layout.task_stream(spec_id);
    if !path.exists() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(&path).map_err(|e| {
        Error::unavailable("task.read", format!("read {}: {e}", path.display()))
    })?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let event: TaskEvent = parse_json_line(line).map_err(|mut e| {
            e.verb = "task.read".into();
            e.message = format!("{} (line {})", e.message, i + 1);
            e
        })?;
        out.push(event);
    }
    Ok(out)
}

/// Append a TaskEvent to `<spec_id>.tasks.jsonl`. Caller must hold the spec
/// lock for logical consistency; `append_jsonl_line` provides the byte-level
/// append atomicity.
fn append_task_event(
    verb: &'static str,
    layout: &OrbitLayout,
    spec_id: &str,
    event: &TaskEvent,
) -> Result<()> {
    let line = serialise_json_line(event).map_err(|mut e| {
        e.verb = verb.into();
        e
    })?;
    let path = layout.task_stream(spec_id);
    append_jsonl_line(&path, &line).map_err(|mut e| {
        e.verb = verb.into();
        e
    })
}

/// Reduce an ordered list of events for ONE task to its current state.
fn reduce_task_events(events: &[TaskEvent]) -> Option<TaskState> {
    let last = events.last()?;
    Some(TaskState {
        task_id: last.task_id.clone(),
        spec_id: last.spec_id.clone(),
        state: task_event_kind_str(last.event).into(),
        body: last.body.clone(),
        labels: last.labels.clone(),
        timestamp: last.timestamp.clone(),
        event_count: events.len(),
    })
}

/// Walk every (or one) spec's task stream and reduce each task to its
/// current state. Used by task.list and task.ready.
fn collect_task_states(
    layout: &OrbitLayout,
    spec_id: Option<&str>,
    verb: &'static str,
) -> Result<Vec<TaskState>> {
    let spec_files = match spec_id {
        Some(id) => {
            validate_spec_id(verb, id)?;
            let p = layout.spec_file(id);
            if !p.exists() {
                return Err(Error::not_found(
                    verb,
                    format!("no spec at {}", p.display()),
                ));
            }
            vec![id.to_string()]
        }
        None => {
            // List all spec files; derive ids from their parent folder names
            // — list_spec_files returns `<id>/spec.yaml` paths under choice
            // 0021's folder layout.
            let files = layout
                .list_spec_files()
                .map_err(|e| Error::unavailable(verb, format!("list specs: {e}")))?;
            files
                .iter()
                .filter_map(|p| {
                    p.parent()
                        .and_then(|d| d.file_name())
                        .and_then(|s| s.to_str())
                        .map(String::from)
                })
                .collect()
        }
    };

    let mut all_states = Vec::new();
    for spec_id in spec_files {
        let events = read_task_events(layout, &spec_id).map_err(|mut e| {
            e.verb = verb.into();
            e
        })?;
        // Group events by task_id, preserving order via BTreeMap (deterministic).
        let mut by_id: BTreeMap<String, Vec<TaskEvent>> = BTreeMap::new();
        for ev in events {
            by_id.entry(ev.task_id.clone()).or_default().push(ev);
        }
        for (_, evs) in by_id {
            if let Some(s) = reduce_task_events(&evs) {
                all_states.push(s);
            }
        }
    }

    // Sort for deterministic output.
    all_states.sort_by(|a, b| a.spec_id.cmp(&b.spec_id).then(a.task_id.cmp(&b.task_id)));
    Ok(all_states)
}

fn task_event_kind_str(kind: TaskEventKind) -> &'static str {
    match kind {
        TaskEventKind::Open => "open",
        TaskEventKind::Claim => "claim",
        TaskEventKind::Update => "update",
        TaskEventKind::Done => "done",
    }
}

fn validate_task_id(verb: &str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(Error::malformed(verb, "task_id must not be empty"));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("task_id must not contain path separators or '..': '{id}'"),
        ));
    }
    Ok(())
}

/// Generate a task_id of the shape `t-<8hex><8hex>` using process pid + nanos.
/// Deterministic per process+time, human-readable, no new deps. Collision
/// risk within a single process is bounded by clock resolution; v0.1's
/// single-machine constraint makes this safe.
fn generate_task_id() -> std::result::Result<String, std::time::SystemTimeError> {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_nanos();
    Ok(format!("t-{pid:08x}{nanos:016x}"))
}

/// Use the supplied timestamp if any; otherwise stamp with substrate clock.
fn stamp_or(verb: &str, supplied: &Option<String>) -> Result<String> {
    match supplied {
        Some(t) => Ok(t.clone()),
        None => current_rfc3339_utc()
            .map_err(|e| Error::unavailable(verb, format!("substrate timestamp: {e}"))),
    }
}

// ============================================================================
// Memory verbs (ac-08) — substrate-written entities; cross-session/cross-machine via git.
// ============================================================================

/// Canonical topology-label nudge text — emitted on the
/// `MemoryRememberResult.nudge` field when the stored memory carries
/// the `topology` label and `--no-nudge` is not set. Per spec
/// 2026-05-18-topology-substrate-wires ac-04.
pub const TOPOLOGY_NUDGE: &str = "consider /orb:topology — labelled memories often correspond to subsystems that should be added or updated in the topology doc";

/// Canonical state-shape warning text — emitted on the
/// `MemoryRememberResult.shape_warning` field when the stored memory's
/// body leads with a state observation rather than a mechanism clause
/// and `--no-warn` is not set. Per spec
/// 2026-05-19-memory-gates-decisions ac-05 (D5b).
pub const MEMORY_SHAPE_WARNING: &str = "memory body leads with state ('X is …'); decision-moment surfacing works better when the body leads with mechanism ('use X for Y', 'prefer X when Y'). Consider rephrasing — the memory is stored as written.";

/// Threshold below which a `memory.match` result is not considered
/// "matching" for the close-time reconciliation gate. Tunable in one
/// place; substrate-wide policy, not per-spec. Per spec
/// 2026-05-19-memory-gates-decisions ac-04 (D4).
pub const MEMORY_MATCH_THRESHOLD: f32 = 0.3;

/// Detect a state-shape opening on the body's first sentence — leading
/// state-verb patterns that decision-moment surfacing can't act on.
/// Returns true when the body opens with a state observation
/// ("X is …", "the problem is …", "Y proved difficult"), false
/// otherwise. Case-insensitive on the leading token; only scans the
/// first sentence so longer bodies with mechanism in the second clause
/// avoid the warning when the first clause is already mechanism-shaped.
///
/// Conservative by design — false positives are the failure mode the
/// D5b decision specifically rejected (vs D5a's strict block). The
/// heuristic only fires on patterns that lead a sentence with a
/// state-verb form. Per spec 2026-05-19-memory-gates-decisions ac-05
/// (D5b).
fn detect_state_shape(body: &str) -> bool {
    // Take the first sentence — split on '.', '!', or '\n'.
    let first = body
        .split(|c: char| c == '.' || c == '!' || c == '\n')
        .next()
        .unwrap_or("")
        .trim();
    if first.is_empty() {
        return false;
    }
    let lower = first.to_lowercase();
    // Phrase-shape patterns. Each matches a leading clause that frames
    // a state observation. The list is intentionally short — every
    // entry is a documented anti-pattern from the spec's D5 discussion.
    const PHRASE_STARTS: &[&str] = &[
        "the problem is ",
        "the issue is ",
        "the trick is ",
        "the catch is ",
    ];
    for p in PHRASE_STARTS {
        if lower.starts_with(p) {
            return true;
        }
    }
    // "X proved <adjective>" / "X turned out <adjective>".
    if lower.contains(" proved ") || lower.contains(" turned out ") {
        return true;
    }
    // "X is hard / brittle / fragile / slow / difficult / tricky / painful / annoying"
    // — state-verb + negative-quality adjective. Restricted to a small
    // adjective set so legitimate mechanism phrasings like
    // "FineType is uv-based" do NOT fire.
    const NEGATIVE_QUALITY_ADJS: &[&str] = &[
        " is hard",
        " is brittle",
        " is fragile",
        " is slow",
        " is difficult",
        " is tricky",
        " is painful",
        " is annoying",
        " is flaky",
        " is broken",
        " is messy",
    ];
    for a in NEGATIVE_QUALITY_ADJS {
        if lower.contains(a) {
            return true;
        }
    }
    false
}

fn memory_remember(layout: &OrbitLayout, args: &MemoryRememberArgs) -> Result<MemoryRememberResult> {
    const VERB: &str = "memory.remember";
    validate_memory_key(VERB, &args.key)?;
    if args.body.is_empty() {
        return Err(Error::malformed(VERB, "body must not be empty"));
    }

    let lock_key = format!("memory-{}", args.key);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    layout
        .ensure_dirs()
        .map_err(|e| Error::unavailable(VERB, format!("ensure dirs: {e}")))?;

    let timestamp = stamp_or(VERB, &args.timestamp)?;
    let memory = Memory {
        key: args.key.clone(),
        body: args.body.clone(),
        timestamp,
        labels: args.labels.clone(),
    };
    let yaml = serialise_yaml(&memory).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    write_atomic(layout.memory_file(&args.key), yaml.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    // Topology-label nudge (ac-04). Fires only when the labels list
    // contains the canonical `topology` label AND the caller did not
    // pass `--no-nudge`. Non-blocking — the memory has already stored.
    let nudge = if !args.no_nudge && args.labels.iter().any(|l| l == "topology") {
        Some(TOPOLOGY_NUDGE.to_string())
    } else {
        None
    };

    // State-shape warning (spec 2026-05-19-memory-gates-decisions ac-05,
    // D5b). Fires when the body's first sentence reads as a state
    // observation rather than a mechanism clause AND the caller did not
    // pass `--no-warn`. Non-blocking — the memory has already stored.
    // Mirrors the topology-nudge pattern.
    let shape_warning = if !args.no_warn && detect_state_shape(&args.body) {
        Some(MEMORY_SHAPE_WARNING.to_string())
    } else {
        None
    };

    Ok(MemoryRememberResult {
        memory,
        nudge,
        shape_warning,
    })
}

fn memory_list(layout: &OrbitLayout, _args: &MemoryListArgs) -> Result<MemoryListResult> {
    const VERB: &str = "memory.list";
    Ok(MemoryListResult {
        memories: read_all_memories(layout, VERB)?,
    })
}

fn memory_search(layout: &OrbitLayout, args: &MemorySearchArgs) -> Result<MemoryListResult> {
    const VERB: &str = "memory.search";
    if args.query.is_empty() {
        return Err(Error::malformed(VERB, "query must not be empty"));
    }
    let needle = args.query.to_lowercase();
    let all = read_all_memories(layout, VERB)?;
    let matched: Vec<Memory> = all
        .into_iter()
        .filter(|m| {
            m.body.to_lowercase().contains(&needle)
                || m.labels.iter().any(|l| l.to_lowercase().contains(&needle))
        })
        .collect();
    Ok(MemoryListResult { memories: matched })
}

/// `memory.match` — rank memories by relevance to a decision-moment topic.
///
/// Distinct semantic from `memory.search` (operator-keyword substring): the
/// caller passes a `topic` (free text — a card slug, spec goal, or
/// approach snippet) and optional `labels`; the v1 ranker scores each
/// memory by `token-overlap(body) + 2 * label-overlap(labels)`, normalised
/// to 0.0..=1.0. Results are sorted by score descending and truncated to
/// `limit` (default 10).
///
/// Per spec 2026-05-19-memory-gates-decisions ac-01/ac-02 (D1). Read by
/// `spec.close` to compute the close-time reconciliation gate (D4).
fn memory_match(layout: &OrbitLayout, args: &MemoryMatchArgs) -> Result<MemoryMatchResult> {
    const VERB: &str = "memory.match";
    if args.topic.is_empty() && args.labels.is_empty() {
        return Err(Error::malformed(
            VERB,
            "at least one of topic or labels must be non-empty",
        ));
    }
    let all = read_all_memories(layout, VERB)?;
    let topic_tokens = tokenise(&args.topic);
    let label_set: BTreeSet<String> = args
        .labels
        .iter()
        .map(|l| l.to_lowercase())
        .collect();

    let mut scored: Vec<MemoryMatch> = Vec::new();
    for m in all {
        let body_tokens = tokenise(&m.body);
        let body_overlap = if topic_tokens.is_empty() || body_tokens.is_empty() {
            0
        } else {
            topic_tokens
                .iter()
                .filter(|t| body_tokens.contains(*t))
                .count()
        };
        let mem_labels: BTreeSet<String> = m.labels.iter().map(|l| l.to_lowercase()).collect();
        let label_overlap = label_set.intersection(&mem_labels).count();

        // Normaliser caps the maximum possible signal at 1.0:
        //   body component normalised by min(topic_tokens, body_tokens) — overlap
        //     up to the shorter side counts as full body coverage.
        //   label component normalised by min(label_set, mem_labels) — same
        //     rationale.
        // Score = 0.5 * body_norm + 1.0 * label_norm, clamped to [0, 1].
        // Coefficients chosen so a perfect label match alone reaches the
        // D4 threshold (0.3) and a perfect body match alone reaches half
        // the label weighting — matching D1's "2x label vs body" ratio.
        let body_norm_denom = topic_tokens.len().min(body_tokens.len()).max(1);
        let body_norm = body_overlap as f32 / body_norm_denom as f32;
        let label_norm_denom = label_set.len().min(mem_labels.len()).max(1);
        let label_norm = if label_set.is_empty() || mem_labels.is_empty() {
            0.0
        } else {
            label_overlap as f32 / label_norm_denom as f32
        };
        let score = (0.5 * body_norm + 1.0 * label_norm).min(1.0);

        if score <= 0.0 {
            continue;
        }

        let reason = if label_overlap > 0 && body_overlap > 0 {
            format!(
                "label overlap ({label_overlap}) + body token overlap ({body_overlap})"
            )
        } else if label_overlap > 0 {
            format!("label overlap ({label_overlap})")
        } else {
            format!("body token overlap ({body_overlap})")
        };

        scored.push(MemoryMatch {
            memory: m,
            score,
            reason,
        });
    }

    // Sort score DESC, tie-break by key for determinism.
    scored.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.memory.key.cmp(&b.memory.key))
    });
    scored.truncate(args.limit);

    Ok(MemoryMatchResult { matches: scored })
}

/// Simple word tokeniser for the v1 `memory.match` ranker. Lower-cases,
/// splits on non-alphanumeric, drops single-character tokens and a small
/// stop-list of common-but-uninformative words. Returns a deduplicated
/// set so repeated tokens don't double-count.
fn tokenise(text: &str) -> BTreeSet<String> {
    const STOP: &[&str] = &[
        "a", "an", "the", "and", "or", "but", "of", "for", "to", "in", "on", "at", "by",
        "is", "are", "was", "were", "be", "been", "being", "as", "with", "from", "this",
        "that", "these", "those", "it", "its", "if", "so", "we", "you", "i", "they", "he",
        "she", "do", "does", "did", "not", "no", "yes", "than", "then", "via", "per",
    ];
    let stop: BTreeSet<&str> = STOP.iter().copied().collect();
    text.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() > 1 && !stop.contains(t))
        .map(|t| t.to_string())
        .collect()
}

fn read_all_memories(layout: &OrbitLayout, verb: &'static str) -> Result<Vec<Memory>> {
    let files = layout
        .list_memory_files()
        .map_err(|e| Error::unavailable(verb, format!("list memories: {e}")))?;
    let mut out = Vec::with_capacity(files.len());
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::unavailable(verb, format!("read {}: {e}", path.display())))?;
        let m: Memory = parse_yaml(&text).map_err(|mut e| {
            e.verb = verb.into();
            e
        })?;
        out.push(m);
    }
    out.sort_by(|a, b| a.key.cmp(&b.key));
    Ok(out)
}

fn validate_memory_key(verb: &str, key: &str) -> Result<()> {
    if key.is_empty() {
        return Err(Error::malformed(verb, "key must not be empty"));
    }
    if key.contains('/') || key.contains('\\') || key.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("key must not contain path separators or '..': '{key}'"),
        ));
    }
    Ok(())
}

// ============================================================================
// Card verbs (ac-09) — read-only; the only substrate-driven card write is
// the `specs` array append from spec.close, handled there.
// ============================================================================

fn card_show(layout: &OrbitLayout, args: &CardShowArgs) -> Result<CardShowResult> {
    const VERB: &str = "card.show";
    validate_card_slug(VERB, &args.slug)?;
    let resolved = resolve_numeric_slug(VERB, &layout.cards_dir(), &args.slug)?
        .unwrap_or_else(|| args.slug.clone());
    let path = layout.card_file(&resolved);
    if !path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no card at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::unavailable(VERB, format!("read {}: {e}", path.display())))?;
    let card: Card = parse_yaml(&text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    Ok(CardShowResult {
        slug: resolved,
        card,
    })
}

fn card_list(layout: &OrbitLayout, args: &CardListArgs) -> Result<CardListResult> {
    const VERB: &str = "card.list";
    if let Some(m) = args.maturity.as_deref() {
        if !matches!(m, "planned" | "emerging" | "established") {
            return Err(Error::malformed(
                VERB,
                format!("maturity must be planned|emerging|established, got '{m}'"),
            ));
        }
    }
    let summaries = collect_card_summaries(layout, VERB)?;
    let filtered = match &args.maturity {
        Some(m) => summaries.into_iter().filter(|s| s.maturity == *m).collect(),
        None => summaries,
    };
    Ok(CardListResult { cards: filtered })
}

fn card_search(layout: &OrbitLayout, args: &CardSearchArgs) -> Result<CardListResult> {
    const VERB: &str = "card.search";
    if args.query.is_empty() {
        return Err(Error::malformed(VERB, "query must not be empty"));
    }
    let needle = args.query.to_lowercase();
    let summaries = collect_card_summaries(layout, VERB)?;
    let matched: Vec<CardSummary> = summaries
        .into_iter()
        .filter(|s| {
            s.feature.to_lowercase().contains(&needle)
                || s.goal.to_lowercase().contains(&needle)
                || s.slug.to_lowercase().contains(&needle)
        })
        .collect();
    Ok(CardListResult { cards: matched })
}

fn card_tree(layout: &OrbitLayout, args: &CardTreeArgs) -> Result<CardTreeResult> {
    const VERB: &str = "card.tree";
    validate_card_slug(VERB, &args.slug)?;

    // Resolve the root slug — same prefix-match semantics as card.show.
    let resolved = resolve_numeric_slug(VERB, &layout.cards_dir(), &args.slug)?
        .unwrap_or_else(|| args.slug.clone());
    let root_path = layout.card_file(&resolved);
    if !root_path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no card at {}", root_path.display()),
        ));
    }

    // Load every card once into a slug→Card map, then build forward and
    // reverse edge indexes. Walking cards once keeps the cost linear in
    // card count regardless of tree depth.
    let cards = load_all_cards(layout, VERB)?;
    if !cards.contains_key(&resolved) {
        // Path existed but parse failed earlier — shouldn't happen, but
        // guard against silent divergence between fs and parsed view.
        return Err(Error::not_found(
            VERB,
            format!("card {resolved} not present in loaded card set"),
        ));
    }

    let forward = build_forward_edges(&cards);
    let reverse = build_reverse_edges(&cards);

    let mut visited = std::collections::HashSet::new();
    let tree = expand_card_node(&resolved, &cards, &forward, &reverse, args.depth, &mut visited);

    Ok(CardTreeResult {
        root: resolved,
        depth: args.depth,
        tree,
    })
}

/// Load every card under `.orbit/cards/` into a `slug -> Card` map. Used by
/// `card.tree` to build forward and reverse edge indexes in one pass.
fn load_all_cards(
    layout: &OrbitLayout,
    verb: &'static str,
) -> Result<BTreeMap<String, Card>> {
    let files = layout
        .list_card_files()
        .map_err(|e| Error::unavailable(verb, format!("list cards: {e}")))?;
    let mut out = BTreeMap::new();
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::unavailable(verb, format!("read {}: {e}", path.display())))?;
        let card: Card = parse_yaml(&text).map_err(|mut e| {
            e.verb = verb.into();
            e
        })?;
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                Error::malformed(verb, format!("card path has no stem: {}", path.display()))
            })?
            .to_string();
        out.insert(slug, card);
    }
    Ok(out)
}

/// Forward edges per card: `slug -> Vec<(target_slug, kind, reason)>`.
fn build_forward_edges(
    cards: &BTreeMap<String, Card>,
) -> BTreeMap<String, Vec<(String, String, String)>> {
    let mut out: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
    for (slug, card) in cards {
        let edges = card
            .relations
            .iter()
            .map(|r| (r.card.clone(), relation_kind_str(&r.kind).into(), r.reason.clone()))
            .collect();
        out.insert(slug.clone(), edges);
    }
    out
}

/// Reverse edges: for each target slug, the list of (source_slug, kind,
/// reason) pointing to it. Resolved against the card set's slug
/// vocabulary; edges to unknown slugs are kept verbatim so the tree
/// surfaces dangling references rather than silently dropping them.
fn build_reverse_edges(
    cards: &BTreeMap<String, Card>,
) -> BTreeMap<String, Vec<(String, String, String)>> {
    let mut out: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
    for (source, card) in cards {
        for relation in &card.relations {
            out.entry(relation.card.clone()).or_default().push((
                source.clone(),
                relation_kind_str(&relation.kind).into(),
                relation.reason.clone(),
            ));
        }
    }
    out
}

fn relation_kind_str(kind: &crate::schema::RelationKind) -> &'static str {
    use crate::schema::RelationKind;
    match kind {
        RelationKind::DependsOn => "depends-on",
        RelationKind::Feeds => "feeds",
        RelationKind::Supersedes => "supersedes",
        RelationKind::SupersededBy => "superseded-by",
    }
}

/// Recursively expand a node up to `depth` hops. Cycle-safe: re-visiting a
/// slug already on the current expansion path produces a truncated leaf.
fn expand_card_node(
    slug: &str,
    cards: &BTreeMap<String, Card>,
    forward: &BTreeMap<String, Vec<(String, String, String)>>,
    reverse: &BTreeMap<String, Vec<(String, String, String)>>,
    depth: u32,
    visited: &mut std::collections::HashSet<String>,
) -> CardTreeNode {
    let feature = cards
        .get(slug)
        .map(|c| c.feature.clone())
        .unwrap_or_default();

    // Already visited → return a truncated leaf to break the cycle without
    // duplicating downstream edges. The caller still sees the slug and
    // feature; the structure is bounded.
    if visited.contains(slug) {
        return CardTreeNode {
            slug: slug.to_string(),
            feature,
            outgoing: Vec::new(),
            incoming: Vec::new(),
            truncated: true,
        };
    }
    // Depth boundary → leaf node with the slug only, no edges expanded.
    if depth == 0 {
        return CardTreeNode {
            slug: slug.to_string(),
            feature,
            outgoing: Vec::new(),
            incoming: Vec::new(),
            truncated: true,
        };
    }

    visited.insert(slug.to_string());

    let outgoing = forward
        .get(slug)
        .map(|edges| {
            edges
                .iter()
                .map(|(target_slug, kind, reason)| CardTreeEdge {
                    kind: kind.clone(),
                    reason: reason.clone(),
                    target: expand_card_node(target_slug, cards, forward, reverse, depth - 1, visited),
                })
                .collect()
        })
        .unwrap_or_default();

    let incoming = reverse
        .get(slug)
        .map(|edges| {
            edges
                .iter()
                .map(|(source_slug, kind, reason)| CardTreeEdge {
                    kind: kind.clone(),
                    reason: reason.clone(),
                    target: expand_card_node(source_slug, cards, forward, reverse, depth - 1, visited),
                })
                .collect()
        })
        .unwrap_or_default();

    visited.remove(slug);

    CardTreeNode {
        slug: slug.to_string(),
        feature,
        outgoing,
        incoming,
        truncated: false,
    }
}

fn card_specs(layout: &OrbitLayout, args: &CardSpecsArgs) -> Result<CardSpecsResult> {
    const VERB: &str = "card.specs";
    validate_card_slug(VERB, &args.slug)?;
    let resolved = resolve_numeric_slug(VERB, &layout.cards_dir(), &args.slug)?
        .unwrap_or_else(|| args.slug.clone());
    let card_path = layout.card_file(&resolved);
    if !card_path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no card at {}", card_path.display()),
        ));
    }
    let card_text = std::fs::read_to_string(&card_path)
        .map_err(|e| Error::unavailable(VERB, format!("read {}: {e}", card_path.display())))?;
    let card: Card = parse_yaml(&card_text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    // Map of all known specs on disk: id -> (path, cards array, status).
    // Built once; consulted for forward dereferences and the reverse scan.
    let spec_files = layout
        .list_spec_files()
        .map_err(|e| Error::unavailable(VERB, format!("list specs: {e}")))?;
    let mut specs_on_disk: BTreeMap<String, (String, Vec<String>, String, bool)> = BTreeMap::new();
    for path in spec_files {
        // Per choice 0021 the per-spec folder is `<id>/spec.yaml`. The spec
        // id is the parent directory name.
        let spec_id = path
            .parent()
            .and_then(|p| p.file_name())
            .and_then(|s| s.to_str())
            .ok_or_else(|| {
                Error::malformed(
                    VERB,
                    format!("spec path has no parent folder name: {}", path.display()),
                )
            })?
            .to_string();
        let path_str = relativise_spec_path(&path, &&layout.root);
        match std::fs::read_to_string(&path)
            .map_err(|e| (e, "read".to_string()))
            .and_then(|t| parse_yaml::<Spec>(&t).map_err(|e| (std::io::Error::other(e.to_string()), "parse".to_string())))
        {
            Ok(spec) => {
                let status = match spec.status {
                    SpecStatus::Open => "open",
                    SpecStatus::Closed => "closed",
                };
                specs_on_disk.insert(spec_id, (path_str, spec.cards, status.to_string(), true));
            }
            Err((_, stage)) => {
                let status = if stage == "read" { "missing" } else { "parse-failed" };
                specs_on_disk.insert(spec_id, (path_str, Vec::new(), status.to_string(), false));
            }
        }
    }

    let mut entries: BTreeMap<String, CardSpecsEntry> = BTreeMap::new();

    // Forward direction: every path the card lists in card.specs[].
    for listed_path in &card.specs {
        let spec_id = spec_id_from_listed_path(listed_path);
        let (path_for_entry, back_ref, status) = match specs_on_disk.get(&spec_id) {
            Some((path, cards, status, parsed)) => {
                let back = *parsed && cards.iter().any(|c| c == &resolved);
                (path.clone(), back, status.clone())
            }
            None => (listed_path.clone(), false, "missing".to_string()),
        };
        entries.insert(
            spec_id.clone(),
            CardSpecsEntry {
                spec_id,
                spec_path: path_for_entry,
                listed_on_card: true,
                back_referenced_by_spec: back_ref,
                status,
            },
        );
    }

    // Reverse direction: every on-disk spec whose cards[] references this
    // card but which isn't already in the entries map (or which is, but with
    // listed_on_card=true already — we only need to upsert the back-ref
    // flag).
    for (spec_id, (path, cards, status, parsed)) in &specs_on_disk {
        if !*parsed {
            continue;
        }
        if cards.iter().any(|c| c == &resolved) {
            entries
                .entry(spec_id.clone())
                .and_modify(|e| {
                    e.back_referenced_by_spec = true;
                })
                .or_insert_with(|| CardSpecsEntry {
                    spec_id: spec_id.clone(),
                    spec_path: path.clone(),
                    listed_on_card: false,
                    back_referenced_by_spec: true,
                    status: status.clone(),
                });
        }
    }

    Ok(CardSpecsResult {
        root: resolved,
        specs: entries.into_values().collect(),
    })
}

/// Render a spec path as a relative `.orbit/specs/<id>/spec.yaml` string for
/// display alongside the human-written form in `card.specs[]`. Falls back to
/// the absolute path if it can't be relativised (e.g. the layout root isn't
/// a prefix — only happens in tests with unusual fixtures).
fn relativise_spec_path(path: &Path, orbit_root: &Path) -> String {
    // orbit_root is the `.orbit/` dir; we want output prefixed `.orbit/...`
    let parent = orbit_root.parent().unwrap_or(orbit_root);
    if let Ok(rel) = path.strip_prefix(parent) {
        return rel.to_string_lossy().into_owned();
    }
    path.to_string_lossy().into_owned()
}

fn overview(layout: &OrbitLayout, args: &OverviewArgs) -> Result<OverviewResult> {
    const VERB: &str = "overview";
    const DEFAULT_CAP: usize = 10;
    let cap = args.memory_cap.unwrap_or(DEFAULT_CAP);

    // Open specs — reuse spec.list, then filter and cap.
    let SpecListResult { specs: all_specs } = spec_list(layout, &SpecListArgs::default())
        .map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;
    let mut open_ids: Vec<String> = all_specs
        .into_iter()
        .filter(|s| s.status == "open")
        .map(|s| s.id)
        .collect();
    open_ids.sort(); // chronological since ids are date-prefixed
    let open_spec_count = open_ids.len();
    let recent_open_spec_ids: Vec<String> = if open_spec_count > cap {
        open_ids.into_iter().rev().take(cap).collect::<Vec<_>>().into_iter().rev().collect()
    } else {
        open_ids
    };
    let spec_overflow = open_spec_count.saturating_sub(recent_open_spec_ids.len());

    // Cards — single pass for maturity counts + degree + orphan detection.
    let cards = load_all_cards(layout, VERB)?;
    let mut maturity = CardMaturityCounts {
        planned: 0,
        emerging: 0,
        established: 0,
    };
    for card in cards.values() {
        match card.maturity {
            crate::schema::CardMaturity::Planned => maturity.planned += 1,
            crate::schema::CardMaturity::Emerging => maturity.emerging += 1,
            crate::schema::CardMaturity::Established => maturity.established += 1,
        }
    }

    // Reverse-edge index — for both "most-connected" (incoming edges count
    // toward degree) and "orphans" (zero incoming relations).
    let reverse = build_reverse_edges(&cards);

    let mut most_connected: Option<MostConnectedCard> = None;
    let mut best_degree: usize = 0;
    for (slug, card) in &cards {
        let outgoing = card.relations.len();
        let incoming = reverse.get(slug).map(Vec::len).unwrap_or(0);
        let degree = outgoing + incoming;
        if degree == 0 {
            continue;
        }
        // New leader if strictly greater, or equal with a lower numeric id.
        let take = degree > best_degree
            || (degree == best_degree
                && most_connected
                    .as_ref()
                    .is_some_and(|c| numeric_prefix(slug) < numeric_prefix(&c.slug)));
        if take || most_connected.is_none() {
            most_connected = Some(MostConnectedCard {
                slug: slug.clone(),
                feature: card.feature.clone(),
                degree,
            });
            best_degree = degree;
        }
    }

    // Orphans — cards with specs: [] AND no incoming relations from other
    // cards. BTreeMap iteration is sorted, so output is deterministic.
    let mut orphans_all: Vec<String> = cards
        .iter()
        .filter(|(slug, card)| {
            card.specs.is_empty() && reverse.get(*slug).map_or(true, Vec::is_empty)
        })
        .map(|(slug, _)| slug.clone())
        .collect();
    let orphan_total = orphans_all.len();
    let orphan_overflow = orphan_total.saturating_sub(cap);
    orphans_all.truncate(cap);

    // Memories — same shape as session.prime: by timestamp DESC, capped.
    let mut memories = read_all_memories(layout, VERB)?;
    memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    memories.truncate(cap);

    Ok(OverviewResult {
        open_spec_count,
        recent_open_spec_ids,
        spec_overflow,
        cards_by_maturity: maturity,
        memories,
        most_connected_card: most_connected,
        orphans: orphans_all,
        orphan_overflow,
    })
}

fn audit_drift(layout: &OrbitLayout, _args: &AuditDriftArgs) -> Result<AuditDriftResult> {
    const VERB: &str = "audit.drift";
    const DEFAULT_DISPOSITION: &str = "quarantine";

    let mut drift: Vec<DriftEntry> = Vec::new();

    // Helper: scan one file as untyped YAML, diff its top-level keys
    // against the known field set. parse-failed files surface as a single
    // drift entry with a special field name so callers see the file at all.
    let scan = |path: &Path, kind: &str, known: &[&str], out: &mut Vec<DriftEntry>| -> Result<()> {
        let display_path = relativise_spec_path(path, &&layout.root);
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(e) => {
                return Err(Error::unavailable(
                    VERB,
                    format!("read {}: {e}", path.display()),
                ));
            }
        };
        let value: serde_yaml::Value = match serde_yaml::from_str(&text) {
            Ok(v) => v,
            Err(e) => {
                out.push(DriftEntry {
                    path: display_path,
                    kind: kind.to_string(),
                    field: format!("<parse-failed: {e}>"),
                    disposition: DEFAULT_DISPOSITION.to_string(),
                });
                return Ok(());
            }
        };
        let mapping = match value.as_mapping() {
            Some(m) => m,
            None => {
                out.push(DriftEntry {
                    path: display_path,
                    kind: kind.to_string(),
                    field: "<root-not-mapping>".into(),
                    disposition: DEFAULT_DISPOSITION.to_string(),
                });
                return Ok(());
            }
        };
        for (key, _) in mapping {
            let key_str = match key.as_str() {
                Some(s) => s,
                None => continue,
            };
            if !known.contains(&key_str) {
                out.push(DriftEntry {
                    path: display_path.clone(),
                    kind: kind.to_string(),
                    field: key_str.to_string(),
                    disposition: DEFAULT_DISPOSITION.to_string(),
                });
            }
        }
        Ok(())
    };

    for path in layout
        .list_card_files()
        .map_err(|e| Error::unavailable(VERB, format!("list cards: {e}")))?
    {
        scan(&path, "card", Card::FIELDS, &mut drift)?;
    }
    for path in layout
        .list_spec_files()
        .map_err(|e| Error::unavailable(VERB, format!("list specs: {e}")))?
    {
        scan(&path, "spec", Spec::FIELDS, &mut drift)?;
    }
    for path in layout
        .list_choice_files()
        .map_err(|e| Error::unavailable(VERB, format!("list choices: {e}")))?
    {
        scan(&path, "choice", Choice::FIELDS, &mut drift)?;
    }
    for path in layout
        .list_memory_files()
        .map_err(|e| Error::unavailable(VERB, format!("list memories: {e}")))?
    {
        scan(&path, "memory", Memory::FIELDS, &mut drift)?;
    }

    Ok(AuditDriftResult { drift })
}

// ----- audit.topology (spec 2026-05-18-documentation-topology ac-06,
//        substrate-folder migration per choice 0025 + spec
//        2026-05-18-topology-substrate-migration ac-02) -----

/// Subdirectories under the repo root that the missing_entry heuristic
/// scans. Top-level dirs under these are candidate subsystems.
const TOPOLOGY_SUBSYSTEM_ROOTS: &[&str] = &["src", "crates"];

/// Load every `.orbit/topology/<subsystem>.yaml` as a parsed
/// `schema::TopologyEntry`. Files that fail to parse OR fail validate()
/// surface as Err entries — the audit verb folds them into the drift
/// envelope (`parse_failed` / `invalid_field` codes); other callers
/// (`compute_topology_warnings`) drop them silently because they're
/// advisory.
fn load_topology_entries(layout: &OrbitLayout) -> Vec<(String, std::result::Result<crate::schema::TopologyEntry, String>)> {
    let mut out = Vec::new();
    let paths = match layout.list_topology_files() {
        Ok(p) => p,
        Err(_) => return out,
    };
    for path in paths {
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let text = match std::fs::read_to_string(&path) {
            Ok(t) => t,
            Err(e) => {
                out.push((stem, Err(format!("read failed: {e}"))));
                continue;
            }
        };
        let entry: crate::schema::TopologyEntry = match serde_yaml::from_str(&text) {
            Ok(e) => e,
            Err(e) => {
                out.push((stem, Err(format!("parse failed: {e}"))));
                continue;
            }
        };
        if let Err(msg) = entry.validate() {
            out.push((stem, Err(format!("validate failed: {msg}"))));
            continue;
        }
        out.push((stem, Ok(entry)));
    }
    out
}

/// Detect top-level subsystem directories under `src/` / `crates/` from
/// the repo root. Returns a sorted, deduplicated list of directory names.
fn detect_subsystem_dirs(repo_root: &Path) -> Vec<String> {
    let mut names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for root in TOPOLOGY_SUBSYSTEM_ROOTS {
        let dir = repo_root.join(root);
        if !dir.is_dir() {
            continue;
        }
        let read = match std::fs::read_dir(&dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        for entry in read.flatten() {
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                if let Some(name) = entry.file_name().to_str() {
                    // Skip hidden / build dirs.
                    if name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                    names.insert(name.to_string());
                }
            }
        }
    }
    names.into_iter().collect()
}

fn audit_topology(
    layout: &OrbitLayout,
    _args: &AuditTopologyArgs,
) -> Result<AuditTopologyResult> {
    // Substrate-folder shape per choice 0025: topology lives at
    // `.orbit/topology/<subsystem>.yaml`, parsed via
    // `schema::TopologyEntry`. `configured` is true iff the directory
    // exists AND contains at least one entry (populated == configured
    // per design pass Q1 of spec
    // `2026-05-18-topology-substrate-migration`).
    let topology_dir = layout.topology_dir();
    let loaded = load_topology_entries(layout);

    let configured = topology_dir.exists() && !loaded.is_empty();
    if !configured {
        return Ok(AuditTopologyResult {
            configured: false,
            topology_drift: Vec::new(),
        });
    }

    let mut drift: Vec<TopologyDriftEntry> = Vec::new();
    let repo_root = &layout.root.parent().unwrap_or(&&layout.root);

    // 1. Per-file structural failures: parse_failed / invalid_field.
    //    Each unparseable or invalid file becomes one drift entry
    //    keyed by the file stem (best-effort subsystem name).
    for (stem, result) in &loaded {
        match result {
            Ok(_) => {}
            Err(msg) => {
                let kind = if msg.starts_with("validate failed") {
                    "invalid_field"
                } else if msg.starts_with("parse failed") {
                    "parse_failed"
                } else {
                    "shape_drift"
                };
                drift.push(TopologyDriftEntry {
                    subsystem: stem.clone(),
                    drift_kind: kind.into(),
                    detail: msg.clone(),
                });
            }
        }
    }

    // 2. Per-entry pointer drift: each path-shaped pointer is checked
    //    for filesystem existence (canonical_code / operational_doc /
    //    test_surface — opaque strings, but the convention is paths).
    //    decision_record entries try resolve_numeric_slug →
    //    layout.choice_file first, and fall through to a direct path
    //    check on resolution failure (per spec ac-01's two-step
    //    pattern).
    for (_, result) in &loaded {
        let entry = match result {
            Ok(e) => e,
            Err(_) => continue,
        };
        for path in &entry.canonical_code {
            if !repo_root.join(path).exists() {
                drift.push(TopologyDriftEntry {
                    subsystem: entry.subsystem.clone(),
                    drift_kind: "stale_pointer".into(),
                    detail: format!("canonical_code: {path}"),
                });
            }
        }
        for id_or_path in &entry.decision_record {
            // First, try resolve_numeric_slug against the choices dir.
            let resolved = resolve_numeric_slug("audit.topology", &layout.choices_dir(), id_or_path)
                .ok()
                .flatten();
            let target_path = match resolved {
                Some(slug) => layout.choice_file(&slug),
                None => repo_root.join(id_or_path),
            };
            if !target_path.exists() {
                drift.push(TopologyDriftEntry {
                    subsystem: entry.subsystem.clone(),
                    drift_kind: "stale_pointer".into(),
                    detail: format!("decision_record: {id_or_path}"),
                });
            }
        }
        for path in &entry.operational_doc {
            if !repo_root.join(path).exists() {
                drift.push(TopologyDriftEntry {
                    subsystem: entry.subsystem.clone(),
                    drift_kind: "stale_pointer".into(),
                    detail: format!("operational_doc: {path}"),
                });
            }
        }
        for path in &entry.test_surface {
            if !repo_root.join(path).exists() {
                drift.push(TopologyDriftEntry {
                    subsystem: entry.subsystem.clone(),
                    drift_kind: "stale_pointer".into(),
                    detail: format!("test_surface: {path}"),
                });
            }
        }
    }

    // 3. missing_entry: codebase subsystems with no topology entry.
    //    Preserves the existing taxonomy entry from the predecessor
    //    parser per envelope-shape continuity (spec ac-02).
    let documented: std::collections::HashSet<String> = loaded
        .iter()
        .filter_map(|(_, r)| r.as_ref().ok())
        .map(|e| e.subsystem.to_lowercase())
        .collect();
    for subsystem in detect_subsystem_dirs(repo_root) {
        if !documented.contains(&subsystem.to_lowercase()) {
            drift.push(TopologyDriftEntry {
                subsystem,
                drift_kind: "missing_entry".into(),
                detail: String::new(),
            });
        }
    }

    Ok(AuditTopologyResult {
        configured: true,
        topology_drift: drift,
    })
}

// ============================================================================
// audit.conformance — workflow-conformance audit
//
// Per spec 2026-05-19-workflow-conformance. Aggregates `audit.drift` and
// `audit.topology` results verbatim under `aggregated.{drift,topology}` and
// surfaces new finding families under `findings`:
//
// - ac-02: card-state — `maturity:planned` + empty specs
// - ac-03: memo-staleness — filename-date >7 days before today
// - ac-04: plugin-canonical-file drift — operator `.orbit/METHOD.md` vs
//   the compile-time canonical bytes embedded via `include_str!`
// - ac-05: plugin-version pin — `unpinned` | `matches` | `pin_behind` |
//   `pin_ahead`; `pin_behind` and `pin_ahead` BOTH suppress ac-04 per-file
//   findings (single-finding dominance).
//
// All findings carry `remediation.verb` — the agent acts without
// translation.
// ============================================================================

/// Memo-staleness threshold in days. Memos older than this fire a
/// finding under ac-03. Fixed at v1; no `.orbit/config.yaml` override.
const MEMO_STALENESS_THRESHOLD_DAYS: i64 = 7;

/// Routine-staleness threshold in days. Routines whose `last_verified`
/// is older than this fire a `routines/stale` finding from
/// `audit.conformance`. Per spec 2026-05-22-routine-proposals ac-07.
const ROUTINE_STALENESS_THRESHOLD_DAYS: i64 = 30;

/// Plugin-canonical-file inventory — `(operator_relative_path,
/// canonical_bytes)` pairs. The bytes are pulled at compile time via
/// `include_str!`, which means they always match the orbit-state
/// binary's `CARGO_PKG_VERSION` (lockstep release contract). The
/// inventory is the set of files `/orb:setup` copies verbatim from the
/// plugin into the operator's `.orbit/`. METHOD.md (the workflow
/// overview) and STYLE.md (the agent prose discipline) both qualify.
/// When future plugin releases add canonical files, extend this const;
/// no spec change required.
// The canonical bytes are vendored into the crate at
// `crates/core/canonical/{METHOD,STYLE}.md` so `include_str!` works
// under cross-compilation (cross's docker mount is limited to the
// workspace dir and can't reach `../plugins/`). The vendored copies are
// kept in sync with `plugins/orb/skills/setup/{METHOD,STYLE}.md` as a
// /orb:release pre-flight step — drift between the two is a
// release-discipline concern, surfaced by `cargo test
// conformance_vendored_{method,style}_md_matches_plugin` when both files
// are readable (local) and by /orb:release's pre-flight when only one
// is (CI).
const CANONICAL_FILES: &[(&str, &str)] = &[
    (".orbit/METHOD.md", include_str!("../canonical/METHOD.md")),
    (".orbit/STYLE.md", include_str!("../canonical/STYLE.md")),
];

/// Public verb entry — calls into the testable helper with today's
/// local date. The helper takes `today` as a parameter so unit tests
/// can drive memo-staleness deterministically.
fn audit_conformance(
    layout: &OrbitLayout,
    args: &AuditConformanceArgs,
) -> Result<AuditConformanceResult> {
    let today = time::OffsetDateTime::now_local()
        .map_err(|e| {
            Error::malformed("audit.conformance", format!("local offset unavailable: {e}"))
        })?
        .date();
    audit_conformance_at(layout, args, today)
}

/// Private testable helper. `today` is injected so unit tests can fix
/// the date — see ac-03 verification.
fn audit_conformance_at(
    layout: &OrbitLayout,
    _args: &AuditConformanceArgs,
    today: time::Date,
) -> Result<AuditConformanceResult> {
    const VERB: &str = "audit.conformance";
    let _ = VERB;
    let _ = today;

    let drift = audit_drift(layout, &AuditDriftArgs::default())?;
    let topology = audit_topology(layout, &AuditTopologyArgs::default())?;
    let pin = derive_pin_state(layout)?;

    let mut findings: Vec<ConformanceFinding> = Vec::new();

    // Layout-dominance check — per spec 2026-05-24-workflow-conformance
    // decisions doc §2 + §6. When `orbit/` substrate is present but
    // `.orbit/cards/` is absent, every other finding family that reads
    // from `.orbit/` is structurally misleading (would fire on missing
    // files that are actually present at the wrong path). Mirror the
    // existing `pin_dominates` pattern: compute once, gate every
    // `.orbit/`-dependent emission. The new `undotted_substrate` finding
    // is the one finding the agent sees in this state — its
    // remediation (`orbit setup`) is the prerequisite for the rest of
    // the audit being meaningful.
    let undotted_finding = undotted_substrate_finding(layout);
    let layout_dominates = undotted_finding.is_some();

    // ac-05: pin_behind / pin_ahead each fire a SINGLE finding AND
    // suppress per-file (ac-04) findings. Single-finding dominance:
    // the pin issue is upstream of file drift. Layout-dominance is
    // upstream of pin (the pin file lives at `.orbit/config.yaml` —
    // when the layout is wrong, the pin state read is meaningless).
    let pin_dominates =
        !layout_dominates && matches!(pin.status.as_str(), "pin_behind" | "pin_ahead");

    if layout_dominates {
        // Single-finding emission — `undotted_substrate` is the only
        // finding the agent sees. The remediation `orbit setup` is the
        // prerequisite; re-running the audit after layout migration
        // surfaces the real state.
        findings.push(undotted_finding.unwrap());
    } else {
        if pin_dominates {
            findings.push(pin_finding(&pin));
        }

        // ac-02: card-state findings.
        findings.extend(card_state_findings(layout)?);

        // ac-03: memo-staleness findings.
        findings.extend(memo_staleness_findings(layout, today)?);

        // ac-04: plugin-canonical-file drift findings (suppressed when
        // pin_dominates).
        if !pin_dominates {
            findings.extend(canonical_file_findings(layout)?);
        }

        // decisions-md-unmigrated findings — per spec
        // 2026-05-24-setup-is-orbit-state-aware ac-15. Brownfield
        // migration renames `decisions/ → .orbit/decisions/` verbatim
        // (no MD→YAML conversion). This finding family surfaces each
        // unconverted .md file so the operator can drive the conversion
        // themselves.
        findings.extend(decisions_md_unmigrated_findings(layout)?);
    }

    // Routine findings (per spec 2026-05-22-routine-proposals ac-07).
    // Two state slugs — `stale` (last_verified > 30d) and `broken_refs`
    // (one or more /orb:<verb> refs no longer resolve). Read-only — the
    // audit aggregator never mutates routines (ac-06 split). Routine
    // files live under `.claude/skills/`, NOT `.orbit/`, so they remain
    // meaningful regardless of the substrate layout state — emitted on
    // every branch.
    findings.extend(routine_findings(layout, today)?);

    Ok(AuditConformanceResult {
        findings,
        aggregated: AggregatedAudits { drift, topology },
        pin,
    })
}

/// ac-02: walk `.orbit/cards/*.yaml`; emit a finding for each card at
/// `maturity:planned` with an empty specs array. Remediation:
/// `/orb:tabletop <numeric-id>`.
fn card_state_findings(layout: &OrbitLayout) -> Result<Vec<ConformanceFinding>> {
    const VERB: &str = "audit.conformance";
    let mut findings = Vec::new();
    let card_files = layout.list_card_files().unwrap_or_default();
    for path in card_files {
        let raw = match std::fs::read_to_string(&path) {
            Ok(r) => r,
            Err(_) => continue,
        };
        let card: crate::schema::Card = match serde_yaml::from_str(&raw) {
            Ok(c) => c,
            Err(_) => continue,
        };
        // Match the deserialised maturity. Card maturity is an enum;
        // serde rename_all = snake_case → "planned" in the on-disk yaml.
        let is_planned = matches!(card.maturity, crate::schema::CardMaturity::Planned);
        if !is_planned || !card.specs.is_empty() || card.park.is_some() {
            // Parked cards skip the ready_for_tabletop finding silently —
            // per spec 2026-05-20-conformance-park-signal. The deliberate-hold
            // signal carries its own reason/until; no envelope trace.
            continue;
        }
        // Card.id is Option<String> for backwards-compat — fall back to
        // the filename slug if unset.
        let card_id = card.id.clone().unwrap_or_else(|| {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string()
        });
        if card_id.is_empty() {
            continue;
        }
        let numeric_id = numeric_id_from_card_id(&card_id);
        let mut evidence = serde_yaml::Mapping::new();
        evidence.insert(
            serde_yaml::Value::String("maturity".into()),
            serde_yaml::Value::String("planned".into()),
        );
        evidence.insert(
            serde_yaml::Value::String("specs_count".into()),
            serde_yaml::Value::Number(0i64.into()),
        );
        findings.push(ConformanceFinding {
            severity: "medium".into(),
            subsystem: "cards".into(),
            subject: card_id,
            state: "ready_for_tabletop".into(),
            evidence: Some(serde_yaml::Value::Mapping(evidence)),
            remediation: Remediation {
                verb: format!("/orb:tabletop {numeric_id}"),
                rationale: Some("card has scenarios but no tabletop pass".into()),
            },
        });
    }
    let _ = VERB;
    Ok(findings)
}

/// Extract the leading numeric id from a card slug like
/// `0039-workflow-conformance` → `"39"` (drops leading zeros). When
/// the slug has no leading digits, returns the slug verbatim.
fn numeric_id_from_card_id(id: &str) -> String {
    let digits: String = id.chars().take_while(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() {
        return id.to_string();
    }
    let trimmed = digits.trim_start_matches('0');
    if trimmed.is_empty() {
        "0".to_string()
    } else {
        trimmed.to_string()
    }
}

/// ac-03: walk `.orbit/memos/*.md`; emit a finding for each memo whose
/// filename-date is more than `MEMO_STALENESS_THRESHOLD_DAYS` before
/// today. Remediation: `/orb:distill <relative-path>`.
fn memo_staleness_findings(
    layout: &OrbitLayout,
    today: time::Date,
) -> Result<Vec<ConformanceFinding>> {
    let mut findings = Vec::new();
    let memo_files = layout.list_memo_files().unwrap_or_default();
    for path in memo_files {
        let filename = match path.file_name().and_then(|s| s.to_str()) {
            Some(f) => f,
            None => continue,
        };
        if filename.len() < 10 {
            continue;
        }
        let date_str = &filename[..10];
        let format = match time::format_description::parse("[year]-[month]-[day]") {
            Ok(f) => f,
            Err(_) => continue,
        };
        let memo_date = match time::Date::parse(date_str, &format) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let days_old = (today - memo_date).whole_days();
        if days_old <= MEMO_STALENESS_THRESHOLD_DAYS {
            continue;
        }
        // Compute path relative to `.orbit/` for the subject/remediation.
        let rel_subject = path
            .strip_prefix(layout.repo_root())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| path.to_string_lossy().into_owned());
        let mut evidence = serde_yaml::Mapping::new();
        evidence.insert(
            serde_yaml::Value::String("filename_date".into()),
            serde_yaml::Value::String(date_str.to_string()),
        );
        evidence.insert(
            serde_yaml::Value::String("days_old".into()),
            serde_yaml::Value::Number(days_old.into()),
        );
        findings.push(ConformanceFinding {
            severity: "medium".into(),
            subsystem: "memos".into(),
            subject: rel_subject.clone(),
            state: "stale".into(),
            evidence: Some(serde_yaml::Value::Mapping(evidence)),
            remediation: Remediation {
                verb: format!("/orb:distill {rel_subject}"),
                rationale: Some("memo undistilled past 7-day threshold".into()),
            },
        });
    }
    Ok(findings)
}

/// ac-04: byte-compare each operator file in `CANONICAL_FILES`
/// against the compile-time canonical bytes. Emit `byte_drift` for
/// differing files and `missing` for absent files. Remediation:
/// `orbit setup`.
fn canonical_file_findings(layout: &OrbitLayout) -> Result<Vec<ConformanceFinding>> {
    let mut findings = Vec::new();
    let root = layout.repo_root();
    for (rel, canonical) in CANONICAL_FILES {
        let path = root.join(rel);
        let exists = path.exists();
        if !exists {
            findings.push(ConformanceFinding {
                severity: "medium".into(),
                subsystem: "setup".into(),
                subject: (*rel).to_string(),
                state: "missing".into(),
                evidence: None,
                remediation: Remediation {
                    verb: "orbit setup".into(),
                    rationale: Some("re-run setup to overwrite with canonical".into()),
                },
            });
            continue;
        }
        let operator = match std::fs::read_to_string(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if operator.as_bytes() == canonical.as_bytes() {
            continue;
        }
        let mut evidence = serde_yaml::Mapping::new();
        evidence.insert(
            serde_yaml::Value::String("canonical_size".into()),
            serde_yaml::Value::Number((canonical.len() as i64).into()),
        );
        evidence.insert(
            serde_yaml::Value::String("operator_size".into()),
            serde_yaml::Value::Number((operator.len() as i64).into()),
        );
        findings.push(ConformanceFinding {
            severity: "medium".into(),
            subsystem: "setup".into(),
            subject: (*rel).to_string(),
            state: "byte_drift".into(),
            evidence: Some(serde_yaml::Value::Mapping(evidence)),
            remediation: Remediation {
                verb: "orbit setup".into(),
                rationale: Some("re-run setup to overwrite with canonical".into()),
            },
        });
    }
    Ok(findings)
}

/// `undotted_substrate`: HIGH-severity finding that fires when substrate
/// content lives at `orbit/` (no dot) and canonical `.orbit/cards/` is
/// absent. The remediation `orbit setup` runs the wrapped-undotted
/// single-rename migration (see `/orb:setup` §3.W).
///
/// Predicate (per spec 2026-05-24-workflow-conformance decisions §1):
/// - any of `orbit/{cards,choices,specs,memos}/` exists (positive signal
///   that substrate is wrapped at the wrong path), AND
/// - `.orbit/cards/` does NOT exist (negative guard — once canonical
///   substrate is in place, the finding stops firing regardless of
///   leftover `orbit/` content).
///
/// Evidence carries per-subdir item counts so consumers (e.g.
/// `/orb:prioritise`) can surface the scale of the migration — a
/// 22-card brownfield repo is materially different from an empty
/// scaffold even though both trigger HIGH equally.
///
/// Returns `None` when the predicate doesn't hold. Caller in
/// `audit_conformance_at` treats `Some` as "layout dominates" — the
/// single emitted finding replaces the suppressed `.orbit/`-dependent
/// families (canonical-files-missing, card-state, memo-staleness,
/// pin-state).
///
/// Per spec 2026-05-24-workflow-conformance ac-09 / ac-10.
fn undotted_substrate_finding(layout: &OrbitLayout) -> Option<ConformanceFinding> {
    let repo_root = layout.repo_root();
    // Negative guard — canonical substrate already in place.
    if repo_root.join(".orbit/cards").is_dir() {
        return None;
    }
    // Positive predicate — any of orbit/{cards,choices,specs,memos}/.
    let subdirs = ["cards", "choices", "specs", "memos"];
    if !subdirs
        .iter()
        .any(|s| repo_root.join("orbit").join(s).is_dir())
    {
        return None;
    }
    // Volume counts for evidence — bounded read_dir on each existing
    // subdir; missing subdirs contribute 0.
    let mut evidence = serde_yaml::Mapping::new();
    for subdir in &subdirs {
        let path = repo_root.join("orbit").join(subdir);
        let count: i64 = if path.is_dir() {
            std::fs::read_dir(&path)
                .map(|entries| entries.flatten().count() as i64)
                .unwrap_or(0)
        } else {
            0
        };
        evidence.insert(
            serde_yaml::Value::String(format!("{subdir}_count")),
            serde_yaml::Value::Number(count.into()),
        );
    }
    Some(ConformanceFinding {
        severity: "high".into(),
        subsystem: "setup".into(),
        subject: "orbit/".into(),
        state: "undotted_substrate".into(),
        evidence: Some(serde_yaml::Value::Mapping(evidence)),
        remediation: Remediation {
            verb: "orbit setup".into(),
            rationale: Some(
                "substrate lives at orbit/ but canonical layout is .orbit/ — \
                 run setup's wrapped-undotted migration before anything else \
                 (downstream findings against .orbit/ are misleading until the rename lands)"
                    .into(),
            ),
        },
    })
}

/// `decisions_md_unmigrated`: walk `.orbit/decisions/*.md`; for each
/// markdown file lacking a matching `.orbit/choices/<slug>.yaml`, emit
/// a finding pointing the operator at the manual MD→YAML conversion.
///
/// Brownfield migration renames `decisions/ → .orbit/decisions/` verbatim
/// (no auto-conversion) — the format gap requires human judgment per
/// file. The finding family makes the residue visible on every audit
/// pass; closing each finding means hand-rewriting the MADR markdown as
/// a Choice yaml and removing the source `.md`.
///
/// Returns an empty vec when `.orbit/decisions/` is absent or empty.
/// The slug match is filename-stem to filename-stem (the conventional
/// shape under both folders); no semantic-content comparison.
///
/// Per spec 2026-05-24-setup-is-orbit-state-aware ac-15.
fn decisions_md_unmigrated_findings(
    layout: &OrbitLayout,
) -> Result<Vec<ConformanceFinding>> {
    let mut findings = Vec::new();
    let decisions_dir = &layout.root.join("decisions");
    if !decisions_dir.is_dir() {
        return Ok(findings);
    }
    let entries = match std::fs::read_dir(&decisions_dir) {
        Ok(e) => e,
        Err(_) => return Ok(findings),
    };
    let mut md_paths: Vec<std::path::PathBuf> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        md_paths.push(path);
    }
    md_paths.sort();
    for md_path in md_paths {
        let stem = match md_path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let choice_path = layout.choice_file(stem);
        if choice_path.exists() {
            // Operator already converted this one. Don't flag.
            continue;
        }
        let rel_subject = md_path
            .strip_prefix(layout.repo_root())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| md_path.to_string_lossy().into_owned());
        let target_rel = choice_path
            .strip_prefix(layout.repo_root())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| choice_path.to_string_lossy().into_owned());
        let mut evidence = serde_yaml::Mapping::new();
        evidence.insert(
            serde_yaml::Value::String("source_md".into()),
            serde_yaml::Value::String(rel_subject.clone()),
        );
        evidence.insert(
            serde_yaml::Value::String("target_yaml".into()),
            serde_yaml::Value::String(target_rel.clone()),
        );
        findings.push(ConformanceFinding {
            severity: "medium".into(),
            subsystem: "setup".into(),
            subject: rel_subject.clone(),
            state: "decisions_md_unmigrated".into(),
            evidence: Some(serde_yaml::Value::Mapping(evidence)),
            remediation: Remediation {
                verb: format!(
                    "manual MD→YAML conversion needed: {rel_subject} → {target_rel}"
                ),
                rationale: Some(
                    "brownfield migration renames decisions/ verbatim; the MADR markdown \
                     needs hand-conversion to the canonical Choice yaml shape (no auto-converter)"
                        .into(),
                ),
            },
        });
    }
    Ok(findings)
}

/// Routine-conformance findings (per spec 2026-05-22-routine-proposals
/// ac-07). Two state slugs:
///
/// - `stale`: `last_verified` older than [`ROUTINE_STALENESS_THRESHOLD_DAYS`].
///   Remediation: `orbit routine verify <path>`.
/// - `broken_refs`: one or more `/orb:<verb>` refs in the routine's
///   SKILL.md body no longer resolves to a live skill. Remediation:
///   `archive via curator`.
///
/// Audit is **read-only** — does NOT mutate routine files (the ac-06
/// split puts `last_verified` writes exclusively in `routine.verify`).
fn routine_findings(
    layout: &OrbitLayout,
    today: time::Date,
) -> Result<Vec<ConformanceFinding>> {
    let mut findings = Vec::new();
    let claude_dir = layout.claude_skills_dir();
    if !claude_dir.exists() {
        return Ok(findings);
    }
    let entries = match std::fs::read_dir(&claude_dir) {
        Ok(e) => e,
        Err(_) => return Ok(findings),
    };
    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        // .archive/ is the curator's bin; routines there are intentionally
        // out of view.
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
        let fm = match crate::routine::parse_front_matter(&body) {
            Ok(f) => f,
            // Routines with malformed front-matter aren't this audit's
            // problem (the AC-04 write-path validation gates that on
            // creation); skip silently here to keep the audit narrow.
            Err(_) => continue,
        };
        // Only flag agent-authored routines. Human-authored skills are
        // out of scope per the curator's invariants (card 0022).
        if fm.created_by != "agent" {
            continue;
        }

        let rel_subject = skill_md
            .strip_prefix(layout.repo_root())
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|_| skill_md.to_string_lossy().into_owned());

        // Stale check.
        if let Some(days_old) = days_since_iso(&fm.last_verified, today) {
            if days_old > ROUTINE_STALENESS_THRESHOLD_DAYS {
                let mut evidence = serde_yaml::Mapping::new();
                evidence.insert(
                    serde_yaml::Value::String("last_verified".into()),
                    serde_yaml::Value::String(fm.last_verified.clone()),
                );
                evidence.insert(
                    serde_yaml::Value::String("days_since_verified".into()),
                    serde_yaml::Value::Number(days_old.into()),
                );
                findings.push(ConformanceFinding {
                    severity: "medium".into(),
                    subsystem: "routines".into(),
                    subject: rel_subject.clone(),
                    state: "stale".into(),
                    evidence: Some(serde_yaml::Value::Mapping(evidence)),
                    remediation: Remediation {
                        verb: format!("orbit routine verify {rel_subject}"),
                        rationale: Some(
                            "last_verified is older than the 30-day routine freshness threshold"
                                .into(),
                        ),
                    },
                });
            }
        }

        // Broken-refs check.
        let mut prose_refs = extract_orb_refs(&body);
        for ch in &fm.chain {
            if !prose_refs.contains(ch) {
                prose_refs.push(ch.clone());
            }
        }
        let (_resolved, broken) = partition_refs(layout, &prose_refs);
        if !broken.is_empty() {
            let mut evidence = serde_yaml::Mapping::new();
            evidence.insert(
                serde_yaml::Value::String("broken_refs".into()),
                serde_yaml::Value::Sequence(
                    broken
                        .iter()
                        .map(|r| serde_yaml::Value::String(r.clone()))
                        .collect(),
                ),
            );
            findings.push(ConformanceFinding {
                severity: "medium".into(),
                subsystem: "routines".into(),
                subject: rel_subject,
                state: "broken_refs".into(),
                evidence: Some(serde_yaml::Value::Mapping(evidence)),
                remediation: Remediation {
                    verb: "archive via curator".into(),
                    rationale: Some(
                        "one or more /orb:<verb> references no longer resolve to a live skill"
                            .into(),
                    ),
                },
            });
        }
    }
    Ok(findings)
}

/// Return the (positive) day-count between an RFC 3339 / ISO 8601
/// timestamp and `today`. Returns `None` when the timestamp can't be
/// parsed (the substrate wrote a malformed value) so the audit skips
/// silently — `audit.conformance` mustn't error on bad routine data.
fn days_since_iso(ts: &str, today: time::Date) -> Option<i64> {
    // Take just the date portion (YYYY-MM-DD prefix). The substrate
    // writes RFC 3339 UTC so this is always the first 10 chars.
    if ts.len() < 10 {
        return None;
    }
    let date_str = &ts[..10];
    let format = time::format_description::parse("[year]-[month]-[day]").ok()?;
    let then = time::Date::parse(date_str, &format).ok()?;
    Some((today - then).whole_days())
}

/// ac-05: derive PinState. The installed plugin version is read from
/// the orbit-state binary's `CARGO_PKG_VERSION` (lockstep release
/// contract with the plugin manifest version). The pinned version is
/// read from `Config.plugin_version`.
fn derive_pin_state(layout: &OrbitLayout) -> Result<PinState> {
    let current = env!("CARGO_PKG_VERSION").to_string();
    let pinned = read_pinned_version(layout);
    let status = match pinned.as_deref() {
        None => "unpinned",
        Some(p) if p == current => "matches",
        Some(p) => match compare_versions(p, &current) {
            std::cmp::Ordering::Less => "pin_behind",
            std::cmp::Ordering::Greater => "pin_ahead",
            std::cmp::Ordering::Equal => "matches",
        },
    }
    .to_string();
    Ok(PinState {
        pinned,
        current,
        status,
    })
}

/// Read `Config.plugin_version` from `.orbit/config.yaml`. Returns
/// `None` if the config file is absent OR the field is absent. Parse
/// failures are also `None` — verify is the gate for malformed config.
fn read_pinned_version(layout: &OrbitLayout) -> Option<String> {
    let config_path = layout.config_file();
    let raw = std::fs::read_to_string(&config_path).ok()?;
    let config: crate::schema::Config = serde_yaml::from_str(&raw).ok()?;
    config.plugin_version
}

/// Read `Config.plugin_repo` from `.orbit/config.yaml`. Returns `false`
/// when the file is absent, the field is unset, or parsing fails — the
/// substrate-typed seed branch is the dangerous default (writes pointers
/// that only exist in the orbit-plugin source repo), so the safe default
/// is "this is not the plugin repo". Per spec
/// 2026-05-24-setup-is-orbit-state-aware ac-12.
fn read_plugin_repo_flag(layout: &OrbitLayout) -> bool {
    let config_path = layout.config_file();
    let Ok(raw) = std::fs::read_to_string(&config_path) else {
        return false;
    };
    let Ok(config): std::result::Result<crate::schema::Config, _> = serde_yaml::from_str(&raw)
    else {
        return false;
    };
    config.plugin_repo.unwrap_or(false)
}

/// Substrate-layout classifier: the six mutually-exclusive states a repo
/// can be in with respect to the orbit substrate. The six variants exhaust
/// the reachable combinations of three independent axes
/// (`.orbit/` present?, `orbit/` present?, any bare artefact dir at root?).
///
/// Used by `/orb:setup`'s state-machine in §1 (the agent runs
/// [`classify_substrate_layout`] before deciding which migration arm to
/// enter) and by `audit.conformance` (the sister verb
/// `2026-05-24-workflow-conformance` consumes the same classifier to
/// suppress `canonical-files-missing` findings on a `wrapped-undotted`
/// repo — fixing the layout is the prerequisite, not the canonical-file
/// drift). Single source of truth: skill prose and audit logic call the
/// same predicate.
///
/// Per spec 2026-05-24-setup-is-orbit-state-aware ac-11.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SubstrateLayoutState {
    /// None of `.orbit/`, `orbit/`, or bare artefact dirs present. Setup
    /// creates `.orbit/` fresh.
    Greenfield,
    /// `.orbit/` present; neither `orbit/` nor any bare artefact dir.
    /// Setup is a no-op on layout (still runs canonical-files check).
    Idempotent,
    /// Bare `cards/` / `specs/` / `decisions/` / `discovery/` at root;
    /// neither `.orbit/` nor `orbit/`. Setup prompts then `git mv`s each
    /// detected bare dir under `.orbit/`.
    BrownfieldBare,
    /// `orbit/` present (wrapped substrate from a pre-`.orbit/` plugin
    /// version); `.orbit/` absent; no bare dirs. Setup prompts then runs
    /// the single-rename `git mv orbit .orbit`.
    WrappedUndotted,
    /// `.orbit/` AND any bare artefact dir present. Refuses with a
    /// collision message — auto-resolution would risk overwriting work.
    MixedBare,
    /// `.orbit/` AND `orbit/` both present. Refuses with the
    /// `.orbit/`+`orbit/` collision message — `git mv orbit .orbit` would
    /// otherwise blow up with an opaque "destination exists" error.
    MixedUndotted,
}

const BARE_ARTEFACT_DIRS: &[&str] = &["cards", "specs", "decisions", "discovery"];

/// Inspect the working tree at `layout.repo_root()` and classify it into
/// one of the six [`SubstrateLayoutState`] variants.
///
/// The predicate is filesystem-only (no git index reads, no config
/// parsing) — it answers the same question that `/orb:setup` §1's table
/// describes, in code. The three axes are inspected once each:
///
/// 1. `.orbit/` directory present at the repo root?
/// 2. `orbit/` directory present at the repo root?
/// 3. Any of {`cards`, `specs`, `decisions`, `discovery`} present as a
///    directory at the repo root?
///
/// The 8-way truth table collapses to 6 reachable states (the two with
/// `orbit/` present AND bare dirs present are not classified separately —
/// they fold into the `mixed-undotted` arm because `.orbit/` may or may
/// not also be present, and the dominant failure mode is the
/// `orbit/`/`.orbit/` collision).
///
/// Per spec 2026-05-24-setup-is-orbit-state-aware ac-11.
pub fn classify_substrate_layout(layout: &OrbitLayout) -> SubstrateLayoutState {
    let repo_root = layout.repo_root();
    let dotted = &layout.root.is_dir();
    let undotted = repo_root.join("orbit").is_dir();
    let bare_present = BARE_ARTEFACT_DIRS
        .iter()
        .any(|name| repo_root.join(name).is_dir());

    match (dotted, undotted, bare_present) {
        (true, true, _) => SubstrateLayoutState::MixedUndotted,
        (true, false, true) => SubstrateLayoutState::MixedBare,
        (true, false, false) => SubstrateLayoutState::Idempotent,
        (false, true, _) => SubstrateLayoutState::WrappedUndotted,
        (false, false, true) => SubstrateLayoutState::BrownfieldBare,
        (false, false, false) => SubstrateLayoutState::Greenfield,
    }
}

/// Verb impl for `substrate.classify`. Pure-read — no filesystem mutation,
/// no config parsing, no Result error path. The classifier returns a
/// state for every reachable repo shape, so the function is infallible.
/// Per spec 2026-05-24-setup-is-orbit-state-aware ac-11 / ac-18.
fn substrate_classify(
    layout: &OrbitLayout,
    _args: &SubstrateClassifyArgs,
) -> Result<SubstrateClassifyResult> {
    Ok(SubstrateClassifyResult {
        state: classify_substrate_layout(layout),
    })
}

/// Lexicographic semver-ish compare on dotted-numeric strings. Treats
/// each dot-separated segment as a u64; non-numeric segments fall back
/// to string compare. Adequate for `MAJOR.MINOR.PATCH` strings; not a
/// full semver implementation.
fn compare_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let a_parts: Vec<&str> = a.split('.').collect();
    let b_parts: Vec<&str> = b.split('.').collect();
    let len = a_parts.len().max(b_parts.len());
    for i in 0..len {
        let av = a_parts.get(i).copied().unwrap_or("0");
        let bv = b_parts.get(i).copied().unwrap_or("0");
        let ord = match (av.parse::<u64>(), bv.parse::<u64>()) {
            (Ok(an), Ok(bn)) => an.cmp(&bn),
            _ => av.cmp(bv),
        };
        if ord != std::cmp::Ordering::Equal {
            return ord;
        }
    }
    std::cmp::Ordering::Equal
}

/// Build the pin-state finding from a derived PinState. Only called
/// when `pin.status` is `pin_behind` or `pin_ahead`.
fn pin_finding(pin: &PinState) -> ConformanceFinding {
    let (severity, rationale) = match pin.status.as_str() {
        "pin_ahead" => (
            "high",
            "installed plugin is older than pinned version — install matching plugin or rewrite pin",
        ),
        _ => (
            "medium",
            "installed plugin is ahead of pinned version — bump or re-run setup",
        ),
    };
    let mut evidence = serde_yaml::Mapping::new();
    if let Some(p) = &pin.pinned {
        evidence.insert(
            serde_yaml::Value::String("pinned".into()),
            serde_yaml::Value::String(p.clone()),
        );
    }
    evidence.insert(
        serde_yaml::Value::String("current".into()),
        serde_yaml::Value::String(pin.current.clone()),
    );
    ConformanceFinding {
        severity: severity.into(),
        subsystem: "setup".into(),
        subject: ".orbit/config.yaml".into(),
        state: pin.status.clone(),
        evidence: Some(serde_yaml::Value::Mapping(evidence)),
        remediation: Remediation {
            verb: "orbit setup --bump-pin".into(),
            rationale: Some(rationale.into()),
        },
    }
}

/// `topology.setup` — scaffold the `.orbit/topology/` substrate folder
/// and write the self-describing seed entries. Per spec
/// 2026-05-18-topology-substrate-migration ac-05.
///
/// Operation order:
/// 1. Brownfield cleanup: strip legacy `docs.topology` from
///    `.orbit/config.yaml` if present.
/// 2. Create `.orbit/topology/` (idempotent).
/// 3. Write one seed entry per `.orbit/` entity type (cards, choices,
///    specs, memories, topology). Existing entries are skipped — operator
///    edits preserved.
///
/// Idempotency: every step is no-op on a clean substrate; re-running
/// produces no on-disk diff.
fn topology_setup(
    layout: &OrbitLayout,
    args: &TopologySetupArgs,
) -> Result<TopologySetupResult> {
    const VERB: &str = "topology.setup";

    // Decline path: caller scripted "n".
    if matches!(args.answer_wire.as_deref(), Some("n") | Some("N") | Some("no")) {
        return Ok(TopologySetupResult {
            config_cleaned: false,
            dir_created: false,
            seeds_created: Vec::new(),
            seeds_skipped: Vec::new(),
            declined: true,
            readme_created: false,
        });
    }

    // 1. Brownfield cleanup: legacy docs.topology in .orbit/config.yaml.
    let mut config_cleaned = false;
    let config_path = layout.config_file();
    if config_path.exists() {
        let text = std::fs::read_to_string(&config_path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", config_path.display()))
        })?;
        let mut config: crate::schema::Config = serde_yaml::from_str(&text)
            .map_err(|e| Error::malformed(VERB, format!("parse config.yaml: {e}")))?;
        if let Some(docs) = config.docs.as_mut() {
            if docs.topology.take().is_some() {
                config_cleaned = true;
            }
        }
        // Elide an empty docs block entirely so verify_all stays clean.
        if let Some(docs) = config.docs.as_ref() {
            if docs.topology.is_none() {
                config.docs = None;
            }
        }
        if config_cleaned {
            let new_text = if config.docs.is_none() {
                // Empty struct serialises as "docs: null\n" or similar —
                // prefer an empty Config which round-trips as "{}\n".
                serde_yaml::to_string(&config).map_err(|e| {
                    Error::malformed(VERB, format!("reserialise config.yaml: {e}"))
                })?
            } else {
                serde_yaml::to_string(&config).map_err(|e| {
                    Error::malformed(VERB, format!("reserialise config.yaml: {e}"))
                })?
            };
            write_atomic(&config_path, new_text.as_bytes()).map_err(|e| {
                Error::unavailable(VERB, format!("write {}: {e}", config_path.display()))
            })?;
        }
    }

    // 2. Create .orbit/topology/ (idempotent).
    let topology_dir = layout.topology_dir();
    let dir_created = !topology_dir.exists();
    if dir_created {
        std::fs::create_dir_all(&topology_dir).map_err(|e| {
            Error::unavailable(VERB, format!("create {}: {e}", topology_dir.display()))
        })?;
    }

    // 3. Seed branch — substrate-typed seeds OR README-only.
    //
    // Per spec 2026-05-24-setup-is-orbit-state-aware ac-12: substrate-typed
    // seeds (`cards` / `choices` / `memories` / `specs-substrate` /
    // `topology`) describe orbit-state's own data types and the pointers
    // in them reach into the orbit-plugin source tree
    // (`orbit-state/crates/core/src/schema.rs`, `plugins/orb/skills/...`).
    // Writing those seeds into a downstream project produces a topology
    // populated with paths that don't exist in that tree — `orbit audit
    // topology` fires stale-pointer drift on every one immediately after
    // setup. The plugin_repo flag in `.orbit/config.yaml` gates the branch:
    // true means "this IS the orbit-plugin source repo, the substrate-typed
    // pointers are load-bearing here"; absent/false means "this is a
    // downstream project, write a one-line README pointer instead".
    let plugin_repo = read_plugin_repo_flag(layout);
    let mut seeds_created = Vec::new();
    let mut seeds_skipped = Vec::new();
    let mut readme_created = false;

    if plugin_repo {
        // Plugin-repo branch: validate canonical_code paths exist in the
        // working tree before writing any seed. Per spec
        // 2026-05-24-setup-is-orbit-state-aware ac-13. A missing path means
        // the seed would be stale on its very first audit pass — far better
        // to refuse loudly here than write a known-broken pointer.
        let repo_root = layout.repo_root();
        let seeds = topology_setup_seeds();
        for seed in &seeds {
            for code_path in &seed.canonical_code {
                let full = repo_root.join(code_path);
                if !full.exists() {
                    return Err(Error::not_found(
                        VERB,
                        format!(
                            "seed `{}` canonical_code path does not exist: {} \
                             (plugin_repo: true seeds must point at extant code in the working tree)",
                            seed.subsystem, code_path
                        ),
                    ));
                }
            }
        }
        for seed in seeds {
            let path = layout.topology_file(&seed.subsystem);
            if path.exists() {
                seeds_skipped.push(seed.subsystem.clone());
                continue;
            }
            let yaml = serde_yaml::to_string(&seed).map_err(|e| {
                Error::malformed(VERB, format!("serialise seed {}: {e}", seed.subsystem))
            })?;
            write_atomic(&path, yaml.as_bytes()).map_err(|e| {
                Error::unavailable(VERB, format!("write {}: {e}", path.display()))
            })?;
            seeds_created.push(seed.subsystem.clone());
        }
    } else {
        // Non-plugin-repo branch: write a one-line README that primes the
        // operator on how to author the first topology entry. Substrate-
        // typed seeds are deliberately NOT written here — they describe
        // orbit-state's own types and are a category error elsewhere.
        let readme_path = topology_dir.join("README.md");
        if !readme_path.exists() {
            let body = TOPOLOGY_NON_PLUGIN_README;
            write_atomic(&readme_path, body.as_bytes()).map_err(|e| {
                Error::unavailable(VERB, format!("write {}: {e}", readme_path.display()))
            })?;
            readme_created = true;
        }
    }

    Ok(TopologySetupResult {
        config_cleaned,
        dir_created,
        seeds_created,
        seeds_skipped,
        declined: false,
        readme_created,
    })
}

/// One-line README written into `.orbit/topology/` on non-plugin-repo
/// setup runs. The substrate-typed seeds (`cards` / `choices` / ...)
/// only make sense inside orbit-plugin's own source tree; downstream
/// projects get this pointer instead so the operator knows the
/// substrate exists but is intentionally unseeded. Per spec
/// 2026-05-24-setup-is-orbit-state-aware ac-12.
const TOPOLOGY_NON_PLUGIN_README: &str = "# .orbit/topology/

This directory holds per-subsystem topology entries — pointer-only
substrate that names each subsystem's canonical code, decision record,
operational doc, and test surface.

Run `/orb:topology` to author your first entry.
";

/// The self-describing seed templates written by `topology.setup`.
/// One entry per `.orbit/` entity type — universal across any orbit-using
/// repo. Subsystem slugs are slug-shaped (lowercase letters, ≥ 5 chars)
/// per `TopologyEntry::validate`.
fn topology_setup_seeds() -> Vec<crate::schema::TopologyEntry> {
    vec![
        crate::schema::TopologyEntry {
            subsystem: "cards".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0016".into()],
            operational_doc: vec!["plugins/orb/skills/card/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        },
        crate::schema::TopologyEntry {
            subsystem: "choices".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0016".into()],
            operational_doc: vec!["plugins/orb/skills/tabletop/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        },
        crate::schema::TopologyEntry {
            subsystem: "memories".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0015".into()],
            operational_doc: vec!["plugins/orb/skills/memo/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        },
        crate::schema::TopologyEntry {
            subsystem: "specs-substrate".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0021".into()],
            operational_doc: vec!["plugins/orb/skills/spec/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        },
        crate::schema::TopologyEntry {
            subsystem: "topology".into(),
            canonical_code: vec!["orbit-state/crates/core/src/schema.rs".into()],
            decision_record: vec!["0025".into()],
            operational_doc: vec!["plugins/orb/skills/topology/SKILL.md".into()],
            test_surface: vec!["orbit-state/crates/core/src/schema.rs".into()],
        },
    ]
}

fn graph(layout: &OrbitLayout, args: &GraphArgs) -> Result<GraphResult> {
    const VERB: &str = "graph";

    let cards = load_all_cards(layout, VERB)?;
    let forward = build_forward_edges(&cards);

    // Decide which cards to include.
    let scope: BTreeMap<String, &Card> = match &args.card {
        Some(query) => {
            validate_card_slug(VERB, query)?;
            let resolved = resolve_numeric_slug(VERB, &layout.cards_dir(), query)?
                .unwrap_or_else(|| query.clone());
            if !cards.contains_key(&resolved) {
                return Err(Error::not_found(
                    VERB,
                    format!("no card at {}", layout.card_file(&resolved).display()),
                ));
            }
            let reverse = build_reverse_edges(&cards);
            let included = bfs_card_neighbourhood(&resolved, &forward, &reverse, args.depth);
            cards
                .iter()
                .filter(|(slug, _)| included.contains(*slug))
                .map(|(s, c)| (s.clone(), c))
                .collect()
        }
        None => cards.iter().map(|(s, c)| (s.clone(), c)).collect(),
    };

    // Card → spec edges come from card.specs[]. Specs become nodes only
    // when at least one in-scope card lists them.
    let mut spec_nodes: BTreeMap<String, String> = BTreeMap::new();
    let mut card_spec_edges: Vec<(String, String)> = Vec::new();
    for (slug, card) in &scope {
        for spec_path in &card.specs {
            let spec_id = spec_id_from_listed_path(spec_path);
            spec_nodes.entry(spec_id.clone()).or_insert_with(|| spec_path.clone());
            card_spec_edges.push((slug.clone(), spec_id));
        }
    }

    let text = match args.format {
        GraphFormat::Mermaid => render_mermaid(&scope, &spec_nodes, &card_spec_edges),
        GraphFormat::Graphviz => render_graphviz(&scope, &spec_nodes, &card_spec_edges),
    };
    let format = match args.format {
        GraphFormat::Mermaid => "mermaid",
        GraphFormat::Graphviz => "graphviz",
    };
    Ok(GraphResult {
        format: format.to_string(),
        text,
    })
}

/// BFS from `root` over forward + reverse edges to gather the set of cards
/// reachable within `depth` hops in either direction. Bounded by
/// HashSet-of-visited; ignores edges to slugs absent from the loaded card
/// set (dangling references).
fn bfs_card_neighbourhood(
    root: &str,
    forward: &BTreeMap<String, Vec<(String, String, String)>>,
    reverse: &BTreeMap<String, Vec<(String, String, String)>>,
    depth: u32,
) -> std::collections::HashSet<String> {
    let mut included = std::collections::HashSet::new();
    let mut frontier: Vec<String> = vec![root.to_string()];
    included.insert(root.to_string());
    for _ in 0..depth {
        let mut next: Vec<String> = Vec::new();
        for slug in &frontier {
            if let Some(edges) = forward.get(slug) {
                for (target, _, _) in edges {
                    if included.insert(target.clone()) {
                        next.push(target.clone());
                    }
                }
            }
            if let Some(edges) = reverse.get(slug) {
                for (source, _, _) in edges {
                    if included.insert(source.clone()) {
                        next.push(source.clone());
                    }
                }
            }
        }
        if next.is_empty() {
            break;
        }
        frontier = next;
    }
    included
}

/// Sanitise a slug for use as a mermaid node id (alphanumeric + underscore).
fn mermaid_id(prefix: char, slug: &str) -> String {
    let body: String = slug
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("{prefix}_{body}")
}

fn render_mermaid(
    cards: &BTreeMap<String, &Card>,
    spec_nodes: &BTreeMap<String, String>,
    card_spec_edges: &[(String, String)],
) -> String {
    let mut out = String::from("graph LR\n");
    // Card nodes.
    for (slug, card) in cards {
        let id = mermaid_id('c', slug);
        let label = format!("{slug}: {}", card.feature);
        out.push_str(&format!("  {id}[\"{label}\"]\n", id = id, label = label.replace('"', "'")));
    }
    // Spec nodes.
    for spec_id in spec_nodes.keys() {
        let id = mermaid_id('s', spec_id);
        out.push_str(&format!("  {id}([\"{spec_id}\"])\n"));
    }
    // Card → card edges (only when both endpoints are in scope).
    for (slug, card) in cards {
        let from = mermaid_id('c', slug);
        for relation in &card.relations {
            if !cards.contains_key(&relation.card) {
                continue;
            }
            let to = mermaid_id('c', &relation.card);
            let label = relation_kind_str(&relation.kind);
            out.push_str(&format!("  {from} -->|{label}| {to}\n"));
        }
    }
    // Card → spec edges.
    for (card_slug, spec_id) in card_spec_edges {
        let from = mermaid_id('c', card_slug);
        let to = mermaid_id('s', spec_id);
        out.push_str(&format!("  {from} -.-> {to}\n"));
    }
    out
}

fn render_graphviz(
    cards: &BTreeMap<String, &Card>,
    spec_nodes: &BTreeMap<String, String>,
    card_spec_edges: &[(String, String)],
) -> String {
    let mut out = String::from("digraph orbit {\n  rankdir=LR;\n");
    for (slug, card) in cards {
        let label = format!("{slug}\\n{}", card.feature).replace('"', "\\\"");
        out.push_str(&format!("  \"{slug}\" [label=\"{label}\", shape=box];\n"));
    }
    for spec_id in spec_nodes.keys() {
        out.push_str(&format!("  \"{spec_id}\" [shape=ellipse];\n"));
    }
    for (slug, card) in cards {
        for relation in &card.relations {
            if !cards.contains_key(&relation.card) {
                continue;
            }
            let label = relation_kind_str(&relation.kind);
            out.push_str(&format!(
                "  \"{slug}\" -> \"{target}\" [label=\"{label}\"];\n",
                target = relation.card
            ));
        }
    }
    for (card_slug, spec_id) in card_spec_edges {
        out.push_str(&format!(
            "  \"{card_slug}\" -> \"{spec_id}\" [style=dashed];\n"
        ));
    }
    out.push_str("}\n");
    out
}

/// Parse the leading numeric prefix of a card slug (e.g. `"0033"` from
/// `"0033-see-the-tree"`). Used as the tie-break for `most-connected card`
/// per ac-03's pinned rule. Returns u32::MAX when no prefix is found so
/// non-numeric slugs sort last (and lose all ties).
fn numeric_prefix(slug: &str) -> u32 {
    let take: String = slug.chars().take_while(|c| c.is_ascii_digit()).collect();
    take.parse().unwrap_or(u32::MAX)
}

/// Extract the spec id from a path string as it appears in `card.specs[]`.
/// The canonical shape is `.orbit/specs/<id>/spec.yaml` (per choice 0021),
/// but legacy values may still appear as `.orbit/specs/<id>.yaml`. Both
/// resolve to `<id>`.
fn spec_id_from_listed_path(listed: &str) -> String {
    // Trim trailing `/spec.yaml` if present.
    let trimmed = listed.trim_end_matches("/spec.yaml");
    let trimmed = trimmed.trim_end_matches(".yaml");
    trimmed
        .rsplit('/')
        .next()
        .unwrap_or(trimmed)
        .to_string()
}

fn collect_card_summaries(layout: &OrbitLayout, verb: &'static str) -> Result<Vec<CardSummary>> {
    let files = layout
        .list_card_files()
        .map_err(|e| Error::unavailable(verb, format!("list cards: {e}")))?;
    let mut out = Vec::with_capacity(files.len());
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::unavailable(verb, format!("read {}: {e}", path.display())))?;
        let card: Card = parse_yaml(&text).map_err(|mut e| {
            e.verb = verb.into();
            e
        })?;
        let slug = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| Error::malformed(verb, format!("card path has no stem: {}", path.display())))?
            .to_string();
        let maturity = match card.maturity {
            crate::schema::CardMaturity::Planned => "planned",
            crate::schema::CardMaturity::Emerging => "emerging",
            crate::schema::CardMaturity::Established => "established",
        };
        out.push(CardSummary {
            slug,
            feature: card.feature,
            goal: card.goal,
            maturity: maturity.into(),
        });
    }
    out.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(out)
}

// ============================================================================
// Choice verbs (ac-10) — read-only; choices are human-written, CI-validated.
// ============================================================================

fn choice_show(layout: &OrbitLayout, args: &ChoiceShowArgs) -> Result<ChoiceShowResult> {
    const VERB: &str = "choice.show";
    if args.id.is_empty() {
        return Err(Error::malformed(VERB, "id must not be empty"));
    }
    if args.id.contains('/') || args.id.contains('\\') || args.id.contains("..") {
        return Err(Error::malformed(
            VERB,
            format!("id must not contain path separators or '..': '{}'", args.id),
        ));
    }
    let resolved = resolve_numeric_slug(VERB, &layout.choices_dir(), &args.id)?
        .unwrap_or_else(|| args.id.clone());
    let path = layout.choice_file(&resolved);
    if !path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no choice at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Error::unavailable(VERB, format!("read {}: {e}", path.display())))?;
    let choice: Choice = parse_yaml(&text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    Ok(ChoiceShowResult { choice })
}

fn choice_list(layout: &OrbitLayout, args: &ChoiceListArgs) -> Result<ChoiceListResult> {
    const VERB: &str = "choice.list";
    if let Some(s) = args.status.as_deref() {
        if !matches!(s, "proposed" | "accepted" | "rejected" | "deprecated" | "superseded") {
            return Err(Error::malformed(
                VERB,
                format!(
                    "status must be proposed|accepted|rejected|deprecated|superseded, got '{s}'"
                ),
            ));
        }
    }
    let summaries = collect_choice_summaries(layout, VERB)?;
    let filtered = match &args.status {
        Some(s) => summaries.into_iter().filter(|c| c.status == *s).collect(),
        None => summaries,
    };
    Ok(ChoiceListResult { choices: filtered })
}

fn choice_search(layout: &OrbitLayout, args: &ChoiceSearchArgs) -> Result<ChoiceListResult> {
    const VERB: &str = "choice.search";
    if args.query.is_empty() {
        return Err(Error::malformed(VERB, "query must not be empty"));
    }
    let needle = args.query.to_lowercase();
    // Search hits title or body, so we must read full Choice (not just summary).
    let files = layout
        .list_choice_files()
        .map_err(|e| Error::unavailable(VERB, format!("list choices: {e}")))?;
    let mut matched = Vec::new();
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::unavailable(VERB, format!("read {}: {e}", path.display())))?;
        let choice: Choice = parse_yaml(&text).map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;
        if choice.title.to_lowercase().contains(&needle)
            || choice.body.to_lowercase().contains(&needle)
        {
            matched.push(ChoiceSummary {
                id: choice.id,
                title: choice.title,
                status: choice_status_str(&choice.status).into(),
                date_created: choice.date_created,
            });
        }
    }
    matched.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(ChoiceListResult { choices: matched })
}

fn collect_choice_summaries(
    layout: &OrbitLayout,
    verb: &'static str,
) -> Result<Vec<ChoiceSummary>> {
    let files = layout
        .list_choice_files()
        .map_err(|e| Error::unavailable(verb, format!("list choices: {e}")))?;
    let mut out = Vec::with_capacity(files.len());
    for path in files {
        let text = std::fs::read_to_string(&path)
            .map_err(|e| Error::unavailable(verb, format!("read {}: {e}", path.display())))?;
        let choice: Choice = parse_yaml(&text).map_err(|mut e| {
            e.verb = verb.into();
            e
        })?;
        out.push(ChoiceSummary {
            id: choice.id,
            title: choice.title,
            status: choice_status_str(&choice.status).into(),
            date_created: choice.date_created,
        });
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

fn choice_status_str(s: &crate::schema::ChoiceStatus) -> &'static str {
    use crate::schema::ChoiceStatus::*;
    match s {
        Proposed => "proposed",
        Accepted => "accepted",
        Rejected => "rejected",
        Deprecated => "deprecated",
        Superseded => "superseded",
    }
}

// ============================================================================
// Session verb (ac-11)
// ============================================================================

/// `session.prime` — agent priming context with bounded output.
///
/// Returns:
/// - All open specs (summaries — id/goal/status/cards/labels)
/// - Up to K memories (default K=10), most recent first
/// - The item bound formula's value, for caller diagnostics
fn session_prime(layout: &OrbitLayout, args: &SessionPrimeArgs) -> Result<SessionPrimeResult> {
    const VERB: &str = "session.prime";
    const DEFAULT_MEMORY_CAP: usize = 10;
    let cap = args.memory_cap.unwrap_or(DEFAULT_MEMORY_CAP);

    // Open specs.
    let SpecListResult { specs: all_specs } =
        spec_list(layout, &SpecListArgs::default())?;
    let open_specs: Vec<SpecSummary> = all_specs
        .into_iter()
        .filter(|s| s.status == "open")
        .collect();

    // Per spec 2026-05-15-agent-learning-loop ac-06: when at least one open
    // spec has a non-empty `labels` field, sort memories first by label-
    // overlap with open-spec labels (descending), then by timestamp DESC,
    // then truncate to cap. When no open spec has labels, sort by timestamp
    // alone (the previous behaviour).
    let open_spec_labels: BTreeSet<String> = open_specs
        .iter()
        .flat_map(|s| s.labels.iter().cloned())
        .collect();
    let use_overlap_sort = !open_spec_labels.is_empty();

    let mut memories = read_all_memories(layout, VERB)?;
    if use_overlap_sort {
        memories.sort_by(|a, b| {
            let a_overlap = a.labels.iter().filter(|l| open_spec_labels.contains(*l)).count();
            let b_overlap = b.labels.iter().filter(|l| open_spec_labels.contains(*l)).count();
            // Higher overlap first; tie-break on timestamp DESC.
            b_overlap.cmp(&a_overlap).then_with(|| b.timestamp.cmp(&a.timestamp))
        });
    } else {
        memories.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    }
    let effective = cap.min(memories.len());
    memories.truncate(effective);

    // Spec 2026-05-16-session-handover ac-07: surface the most-recent
    // Session globally (no card filter at prime — per-card lookup is via
    // `orbit session handover --card <id>`).
    let handover = session_handover(layout, &SessionHandoverArgs::default())?.handover;

    let mut item_bound = 40 + 2 * open_specs.len() + cap.min(DEFAULT_MEMORY_CAP);
    if handover.is_some() {
        item_bound += 1;
    }

    const HANDOVER_PREFIX: &str = "Read the handover above before any other action. ";
    let base_next_step = "Run `orbit overview` for a single-screen project synthesis (open specs, cards-by-maturity, recent memories, most-connected card, orphans).";
    let next_step = if handover.is_some() {
        format!("{HANDOVER_PREFIX}{base_next_step}")
    } else {
        base_next_step.to_string()
    };

    // Topology drift surface (spec 2026-05-18-topology-substrate-wires
    // ac-02). The audit returns `configured: false` when `.orbit/config.yaml`
    // is absent OR when `docs.topology` is unset — both cases collapse to
    // None on the envelope side (skip-on-default). When configured, Some
    // is populated even on the clean path (empty vec → empty array in
    // the envelope, which is the agreed shape distinguishing
    // configured-clean from not-configured).
    let topology_audit = audit_topology(layout, &AuditTopologyArgs::default())?;
    let topology_drift = if topology_audit.configured {
        Some(topology_audit.topology_drift)
    } else {
        None
    };

    Ok(SessionPrimeResult {
        open_specs,
        memories,
        handover,
        item_bound,
        next_step,
        topology_drift,
    })
}

/// `session.start` — generate a session id (UUIDv4 by default) and write it
/// to `.orbit/.session-id` atomically. When `id` is supplied (test fixtures,
/// replay scenarios) it is used verbatim instead.
fn session_start(layout: &OrbitLayout, args: &SessionStartArgs) -> Result<SessionStartResult> {
    const VERB: &str = "session.start";

    let session_id = match &args.id {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(Error::malformed(VERB, "id must not be empty when supplied"));
            }
            if trimmed.contains('\n') || trimmed.contains('\r') {
                return Err(Error::malformed(
                    VERB,
                    "id must not contain newline characters",
                ));
            }
            trimmed.to_string()
        }
        None => uuid::Uuid::new_v4().to_string(),
    };

    layout
        .ensure_dirs()
        .map_err(|e| Error::unavailable(VERB, format!("ensure dirs: {e}")))?;

    let path = layout.session_id_file();
    let mut contents = session_id.clone();
    contents.push('\n');
    write_atomic(&path, contents.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SessionStartResult {
        session_id,
        path: path.display().to_string(),
    })
}

/// `session.distill` — write or update `.orbit/sessions/<id>.yaml` keyed by
/// session id. Idempotent: re-running on the same id preserves `started_at`
/// and advances `ended_at`.
fn session_distill(
    layout: &OrbitLayout,
    args: &SessionDistillArgs,
) -> Result<SessionDistillResult> {
    const VERB: &str = "session.distill";

    let session_id = match args.session_id.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(Error::malformed(VERB, "session_id must not be empty"));
            }
            trimmed.to_string()
        }
        None => read_session_id(layout, VERB)?,
    };

    validate_session_id(VERB, &session_id)?;

    if args.distillate.is_empty() {
        return Err(Error::malformed(VERB, "distillate must not be empty"));
    }

    let lock_key = format!("session-{}", session_id);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    std::fs::create_dir_all(layout.sessions_dir())
        .map_err(|e| Error::unavailable(VERB, format!("ensure sessions dir: {e}")))?;

    let now = current_rfc3339_utc().map_err(|e| {
        Error::unavailable(VERB, format!("substrate timestamp generation failed: {e}"))
    })?;
    let path = layout.session_file(&session_id);

    let started_at = if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        let existing: Session = parse_yaml(&text).map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;
        existing.started_at
    } else {
        now.clone()
    };

    // Spec 2026-05-16-session-handover ac-03: card_id resolution precedence
    // is explicit arg first, else `.orbit/.session-card` fallback, else None.
    // No validation here — validation lives at `session.set-card` time so the
    // hot path stays cheap. Idempotent latest-write-wins matches the rest of
    // the distill contract.
    let card_id = match args.card_id.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                read_session_card(layout, VERB)?
            } else {
                Some(trimmed.to_string())
            }
        }
        None => read_session_card(layout, VERB)?,
    };

    let session = Session {
        id: session_id,
        started_at,
        ended_at: Some(now),
        distillate: args.distillate.clone(),
        card_id,
        labels: args.labels.clone(),
    };
    let yaml = serialise_yaml(&session).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    write_atomic(&path, yaml.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SessionDistillResult { session })
}

/// `session.set-card` — validate `args.card_id` against the card-lookup
/// prefix-match helper, then write the resolved canonical slug atomically
/// to `.orbit/.session-card`. On unknown card, returns `Error::not_found`
/// and writes nothing. See spec 2026-05-16-session-handover ac-04.
fn session_set_card(
    layout: &OrbitLayout,
    args: &SessionSetCardArgs,
) -> Result<SessionSetCardResult> {
    const VERB: &str = "session.set-card";

    let raw = args.card_id.trim();
    if raw.is_empty() {
        return Err(Error::malformed(VERB, "card_id must not be empty"));
    }
    validate_card_slug(VERB, raw)?;

    // Resolve the slug. resolve_numeric_slug handles bare/padded numeric;
    // a full slug like "0036-session-handover" requires the literal-file
    // existence check below.
    let resolved = match resolve_numeric_slug(VERB, &layout.cards_dir(), raw)? {
        Some(slug) => slug,
        None => raw.to_string(),
    };
    let path = layout.card_file(&resolved);
    if !path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no card matching `{raw}` (looked for {})", path.display()),
        ));
    }

    layout.ensure_dirs().map_err(|e| {
        Error::unavailable(VERB, format!("ensure dirs: {e}"))
    })?;

    let card_path = layout.session_card_file();
    let mut contents = resolved.clone();
    contents.push('\n');
    write_atomic(&card_path, contents.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SessionSetCardResult {
        card_id: resolved,
        path: card_path.display().to_string(),
    })
}

/// `session.handover` — walk `.orbit/sessions/*.yaml`, filter by `card_id`
/// (when provided) and `started_at >= since` (when provided), and return
/// the session with the maximum `started_at`. Returns `handover: None`
/// when no match — querying for an unrecorded card is a legitimate question
/// per the `skill.recurrence` precedent. See spec 2026-05-16-session-handover
/// ac-06.
fn session_handover(
    layout: &OrbitLayout,
    args: &SessionHandoverArgs,
) -> Result<SessionHandoverResult> {
    const VERB: &str = "session.handover";

    // Resolve a positional/long card id via the same prefix-match helper as
    // session.set-card so the operator can write `--card 36` or `--card 0036`
    // or the full slug.
    let resolved_card = match args.card_id.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                None
            } else {
                validate_card_slug(VERB, trimmed)?;
                let slug =
                    match resolve_numeric_slug(VERB, &layout.cards_dir(), trimmed)? {
                        Some(s) => s,
                        None => trimmed.to_string(),
                    };
                let card_path = layout.card_file(&slug);
                if !card_path.exists() {
                    return Err(Error::not_found(
                        VERB,
                        format!(
                            "no card matching `{trimmed}` (looked for {})",
                            card_path.display()
                        ),
                    ));
                }
                Some(slug)
            }
        }
        None => None,
    };

    let since = args.since.as_deref().map(str::trim).filter(|s| !s.is_empty());

    let dir = layout.sessions_dir();
    let entries = match std::fs::read_dir(&dir) {
        Ok(it) => it,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(SessionHandoverResult { handover: None });
        }
        Err(e) => {
            return Err(Error::unavailable(
                VERB,
                format!("read {}: {e}", dir.display()),
            ));
        }
    };

    let mut best: Option<Session> = None;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("yaml") {
            continue;
        }
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        let session: Session = parse_yaml(&text).map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;

        if let Some(card) = &resolved_card {
            match &session.card_id {
                Some(c) if c == card => {}
                _ => continue,
            }
        }
        if let Some(s) = since {
            if session.started_at.as_str() < s {
                continue;
            }
        }
        match &best {
            Some(b) if b.started_at >= session.started_at => {}
            _ => best = Some(session),
        }
    }

    let handover = best.map(|s| HandoverSummary {
        session_id: s.id,
        card_id: s.card_id,
        started_at: s.started_at,
        ended_at: s.ended_at,
        distillate: s.distillate,
    });
    Ok(SessionHandoverResult { handover })
}

/// `skill.record-invocation` — append one row to
/// `.orbit/skills/<skill_id>.invocations.jsonl`.
fn skill_record_invocation(
    layout: &OrbitLayout,
    args: &SkillRecordInvocationArgs,
) -> Result<SkillRecordInvocationResult> {
    const VERB: &str = "skill.record-invocation";

    validate_skill_id(VERB, &args.skill_id)?;

    let outcome = parse_invocation_outcome(VERB, &args.outcome)?;

    let session_id = match args.session_id.as_deref() {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                return Err(Error::malformed(VERB, "session_id must not be empty"));
            }
            trimmed.to_string()
        }
        None => read_session_id(layout, VERB)?,
    };

    let correction = match args.correction.as_deref() {
        Some(s) if s.is_empty() => None,
        Some(s) => Some(s.to_string()),
        None => None,
    };

    let timestamp = match &args.timestamp {
        Some(t) => t.clone(),
        None => current_rfc3339_utc().map_err(|e| {
            Error::unavailable(VERB, format!("substrate timestamp generation failed: {e}"))
        })?,
    };

    let invocation = SkillInvocation {
        skill_id: args.skill_id.clone(),
        session_id,
        outcome,
        correction,
        timestamp,
    };

    let lock_key = format!("skill-{}", args.skill_id);
    let _guard = locks::acquire_default(layout, &lock_key).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    std::fs::create_dir_all(layout.skills_dir())
        .map_err(|e| Error::unavailable(VERB, format!("ensure skills dir: {e}")))?;

    let line = serialise_json_line(&invocation).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    let path = layout.skill_invocations_file(&args.skill_id);
    append_jsonl_line(&path, &line).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(SkillRecordInvocationResult { invocation })
}

/// `skill.recurrence` — read the per-skill invocation stream and bucket rows
/// by outcome. Returns an empty-shape response when the file is absent.
fn skill_recurrence(
    layout: &OrbitLayout,
    args: &SkillRecurrenceArgs,
) -> Result<SkillRecurrenceResult> {
    const VERB: &str = "skill.recurrence";

    validate_skill_id(VERB, &args.skill_id)?;

    let path = layout.skill_invocations_file(&args.skill_id);
    let mut by_outcome = RecurrenceByOutcome::default();
    let mut total = 0usize;

    if path.exists() {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(VERB, format!("read {}: {e}", path.display()))
        })?;
        for (lineno, raw) in text.lines().enumerate() {
            if raw.is_empty() {
                continue;
            }
            let invocation: SkillInvocation = parse_json_line(raw).map_err(|mut e| {
                e.verb = VERB.into();
                e.message = format!("{} (line {})", e.message, lineno + 1);
                e
            })?;
            if let Some(cutoff) = args.since.as_deref() {
                if invocation.timestamp.as_str() < cutoff {
                    continue;
                }
            }
            total += 1;
            let bucket = match invocation.outcome {
                InvocationOutcome::Worked => &mut by_outcome.worked,
                InvocationOutcome::Partial => &mut by_outcome.partial,
                InvocationOutcome::DidntApply => &mut by_outcome.didnt_apply,
                InvocationOutcome::Incorrect => &mut by_outcome.incorrect,
            };
            bucket.count += 1;
            bucket.invocations.push(RecurrenceInvocation {
                timestamp: invocation.timestamp,
                correction: invocation.correction,
            });
        }
    }

    Ok(SkillRecurrenceResult {
        skill_id: args.skill_id.clone(),
        by_outcome,
        total,
    })
}

// ============================================================================
// Routine verbs — per spec 2026-05-22-routine-proposals.
//
// AC-08 boundary: this section explicitly does NOT import any "skill author"
// module — card 0022's skill-author flow is in spec/verb space, not here.
// The boundary is enforced by `routine_no_cross_imports_with_skill_author`
// in the test module below: it greps both routine.rs and this section for
// any `skill_author` reference and asserts none exist.
// ============================================================================

/// `routine.chains` — reconstruct per-session chains from the
/// SkillInvocation substrate. Per ac-01 (option b: aggregator verb,
/// no schema change).
fn routine_chains(
    layout: &OrbitLayout,
    _args: &RoutineChainsArgs,
) -> Result<RoutineChainsResult> {
    let chains = crate::routine::reconstruct_chains(&layout.skills_dir())?;
    Ok(RoutineChainsResult { chains })
}

/// `routine.detect` — surface recurring sequential chains at or above
/// the threshold. v1 is sequential-only (ac-05) so DAG-shaped patterns
/// are recorded in the invocation stream but never returned here.
fn routine_detect(
    layout: &OrbitLayout,
    _args: &RoutineDetectArgs,
) -> Result<RoutineDetectResult> {
    let chains = crate::routine::reconstruct_chains(&layout.skills_dir())?;
    let recurring = crate::routine::detect_recurring_chains(&chains);
    Ok(RoutineDetectResult { recurring })
}

/// `routine.author` — write a routine SKILL.md at
/// `.claude/skills/<name>/SKILL.md` carrying validated front-matter
/// (created_by, created_at, pinned, last_verified, chain_id, chain).
/// Per ac-03 + ac-04.
///
/// Content-addressed idempotency (ac-09): if a routine with the same
/// `chain_id` already exists under `.claude/skills/` or its `.archive/`
/// subtree the verb returns `written: false` and points at the
/// existing file. Author archives permanently silence re-authoring.
///
/// The verb validates the front-matter against [`RoutineFrontMatter`]
/// before writing — schema-validation failure surfaces as a Malformed
/// error so the agent can react per the tabletop's halt condition #1.
fn routine_author(
    layout: &OrbitLayout,
    args: &RoutineAuthorArgs,
) -> Result<RoutineAuthorResult> {
    const VERB: &str = "routine.author";

    if args.chain.len() < 2 {
        return Err(Error::malformed(
            VERB,
            "chain must contain ≥ 2 skill_ids (single-skill routines aren't chains)",
        ));
    }

    let chain_id = crate::routine::chain_id(&args.chain);
    let name = args
        .name
        .clone()
        .unwrap_or_else(|| crate::routine::default_routine_name(&args.chain));
    if name.is_empty() {
        return Err(Error::malformed(VERB, "routine name must not be empty"));
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(Error::malformed(
            VERB,
            format!("name must not contain path separators or '..': '{name}'"),
        ));
    }

    let claude_dir = layout.claude_skills_dir();

    // AC-09: content-addressed dedupe across .claude/skills/ AND .archive/.
    if let Some(existing) = crate::routine::existing_routine_for_chain(&claude_dir, &chain_id)? {
        return Ok(RoutineAuthorResult {
            written: false,
            path: existing.to_string_lossy().into_owned(),
            chain_id,
            name,
        });
    }

    let timestamp = match &args.timestamp {
        Some(t) => t.clone(),
        None => current_rfc3339_utc().map_err(|e| {
            Error::unavailable(VERB, format!("substrate timestamp generation failed: {e}"))
        })?,
    };

    let description = args.description.clone().unwrap_or_else(|| {
        format!(
            "Routine chain: {}",
            args.chain.join(" → "),
        )
    });

    let body = args.body.clone().unwrap_or_else(|| default_routine_body(&args.chain));

    let fm = crate::routine::RoutineFrontMatter {
        name: name.clone(),
        description,
        created_by: "agent".into(),
        created_at: timestamp.clone(),
        pinned: false,
        last_verified: timestamp,
        chain_id: chain_id.clone(),
        chain: args.chain.clone(),
    };

    let rendered = crate::routine::render_skill_md(&fm, &body);

    // Re-parse what we just rendered — AC-04 validates the front-matter
    // schema on every agent write. If the schema check ever fails on a
    // freshly-rendered payload, the rendering itself is broken: surface
    // as Malformed so the agent halts (tabletop halt condition #1).
    crate::routine::parse_front_matter(&rendered).map_err(|e| {
        Error::malformed(VERB, format!("rendered front-matter failed validation: {}", e.0))
    })?;

    let target_dir = claude_dir.join(&name);
    std::fs::create_dir_all(&target_dir).map_err(|e| {
        Error::unavailable(VERB, format!("create {}: {e}", target_dir.display()))
    })?;
    let target_path = target_dir.join("SKILL.md");
    write_atomic(&target_path, rendered.as_bytes()).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;

    Ok(RoutineAuthorResult {
        written: true,
        path: target_path.to_string_lossy().into_owned(),
        chain_id,
        name,
    })
}

/// `routine.verify` — re-validate every `/orb:<verb>` reference in the
/// routine's SKILL.md body resolves to a live skill, and on pass write
/// the run timestamp to `last_verified`. Per ac-06.
///
/// Atomic semantics (per ac-06 verification): read the file, mutate
/// the front-matter in memory, atomic rewrite. Concurrent
/// `audit.conformance` + verify can't corrupt because
/// `audit.conformance` doesn't write.
///
/// Resolution surface: a `/orb:<verb>` reference resolves when
/// `<repo_root>/plugins/orb/skills/<verb>/SKILL.md` exists OR
/// `<repo_root>/.claude/skills/<verb>/SKILL.md` exists. The plugins/
/// path is the orbit-owned skill set; `.claude/skills/` is the
/// project-local agent-authored set (where routines themselves live).
fn routine_verify(
    layout: &OrbitLayout,
    args: &RoutineVerifyArgs,
) -> Result<RoutineVerifyResult> {
    const VERB: &str = "routine.verify";

    if args.path.is_empty() {
        return Err(Error::malformed(VERB, "path must not be empty"));
    }
    if args.path.contains("..") {
        return Err(Error::malformed(
            VERB,
            format!("path must not contain '..': '{}'", args.path),
        ));
    }

    // Resolve `path` relative to the repo root when it isn't absolute.
    let path_buf = std::path::PathBuf::from(&args.path);
    let resolved_path = if path_buf.is_absolute() {
        path_buf
    } else {
        layout.repo_root().join(&path_buf)
    };
    if !resolved_path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no SKILL.md at {}", resolved_path.display()),
        ));
    }

    let body = std::fs::read_to_string(&resolved_path).map_err(|e| {
        Error::unavailable(VERB, format!("read {}: {e}", resolved_path.display()))
    })?;

    // Parse + validate front-matter to enforce the AC-04 invariant on every
    // write path that touches the file.
    let mut fm = crate::routine::parse_front_matter(&body).map_err(|e| {
        Error::malformed(VERB, e.0)
    })?;

    // Walk the body (everything after the closing `---`) and pull out
    // every `/orb:<verb>` reference. We also consult `fm.chain` so the
    // declarative chain is verified even if the body prose drifted.
    let prose_refs = extract_orb_refs(&body);
    let mut all_refs: Vec<String> = fm.chain.clone();
    for r in prose_refs {
        if !all_refs.contains(&r) {
            all_refs.push(r);
        }
    }

    let (resolved, broken_refs) = partition_refs(layout, &all_refs);

    let timestamp = match &args.timestamp {
        Some(t) => t.clone(),
        None => current_rfc3339_utc().map_err(|e| {
            Error::unavailable(VERB, format!("substrate timestamp generation failed: {e}"))
        })?,
    };

    let last_verified = if broken_refs.is_empty() {
        // Pass: advance last_verified via atomic rewrite. Read again
        // before write to preserve any body edits the author made
        // since parse.
        fm.last_verified = timestamp.clone();
        // Extract the post-front-matter body and rewrite the SKILL.md
        // with the new front-matter block.
        let body_after = split_off_body(&body).unwrap_or_default();
        let rewritten = crate::routine::render_skill_md(&fm, body_after);
        write_atomic(&resolved_path, rewritten.as_bytes()).map_err(|mut e| {
            e.verb = VERB.into();
            e
        })?;
        Some(timestamp)
    } else {
        None
    };

    Ok(RoutineVerifyResult {
        path: resolved_path.to_string_lossy().into_owned(),
        resolved,
        broken_refs,
        last_verified,
    })
}

/// Walk `body` and return every `/orb:<verb>` reference, in
/// first-occurrence order. Used by [`routine_verify`] to extract the
/// references whose live-skill status determines `last_verified`.
fn extract_orb_refs(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut chars = body.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '/' {
            continue;
        }
        // Match `/orb:`...
        let rest = &body[i..];
        if !rest.starts_with("/orb:") {
            continue;
        }
        let after = &rest[5..];
        let end = after
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-' || c == '_'))
            .unwrap_or(after.len());
        if end == 0 {
            continue;
        }
        let verb = &after[..end];
        let full = format!("/orb:{verb}");
        if !out.contains(&full) {
            out.push(full);
        }
        // Consume the matched chars so the outer iterator advances.
        for _ in 0..(5 + end - 1) {
            chars.next();
        }
    }
    out
}

/// Split `body` into front-matter and after-block. Returns the slice
/// AFTER the closing `---\n`. Returns `None` when the front-matter
/// markers are not present (which can't happen for a body that
/// [`crate::routine::parse_front_matter`] just accepted).
fn split_off_body(body: &str) -> Option<&str> {
    let trimmed = body.trim_start_matches('\u{feff}');
    let after_open = trimmed.strip_prefix("---\n")?;
    let close_offset = after_open.find("\n---")?;
    let after_close = &after_open[close_offset + 4..]; // skip `\n---`
    // The closing marker is `\n---` followed by `\n` and the body.
    Some(after_close.strip_prefix('\n').unwrap_or(after_close))
}

/// Partition a list of `/orb:<verb>` refs into (resolved, broken).
/// A ref resolves when either `plugins/orb/skills/<verb>/SKILL.md` or
/// `.claude/skills/<verb>/SKILL.md` exists.
fn partition_refs(layout: &OrbitLayout, refs: &[String]) -> (Vec<String>, Vec<String>) {
    let mut resolved = Vec::new();
    let mut broken = Vec::new();
    let plugin_root = layout.repo_root().join("plugins").join("orb").join("skills");
    let claude_root = layout.claude_skills_dir();
    for r in refs {
        let verb = r.strip_prefix("/orb:").unwrap_or(r);
        let plugin_path = plugin_root.join(verb).join("SKILL.md");
        let claude_path = claude_root.join(verb).join("SKILL.md");
        if plugin_path.is_file() || claude_path.is_file() {
            resolved.push(r.clone());
        } else {
            broken.push(r.clone());
        }
    }
    (resolved, broken)
}

/// Default body the substrate emits when `routine.author` is called
/// without a `body` arg. Lists the chain steps and points at the
/// verify verb so authors know how to refresh `last_verified`.
fn default_routine_body(chain: &[String]) -> String {
    let mut out = String::from("# Routine\n\n");
    out.push_str("Run the following skill chain in order:\n\n");
    for step in chain {
        out.push_str(&format!("1. `{}`\n", step));
    }
    out.push_str(
        "\n---\n\nGenerated by `orbit routine author` — re-verify with `orbit routine verify <path>`.\n",
    );
    out
}

fn parse_invocation_outcome(verb: &str, raw: &str) -> Result<InvocationOutcome> {
    match raw {
        "worked" => Ok(InvocationOutcome::Worked),
        "partial" => Ok(InvocationOutcome::Partial),
        "didnt-apply" => Ok(InvocationOutcome::DidntApply),
        "incorrect" => Ok(InvocationOutcome::Incorrect),
        other => Err(Error::malformed(
            verb,
            format!(
                "outcome must be one of 'worked', 'partial', 'didnt-apply', 'incorrect'; got '{other}'"
            ),
        )),
    }
}

fn validate_skill_id(verb: &str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(Error::malformed(verb, "skill_id must not be empty"));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("skill_id must not contain path separators or '..': '{id}'"),
        ));
    }
    Ok(())
}

fn validate_session_id(verb: &str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(Error::malformed(verb, "session_id must not be empty"));
    }
    if id.contains('/') || id.contains('\\') || id.contains("..") {
        return Err(Error::malformed(
            verb,
            format!("session_id must not contain path separators or '..': '{id}'"),
        ));
    }
    Ok(())
}

/// Generate an RFC 3339 UTC timestamp. The substrate's default clock for
/// any verb that needs to stamp an event.
fn current_rfc3339_utc() -> std::result::Result<String, time::error::Format> {
    OffsetDateTime::now_utc().format(&Rfc3339)
}

/// `spec.show` — read the spec at `<id>.yaml`, parse, return.
///
/// NotFound when the file doesn't exist; Malformed if it parses badly.
fn spec_show(layout: &OrbitLayout, args: &SpecShowArgs) -> Result<SpecShowResult> {
    const VERB: &str = "spec.show";

    if args.id.is_empty() {
        return Err(Error::malformed(VERB, "id must not be empty"));
    }
    // Defensive: reject ids that contain path separators. Spec ids are slug-
    // shaped and the layout already enforces .yaml extension; a `..` or `/`
    // would let a caller read arbitrary YAML files in the workspace.
    if args.id.contains('/') || args.id.contains('\\') || args.id.contains("..") {
        return Err(Error::malformed(
            VERB,
            format!("id must not contain path separators or '..': '{}'", args.id),
        ));
    }

    let path = layout.spec_file(&args.id);
    if !path.exists() {
        return Err(Error::not_found(
            VERB,
            format!("no spec at {}", path.display()),
        ));
    }
    let text = std::fs::read_to_string(&path).map_err(|e| {
        Error::unavailable(VERB, format!("read {}: {e}", path.display()))
    })?;
    let spec: Spec = parse_yaml(&text).map_err(|mut e| {
        e.verb = VERB.into();
        e
    })?;
    Ok(SpecShowResult { spec })
}

/// `spec.resolve` — return the spec id a skill should act on when no
/// explicit id was passed. Implements the three-step recovery (infer →
/// prompt → halt) from spec
/// `2026-05-19-skills-infer-or-prompt-before-halt`.
///
/// Resolution order:
///
/// 1. **Read the card binding.** If `args.card` is set, use it. Otherwise
///    read `.orbit/.session-card`. The card slug — if any — narrows the
///    open-spec pool to the specs that list this card in their
///    `cards` array.
/// 2. **Enumerate open specs.** If a card binding is in force, scope the
///    list to specs where `Spec.cards` contains the resolved card slug.
///    Otherwise list every open spec project-wide.
/// 3. **Branch on count.**
///    - **Exactly one** → `Resolved { id, source }`.
///    - **Multiple** → `Prompt { candidates, source }`.
///    - **Zero** → `Error::unavailable` with one of the two D5 halt-
///      message templates: a card-narrowed halt names the bound card and
///      tells the user how to create a spec under it; an unscoped halt
///      names the lack of both fallbacks.
///
/// The skill prose owns the AskUserQuestion call — this verb only
/// returns the menu data structurally. Per decision D1, the prose is
/// fixed in the verb so all skills emit the same halt message.
fn spec_resolve(layout: &OrbitLayout, args: &SpecResolveArgs) -> Result<SpecResolveResult> {
    const VERB: &str = "spec.resolve";

    let skill_label = args
        .skill
        .as_deref()
        .map(|s| format!("/orb:{s}"))
        .unwrap_or_else(|| "/orb:<skill>".to_string());

    // Card binding: explicit `--card` arg beats `.session-card`. An empty
    // string arg is treated as malformed so callers never accidentally
    // request "the bound card and just kidding, no card" semantics.
    if let Some(c) = args.card.as_deref() {
        if c.is_empty() {
            return Err(Error::malformed(VERB, "card must not be empty when provided"));
        }
        if c.contains('/') || c.contains('\\') || c.contains("..") {
            return Err(Error::malformed(
                VERB,
                format!("card must not contain path separators or '..': '{c}'"),
            ));
        }
    }

    let bound_card_source = if args.card.is_some() {
        Some(BoundCardSource::Arg)
    } else {
        match read_session_card(layout, VERB)? {
            Some(_) => Some(BoundCardSource::Session),
            None => None,
        }
    };

    let bound_card_slug: Option<String> = match bound_card_source {
        Some(BoundCardSource::Arg) => args.card.clone(),
        Some(BoundCardSource::Session) => read_session_card(layout, VERB)?,
        None => None,
    };

    // Resolve the card slug to its canonical form (matches card.show
    // semantics: bare unpadded number, padded NNNN, or full slug all work).
    let resolved_card: Option<String> = if let Some(slug) = bound_card_slug.as_deref() {
        let cards_dir = layout.cards_dir();
        match resolve_numeric_slug(VERB, &cards_dir, slug)? {
            Some(canonical) => Some(canonical),
            None => Some(slug.to_string()),
        }
    } else {
        None
    };

    // Enumerate every open spec, optionally narrowed by the bound card.
    let open_specs = enumerate_open_specs(layout, VERB)?;
    let candidates: Vec<&OpenSpec> = match resolved_card.as_deref() {
        Some(card_slug) => open_specs
            .iter()
            .filter(|s| s.cards.iter().any(|c| c == card_slug))
            .collect(),
        None => open_specs.iter().collect(),
    };

    match candidates.len() {
        1 => {
            let s = candidates[0];
            let source = match bound_card_source {
                Some(BoundCardSource::Arg) => "card_arg",
                Some(BoundCardSource::Session) => "bound_card",
                None => "single_open",
            };
            Ok(SpecResolveResult::Resolved {
                id: s.id.clone(),
                source: source.to_string(),
            })
        }
        n if n >= 2 => {
            let mut cands: Vec<SpecResolveCandidate> = candidates
                .iter()
                .map(|s| SpecResolveCandidate {
                    id: s.id.clone(),
                    goal_first_line: first_line(&s.goal).to_string(),
                })
                .collect();
            cands.sort_by(|a, b| a.id.cmp(&b.id));
            let source = match bound_card_source {
                Some(BoundCardSource::Arg) => "card_arg_multi",
                Some(BoundCardSource::Session) => "bound_card_multi",
                None => "unbound",
            };
            Ok(SpecResolveResult::Prompt {
                candidates: cands,
                source: source.to_string(),
            })
        }
        _ => {
            // Zero candidates: emit one of the two canonical D5 halt
            // templates as the `unavailable` message. Skills surface the
            // string verbatim — never paraphrase or wrap with prose.
            let message = match resolved_card.as_deref() {
                Some(card_slug) => {
                    // Recoverable: a card is bound but has no open specs.
                    // The user can either create one under this card or
                    // rebind to another.
                    format!(
                        "no open spec under the bound card {card_slug}. Create one with /orb:spec, or rebind with orbit session set-card <id>."
                    )
                }
                None => {
                    // Terminal: nothing bound AND no open specs.
                    format!(
                        "no spec to act on for {skill_label} — both fallbacks failed (.session-card is unbound and no open specs exist). Create one with /orb:spec."
                    )
                }
            };
            Err(Error::unavailable(VERB, message))
        }
    }
}

/// Tiny enum tracking where the card binding came from, so the resolver
/// can label `Resolved.source` / `Prompt.source` precisely without
/// re-reading the env later.
enum BoundCardSource {
    /// Explicit `--card` argument on the CLI.
    Arg,
    /// `.orbit/.session-card` file.
    Session,
}

/// Minimal projection of an open spec — id, goal, cards. Built once by
/// `enumerate_open_specs` so the resolver can filter and count without
/// re-reading the file system.
struct OpenSpec {
    id: String,
    goal: String,
    cards: Vec<String>,
}

/// Walk `.orbit/specs/<id>/spec.yaml`, parse each, return the open ones.
/// Shares parse semantics with `spec_list` but returns the leaner
/// projection the resolver needs.
fn enumerate_open_specs(layout: &OrbitLayout, verb: &str) -> Result<Vec<OpenSpec>> {
    let files = layout
        .list_spec_files()
        .map_err(|e| Error::unavailable(verb, format!("list specs dir: {e}")))?;

    let mut out = Vec::with_capacity(files.len());
    for path in files {
        let text = std::fs::read_to_string(&path).map_err(|e| {
            Error::unavailable(verb, format!("read {}: {e}", path.display()))
        })?;
        let spec: Spec = parse_yaml(&text).map_err(|mut e| {
            e.verb = verb.into();
            e
        })?;
        if matches!(spec.status, SpecStatus::Open) {
            out.push(OpenSpec {
                id: spec.id,
                goal: spec.goal,
                cards: spec.cards,
            });
        }
    }
    Ok(out)
}

/// First newline-bounded line of a string. Empty input yields empty
/// string. Mirrors the CLI's `first_line` helper at `cli/src/main.rs`.
fn first_line(s: &str) -> &str {
    s.lines().next().unwrap_or("")
}

// ============================================================================
// Wire envelope
// ============================================================================
//
// Both CLI (`--json` mode) and MCP (`tools/call` response payload) emit the
// same envelope shape so byte-equal output falls out for free:
//
//   ok  : {"data":<verb-response>,"ok":true}
//   err : {"error":{"category":"<cat>","message":"<msg>","verb":"<verb>"},"ok":false}
//
// serde_json sorts object keys alphabetically by default, so the exact byte
// layout is deterministic across both surfaces. Inner struct fields preserve
// declaration order via the Serialize derive.

/// Build the OK envelope as a JSON [`Value`]. Callers stringify via
/// [`serde_json::to_string`] when they want bytes.
pub fn envelope_ok<T: Serialize>(data: &T) -> Value {
    json!({ "ok": true, "data": data })
}

/// Build the error envelope as a JSON [`Value`].
pub fn envelope_err(err: &Error) -> Value {
    json!({
        "ok": false,
        "error": {
            "verb": err.verb,
            "category": err.category.as_str(),
            "message": err.message,
        }
    })
}

/// Convenience: stringify the OK envelope. Returns the canonical wire bytes
/// as a UTF-8 string. Infallible for any `T: Serialize` whose serialise is
/// itself infallible (the envelope wrapper introduces no new failure modes).
pub fn envelope_ok_string<T: Serialize>(data: &T) -> Result<String> {
    serde_json::to_string(&envelope_ok(data)).map_err(|e| {
        Error::malformed("envelope", format!("serialise ok envelope: {e}")).with_source(e)
    })
}

/// Convenience: stringify the error envelope. Cannot fail in practice —
/// errors are simple owned strings + an enum.
pub fn envelope_err_string(err: &Error) -> String {
    // unwrap-justified: envelope_err produces only owned strings + a fixed
    // shape; serde_json::to_string on a Value cannot fail for these inputs.
    serde_json::to_string(&envelope_err(err)).expect("error envelope serialisation is infallible")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canonical::serialise_yaml;
    use crate::error::Category;
    use crate::schema::AcType;
    use crate::schema::Spec;
    use tempfile::tempdir;

    fn write_spec(layout: &OrbitLayout, id: &str, goal: &str, status: SpecStatus) {
        let spec = Spec {
            id: id.into(),
            goal: goal.into(),
            cards: vec![],
            status,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir(id).unwrap();
        std::fs::write(layout.spec_file(id), serialise_yaml(&spec).unwrap()).unwrap();
    }

    fn unwrap_spec_list(resp: VerbResponse) -> SpecListResult {
        match resp {
            VerbResponse::SpecList(r) => r,
            other => panic!("expected SpecList variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_list_returns_empty_when_no_specs() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let resp = execute(&layout, &VerbRequest::SpecList(SpecListArgs::default())).unwrap();
        assert!(unwrap_spec_list(resp).specs.is_empty());
    }

    #[test]
    fn spec_list_returns_specs_sorted_by_id() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0002", "second", SpecStatus::Open);
        write_spec(&layout, "0001", "first", SpecStatus::Open);

        let resp = execute(&layout, &VerbRequest::SpecList(SpecListArgs::default())).unwrap();
        let r = unwrap_spec_list(resp);
        let ids: Vec<_> = r.specs.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(ids, vec!["0001", "0002"]);
    }

    #[test]
    fn spec_list_status_filter_open_excludes_closed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "first", SpecStatus::Open);
        write_spec(&layout, "0002", "second", SpecStatus::Closed);

        let args = SpecListArgs { status: Some("open".into()) };
        let resp = execute(&layout, &VerbRequest::SpecList(args)).unwrap();
        let r = unwrap_spec_list(resp);
        assert_eq!(r.specs.len(), 1);
        assert_eq!(r.specs[0].id, "0001");
    }

    #[test]
    fn spec_list_invalid_status_filter_is_malformed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let args = SpecListArgs { status: Some("nope".into()) };
        let err = execute(&layout, &VerbRequest::SpecList(args)).unwrap_err();
        assert_eq!(err.to_string(), "spec.list: malformed: status must be 'open' or 'closed', got 'nope'");
    }

    #[test]
    fn spec_list_malformed_file_surfaces_with_correct_verb() {
        // ac-05 verification: error format `<verb>: <category>: <sentence>`,
        // and the verb is the one the caller invoked (not the canonical
        // layer's generic tag).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        layout.ensure_spec_dir("bad").unwrap();
        std::fs::write(layout.spec_file("bad"), "id: '0001'\nunknown_field: oops\n").unwrap();

        let err = execute(&layout, &VerbRequest::SpecList(SpecListArgs::default())).unwrap_err();
        assert!(
            err.to_string().starts_with("spec.list: malformed: "),
            "expected spec.list-tagged malformed error, got {err}"
        );
    }

    #[test]
    fn verb_request_round_trips_through_json() {
        // The MCP surface translates `tools/call` into VerbRequest by
        // constructing `{"verb": name, "args": arguments}` and deserialising.
        // This test pins that contract.
        let json = serde_json::json!({
            "verb": "spec.list",
            "args": { "status": "open" }
        });
        let req: VerbRequest = serde_json::from_value(json).unwrap();
        match req {
            VerbRequest::SpecList(args) => assert_eq!(args.status.as_deref(), Some("open")),
            other => panic!("wrong variant: {other:?}"),
        }
    }

    #[test]
    fn verb_request_rejects_unknown_args_field() {
        // deny_unknown_fields on args means typo'd MCP arguments fail loudly
        // rather than being silently ignored.
        let json = serde_json::json!({
            "verb": "spec.list",
            "args": { "stutus": "open" }
        });
        let err = serde_json::from_value::<VerbRequest>(json).unwrap_err();
        assert!(err.to_string().contains("unknown"));
    }

    #[test]
    fn envelope_ok_shape_is_stable() {
        let resp = VerbResponse::SpecList(SpecListResult {
            specs: vec![SpecSummary {
                id: "0001".into(),
                goal: "g".into(),
                status: "open".into(),
                cards: vec![],
                labels: vec![],
            }],
        });
        let s = envelope_ok_string(&resp).unwrap();
        // Object keys are alphabetically ordered by default in serde_json,
        // so "data" comes before "ok". Inner struct fields follow declaration
        // order via the derive: id, goal, status, cards, labels.
        assert!(s.starts_with(r#"{"data":"#), "got {s}");
        assert!(s.contains(r#""ok":true"#), "got {s}");
    }

    #[test]
    fn envelope_err_shape_matches_error_format() {
        let err = Error::not_found("spec.list", "no specs dir");
        let s = envelope_err_string(&err);
        // Outer keys alphabetical: error, ok. Inner keys alphabetical:
        // category, message, verb.
        assert_eq!(
            s,
            r#"{"error":{"category":"not-found","message":"no specs dir","verb":"spec.list"},"ok":false}"#
        );
    }

    #[test]
    fn spec_show_returns_full_spec() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "the goal", SpecStatus::Open);

        let resp = execute(
            &layout,
            &VerbRequest::SpecShow(SpecShowArgs { id: "0001".into() }),
        )
        .unwrap();
        let VerbResponse::SpecShow(r) = resp else {
            panic!("wrong variant")
        };
        assert_eq!(r.spec.id, "0001");
        assert_eq!(r.spec.goal, "the goal");
        assert_eq!(r.spec.status, SpecStatus::Open);
    }

    #[test]
    fn spec_show_missing_id_is_not_found() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let err = execute(
            &layout,
            &VerbRequest::SpecShow(SpecShowArgs { id: "0099".into() }),
        )
        .unwrap_err();
        assert!(
            err.to_string().starts_with("spec.show: not-found: no spec at "),
            "got {err}"
        );
    }

    #[test]
    fn spec_show_empty_id_is_malformed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let err = execute(
            &layout,
            &VerbRequest::SpecShow(SpecShowArgs { id: String::new() }),
        )
        .unwrap_err();
        assert_eq!(err.to_string(), "spec.show: malformed: id must not be empty");
    }

    #[test]
    fn spec_show_path_traversal_id_is_malformed() {
        // Defence: a slash or `..` in id MUST fail before any filesystem op.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        for bad in ["../etc/passwd", "..", "0001/../..", "a/b"] {
            let err = execute(
                &layout,
                &VerbRequest::SpecShow(SpecShowArgs { id: bad.into() }),
            )
            .unwrap_err();
            assert!(
                err.to_string().starts_with("spec.show: malformed: "),
                "expected malformed for id={bad:?}, got {err}"
            );
        }
    }

    // ------------------------------------------------------------------------
    // spec.resolve tests (spec 2026-05-19-skills-infer-or-prompt-before-halt)
    // ------------------------------------------------------------------------
    //
    // The verb implements three-step recovery: infer (single open spec under
    // a bound card → Resolved), prompt (multi-open → Prompt with candidates),
    // halt (zero opens → Error::unavailable with one of two canonical
    // D5 halt-message templates). These tests cover all three branches plus
    // the boundary cases that drive the bound-card vs unbound paths.

    fn write_spec_with_cards(
        layout: &OrbitLayout,
        id: &str,
        goal: &str,
        status: SpecStatus,
        cards: Vec<String>,
    ) {
        let spec = Spec {
            id: id.into(),
            goal: goal.into(),
            cards,
            status,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir(id).unwrap();
        std::fs::write(layout.spec_file(id), serialise_yaml(&spec).unwrap()).unwrap();
    }

    fn write_card_for_resolve(layout: &OrbitLayout, slug: &str) {
        // Lightweight card writer for resolve tests — the spec.resolve verb
        // never reads card content beyond the resolve_numeric_slug pass that
        // canonicalises the slug shape.
        let card = crate::schema::Card {
            id: Some(slug.to_string()),
            feature: format!("feature-{slug}"),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: crate::schema::CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        let yaml = serialise_yaml(&card).unwrap();
        std::fs::write(layout.card_file(slug), yaml).unwrap();
    }

    fn write_session_card(layout: &OrbitLayout, slug: &str) {
        std::fs::write(layout.session_card_file(), format!("{slug}\n")).unwrap();
    }

    fn unwrap_resolve(resp: VerbResponse) -> SpecResolveResult {
        match resp {
            VerbResponse::SpecResolve(r) => r,
            other => panic!("expected SpecResolve variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_resolve_ac01_uses_bound_card_when_card_has_single_open_spec() {
        // ac-01: skill uses the bound spec when no argument is supplied,
        // surfacing which spec it picked. Resolved.source = "bound_card".
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card_for_resolve(&layout, "0001-card-a");
        write_card_for_resolve(&layout, "0002-card-b");
        write_spec_with_cards(
            &layout,
            "2026-05-01-a",
            "goal a",
            SpecStatus::Open,
            vec!["0001-card-a".into()],
        );
        write_spec_with_cards(
            &layout,
            "2026-05-02-b",
            "goal b",
            SpecStatus::Open,
            vec!["0002-card-b".into()],
        );
        write_session_card(&layout, "0001-card-a");

        let resp = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs::default()),
        )
        .unwrap();

        match unwrap_resolve(resp) {
            SpecResolveResult::Resolved { id, source } => {
                assert_eq!(id, "2026-05-01-a");
                assert_eq!(source, "bound_card");
            }
            other => panic!("expected Resolved variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_resolve_ac01_falls_back_to_single_open_when_unbound() {
        // ac-01 boundary: no .session-card binding but a single open spec
        // project-wide still resolves (Resolved.source = "single_open").
        // This is the "argumentless `/orb:implement` on a one-spec project"
        // shape.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec_with_cards(
            &layout,
            "2026-05-01-only",
            "lone goal",
            SpecStatus::Open,
            vec![],
        );

        let resp = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs::default()),
        )
        .unwrap();

        match unwrap_resolve(resp) {
            SpecResolveResult::Resolved { id, source } => {
                assert_eq!(id, "2026-05-01-only");
                assert_eq!(source, "single_open");
            }
            other => panic!("expected Resolved variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_resolve_ac02_prompts_when_unbound_and_multiple_open() {
        // ac-02: skill prompts with a menu when nothing is bound. Verb
        // returns Prompt { candidates, source: "unbound" } with each
        // candidate carrying a goal_first_line so the skill's
        // AskUserQuestion can be self-describing without a second
        // spec.show.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec_with_cards(
            &layout,
            "2026-05-01-a",
            "first goal line\nsecond line should not appear",
            SpecStatus::Open,
            vec![],
        );
        write_spec_with_cards(
            &layout,
            "2026-05-02-b",
            "second spec goal",
            SpecStatus::Open,
            vec![],
        );

        let resp = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs::default()),
        )
        .unwrap();

        match unwrap_resolve(resp) {
            SpecResolveResult::Prompt { candidates, source } => {
                assert_eq!(source, "unbound");
                assert_eq!(candidates.len(), 2);
                // Candidates are sorted by id (deterministic menu order).
                assert_eq!(candidates[0].id, "2026-05-01-a");
                assert_eq!(candidates[0].goal_first_line, "first goal line");
                assert_eq!(candidates[1].id, "2026-05-02-b");
                assert_eq!(candidates[1].goal_first_line, "second spec goal");
            }
            other => panic!("expected Prompt variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_resolve_ac02_prompts_when_bound_card_has_multiple_open_specs() {
        // ac-02 boundary: a bound card with multiple open specs prompts
        // with a card-narrowed menu. Prompt.source = "bound_card_multi".
        // Specs for other cards are not in the menu.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card_for_resolve(&layout, "0001-card-a");
        write_card_for_resolve(&layout, "0002-card-b");
        write_spec_with_cards(
            &layout,
            "2026-05-01-a1",
            "card-a goal 1",
            SpecStatus::Open,
            vec!["0001-card-a".into()],
        );
        write_spec_with_cards(
            &layout,
            "2026-05-02-a2",
            "card-a goal 2",
            SpecStatus::Open,
            vec!["0001-card-a".into()],
        );
        write_spec_with_cards(
            &layout,
            "2026-05-03-b1",
            "card-b goal — must not appear",
            SpecStatus::Open,
            vec!["0002-card-b".into()],
        );
        write_session_card(&layout, "0001-card-a");

        let resp = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs::default()),
        )
        .unwrap();

        match unwrap_resolve(resp) {
            SpecResolveResult::Prompt { candidates, source } => {
                assert_eq!(source, "bound_card_multi");
                let ids: Vec<&str> = candidates.iter().map(|c| c.id.as_str()).collect();
                assert_eq!(ids, vec!["2026-05-01-a1", "2026-05-02-a2"]);
            }
            other => panic!("expected Prompt variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_resolve_ac03_halts_terminal_when_unbound_and_no_open_specs() {
        // ac-03: both fallbacks fail. No .session-card, no open specs.
        // The error message is the D5 terminal halt template verbatim
        // and skills surface it without paraphrasing.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // No specs, no .session-card.

        let err = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs {
                skill: Some("implement".into()),
                card: None,
            }),
        )
        .unwrap_err();

        assert_eq!(err.category, Category::Unavailable);
        assert_eq!(err.verb, "spec.resolve");
        // D5 terminal template — byte-identical, named skill rendered as
        // /orb:<skill>.
        assert_eq!(
            err.message,
            "no spec to act on for /orb:implement — both fallbacks failed (.session-card is unbound and no open specs exist). Create one with /orb:spec."
        );
    }

    #[test]
    fn spec_resolve_ac03_halts_recoverable_when_bound_card_has_no_open_specs() {
        // ac-03 boundary: .session-card bound to a card with no open
        // specs. The D5 recoverable halt template fires; the message
        // names the bound card and tells the user how to make progress
        // (create a spec under it, or rebind).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card_for_resolve(&layout, "0001-card-a");
        // A closed spec under card-a — does not count as open.
        write_spec_with_cards(
            &layout,
            "2026-05-01-closed",
            "closed",
            SpecStatus::Closed,
            vec!["0001-card-a".into()],
        );
        write_session_card(&layout, "0001-card-a");

        let err = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs {
                skill: Some("review-spec".into()),
                card: None,
            }),
        )
        .unwrap_err();

        assert_eq!(err.category, Category::Unavailable);
        // D5 recoverable template — byte-identical, names the resolved
        // bound card slug.
        assert_eq!(
            err.message,
            "no open spec under the bound card 0001-card-a. Create one with /orb:spec, or rebind with orbit session set-card <id>."
        );
    }

    #[test]
    fn spec_resolve_card_arg_overrides_session_card() {
        // The explicit `--card` arg beats `.session-card`. Resolved
        // single-open spec under the arg-supplied card is labelled
        // source = "card_arg" so the skill's "before doing other work"
        // preamble can distinguish arg-scoped from session-scoped
        // inference.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card_for_resolve(&layout, "0001-card-a");
        write_card_for_resolve(&layout, "0002-card-b");
        write_spec_with_cards(
            &layout,
            "2026-05-01-a",
            "card-a spec",
            SpecStatus::Open,
            vec!["0001-card-a".into()],
        );
        write_spec_with_cards(
            &layout,
            "2026-05-02-b",
            "card-b spec",
            SpecStatus::Open,
            vec!["0002-card-b".into()],
        );
        write_session_card(&layout, "0001-card-a");

        let resp = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs {
                skill: None,
                card: Some("0002-card-b".into()),
            }),
        )
        .unwrap();

        match unwrap_resolve(resp) {
            SpecResolveResult::Resolved { id, source } => {
                assert_eq!(id, "2026-05-02-b");
                assert_eq!(source, "card_arg");
            }
            other => panic!("expected Resolved variant, got {other:?}"),
        }
    }

    #[test]
    fn spec_resolve_omits_skill_label_uses_placeholder() {
        // Without `skill`, the halt template falls back to a
        // placeholder so the message format stays consistent. Useful
        // for ad-hoc inline invocation where the caller is a human
        // running `orbit spec resolve` directly.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let err = execute(
            &layout,
            &VerbRequest::SpecResolve(SpecResolveArgs::default()),
        )
        .unwrap_err();

        assert!(
            err.message.contains("/orb:<skill>"),
            "expected placeholder skill label, got {}",
            err.message
        );
    }

    #[test]
    fn spec_resolve_rejects_path_traversal_in_card_arg() {
        // Defence-in-depth: `--card ..` must not escape the cards dir.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        for bad in ["../etc/passwd", "..", "0001/../..", "a/b", ""] {
            let err = execute(
                &layout,
                &VerbRequest::SpecResolve(SpecResolveArgs {
                    skill: None,
                    card: Some(bad.into()),
                }),
            )
            .unwrap_err();
            assert_eq!(err.category, Category::Malformed, "for card={bad:?}");
            assert_eq!(err.verb, "spec.resolve");
        }
    }

    #[test]
    fn spec_resolve_round_trips_through_json_wire() {
        // MCP surface: the resolver's response must serialise through the
        // standard envelope. This pins the wire shape so consumers
        // (MCP clients, the rally lead, future SDK bindings) can rely
        // on `outcome=resolved|prompt` as the tag.
        let resp = VerbResponse::SpecResolve(SpecResolveResult::Resolved {
            id: "2026-05-19-x".into(),
            source: "bound_card".into(),
        });
        let s = envelope_ok_string(&resp).unwrap();
        assert!(s.contains(r#""outcome":"resolved""#), "got {s}");
        assert!(s.contains(r#""id":"2026-05-19-x""#), "got {s}");
        assert!(s.contains(r#""source":"bound_card""#), "got {s}");
    }

    #[test]
    fn spec_resolve_prompt_round_trips_through_json_wire() {
        let resp = VerbResponse::SpecResolve(SpecResolveResult::Prompt {
            source: "unbound".into(),
            candidates: vec![
                SpecResolveCandidate {
                    id: "2026-05-01-a".into(),
                    goal_first_line: "goal a".into(),
                },
                SpecResolveCandidate {
                    id: "2026-05-02-b".into(),
                    goal_first_line: "goal b".into(),
                },
            ],
        });
        let s = envelope_ok_string(&resp).unwrap();
        assert!(s.contains(r#""outcome":"prompt""#), "got {s}");
        assert!(s.contains(r#""goal_first_line":"goal a""#), "got {s}");
    }

    // ------------------------------------------------------------------------
    // spec.note tests
    // ------------------------------------------------------------------------

    fn read_notes_stream(layout: &OrbitLayout, id: &str) -> String {
        std::fs::read_to_string(layout.notes_stream(id)).unwrap_or_default()
    }

    #[test]
    fn spec_note_appends_jsonl_line_with_supplied_timestamp() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let args = SpecNoteArgs {
            id: "0001".into(),
            body: "first note".into(),
            labels: vec![],
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        let resp = execute(&layout, &VerbRequest::SpecNote(args)).unwrap();
        let VerbResponse::SpecNote(r) = resp else {
            panic!("wrong variant")
        };
        assert_eq!(r.note.spec_id, "0001");
        assert_eq!(r.note.body, "first note");
        assert_eq!(r.note.timestamp, "2026-05-07T12:00:00Z");

        let stream = read_notes_stream(&layout, "0001");
        // One line, JSON-shaped, ends with newline.
        let lines: Vec<_> = stream.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(stream.ends_with('\n'));
        // JSONL streams use direct struct serialisation (declaration order),
        // not envelope serialisation (alphabetical via serde_json::Value).
        // NoteEvent declaration order: spec_id, body, labels, timestamp.
        assert_eq!(
            lines[0],
            r#"{"spec_id":"0001","body":"first note","labels":[],"timestamp":"2026-05-07T12:00:00Z"}"#
        );
    }

    #[test]
    fn spec_note_appends_in_order_across_calls() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        for (i, body) in ["one", "two", "three"].iter().enumerate() {
            let args = SpecNoteArgs {
                id: "0001".into(),
                body: (*body).into(),
                labels: vec![],
                timestamp: Some(format!("2026-05-07T12:00:0{i}Z")),
            };
            execute(&layout, &VerbRequest::SpecNote(args)).unwrap();
        }
        let stream = read_notes_stream(&layout, "0001");
        let bodies: Vec<_> = stream
            .lines()
            .filter_map(|l| serde_json::from_str::<NoteEvent>(l).ok())
            .map(|e| e.body)
            .collect();
        assert_eq!(bodies, vec!["one", "two", "three"]);
    }

    #[test]
    fn spec_note_default_timestamp_is_rfc3339_shaped() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let args = SpecNoteArgs {
            id: "0001".into(),
            body: "auto-stamped".into(),
            labels: vec![],
            timestamp: None,
        };
        let resp = execute(&layout, &VerbRequest::SpecNote(args)).unwrap();
        let VerbResponse::SpecNote(r) = resp else {
            panic!()
        };
        // Sanity: looks like 2026-MM-DDTHH:MM:SSZ (RFC 3339 UTC). We avoid
        // checking the actual time because tests must be deterministic.
        assert!(
            r.note.timestamp.len() >= 20,
            "timestamp too short: {}",
            r.note.timestamp
        );
        assert!(
            r.note.timestamp.contains('T') && r.note.timestamp.ends_with('Z'),
            "timestamp not RFC 3339 UTC shaped: {}",
            r.note.timestamp
        );
    }

    #[test]
    fn spec_note_missing_spec_is_not_found() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let args = SpecNoteArgs {
            id: "9999".into(),
            body: "x".into(),
            labels: vec![],
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        let err = execute(&layout, &VerbRequest::SpecNote(args)).unwrap_err();
        assert!(
            err.to_string().starts_with("spec.note: not-found: no spec at "),
            "got {err}"
        );
    }

    #[test]
    fn spec_note_empty_body_is_malformed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let args = SpecNoteArgs {
            id: "0001".into(),
            body: String::new(),
            labels: vec![],
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        let err = execute(&layout, &VerbRequest::SpecNote(args)).unwrap_err();
        assert_eq!(err.to_string(), "spec.note: malformed: body must not be empty");
    }

    #[test]
    fn spec_note_path_traversal_id_is_malformed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let args = SpecNoteArgs {
            id: "../etc/passwd".into(),
            body: "x".into(),
            labels: vec![],
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        let err = execute(&layout, &VerbRequest::SpecNote(args)).unwrap_err();
        assert!(err.to_string().starts_with("spec.note: malformed: "));
    }

    // ------------------------------------------------------------------------
    // spec.create / spec.update / spec.close tests
    // ------------------------------------------------------------------------

    use crate::schema::{Card, CardMaturity};

    fn write_card(layout: &OrbitLayout, slug: &str) {
        let card = Card {
            id: Some(slug.to_string()),
            feature: format!("feature-{slug}"),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        let yaml = crate::canonical::serialise_yaml(&card).unwrap();
        std::fs::write(layout.card_file(slug), yaml).unwrap();
    }

    fn read_card(layout: &OrbitLayout, slug: &str) -> Card {
        let text = std::fs::read_to_string(layout.card_file(slug)).unwrap();
        parse_yaml(&text).unwrap()
    }

    #[test]
    fn spec_create_writes_yaml_and_returns_spec() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());

        let args = SpecCreateArgs {
            id: "0001".into(),
            goal: "ship it".into(),
            cards: vec!["0020-orbit-state".into()],
            labels: vec!["spec".into()],
            acceptance_criteria: vec![],
        };
        let resp = execute(&layout, &VerbRequest::SpecCreate(args)).unwrap();
        let VerbResponse::SpecCreate(r) = resp else {
            panic!("wrong variant")
        };
        assert_eq!(r.spec.id, "0001");
        assert_eq!(r.spec.status, SpecStatus::Open);

        // File on disk parses back identically.
        let text = std::fs::read_to_string(layout.spec_file("0001")).unwrap();
        let parsed: Spec = parse_yaml(&text).unwrap();
        assert_eq!(parsed, r.spec);
    }

    #[test]
    fn spec_create_conflict_when_already_exists() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let args = SpecCreateArgs {
            id: "0001".into(),
            goal: "ship".into(),
            cards: vec![],
            labels: vec![],
            acceptance_criteria: vec![],
        };
        let err = execute(&layout, &VerbRequest::SpecCreate(args)).unwrap_err();
        assert!(
            err.to_string().starts_with("spec.create: conflict: "),
            "got {err}"
        );
    }

    #[test]
    fn spec_create_empty_goal_is_malformed() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());

        let args = SpecCreateArgs {
            id: "0001".into(),
            goal: String::new(),
            cards: vec![],
            labels: vec![],
            acceptance_criteria: vec![],
        };
        let err = execute(&layout, &VerbRequest::SpecCreate(args)).unwrap_err();
        assert_eq!(err.to_string(), "spec.create: malformed: goal must not be empty");
    }

    #[test]
    fn spec_update_replaces_specified_fields_only() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let original = Spec {
            id: "0001".into(),
            goal: "original".into(),
            cards: vec!["c1".into()],
            status: SpecStatus::Open,
            labels: vec!["spec".into()],
            acceptance_criteria: vec![AcceptanceCriterion {
                id: "ac-01".into(),
                description: "first".into(),
                gate: false,
                checked: false,
                verification: None,
                ac_type: AcType::Code,
            }],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0001").unwrap();
        std::fs::write(
            layout.spec_file("0001"),
            crate::canonical::serialise_yaml(&original).unwrap(),
        )
        .unwrap();

        // Update only goal and labels — cards and ACs must stay.
        let args = SpecUpdateArgs {
            id: "0001".into(),
            goal: Some("revised".into()),
            cards: None,
            labels: Some(vec!["spec".into(), "experimental".into()]),
            acceptance_criteria: None,
        };
        let resp = execute(&layout, &VerbRequest::SpecUpdate(args)).unwrap();
        let VerbResponse::SpecUpdate(r) = resp else {
            panic!("wrong variant")
        };
        assert_eq!(r.spec.goal, "revised");
        assert_eq!(r.spec.cards, vec!["c1".to_string()]);
        assert_eq!(r.spec.labels, vec!["spec".to_string(), "experimental".to_string()]);
        assert_eq!(r.spec.acceptance_criteria.len(), 1);
        // Status must not have changed via update.
        assert_eq!(r.spec.status, SpecStatus::Open);
    }

    #[test]
    fn spec_update_rejects_empty_goal() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let args = SpecUpdateArgs {
            id: "0001".into(),
            goal: Some(String::new()),
            ..Default::default()
        };
        let err = execute(&layout, &VerbRequest::SpecUpdate(args)).unwrap_err();
        assert_eq!(err.to_string(), "spec.update: malformed: goal must not be empty");
    }

    #[test]
    fn spec_update_missing_spec_is_not_found() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let args = SpecUpdateArgs {
            id: "0099".into(),
            goal: Some("x".into()),
            ..Default::default()
        };
        let err = execute(&layout, &VerbRequest::SpecUpdate(args)).unwrap_err();
        assert!(err.to_string().starts_with("spec.update: not-found: "));
    }

    #[test]
    fn spec_close_flips_status_and_appends_to_linked_cards() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_card(&layout, "0021-tasks");

        // Spec linked to two cards.
        let spec = Spec {
            id: "0001".into(),
            goal: "g".into(),
            cards: vec!["0020-orbit-state".into(), "0021-tasks".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0001").unwrap();
        std::fs::write(
            layout.spec_file("0001"),
            crate::canonical::serialise_yaml(&spec).unwrap(),
        )
        .unwrap();

        let resp = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap();
        let VerbResponse::SpecClose(r) = resp else {
            panic!()
        };
        assert_eq!(r.spec.status, SpecStatus::Closed);
        assert_eq!(r.cards_updated.len(), 2);

        // Both cards now have the spec ref.
        let expected_ref = ".orbit/specs/0001/spec.yaml";
        for slug in ["0020-orbit-state", "0021-tasks"] {
            let card = read_card(&layout, slug);
            assert!(
                card.specs.iter().any(|s| s == expected_ref),
                "card {slug} missing spec ref: {:?}",
                card.specs
            );
        }

        // Spec on disk reflects the closed status.
        let text = std::fs::read_to_string(layout.spec_file("0001")).unwrap();
        let reread: Spec = parse_yaml(&text).unwrap();
        assert_eq!(reread.status, SpecStatus::Closed);
    }

    #[test]
    fn spec_close_idempotent_when_card_already_has_ref() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // Pre-stage card already containing the spec ref (simulates a
        // previous partial close).
        let card = Card {
            id: Some("0020-x".into()),
            feature: "f".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![".orbit/specs/0001/spec.yaml".into()],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::write(
            layout.card_file("0020-x"),
            crate::canonical::serialise_yaml(&card).unwrap(),
        )
        .unwrap();

        let spec = Spec {
            id: "0001".into(),
            goal: "g".into(),
            cards: vec!["0020-x".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0001").unwrap();
        std::fs::write(
            layout.spec_file("0001"),
            crate::canonical::serialise_yaml(&spec).unwrap(),
        )
        .unwrap();

        let resp = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap();
        let VerbResponse::SpecClose(r) = resp else {
            panic!()
        };
        // Card was a no-op, so cards_updated is empty.
        assert!(r.cards_updated.is_empty());
        // Card still has exactly one ref (no duplicate).
        let post = read_card(&layout, "0020-x");
        assert_eq!(post.specs, vec![".orbit/specs/0001/spec.yaml".to_string()]);
    }

    #[test]
    fn spec_close_already_closed_is_conflict() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Closed);

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(err.to_string().starts_with("spec.close: conflict: "));
    }

    #[test]
    fn spec_close_missing_linked_card_rolls_back_no_writes() {
        // Validate the "all linked cards update or none do" contract: if a
        // card is missing, no card writes happen and the spec stays open.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-present");
        // 0021-missing is intentionally absent.

        let spec = Spec {
            id: "0001".into(),
            goal: "g".into(),
            cards: vec!["0020-present".into(), "0021-missing".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0001").unwrap();
        std::fs::write(
            layout.spec_file("0001"),
            crate::canonical::serialise_yaml(&spec).unwrap(),
        )
        .unwrap();

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(err.to_string().starts_with("spec.close: not-found: "));

        // Present card was NOT written — phase 1 collected before phase 2.
        let present = read_card(&layout, "0020-present");
        assert!(present.specs.is_empty(), "card was modified despite atomicity contract: {:?}", present.specs);

        // Spec still open.
        let reread: Spec =
            parse_yaml(&std::fs::read_to_string(layout.spec_file("0001")).unwrap()).unwrap();
        assert_eq!(reread.status, SpecStatus::Open);
    }

    // ------------------------------------------------------------------------
    // Task verb tests (ac-07)
    // ------------------------------------------------------------------------

    fn open_task(layout: &OrbitLayout, spec_id: &str, task_id: &str, body: &str) {
        let args = TaskOpenArgs {
            spec_id: spec_id.into(),
            body: body.into(),
            labels: vec![],
            task_id: Some(task_id.into()),
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        execute(layout, &VerbRequest::TaskOpen(args)).unwrap();
    }

    #[test]
    fn task_open_appends_event_with_substrate_or_supplied_timestamp() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let args = TaskOpenArgs {
            spec_id: "0001".into(),
            body: "investigate flake".into(),
            labels: vec!["bug".into()],
            task_id: Some("t-001".into()),
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        let resp = execute(&layout, &VerbRequest::TaskOpen(args)).unwrap();
        let VerbResponse::TaskOpen(r) = resp else {
            panic!()
        };
        assert_eq!(r.task_id, "t-001");
        assert_eq!(r.event.event, TaskEventKind::Open);

        // JSONL stream contains exactly one event.
        let text = std::fs::read_to_string(layout.task_stream("0001")).unwrap();
        assert_eq!(text.lines().count(), 1);
        assert!(text.contains(r#""event":"open""#));
    }

    #[test]
    fn task_open_generates_unique_task_id_when_none_supplied() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let mk = || TaskOpenArgs {
            spec_id: "0001".into(),
            body: "x".into(),
            labels: vec![],
            task_id: None,
            timestamp: None,
        };
        let r1 = execute(&layout, &VerbRequest::TaskOpen(mk())).unwrap();
        let r2 = execute(&layout, &VerbRequest::TaskOpen(mk())).unwrap();
        let (VerbResponse::TaskOpen(a), VerbResponse::TaskOpen(b)) = (r1, r2) else {
            panic!()
        };
        assert_ne!(a.task_id, b.task_id, "task ids must be unique within a process");
        assert!(a.task_id.starts_with("t-"));
    }

    #[test]
    fn task_open_duplicate_id_is_conflict() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "first");

        let dup = TaskOpenArgs {
            spec_id: "0001".into(),
            body: "again".into(),
            labels: vec![],
            task_id: Some("t1".into()),
            timestamp: Some("2026-05-07T12:00:00Z".into()),
        };
        let err = execute(&layout, &VerbRequest::TaskOpen(dup)).unwrap_err();
        assert!(
            err.to_string().starts_with("task.open: conflict: "),
            "got {err}"
        );
    }

    #[test]
    fn task_list_reduces_to_current_state() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "first");
        open_task(&layout, "0001", "t2", "second");

        // Claim t1.
        execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();

        let resp = execute(&layout, &VerbRequest::TaskList(TaskListArgs::default())).unwrap();
        let VerbResponse::TaskList(r) = resp else {
            panic!()
        };
        assert_eq!(r.tasks.len(), 2);
        let by_id: std::collections::HashMap<_, _> =
            r.tasks.iter().map(|t| (t.task_id.as_str(), t.state.as_str())).collect();
        assert_eq!(by_id["t1"], "claim");
        assert_eq!(by_id["t2"], "open");
    }

    #[test]
    fn task_ready_excludes_claimed_and_done() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "ready1");
        open_task(&layout, "0001", "t2", "claimed");
        open_task(&layout, "0001", "t3", "done");

        execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t2".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();
        execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t3".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();
        execute(
            &layout,
            &VerbRequest::TaskDone(TaskDoneArgs {
                spec_id: "0001".into(),
                task_id: "t3".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:02Z".into()),
            }),
        )
        .unwrap();

        let resp = execute(&layout, &VerbRequest::TaskReady(TaskReadyArgs::default())).unwrap();
        let VerbResponse::TaskReady(r) = resp else {
            panic!()
        };
        assert_eq!(r.tasks.len(), 1);
        assert_eq!(r.tasks[0].task_id, "t1");
    }

    #[test]
    fn task_claim_rejects_non_open_state() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "x");

        execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();

        // Second claim — current state is "claim", not "open".
        let err = execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:02Z".into()),
            }),
        )
        .unwrap_err();
        assert!(err.to_string().starts_with("task.claim: conflict: "));
    }

    #[test]
    fn task_update_after_done_is_conflict() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "x");

        execute(
            &layout,
            &VerbRequest::TaskDone(TaskDoneArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();

        let err = execute(
            &layout,
            &VerbRequest::TaskUpdate(TaskUpdateArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: "post-mortem".into(),
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:02Z".into()),
            }),
        )
        .unwrap_err();
        assert!(err.to_string().starts_with("task.update: conflict: "));
    }

    #[test]
    fn task_show_returns_full_event_history() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "x");
        execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();
        execute(
            &layout,
            &VerbRequest::TaskUpdate(TaskUpdateArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: "in progress".into(),
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:02Z".into()),
            }),
        )
        .unwrap();

        let resp = execute(
            &layout,
            &VerbRequest::TaskShow(TaskShowArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
            }),
        )
        .unwrap();
        let VerbResponse::TaskShow(r) = resp else {
            panic!()
        };
        assert_eq!(r.events.len(), 3);
        assert_eq!(r.state.state, "update");
        assert_eq!(r.state.event_count, 3);
    }

    #[test]
    fn task_state_survives_session_reset() {
        // ac-07 verification: after an open-and-claim, a fresh layout reads
        // the JSONL stream and reproduces the prior state. Tasks live on
        // disk; the index is derivable but not the source of truth.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);
        open_task(&layout, "0001", "t1", "x");
        execute(
            &layout,
            &VerbRequest::TaskClaim(TaskClaimArgs {
                spec_id: "0001".into(),
                task_id: "t1".into(),
                body: None,
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
            }),
        )
        .unwrap();

        // "Restart" — drop and rebuild the layout handle. The disk state is
        // unchanged; the in-memory index is a derived view we don't keep.
        let layout2 = OrbitLayout::at(dir.path());
        let resp = execute(&layout2, &VerbRequest::TaskList(TaskListArgs::default())).unwrap();
        let VerbResponse::TaskList(r) = resp else {
            panic!()
        };
        assert_eq!(r.tasks.len(), 1);
        assert_eq!(r.tasks[0].state, "claim");
    }

    #[test]
    fn spec_close_rejects_unfinished_tasks() {
        // ac-06 verification: "spec.close requires all child tasks done;
        // rejects with a clear error otherwise."
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let spec = Spec {
            id: "0001".into(),
            goal: "g".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0001").unwrap();
        std::fs::write(
            layout.spec_file("0001"),
            crate::canonical::serialise_yaml(&spec).unwrap(),
        )
        .unwrap();
        open_task(&layout, "0001", "t1", "still going");

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(err.to_string().starts_with("spec.close: conflict: "));
        assert!(err.message.contains("unfinished"));
    }

    #[test]
    fn spec_close_full_lifecycle_integration() {
        // ac-06 integration test: create spec → open tasks → close spec
        // without finishing tasks (rejected) → finish tasks → close spec
        // (succeeds, linked cards' specs_array updated).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-test");

        // 1. Create spec linked to one card
        execute(
            &layout,
            &VerbRequest::SpecCreate(SpecCreateArgs {
                id: "0001".into(),
                goal: "do the thing".into(),
                cards: vec!["0020-test".into()],
                labels: vec![],
                acceptance_criteria: vec![],
            }),
        )
        .unwrap();

        // 2. Open two tasks
        open_task(&layout, "0001", "t1", "task one");
        open_task(&layout, "0001", "t2", "task two");

        // 3. Close fails — tasks unfinished
        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(err.message.contains("unfinished"));

        // 4. Finish both tasks
        for tid in ["t1", "t2"] {
            execute(
                &layout,
                &VerbRequest::TaskDone(TaskDoneArgs {
                    spec_id: "0001".into(),
                    task_id: tid.into(),
                    body: None,
                    labels: vec![],
                    timestamp: Some("2026-05-07T12:00:00Z".into()),
                }),
            )
            .unwrap();
        }

        // 5. Close succeeds
        let resp = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap();
        let VerbResponse::SpecClose(r) = resp else {
            panic!()
        };
        assert_eq!(r.spec.status, SpecStatus::Closed);
        assert_eq!(r.cards_updated, vec!["0020-test".to_string()]);

        // 6. Linked card's specs array now contains the ref
        let card = read_card(&layout, "0020-test");
        assert_eq!(card.specs, vec![".orbit/specs/0001/spec.yaml".to_string()]);
    }

    #[test]
    fn task_show_unknown_task_is_not_found() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "g", SpecStatus::Open);

        let err = execute(
            &layout,
            &VerbRequest::TaskShow(TaskShowArgs {
                spec_id: "0001".into(),
                task_id: "nope".into(),
            }),
        )
        .unwrap_err();
        assert!(err.to_string().starts_with("task.show: not-found: "));
    }

    // ------------------------------------------------------------------------
    // Memory / card / choice tests (ac-08, ac-09, ac-10)
    // ------------------------------------------------------------------------

    use crate::schema::{ChoiceStatus, Memory};

    fn write_memory(layout: &OrbitLayout, key: &str, body: &str) {
        layout.ensure_dirs().unwrap();
        let m = Memory {
            key: key.into(),
            body: body.into(),
            timestamp: "2026-05-07T12:00:00Z".into(),
            labels: vec![],
        };
        std::fs::write(
            layout.memory_file(key),
            crate::canonical::serialise_yaml(&m).unwrap(),
        )
        .unwrap();
    }

    fn write_choice(layout: &OrbitLayout, slug: &str, title: &str, body: &str, status: ChoiceStatus) {
        layout.ensure_dirs().unwrap();
        // Real choices use NNNN-suffixed filenames; the `id` field carries just
        // the four-digit prefix per existing convention. Test fixtures supply
        // the full slug (`"0015-foo"`) and we derive the numeric id from it.
        let id = slug.split('-').next().unwrap_or(slug).to_string();
        let c = Choice {
            id,
            title: title.into(),
            status,
            date_created: "2026-05-07".into(),
            date_modified: None,
            body: body.into(),
            references: vec![],
        };
        std::fs::write(
            layout.choice_file(slug),
            crate::canonical::serialise_yaml(&c).unwrap(),
        )
        .unwrap();
    }

    #[test]
    fn memory_remember_writes_yaml_and_returns_memory() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let resp = execute(
            &layout,
            &VerbRequest::MemoryRemember(MemoryRememberArgs {
                key: "estimate-guard".into(),
                body: "recut at Claude-pace".into(),
                labels: vec!["methodology".into()],
                timestamp: Some("2026-05-07T12:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            }),
        )
        .unwrap();
        let VerbResponse::MemoryRemember(r) = resp else {
            panic!()
        };
        assert_eq!(r.memory.key, "estimate-guard");
        assert!(layout.memory_file("estimate-guard").exists());
    }

    #[test]
    fn memory_remember_upserts_existing_key() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        execute(
            &layout,
            &VerbRequest::MemoryRemember(MemoryRememberArgs {
                key: "k".into(),
                body: "v1".into(),
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            }),
        )
        .unwrap();
        execute(
            &layout,
            &VerbRequest::MemoryRemember(MemoryRememberArgs {
                key: "k".into(),
                body: "v2".into(),
                labels: vec![],
                timestamp: Some("2026-05-07T12:00:01Z".into()),
                no_nudge: false,
                no_warn: false,
            }),
        )
        .unwrap();
        let text = std::fs::read_to_string(layout.memory_file("k")).unwrap();
        assert!(text.contains("v2"), "upsert failed: {text}");
        assert!(!text.contains("v1"), "v1 still present: {text}");
    }

    #[test]
    fn memory_search_substring_case_insensitive_over_body_and_labels() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_memory(&layout, "k1", "Recut at Claude-pace");
        write_memory(&layout, "k2", "atomic writes for substrate");
        write_memory(&layout, "k3", "completely unrelated");

        let resp = execute(
            &layout,
            &VerbRequest::MemorySearch(MemorySearchArgs {
                query: "CLAUDE".into(),
            }),
        )
        .unwrap();
        let VerbResponse::MemorySearch(r) = resp else {
            panic!()
        };
        assert_eq!(r.memories.len(), 1);
        assert_eq!(r.memories[0].key, "k1");
    }

    #[test]
    fn memory_list_returns_sorted_by_key() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_memory(&layout, "zebra", "z");
        write_memory(&layout, "apple", "a");

        let resp = execute(&layout, &VerbRequest::MemoryList(MemoryListArgs::default())).unwrap();
        let VerbResponse::MemoryList(r) = resp else {
            panic!()
        };
        let keys: Vec<_> = r.memories.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(keys, vec!["apple", "zebra"]);
    }

    // ============================================================================
    // memory.match tests — spec 2026-05-19-memory-gates-decisions D1 (ac-01/ac-02)
    // ============================================================================

    #[test]
    fn memory_match_ranks_by_token_and_label_overlap() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // Three memories of varying overlap:
        // - perfect label overlap → should score above threshold
        // - body overlap only → should score lower
        // - no overlap → score 0, dropped
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "label-match".into(),
                body: "unrelated mechanism".into(),
                labels: vec!["0037-memory-gates".into()],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "body-match".into(),
                body: "decision-moment surfacing of memories".into(),
                labels: vec!["other".into()],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "no-match".into(),
                body: "totally unrelated content".into(),
                labels: vec!["nope".into()],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();

        let resp = execute(
            &layout,
            &VerbRequest::MemoryMatch(MemoryMatchArgs {
                topic: "decision-moment surfacing of memories at design time".into(),
                labels: vec!["0037-memory-gates".into()],
                limit: 10,
            }),
        )
        .unwrap();
        let VerbResponse::MemoryMatch(r) = resp else {
            panic!()
        };
        // label-match outranks body-match (label weighting 2x); no-match dropped.
        let keys: Vec<_> = r.matches.iter().map(|m| m.memory.key.as_str()).collect();
        assert!(keys.contains(&"label-match"), "missing label-match: {keys:?}");
        assert!(keys.contains(&"body-match"), "missing body-match: {keys:?}");
        assert!(!keys.contains(&"no-match"), "no-match leaked: {keys:?}");
        // First entry must be the label-match (higher score).
        assert_eq!(r.matches[0].memory.key, "label-match");
        // Label match alone reaches the D4 threshold (>= 0.3).
        assert!(
            r.matches[0].score >= MEMORY_MATCH_THRESHOLD,
            "label match score {} below threshold {}",
            r.matches[0].score,
            MEMORY_MATCH_THRESHOLD,
        );
    }

    #[test]
    fn memory_match_rejects_empty_topic_and_labels() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let err = execute(
            &layout,
            &VerbRequest::MemoryMatch(MemoryMatchArgs {
                topic: String::new(),
                labels: vec![],
                limit: 10,
            }),
        )
        .unwrap_err();
        assert_eq!(err.category, Category::Malformed);
    }

    #[test]
    fn memory_match_respects_limit() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        for i in 0..5 {
            memory_remember(
                &layout,
                &MemoryRememberArgs {
                    key: format!("m{i}"),
                    body: format!("body {i} decision"),
                    labels: vec!["0037".into()],
                    timestamp: Some("2026-05-19T00:00:00Z".into()),
                    no_nudge: false,
                    no_warn: false,
                },
            )
            .unwrap();
        }
        let resp = execute(
            &layout,
            &VerbRequest::MemoryMatch(MemoryMatchArgs {
                topic: "decision".into(),
                labels: vec!["0037".into()],
                limit: 2,
            }),
        )
        .unwrap();
        let VerbResponse::MemoryMatch(r) = resp else {
            panic!()
        };
        assert_eq!(r.matches.len(), 2);
    }

    // ============================================================================
    // memory.remember shape_warning tests — D5b (ac-05)
    // ============================================================================

    #[test]
    fn memory_remember_warns_on_state_shape_body() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let r = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "state-shape".into(),
                body: "the problem is recency bias".into(),
                labels: vec![],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        assert!(r.shape_warning.is_some(), "expected state-shape warning");
        // Memory was still written.
        assert!(layout.memory_file("state-shape").exists());
    }

    #[test]
    fn memory_remember_no_warning_on_mechanism_shape() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let r = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "mechanism".into(),
                body: "use orbit memory match before /orb:tabletop".into(),
                labels: vec![],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        assert!(r.shape_warning.is_none(), "unexpected warning: {:?}", r.shape_warning);
    }

    #[test]
    fn memory_remember_no_warn_flag_suppresses_warning() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let r = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "state-shape".into(),
                body: "the problem is recency bias".into(),
                labels: vec![],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: true,
            },
        )
        .unwrap();
        assert!(r.shape_warning.is_none());
    }

    #[test]
    fn memory_remember_no_false_positive_on_is_classifier() {
        // Per D5b rationale — "FineType is uv-based" must NOT fire the
        // warning (legitimate mechanism phrasing despite the "is").
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let r = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "classifier".into(),
                body: "FineType is uv-based with cargo-style lockfiles".into(),
                labels: vec![],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        assert!(r.shape_warning.is_none(), "false positive: {:?}", r.shape_warning);
    }

    // ============================================================================
    // spec.close memory-reconciliation gate tests — D4 (ac-04)
    // ============================================================================

    #[test]
    fn spec_close_blocks_when_matching_memory_is_unreconciled() {
        use crate::schema::{Card, CardMaturity};
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // Plant a card and a spec.
        let card = Card {
            id: Some("0037-memory-gates".into()),
            feature: "memory gates".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::write(
            layout.card_file("0037-memory-gates"),
            serialise_yaml(&card).unwrap(),
        )
        .unwrap();
        // Plant a memory that will match this spec via label overlap.
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "matching-memory".into(),
                body: "use mechanism X for spec close".into(),
                labels: vec!["0037-memory-gates".into()],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        // Spec with no memories_considered.
        let spec = Spec {
            id: "2026-05-19-test".into(),
            goal: "wire memory gates into spec close".into(),
            cards: vec!["0037-memory-gates".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir(&spec.id).unwrap();
        std::fs::write(layout.spec_file(&spec.id), serialise_yaml(&spec).unwrap()).unwrap();

        let err = spec_close(
            &layout,
            &SpecCloseArgs {
                id: spec.id.clone(),
                force: false,
            },
        )
        .unwrap_err();
        assert_eq!(err.category, Category::Conflict);
        assert!(
            err.message.contains("matching-memory"),
            "expected unreconciled key in error: {}",
            err.message
        );
    }

    #[test]
    fn spec_close_succeeds_when_matching_memory_is_reconciled() {
        use crate::schema::{Card, CardMaturity, MemoryReconciliation, ReconciliationDisposition};
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let card = Card {
            id: Some("0037-memory-gates".into()),
            feature: "memory gates".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::write(
            layout.card_file("0037-memory-gates"),
            serialise_yaml(&card).unwrap(),
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "matching-memory".into(),
                body: "use mechanism X for spec close".into(),
                labels: vec!["0037-memory-gates".into()],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        let spec = Spec {
            id: "2026-05-19-test2".into(),
            goal: "wire memory gates into spec close".into(),
            cards: vec!["0037-memory-gates".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![MemoryReconciliation {
                key: "matching-memory".into(),
                disposition: ReconciliationDisposition::Adopted,
                reason: "wired the close-time gate as described".into(),
            }],
        };
        layout.ensure_spec_dir(&spec.id).unwrap();
        std::fs::write(layout.spec_file(&spec.id), serialise_yaml(&spec).unwrap()).unwrap();

        let result = spec_close(
            &layout,
            &SpecCloseArgs {
                id: spec.id.clone(),
                force: false,
            },
        )
        .unwrap();
        assert!(result.forced_unreconciled.is_empty());
    }

    #[test]
    fn spec_close_force_bypasses_memory_gate_and_records_forced_unreconciled() {
        use crate::schema::{Card, CardMaturity};
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let card = Card {
            id: Some("0037-memory-gates".into()),
            feature: "memory gates".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::write(
            layout.card_file("0037-memory-gates"),
            serialise_yaml(&card).unwrap(),
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "matching-memory".into(),
                body: "use mechanism X for spec close".into(),
                labels: vec!["0037-memory-gates".into()],
                timestamp: Some("2026-05-19T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        let spec = Spec {
            id: "2026-05-19-test3".into(),
            goal: "wire memory gates into spec close".into(),
            cards: vec!["0037-memory-gates".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir(&spec.id).unwrap();
        std::fs::write(layout.spec_file(&spec.id), serialise_yaml(&spec).unwrap()).unwrap();

        let result = spec_close(
            &layout,
            &SpecCloseArgs {
                id: spec.id.clone(),
                force: true,
            },
        )
        .unwrap();
        assert_eq!(result.forced_unreconciled, vec!["matching-memory".to_string()]);
    }

    #[test]
    fn spec_close_does_not_block_when_no_memories_match() {
        use crate::schema::{Card, CardMaturity};
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let card = Card {
            id: Some("0037-memory-gates".into()),
            feature: "memory gates".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::write(
            layout.card_file("0037-memory-gates"),
            serialise_yaml(&card).unwrap(),
        )
        .unwrap();
        // No memories at all → no match set, no block.
        let spec = Spec {
            id: "2026-05-19-test4".into(),
            goal: "unrelated spec".into(),
            cards: vec!["0037-memory-gates".into()],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir(&spec.id).unwrap();
        std::fs::write(layout.spec_file(&spec.id), serialise_yaml(&spec).unwrap()).unwrap();

        let result = spec_close(
            &layout,
            &SpecCloseArgs {
                id: spec.id.clone(),
                force: false,
            },
        )
        .unwrap();
        assert!(result.forced_unreconciled.is_empty());
    }

    #[test]
    fn card_show_returns_full_card() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");

        // Full slug.
        let resp = execute(
            &layout,
            &VerbRequest::CardShow(CardShowArgs {
                slug: "0020-orbit-state".into(),
            }),
        )
        .unwrap();
        let VerbResponse::CardShow(r) = resp else {
            panic!()
        };
        assert_eq!(r.slug, "0020-orbit-state");

        // Bare NNNN resolves via prefix-match per choice 0022.
        let resp = execute(
            &layout,
            &VerbRequest::CardShow(CardShowArgs { slug: "20".into() }),
        )
        .unwrap();
        let VerbResponse::CardShow(r) = resp else {
            panic!()
        };
        assert_eq!(r.slug, "0020-orbit-state");

        // Padded form.
        let resp = execute(
            &layout,
            &VerbRequest::CardShow(CardShowArgs {
                slug: "0020".into(),
            }),
        )
        .unwrap();
        let VerbResponse::CardShow(r) = resp else {
            panic!()
        };
        assert_eq!(r.slug, "0020-orbit-state");

        // Zero-match returns not-found.
        let err = execute(
            &layout,
            &VerbRequest::CardShow(CardShowArgs { slug: "99".into() }),
        )
        .unwrap_err();
        assert_eq!(err.category, Category::NotFound);
    }

    #[test]
    fn card_show_bare_numeric_ambiguous_errors() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // Two cards both starting `0020-` — ambiguity case the resolver names.
        write_card(&layout, "0020-foo");
        write_card(&layout, "0020-bar");

        let err = execute(
            &layout,
            &VerbRequest::CardShow(CardShowArgs { slug: "20".into() }),
        )
        .unwrap_err();
        assert!(
            err.message.contains("ambiguous"),
            "expected ambiguous error, got: {}",
            err.message
        );
    }

    #[test]
    fn card_list_filters_by_maturity() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // write_card uses Planned. Add one Established manually.
        write_card(&layout, "0020-planned");
        let est = Card {
            id: Some("0021-established".into()),
            feature: "f".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "g".into(),
            maturity: CardMaturity::Established,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::write(
            layout.card_file("0021-established"),
            crate::canonical::serialise_yaml(&est).unwrap(),
        )
        .unwrap();

        let resp = execute(
            &layout,
            &VerbRequest::CardList(CardListArgs {
                maturity: Some("established".into()),
            }),
        )
        .unwrap();
        let VerbResponse::CardList(r) = resp else {
            panic!()
        };
        assert_eq!(r.cards.len(), 1);
        assert_eq!(r.cards[0].slug, "0021-established");
    }

    #[test]
    fn card_search_hits_feature_or_goal() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_card(&layout, "0021-tasks");

        let resp = execute(
            &layout,
            &VerbRequest::CardSearch(CardSearchArgs {
                query: "TASKS".into(),
            }),
        )
        .unwrap();
        let VerbResponse::CardSearch(r) = resp else {
            panic!()
        };
        assert_eq!(r.cards.len(), 1);
        assert_eq!(r.cards[0].slug, "0021-tasks");
    }

    #[test]
    fn choice_show_returns_full_choice() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        write_choice(&layout, "0015-orbit-state", "title", "body", ChoiceStatus::Accepted);

        // Bare NNNN resolves via prefix-match per choice 0022.
        let resp = execute(
            &layout,
            &VerbRequest::ChoiceShow(ChoiceShowArgs { id: "0015".into() }),
        )
        .unwrap();
        let VerbResponse::ChoiceShow(r) = resp else {
            panic!()
        };
        assert_eq!(r.choice.title, "title");

        // Bare unpadded form (`15`) resolves identically.
        let resp = execute(
            &layout,
            &VerbRequest::ChoiceShow(ChoiceShowArgs { id: "15".into() }),
        )
        .unwrap();
        let VerbResponse::ChoiceShow(r) = resp else {
            panic!()
        };
        assert_eq!(r.choice.title, "title");

        // Full slug still works.
        let resp = execute(
            &layout,
            &VerbRequest::ChoiceShow(ChoiceShowArgs {
                id: "0015-orbit-state".into(),
            }),
        )
        .unwrap();
        let VerbResponse::ChoiceShow(_) = resp else {
            panic!()
        };

        // Zero-match returns not-found.
        let err = execute(
            &layout,
            &VerbRequest::ChoiceShow(ChoiceShowArgs { id: "99".into() }),
        )
        .unwrap_err();
        assert_eq!(err.category, Category::NotFound);
    }

    #[test]
    fn choice_list_filters_by_status() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        write_choice(&layout, "0015-a", "first", "b", ChoiceStatus::Accepted);
        write_choice(&layout, "0016-b", "second", "b", ChoiceStatus::Proposed);

        let resp = execute(
            &layout,
            &VerbRequest::ChoiceList(ChoiceListArgs {
                status: Some("accepted".into()),
            }),
        )
        .unwrap();
        let VerbResponse::ChoiceList(r) = resp else {
            panic!()
        };
        assert_eq!(r.choices.len(), 1);
        assert_eq!(r.choices[0].id, "0015");
    }

    #[test]
    fn choice_search_hits_title_or_body() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        write_choice(&layout, "0015-atomic", "Atomic writes", "trade-off discussion", ChoiceStatus::Accepted);
        write_choice(&layout, "0016-other", "Other", "irrelevant", ChoiceStatus::Accepted);

        let resp = execute(
            &layout,
            &VerbRequest::ChoiceSearch(ChoiceSearchArgs {
                query: "TRADE".into(),
            }),
        )
        .unwrap();
        let VerbResponse::ChoiceSearch(r) = resp else {
            panic!()
        };
        assert_eq!(r.choices.len(), 1);
        assert_eq!(r.choices[0].id, "0015");
    }

    // ------------------------------------------------------------------------
    // session.prime tests (ac-11)
    // ------------------------------------------------------------------------

    #[test]
    fn session_prime_returns_open_specs_and_capped_memories() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "open one", SpecStatus::Open);
        write_spec(&layout, "0002", "closed one", SpecStatus::Closed);
        write_spec(&layout, "0003", "open two", SpecStatus::Open);
        for i in 0..15 {
            write_memory(
                &layout,
                &format!("k{i:02}"),
                &format!("memory body {i}"),
            );
        }

        let resp = execute(
            &layout,
            &VerbRequest::SessionPrime(SessionPrimeArgs::default()),
        )
        .unwrap();
        let VerbResponse::SessionPrime(r) = resp else {
            panic!()
        };

        // Only open specs.
        assert_eq!(r.open_specs.len(), 2);
        assert!(r.open_specs.iter().all(|s| s.status == "open"));

        // Memories capped at K=10.
        assert_eq!(r.memories.len(), 10);

        // Bound formula: 40 + 2*open + min(10, 10) = 40 + 4 + 10 = 54.
        assert_eq!(r.item_bound, 54);
    }

    #[test]
    fn session_prime_includes_global_latest_handover_and_bumps_bound() {
        // spec 2026-05-16-session-handover ac-07: prime surfaces the most-
        // recent Session globally, bumps item_bound by +1, and prefixes the
        // next_step sentinel.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "open one", SpecStatus::Open);

        let s = Session {
            id: "sess-X".into(),
            started_at: "2026-05-15T12:00:00Z".into(),
            ended_at: Some("2026-05-15T13:00:00Z".into()),
            distillate: "what I tried, what worked".into(),
            card_id: Some("0036-session-handover".into()),
            labels: vec![],
        };
        std::fs::write(layout.session_file("sess-X"), serialise_yaml(&s).unwrap()).unwrap();

        let resp = execute(
            &layout,
            &VerbRequest::SessionPrime(SessionPrimeArgs::default()),
        )
        .unwrap();
        let VerbResponse::SessionPrime(r) = resp else { panic!() };

        let h = r.handover.expect("handover should be Some");
        assert_eq!(h.session_id, "sess-X");
        // Bound: 40 + 2*1 + 10 (default cap.min(DEFAULT_MEMORY_CAP))
        //        + 1 (handover) = 53
        assert_eq!(r.item_bound, 53);
        // next_step prefix matches the stable sentinel.
        assert!(
            r.next_step.starts_with("Read the handover above before any other action. "),
            "expected sentinel prefix on next_step; got: {}",
            r.next_step,
        );
    }

    #[test]
    fn session_prime_handover_absent_keeps_next_step_unchanged() {
        // spec 2026-05-16-session-handover ac-07: when no sessions exist,
        // handover stays None, item_bound has no +1 addend, and next_step
        // is the un-prefixed base text.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_spec(&layout, "0001", "open one", SpecStatus::Open);

        let resp = execute(
            &layout,
            &VerbRequest::SessionPrime(SessionPrimeArgs::default()),
        )
        .unwrap();
        let VerbResponse::SessionPrime(r) = resp else { panic!() };

        assert!(r.handover.is_none());
        // 40 + 2*1 + 10 = 52 (no +1 for handover).
        assert_eq!(r.item_bound, 52);
        assert!(
            !r.next_step.starts_with("Read the handover above"),
            "unprefixed next_step expected; got: {}",
            r.next_step,
        );
    }

    #[test]
    fn session_prime_respects_custom_memory_cap() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        for i in 0..5 {
            write_memory(&layout, &format!("k{i}"), &format!("body {i}"));
        }

        let resp = execute(
            &layout,
            &VerbRequest::SessionPrime(SessionPrimeArgs {
                memory_cap: Some(3),
            }),
        )
        .unwrap();
        let VerbResponse::SessionPrime(r) = resp else {
            panic!()
        };
        assert_eq!(r.memories.len(), 3);
    }

    #[test]
    fn envelope_round_trip_deterministic() {
        // Two independent serialisations of the same response must produce
        // byte-identical envelopes — this is the parity guarantee for ac-05
        // expressed at the envelope layer.
        let resp = VerbResponse::SpecList(SpecListResult {
            specs: vec![
                SpecSummary {
                    id: "0001".into(),
                    goal: "first".into(),
                    status: "open".into(),
                    cards: vec!["0020-orbit-state".into()],
                    labels: vec!["spec".into()],
                },
                SpecSummary {
                    id: "0002".into(),
                    goal: "second".into(),
                    status: "closed".into(),
                    cards: vec![],
                    labels: vec![],
                },
            ],
        });
        let a = envelope_ok_string(&resp).unwrap();
        let b = envelope_ok_string(&resp).unwrap();
        assert_eq!(a, b);
    }

    // -----------------------------------------------------------------------
    // spec.close AC pre-flight (spec 2026-05-13-spec-close-ac-preflight)
    // -----------------------------------------------------------------------

    /// Helper: write a spec with the given ACs to disk, ready for spec.close.
    fn write_spec_with_acs(
        layout: &OrbitLayout,
        id: &str,
        cards: Vec<String>,
        acs: Vec<AcceptanceCriterion>,
    ) {
        let spec = Spec {
            id: id.into(),
            goal: "g".into(),
            cards,
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: acs,
            memories_considered: vec![],
        };
        layout.ensure_spec_dir(id).unwrap();
        std::fs::write(
            layout.spec_file(id),
            crate::canonical::serialise_yaml(&spec).unwrap(),
        )
        .unwrap();
    }

    fn ac(id: &str, gate: bool, checked: bool, ac_type: AcType) -> AcceptanceCriterion {
        AcceptanceCriterion {
            id: id.into(),
            description: format!("description for {id}"),
            gate,
            checked,
            verification: None,
            ac_type,
        }
    }

    #[test]
    fn spec_close_rejects_unchecked_acs() {
        // ac-02 verification (spec 2026-05-13-spec-close-ac-preflight,
        // generalised by spec 2026-05-16-ac-taxonomy ac-02): spec.close
        // returns Error::conflict when one or more blocking-kind
        // (Code/Config/Doc) ACs are unchecked, listing them by id.
        // No files are written.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", false, true, AcType::Code),
                ac("ac-02", false, false, AcType::Code),
                ac("ac-03", false, false, AcType::Code),
            ],
        );

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(
            err.to_string().starts_with("spec.close: conflict: "),
            "expected spec.close conflict, got: {err}"
        );
        assert!(err.message.contains("ac-02"), "missing ac-02 in: {err}");
        assert!(err.message.contains("ac-03"), "missing ac-03 in: {err}");
        // Spec is untouched on disk.
        let on_disk: Spec = parse_yaml(&std::fs::read_to_string(layout.spec_file("0001")).unwrap()).unwrap();
        assert_eq!(on_disk.status, SpecStatus::Open);
        // Linked card's specs array unchanged.
        let card = read_card(&layout, "0020-orbit-state");
        assert!(card.specs.is_empty(), "card mutated: {:?}", card.specs);
    }

    #[test]
    fn spec_close_unchecked_gate_ac_flagged_in_error() {
        // ac-02 verification: gate ACs in the unchecked set are flagged
        // separately in the error message.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", true, false, AcType::Code),  // unchecked gate
                ac("ac-02", false, false, AcType::Code), // unchecked non-gate
            ],
        );

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        // Both ids appear in the message.
        assert!(err.message.contains("ac-01"), "missing ac-01 in: {err}");
        assert!(err.message.contains("ac-02"), "missing ac-02 in: {err}");
        // The gate suffix "(gate: ac-01)" names only the gate AC.
        assert!(
            err.message.contains("(gate: ac-01)"),
            "missing gate suffix in: {err}",
        );
    }

    #[test]
    fn spec_close_force_proceeds_despite_unchecked() {
        // ac-03 verification (spec 2026-05-13-spec-close-ac-preflight):
        // --force closes despite unchecked blocking-kind ACs; the bypassed
        // AC ids land in SpecCloseResult.forced_unchecked.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", false, true, AcType::Code),
                ac("ac-02", false, false, AcType::Code),
                ac("ac-03", false, false, AcType::Code),
            ],
        );

        let resp = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: true }),
        )
        .unwrap();
        let VerbResponse::SpecClose(r) = resp else { panic!() };
        assert_eq!(r.spec.status, SpecStatus::Closed);
        assert_eq!(r.cards_updated, vec!["0020-orbit-state".to_string()]);
        assert_eq!(
            r.forced_unchecked,
            vec!["ac-02".to_string(), "ac-03".to_string()]
        );
        assert!(r.deferrable_open.is_empty());
    }

    #[test]
    fn spec_close_observation_acs_do_not_block() {
        // spec 2026-05-16-ac-taxonomy ac-02 verification: deferrable-kind
        // (Observation in this case) unchecked ACs do not block close
        // (no --force needed) and are reported in deferrable_open.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", false, true, AcType::Code),
                ac("ac-02", false, false, AcType::Observation), // unchecked but deferrable
                ac("ac-03", false, false, AcType::Observation), // unchecked but deferrable
            ],
        );

        let resp = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap();
        let VerbResponse::SpecClose(r) = resp else { panic!() };
        assert_eq!(r.spec.status, SpecStatus::Closed);
        assert!(r.forced_unchecked.is_empty());
        assert_eq!(
            r.deferrable_open,
            vec!["ac-02".to_string(), "ac-03".to_string()]
        );
    }

    #[test]
    fn spec_close_mixed_blocking_and_deferrable() {
        // spec 2026-05-16-ac-taxonomy ac-02 verification: a spec with one
        // unchecked blocking AC + one unchecked deferrable AC: exit=conflict,
        // blocking list names only the blocking AC, deferrable_open is not
        // populated (the error path returns before it would be).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", false, false, AcType::Code),        // unchecked blocking
                ac("ac-02", false, false, AcType::Observation), // unchecked deferrable
            ],
        );

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(
            err.to_string().contains("blocking AC"),
            "expected 'blocking AC' wording in: {err}"
        );
        assert!(err.message.contains("ac-01"), "blocking AC ac-01 missing in: {err}");
        assert!(
            !err.message.contains("ac-02"),
            "deferrable ac-02 must NOT appear in blocking error in: {err}"
        );
    }

    #[test]
    fn spec_close_doc_ac_blocks() {
        // spec 2026-05-16-ac-taxonomy ac-02 verification: AcType::Doc is in
        // the blocking band (Code/Config/Doc all block close).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", false, true, AcType::Code),
                ac("ac-02", false, false, AcType::Doc), // unchecked, doc, must block
            ],
        );

        let err = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap_err();
        assert!(err.message.contains("ac-02"), "doc AC ac-02 must block: {err}");
    }

    #[test]
    fn spec_close_ops_ac_defers() {
        // spec 2026-05-16-ac-taxonomy ac-02 verification: AcType::Ops is in
        // the deferrable band (Ops/Observation both defer).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0020-orbit-state");
        write_spec_with_acs(
            &layout,
            "0001",
            vec!["0020-orbit-state".into()],
            vec![
                ac("ac-01", false, true, AcType::Code),
                ac("ac-02", false, false, AcType::Ops), // unchecked, ops, must defer
            ],
        );

        let resp = execute(
            &layout,
            &VerbRequest::SpecClose(SpecCloseArgs { id: "0001".into(), force: false }),
        )
        .unwrap();
        let VerbResponse::SpecClose(r) = resp else { panic!() };
        assert_eq!(r.spec.status, SpecStatus::Closed);
        assert_eq!(r.deferrable_open, vec!["ac-02".to_string()]);
    }

    // ========================================================================
    // Spec 2026-05-15-agent-learning-loop — Track A (skill self-improvement)
    // ========================================================================

    fn record_invocation(
        layout: &OrbitLayout,
        skill_id: &str,
        outcome: &str,
        correction: Option<&str>,
        session_id: &str,
        timestamp: Option<&str>,
    ) -> Result<SkillInvocation> {
        let args = SkillRecordInvocationArgs {
            skill_id: skill_id.into(),
            outcome: outcome.into(),
            correction: correction.map(|s| s.to_string()),
            session_id: Some(session_id.into()),
            timestamp: timestamp.map(|s| s.to_string()),
        };
        skill_record_invocation(layout, &args).map(|r| r.invocation)
    }

    #[test]
    fn skill_record_invocation_appends_row() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let inv = record_invocation(&layout, "card", "worked", None, "sess-1", None).unwrap();
        assert_eq!(inv.skill_id, "card");
        assert_eq!(inv.session_id, "sess-1");
        assert_eq!(inv.outcome, InvocationOutcome::Worked);
        assert!(!inv.timestamp.is_empty());

        let path = layout.skill_invocations_file("card");
        let body = std::fs::read_to_string(&path).unwrap();
        assert_eq!(body.lines().count(), 1, "exactly one JSONL row");

        let parsed: SkillInvocation = serde_json::from_str(body.lines().next().unwrap()).unwrap();
        assert_eq!(parsed, inv);
    }

    #[test]
    fn skill_record_invocation_rejects_bad_outcome() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let err = record_invocation(&layout, "card", "fantastic", None, "sess-1", None)
            .unwrap_err();
        assert_eq!(err.category, Category::Malformed);
        // The accepted set must surface in the message so agents see the
        // valid options without re-reading the spec.
        for expected in ["worked", "partial", "didnt-apply", "incorrect"] {
            assert!(
                err.message.contains(expected),
                "expected '{expected}' in error message: {}",
                err.message
            );
        }
    }

    #[test]
    fn skill_record_invocation_missing_session_id() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // Don't set ORBIT_SESSION_ID, don't write .orbit/.session-id.
        // session_id arg also None → should be unavailable.
        let args = SkillRecordInvocationArgs {
            skill_id: "card".into(),
            outcome: "worked".into(),
            correction: None,
            session_id: None,
            timestamp: None,
        };
        let _g = ENV_LOCK.lock().unwrap();
        let prior = std::env::var("ORBIT_SESSION_ID").ok();
        std::env::remove_var("ORBIT_SESSION_ID");
        let result = skill_record_invocation(&layout, &args);
        if let Some(v) = prior {
            std::env::set_var("ORBIT_SESSION_ID", v);
        }
        let err = result.unwrap_err();
        assert_eq!(err.category, Category::Unavailable);
        assert!(err.message.contains("ORBIT_SESSION_ID"));
        assert!(err.message.contains(".session-id"));
    }

    #[test]
    fn skill_record_invocation_omits_null_correction() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        record_invocation(&layout, "card", "worked", None, "sess-1", None).unwrap();
        let body = std::fs::read_to_string(layout.skill_invocations_file("card")).unwrap();
        let line = body.lines().next().unwrap();
        assert!(
            !line.contains("\"correction\""),
            "absent correction must be omitted, not null: {line}"
        );
    }

    #[test]
    fn skill_record_invocation_creates_skills_dir() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        // ensure_dirs deliberately not called — skills_dir is created lazily.
        layout.ensure_dirs().unwrap();
        std::fs::remove_dir_all(layout.skills_dir()).ok();
        assert!(!layout.skills_dir().exists());

        record_invocation(&layout, "card", "worked", None, "sess-1", None).unwrap();
        assert!(layout.skills_dir().is_dir());
    }

    #[test]
    fn skill_recurrence_returns_per_outcome_counts() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        for (outcome, sess, t) in [
            ("worked", "s1", "2026-05-15T10:00:00Z"),
            ("worked", "s2", "2026-05-15T11:00:00Z"),
            ("partial", "s1", "2026-05-15T12:00:00Z"),
            ("incorrect", "s2", "2026-05-15T13:00:00Z"),
        ] {
            record_invocation(&layout, "tabletop", outcome, None, sess, Some(t)).unwrap();
        }

        let resp = skill_recurrence(
            &layout,
            &SkillRecurrenceArgs {
                skill_id: "tabletop".into(),
                since: None,
            },
        )
        .unwrap();
        assert_eq!(resp.total, 4);
        assert_eq!(resp.by_outcome.worked.count, 2);
        assert_eq!(resp.by_outcome.partial.count, 1);
        assert_eq!(resp.by_outcome.didnt_apply.count, 0);
        assert_eq!(resp.by_outcome.incorrect.count, 1);
    }

    #[test]
    fn skill_recurrence_all_outcome_keys_present() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        record_invocation(&layout, "tabletop", "worked", None, "s1", None).unwrap();

        let resp = skill_recurrence(
            &layout,
            &SkillRecurrenceArgs {
                skill_id: "tabletop".into(),
                since: None,
            },
        )
        .unwrap();
        let json = serde_json::to_value(&resp).unwrap();
        let by = &json["by_outcome"];
        for key in ["worked", "partial", "didnt-apply", "incorrect"] {
            assert!(by[key].is_object(), "missing outcome key: {key}");
            assert!(by[key]["count"].is_number());
            assert!(by[key]["invocations"].is_array());
        }
    }

    #[test]
    fn skill_recurrence_returns_corrections() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        record_invocation(
            &layout,
            "tabletop",
            "incorrect",
            Some("missed the cold-fork contract"),
            "s1",
            Some("2026-05-15T10:00:00Z"),
        )
        .unwrap();

        let resp = skill_recurrence(
            &layout,
            &SkillRecurrenceArgs {
                skill_id: "tabletop".into(),
                since: None,
            },
        )
        .unwrap();
        assert_eq!(resp.by_outcome.incorrect.invocations.len(), 1);
        assert_eq!(
            resp.by_outcome.incorrect.invocations[0].correction.as_deref(),
            Some("missed the cold-fork contract")
        );
    }

    #[test]
    fn skill_recurrence_omits_null_correction() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        record_invocation(&layout, "tabletop", "worked", None, "s1", None).unwrap();

        let resp = skill_recurrence(
            &layout,
            &SkillRecurrenceArgs {
                skill_id: "tabletop".into(),
                since: None,
            },
        )
        .unwrap();
        let json = serde_json::to_value(&resp).unwrap();
        let inv = &json["by_outcome"]["worked"]["invocations"][0];
        assert!(
            inv.get("correction").is_none(),
            "correction must be absent (not null) when not recorded: {inv}"
        );
    }

    #[test]
    fn skill_recurrence_filters_by_since() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        record_invocation(&layout, "tabletop", "worked", None, "s1", Some("2026-05-10T00:00:00Z"))
            .unwrap();
        record_invocation(&layout, "tabletop", "worked", None, "s1", Some("2026-05-15T00:00:00Z"))
            .unwrap();

        let resp = skill_recurrence(
            &layout,
            &SkillRecurrenceArgs {
                skill_id: "tabletop".into(),
                since: Some("2026-05-12T00:00:00Z".into()),
            },
        )
        .unwrap();
        assert_eq!(resp.total, 1);
        assert_eq!(resp.by_outcome.worked.count, 1);
    }

    #[test]
    fn skill_recurrence_empty_when_no_file() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let resp = skill_recurrence(
            &layout,
            &SkillRecurrenceArgs {
                skill_id: "tabletop".into(),
                since: None,
            },
        )
        .unwrap();
        assert_eq!(resp.total, 0);
        assert_eq!(resp.by_outcome.worked.count, 0);
        assert_eq!(resp.by_outcome.partial.count, 0);
        assert_eq!(resp.by_outcome.didnt_apply.count, 0);
        assert_eq!(resp.by_outcome.incorrect.count, 0);
    }

    // ========================================================================
    // Spec 2026-05-15-agent-learning-loop — Track B (session continuity)
    // ========================================================================

    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    #[test]
    fn session_start_writes_uuid_v4() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let r = session_start(&layout, &SessionStartArgs::default()).unwrap();
        let uuid: uuid::Uuid = r.session_id.parse().expect("session id must be a UUID");
        assert_eq!(uuid.get_version(), Some(uuid::Version::Random));

        let on_disk = std::fs::read_to_string(layout.session_id_file()).unwrap();
        assert_eq!(on_disk.trim(), r.session_id);
    }

    #[test]
    fn session_start_with_id_arg_uses_verbatim() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let r = session_start(
            &layout,
            &SessionStartArgs {
                id: Some("fixture-session-42".into()),
            },
        )
        .unwrap();
        assert_eq!(r.session_id, "fixture-session-42");
        assert_eq!(
            std::fs::read_to_string(layout.session_id_file())
                .unwrap()
                .trim(),
            "fixture-session-42"
        );
    }

    #[test]
    fn session_distill_first_call_creates_file() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-A".into()),
                distillate: "first reflection".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.id, "sess-A");
        assert_eq!(r.session.distillate, "first reflection");
        assert_eq!(r.session.started_at, r.session.ended_at.as_deref().unwrap_or(""));
        assert!(layout.session_file("sess-A").exists());
    }

    #[test]
    fn session_distill_is_idempotent() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let r1 = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-B".into()),
                distillate: "v1".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        let started = r1.session.started_at.clone();

        // Sleep briefly so the second call's RFC 3339 timestamp differs.
        std::thread::sleep(std::time::Duration::from_millis(1100));

        let r2 = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-B".into()),
                distillate: "v2".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r2.session.started_at, started, "started_at preserved");
        assert_ne!(
            r2.session.ended_at.as_deref(),
            Some(started.as_str()),
            "ended_at advances"
        );
        assert_eq!(r2.session.distillate, "v2");

        // Exactly one file on disk.
        let count = std::fs::read_dir(layout.sessions_dir()).unwrap().count();
        assert_eq!(count, 1);
    }

    #[test]
    fn session_distill_resolves_card_id_arg_first() {
        // spec 2026-05-16-session-handover ac-03: explicit --card / card_id
        // arg wins over .orbit/.session-card fallback. No validation at
        // distill time — id is opaque here.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        std::fs::write(layout.session_card_file(), "fallback-card\n").unwrap();

        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-card-A".into()),
                distillate: "first".into(),
                card_id: Some("explicit-card".into()),
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.card_id.as_deref(), Some("explicit-card"));
    }

    #[test]
    fn session_distill_falls_back_to_session_card_file() {
        // spec 2026-05-16-session-handover ac-03: when no arg is passed,
        // read .orbit/.session-card and write the trimmed slug to Session.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        std::fs::write(layout.session_card_file(), "0036-session-handover\n").unwrap();

        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-card-B".into()),
                distillate: "second".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.card_id.as_deref(), Some("0036-session-handover"));
    }

    #[test]
    fn session_distill_card_id_none_when_no_arg_and_no_file() {
        // spec 2026-05-16-session-handover ac-03: missing .session-card and
        // no arg → card_id stays None. Absence is normal.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-card-C".into()),
                distillate: "third".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.card_id, None);
    }

    #[test]
    fn session_distill_overwrites_card_id_on_subsequent_call() {
        // spec 2026-05-16-session-handover ac-03 idempotency contract:
        // latest write wins for everything except started_at.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let _ = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-card-D".into()),
                distillate: "v1".into(),
                card_id: Some("first-card".into()),
                labels: vec![],
            },
        )
        .unwrap();
        let r2 = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("sess-card-D".into()),
                distillate: "v2".into(),
                card_id: Some("second-card".into()),
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r2.session.card_id.as_deref(), Some("second-card"));
    }

    #[test]
    fn session_distill_does_not_delete_session_id_file() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        std::fs::write(layout.session_id_file(), "sess-C\n").unwrap();

        for _ in 0..2 {
            session_distill(
                &layout,
                &SessionDistillArgs {
                    session_id: Some("sess-C".into()),
                    distillate: "x".into(),
                    card_id: None,
                    labels: vec![],
                },
            )
            .unwrap();
        }
        assert!(layout.session_id_file().exists(), "Stop hook owns deletion, not distill");
    }

    #[test]
    fn session_distill_session_id_precedence() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        std::fs::write(layout.session_id_file(), "from-file\n").unwrap();

        let _g = ENV_LOCK.lock().unwrap();
        let prior = std::env::var("ORBIT_SESSION_ID").ok();

        // Arg overrides env + file.
        std::env::set_var("ORBIT_SESSION_ID", "from-env");
        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: Some("from-arg".into()),
                distillate: "d".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.id, "from-arg");

        // Env overrides file when arg is absent.
        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: None,
                distillate: "d".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.id, "from-env");

        // File only when env unset.
        std::env::remove_var("ORBIT_SESSION_ID");
        let r = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: None,
                distillate: "d".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(r.session.id, "from-file");

        match prior {
            Some(v) => std::env::set_var("ORBIT_SESSION_ID", v),
            None => std::env::remove_var("ORBIT_SESSION_ID"),
        }
    }

    #[test]
    fn session_verbs_work_without_hooks() {
        // ac-09 invariant: even with no hooks installed, CLI verbs succeed
        // when ORBIT_SESSION_ID is set in env.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let _g = ENV_LOCK.lock().unwrap();
        let prior = std::env::var("ORBIT_SESSION_ID").ok();
        std::env::set_var("ORBIT_SESSION_ID", "env-only-session");

        let inv = skill_record_invocation(
            &layout,
            &SkillRecordInvocationArgs {
                skill_id: "tabletop".into(),
                outcome: "worked".into(),
                correction: None,
                session_id: None,
                timestamp: None,
            },
        )
        .unwrap();
        assert_eq!(inv.invocation.session_id, "env-only-session");

        let dist = session_distill(
            &layout,
            &SessionDistillArgs {
                session_id: None,
                distillate: "x".into(),
                card_id: None,
                labels: vec![],
            },
        )
        .unwrap();
        assert_eq!(dist.session.id, "env-only-session");

        match prior {
            Some(v) => std::env::set_var("ORBIT_SESSION_ID", v),
            None => std::env::remove_var("ORBIT_SESSION_ID"),
        }
    }

    // ------------------------------------------------------------------------
    // spec 2026-05-16-session-handover — set-card + handover verbs
    // ------------------------------------------------------------------------

    #[test]
    fn session_set_card_writes_canonical_slug_atomically() {
        // ac-04: validate the slug, then write it newline-terminated to
        // .orbit/.session-card. Output echoes the resolved canonical slug
        // and the path.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0036-session-handover");

        let r = session_set_card(
            &layout,
            &SessionSetCardArgs {
                card_id: "0036-session-handover".into(),
            },
        )
        .unwrap();
        assert_eq!(r.card_id, "0036-session-handover");
        let on_disk = std::fs::read_to_string(layout.session_card_file()).unwrap();
        assert_eq!(on_disk, "0036-session-handover\n");
    }

    #[test]
    fn session_set_card_resolves_bare_numeric() {
        // ac-04: bare-NNNN and padded NNNN both resolve via the same
        // prefix-match helper as card.show.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0036-session-handover");

        let r =
            session_set_card(&layout, &SessionSetCardArgs { card_id: "36".into() }).unwrap();
        assert_eq!(r.card_id, "0036-session-handover");

        let r2 =
            session_set_card(&layout, &SessionSetCardArgs { card_id: "0036".into() }).unwrap();
        assert_eq!(r2.card_id, "0036-session-handover");
    }

    #[test]
    fn session_set_card_unknown_card_returns_not_found() {
        // ac-04: unknown card → Error::not_found; nothing is written.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let err = session_set_card(
            &layout,
            &SessionSetCardArgs { card_id: "9999".into() },
        )
        .unwrap_err();
        assert_eq!(err.category, crate::error::Category::NotFound);
        assert!(!layout.session_card_file().exists());
    }

    #[test]
    fn session_set_card_overwrites_existing() {
        // ac-04 + ac-10(g): mid-session re-set-card is legal and overwrites.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0036-session-handover");
        write_card(&layout, "0001-other-card");

        session_set_card(
            &layout,
            &SessionSetCardArgs { card_id: "36".into() },
        )
        .unwrap();
        session_set_card(
            &layout,
            &SessionSetCardArgs { card_id: "1".into() },
        )
        .unwrap();
        let on_disk = std::fs::read_to_string(layout.session_card_file()).unwrap();
        assert_eq!(on_disk, "0001-other-card\n");
    }

    #[test]
    fn session_handover_returns_null_when_no_sessions() {
        // ac-06: empty sessions dir → handover: None (NOT not_found).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        let r = session_handover(&layout, &SessionHandoverArgs::default()).unwrap();
        assert!(r.handover.is_none());
    }

    #[test]
    fn session_handover_global_latest_across_cards() {
        // ac-06: no --card → most-recent session across all cards.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0001-card-a");
        write_card(&layout, "0036-session-handover");

        // Plant three sessions; sess-3 is the latest.
        let plant = |slug: &str, started: &str, card: Option<&str>| {
            let s = Session {
                id: slug.into(),
                started_at: started.into(),
                ended_at: Some(started.into()),
                distillate: format!("hi from {slug}"),
                card_id: card.map(String::from),
                labels: vec![],
            };
            std::fs::write(
                layout.session_file(slug),
                serialise_yaml(&s).unwrap(),
            )
            .unwrap();
        };
        plant("sess-1", "2026-05-15T10:00:00Z", Some("0001-card-a"));
        plant("sess-2", "2026-05-15T11:00:00Z", None);
        plant("sess-3", "2026-05-15T12:00:00Z", Some("0036-session-handover"));

        let r = session_handover(&layout, &SessionHandoverArgs::default()).unwrap();
        let h = r.handover.expect("expected a handover");
        assert_eq!(h.session_id, "sess-3");
    }

    #[test]
    fn session_handover_filters_by_card_and_since() {
        // ac-06: --card filters by card_id; --since drops rows whose
        // started_at lexically predates the cutoff.
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0036-session-handover");
        write_card(&layout, "0001-other-card");

        let plant = |slug: &str, started: &str, card: &str| {
            let s = Session {
                id: slug.into(),
                started_at: started.into(),
                ended_at: Some(started.into()),
                distillate: format!("hi from {slug}"),
                card_id: Some(card.into()),
                labels: vec![],
            };
            std::fs::write(
                layout.session_file(slug),
                serialise_yaml(&s).unwrap(),
            )
            .unwrap();
        };
        plant("sess-old", "2026-05-10T10:00:00Z", "0036-session-handover");
        plant("sess-new", "2026-05-15T12:00:00Z", "0036-session-handover");
        plant("sess-other", "2026-05-15T13:00:00Z", "0001-other-card");

        // Card filter alone.
        let r = session_handover(
            &layout,
            &SessionHandoverArgs {
                card_id: Some("36".into()),
                since: None,
            },
        )
        .unwrap();
        let h = r.handover.expect("expected match");
        assert_eq!(h.session_id, "sess-new");

        // Card + since filter drops sess-old.
        let r = session_handover(
            &layout,
            &SessionHandoverArgs {
                card_id: Some("0036-session-handover".into()),
                since: Some("2026-05-12T00:00:00Z".into()),
            },
        )
        .unwrap();
        let h = r.handover.expect("expected match");
        assert_eq!(h.session_id, "sess-new");

        // Unrecorded card returns Err — caller asked for a card that
        // doesn't exist on disk; this is the not-found path on the
        // cards directory itself (per ac-04 resolution semantics).
        let err = session_handover(
            &layout,
            &SessionHandoverArgs {
                card_id: Some("9999-missing".into()),
                since: None,
            },
        )
        .unwrap_err();
        assert_eq!(err.category, crate::error::Category::NotFound);
    }

    #[test]
    fn session_handover_null_when_card_has_no_sessions() {
        // ac-06: --card pointing at a real card with no sessions returns
        // handover: None (legitimate question, not an error).
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        write_card(&layout, "0036-session-handover");

        let r = session_handover(
            &layout,
            &SessionHandoverArgs {
                card_id: Some("36".into()),
                since: None,
            },
        )
        .unwrap();
        assert!(r.handover.is_none());
    }

    #[test]
    fn session_prime_prefers_label_overlap_memories() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();

        // Spec with labels.
        let spec = Spec {
            id: "0010".into(),
            goal: "do the foo".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec!["foo".into(), "bar".into()],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0010").unwrap();
        std::fs::write(layout.spec_file("0010"), serialise_yaml(&spec).unwrap()).unwrap();

        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "older-overlap".into(),
                body: "matches foo".into(),
                labels: vec!["foo".into()],
                timestamp: Some("2026-05-01T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "newer-unrelated".into(),
                body: "no overlap".into(),
                labels: vec!["unrelated".into()],
                timestamp: Some("2026-05-14T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();

        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        let keys: Vec<_> = resp.memories.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(
            keys,
            vec!["older-overlap", "newer-unrelated"],
            "label-overlap memory comes first even when older"
        );
    }

    #[test]
    fn session_prime_falls_back_to_recency_when_no_overlap() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let spec = Spec {
            id: "0011".into(),
            goal: "x".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec!["xyz".into()],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0011").unwrap();
        std::fs::write(layout.spec_file("0011"), serialise_yaml(&spec).unwrap()).unwrap();

        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "older".into(),
                body: "x".into(),
                labels: vec!["a".into()],
                timestamp: Some("2026-05-01T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "newer".into(),
                body: "x".into(),
                labels: vec!["b".into()],
                timestamp: Some("2026-05-14T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();

        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        let keys: Vec<_> = resp.memories.iter().map(|m| m.key.as_str()).collect();
        // Both have zero overlap; tie-break by timestamp DESC.
        assert_eq!(keys, vec!["newer", "older"]);
    }

    #[test]
    fn session_prime_unchanged_when_no_spec_labels() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let spec = Spec {
            id: "0012".into(),
            goal: "x".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        layout.ensure_spec_dir("0012").unwrap();
        std::fs::write(layout.spec_file("0012"), serialise_yaml(&spec).unwrap()).unwrap();

        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "older".into(),
                body: "x".into(),
                labels: vec!["foo".into()],
                timestamp: Some("2026-05-01T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "newer".into(),
                body: "x".into(),
                labels: vec!["bar".into()],
                timestamp: Some("2026-05-14T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();

        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        let keys: Vec<_> = resp.memories.iter().map(|m| m.key.as_str()).collect();
        assert_eq!(keys, vec!["newer", "older"]);
    }

    // ----- audit.topology tests (spec 2026-05-18-documentation-topology ac-06) -----

    /// Build a layout rooted at a tmp `.orbit/` dir. The repo root is the
    /// parent of `.orbit/`, so anchor pointers in topology.md resolve from
    /// the tmp dir itself.
    fn fresh_topology_layout() -> (tempfile::TempDir, OrbitLayout) {
        let dir = tempfile::tempdir().unwrap();
        let orbit_dir = dir.path().join(".orbit");
        std::fs::create_dir_all(&orbit_dir).unwrap();
        let layout = OrbitLayout::at_orbit_dir(&orbit_dir);
        // Pre-spec-2026-05-24, topology_setup unconditionally wrote 5
        // substrate-typed seeds. Since plugin_repo gating shipped, those
        // seeds only fire when (a) config.yaml carries plugin_repo: true
        // and (b) each seed's canonical_code path exists in the working
        // tree. Stamp both prerequisites into the fixture so the existing
        // tests continue to exercise the plugin-repo branch; the
        // README-only branch has its own targeted tests.
        std::fs::write(
            layout.config_file(),
            "plugin_repo: true\n",
        )
        .unwrap();
        let stub_dir = dir.path().join("orbit-state/crates/core/src");
        std::fs::create_dir_all(&stub_dir).unwrap();
        std::fs::write(stub_dir.join("schema.rs"), "// stub for topology test\n").unwrap();
        (dir, layout)
    }

    #[test]
    fn audit_topology_not_configured_when_config_absent() {
        let (_dir, layout) = fresh_topology_layout();
        let result = audit_topology(&layout, &AuditTopologyArgs::default()).unwrap();
        assert!(!result.configured);
        assert!(result.topology_drift.is_empty());
    }

    #[test]
    fn audit_topology_not_configured_when_topology_dir_empty() {
        // Substrate-folder shape per choice 0025: `configured` is true iff
        // `.orbit/topology/` exists AND contains ≥ 1 entry (populated ==
        // configured per spec `2026-05-18-topology-substrate-migration`
        // ac-02). An empty directory reads as unconfigured. Replaces the
        // legacy `audit_topology_not_configured_when_docs_topology_unset`
        // and `audit_topology_stale_pointer_when_topology_doc_missing`
        // tests (predicates no longer exist).
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        assert!(layout.topology_dir().exists());
        assert!(layout.list_topology_files().unwrap().is_empty());
        let result = audit_topology(&layout, &AuditTopologyArgs::default()).unwrap();
        assert!(!result.configured);
        assert!(result.topology_drift.is_empty());
    }

    #[test]
    fn audit_topology_clean_when_entries_match_codebase() {
        let (dir, layout) = fresh_topology_layout();
        let repo = dir.path();
        layout.ensure_dirs().unwrap();
        // Create a subsystem dir + an authoritative file inside it.
        std::fs::create_dir_all(repo.join("src/myauth")).unwrap();
        std::fs::write(repo.join("src/myauth/mod.rs"), "// auth module\n").unwrap();
        std::fs::write(repo.join("docs-decision.md"), "# Decision\n").unwrap();
        std::fs::write(repo.join("docs-ops.md"), "# Ops\n").unwrap();
        std::fs::write(repo.join("tests-auth.rs"), "// tests\n").unwrap();
        // Write the per-subsystem yaml entry (substrate-folder shape per
        // choice 0025).
        let entry = crate::schema::TopologyEntry {
            subsystem: "myauth".into(),
            canonical_code: vec!["src/myauth/mod.rs".into()],
            decision_record: vec!["docs-decision.md".into()],
            operational_doc: vec!["docs-ops.md".into()],
            test_surface: vec!["tests-auth.rs".into()],
        };
        std::fs::write(layout.topology_file("myauth"), serialise_yaml(&entry).unwrap())
            .unwrap();
        let result = audit_topology(&layout, &AuditTopologyArgs::default()).unwrap();
        assert!(result.configured);
        assert!(
            result.topology_drift.is_empty(),
            "expected clean, got {:?}",
            result.topology_drift
        );
    }

    #[test]
    fn audit_topology_detects_stale_pointer_in_entry() {
        let (dir, layout) = fresh_topology_layout();
        let repo = dir.path();
        layout.ensure_dirs().unwrap();
        // canonical_code resolves; decision_record / operational_doc /
        // test_surface dangle → 3 stale_pointer entries.
        std::fs::create_dir_all(repo.join("src/myauth")).unwrap();
        std::fs::write(repo.join("src/myauth/mod.rs"), "// auth\n").unwrap();
        let entry = crate::schema::TopologyEntry {
            subsystem: "myauth".into(),
            canonical_code: vec!["src/myauth/mod.rs".into()],
            decision_record: vec!["nonexistent.md".into()],
            operational_doc: vec!["missing-too.md".into()],
            test_surface: vec!["also-missing.rs".into()],
        };
        std::fs::write(layout.topology_file("myauth"), serialise_yaml(&entry).unwrap())
            .unwrap();
        let result = audit_topology(&layout, &AuditTopologyArgs::default()).unwrap();
        assert!(result.configured);
        let stale: Vec<_> = result
            .topology_drift
            .iter()
            .filter(|d| d.drift_kind == "stale_pointer")
            .collect();
        assert_eq!(stale.len(), 3, "expected 3 stale pointers, got {stale:?}");
    }

    #[test]
    fn audit_topology_detects_invalid_field_on_validate_failure() {
        // Replaces the legacy `audit_topology_detects_shape_drift_when_anchors_missing`
        // test. Under the substrate-folder shape (choice 0025), `shape_drift`
        // by missing-markdown-anchor is replaced by serde+validate failures
        // surfaced as `invalid_field` / `parse_failed` drift codes.
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        // Write a syntactically valid yaml whose subsystem slug is below
        // the MIN_SUBSYSTEM_LEN threshold (4 chars; min is 5). Round-trip
        // succeeds; validate() fails.
        let bad_yaml = "\
subsystem: auth
canonical_code:
- src/x.rs
";
        std::fs::write(layout.topology_file("auth"), bad_yaml).unwrap();
        let result = audit_topology(&layout, &AuditTopologyArgs::default()).unwrap();
        assert!(result.configured, "non-empty topology dir is configured");
        let invalid: Vec<_> = result
            .topology_drift
            .iter()
            .filter(|d| d.drift_kind == "invalid_field")
            .collect();
        assert!(
            !invalid.is_empty(),
            "expected at least one invalid_field drift entry, got {:?}",
            result.topology_drift
        );
    }

    #[test]
    fn audit_topology_detects_missing_entry_for_undocumented_subsystem() {
        let (dir, layout) = fresh_topology_layout();
        let repo = dir.path();
        layout.ensure_dirs().unwrap();
        // Two subsystems in the codebase, one undocumented.
        std::fs::create_dir_all(repo.join("src/myauth")).unwrap();
        std::fs::create_dir_all(repo.join("src/ingest")).unwrap();
        // Topology entry only covers `myauth`.
        let entry = crate::schema::TopologyEntry {
            subsystem: "myauth".into(),
            canonical_code: vec!["src/myauth".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        std::fs::write(layout.topology_file("myauth"), serialise_yaml(&entry).unwrap())
            .unwrap();
        let result = audit_topology(&layout, &AuditTopologyArgs::default()).unwrap();
        let missing: Vec<_> = result
            .topology_drift
            .iter()
            .filter(|d| d.drift_kind == "missing_entry")
            .collect();
        assert_eq!(missing.len(), 1, "expected ingest as missing");
        assert_eq!(missing[0].subsystem, "ingest");
    }

    #[test]
    fn audit_topology_dispatched_through_execute() {
        // The verb is wired through the execute() entry point — confirms
        // VerbRequest::AuditTopology routes to VerbResponse::AuditTopology.
        let (_dir, layout) = fresh_topology_layout();
        let response = execute(&layout, &VerbRequest::AuditTopology(Default::default())).unwrap();
        match response {
            VerbResponse::AuditTopology(result) => {
                assert!(!result.configured);
            }
            other => panic!("expected AuditTopology, got {other:?}"),
        }
    }

    // ----- topology.setup (spec 2026-05-18-topology-substrate-migration ac-05) -----

    #[test]
    fn topology_setup_greenfield_creates_dir_and_seeds() {
        let (_dir, layout) = fresh_topology_layout();
        // Greenfield: .orbit/topology/ absent, no legacy config.
        let result = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(!result.declined);
        assert!(result.dir_created, "should create .orbit/topology/");
        assert!(!result.config_cleaned, "no legacy config to clean");
        assert_eq!(result.seeds_created.len(), 5, "five orbit-substrate entries");
        assert_eq!(result.seeds_skipped, Vec::<String>::new());
        // All five seed files exist on disk and validate.
        for slug in &["cards", "choices", "memories", "specs-substrate", "topology"] {
            let path = layout.topology_file(slug);
            assert!(path.exists(), "missing seed: {slug}");
            let text = std::fs::read_to_string(&path).unwrap();
            let entry: crate::schema::TopologyEntry = serde_yaml::from_str(&text).unwrap();
            assert!(entry.validate().is_ok(), "seed `{slug}` failed validate");
        }
    }

    #[test]
    fn topology_setup_is_idempotent() {
        // Two-stage idempotency per spec ac-05: first invocation mutates;
        // subsequent invocations are no-ops on every surface.
        let (_dir, layout) = fresh_topology_layout();
        let first = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(first.dir_created);
        assert_eq!(first.seeds_created.len(), 5);

        let second = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(!second.declined);
        assert!(!second.dir_created, "dir already exists on re-run");
        assert!(!second.config_cleaned, "config already cleaned on re-run");
        assert_eq!(second.seeds_created, Vec::<String>::new(), "no new seeds");
        assert_eq!(second.seeds_skipped.len(), 5, "all five seeds skipped on re-run");
    }

    #[test]
    fn topology_setup_brownfield_strips_legacy_config() {
        // Brownfield arm: an existing .orbit/config.yaml carrying
        // docs.topology is stripped of that key.
        let (_dir, layout) = fresh_topology_layout();
        std::fs::create_dir_all(&&layout.root).unwrap();
        std::fs::write(
            layout.config_file(),
            "docs:\n  topology: docs/topology.md\n",
        )
        .unwrap();
        let result = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(result.config_cleaned, "legacy docs.topology must be stripped");
        // Post-cleanup, the file no longer carries docs.topology.
        let post = std::fs::read_to_string(layout.config_file()).unwrap();
        assert!(
            !post.contains("docs.topology") && !post.contains("topology: docs/topology.md"),
            "post-cleanup config must not carry docs.topology: {post}"
        );
        // Re-run is a no-op on the config side.
        let again = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(!again.config_cleaned, "second run sees no legacy key");
    }

    #[test]
    fn topology_setup_brownfield_preserves_operator_edits() {
        // Existing .orbit/topology/cards.yaml with operator content is
        // NOT overwritten — skip-on-exist preserves operator agency.
        let (_dir, layout) = fresh_topology_layout();
        std::fs::create_dir_all(layout.topology_dir()).unwrap();
        let custom = "\
subsystem: cards
canonical_code:
- custom/path.rs
";
        std::fs::write(layout.topology_file("cards"), custom).unwrap();
        let result = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(
            result.seeds_skipped.iter().any(|s| s == "cards"),
            "cards seed must be skipped, got {result:?}"
        );
        // Operator content preserved verbatim.
        let post = std::fs::read_to_string(layout.topology_file("cards")).unwrap();
        assert_eq!(post, custom, "operator-edited entry must not be overwritten");
    }

    #[test]
    fn topology_setup_declined_when_answer_wire_is_no() {
        let (_dir, layout) = fresh_topology_layout();
        let result = topology_setup(
            &layout,
            &TopologySetupArgs {
                answer_wire: Some("n".into()),
            },
        )
        .unwrap();
        assert!(result.declined);
        assert!(!result.dir_created);
        assert!(result.seeds_created.is_empty());
        // Confirm nothing was written.
        assert!(!layout.topology_dir().exists() || layout.list_topology_files().unwrap().is_empty());
    }

    #[test]
    fn topology_setup_dispatched_through_execute() {
        // VerbRequest::TopologySetup routes to VerbResponse::TopologySetup.
        let (_dir, layout) = fresh_topology_layout();
        let response = execute(&layout, &VerbRequest::TopologySetup(Default::default())).unwrap();
        match response {
            VerbResponse::TopologySetup(result) => {
                assert!(!result.declined);
                assert_eq!(result.seeds_created.len(), 5);
            }
            other => panic!("expected TopologySetup, got {other:?}"),
        }
    }

    #[test]
    fn topology_setup_seeds_validate_against_schema() {
        // The hard-coded seed templates must each pass validate() —
        // protects against the templates drifting from the schema.
        for seed in topology_setup_seeds() {
            assert!(
                seed.validate().is_ok(),
                "seed `{}` failed validate: {:?}",
                seed.subsystem,
                seed.validate()
            );
        }
    }

    // ============================================================
    // Tests for spec 2026-05-18-topology-substrate-wires
    // ============================================================

    /// Build a `.orbit/topology/<subsystem>.yaml` per requested subsystem
    /// with every pointer resolved against a created path under `src/` so
    /// audit_topology stays clean. Substrate-folder shape per choice 0025.
    fn install_topology(layout: &OrbitLayout, subsystems: &[&str]) {
        let repo = &layout.root.parent().unwrap();
        std::fs::create_dir_all(repo.join("src")).unwrap();
        std::fs::create_dir_all(layout.topology_dir()).unwrap();
        for s in subsystems {
            std::fs::create_dir_all(repo.join(format!("src/{s}"))).unwrap();
            std::fs::write(repo.join(format!("src/{s}/mod.rs")), "// mod\n").unwrap();
            let entry = crate::schema::TopologyEntry {
                subsystem: (*s).into(),
                canonical_code: vec![format!("src/{s}/mod.rs")],
                decision_record: vec![],
                operational_doc: vec![],
                test_surface: vec![],
            };
            std::fs::write(
                layout.topology_file(s),
                serialise_yaml(&entry).unwrap(),
            )
            .unwrap();
        }
    }

    // ----- ac-02: session_prime topology_drift -----

    #[test]
    fn session_prime_topology_drift_none_when_config_absent() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        assert!(
            resp.topology_drift.is_none(),
            "expected None (key omitted), got {:?}",
            resp.topology_drift
        );
    }

    #[test]
    fn session_prime_topology_drift_none_when_topology_dir_empty() {
        // Substrate-folder shape per choice 0025: empty .orbit/topology/
        // reads as unconfigured → topology_drift key absent. Replaces the
        // legacy `session_prime_topology_drift_none_when_docs_topology_unset`
        // test (predicate no longer exists).
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        assert!(layout.topology_dir().exists());
        assert!(layout.list_topology_files().unwrap().is_empty());
        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        assert!(
            resp.topology_drift.is_none(),
            "expected None on empty-topology-dir, got {:?}",
            resp.topology_drift
        );
    }

    #[test]
    fn session_prime_topology_drift_some_empty_when_configured_clean() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        install_topology(&layout, &["myauth"]);
        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        match resp.topology_drift {
            Some(d) => assert!(d.is_empty(), "expected empty drift, got {d:?}"),
            None => panic!("expected Some(empty), got None"),
        }
    }

    #[test]
    fn session_prime_topology_drift_some_populated_when_drift_present() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        let repo = &layout.root.parent().unwrap();
        // Topology covers `myauth` but codebase also has `ingest` → missing_entry.
        std::fs::create_dir_all(repo.join("src/myauth")).unwrap();
        std::fs::create_dir_all(repo.join("src/ingest")).unwrap();
        std::fs::write(repo.join("src/myauth/mod.rs"), "// myauth\n").unwrap();
        let entry = crate::schema::TopologyEntry {
            subsystem: "myauth".into(),
            canonical_code: vec!["src/myauth/mod.rs".into()],
            decision_record: vec![],
            operational_doc: vec![],
            test_surface: vec![],
        };
        std::fs::write(layout.topology_file("myauth"), serialise_yaml(&entry).unwrap())
            .unwrap();
        let resp = session_prime(&layout, &SessionPrimeArgs::default()).unwrap();
        let drift = resp.topology_drift.expect("Some when configured");
        assert!(!drift.is_empty(), "expected populated drift");
        assert!(
            drift
                .iter()
                .any(|d| d.subsystem == "ingest" && d.drift_kind == "missing_entry")
        );
    }

    // ----- ac-03: spec.close topology_warnings -----

    /// Plant a spec + sidecars under `layout.spec_dir(id)` with the given
    /// text inside spec.yaml's goal. Spec ACs are empty so spec.close does
    /// not block.
    fn install_spec_for_warnings(layout: &OrbitLayout, id: &str, spec_text: &str, interview: Option<&str>, tabletop_note: Option<&str>) {
        layout.ensure_spec_dir(id).unwrap();
        let spec = Spec {
            id: id.into(),
            goal: spec_text.to_string(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
        };
        std::fs::write(layout.spec_file(id), serialise_yaml(&spec).unwrap()).unwrap();
        if let Some(body) = interview {
            std::fs::write(layout.spec_dir(id).join("interview.md"), body).unwrap();
        }
        if let Some(body) = tabletop_note {
            std::fs::write(layout.spec_dir(id).join("tabletop-note.md"), body).unwrap();
        }
    }

    #[test]
    fn spec_close_topology_warnings_populated_on_word_boundary_match() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        install_topology(&layout, &["session-prime"]);
        install_spec_for_warnings(
            &layout,
            "0001",
            "Adding a topology_drift field to session-prime envelope.",
            None,
            None,
        );
        let result = spec_close(&layout, &SpecCloseArgs { id: "0001".into(), force: false }).unwrap();
        assert!(
            result.topology_warnings.iter().any(|w| w.subsystem == "session-prime"),
            "expected session_prime warning, got {:?}",
            result.topology_warnings
        );
    }

    #[test]
    fn spec_close_topology_warnings_empty_on_substring_only() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        install_topology(&layout, &["session-prime"]);
        // Substring (no word boundaries — "session_primer" contains
        // "session_prime" as a substring but not on word boundaries).
        install_spec_for_warnings(
            &layout,
            "0002",
            "Spec touches the session-primer module which is unrelated.",
            None,
            None,
        );
        let result = spec_close(&layout, &SpecCloseArgs { id: "0002".into(), force: false }).unwrap();
        assert!(
            !result.topology_warnings.iter().any(|w| w.subsystem == "session-prime"),
            "substring should not match: {:?}",
            result.topology_warnings
        );
    }

    #[test]
    fn spec_close_topology_warnings_excludes_short_subsystem_names() {
        // ≥5 char filter — "memo" (4 chars) should be excluded even when
        // matched in the spec text.
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        install_topology(&layout, &["memo"]);
        install_spec_for_warnings(
            &layout,
            "0003",
            "We propagate memo handling across the new layer.",
            None,
            None,
        );
        let result = spec_close(&layout, &SpecCloseArgs { id: "0003".into(), force: false }).unwrap();
        assert!(
            !result.topology_warnings.iter().any(|w| w.subsystem == "memo"),
            "4-char subsystem must be filtered out: {:?}",
            result.topology_warnings
        );
    }

    #[test]
    fn spec_close_topology_warnings_match_in_tabletop_note_only() {
        // ac-03 cycle-1 LOW: tabletop-note.md must be in the scan set, not
        // just spec.yaml + interview.md.
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        install_topology(&layout, &["session-prime"]);
        install_spec_for_warnings(
            &layout,
            "0004",
            "Goal text mentions nothing relevant.",
            Some("# Interview\n\nNo subsystem names here.\n"),
            Some("# Design Note\n\nThis pinned approach extends session-prime.\n"),
        );
        let result = spec_close(&layout, &SpecCloseArgs { id: "0004".into(), force: false }).unwrap();
        assert!(
            result.topology_warnings.iter().any(|w| w.subsystem == "session-prime"),
            "tabletop-note.md must be scanned: {:?}",
            result.topology_warnings
        );
    }

    // The legacy `spec_close_topology_warnings_regex_escape_on_metachars`
    // test (foo.bar subsystem) was removed under the substrate-folder
    // migration: choice 0025's schema (`TopologyEntry::validate`) constrains
    // subsystem slugs to lowercase letters, digits, and hyphens — `.`,
    // `_`, `/` are now rejected at parse/validate time, so a
    // metacharacter-bearing subsystem cannot reach `compute_topology_warnings`.
    // The `regex::escape` call in that helper is preserved as defence-in-
    // depth (a no-op on slug-valid names) but no longer testable through
    // a constructable fixture.

    #[test]
    fn spec_close_topology_warnings_empty_when_not_configured() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        install_spec_for_warnings(
            &layout,
            "0007",
            "session_prime mentioned but topology not configured.",
            None,
            None,
        );
        let result = spec_close(&layout, &SpecCloseArgs { id: "0007".into(), force: false }).unwrap();
        assert!(
            result.topology_warnings.is_empty(),
            "no warnings when capability unconfigured: {:?}",
            result.topology_warnings
        );
    }

    // ----- ac-04: memory.remember topology nudge -----

    #[test]
    fn memory_remember_topology_label_emits_nudge() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        let result = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "k-with-topology".into(),
                body: "body".into(),
                labels: vec!["topology".into()],
                timestamp: Some("2026-05-18T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        assert!(
            result.nudge.is_some(),
            "expected nudge populated when topology label present"
        );
        let nudge = result.nudge.unwrap();
        assert!(
            nudge.contains("/orb:topology"),
            "nudge text must mention /orb:topology, got {nudge}"
        );
    }

    #[test]
    fn memory_remember_without_topology_label_emits_no_nudge() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        let result = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "k-no-label".into(),
                body: "body".into(),
                labels: vec!["unrelated".into()],
                timestamp: Some("2026-05-18T00:00:00Z".into()),
                no_nudge: false,
                no_warn: false,
            },
        )
        .unwrap();
        assert!(
            result.nudge.is_none(),
            "no nudge when topology label absent, got {:?}",
            result.nudge
        );
    }

    #[test]
    fn memory_remember_no_nudge_flag_suppresses_nudge() {
        let (_dir, layout) = fresh_topology_layout();
        layout.ensure_dirs().unwrap();
        let result = memory_remember(
            &layout,
            &MemoryRememberArgs {
                key: "k-suppressed".into(),
                body: "body".into(),
                labels: vec!["topology".into()],
                timestamp: Some("2026-05-18T00:00:00Z".into()),
                no_nudge: true,
                no_warn: false,
            },
        )
        .unwrap();
        assert!(
            result.nudge.is_none(),
            "--no-nudge must suppress even with topology label, got {:?}",
            result.nudge
        );
    }

    #[test]
    fn memory_remember_canonical_nudge_text_const() {
        // Lock the canonical text via a const — tests grep for this to
        // confirm the implementation matches the documented contract.
        assert!(TOPOLOGY_NUDGE.contains("consider /orb:topology"));
    }

    // ----- audit.conformance tests (spec 2026-05-19-workflow-conformance) -----

    fn fresh_conformance_layout() -> (tempfile::TempDir, OrbitLayout) {
        let dir = tempfile::tempdir().unwrap();
        let orbit_dir = dir.path().join(".orbit");
        std::fs::create_dir_all(&orbit_dir).unwrap();
        let layout = OrbitLayout::at_orbit_dir(&orbit_dir);
        (dir, layout)
    }

    fn date(y: i32, m: u8, d: u8) -> time::Date {
        time::Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
    }

    fn write_card_v2(
        layout: &OrbitLayout,
        slug: &str,
        maturity: crate::schema::CardMaturity,
        specs: Vec<String>,
    ) {
        use crate::schema::{Card, Scenario};
        let card = Card {
            id: Some(slug.to_string()),
            feature: "feature".into(),
            as_a: Some("agent".into()),
            i_want: Some("want".into()),
            so_that: Some("because".into()),
            goal: "goal".into(),
            maturity,
            park: None,
            scenarios: vec![Scenario {
                name: "s1".into(),
                given: "g".into(),
                when: "w".into(),
                then: "t".into(),
                gate: true,
            }],
            specs,
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::create_dir_all(layout.cards_dir()).unwrap();
        std::fs::write(layout.card_file(slug), serialise_yaml(&card).unwrap()).unwrap();
    }

    #[test]
    fn conformance_finding_serde_round_trip() {
        let finding = ConformanceFinding {
            severity: "medium".into(),
            subsystem: "memos".into(),
            subject: ".orbit/memos/2026-01-01-foo.md".into(),
            state: "stale".into(),
            evidence: None,
            remediation: Remediation {
                verb: "/orb:distill .orbit/memos/2026-01-01-foo.md".into(),
                rationale: Some("memo undistilled past 7-day threshold".into()),
            },
        };
        let yaml = serde_yaml::to_string(&finding).unwrap();
        let parsed: ConformanceFinding = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(finding, parsed);
    }

    #[test]
    fn conformance_finding_rejects_unknown_field() {
        let yaml = r#"
severity: medium
subsystem: memos
subject: foo
state: stale
remediation:
  verb: bar
unknown: surprise
"#;
        let result: std::result::Result<ConformanceFinding, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }

    fn write_canonical_method_md(layout: &OrbitLayout) {
        let canonical = CANONICAL_FILES
            .iter()
            .find(|(p, _)| *p == ".orbit/METHOD.md")
            .map(|(_, c)| *c)
            .unwrap();
        std::fs::write(&layout.root.join("METHOD.md"), canonical).unwrap();
    }

    fn write_canonical_style_md(layout: &OrbitLayout) {
        let canonical = CANONICAL_FILES
            .iter()
            .find(|(p, _)| *p == ".orbit/STYLE.md")
            .map(|(_, c)| *c)
            .unwrap();
        std::fs::write(&layout.root.join("STYLE.md"), canonical).unwrap();
    }

    fn write_canonical_files(layout: &OrbitLayout) {
        write_canonical_method_md(layout);
        write_canonical_style_md(layout);
    }

    #[test]
    fn conformance_clean_repo_zero_findings() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.is_empty(), "expected zero findings, got {:?}", result.findings);
        assert!(result.aggregated.drift.drift.is_empty());
        assert!(!result.aggregated.topology.configured);
        assert_eq!(result.pin.status, "unpinned");
    }

    #[test]
    fn conformance_idempotent_two_invocations_byte_equal() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        let r1 =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let r2 =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let j1 = serde_json::to_string(&r1).unwrap();
        let j2 = serde_json::to_string(&r2).unwrap();
        assert_eq!(j1, j2);
    }

    #[test]
    fn conformance_card_state_fires_on_planned_empty_specs() {
        let (_dir, layout) = fresh_conformance_layout();
        write_card_v2(&layout, "0099-planned-empty", crate::schema::CardMaturity::Planned, vec![]);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let card_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "cards").collect();
        assert_eq!(card_findings.len(), 1);
        let f = card_findings[0];
        assert_eq!(f.subject, "0099-planned-empty");
        assert_eq!(f.state, "ready_for_tabletop");
        assert_eq!(f.severity, "medium");
        assert_eq!(f.remediation.verb, "/orb:tabletop 99");
    }

    #[test]
    fn conformance_card_state_skips_planned_with_specs() {
        let (_dir, layout) = fresh_conformance_layout();
        write_card_v2(
            &layout,
            "0099-planned-non-empty",
            crate::schema::CardMaturity::Planned,
            vec![".orbit/specs/foo/spec.yaml".into()],
        );
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "cards"));
    }

    #[test]
    fn conformance_card_state_skips_emerging_maturity() {
        let (_dir, layout) = fresh_conformance_layout();
        write_card_v2(&layout, "0099-emerging", crate::schema::CardMaturity::Emerging, vec![]);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "cards"));
    }

    fn write_parked_card(
        layout: &OrbitLayout,
        slug: &str,
        maturity: crate::schema::CardMaturity,
        specs: Vec<String>,
    ) {
        use crate::schema::{Card, ParkSignal, Scenario};
        let card = Card {
            id: Some(slug.to_string()),
            feature: "feature".into(),
            as_a: Some("agent".into()),
            i_want: Some("want".into()),
            so_that: Some("because".into()),
            goal: "goal".into(),
            maturity,
            park: Some(ParkSignal {
                reason: "awaiting third use-case forcing".into(),
                until: "N=2 evidence".into(),
            }),
            scenarios: vec![Scenario {
                name: "s1".into(),
                given: "g".into(),
                when: "w".into(),
                then: "t".into(),
                gate: true,
            }],
            specs,
            relations: vec![],
            references: vec![],
            notes: vec![],
        };
        std::fs::create_dir_all(layout.cards_dir()).unwrap();
        std::fs::write(layout.card_file(slug), serialise_yaml(&card).unwrap()).unwrap();
    }

    #[test]
    fn conformance_card_state_skips_parked_card() {
        // Spec 2026-05-20-conformance-park-signal ac-02 (b): a card at
        // maturity:planned with empty specs but a park: block produces NO
        // ready_for_tabletop finding — the deliberate-hold carve-out.
        let (_dir, layout) = fresh_conformance_layout();
        write_parked_card(&layout, "0099-parked", crate::schema::CardMaturity::Planned, vec![]);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(
            result.findings.iter().all(|f| f.subsystem != "cards"),
            "parked card should produce no card-state findings, got {:?}",
            result.findings,
        );
    }

    #[test]
    fn conformance_card_state_park_only_affects_card_state_family() {
        // Spec 2026-05-20-conformance-park-signal ac-02: parked card alongside
        // a non-parked card — only the non-parked one fires. Other finding
        // families (memo staleness) continue to fire on the same fixture.
        let (_dir, layout) = fresh_conformance_layout();
        write_card_v2(&layout, "0098-planned", crate::schema::CardMaturity::Planned, vec![]);
        write_parked_card(&layout, "0099-parked", crate::schema::CardMaturity::Planned, vec![]);
        std::fs::create_dir_all(layout.memos_dir()).unwrap();
        std::fs::write(
            layout.memos_dir().join("2026-05-11-old.md"),
            "stale memo body\n",
        )
        .unwrap();

        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let card_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "cards").collect();
        assert_eq!(card_findings.len(), 1, "exactly one card finding expected");
        assert_eq!(card_findings[0].subject, "0098-planned");

        let memo_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "memos").collect();
        assert_eq!(memo_findings.len(), 1, "memo staleness still fires");
    }

    #[test]
    fn conformance_card_state_park_irrelevant_when_emerging() {
        // ac-02 (c): a parked card at maturity:emerging produces no finding
        // either way — the maturity check is upstream of the park check, so
        // the carve-out is a no-op here.
        let (_dir, layout) = fresh_conformance_layout();
        write_parked_card(&layout, "0099-parked-emerging", crate::schema::CardMaturity::Emerging, vec![]);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "cards"));
    }

    #[test]
    fn conformance_card_state_park_irrelevant_with_specs() {
        // ac-02 (d): a parked card with non-empty specs produces no finding
        // — the specs check is upstream of the park check.
        let (_dir, layout) = fresh_conformance_layout();
        write_parked_card(
            &layout,
            "0099-parked-with-specs",
            crate::schema::CardMaturity::Planned,
            vec![".orbit/specs/foo/spec.yaml".into()],
        );
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "cards"));
    }

    #[test]
    fn conformance_memo_stale_fires_at_eight_days() {
        let (_dir, layout) = fresh_conformance_layout();
        std::fs::create_dir_all(layout.memos_dir()).unwrap();
        std::fs::write(
            layout.memos_dir().join("2026-05-11-old.md"),
            "stale memo body\n",
        )
        .unwrap();
        // 2026-05-19 - 2026-05-11 = 8 days
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let memo_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "memos").collect();
        assert_eq!(memo_findings.len(), 1);
        let f = memo_findings[0];
        assert_eq!(f.state, "stale");
        assert!(f.subject.ends_with("2026-05-11-old.md"));
        assert!(f.remediation.verb.starts_with("/orb:distill "));
    }

    #[test]
    fn conformance_memo_no_finding_at_seven_days_strict() {
        let (_dir, layout) = fresh_conformance_layout();
        std::fs::create_dir_all(layout.memos_dir()).unwrap();
        std::fs::write(
            layout.memos_dir().join("2026-05-12-recent.md"),
            "memo body\n",
        )
        .unwrap();
        // 2026-05-19 - 2026-05-12 = 7 days; strict > means no finding.
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "memos"));
    }

    #[test]
    fn conformance_memo_no_finding_at_six_days() {
        let (_dir, layout) = fresh_conformance_layout();
        std::fs::create_dir_all(layout.memos_dir()).unwrap();
        std::fs::write(
            layout.memos_dir().join("2026-05-13-fresh.md"),
            "memo body\n",
        )
        .unwrap();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "memos"));
    }

    #[test]
    fn conformance_memo_malformed_filename_no_panic() {
        let (_dir, layout) = fresh_conformance_layout();
        std::fs::create_dir_all(layout.memos_dir()).unwrap();
        std::fs::write(layout.memos_dir().join("notes.md"), "body\n").unwrap();
        std::fs::write(layout.memos_dir().join("2026-99-99-bad.md"), "body\n").unwrap();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "memos"));
    }

    #[test]
    fn conformance_byte_drift_fires_when_method_md_differs() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_style_md(&layout);
        std::fs::write(&layout.root.join("METHOD.md"), "hand-edited\n").unwrap();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let setup_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "setup").collect();
        assert_eq!(setup_findings.len(), 1);
        assert_eq!(setup_findings[0].state, "byte_drift");
        assert_eq!(setup_findings[0].subject, ".orbit/METHOD.md");
        assert_eq!(setup_findings[0].remediation.verb, "orbit setup");
    }

    #[test]
    fn conformance_byte_drift_fires_when_style_md_differs() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_method_md(&layout);
        std::fs::write(&layout.root.join("STYLE.md"), "hand-edited\n").unwrap();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let setup_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "setup").collect();
        assert_eq!(setup_findings.len(), 1);
        assert_eq!(setup_findings[0].state, "byte_drift");
        assert_eq!(setup_findings[0].subject, ".orbit/STYLE.md");
        assert_eq!(setup_findings[0].remediation.verb, "orbit setup");
    }

    #[test]
    fn conformance_byte_drift_silent_when_canonical_files_match() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert!(result.findings.iter().all(|f| f.subsystem != "setup"));
    }

    #[test]
    fn conformance_missing_fires_when_method_md_absent() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_style_md(&layout);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let setup_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "setup").collect();
        assert_eq!(setup_findings.len(), 1);
        assert_eq!(setup_findings[0].state, "missing");
        assert_eq!(setup_findings[0].subject, ".orbit/METHOD.md");
    }

    #[test]
    fn conformance_missing_fires_when_style_md_absent() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_method_md(&layout);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let setup_findings: Vec<_> =
            result.findings.iter().filter(|f| f.subsystem == "setup").collect();
        assert_eq!(setup_findings.len(), 1);
        assert_eq!(setup_findings[0].state, "missing");
        assert_eq!(setup_findings[0].subject, ".orbit/STYLE.md");
    }

    fn write_config_with_pin(layout: &OrbitLayout, pinned: &str) {
        let yaml = format!("plugin_version: \"{pinned}\"\n");
        std::fs::write(layout.config_file(), yaml).unwrap();
    }

    #[test]
    fn conformance_pin_unpinned_when_field_absent() {
        let (_dir, layout) = fresh_conformance_layout();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert_eq!(result.pin.status, "unpinned");
        assert!(result.pin.pinned.is_none());
        assert!(result.findings.iter().all(|f| f.state != "pin_behind" && f.state != "pin_ahead"));
    }

    #[test]
    fn conformance_pin_matches_when_field_equals_current() {
        let (_dir, layout) = fresh_conformance_layout();
        write_config_with_pin(&layout, env!("CARGO_PKG_VERSION"));
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert_eq!(result.pin.status, "matches");
        assert!(result.findings.iter().all(|f| f.state != "pin_behind" && f.state != "pin_ahead"));
    }

    #[test]
    fn conformance_pin_behind_fires_and_suppresses_per_file() {
        let (_dir, layout) = fresh_conformance_layout();
        write_config_with_pin(&layout, "0.0.1");
        // Add a byte-drift fixture that WOULD fire under matches/unpinned
        // (and STYLE.md absent, which would fire a "missing" finding).
        std::fs::write(&layout.root.join("METHOD.md"), "hand-edited\n").unwrap();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert_eq!(result.pin.status, "pin_behind");
        let pin_findings: Vec<_> =
            result.findings.iter().filter(|f| f.state == "pin_behind").collect();
        assert_eq!(pin_findings.len(), 1);
        assert_eq!(pin_findings[0].severity, "medium");
        assert_eq!(pin_findings[0].remediation.verb, "orbit setup --bump-pin");
        // ac-04 byte-drift AND missing findings are SUPPRESSED under pin_behind.
        assert!(
            result.findings.iter().all(|f| f.state != "byte_drift" && f.state != "missing"),
            "expected per-file suppression, got {:?}",
            result.findings
        );
    }

    #[test]
    fn conformance_pin_ahead_fires_and_suppresses_per_file() {
        let (_dir, layout) = fresh_conformance_layout();
        write_config_with_pin(&layout, "99.99.99");
        std::fs::write(&layout.root.join("METHOD.md"), "hand-edited\n").unwrap();
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        assert_eq!(result.pin.status, "pin_ahead");
        let pin_findings: Vec<_> =
            result.findings.iter().filter(|f| f.state == "pin_ahead").collect();
        assert_eq!(pin_findings.len(), 1);
        assert_eq!(pin_findings[0].severity, "high");
        // ac-05 single-finding dominance: pin_ahead also suppresses byte_drift and missing.
        assert!(
            result.findings.iter().all(|f| f.state != "byte_drift" && f.state != "missing"),
            "expected per-file suppression, got {:?}",
            result.findings
        );
    }

    #[test]
    fn conformance_aggregated_drift_and_topology_present() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        let result =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        // The aggregated fields are populated unconditionally — even on
        // an empty fixture.
        assert!(result.aggregated.drift.drift.is_empty());
        assert!(!result.aggregated.topology.configured);
        assert!(result.aggregated.topology.topology_drift.is_empty());
    }

    #[test]
    fn conformance_aggregated_drift_byte_equal_to_standalone() {
        // Build a fixture with one schema drift entry, invoke conformance
        // and standalone audit_drift, assert byte-equal JSON for the
        // aggregated.drift slice vs the standalone result.
        let (dir, layout) = fresh_conformance_layout();
        std::fs::create_dir_all(layout.cards_dir()).unwrap();
        let card_with_unknown_field = "id: 0099-bad
feature: test
as_a: u
i_want: i
so_that: s
goal: g
maturity: planned
scenarios:
- name: s
  given: g
  when: w
  then: t
  gate: true
specs: []
relations: []
references: []
notes: []
mystery_field: surprise
";
        std::fs::write(
            layout.card_file("0099-bad"),
            card_with_unknown_field,
        )
        .unwrap();
        let standalone = audit_drift(&layout, &AuditDriftArgs::default()).unwrap();
        let conformance =
            audit_conformance_at(&layout, &AuditConformanceArgs::default(), date(2026, 5, 19))
                .unwrap();
        let j_standalone = serde_json::to_string(&standalone).unwrap();
        let j_agg = serde_json::to_string(&conformance.aggregated.drift).unwrap();
        assert_eq!(j_standalone, j_agg);
        let _ = dir;
    }

    #[test]
    fn numeric_id_from_card_id_strips_leading_zeros() {
        assert_eq!(numeric_id_from_card_id("0039-workflow-conformance"), "39");
        assert_eq!(numeric_id_from_card_id("0001-spec-foo"), "1");
        assert_eq!(numeric_id_from_card_id("0010-foo"), "10");
        assert_eq!(numeric_id_from_card_id("9999-foo"), "9999");
    }

    #[test]
    fn conformance_vendored_method_md_matches_plugin() {
        // The vendored `crates/core/canonical/METHOD.md` is the canonical
        // bytes baked into the orbit-state binary; it must match the
        // plugin's `plugins/orb/skills/setup/METHOD.md` (the source
        // /orb:setup copies into operator repos). They drift only between
        // a plugin edit and the next /orb:release pre-flight sync.
        //
        // The plugin path resolves locally but NOT under cross-build's
        // restricted docker mount (the very reason we vendor in the
        // first place). Read-and-compare; if the plugin path is
        // unreadable, skip rather than fail — this test is a local
        // drift detector, not a CI gate.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let plugin_path = std::path::PathBuf::from(manifest_dir)
            .join("../../../plugins/orb/skills/setup/METHOD.md");
        let plugin_bytes = match std::fs::read_to_string(&plugin_path) {
            Ok(b) => b,
            Err(_) => {
                eprintln!(
                    "skipping vendored-METHOD.md sync check: plugin path unreadable at {}",
                    plugin_path.display()
                );
                return;
            }
        };
        let vendored = include_str!("../canonical/METHOD.md");
        assert_eq!(
            vendored, plugin_bytes,
            "vendored METHOD.md is out of sync with plugins/orb/skills/setup/METHOD.md — run `cp plugins/orb/skills/setup/METHOD.md orbit-state/crates/core/canonical/METHOD.md` before release",
        );
    }

    #[test]
    fn conformance_vendored_style_md_matches_plugin() {
        // Parallel to `conformance_vendored_method_md_matches_plugin`.
        // The vendored `crates/core/canonical/STYLE.md` must match
        // `plugins/orb/skills/setup/STYLE.md` — kept in sync as a
        // /orb:release pre-flight step.
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let plugin_path = std::path::PathBuf::from(manifest_dir)
            .join("../../../plugins/orb/skills/setup/STYLE.md");
        let plugin_bytes = match std::fs::read_to_string(&plugin_path) {
            Ok(b) => b,
            Err(_) => {
                eprintln!(
                    "skipping vendored-STYLE.md sync check: plugin path unreadable at {}",
                    plugin_path.display()
                );
                return;
            }
        };
        let vendored = include_str!("../canonical/STYLE.md");
        assert_eq!(
            vendored, plugin_bytes,
            "vendored STYLE.md is out of sync with plugins/orb/skills/setup/STYLE.md — run `cp plugins/orb/skills/setup/STYLE.md orbit-state/crates/core/canonical/STYLE.md` before release",
        );
    }

    #[test]
    fn audit_conformance_dispatched_through_execute() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        let request = VerbRequest::AuditConformance(AuditConformanceArgs::default());
        let response = execute(&layout, &request).unwrap();
        match response {
            VerbResponse::AuditConformance(result) => {
                assert!(result.findings.is_empty());
                assert_eq!(result.pin.status, "unpinned");
            }
            other => panic!("unexpected response: {other:?}"),
        }
    }

    // ========================================================================
    // Spec 2026-05-22-routine-proposals — per-AC test fixtures.
    //
    // Test naming follows the convention `routine_ac_<NN>_<what>`. Each AC
    // owns its block. AC-10 (`ac_type: observation`) defers — no test here,
    // closure is post-ship soak per the spec.
    // ========================================================================

    /// Helper: write a SkillInvocation row directly to the skill stream.
    /// Bypasses `skill_record_invocation` so tests can control timestamp +
    /// session_id without env-var dancing.
    fn write_invocation(
        layout: &OrbitLayout,
        skill_id: &str,
        session_id: &str,
        timestamp: &str,
    ) {
        std::fs::create_dir_all(layout.skills_dir()).unwrap();
        let inv = SkillInvocation {
            skill_id: skill_id.into(),
            session_id: session_id.into(),
            outcome: InvocationOutcome::Worked,
            correction: None,
            timestamp: timestamp.into(),
        };
        let line = serde_json::to_string(&inv).unwrap();
        let path = layout.skill_invocations_file(skill_id);
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        writeln!(f, "{line}").unwrap();
    }

    // ---- AC-01: chain reconstruction via aggregator (option b) -------------

    #[test]
    fn routine_ac_01_chain_reconstruction_via_aggregator() {
        // Substrate fixture: two sessions, each invoking a 3-step chain.
        // The aggregator should reconstruct one chain per session_id from
        // session_id + timestamp ordering — no SkillInvocation schema change.
        let (_dir, layout) = fresh_conformance_layout();
        write_invocation(&layout, "tabletop", "s1", "2026-05-21T10:00:00Z");
        write_invocation(&layout, "spec", "s1", "2026-05-21T10:01:00Z");
        write_invocation(&layout, "implement", "s1", "2026-05-21T10:02:00Z");
        write_invocation(&layout, "tabletop", "s2", "2026-05-22T09:00:00Z");
        write_invocation(&layout, "spec", "s2", "2026-05-22T09:01:00Z");
        write_invocation(&layout, "implement", "s2", "2026-05-22T09:02:00Z");

        let resp = execute(&layout, &VerbRequest::RoutineChains(RoutineChainsArgs::default()))
            .unwrap();
        let chains = match resp {
            VerbResponse::RoutineChains(r) => r.chains,
            other => panic!("unexpected: {other:?}"),
        };
        assert_eq!(chains.len(), 2);
        for c in &chains {
            assert_eq!(c.chain, vec!["tabletop", "spec", "implement"]);
        }
    }

    // ---- AC-02: recurrence detection matching rules -----------------------

    #[test]
    fn routine_ac_02_exact_match_length_two() {
        let chains = vec![
            crate::routine::SessionChain {
                session_id: "s1".into(),
                chain: vec!["a".into(), "b".into()],
            },
            crate::routine::SessionChain {
                session_id: "s2".into(),
                chain: vec!["a".into(), "b".into()],
            },
        ];
        let recurring = crate::routine::detect_recurring_chains(&chains);
        assert_eq!(recurring.len(), 1);
        assert_eq!(recurring[0].chain, vec!["a", "b"]);
        assert_eq!(recurring[0].occurrences, 2);
    }

    #[test]
    fn routine_ac_02_length_three_allows_one_skipped_step() {
        // s1 has the exact chain a-b-c; s2 has a-b-X-c with one skip.
        let chains = vec![
            crate::routine::SessionChain {
                session_id: "s1".into(),
                chain: vec!["a".into(), "b".into(), "c".into()],
            },
            crate::routine::SessionChain {
                session_id: "s2".into(),
                chain: vec!["a".into(), "b".into(), "x".into(), "c".into()],
            },
        ];
        let recurring = crate::routine::detect_recurring_chains(&chains);
        // The a-b-c chain should be recognised across both sessions.
        let abc = recurring
            .iter()
            .find(|r| r.chain == vec!["a".to_string(), "b".to_string(), "c".to_string()]);
        assert!(abc.is_some(), "a-b-c should match with one skip: {recurring:?}");
        assert_eq!(abc.unwrap().occurrences, 2);
    }

    #[test]
    fn routine_ac_02_longest_chain_wins() {
        // Both sessions run a-b-c-d. The length-3 sub-chains (a-b-c, b-c-d)
        // also "recur" in the same sessions; the longest-wins rule means
        // only a-b-c-d is returned.
        let chains = vec![
            crate::routine::SessionChain {
                session_id: "s1".into(),
                chain: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            },
            crate::routine::SessionChain {
                session_id: "s2".into(),
                chain: vec!["a".into(), "b".into(), "c".into(), "d".into()],
            },
        ];
        let recurring = crate::routine::detect_recurring_chains(&chains);
        assert_eq!(recurring.len(), 1);
        assert_eq!(recurring[0].chain, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn routine_ac_02_single_session_does_not_trigger() {
        let chains = vec![crate::routine::SessionChain {
            session_id: "only".into(),
            chain: vec!["a".into(), "b".into(), "c".into()],
        }];
        let recurring = crate::routine::detect_recurring_chains(&chains);
        assert!(recurring.is_empty(), "below threshold must not surface");
    }

    // ---- AC-03: routine.author writes a SKILL.md ---------------------------

    #[test]
    fn routine_ac_03_author_writes_skill_md() {
        let (_dir, layout) = fresh_conformance_layout();
        let args = RoutineAuthorArgs {
            chain: vec![
                "/orb:tabletop".into(),
                "/orb:spec".into(),
                "/orb:implement".into(),
            ],
            name: None,
            description: None,
            body: None,
            timestamp: Some("2026-05-22T10:00:00Z".into()),
            occurrences: Some(3),
        };
        let resp = execute(&layout, &VerbRequest::RoutineAuthor(args)).unwrap();
        let result = match resp {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        assert!(result.written);
        assert_eq!(result.name, "tabletop-spec-implement");
        let path = std::path::PathBuf::from(&result.path);
        assert!(path.exists());
        let body = std::fs::read_to_string(&path).unwrap();
        // Front-matter parseable + chain matches.
        let fm = crate::routine::parse_front_matter(&body).expect("parses");
        assert_eq!(fm.chain, vec!["/orb:tabletop", "/orb:spec", "/orb:implement"]);
        assert_eq!(fm.created_by, "agent");
        assert!(!fm.pinned);
    }

    // ---- AC-04: front-matter compliance + chain_id correctness -------------

    #[test]
    fn routine_ac_04_chain_id_matches_canonical_example() {
        // The spec gives a worked example: SHA-256 of the canonical JSON for
        // ["/orb:tabletop","/orb:spec","/orb:implement"]. The substrate
        // function must produce exactly that.
        let id = crate::routine::chain_id(&[
            "/orb:tabletop".into(),
            "/orb:spec".into(),
            "/orb:implement".into(),
        ]);
        // Recompute the expected hash from the exact canonical bytes.
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(br#"["/orb:tabletop","/orb:spec","/orb:implement"]"#);
        let expected = h.finalize();
        // Hex-encode for comparison.
        let mut hex = String::with_capacity(64);
        for b in &expected {
            hex.push_str(&format!("{:02x}", b));
        }
        assert_eq!(id, hex);
        assert_eq!(id.len(), 64);
    }

    #[test]
    fn routine_ac_04_front_matter_validates_on_write() {
        let (_dir, layout) = fresh_conformance_layout();
        let args = RoutineAuthorArgs {
            chain: vec!["/orb:a".into(), "/orb:b".into()],
            name: None,
            description: None,
            body: None,
            timestamp: Some("2026-05-22T10:00:00Z".into()),
            occurrences: None,
        };
        let result = match execute(&layout, &VerbRequest::RoutineAuthor(args)).unwrap() {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        let body = std::fs::read_to_string(&result.path).unwrap();
        let fm = crate::routine::parse_front_matter(&body).expect("schema passes");
        // Additive fields per ac-04 are present.
        assert_eq!(fm.last_verified, "2026-05-22T10:00:00Z");
        assert_eq!(fm.chain_id, crate::routine::chain_id(&fm.chain));
        // Card 0022-compatible fields present.
        assert_eq!(fm.created_by, "agent");
        assert_eq!(fm.created_at, "2026-05-22T10:00:00Z");
        assert!(!fm.pinned);
    }

    #[test]
    fn routine_ac_04_rejects_short_chain() {
        let (_dir, layout) = fresh_conformance_layout();
        let args = RoutineAuthorArgs {
            chain: vec!["/orb:only-one".into()],
            name: None,
            description: None,
            body: None,
            timestamp: None,
            occurrences: None,
        };
        let err = execute(&layout, &VerbRequest::RoutineAuthor(args)).unwrap_err();
        assert_eq!(err.category, Category::Malformed);
        assert!(err.message.contains("≥ 2"));
    }

    // ---- AC-05: v1 scope is sequential chains only -------------------------

    #[test]
    fn routine_ac_05_fan_out_pattern_does_not_trigger_authoring() {
        // A fan-out pattern: one session runs rally → drive(card-a)
        // → drive(card-b). The detection algorithm is sequential-only,
        // so even if a similar shape repeats, the DAG (variable ordering
        // across the two drive branches) is not authored as a routine.
        //
        // We model this by having two sessions where the rally step is
        // identical but the post-rally ordering varies (a then b vs b
        // then a). The longest exact-match sub-chain across both is
        // just `[rally]`, which is length-1 and excluded. So no
        // recurring chain should surface.
        let chains = vec![
            crate::routine::SessionChain {
                session_id: "s1".into(),
                chain: vec!["rally".into(), "drive-a".into(), "drive-b".into()],
            },
            crate::routine::SessionChain {
                session_id: "s2".into(),
                chain: vec!["rally".into(), "drive-b".into(), "drive-a".into()],
            },
        ];
        let recurring = crate::routine::detect_recurring_chains(&chains);
        // No length-≥2 sub-chain matches across both sessions (the only
        // common length-2 subslice would be [rally, drive-X] and they
        // differ).
        assert!(
            recurring.is_empty(),
            "fan-out / DAG-shaped pattern must not trigger sequential routine authoring: {recurring:?}"
        );
    }

    // ---- AC-06: routine.verify is the only writer of last_verified ---------

    #[test]
    fn routine_ac_06_verify_advances_last_verified_on_pass() {
        let (_dir, layout) = fresh_conformance_layout();
        // Author a routine pointing at /orb:audit (a real shipped skill
        // in plugins/orb/skills/audit/SKILL.md — but we don't have that
        // skill tree in the test temp dir, so create the plugin path
        // explicitly to make the ref resolve).
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/audit")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/audit/SKILL.md"),
            "---\nname: audit\n---\n# /orb:audit\n",
        )
        .unwrap();
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/spec")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/spec/SKILL.md"),
            "---\nname: spec\n---\n",
        )
        .unwrap();

        let args = RoutineAuthorArgs {
            chain: vec!["/orb:audit".into(), "/orb:spec".into()],
            name: Some("aud-spec".into()),
            description: None,
            body: None,
            timestamp: Some("2026-04-01T00:00:00Z".into()),
            occurrences: None,
        };
        let authored = match execute(&layout, &VerbRequest::RoutineAuthor(args)).unwrap() {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };

        // Snapshot the file content before verify.
        let before = std::fs::read_to_string(&authored.path).unwrap();
        let fm_before = crate::routine::parse_front_matter(&before).unwrap();
        assert_eq!(fm_before.last_verified, "2026-04-01T00:00:00Z");

        // Verify with a fresh timestamp.
        let verify_resp = execute(
            &layout,
            &VerbRequest::RoutineVerify(RoutineVerifyArgs {
                path: authored.path.clone(),
                timestamp: Some("2026-05-22T12:00:00Z".into()),
            }),
        )
        .unwrap();
        let v = match verify_resp {
            VerbResponse::RoutineVerify(r) => r,
            other => panic!("{other:?}"),
        };
        assert!(v.broken_refs.is_empty(), "refs should resolve: {v:?}");
        assert_eq!(v.last_verified.as_deref(), Some("2026-05-22T12:00:00Z"));

        let after = std::fs::read_to_string(&authored.path).unwrap();
        let fm_after = crate::routine::parse_front_matter(&after).unwrap();
        assert_eq!(fm_after.last_verified, "2026-05-22T12:00:00Z");
    }

    #[test]
    fn routine_ac_06_verify_does_not_advance_when_ref_broken() {
        let (_dir, layout) = fresh_conformance_layout();
        // Author a routine pointing at a skill that DOESN'T exist.
        // Verify must NOT advance last_verified.
        // Create the spec ref so chain parses with ≥ 2 valid-ish refs.
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/spec")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/spec/SKILL.md"),
            "---\nname: spec\n---\n",
        )
        .unwrap();

        let args = RoutineAuthorArgs {
            chain: vec!["/orb:retired-skill".into(), "/orb:spec".into()],
            name: Some("retired-spec".into()),
            description: None,
            body: None,
            timestamp: Some("2026-04-01T00:00:00Z".into()),
            occurrences: None,
        };
        let authored = match execute(&layout, &VerbRequest::RoutineAuthor(args)).unwrap() {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };

        let verify_resp = execute(
            &layout,
            &VerbRequest::RoutineVerify(RoutineVerifyArgs {
                path: authored.path.clone(),
                timestamp: Some("2026-05-22T12:00:00Z".into()),
            }),
        )
        .unwrap();
        let v = match verify_resp {
            VerbResponse::RoutineVerify(r) => r,
            other => panic!("{other:?}"),
        };
        assert!(v.broken_refs.contains(&"/orb:retired-skill".into()));
        assert!(v.last_verified.is_none());

        // File on disk must still carry the OLD last_verified.
        let after = std::fs::read_to_string(&authored.path).unwrap();
        let fm_after = crate::routine::parse_front_matter(&after).unwrap();
        assert_eq!(fm_after.last_verified, "2026-04-01T00:00:00Z");
    }

    #[test]
    fn routine_ac_06_audit_is_read_only_byte_equal_across_runs() {
        // AC-06 verification (b): audit.conformance must not modify
        // SKILL.md content — byte-equal across two consecutive runs.
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        // Author a routine whose refs all resolve, then run audit twice.
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/audit")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/audit/SKILL.md"),
            "---\nname: audit\n---\n",
        )
        .unwrap();
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/spec")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/spec/SKILL.md"),
            "---\nname: spec\n---\n",
        )
        .unwrap();

        let authored = match execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: vec!["/orb:audit".into(), "/orb:spec".into()],
                name: Some("a-s".into()),
                description: None,
                body: None,
                timestamp: Some("2026-05-22T10:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap()
        {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        let path = authored.path.clone();
        let snap_before = std::fs::read(&path).unwrap();
        // Two audit runs — neither must mutate the routine file.
        let _ = execute(
            &layout,
            &VerbRequest::AuditConformance(AuditConformanceArgs::default()),
        )
        .unwrap();
        let _ = execute(
            &layout,
            &VerbRequest::AuditConformance(AuditConformanceArgs::default()),
        )
        .unwrap();
        let snap_after = std::fs::read(&path).unwrap();
        assert_eq!(snap_before, snap_after, "audit must be read-only on routines");
    }

    // ---- AC-07: audit finding family `routines` ----------------------------

    #[test]
    fn routine_ac_07_stale_finding_at_thirty_days() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/audit")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/audit/SKILL.md"),
            "---\nname: audit\n---\n",
        )
        .unwrap();
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/spec")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/spec/SKILL.md"),
            "---\nname: spec\n---\n",
        )
        .unwrap();
        // Author with a last_verified well over 30 days before our injected today.
        let _ = execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: vec!["/orb:audit".into(), "/orb:spec".into()],
                name: Some("stale-routine".into()),
                description: None,
                body: None,
                timestamp: Some("2026-01-01T00:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap();

        let findings = audit_conformance_at(
            &layout,
            &AuditConformanceArgs::default(),
            date(2026, 5, 22),
        )
        .unwrap()
        .findings;
        let stale: Vec<_> = findings
            .iter()
            .filter(|f| f.subsystem == "routines" && f.state == "stale")
            .collect();
        assert_eq!(stale.len(), 1, "expected 1 routines/stale finding: {findings:#?}");
        let f = stale[0];
        assert_eq!(f.severity, "medium");
        assert!(f.remediation.verb.starts_with("orbit routine verify "));
        assert!(f.subject.contains("stale-routine"));
    }

    #[test]
    fn routine_ac_07_broken_refs_finding() {
        let (_dir, layout) = fresh_conformance_layout();
        write_canonical_files(&layout);
        // No plugin/skill tree at all — every /orb:<verb> ref resolves
        // nowhere. Author a routine; both refs should land in broken_refs.
        // We need at least one valid ref so the chain has ≥2 entries
        // that get authored; but for the broken-refs finding to fire it
        // suffices that at least one ref is broken. Use one resolvable
        // ref + one broken ref.
        std::fs::create_dir_all(layout.repo_root().join("plugins/orb/skills/spec")).unwrap();
        std::fs::write(
            layout.repo_root().join("plugins/orb/skills/spec/SKILL.md"),
            "---\nname: spec\n---\n",
        )
        .unwrap();

        let _ = execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: vec!["/orb:gone".into(), "/orb:spec".into()],
                name: Some("broken-routine".into()),
                description: None,
                body: None,
                timestamp: Some("2026-05-22T10:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap();

        let findings = audit_conformance_at(
            &layout,
            &AuditConformanceArgs::default(),
            date(2026, 5, 22),
        )
        .unwrap()
        .findings;
        let broken: Vec<_> = findings
            .iter()
            .filter(|f| f.subsystem == "routines" && f.state == "broken_refs")
            .collect();
        assert_eq!(broken.len(), 1, "expected 1 routines/broken_refs finding: {findings:#?}");
        let f = broken[0];
        assert_eq!(f.severity, "medium");
        assert_eq!(f.remediation.verb, "archive via curator");
    }

    // ---- AC-08: no cross-imports between routine-author and skill-author ---

    /// AC-08: the routine-author code paths must not invoke any
    /// `skill_author` module. There is no `skill_author` module today
    /// (card 0022's skill-author is in spec/verb space, not core), so
    /// the assertion is: zero references to the literal `skill_author`
    /// identifier in routine.rs and in the routine-verb block of
    /// verbs.rs. The test reads its own source tree under the crate
    /// manifest dir to do the grep — read-only.
    #[test]
    fn routine_ac_08_no_cross_imports_with_skill_author() {
        let manifest_dir = env!("CARGO_MANIFEST_DIR");
        let routine_rs = std::path::PathBuf::from(manifest_dir).join("src/routine.rs");
        let body = std::fs::read_to_string(&routine_rs).expect("read routine.rs");
        // Strip comments to avoid false-positives on documentation prose
        // (which is allowed to mention skill_author in the AC-08 note).
        let mut active = String::with_capacity(body.len());
        for line in body.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            active.push_str(line);
            active.push('\n');
        }
        assert!(
            !active.contains("skill_author"),
            "routine.rs must not reference skill_author (AC-08): {}",
            routine_rs.display()
        );
        // Symmetric check on the routine verb block in verbs.rs — anything
        // between the routine-verbs header and the end of the routine
        // implementations. We grep more narrowly with a marker comment.
        let verbs_rs = std::path::PathBuf::from(manifest_dir).join("src/verbs.rs");
        let verbs_body = std::fs::read_to_string(&verbs_rs).expect("read verbs.rs");
        // Locate the routine verb block by the AC-08 marker line we wrote
        // into the source. The block extends from that marker to the
        // `fn parse_invocation_outcome` boundary (which sits right after
        // the routine impls).
        let start = verbs_body
            .find("// Routine verbs — per spec 2026-05-22-routine-proposals.")
            .expect("AC-08 marker present in verbs.rs");
        let end = verbs_body[start..]
            .find("fn parse_invocation_outcome(")
            .expect("routine block boundary marker missing")
            + start;
        let block = &verbs_body[start..end];
        // Strip comment-only lines.
        let mut active_block = String::with_capacity(block.len());
        for line in block.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            active_block.push_str(line);
            active_block.push('\n');
        }
        assert!(
            !active_block.contains("skill_author"),
            "routine verb block in verbs.rs must not reference skill_author (AC-08)"
        );
    }

    // ---- AC-09: content-addressed lookup by chain_id -----------------------

    #[test]
    fn routine_ac_09_archived_chain_is_not_reauthored() {
        let (_dir, layout) = fresh_conformance_layout();
        let chain = vec!["/orb:tabletop".into(), "/orb:spec".into()];
        // Author once.
        let first = match execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: chain.clone(),
                name: Some("first".into()),
                description: None,
                body: None,
                timestamp: Some("2026-05-22T10:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap()
        {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        assert!(first.written);

        // Simulate curator archive: git-mv-equivalent move to .archive/.
        let archive_dir = layout.claude_skills_archive_dir().join("first");
        std::fs::create_dir_all(archive_dir.parent().unwrap()).unwrap();
        let live_dir = layout.claude_skills_dir().join("first");
        std::fs::rename(&live_dir, &archive_dir).unwrap();
        assert!(!live_dir.exists());
        assert!(archive_dir.join("SKILL.md").exists());

        // Author again with the same chain — must be a no-op pointing at
        // the archived file.
        let second = match execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: chain.clone(),
                name: Some("second".into()),
                description: None,
                body: None,
                timestamp: Some("2026-05-23T10:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap()
        {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        assert!(!second.written, "must not re-author after archive");
        assert!(second.path.contains(".archive"));
    }

    #[test]
    fn routine_ac_09_renamed_directory_still_matches() {
        let (_dir, layout) = fresh_conformance_layout();
        let chain = vec!["/orb:a".into(), "/orb:b".into()];
        // Author.
        let first = match execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: chain.clone(),
                name: Some("original-name".into()),
                description: None,
                body: None,
                timestamp: Some("2026-05-22T10:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap()
        {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        // Simulate author rename of the routine directory.
        let original = layout.claude_skills_dir().join("original-name");
        let renamed = layout.claude_skills_dir().join("renamed-by-author");
        std::fs::rename(&original, &renamed).unwrap();

        // Second author with the same chain — must dedupe by chain_id
        // (path-independent).
        let second = match execute(
            &layout,
            &VerbRequest::RoutineAuthor(RoutineAuthorArgs {
                chain: chain.clone(),
                name: Some("trying-again".into()),
                description: None,
                body: None,
                timestamp: Some("2026-05-23T10:00:00Z".into()),
                occurrences: None,
            }),
        )
        .unwrap()
        {
            VerbResponse::RoutineAuthor(r) => r,
            other => panic!("{other:?}"),
        };
        assert!(!second.written, "rename must not break chain_id dedupe");
        assert_eq!(second.chain_id, first.chain_id);
        assert!(second.path.contains("renamed-by-author"));
    }

    // -----------------------------------------------------------------
    // classify_substrate_layout — spec 2026-05-24-setup-is-orbit-state-aware
    // -----------------------------------------------------------------

    fn classify_at(root: &std::path::Path) -> SubstrateLayoutState {
        classify_substrate_layout(&OrbitLayout::at(root))
    }

    #[test]
    fn classifier_returns_greenfield_for_empty_tree() {
        let dir = tempdir().unwrap();
        assert_eq!(classify_at(dir.path()), SubstrateLayoutState::Greenfield);
    }

    #[test]
    fn classifier_returns_idempotent_for_dotted_only() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();
        assert_eq!(classify_at(dir.path()), SubstrateLayoutState::Idempotent);
    }

    #[test]
    fn classifier_returns_wrapped_undotted_for_bare_orbit_dir() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("orbit/cards")).unwrap();
        assert_eq!(
            classify_at(dir.path()),
            SubstrateLayoutState::WrappedUndotted
        );
    }

    #[test]
    fn classifier_returns_brownfield_bare_for_root_bare_dirs() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("cards")).unwrap();
        std::fs::create_dir_all(dir.path().join("specs")).unwrap();
        assert_eq!(
            classify_at(dir.path()),
            SubstrateLayoutState::BrownfieldBare
        );
    }

    #[test]
    fn classifier_returns_mixed_bare_when_dotted_and_bare_coexist() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();
        std::fs::create_dir_all(dir.path().join("cards")).unwrap();
        assert_eq!(classify_at(dir.path()), SubstrateLayoutState::MixedBare);
    }

    #[test]
    fn classifier_returns_mixed_undotted_when_dotted_and_orbit_coexist() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join(".orbit")).unwrap();
        std::fs::create_dir_all(dir.path().join("orbit")).unwrap();
        assert_eq!(
            classify_at(dir.path()),
            SubstrateLayoutState::MixedUndotted
        );
    }

    #[test]
    fn classifier_collision_orbit_present_dominates_bare() {
        // wrapped-undotted dominates the bare-dir signal — when orbit/
        // exists, the migration arm is the single-rename path and any
        // root-level bare dirs are part of the wrapped tree the operator
        // hasn't yet cleaned up. Classifier returns WrappedUndotted, not
        // MixedBare.
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("orbit")).unwrap();
        std::fs::create_dir_all(dir.path().join("cards")).unwrap();
        assert_eq!(
            classify_at(dir.path()),
            SubstrateLayoutState::WrappedUndotted
        );
    }

    // -----------------------------------------------------------------
    // decisions_md_unmigrated_findings — spec 2026-05-24-setup-is-orbit-state-aware ac-15
    // -----------------------------------------------------------------

    #[test]
    fn decisions_md_finding_fires_for_unconverted_file() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let decisions = dir.path().join(".orbit/decisions");
        std::fs::create_dir_all(&decisions).unwrap();
        std::fs::write(
            decisions.join("0001-some-choice.md"),
            "# Title\n\nA legacy MADR.\n",
        )
        .unwrap();

        let findings = decisions_md_unmigrated_findings(&layout).unwrap();
        assert_eq!(findings.len(), 1);
        let f = &findings[0];
        assert_eq!(f.subsystem, "setup");
        assert_eq!(f.state, "decisions_md_unmigrated");
        assert!(
            f.subject.ends_with(".orbit/decisions/0001-some-choice.md"),
            "subject should be the relative md path, got {}",
            f.subject
        );
        assert!(f.remediation.verb.contains("MD→YAML"));
    }

    #[test]
    fn decisions_md_finding_suppressed_when_choice_yaml_exists() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let decisions = dir.path().join(".orbit/decisions");
        std::fs::create_dir_all(&decisions).unwrap();
        std::fs::write(decisions.join("0001-x.md"), "legacy").unwrap();
        // Operator already migrated this one to YAML.
        std::fs::write(
            layout.choice_file("0001-x"),
            "id: 0001-x\ntitle: x\nstatus: accepted\ndate_created: 2026-05-24\nbody: ok\n",
        )
        .unwrap();

        let findings = decisions_md_unmigrated_findings(&layout).unwrap();
        assert!(
            findings.is_empty(),
            "matched .orbit/choices/<slug>.yaml suppresses the finding, got: {findings:?}"
        );
    }

    #[test]
    fn decisions_md_finding_empty_when_decisions_dir_absent() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // No .orbit/decisions/ dir at all.
        let findings = decisions_md_unmigrated_findings(&layout).unwrap();
        assert!(findings.is_empty());
    }

    // -----------------------------------------------------------------
    // topology_setup plugin_repo gating — spec 2026-05-24-setup-is-orbit-state-aware ac-12 / ac-13
    // -----------------------------------------------------------------

    #[test]
    fn topology_setup_writes_readme_when_plugin_repo_unset() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let result = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(
            result.readme_created,
            "non-plugin-repo branch must write README, got result={result:?}"
        );
        assert!(
            result.seeds_created.is_empty(),
            "non-plugin-repo branch must NOT write substrate-typed seeds"
        );
        assert!(
            layout.topology_dir().join("README.md").exists(),
            "README.md must land on disk"
        );
    }

    #[test]
    fn topology_setup_idempotent_readme_branch() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        let first = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(first.readme_created);
        let second = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(!second.readme_created, "re-run must be a no-op");
    }

    #[test]
    fn topology_setup_writes_seeds_when_plugin_repo_true_and_paths_exist() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        // Set plugin_repo: true.
        std::fs::write(
            layout.config_file(),
            "plugin_repo: true\n",
        )
        .unwrap();
        // Create the substrate-typed seeds' canonical_code path so
        // validation passes.
        let code_dir = dir.path().join("orbit-state/crates/core/src");
        std::fs::create_dir_all(&code_dir).unwrap();
        std::fs::write(code_dir.join("schema.rs"), "// stub").unwrap();

        let result = topology_setup(&layout, &TopologySetupArgs::default()).unwrap();
        assert!(
            !result.readme_created,
            "plugin-repo branch must NOT write README"
        );
        assert_eq!(
            result.seeds_created.len(),
            5,
            "plugin-repo branch must write 5 substrate-typed seeds"
        );
    }

    #[test]
    fn topology_setup_rejects_when_plugin_repo_true_and_canonical_code_missing() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        layout.ensure_dirs().unwrap();
        std::fs::write(layout.config_file(), "plugin_repo: true\n").unwrap();
        // Deliberately do NOT create orbit-state/crates/core/src/schema.rs —
        // canonical_code validation must refuse.

        let err = topology_setup(&layout, &TopologySetupArgs::default()).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("canonical_code") && msg.contains("does not exist"),
            "error must name the missing canonical_code path, got: {msg}"
        );
    }

    // -----------------------------------------------------------------
    // substrate.classify verb dispatch — ac-11 + ac-18
    // -----------------------------------------------------------------

    #[test]
    fn substrate_classify_verb_returns_greenfield_for_empty_tree() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        let response = execute(
            &layout,
            &VerbRequest::SubstrateClassify(SubstrateClassifyArgs::default()),
        )
        .unwrap();
        let result = match response {
            VerbResponse::SubstrateClassify(r) => r,
            other => panic!("unexpected response: {other:?}"),
        };
        assert_eq!(result.state, SubstrateLayoutState::Greenfield);
    }

    #[test]
    fn substrate_classify_verb_returns_wrapped_undotted() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("orbit/cards")).unwrap();
        let layout = OrbitLayout::at(dir.path());
        let response = execute(
            &layout,
            &VerbRequest::SubstrateClassify(SubstrateClassifyArgs::default()),
        )
        .unwrap();
        let result = match response {
            VerbResponse::SubstrateClassify(r) => r,
            other => panic!("unexpected response: {other:?}"),
        };
        assert_eq!(result.state, SubstrateLayoutState::WrappedUndotted);
    }

    // -----------------------------------------------------------------
    // ac-20: cross-drive canonical-files-missing suppression on
    // wrapped-undotted. This drive ships the suppression mechanism using
    // its own classifier — when the layout is wrapped-undotted, the
    // missing canonical files are derivative of the layout problem, not
    // separate findings. Sister drive 2026-05-24-workflow-conformance
    // ships the undotted_substrate finding family that surfaces the
    // layout issue itself.
    // -----------------------------------------------------------------

    #[test]
    fn audit_conformance_suppresses_canonical_files_missing_on_wrapped_undotted() {
        let dir = tempdir().unwrap();
        // Wrapped-undotted shape: orbit/ exists, no .orbit/.
        std::fs::create_dir_all(dir.path().join("orbit/cards")).unwrap();
        let layout = OrbitLayout::at(dir.path());
        // No .orbit/METHOD.md or .orbit/STYLE.md exist — without the
        // suppression, canonical_file_findings fires "missing" for both.
        let response = execute(
            &layout,
            &VerbRequest::AuditConformance(AuditConformanceArgs::default()),
        )
        .unwrap();
        let result = match response {
            VerbResponse::AuditConformance(r) => r,
            other => panic!("unexpected response: {other:?}"),
        };
        let canonical_missing: Vec<&ConformanceFinding> = result
            .findings
            .iter()
            .filter(|f| f.subsystem == "setup" && f.state == "missing")
            .collect();
        assert!(
            canonical_missing.is_empty(),
            "canonical-files-missing must be suppressed on a wrapped-undotted repo \
             (sister drive contract), got: {canonical_missing:?}"
        );
    }

    // -----------------------------------------------------------------
    // undotted_substrate_finding — spec 2026-05-24-workflow-conformance
    // -----------------------------------------------------------------

    #[test]
    fn undotted_finding_fires_when_orbit_present_and_dotted_absent() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("orbit/cards")).unwrap();
        std::fs::create_dir_all(dir.path().join("orbit/specs")).unwrap();
        std::fs::write(dir.path().join("orbit/cards/0001-x.yaml"), "id: 0001-x\n").unwrap();
        std::fs::write(dir.path().join("orbit/cards/0002-y.yaml"), "id: 0002-y\n").unwrap();
        let layout = OrbitLayout::at(dir.path());
        let f = undotted_substrate_finding(&layout)
            .expect("finding must fire when orbit/ present and .orbit/cards/ absent");
        assert_eq!(f.severity, "high");
        assert_eq!(f.subsystem, "setup");
        assert_eq!(f.state, "undotted_substrate");
        assert_eq!(f.subject, "orbit/");
        assert_eq!(f.remediation.verb, "orbit setup");
        // Evidence carries the four per-subdir counts.
        let evidence = f.evidence.as_ref().expect("evidence map");
        let map = evidence.as_mapping().expect("evidence is a mapping");
        let cards_count = map
            .get(serde_yaml::Value::String("cards_count".into()))
            .and_then(|v| v.as_i64())
            .unwrap();
        assert_eq!(cards_count, 2, "two card files seeded in orbit/cards/");
    }

    #[test]
    fn undotted_finding_suppressed_when_canonical_substrate_present() {
        let dir = tempdir().unwrap();
        // Both orbit/cards/ AND .orbit/cards/ — the negative guard wins.
        std::fs::create_dir_all(dir.path().join("orbit/cards")).unwrap();
        std::fs::create_dir_all(dir.path().join(".orbit/cards")).unwrap();
        let layout = OrbitLayout::at(dir.path());
        assert!(
            undotted_substrate_finding(&layout).is_none(),
            "negative guard: .orbit/cards/ presence suppresses the finding"
        );
    }

    #[test]
    fn undotted_finding_absent_on_greenfield() {
        let dir = tempdir().unwrap();
        let layout = OrbitLayout::at(dir.path());
        assert!(undotted_substrate_finding(&layout).is_none());
    }

    #[test]
    fn undotted_finding_absent_when_only_orbit_dir_with_no_substrate_subdirs() {
        let dir = tempdir().unwrap();
        // orbit/ exists but no substrate subdirs — could be a build artefact,
        // a workspace member, anything. Don't false-positive.
        std::fs::create_dir_all(dir.path().join("orbit")).unwrap();
        let layout = OrbitLayout::at(dir.path());
        assert!(
            undotted_substrate_finding(&layout).is_none(),
            "bare orbit/ dir without substrate subdirs must not fire"
        );
    }

    #[test]
    fn audit_conformance_emits_undotted_substrate_and_suppresses_others_on_wrapped_undotted() {
        let dir = tempdir().unwrap();
        std::fs::create_dir_all(dir.path().join("orbit/cards")).unwrap();
        let layout = OrbitLayout::at(dir.path());
        let response = execute(
            &layout,
            &VerbRequest::AuditConformance(AuditConformanceArgs::default()),
        )
        .unwrap();
        let result = match response {
            VerbResponse::AuditConformance(r) => r,
            other => panic!("unexpected response: {other:?}"),
        };
        // Exactly one conformance finding: undotted_substrate.
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].state, "undotted_substrate");
        assert_eq!(result.findings[0].severity, "high");
        // All other .orbit/-dependent families suppressed.
        for f in &result.findings {
            assert_ne!(f.state, "missing");
            assert_ne!(f.state, "ready_for_tabletop");
            assert_ne!(f.state, "stale");
            assert_ne!(f.state, "pin_behind");
            assert_ne!(f.state, "pin_ahead");
            assert_ne!(f.state, "decisions_md_unmigrated");
        }
    }
}
