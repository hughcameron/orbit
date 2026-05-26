//! Shared fixtures for parity tests.
//!
//! Mirror of `crates/cli/tests/common/mod.rs` — both copies MUST agree on
//! the fixture and the expected envelope. The parity claim: both surfaces
//! produce the same bytes core would produce for the same input. Both
//! surface tests assert against the same library-computed reference.

use orbit_state_core::{envelope_ok_string, SpecListResult, SpecSummary, VerbResponse};
use std::path::Path;

/// Populate `<root>/.orbit/specs/` with two specs (folder layout per
/// choice 0021):
/// - `0001/spec.yaml` — open, "first spec"
/// - `0002/spec.yaml` — closed, "second spec"
pub fn populate_two_specs(root: &Path) {
    let specs_dir = root.join(".orbit/specs");
    for (id, body) in [
        ("0001", "id: '0001'\ngoal: first spec\nstatus: open\n"),
        ("0002", "id: '0002'\ngoal: second spec\nstatus: closed\n"),
    ] {
        let folder = specs_dir.join(id);
        std::fs::create_dir_all(&folder).unwrap();
        std::fs::write(folder.join("spec.yaml"), body).unwrap();
    }
}

/// The canonical envelope expected from `spec.list` against the two-spec
/// fixture, computed by the same library helper that the surfaces use.
pub fn expected_envelope_for_two_specs() -> String {
    let response = VerbResponse::SpecList(SpecListResult {
        specs: vec![
            SpecSummary {
                id: "0001".into(),
                goal: "first spec".into(),
                status: "open".into(),
                cards: vec![],
                labels: vec![],
            },
            SpecSummary {
                id: "0002".into(),
                goal: "second spec".into(),
                status: "closed".into(),
                cards: vec![],
                labels: vec![],
            },
        ],
    });
    envelope_ok_string(&response).expect("envelope serialisation infallible for fixture")
}

/// The expected error envelope for `--status nope`.
pub fn expected_envelope_for_invalid_status() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::malformed("spec.list", "status must be 'open' or 'closed', got 'nope'");
    envelope_err_string(&err)
}

/// Expected envelope for `spec.show 0001` against the two-spec fixture.
pub fn expected_envelope_for_spec_show_0001() -> String {
    use orbit_state_core::schema::{Spec, SpecStatus};
    use orbit_state_core::{SpecShowResult, VerbResponse};
    let response = VerbResponse::SpecShow(SpecShowResult {
        spec: Spec {
            id: "0001".into(),
            goal: "first spec".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: vec![],
            memories_considered: vec![],
            closed_at: None,
        },
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected error envelope for `spec.show 0099` (not present).
pub fn expected_envelope_for_spec_show_missing(root: &Path) -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let path = root.join(".orbit/specs/0099/spec.yaml");
    let err = Error::not_found("spec.show", format!("no spec at {}", path.display()));
    envelope_err_string(&err)
}

/// The deterministic note used by spec.note parity tests. MUST match
/// `crates/cli/tests/common/mod.rs::fixture_note` so both surface tests
/// assert against the same library-computed reference.
pub fn fixture_note() -> orbit_state_core::schema::NoteEvent {
    use orbit_state_core::schema::NoteEvent;
    NoteEvent {
        spec_id: "0001".into(),
        body: "parity test note".into(),
        labels: vec!["test".into()],
        timestamp: "2026-05-07T12:00:00Z".into(),
    }
}

pub fn expected_envelope_for_fixture_note() -> String {
    use orbit_state_core::{SpecNoteResult, VerbResponse};
    let response = VerbResponse::SpecNote(SpecNoteResult { note: fixture_note() });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

pub fn expected_notes_jsonl_for_fixture_note() -> String {
    orbit_state_core::canonical::serialise_json_line(&fixture_note())
        .expect("serialise_json_line is infallible for fixture")
}

/// Populate `<root>/.orbit/cards/` with two cards joined by a `feeds`
/// relation: 0001-alpha → 0002-beta. Used by card.tree parity tests.
pub fn populate_two_related_cards(root: &Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0001-alpha.yaml"),
        "id: 0001-alpha\nfeature: alpha\ngoal: alpha goal\nmaturity: planned\nrelations:\n- card: 0002-beta\n  type: feeds\n  reason: alpha feeds beta\n",
    )
    .unwrap();
    std::fs::write(
        cards_dir.join("0002-beta.yaml"),
        "id: 0002-beta\nfeature: beta\ngoal: beta goal\nmaturity: planned\n",
    )
    .unwrap();
}

/// Expected canonical envelope for `card.tree` with `slug=0001-alpha` and
/// `depth=1` against the two-related-cards fixture.
pub fn expected_envelope_for_card_tree_alpha_depth1() -> String {
    use orbit_state_core::{CardTreeEdge, CardTreeNode, CardTreeResult, VerbResponse};
    let response = VerbResponse::CardTree(CardTreeResult {
        root: "0001-alpha".into(),
        depth: 1,
        tree: CardTreeNode {
            slug: "0001-alpha".into(),
            feature: "alpha".into(),
            outgoing: vec![CardTreeEdge {
                kind: "feeds".into(),
                reason: "alpha feeds beta".into(),
                target: CardTreeNode {
                    slug: "0002-beta".into(),
                    feature: "beta".into(),
                    outgoing: vec![],
                    incoming: vec![],
                    truncated: true,
                },
            }],
            incoming: vec![],
            truncated: false,
        },
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected error envelope for `card.tree` with an unknown numeric id.
pub fn expected_envelope_for_card_tree_unknown(cards_dir: &Path) -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::not_found(
        "card.tree",
        format!("no entry matching `9999-*` in {}", cards_dir.display()),
    );
    envelope_err_string(&err)
}

pub fn expected_envelope_for_card_specs_unknown(cards_dir: &Path) -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::not_found(
        "card.specs",
        format!("no entry matching `9999-*` in {}", cards_dir.display()),
    );
    envelope_err_string(&err)
}

pub fn expected_envelope_for_graph_unknown(cards_dir: &Path) -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::not_found(
        "graph",
        format!("no entry matching `9999-*` in {}", cards_dir.display()),
    );
    envelope_err_string(&err)
}

/// Populate `<root>/.orbit/` with one card (`0001-alpha`) listing a spec
/// (`s1`) that back-references it. Used by card.specs parity tests.
pub fn populate_card_with_linked_spec(root: &Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0001-alpha.yaml"),
        "id: 0001-alpha\nfeature: alpha\ngoal: alpha goal\nmaturity: planned\nspecs:\n- .orbit/specs/s1/spec.yaml\n",
    )
    .unwrap();
    let spec_dir = root.join(".orbit/specs/s1");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: s1\ngoal: spec one\ncards:\n- 0001-alpha\nstatus: open\n",
    )
    .unwrap();
}

