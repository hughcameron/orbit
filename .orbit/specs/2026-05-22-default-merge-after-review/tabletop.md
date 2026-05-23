# Tabletop — Default-merge after review-pr APPROVE

**Date:** 2026-05-22
**Facilitator + domain expert:** Hugh Cameron
**Scribe + driver:** Claude (Opus 4.7)
**Cards in scope:** 0014-default-merge-after-review
**Methodology:** Card 0019 — 10-question methodology; choice 0017 — output is contract, not solution
**Output spec:** .orbit/specs/2026-05-22-default-merge-after-review/spec.yaml

---

## Goal (Q1)

At `/orb:drive` **full autonomy**, an APPROVE verdict from the forked review-pr triggers an automatic merge with notification. `guided` and `supervised` retain today's author-confirms-merge behaviour. REQUEST_CHANGES and BLOCK route through existing NO-GO paths unchanged.

Narrowed from card 14's stated goal — drive-only and full-autonomy-only. Merge mechanism (squash vs merge-commit vs rebase, immediate vs `--auto` queue) deferred to Implementation Notes.

## Values (Q2)

**Load-bearing value: full-autonomy throughput.** Every step-by-step author intervention taxes higher-order thinking; the merge click is the most ceremonial of those taxes. Throughput is the spine; the other candidate values fall out of it:

- *Trust the substantive review* — mechanism that lets throughput land safely (the forked cold-fork IS the gate).
- *Visibility without blocking* — the shape throughput preserves (author still sees outcomes via notification).
- *Reversibility / override* — the safety floor under trust (`git revert` is always available; pre-merge override would reinstate the bottleneck).

## Trade-offs (Q3)

The simplest cut that holds throughput:
- Full autonomy on APPROVE → `gh pr merge --auto` (queue-merge handles branch protection + CI flakes naturally; defers to "when checks pass").
- Notification surfaces PR url + verdict + merge state (queued / merged / deferred) regardless of merge success.
- No pre-merge hold window — that would just be guided autonomy with extra steps.
- Override after the fact = `git revert`.

| Trade-off | Classification | Reasoning |
|---|---|---|
| Bad code lands faster (no second-look at merge) | expensive-but-worth-it | The whole bet is that the review IS the gate; second-look IS the ceremony being removed |
| Merges happen while author offline (heartbeat-driven) | expensive-but-worth-it | This is the point; notification surfaces the merge cleanly on next look |
| Lost ability to triage at merge time | acceptable | Defer-worthy work should be caught in tabletop, not at merge |
| Branch protection / required-checks / CI flakes | engineering hygiene | `--auto` queue handles by deferring merge to checks-passed |
| Confidence inflation in cold-fork reviewer | expensive-but-worth-it | Observable as drift; surface in spec/handover for first N drives |
| Notification noise (N merges = N notifications) | acceptable | Five clean merges is the desired state; noise is texture of throughput |

The cut almost reached for and rejected: pre-merge hold window. It looks safer but reintroduces the bottleneck. If you trust the review, the window is dead weight; if you don't, you shouldn't be at full.

## Halt conditions (Q4)

**Zero halt conditions.** Deliberate. A halt on merge-failure would reinstate the operator-bottleneck for the failure case — opposite of the Q2 throughput value.

Failure modes that *would* be halts under a different value pick are downgraded to **graceful degradation** here: drive logs the failure, closes the spec with a `merge deferred — manual action` note, surfaces the PR url in the close-comment, exits. The code is committed, the PR is open, the review passed; only the convenience layer dropped off.

Engineering-hygiene failure modes (pin as ACs, not halts):

1. Sequencing — merge runs only after commit-2 (card updates) is pushed. `[imagined]`
2. `gh pr merge --auto` non-zero exit → graceful degradation per above. `[imagined]`
3. Notification fires regardless of merge outcome — PR url + verdict + merge state. `[imagined]`
4. PR draft check before merge — `gh pr ready` if draft; at full autonomy a PR shouldn't be draft anyway. `[imagined]`
5. Resume after crash — drive recovery detects "PR exists, merge not attempted" from drive.yaml + `gh pr view --json mergeStateStatus`; re-attempts without duplicate PR. `[imagined; mirrors today's PR-already-exists recovery]`
6. Confidence-inflation observation window — first 3–5 auto-merged drives carry a note capturing whether the author would have intervened. Not a gate. `[imagined]`

All `[imagined]` because no prior drive has done auto-merge. The mirror-image substrate is today's `gh pr create` step in drive completion — its failure modes apply by analogy.

## Escalation triggers (Q7)

