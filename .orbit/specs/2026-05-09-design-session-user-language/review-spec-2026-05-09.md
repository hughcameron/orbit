# Spec Review

**Date:** 2026-05-09
**Reviewer:** Context-separated agent (fresh session)
**Spec:** 2026-05-09-design-session-user-language
**Verdict:** APPROVE

---

## Review Depth

| Pass | Triggered by | Findings |
|------|-------------|----------|
| 1 — Structural scan | always | 2 (both LOW) |
| 2 — Assumption & failure | not triggered (no MEDIUM+ findings, no content signals) | — |
| 3 — Adversarial | not triggered | — |

## Findings

### [LOW] ac-05 is a roll-up of ac-01..ac-04
**Category:** test-gap
**Pass:** 1
**Description:** ac-05 reasserts that the wires from ac-01, ac-02, ac-03, and ac-04 must land in the named skill files. It's a meta-AC that passes only when the four it references already pass. The AC's own wording ("without these four edits the card stays aspirational") concedes the redundancy.
**Evidence:** Compare ac-05 ("/orb:design SKILL.md encodes the pre-flight design-space gate (ac-01), names the implementation-question filter as a question-generation test (ac-03), and references the user-voice paragraph slot in its interview output instructions (ac-04). /orb:spec SKILL.md accepts short design notes alongside interview.md (ac-02).") against ac-01..ac-04 verbatim — the only thing ac-05 adds is the framing that all four landed in the named files.
**Recommendation:** Keep. ac-05 is a "framework wires landed" gate that protects against ac-01..ac-04 being marked done while the skill files stay aspirational. The card explicitly carries the equivalent gate scenario ("Wired into the framework"). Cost is one redundant tick, value is one explicit cross-check before drive marks the spec done.

### [LOW] ac-07 mode-switch trigger leaves rejection-detection unspecified
**Category:** assumption
**Pass:** 1
**Description:** ac-07 requires the skill to detect "the author has rejected questions as implementation-shaped twice in a session" and treat the third reformulation as a mode-switch signal. The detection mechanism — keyword match, AskUserQuestion choice, free-text classification — is left to the implementing agent. The AC is verifiable as written (the skill text either documents the trigger or it doesn't) but the runtime behaviour depends on a detection heuristic the spec doesn't pin.
**Evidence:** ac-07 prose: "when the author has rejected questions as implementation-shaped twice in a session, the agent treats the third reformulation as a mode-switch signal" — no statement of how rejection is detected.
**Recommendation:** No spec change required. Flag as an implementation note for /orb:implement — the implementing agent should pin a concrete detection rule (e.g. explicit author phrase, AskUserQuestion's "this is implementation" choice) when editing the skill. Fine to leave the rule choice to implementation; just call it out so it doesn't get silently invented.

---

## Honest Assessment

This spec is ready to implement. It is documentation-only work against two skill files (`plugins/orb/skills/design/SKILL.md`, `plugins/orb/skills/spec/SKILL.md`), the AC list maps cleanly to the card's scenarios, and every gate AC has a substantive description naming the file and section to edit. No content signals (no infra, security, migrations, cross-system boundaries) — no need to deepen.

Biggest risk is ac-07's unspecified rejection-detection heuristic. It's a low risk because the AC verifies the skill *documents* the trigger, not the agent's runtime accuracy at detecting rejection. The implementing agent should pick a concrete detection rule when amending /orb:design, and ac-07's verification will pass on the prose, not on observed behaviour.

ac-05 is intentional belt-and-braces. Living with the redundancy is cheaper than risking ac-01..ac-04 being marked done while the skill files stay aspirational.