/// Populate `<root>/.orbit/cards/0001-alpha.yaml` with a top-level unknown
/// field. Used by audit.drift parity tests.
pub fn populate_card_with_drift(root: &Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0001-alpha.yaml"),
        "id: 0001-alpha\nfeature: alpha\ngoal: alpha goal\nmaturity: planned\nlegacy_field: x\n",
    )
    .unwrap();
}

/// Expected canonical envelope for `audit.drift` against the
/// card-with-drift fixture.
pub fn expected_envelope_for_audit_drift_one_unknown() -> String {
    use orbit_state_core::{AuditDriftResult, DriftEntry, VerbResponse};
    let response = VerbResponse::AuditDrift(AuditDriftResult {
        drift: vec![DriftEntry {
            path: ".orbit/cards/0001-alpha.yaml".into(),
            kind: "card".into(),
            field: "legacy_field".into(),
            disposition: "quarantine".into(),
        }],
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Populate `<root>/.orbit/` with the canonical METHOD.md and STYLE.md
/// so `audit.conformance` returns zero findings (parity baseline for
/// the new verb). Per spec 2026-05-19-workflow-conformance ac-01,
/// extended for STYLE.md per spec 2026-05-20-style-md-plugin-shipping.
pub fn populate_conformance_clean_fixture(root: &Path) {
    let orbit_dir = root.join(".orbit");
    std::fs::create_dir_all(&orbit_dir).unwrap();
    let method = include_str!("../../../../../plugins/orb/skills/setup/METHOD.md");
    std::fs::write(orbit_dir.join("METHOD.md"), method).unwrap();
    let style = include_str!("../../../../../plugins/orb/skills/setup/STYLE.md");
    std::fs::write(orbit_dir.join("STYLE.md"), style).unwrap();
}

/// Populate `<root>/.orbit/` with the canonical METHOD.md + STYLE.md,
/// one parked card, and one planned-empty non-park card. Used by the
/// park-signal parity test (spec 2026-05-20-conformance-park-signal ac-02):
/// the parked card should produce no card-state finding, only the
/// non-park card fires.
pub fn populate_conformance_park_signal_fixture(root: &Path) {
    populate_conformance_clean_fixture(root);
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    let parked = "id: 0099-parked\n\
feature: parked feature\n\
goal: parked goal\n\
maturity: planned\n\
park:\n  \
  reason: awaiting third use-case forcing\n  \
  until: N=2 evidence\n\
scenarios: []\n\
specs: []\n\
relations: []\n\
references: []\n\
notes: []\n";
    std::fs::write(cards_dir.join("0099-parked.yaml"), parked).unwrap();
    let active = "id: 0098-planned\n\
feature: active feature\n\
goal: active goal\n\
maturity: planned\n\
scenarios: []\n\
specs: []\n\
relations: []\n\
references: []\n\
notes: []\n";
    std::fs::write(cards_dir.join("0098-planned.yaml"), active).unwrap();
}

/// Expected canonical envelope for `audit.conformance` against the
/// park-signal fixture: exactly one card-state finding for the
/// non-parked card, zero envelope trace for the parked card. Derived
/// via `execute(...)` against the live fixture so the expected reference
/// tracks the library's serialisation contract.
pub fn expected_envelope_for_audit_conformance_park_signal_fixture(root: &Path) -> String {
    use orbit_state_core::{execute, AuditConformanceArgs, VerbRequest, VerbResponse};
    use orbit_state_core::layout::OrbitLayout;
    let layout = OrbitLayout::at_orbit_dir(&root.join(".orbit"));
    let request = VerbRequest::AuditConformance(AuditConformanceArgs::default());
    let response = execute(&layout, &request).expect("audit conformance must succeed on fixture");
    match response {
        VerbResponse::AuditConformance(ref r) => {
            assert_eq!(
                r.findings.len(),
                1,
                "park-signal fixture: expected exactly 1 finding (the non-park card), got {}",
                r.findings.len(),
            );
            assert_eq!(r.findings[0].subject, "0098-planned");
            assert_eq!(r.findings[0].state, "ready_for_tabletop");
        }
        _ => panic!("unexpected response shape"),
    }
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Populate `<root>/` to land the substrate-layout classifier in the
/// `idempotent` state. Mirrors the CLI common helper of the same name —
/// both surfaces test against the same fixture shapes. Per spec
/// 2026-05-24-setup-is-orbit-state-aware ac-18.
pub fn populate_substrate_idempotent_fixture(root: &Path) {
    std::fs::create_dir_all(root.join(".orbit")).unwrap();
}

/// Populate `<root>/` to land the classifier in the `wrapped-undotted`
/// state. Per spec 2026-05-24-setup-is-orbit-state-aware ac-18.
pub fn populate_substrate_wrapped_undotted_fixture(root: &Path) {
    std::fs::create_dir_all(root.join("orbit/cards")).unwrap();
}

/// Populate `<root>/` to land the classifier in the `brownfield-bare`
/// state. Per spec 2026-05-24-setup-is-orbit-state-aware ac-18.
pub fn populate_substrate_brownfield_bare_fixture(root: &Path) {
    std::fs::create_dir_all(root.join("cards")).unwrap();
    std::fs::create_dir_all(root.join("specs")).unwrap();
}

/// Expected canonical envelope for `substrate classify` with the named
/// layout state. Single helper shared by CLI and MCP parity tests so
/// the byte-equality contract is enforced against one reference. Per
/// spec 2026-05-24-setup-is-orbit-state-aware ac-18.
pub fn expected_envelope_for_substrate_classify(
    state: orbit_state_core::SubstrateLayoutState,
) -> String {
    use orbit_state_core::{SubstrateClassifyResult, VerbResponse};
    let response =
        VerbResponse::SubstrateClassify(SubstrateClassifyResult { state });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected canonical envelope for `audit conformance` against the
/// wrapped-undotted fixture: exactly one finding (undotted_substrate,
/// HIGH, subject `orbit/`), every `.orbit/`-dependent family suppressed.
/// Per spec 2026-05-24-workflow-conformance.
pub fn expected_envelope_for_audit_conformance_wrapped_undotted(root: &Path) -> String {
    use orbit_state_core::layout::OrbitLayout;
    use orbit_state_core::{execute, AuditConformanceArgs, VerbRequest, VerbResponse};
    let layout = OrbitLayout::at_orbit_dir(&root.join(".orbit"));
    let request = VerbRequest::AuditConformance(AuditConformanceArgs::default());
    let response =
        execute(&layout, &request).expect("audit conformance must succeed on fixture");
    match response {
        VerbResponse::AuditConformance(ref r) => {
            assert_eq!(r.findings.len(), 1);
            assert_eq!(r.findings[0].state, "undotted_substrate");
            assert_eq!(r.findings[0].severity, "high");
            assert_eq!(r.findings[0].subject, "orbit/");
        }
        _ => panic!("unexpected response shape"),
    }
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected canonical envelope for `audit.conformance` against the
/// conformance-clean fixture: empty findings, drift clean, topology
/// unconfigured, pin unpinned.
pub fn expected_envelope_for_audit_conformance_clean() -> String {
    use orbit_state_core::{
        AggregatedAudits, AuditConformanceResult, AuditDriftResult, AuditTopologyResult, PinState,
        VerbResponse,
    };
    let response = VerbResponse::AuditConformance(AuditConformanceResult {
        findings: vec![],
        aggregated: AggregatedAudits {
            drift: AuditDriftResult { drift: vec![] },
            topology: AuditTopologyResult {
                configured: false,
                topology_drift: vec![],
            },
        },
        pin: PinState {
            pinned: None,
            current: env!("CARGO_PKG_VERSION").to_string(),
            status: "unpinned".into(),
        },
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected canonical envelope for `graph` (mermaid, unscoped) against
/// the two-related-cards fixture.
pub fn expected_envelope_for_graph_mermaid_two_related_cards() -> String {
    use orbit_state_core::{GraphResult, VerbResponse};
    let text = String::from(
        "graph LR\n\
         \x20\x20c_0001_alpha[\"0001-alpha: alpha\"]\n\
         \x20\x20c_0002_beta[\"0002-beta: beta\"]\n\
         \x20\x20c_0001_alpha -->|feeds| c_0002_beta\n",
    );
    let response = VerbResponse::Graph(GraphResult {
        format: "mermaid".into(),
        text,
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected canonical envelope for `overview` against the two-related-cards
/// fixture (alpha feeds beta; both planned; no specs; no memories).
pub fn expected_envelope_for_overview_two_related_cards() -> String {
    use orbit_state_core::{
        CardMaturityCounts, MostConnectedCard, OverviewResult, VerbResponse,
    };
    let response = VerbResponse::Overview(OverviewResult {
        open_spec_count: 0,
        recent_open_spec_ids: vec![],
        spec_overflow: 0,
        cards_by_maturity: CardMaturityCounts {
            planned: 2,
            emerging: 0,
            established: 0,
        },
        memories: vec![],
        most_connected_card: Some(MostConnectedCard {
            slug: "0001-alpha".into(),
            feature: "alpha".into(),
            degree: 1,
        }),
        orphans: vec!["0001-alpha".into()],
        orphan_overflow: 0,
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

/// Expected canonical envelope for `card.specs` with `slug=0001-alpha`.
pub fn expected_envelope_for_card_specs_alpha() -> String {
    use orbit_state_core::{CardSpecsEntry, CardSpecsResult, VerbResponse};
    let response = VerbResponse::CardSpecs(CardSpecsResult {
        root: "0001-alpha".into(),
        specs: vec![CardSpecsEntry {
            spec_id: "s1".into(),
            spec_path: ".orbit/specs/s1/spec.yaml".into(),
            listed_on_card: true,
            back_referenced_by_spec: true,
            status: "open".into(),
        }],
    });
    orbit_state_core::envelope_ok_string(&response).expect("infallible")
}

// ---------------------------------------------------------------------------
// spec.close AC pre-flight (spec 2026-05-13-spec-close-ac-preflight, ac-05)
// ---------------------------------------------------------------------------

/// Populate `<root>/.orbit/` with one card and one open spec carrying ACs:
/// - `ac-01` checked, non-gate, non-time-gated
/// - `ac-02` unchecked, non-gate, non-time-gated  ← blocks close
/// - `ac-03` unchecked, non-gate, time-gated      ← reported, does not block
pub fn populate_spec_close_preflight_fixture(root: &Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0020-orbit-state.yaml"),
        "id: 0020-orbit-state\nfeature: orbit-state\ngoal: substrate\nmaturity: planned\n",
    )
    .unwrap();
    let spec_dir = root.join(".orbit/specs/0001");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: '0001'\n\
         goal: g\n\
         cards:\n\
         - 0020-orbit-state\n\
         status: open\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: first\n  gate: false\n  checked: true\n\
         - id: ac-02\n  description: second\n  gate: false\n  checked: false\n\
         - id: ac-03\n  description: third\n  gate: false\n  checked: false\n  ac_type: observation\n",
    )
    .unwrap();
}

/// Populate a fixture where only a deferrable-kind AC remains unchecked —
/// used to verify spec.close succeeds without `--force` when the sole open
/// AC is `ac_type: observation` (spec 2026-05-16-ac-taxonomy ac-02).
pub fn populate_spec_close_only_deferrable_fixture(root: &Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0020-orbit-state.yaml"),
        "id: 0020-orbit-state\nfeature: orbit-state\ngoal: substrate\nmaturity: planned\n",
    )
    .unwrap();
    let spec_dir = root.join(".orbit/specs/0001");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: '0001'\n\
         goal: g\n\
         cards:\n\
         - 0020-orbit-state\n\
         status: open\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: first\n  gate: false\n  checked: true\n\
         - id: ac-02\n  description: second\n  gate: false\n  checked: false\n  ac_type: observation\n",
    )
    .unwrap();
}

/// Expected error envelope when `spec close 0001` runs against the
/// pre-flight fixture (ac-02 is unchecked, blocking-kind).
pub fn expected_envelope_for_spec_close_unchecked_blocking() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::conflict("spec.close", "1 unchecked blocking AC(s) in spec '0001': ac-02");
    envelope_err_string(&err)
}

/// Expected ok envelope when `spec close --force 0001` runs against the
/// pre-flight fixture. The closed spec includes the new fields:
/// `forced_unchecked: [ac-02]`, `deferrable_open: [ac-03]`.
pub fn expected_envelope_for_spec_close_force() -> String {
    use orbit_state_core::schema::{AcType, AcceptanceCriterion, Spec, SpecStatus};
    use orbit_state_core::{envelope_ok_string, SpecCloseResult, VerbResponse};
    let response = VerbResponse::SpecClose(SpecCloseResult {
        spec: Spec {
            id: "0001".into(),
            goal: "g".into(),
            cards: vec!["0020-orbit-state".into()],
            status: SpecStatus::Closed,
            labels: vec![],
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    id: "ac-01".into(),
                    description: "first".into(),
                    gate: false,
                    checked: true,
                    verification: None,
                    ac_type: AcType::Code,
                },
                AcceptanceCriterion {
                    id: "ac-02".into(),
                    description: "second".into(),
                    gate: false,
                    checked: false,
                    verification: None,
                    ac_type: AcType::Code,
                },
                AcceptanceCriterion {
                    id: "ac-03".into(),
                    description: "third".into(),
                    gate: false,
                    checked: false,
                    verification: None,
                    ac_type: AcType::Observation,
                },
            ],
            memories_considered: vec![],
            closed_at: None,
        },
        cards_updated: vec!["0020-orbit-state".into()],
        forced_unchecked: vec!["ac-02".into()],
        deferrable_open: vec!["ac-03".into()],
        forced_unreconciled: vec![],
        topology_warnings: vec![],
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected ok envelope when `spec close 0001` runs against the
/// only-deferrable fixture. Closure succeeds without `--force`;
/// `deferrable_open: [ac-02]`, `forced_unchecked` empty.
pub fn expected_envelope_for_spec_close_only_deferrable() -> String {
    use orbit_state_core::schema::{AcType, AcceptanceCriterion, Spec, SpecStatus};
    use orbit_state_core::{envelope_ok_string, SpecCloseResult, VerbResponse};
    let response = VerbResponse::SpecClose(SpecCloseResult {
        spec: Spec {
            id: "0001".into(),
            goal: "g".into(),
            cards: vec!["0020-orbit-state".into()],
            status: SpecStatus::Closed,
            labels: vec![],
            acceptance_criteria: vec![
                AcceptanceCriterion {
                    id: "ac-01".into(),
                    description: "first".into(),
                    gate: false,
                    checked: true,
                    verification: None,
                    ac_type: AcType::Code,
                },
                AcceptanceCriterion {
                    id: "ac-02".into(),
                    description: "second".into(),
                    gate: false,
                    checked: false,
                    verification: None,
                    ac_type: AcType::Observation,
                },
            ],
            memories_considered: vec![],
            closed_at: None,
        },
        cards_updated: vec!["0020-orbit-state".into()],
        forced_unchecked: vec![],
        deferrable_open: vec!["ac-02".into()],
        forced_unreconciled: vec![],
        topology_warnings: vec![],
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Fixed UUID for deterministic `session.start` parity tests.
pub const PARITY_SESSION_ID: &str = "00000000-0000-4000-8000-000000000001";

/// Fixed timestamp for deterministic skill-invocation parity tests.
pub const PARITY_TIMESTAMP: &str = "2026-05-15T12:00:00Z";

/// Expected ok envelope for `session start --id <PARITY_SESSION_ID>`
/// against the given root.
pub fn expected_envelope_for_session_start(root: &Path) -> String {
    use orbit_state_core::{envelope_ok_string, SessionStartResult, VerbResponse};
    let path = root.join(".orbit").join(".session-id");
    let response = VerbResponse::SessionStart(SessionStartResult {
        session_id: PARITY_SESSION_ID.into(),
        path: path.display().to_string(),
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected ok envelope for `skill record-invocation card --outcome worked
/// --session-id <PARITY_SESSION_ID> --timestamp <PARITY_TIMESTAMP>`.
pub fn expected_envelope_for_skill_record_invocation() -> String {
    use orbit_state_core::schema::{InvocationOutcome, SkillInvocation};
    use orbit_state_core::{envelope_ok_string, SkillRecordInvocationResult, VerbResponse};
    let response = VerbResponse::SkillRecordInvocation(SkillRecordInvocationResult {
        invocation: SkillInvocation {
            skill_id: "card".into(),
            session_id: PARITY_SESSION_ID.into(),
            outcome: InvocationOutcome::Worked,
            correction: None,
            timestamp: PARITY_TIMESTAMP.into(),
        },
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected ok envelope for `skill recurrence design` against an empty
/// (or absent) invocation file.
pub fn expected_envelope_for_skill_recurrence_empty() -> String {
    use orbit_state_core::{
        envelope_ok_string, RecurrenceByOutcome, SkillRecurrenceResult, VerbResponse,
    };
    let response = VerbResponse::SkillRecurrence(SkillRecurrenceResult {
        skill_id: "design".into(),
        by_outcome: RecurrenceByOutcome::default(),
        total: 0,
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected ok envelope for `session distill --session-id <PARITY_SESSION_ID>`
/// with the given distillate text. Caller must read `started_at` / `ended_at`
/// from disk after the call before computing this.
pub fn expected_envelope_for_session_distill(
    distillate: &str,
    started_at: &str,
    ended_at: &str,
) -> String {
    use orbit_state_core::schema::Session;
    use orbit_state_core::{envelope_ok_string, SessionDistillResult, VerbResponse};
    let response = VerbResponse::SessionDistill(SessionDistillResult {
        session: Session {
            id: PARITY_SESSION_ID.into(),
            started_at: started_at.into(),
            ended_at: Some(ended_at.into()),
            distillate: distillate.into(),
            card_id: None,
            labels: vec![],
        },
    });
    envelope_ok_string(&response).expect("infallible")
}
// ---------------------------------------------------------------------------
// spec.acs / next-ac / blocking-gate / has-unchecked / check / uncheck parity
// (per spec 2026-05-24-port-acceptance-shim ac-07).
// ---------------------------------------------------------------------------

/// Populate `<root>/.orbit/specs/0010/spec.yaml` with a mixed AC roster —
/// one unchecked gate (ac-01), one unchecked non-gate (ac-02), one checked
/// non-gate (ac-03), one checked gate (ac-04). Exercises every traversal
/// branch of the new verbs.
pub fn populate_spec_acs_mixed_fixture(root: &std::path::Path) {
    let spec_dir = root.join(".orbit/specs/0010");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: '0010'\n\
         goal: mixed roster\n\
         status: open\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: first\n  gate: true\n  checked: false\n\
         - id: ac-02\n  description: second\n  gate: false\n  checked: false\n\
         - id: ac-03\n  description: third\n  gate: false\n  checked: true\n\
         - id: ac-04\n  description: fourth\n  gate: true\n  checked: true\n",
    )
    .unwrap();
}

/// Populate `<root>/.orbit/specs/0011/spec.yaml` with every AC checked —
/// drives the `has-unchecked: false` branch (CLI exit 1).
pub fn populate_spec_acs_all_checked_fixture(root: &std::path::Path) {
    let spec_dir = root.join(".orbit/specs/0011");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.yaml"),
        "id: '0011'\n\
         goal: all done\n\
         status: open\n\
         acceptance_criteria:\n\
         - id: ac-01\n  description: first\n  gate: false\n  checked: true\n\
         - id: ac-02\n  description: second\n  gate: false\n  checked: true\n",
    )
    .unwrap();
}

fn mixed_acs() -> Vec<orbit_state_core::schema::AcceptanceCriterion> {
    use orbit_state_core::schema::{AcType, AcceptanceCriterion};
    vec![
        AcceptanceCriterion {
            id: "ac-01".into(),
            description: "first".into(),
            gate: true,
            checked: false,
            verification: None,
            ac_type: AcType::Code,
        },
        AcceptanceCriterion {
            id: "ac-02".into(),
            description: "second".into(),
            gate: false,
            checked: false,
            verification: None,
            ac_type: AcType::Code,
        },
        AcceptanceCriterion {
            id: "ac-03".into(),
            description: "third".into(),
            gate: false,
            checked: true,
            verification: None,
            ac_type: AcType::Code,
        },
        AcceptanceCriterion {
            id: "ac-04".into(),
            description: "fourth".into(),
            gate: true,
            checked: true,
            verification: None,
            ac_type: AcType::Code,
        },
    ]
}

pub fn expected_envelope_for_spec_acs_mixed() -> String {
    use orbit_state_core::{envelope_ok_string, SpecAcsResult, VerbResponse};
    let response = VerbResponse::SpecAcs(SpecAcsResult { acs: mixed_acs() });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_spec_next_ac_mixed() -> String {
    use orbit_state_core::{envelope_ok_string, NextAc, SpecNextAcResult, VerbResponse};
    let response = VerbResponse::SpecNextAc(SpecNextAcResult {
        next: Some(NextAc {
            id: "ac-01".into(),
            gate: true,
        }),
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_spec_blocking_gate_mixed() -> String {
    use orbit_state_core::{
        envelope_ok_string, BlockingGate, SpecBlockingGateResult, VerbResponse,
    };
    let response = VerbResponse::SpecBlockingGate(SpecBlockingGateResult {
        blocking: Some(BlockingGate {
            id: "ac-01".into(),
            description: "first".into(),
        }),
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_spec_has_unchecked_true() -> String {
    use orbit_state_core::{envelope_ok_string, SpecHasUncheckedResult, VerbResponse};
    let response = VerbResponse::SpecHasUnchecked(SpecHasUncheckedResult { has_unchecked: true });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_spec_has_unchecked_false() -> String {
    use orbit_state_core::{envelope_ok_string, SpecHasUncheckedResult, VerbResponse};
    let response = VerbResponse::SpecHasUnchecked(SpecHasUncheckedResult { has_unchecked: false });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected envelope after `spec check 0010 ac-02` against the mixed fixture
/// — ac-02 is now checked, the rest of the spec is unchanged.
pub fn expected_envelope_for_spec_check_ac02() -> String {
    use orbit_state_core::schema::{AcType, AcceptanceCriterion, Spec, SpecStatus};
    use orbit_state_core::{envelope_ok_string, SpecCheckResult, VerbResponse};
    let mut acs = mixed_acs();
    acs[1].checked = true;
    let response = VerbResponse::SpecCheck(SpecCheckResult {
        spec: Spec {
            id: "0010".into(),
            goal: "mixed roster".into(),
            cards: vec![],
            status: SpecStatus::Open,
            labels: vec![],
            acceptance_criteria: acs,
            memories_considered: vec![],
            closed_at: None,
        },
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected error envelope: `spec check 0010 ac-99` (AC missing).
pub fn expected_envelope_for_spec_check_missing() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::not_found("spec.check", "AC ac-99 not found on spec 0010");
    envelope_err_string(&err)
}

/// Expected error envelope: `spec check 0010 ac-03` (AC already checked).
pub fn expected_envelope_for_spec_check_already_checked() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::conflict("spec.check", "AC ac-03 is already checked");
    envelope_err_string(&err)
}

// ---------------------------------------------------------------------------
// spec.promote parity (per spec 2026-05-25-port-promote-sh ac-06).
// ---------------------------------------------------------------------------

/// Populate `<root>/.orbit/cards/0050-promote-fixture.yaml` with a card
/// carrying three scenarios — mixed gate flags — so the expected envelope
/// exercises every branch of the AC-list builder.
pub fn populate_promote_card_fixture(root: &std::path::Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    std::fs::write(
        cards_dir.join("0050-promote-fixture.yaml"),
        "id: 0050-promote-fixture\n\
         feature: promote fixture\n\
         goal: a goal for promotion\n\
         maturity: planned\n\
         scenarios:\n\
         - name: first\n  given: g\n  when: w\n  then: t1\n  gate: true\n\
         - name: second\n  given: g\n  when: w\n  then: t2\n\
         - name: third\n  given: g\n  when: w\n  then: t3\n  gate: true\n",
    )
    .unwrap();
}

/// Canonical date token used by promote parity tests so the derived spec id
/// is byte-deterministic.
pub const PROMOTE_FIXTURE_TODAY: &str = "2026-05-25";

fn promote_fixture_expected_spec() -> orbit_state_core::schema::Spec {
    use orbit_state_core::schema::{AcType, AcceptanceCriterion, Spec, SpecStatus};
    Spec {
        id: format!("{PROMOTE_FIXTURE_TODAY}-promote-fixture"),
        goal: "a goal for promotion".into(),
        cards: vec!["0050-promote-fixture".into()],
        status: SpecStatus::Open,
        labels: vec![],
        acceptance_criteria: vec![
            AcceptanceCriterion {
                id: "ac-01".into(),
                description: "first — t1".into(),
                gate: true,
                checked: false,
                verification: None,
                ac_type: AcType::Code,
            },
            AcceptanceCriterion {
                id: "ac-02".into(),
                description: "second — t2".into(),
                gate: false,
                checked: false,
                verification: None,
                ac_type: AcType::Code,
            },
            AcceptanceCriterion {
                id: "ac-03".into(),
                description: "third — t3".into(),
                gate: true,
                checked: false,
                verification: None,
                ac_type: AcType::Code,
            },
        ],
        memories_considered: vec![],
        closed_at: None,
    }
}

/// Expected envelope for `spec.promote .orbit/cards/0050-promote-fixture.yaml
/// --today 2026-05-25` against the fixture (non-dry-run path).
pub fn expected_envelope_for_spec_promote_fixture() -> String {
    use orbit_state_core::{envelope_ok_string, SpecPromoteResult, VerbResponse};
    let response = VerbResponse::SpecPromote(SpecPromoteResult {
        spec: promote_fixture_expected_spec(),
        dry_run: false,
    });
    envelope_ok_string(&response).expect("infallible")
}

/// Expected envelope for `--dry-run` against the fixture — same shape as the
/// non-dry-run path EXCEPT `dry_run: true`.
pub fn expected_envelope_for_spec_promote_fixture_dry_run() -> String {
    use orbit_state_core::{envelope_ok_string, SpecPromoteResult, VerbResponse};
    let response = VerbResponse::SpecPromote(SpecPromoteResult {
        spec: promote_fixture_expected_spec(),
        dry_run: true,
    });
    envelope_ok_string(&response).expect("infallible")
}

// ---------------------------------------------------------------------------
// card.show with mixed relations (per spec
// 2026-05-25-relation-schema-choice-targets ac-06) — fixture has BOTH a
// card-target relation and a choice-target relation. Asserts the new schema
// serialises through the wire envelope correctly on both surfaces.
// ---------------------------------------------------------------------------

pub fn populate_card_with_mixed_relations(root: &std::path::Path) {
    let cards_dir = root.join(".orbit/cards");
    std::fs::create_dir_all(&cards_dir).unwrap();
    // Sibling card so the card-target relation references a real id.
    std::fs::write(
        cards_dir.join("0099-sibling.yaml"),
        "id: 0099-sibling\nfeature: sibling\ngoal: a sibling\nmaturity: planned\n",
    )
    .unwrap();
    // Subject card carrying both kinds of relation.
    std::fs::write(
        cards_dir.join("0050-mixed.yaml"),
        "id: 0050-mixed\n\
         feature: mixed-relations fixture\n\
         goal: hold one card-target and one choice-target relation\n\
         maturity: planned\n\
         relations:\n\
         - card: 0099-sibling\n  type: feeds\n  reason: feeds the sibling\n\
         - choice: '0020'\n  type: respects\n  reason: honours the policy\n",
    )
    .unwrap();
}

pub fn expected_envelope_for_card_show_mixed_relations() -> String {
    use orbit_state_core::schema::{Card, CardMaturity, Relation, RelationKind};
    use orbit_state_core::{envelope_ok_string, CardShowResult, VerbResponse};
    let response = VerbResponse::CardShow(CardShowResult {
        slug: "0050-mixed".into(),
        card: Card {
            id: Some("0050-mixed".into()),
            feature: "mixed-relations fixture".into(),
            as_a: None,
            i_want: None,
            so_that: None,
            goal: "hold one card-target and one choice-target relation".into(),
            maturity: CardMaturity::Planned,
            park: None,
            scenarios: vec![],
            specs: vec![],
            relations: vec![
                Relation {
                    card: Some("0099-sibling".into()),
                    choice: None,
                    kind: RelationKind::Feeds,
                    reason: "feeds the sibling".into(),
                },
                Relation {
                    card: None,
                    choice: Some("0020".into()),
                    kind: RelationKind::Respects,
                    reason: "honours the policy".into(),
                },
            ],
            references: vec![],
            notes: vec![],
        },
    });
    envelope_ok_string(&response).expect("infallible")
}

// ---------------------------------------------------------------------------
// setup.files parity (per spec 2026-05-25-port-setup-method-sh ac-06).
// Fixtures: greenfield, legacy-present, drift-on-method, drift-on-style.
// Shared by CLI + MCP parity assertions.
// ---------------------------------------------------------------------------

/// Canonical METHOD.md / STYLE.md contents used by all setup.files fixtures
/// so parity-test expectations are deterministic without depending on the
/// real in-plugin files.
pub const FIXTURE_CANONICAL_METHOD: &str = "# Canonical METHOD test content\n";
pub const FIXTURE_CANONICAL_STYLE: &str = "# Canonical STYLE test content\n";

/// Write the canonical METHOD.md + STYLE.md into a temp plugin-root and
/// return the per-file source paths. Used to drive setup.files without
/// touching the real plugin tree.
pub fn write_setup_files_canonicals(plugin_root: &std::path::Path) -> (std::path::PathBuf, std::path::PathBuf) {
    let setup_dir = plugin_root.join("skills/setup");
    std::fs::create_dir_all(&setup_dir).unwrap();
    let method = setup_dir.join("METHOD.md");
    let style = setup_dir.join("STYLE.md");
    std::fs::write(&method, FIXTURE_CANONICAL_METHOD).unwrap();
    std::fs::write(&style, FIXTURE_CANONICAL_STYLE).unwrap();
    (method, style)
}

/// Fixture (a): greenfield — no CLAUDE.md, no .orbit/, no legacy. The
/// verb creates both canonicals + CLAUDE.md with both imports.
pub fn populate_setup_files_greenfield(_project_root: &std::path::Path) {
    // Nothing to write — greenfield is the absence of CLAUDE.md and .orbit/.
}

/// Fixture (b/c): CLAUDE.md exists and carries the three legacy markers.
pub fn populate_setup_files_legacy(project_root: &std::path::Path) {
    std::fs::write(
        project_root.join("CLAUDE.md"),
        "## Workflow (orbit)\nlegacy text\n\n## Orbit vocabulary\nmore legacy\n\n## Current Sprint\nstill legacy\n\n## Keep this section\nthis stays\n",
    )
    .unwrap();
}

/// Fixture (d/e): CLAUDE.md exists clean; .orbit/METHOD.md exists but
/// differs from canonical.
pub fn populate_setup_files_method_drift(project_root: &std::path::Path) {
    std::fs::write(project_root.join("CLAUDE.md"), "# CLAUDE\n\n@.orbit/METHOD.md\n\n@.orbit/STYLE.md\n").unwrap();
    let orbit = project_root.join(".orbit");
    std::fs::create_dir_all(&orbit).unwrap();
    std::fs::write(orbit.join("METHOD.md"), "# Local edited METHOD\n").unwrap();
    std::fs::write(orbit.join("STYLE.md"), FIXTURE_CANONICAL_STYLE).unwrap();
}

/// Fixture (f/g): CLAUDE.md exists clean; .orbit/STYLE.md exists but
/// differs from canonical.
pub fn populate_setup_files_style_drift(project_root: &std::path::Path) {
    std::fs::write(project_root.join("CLAUDE.md"), "# CLAUDE\n\n@.orbit/METHOD.md\n\n@.orbit/STYLE.md\n").unwrap();
    let orbit = project_root.join(".orbit");
    std::fs::create_dir_all(&orbit).unwrap();
    std::fs::write(orbit.join("METHOD.md"), FIXTURE_CANONICAL_METHOD).unwrap();
    std::fs::write(orbit.join("STYLE.md"), "# Local edited STYLE\n").unwrap();
}

pub fn expected_envelope_for_setup_files_greenfield() -> String {
    use orbit_state_core::{envelope_ok_string, FileAction, SetupFilesResult, VerbResponse};
    let response = VerbResponse::SetupFiles(SetupFilesResult {
        legacy_migrated: false,
        method_md_action: FileAction::Created,
        style_md_action: FileAction::Created,
        method_import_added: true,
        style_import_added: true,
        claude_md_created: true,
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_setup_files_legacy_migrate() -> String {
    use orbit_state_core::{envelope_ok_string, FileAction, SetupFilesResult, VerbResponse};
    let response = VerbResponse::SetupFiles(SetupFilesResult {
        legacy_migrated: true,
        method_md_action: FileAction::Overwritten,
        style_md_action: FileAction::Overwritten,
        method_import_added: true,
        style_import_added: true,
        claude_md_created: false,
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_setup_files_legacy_refuse() -> String {
    use orbit_state_core::{envelope_err_string, Error};
    let err = Error::conflict(
        "setup.files",
        "legacy CLAUDE.md blocks present; legacy_action=refuse halts setup — no files copied, no @-imports added",
    );
    envelope_err_string(&err)
}

pub fn expected_envelope_for_setup_files_method_overwrite() -> String {
    use orbit_state_core::{envelope_ok_string, FileAction, SetupFilesResult, VerbResponse};
    let response = VerbResponse::SetupFiles(SetupFilesResult {
        legacy_migrated: false,
        method_md_action: FileAction::Overwritten,
        style_md_action: FileAction::Identical,
        method_import_added: false,
        style_import_added: false,
        claude_md_created: false,
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_setup_files_method_keep() -> String {
    use orbit_state_core::{envelope_ok_string, FileAction, SetupFilesResult, VerbResponse};
    let response = VerbResponse::SetupFiles(SetupFilesResult {
        legacy_migrated: false,
        method_md_action: FileAction::KeptDrift,
        style_md_action: FileAction::Identical,
        method_import_added: false,
        style_import_added: false,
        claude_md_created: false,
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_setup_files_style_overwrite() -> String {
    use orbit_state_core::{envelope_ok_string, FileAction, SetupFilesResult, VerbResponse};
    let response = VerbResponse::SetupFiles(SetupFilesResult {
        legacy_migrated: false,
        method_md_action: FileAction::Identical,
        style_md_action: FileAction::Overwritten,
        method_import_added: false,
        style_import_added: false,
        claude_md_created: false,
    });
    envelope_ok_string(&response).expect("infallible")
}

pub fn expected_envelope_for_setup_files_style_keep() -> String {
    use orbit_state_core::{envelope_ok_string, FileAction, SetupFilesResult, VerbResponse};
    let response = VerbResponse::SetupFiles(SetupFilesResult {
        legacy_migrated: false,
        method_md_action: FileAction::Identical,
        style_md_action: FileAction::KeptDrift,
        method_import_added: false,
        style_import_added: false,
        claude_md_created: false,
    });
    envelope_ok_string(&response).expect("infallible")
}