**Zero runtime escalations.** Same reasoning as zero halts — a runtime escalation reinstates the operator-bottleneck for a class of failure. The observation-window (Q4 #6) handles the legitimate "did we get this right?" question without interrupting the drive.

## Kill conditions (Q10)

- **K1 — Forked review trust claim.** If, within the first 5 auto-merged drives, ≥2 land code the author would have caught at the merge-time second-look, the cold-fork reviewer trust is dead.
  - *Pivot:* revert to today's manual-merge behaviour; tighten the cold-fork reviewer prompt (card 0007 territory); or invest in the pre-merge hold window (lateral held in reserve).

- **K2 — `--auto` queue gracefully-handles-edge-cases claim.** If branch protection on `main` makes `--auto` fail in >30% of drives, the graceful-degradation cut doesn't hold — every drive falls back to manual merge, throughput gain evaporates.
  - *Pivot:* synchronous immediate `gh pr merge` instead of `--auto`; or relax branch protection on `main`; or accept that branch-protected repos require a different shape (downstream-project posture, not orbit's).

- **K3 — Notification-without-bottleneck claim.** If the notification surface ends up requiring author acknowledgment to clear (push notification with required action), it becomes the new ceremony.
  - *Pivot:* simpler notification (close-comment line + PR url, nothing else); accept that the author finds out when they next look at the session.

## Laterals (Q5 — named, not picked)

Held in reserve:

- **Pre-merge hold window** (5-min cancellable). Held as K1's pivot path. Today: rejected — reintroduces the bottleneck.
- **Author-gated push notification at merge time.** Held as K1's softer pivot. Today: rejected — extra round-trip without proportionate safety benefit given Q2.
- **Synchronous immediate `gh pr merge`** (no `--auto`). Held as K2's pivot. Today: rejected — breaks on branch protection / CI not yet green.

Rejected outright (not in reserve):

- **Auto-merge from standalone `/orb:review-pr` too.** Scope creep relative to Q1 narrowing.
- **Branch-strategy change** (feature-branch auto-rebase, etc.). Out of scope.
- **GitHub Auto-merge via API only (no `gh` CLI).** Implementation choice; routed to Implementation Notes.

## Adjacent code (Q8 — layer-level)

Layers touched:

- `plugins/orb/skills/drive/SKILL.md` §Completion (L535+) — insert merge step between commit-2-push and `spec.close`.
- `plugins/orb/skills/drive/SKILL.md` autonomy table (L26) — update `full` mode description; remove "pauses only for PR merge" line.
- `plugins/orb/skills/drive/SKILL.md` §NO-GO Handling — unchanged.
- Notification surface — likely just augment today's close-comment with PR url + merge state; no new notification infra unless author has a preference.

Layers explicitly NOT touched:

- `plugins/orb/skills/review-pr/SKILL.md` — review-pr's verdict shape is the input, not the work.
- `plugins/orb/skills/drive/SKILL.md` §1.6 budget / §NO-GO — only the success path changes.
- `cli/src/` (orbit CLI Rust crate) — this is SKILL.md prose; no Rust changes.
- `.orbit/cards/0007-drive-forked-reviews.yaml` — upstream dependency, unchanged.
- `spec.close` logic — unchanged; merge fires before `orbit spec close` runs.

## Hot-wash

**Recurred:**
- "Implementation-shaped" tag came up across Q1, Q4, Q8. Right answer each time was to route to Implementation Notes — the role-line discipline held.

**Surprised:**
- Zero halts + zero escalations as the *correct* shape for a work item that touches `main`. First instinct was safety gates; the Q2 value reframes them as the bottleneck being removed.
- The graceful-degradation pattern collapsed five candidate failure modes into a single AC line — a stronger value pick simplifies the contract dramatically.

**Friction:**
- Q1 fired AUQ when the skill specifies prose-only. Author returned-to-prose by reframing the question (asking the "why"). Pattern for next time: surface Q1 forks in prose with a pick, never AUQ.
- Mid-session naming question (`guided` vs `supervised`) was tempting to chase. Right move was to file as a memo against card 0005 and keep going.

**Meta-patterns for future tabletops:**
- Once Q2 (load-bearing value) is genuinely settled, Q3–Q4 become mechanical — the value picks the trade-off cut, which picks the failure-mode classification. The session crisp-up after Q2 was striking.
- "Happy for you to proceed from here" after Q4 was an authority-transfer signal — Q5–Q10 are derivative once values and trade-offs are locked, so the author's role can taper.

## Implementation notes (means-level leads, not contract)

- Merge mechanism: `gh pr merge --auto --squash` is the likely default — squash matches orbit's recent merge style. Confirm against `gh pr view` of recent merges; consider `--merge` if the repo convention is merge-commits (recent log shows `Merge pull request #28`).
- `--auto` requires "Allow auto-merge" enabled in the GitHub repo settings; verify enabled before shipping.
- Notification surface: simplest shape is to extend the existing close-comment + drive.yaml.stage=complete payload with PR url + merge state. Avoid building new notification infra in this spec.
- Resume-after-crash: drive.yaml + `gh pr view --json number,mergeStateStatus,state` is sufficient state; no new sidecar needed.
- Recommended `ac_type` per candidate AC:
  - SC-1 (full-autonomy APPROVE invokes merge) → `code`
  - SC-2 (guided/supervised preserve four-option prompt) → `code`
  - SC-3 (non-APPROVE doesn't merge) → `code`
  - SC-4 (notification fires regardless of merge outcome) → `code`
  - SC-5 (graceful degradation on merge failure) → `code`
  - SC-6 (resume detects PR-exists-merge-not-attempted) → `code`
  - Confidence-inflation observation window (first N drives) → `observation` (deferrable)
